use std::collections::HashMap;

use crate::git::GitInfo;
use crate::types::{AgentEntry, Config, StdinData, TaskItem, TaskStatus, TodoItem, ToolEntry, TranscriptData};
use crate::types::UsageInfo;

fn context_percentage(data: &StdinData) -> Option<u8> {
    data.context_window.as_ref()?.used_percentage
}

fn color_for_percentage(pct: u8, config: &Config) -> &'static str {
    let warning = config.context_warning.unwrap_or(70);
    let critical = config.context_critical.unwrap_or(85);
    if pct > critical {
        "\x1b[31m"
    } else if pct >= warning {
        "\x1b[33m"
    } else {
        "\x1b[32m"
    }
}

const RESET: &str = "\x1b[0m";

const BRIGHT: &str = "\x1b[1;37m";
const DIM: &str = "\x1b[2m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const USAGE_HIGH: f64 = 80.0;

fn colorize(text: String, color: &str, enabled: bool) -> String {
    if enabled {
        format!("{color}{text}{RESET}")
    } else {
        text
    }
}

fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        return "<1m".to_string();
    }
    let minutes = seconds / 60;
    let hours = minutes / 60;
    let days = hours / 24;
    let remaining_hours = hours % 24;
    let remaining_minutes = minutes % 60;
    if days > 0 {
        format!("{days}d {remaining_hours}h")
    } else if hours > 0 {
        format!("{hours}h {remaining_minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn format_agent_duration(seconds: i64) -> String {
    if seconds < 0 {
        return "0s".to_string();
    }
    let minutes = seconds / 60;
    let secs = seconds % 60;
    if minutes == 0 {
        format!("{secs}s")
    } else {
        format!("{minutes}m {secs}s")
    }
}

fn render_agents(agents: &HashMap<String, AgentEntry>, config: &Config) -> Vec<String> {
    let now = chrono::Utc::now().timestamp();
    let colors = config.colors();
    agents
        .values()
        .filter(|a| !a.completed)
        .map(|a| {
            let name = a
                .subagent_type
                .as_deref()
                .unwrap_or("agent");
            let model_part = a
                .model
                .as_ref()
                .map(|m| format!("[{m}]"))
                .unwrap_or_default();
            let dur = a
                .start_time
                .map(|t| format_agent_duration(now - t))
                .unwrap_or_default();
            let label = format!("{name}{model_part} {dur}").trim().to_string();
            colorize(label, YELLOW, colors)
        })
        .collect()
}

fn render_usage(usage: Option<&UsageInfo>, config: &Config) -> Vec<String> {
    let Some(info) = usage else {
        return vec![];
    };
    let colors = config.colors();
    let items = [
        ("hourly", info.usage_5h, info.reset_5h),
        ("weekly", info.usage_7d, info.reset_7d),
    ];
    items
        .into_iter()
        .filter_map(|(label, pct_opt, reset_opt)| {
            let pct = pct_opt?;
            let pct_int = pct.round() as u8;
            let color = if pct > USAGE_HIGH { MAGENTA } else { BLUE };
            let reset_part = reset_opt
                .map(|s| format!(" ({} reset)", format_duration(s)))
                .unwrap_or_default();
            Some(colorize(format!("{label} {pct_int}%{reset_part}"), color, colors))
        })
        .collect()
}

fn render_tasks(todos: &[TodoItem], tasks: &HashMap<String, TaskItem>, config: &Config) -> Option<String> {
    let todo_total = todos.len();
    let todo_completed = todos.iter().filter(|t| t.completed).count();

    let task_total = tasks.len();
    let task_completed = tasks.values().filter(|t| t.status == TaskStatus::Completed).count();

    let total = todo_total + task_total;
    let completed = todo_completed + task_completed;

    if total == 0 {
        return None;
    }

    let label = format!("tasks {completed}/{total}");
    let color = if completed == total { GREEN } else { DIM };
    Some(colorize(label, color, config.colors()))
}

fn render_activity_line(tools: &HashMap<String, ToolEntry>, agents: &HashMap<String, AgentEntry>, config: &Config) -> Option<String> {
    let running: Vec<&ToolEntry> = tools.values().filter(|t| !t.completed).collect();
    let completed: Vec<&ToolEntry> = tools.values().filter(|t| t.completed).collect();
    let has_running_agents = agents.values().any(|a| !a.completed);

    if running.is_empty() && completed.is_empty() && !has_running_agents {
        return None;
    }

    let sep = config.separator();
    let colors = config.colors();
    let mut parts: Vec<String> = Vec::new();

    if let Some(tool) = running.last() {
        let label = match &tool.target {
            Some(t) => format!("{} {}", tool.name, t),
            None => tool.name.clone(),
        };
        parts.push(colorize(label, BRIGHT, colors));
    }

    let mut counts: Vec<(String, usize)> = Vec::new();
    {
        let mut map: HashMap<&str, usize> = HashMap::new();
        let mut order: Vec<&str> = Vec::new();
        for t in &completed {
            let n = t.name.as_str();
            let count = map.entry(n).or_insert(0);
            if *count == 0 {
                order.push(n);
            }
            *count += 1;
        }
        for name in order {
            counts.push((name.to_string(), map[name]));
        }
    }

    for (name, count) in counts.iter().rev().take(3).rev() {
        let label = if *count == 1 {
            name.clone()
        } else {
            format!("{name} x{count}")
        };
        parts.push(colorize(label, DIM, colors));
    }

    let agent_parts = render_agents(agents, config);
    parts.extend(agent_parts);

    if parts.is_empty() {
        return None;
    }

    Some(parts.join(sep))
}

pub fn render(data: &StdinData, config: &Config, transcript: &TranscriptData, git: Option<&GitInfo>, usage: Option<&UsageInfo>) -> String {
    let model_name = data
        .model
        .as_ref()
        .and_then(|m| m.display_name.as_deref())
        .unwrap_or("cstat");

    let project_name = data
        .cwd
        .as_deref()
        .map(|p| {
            let mut parts: Vec<&str> = p.rsplit('/').take(config.path_levels() as usize).collect();
            parts.reverse();
            parts.join("/")
        })
        .unwrap_or_else(|| "no data".into());

    let sep = config.separator();
    let colors = config.colors();

    let mut line1 = model_name.to_string();
    for part in render_usage(usage, config) {
        line1.push_str(&format!("{sep}{part}"));
    }

    let mut line2_parts: Vec<String> = vec![project_name];

    if let Some(gi) = git {
        let dirty = if gi.dirty { "*" } else { "" };
        line2_parts.push(colorize(format!("{}{dirty}", gi.branch), DIM, colors));
    }

    if let Some(pct) = context_percentage(data) {
        let color = color_for_percentage(pct, config);
        line2_parts.push(colorize(format!("context {pct}%"), color, colors));
    }

    let activity = render_activity_line(&transcript.tools, &transcript.agents, config);
    if let Some(a) = activity {
        line2_parts.push(a);
    }

    let tasks = render_tasks(&transcript.todos, &transcript.tasks, config);
    if let Some(t) = tasks {
        line2_parts.push(t);
    }

    let line2 = line2_parts.join(sep);
    format!("{line1}\n{line2}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentEntry, ContextWindow, Model, StdinData, TaskItem, TaskStatus, TodoItem, ToolEntry, TranscriptData};

    fn make_data(pct: Option<u8>) -> StdinData {
        StdinData {
            model: Some(Model {
                display_name: Some("Opus".into()),
            }),
            cwd: Some("/home/user/my-project".into()),
            context_window: Some(ContextWindow {
                used_percentage: pct,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn render_model_and_project() {
        let data = StdinData {
            model: Some(Model {
                display_name: Some("Opus".into()),
            }),
            cwd: Some("/home/user/my-project".into()),
            ..Default::default()
        };
        let out = render(&data, &Config::default(), &TranscriptData::default(), None, None);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "Opus");
        assert_eq!(lines[1], "my-project");
    }

    #[test]
    fn render_empty_stdin() {
        let data = StdinData::default();
        let out = render(&data, &Config::default(), &TranscriptData::default(), None, None);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "cstat");
        assert_eq!(lines[1], "no data");
    }

    #[test]
    fn render_missing_model_name() {
        let data = StdinData {
            model: Some(Model { display_name: None }),
            cwd: Some("/tmp/foo".into()),
            ..Default::default()
        };
        let out = render(&data, &Config::default(), &TranscriptData::default(), None, None);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "cstat");
        assert_eq!(lines[1], "foo");
    }

    #[test]
    fn context_green_below_70() {
        let data = make_data(Some(45));
        let cfg = Config::default();
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("\x1b[32mcontext 45%\x1b[0m"));
    }

    #[test]
    fn context_yellow_at_70() {
        let data = make_data(Some(70));
        let cfg = Config::default();
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("\x1b[33mcontext 70%\x1b[0m"));
    }

    #[test]
    fn context_yellow_at_85() {
        let data = make_data(Some(85));
        let cfg = Config::default();
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("\x1b[33mcontext 85%\x1b[0m"));
    }

    #[test]
    fn context_red_above_85() {
        let data = make_data(Some(86));
        let cfg = Config::default();
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("\x1b[31mcontext 86%\x1b[0m"));
    }

    #[test]
    fn context_no_colors() {
        let data = make_data(Some(45));
        let cfg = Config {
            colors: Some(false),
            ..Default::default()
        };
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("context 45%"));
    }

    #[test]
    fn context_missing_window() {
        let data = StdinData {
            model: Some(Model {
                display_name: Some("Opus".into()),
            }),
            cwd: Some("/home/user/my-project".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, None);
        assert!(!out.contains("context"));
    }

    #[test]
    fn context_missing_tokens() {
        let data = make_data(None);
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, None);
        assert!(!out.contains("context"));
    }

    #[test]
    fn context_zero_percentage() {
        let data = make_data(Some(0));
        let out = render(&data, &Config::default(), &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("\x1b[32mcontext 0%\x1b[0m"));
    }

    #[test]
    fn context_custom_thresholds() {
        let data = make_data(Some(55));
        let cfg = Config {
            context_warning: Some(50),
            context_critical: Some(60),
            ..Default::default()
        };
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("\x1b[33mcontext 55%\x1b[0m"));
    }

    #[test]
    fn context_integer_percentage() {
        let data = make_data(Some(33));
        let cfg = Config {
            colors: Some(false),
            ..Default::default()
        };
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("context 33%"));
    }

    #[test]
    fn path_levels_2() {
        let data = StdinData {
            model: Some(Model {
                display_name: Some("Opus".into()),
            }),
            cwd: Some("/home/user/my-project".into()),
            ..Default::default()
        };
        let cfg = Config {
            path_levels: Some(2),
            ..Default::default()
        };
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.starts_with("user/my-project"));
    }

    #[test]
    fn path_levels_3() {
        let data = StdinData {
            model: Some(Model {
                display_name: Some("Opus".into()),
            }),
            cwd: Some("/home/user/my-project".into()),
            ..Default::default()
        };
        let cfg = Config {
            path_levels: Some(3),
            ..Default::default()
        };
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.starts_with("home/user/my-project"));
    }

    #[test]
    fn custom_separator() {
        let data = make_data(Some(10));
        let cfg = Config {
            colors: Some(false),
            separator: Some(" | ".into()),
            ..Default::default()
        };
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("my-project | context 10%"));
    }

    #[test]
    fn context_double_space_separator() {
        let data = make_data(Some(10));
        let cfg = Config {
            colors: Some(false),
            ..Default::default()
        };
        let out = render(&data, &cfg, &TranscriptData::default(), None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("my-project  context"));
    }

    fn tool(name: &str, target: Option<&str>, completed: bool) -> ToolEntry {
        ToolEntry {
            name: name.to_string(),
            target: target.map(|s| s.to_string()),
            completed,
            error: false,
        }
    }

    fn no_colors_cfg() -> Config {
        Config {
            colors: Some(false),
            ..Default::default()
        }
    }

    #[test]
    fn activity_shown_on_line2() {
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Edit", Some("auth.ts"), false));
        let transcript = TranscriptData {
            tools,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("Edit auth.ts"));
    }

    #[test]
    fn activity_running_tool_without_target() {
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Glob", None, false));
        let transcript = TranscriptData {
            tools,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("Glob"));
    }

    #[test]
    fn activity_completed_tools_grouped() {
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Read", Some("a.rs"), true));
        tools.insert("t2".into(), tool("Read", Some("b.rs"), true));
        tools.insert("t3".into(), tool("Read", Some("c.rs"), true));
        tools.insert("t4".into(), tool("Grep", Some("TODO"), true));
        let transcript = TranscriptData {
            tools,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("Read x3"));
        assert!(line2.contains("Grep"));
    }

    #[test]
    fn activity_max_3_completed_groups() {
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Read", None, true));
        tools.insert("t2".into(), tool("Grep", None, true));
        tools.insert("t3".into(), tool("Edit", None, true));
        tools.insert("t4".into(), tool("Write", None, true));
        let transcript = TranscriptData {
            tools,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        let activity_part = line2.strip_prefix("p  ").unwrap_or(line2);
        let group_count = activity_part.split("  ").count();
        assert!(group_count <= 3);
    }

    #[test]
    fn activity_running_plus_completed() {
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Read", Some("a.rs"), true));
        tools.insert("t2".into(), tool("Read", Some("b.rs"), true));
        tools.insert("t3".into(), tool("Edit", Some("main.rs"), false));
        let transcript = TranscriptData {
            tools,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("Edit main.rs"));
        assert!(line2.contains("Read x2"));
    }

    #[test]
    fn activity_with_colors() {
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Edit", Some("auth.ts"), false));
        tools.insert("t2".into(), tool("Read", Some("a.rs"), true));
        let transcript = TranscriptData {
            tools,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &Config::default(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains(BRIGHT));
        assert!(line2.contains(DIM));
        assert!(line2.contains(RESET));
    }

    #[test]
    fn activity_single_completed_no_count() {
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Grep", Some("TODO"), true));
        let transcript = TranscriptData {
            tools,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("Grep"));
        assert!(!line2.contains("x1"));
    }

    #[test]
    fn activity_running_agent_with_model() {
        let mut agents = HashMap::new();
        agents.insert(
            "a1".into(),
            AgentEntry {
                subagent_type: Some("explore".into()),
                model: Some("haiku".into()),
                description: Some("find files".into()),
                start_time: Some(chrono::Utc::now().timestamp() - 135),
                completed: false,
            },
        );
        let transcript = TranscriptData {
            agents,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("explore[haiku] 2m 15s"));
    }

    #[test]
    fn activity_running_agent_without_model() {
        let mut agents = HashMap::new();
        agents.insert(
            "a1".into(),
            AgentEntry {
                subagent_type: Some("general-purpose".into()),
                model: None,
                description: None,
                start_time: Some(chrono::Utc::now().timestamp() - 45),
                completed: false,
            },
        );
        let transcript = TranscriptData {
            agents,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("general-purpose 45s"));
    }

    #[test]
    fn activity_completed_agent_hidden() {
        let mut agents = HashMap::new();
        agents.insert(
            "a1".into(),
            AgentEntry {
                subagent_type: Some("explore".into()),
                model: Some("haiku".into()),
                description: None,
                start_time: Some(chrono::Utc::now().timestamp() - 60),
                completed: true,
            },
        );
        let transcript = TranscriptData {
            agents,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        assert!(!out.contains("explore"));
    }

    #[test]
    fn activity_agent_yellow_with_colors() {
        let mut agents = HashMap::new();
        agents.insert(
            "a1".into(),
            AgentEntry {
                subagent_type: Some("explore".into()),
                model: None,
                description: None,
                start_time: Some(chrono::Utc::now().timestamp() - 10),
                completed: false,
            },
        );
        let transcript = TranscriptData {
            agents,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &Config::default(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains(YELLOW));
    }

    #[test]
    fn activity_uses_config_separator() {
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Read", None, true));
        tools.insert("t2".into(), tool("Grep", None, true));
        let transcript = TranscriptData {
            tools,
            ..Default::default()
        };
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let cfg = Config {
            colors: Some(false),
            separator: Some(" | ".into()),
            ..Default::default()
        };
        let out = render(&data, &cfg, &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains(" | "));
    }

    #[test]
    fn git_branch_shown() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let git = GitInfo { branch: "main".into(), dirty: false };
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), Some(&git), None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("main"));
        assert!(!line2.contains("git:"));
    }

    #[test]
    fn git_dirty_indicator() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let git = GitInfo { branch: "feat/x".into(), dirty: true };
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), Some(&git), None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("feat/x*"));
    }

    #[test]
    fn git_with_colors() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let git = GitInfo { branch: "main".into(), dirty: false };
        let out = render(&data, &Config::default(), &TranscriptData::default(), Some(&git), None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains(DIM));
        assert!(line2.contains("main"));
        assert!(!line2.contains("git:"));
    }

    #[test]
    fn git_omitted_when_none() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, None);
        assert!(!out.contains("main"));
    }

    #[test]
    fn git_with_context() {
        let data = make_data(Some(45));
        let git = GitInfo { branch: "dev".into(), dirty: false };
        let cfg = Config { colors: Some(false), ..Default::default() };
        let out = render(&data, &cfg, &TranscriptData::default(), Some(&git), None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("my-project"));
        assert!(line2.contains("dev"));
        assert!(line2.contains("context 45%"));
    }

    #[test]
    fn format_duration_under_minute() {
        assert_eq!(format_duration(0), "<1m");
        assert_eq!(format_duration(30), "<1m");
        assert_eq!(format_duration(59), "<1m");
    }

    #[test]
    fn format_duration_minutes() {
        assert_eq!(format_duration(60), "1m");
        assert_eq!(format_duration(120), "2m");
        assert_eq!(format_duration(3599), "59m");
    }

    #[test]
    fn format_duration_hours_and_minutes() {
        assert_eq!(format_duration(3600), "1h 0m");
        assert_eq!(format_duration(5400), "1h 30m");
        assert_eq!(format_duration(7200), "2h 0m");
    }

    #[test]
    fn tasks_shown_from_todos() {
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let transcript = TranscriptData {
            todos: vec![
                TodoItem { content: "a".into(), completed: true },
                TodoItem { content: "b".into(), completed: false },
                TodoItem { content: "c".into(), completed: true },
            ],
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("tasks 2/3"));
    }

    #[test]
    fn tasks_shown_from_task_items() {
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let mut tasks = HashMap::new();
        tasks.insert("t1".into(), TaskItem { status: TaskStatus::Completed });
        tasks.insert("t2".into(), TaskItem { status: TaskStatus::Pending });
        tasks.insert("t3".into(), TaskItem { status: TaskStatus::InProgress });
        let transcript = TranscriptData {
            tasks,
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("tasks 1/3"));
    }

    #[test]
    fn tasks_combined_todos_and_task_items() {
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let mut tasks = HashMap::new();
        tasks.insert("t1".into(), TaskItem { status: TaskStatus::Completed });
        tasks.insert("t2".into(), TaskItem { status: TaskStatus::Pending });
        let transcript = TranscriptData {
            todos: vec![
                TodoItem { content: "a".into(), completed: true },
                TodoItem { content: "b".into(), completed: false },
            ],
            tasks,
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("tasks 2/4"));
    }

    #[test]
    fn tasks_hidden_when_empty() {
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, None);
        assert!(!out.contains("tasks"));
    }

    #[test]
    fn tasks_green_when_all_completed() {
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let transcript = TranscriptData {
            todos: vec![
                TodoItem { content: "a".into(), completed: true },
                TodoItem { content: "b".into(), completed: true },
            ],
            ..Default::default()
        };
        let out = render(&data, &Config::default(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains(GREEN));
        assert!(line2.contains("tasks 2/2"));
    }

    #[test]
    fn tasks_dim_when_not_all_completed() {
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let transcript = TranscriptData {
            todos: vec![
                TodoItem { content: "a".into(), completed: true },
                TodoItem { content: "b".into(), completed: false },
            ],
            ..Default::default()
        };
        let out = render(&data, &Config::default(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains(DIM));
        assert!(line2.contains("tasks 1/2"));
    }

    #[test]
    fn tasks_alongside_tools() {
        let data = StdinData {
            cwd: Some("/tmp/p".into()),
            ..Default::default()
        };
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Read", Some("a.rs"), true));
        let transcript = TranscriptData {
            tools,
            todos: vec![
                TodoItem { content: "a".into(), completed: true },
                TodoItem { content: "b".into(), completed: false },
            ],
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("Read"));
        assert!(line2.contains("tasks 1/2"));
    }

    #[test]
    fn usage_hourly_and_weekly_shown() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let usage = UsageInfo {
            usage_5h: Some(25.0),
            usage_7d: Some(60.0),
            reset_5h: Some(5400),
            reset_7d: None,
        };
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, Some(&usage));
        let line1 = out.lines().next().unwrap();
        assert!(line1.contains("hourly 25% (1h 30m reset)"));
        assert!(line1.contains("weekly 60%"));
    }

    #[test]
    fn usage_hourly_only() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let usage = UsageInfo {
            usage_5h: Some(10.0),
            usage_7d: None,
            reset_5h: None,
            reset_7d: None,
        };
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, Some(&usage));
        let line1 = out.lines().next().unwrap();
        assert!(line1.contains("hourly 10%"));
        assert!(!line1.contains("weekly"));
    }

    #[test]
    fn usage_weekly_only() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let usage = UsageInfo {
            usage_5h: None,
            usage_7d: Some(40.0),
            reset_5h: None,
            reset_7d: None,
        };
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, Some(&usage));
        let line1 = out.lines().next().unwrap();
        assert!(!line1.contains("hourly"));
        assert!(line1.contains("weekly 40%"));
    }

    #[test]
    fn usage_omitted_when_none() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, None);
        let line1 = out.lines().next().unwrap();
        assert!(!out.contains("hourly"));
        assert!(!out.contains("weekly"));
        assert_eq!(line1, "Opus");
    }

    #[test]
    fn usage_blue_color_below_80() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let usage = UsageInfo {
            usage_5h: Some(25.0),
            usage_7d: Some(60.0),
            reset_5h: None,
            reset_7d: None,
        };
        let out = render(&data, &Config::default(), &TranscriptData::default(), None, Some(&usage));
        assert!(out.contains(BLUE));
        assert!(!out.contains(MAGENTA));
    }

    #[test]
    fn usage_magenta_color_above_80() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let usage = UsageInfo {
            usage_5h: Some(85.0),
            usage_7d: Some(90.0),
            reset_5h: None,
            reset_7d: None,
        };
        let out = render(&data, &Config::default(), &TranscriptData::default(), None, Some(&usage));
        assert!(out.contains(MAGENTA));
    }

    #[test]
    fn usage_on_line1_context_on_line2() {
        let data = make_data(Some(45));
        let usage = UsageInfo {
            usage_5h: Some(25.0),
            usage_7d: Some(60.0),
            reset_5h: Some(5400),
            reset_7d: None,
        };
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, Some(&usage));
        let line1 = out.lines().next().unwrap();
        let line2 = out.lines().nth(1).unwrap();
        assert!(line1.contains("hourly 25%"));
        assert!(line1.contains("weekly 60%"));
        assert!(!line1.contains("context"));
        assert!(line2.contains("context 45%"));
    }

    fn full_transcript() -> TranscriptData {
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Read", Some("a.rs"), true));
        tools.insert("t2".into(), tool("Edit", Some("b.rs"), false));
        let mut agents = HashMap::new();
        agents.insert(
            "a1".into(),
            AgentEntry {
                subagent_type: Some("explore".into()),
                model: Some("haiku".into()),
                description: None,
                start_time: Some(chrono::Utc::now().timestamp() - 30),
                completed: false,
            },
        );
        let mut tasks = HashMap::new();
        tasks.insert("tk1".into(), TaskItem { status: TaskStatus::Completed });
        tasks.insert("tk2".into(), TaskItem { status: TaskStatus::Pending });
        TranscriptData {
            tools,
            agents,
            todos: vec![
                TodoItem { content: "x".into(), completed: true },
                TodoItem { content: "y".into(), completed: false },
            ],
            tasks,
        }
    }

    fn full_usage() -> UsageInfo {
        UsageInfo {
            usage_5h: Some(25.0),
            usage_7d: Some(60.0),
            reset_5h: Some(3600),
            reset_7d: Some(259200),
        }
    }

    #[test]
    fn all_data_present_no_colors() {
        let data = make_data(Some(45));
        let git = GitInfo { branch: "main".into(), dirty: true };
        let usage = full_usage();
        let transcript = full_transcript();
        let out = render(&data, &no_colors_cfg(), &transcript, Some(&git), Some(&usage));
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("Opus"));
        assert!(lines[0].contains("hourly 25%"));
        assert!(lines[0].contains("weekly 60%"));
        assert!(lines[1].contains("my-project"));
        assert!(lines[1].contains("main*"));
        assert!(lines[1].contains("context 45%"));
        assert!(lines[1].contains("Edit b.rs"));
        assert!(lines[1].contains("Read"));
        assert!(lines[1].contains("explore[haiku]"));
        assert!(lines[1].contains("tasks 2/4"));
    }

    #[test]
    fn all_data_present_with_colors() {
        let data = make_data(Some(45));
        let git = GitInfo { branch: "main".into(), dirty: false };
        let usage = full_usage();
        let transcript = full_transcript();
        let out = render(&data, &Config::default(), &transcript, Some(&git), Some(&usage));
        assert!(out.contains(GREEN));
        assert!(out.contains(BLUE));
        assert!(out.contains(DIM));
        assert!(out.contains(BRIGHT));
        assert!(out.contains(YELLOW));
        assert!(out.contains(RESET));
    }

    #[test]
    fn all_data_present_custom_separator() {
        let data = make_data(Some(45));
        let git = GitInfo { branch: "main".into(), dirty: false };
        let usage = full_usage();
        let transcript = full_transcript();
        let cfg = Config {
            colors: Some(false),
            separator: Some(" | ".into()),
            ..Default::default()
        };
        let out = render(&data, &cfg, &transcript, Some(&git), Some(&usage));
        let line1 = out.lines().next().unwrap();
        let line2 = out.lines().nth(1).unwrap();
        assert!(line1.contains(" | hourly 25%"));
        assert!(line2.contains(" | context 45%"));
    }

    #[test]
    fn missing_git_only() {
        let data = make_data(Some(45));
        let usage = full_usage();
        let transcript = full_transcript();
        let out = render(&data, &no_colors_cfg(), &transcript, None, Some(&usage));
        assert!(!out.contains("main"));
        assert!(out.contains("context 45%"));
        assert!(out.contains("hourly 25%"));
    }

    #[test]
    fn missing_context_only() {
        let data = StdinData {
            model: Some(Model { display_name: Some("Opus".into()) }),
            cwd: Some("/home/user/my-project".into()),
            ..Default::default()
        };
        let git = GitInfo { branch: "main".into(), dirty: false };
        let usage = full_usage();
        let transcript = full_transcript();
        let out = render(&data, &no_colors_cfg(), &transcript, Some(&git), Some(&usage));
        assert!(!out.contains("context"));
        assert!(out.contains("main"));
        assert!(out.contains("hourly 25%"));
    }

    #[test]
    fn missing_usage_only() {
        let data = make_data(Some(45));
        let git = GitInfo { branch: "main".into(), dirty: false };
        let transcript = full_transcript();
        let out = render(&data, &no_colors_cfg(), &transcript, Some(&git), None);
        assert!(!out.contains("hourly"));
        assert!(!out.contains("weekly"));
        assert!(out.contains("context 45%"));
        assert!(out.contains("main"));
    }

    #[test]
    fn missing_activity_only() {
        let data = make_data(Some(45));
        let git = GitInfo { branch: "main".into(), dirty: false };
        let usage = full_usage();
        let transcript = TranscriptData {
            todos: vec![
                TodoItem { content: "x".into(), completed: true },
                TodoItem { content: "y".into(), completed: false },
            ],
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, Some(&git), Some(&usage));
        let line2 = out.lines().nth(1).unwrap();
        assert!(line2.contains("tasks 1/2"));
    }

    #[test]
    fn missing_tasks_only() {
        let data = make_data(Some(45));
        let git = GitInfo { branch: "main".into(), dirty: false };
        let usage = full_usage();
        let mut tools = HashMap::new();
        tools.insert("t1".into(), tool("Read", Some("a.rs"), true));
        let transcript = TranscriptData {
            tools,
            ..Default::default()
        };
        let out = render(&data, &no_colors_cfg(), &transcript, Some(&git), Some(&usage));
        let line2 = out.lines().nth(1).unwrap();
        assert!(!line2.contains("tasks"));
    }

    #[test]
    fn all_blocks_missing() {
        let data = StdinData::default();
        let out = render(&data, &Config::default(), &TranscriptData::default(), None, None);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0], "cstat");
        assert_eq!(lines[1], "no data");
    }

    #[test]
    fn all_blocks_missing_no_colors() {
        let data = StdinData::default();
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, None);
        assert!(!out.contains('\x1b'));
    }

    #[test]
    fn no_ansi_codes_when_colors_off() {
        let data = make_data(Some(45));
        let git = GitInfo { branch: "main".into(), dirty: true };
        let usage = full_usage();
        let transcript = full_transcript();
        let out = render(&data, &no_colors_cfg(), &transcript, Some(&git), Some(&usage));
        assert!(!out.contains('\x1b'));
    }

    #[test]
    fn output_never_empty() {
        let data = StdinData::default();
        let out = render(&data, &Config::default(), &TranscriptData::default(), None, None);
        assert!(!out.is_empty());
    }

    #[test]
    fn output_no_trailing_newline() {
        let data = make_data(Some(45));
        let transcript = full_transcript();
        let out = render(&data, &no_colors_cfg(), &transcript, None, None);
        assert!(!out.ends_with('\n'));
    }

    #[test]
    fn first_line_never_empty() {
        let data = StdinData::default();
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, None);
        let first = out.lines().next().unwrap();
        assert!(!first.is_empty());
    }

    #[test]
    fn always_two_lines() {
        let data = StdinData::default();
        let out = render(&data, &no_colors_cfg(), &TranscriptData::default(), None, None);
        assert_eq!(out.lines().count(), 2);
    }

    #[test]
    fn format_agent_duration_zero() {
        assert_eq!(format_agent_duration(0), "0s");
    }

    #[test]
    fn format_agent_duration_negative() {
        assert_eq!(format_agent_duration(-5), "0s");
    }

    #[test]
    fn format_agent_duration_seconds() {
        assert_eq!(format_agent_duration(45), "45s");
    }

    #[test]
    fn format_agent_duration_minutes_and_seconds() {
        assert_eq!(format_agent_duration(135), "2m 15s");
    }

    #[test]
    fn context_percentage_none_when_no_context_window() {
        let data = StdinData::default();
        assert!(context_percentage(&data).is_none());
    }

    #[test]
    fn context_percentage_returns_value() {
        let data = make_data(Some(42));
        assert_eq!(context_percentage(&data), Some(42));
    }

    #[test]
    fn render_usage_empty_when_none() {
        let parts = render_usage(None, &Config::default());
        assert!(parts.is_empty());
    }

    #[test]
    fn render_tasks_none_when_empty() {
        assert!(render_tasks(&[], &HashMap::new(), &Config::default()).is_none());
    }

    #[test]
    fn activity_line_none_when_empty() {
        assert!(render_activity_line(&HashMap::new(), &HashMap::new(), &Config::default()).is_none());
    }

    #[test]
    fn activity_line_with_only_completed_agents_is_none() {
        let mut agents = HashMap::new();
        agents.insert(
            "a1".into(),
            AgentEntry {
                subagent_type: Some("explore".into()),
                model: None,
                description: None,
                start_time: Some(chrono::Utc::now().timestamp() - 30),
                completed: true,
            },
        );
        assert!(render_activity_line(&HashMap::new(), &agents, &Config::default()).is_none());
    }
}
