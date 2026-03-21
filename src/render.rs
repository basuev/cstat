use std::collections::HashMap;

use crate::types::{Config, StdinData, ToolEntry, TranscriptData};

fn context_percentage(data: &StdinData) -> Option<u8> {
    let cw = data.context_window.as_ref()?;
    let size = cw.context_window_size?;
    let tokens = cw.current_usage.as_ref()?.input_tokens?;
    if size == 0 {
        return None;
    }
    Some(((tokens as f64 / size as f64) * 100.0) as u8)
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

fn render_activity_line(tools: &HashMap<String, ToolEntry>, config: &Config) -> Option<String> {
    let running: Vec<&ToolEntry> = tools.values().filter(|t| !t.completed).collect();
    let completed: Vec<&ToolEntry> = tools.values().filter(|t| t.completed).collect();

    if running.is_empty() && completed.is_empty() {
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
        if colors {
            parts.push(format!("{BRIGHT}{label}{RESET}"));
        } else {
            parts.push(label);
        }
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
        if colors {
            parts.push(format!("{DIM}{label}{RESET}"));
        } else {
            parts.push(label);
        }
    }

    if parts.is_empty() {
        return None;
    }

    Some(parts.join(sep))
}

pub fn render(data: &StdinData, config: &Config, transcript: &TranscriptData) -> String {
    let model_name = data
        .model
        .as_ref()
        .and_then(|m| m.display_name.as_deref())
        .unwrap_or("cstat");

    let project_name = data
        .cwd
        .as_deref()
        .map(|p| {
            let parts: Vec<&str> = p.rsplit('/').take(config.path_levels() as usize).collect();
            let mut parts = parts;
            parts.reverse();
            parts.join("/")
        })
        .unwrap_or_else(|| "no data".into());

    let sep = config.separator();
    let colors = config.colors();

    let mut line = format!("[{model_name}] {project_name}");

    if let Some(pct) = context_percentage(data) {
        if colors {
            let color = color_for_percentage(pct, config);
            line.push_str(&format!("{sep}{color}ctx {pct}%{RESET}"));
        } else {
            line.push_str(&format!("{sep}ctx {pct}%"));
        }
    }

    if let Some(activity) = render_activity_line(&transcript.tools, config) {
        line.push('\n');
        line.push_str(&activity);
    }

    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContextWindow, CurrentUsage, Model, StdinData, ToolEntry, TranscriptData};

    fn make_data(tokens: Option<u64>, window_size: Option<u64>) -> StdinData {
        StdinData {
            model: Some(Model {
                display_name: Some("Opus".into()),
            }),
            cwd: Some("/home/user/my-project".into()),
            context_window: Some(ContextWindow {
                current_usage: tokens.map(|t| CurrentUsage {
                    input_tokens: Some(t),
                }),
                context_window_size: window_size,
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
        assert_eq!(render(&data, &Config::default(), &TranscriptData::default()), "[Opus] my-project");
    }

    #[test]
    fn render_empty_stdin() {
        let data = StdinData::default();
        assert_eq!(render(&data, &Config::default(), &TranscriptData::default()), "[cstat] no data");
    }

    #[test]
    fn render_missing_model_name() {
        let data = StdinData {
            model: Some(Model { display_name: None }),
            cwd: Some("/tmp/foo".into()),
            ..Default::default()
        };
        assert_eq!(render(&data, &Config::default(), &TranscriptData::default()), "[cstat] foo");
    }

    #[test]
    fn context_green_below_70() {
        let data = make_data(Some(45_000), Some(100_000));
        let cfg = Config::default();
        let out = render(&data, &cfg, &TranscriptData::default());
        assert_eq!(out, "[Opus] my-project  \x1b[32mctx 45%\x1b[0m");
    }

    #[test]
    fn context_yellow_at_70() {
        let data = make_data(Some(70_000), Some(100_000));
        let cfg = Config::default();
        let out = render(&data, &cfg, &TranscriptData::default());
        assert_eq!(out, "[Opus] my-project  \x1b[33mctx 70%\x1b[0m");
    }

    #[test]
    fn context_yellow_at_85() {
        let data = make_data(Some(85_000), Some(100_000));
        let cfg = Config::default();
        let out = render(&data, &cfg, &TranscriptData::default());
        assert_eq!(out, "[Opus] my-project  \x1b[33mctx 85%\x1b[0m");
    }

    #[test]
    fn context_red_above_85() {
        let data = make_data(Some(86_000), Some(100_000));
        let cfg = Config::default();
        let out = render(&data, &cfg, &TranscriptData::default());
        assert_eq!(out, "[Opus] my-project  \x1b[31mctx 86%\x1b[0m");
    }

    #[test]
    fn context_no_colors() {
        let data = make_data(Some(45_000), Some(100_000));
        let cfg = Config {
            colors: Some(false),
            ..Default::default()
        };
        assert_eq!(render(&data, &cfg, &TranscriptData::default()), "[Opus] my-project  ctx 45%");
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
        assert_eq!(render(&data, &Config::default(), &TranscriptData::default()), "[Opus] my-project");
    }

    #[test]
    fn context_missing_tokens() {
        let data = make_data(None, Some(100_000));
        assert_eq!(render(&data, &Config::default(), &TranscriptData::default()), "[Opus] my-project");
    }

    #[test]
    fn context_zero_window_size() {
        let data = make_data(Some(1000), Some(0));
        assert_eq!(render(&data, &Config::default(), &TranscriptData::default()), "[Opus] my-project");
    }

    #[test]
    fn context_custom_thresholds() {
        let data = make_data(Some(55_000), Some(100_000));
        let cfg = Config {
            context_warning: Some(50),
            context_critical: Some(60),
            ..Default::default()
        };
        let out = render(&data, &cfg, &TranscriptData::default());
        assert_eq!(out, "[Opus] my-project  \x1b[33mctx 55%\x1b[0m");
    }

    #[test]
    fn context_integer_percentage() {
        let data = make_data(Some(33_333), Some(100_000));
        let cfg = Config {
            colors: Some(false),
            ..Default::default()
        };
        assert_eq!(render(&data, &cfg, &TranscriptData::default()), "[Opus] my-project  ctx 33%");
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
        assert_eq!(render(&data, &cfg, &TranscriptData::default()), "[Opus] user/my-project");
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
        assert_eq!(render(&data, &cfg, &TranscriptData::default()), "[Opus] home/user/my-project");
    }

    #[test]
    fn custom_separator() {
        let data = make_data(Some(10_000), Some(100_000));
        let cfg = Config {
            colors: Some(false),
            separator: Some(" | ".into()),
            ..Default::default()
        };
        let out = render(&data, &cfg, &TranscriptData::default());
        assert_eq!(out, "[Opus] my-project | ctx 10%");
    }

    #[test]
    fn context_double_space_separator() {
        let data = make_data(Some(10_000), Some(100_000));
        let cfg = Config {
            colors: Some(false),
            ..Default::default()
        };
        let out = render(&data, &cfg, &TranscriptData::default());
        assert!(out.contains("my-project  ctx"));
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
    fn activity_line_hidden_when_no_tools() {
        let data = StdinData {
            model: Some(Model {
                display_name: Some("Opus".into()),
            }),
            cwd: Some("/tmp/proj".into()),
            ..Default::default()
        };
        let transcript = TranscriptData::default();
        let out = render(&data, &no_colors_cfg(), &transcript);
        assert!(!out.contains('\n'));
    }

    #[test]
    fn activity_line_running_tool_with_target() {
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
        let out = render(&data, &no_colors_cfg(), &transcript);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1], "Edit auth.ts");
    }

    #[test]
    fn activity_line_running_tool_without_target() {
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
        let out = render(&data, &no_colors_cfg(), &transcript);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[1], "Glob");
    }

    #[test]
    fn activity_line_completed_tools_grouped() {
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
        let out = render(&data, &no_colors_cfg(), &transcript);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        let activity = lines[1];
        assert!(activity.contains("Read x3"));
        assert!(activity.contains("Grep"));
    }

    #[test]
    fn activity_line_max_3_completed_groups() {
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
        let out = render(&data, &no_colors_cfg(), &transcript);
        let activity = out.lines().nth(1).unwrap();
        let group_count = activity.split("  ").count();
        assert!(group_count <= 3);
    }

    #[test]
    fn activity_line_running_plus_completed() {
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
        let out = render(&data, &no_colors_cfg(), &transcript);
        let activity = out.lines().nth(1).unwrap();
        assert!(activity.contains("Edit main.rs"));
        assert!(activity.contains("Read x2"));
    }

    #[test]
    fn activity_line_with_colors() {
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
        let out = render(&data, &Config::default(), &transcript);
        let activity = out.lines().nth(1).unwrap();
        assert!(activity.contains(BRIGHT));
        assert!(activity.contains(DIM));
        assert!(activity.contains(RESET));
    }

    #[test]
    fn activity_line_single_completed_no_count() {
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
        let out = render(&data, &no_colors_cfg(), &transcript);
        let activity = out.lines().nth(1).unwrap();
        assert_eq!(activity, "Grep");
        assert!(!activity.contains("x1"));
    }

    #[test]
    fn activity_line_uses_config_separator() {
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
        let out = render(&data, &cfg, &transcript);
        let activity = out.lines().nth(1).unwrap();
        assert!(activity.contains(" | "));
    }
}
