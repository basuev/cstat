use crate::types::{Config, StdinData};

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

pub fn render(data: &StdinData, config: &Config) -> String {
    let model_name = data
        .model
        .as_ref()
        .and_then(|m| m.display_name.as_deref())
        .unwrap_or("cstat");

    let project_name = data
        .cwd
        .as_deref()
        .and_then(|p| p.rsplit('/').next())
        .unwrap_or("no data");

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

    line
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ContextWindow, CurrentUsage, Model, StdinData};

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
        assert_eq!(render(&data, &Config::default()), "[Opus] my-project");
    }

    #[test]
    fn render_empty_stdin() {
        let data = StdinData::default();
        assert_eq!(render(&data, &Config::default()), "[cstat] no data");
    }

    #[test]
    fn render_missing_model_name() {
        let data = StdinData {
            model: Some(Model { display_name: None }),
            cwd: Some("/tmp/foo".into()),
            ..Default::default()
        };
        assert_eq!(render(&data, &Config::default()), "[cstat] foo");
    }

    #[test]
    fn context_green_below_70() {
        let data = make_data(Some(45_000), Some(100_000));
        let cfg = Config::default();
        let out = render(&data, &cfg);
        assert_eq!(out, "[Opus] my-project  \x1b[32mctx 45%\x1b[0m");
    }

    #[test]
    fn context_yellow_at_70() {
        let data = make_data(Some(70_000), Some(100_000));
        let cfg = Config::default();
        let out = render(&data, &cfg);
        assert_eq!(out, "[Opus] my-project  \x1b[33mctx 70%\x1b[0m");
    }

    #[test]
    fn context_yellow_at_85() {
        let data = make_data(Some(85_000), Some(100_000));
        let cfg = Config::default();
        let out = render(&data, &cfg);
        assert_eq!(out, "[Opus] my-project  \x1b[33mctx 85%\x1b[0m");
    }

    #[test]
    fn context_red_above_85() {
        let data = make_data(Some(86_000), Some(100_000));
        let cfg = Config::default();
        let out = render(&data, &cfg);
        assert_eq!(out, "[Opus] my-project  \x1b[31mctx 86%\x1b[0m");
    }

    #[test]
    fn context_no_colors() {
        let data = make_data(Some(45_000), Some(100_000));
        let cfg = Config {
            colors: Some(false),
            ..Default::default()
        };
        assert_eq!(render(&data, &cfg), "[Opus] my-project  ctx 45%");
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
        assert_eq!(render(&data, &Config::default()), "[Opus] my-project");
    }

    #[test]
    fn context_missing_tokens() {
        let data = make_data(None, Some(100_000));
        assert_eq!(render(&data, &Config::default()), "[Opus] my-project");
    }

    #[test]
    fn context_zero_window_size() {
        let data = make_data(Some(1000), Some(0));
        assert_eq!(render(&data, &Config::default()), "[Opus] my-project");
    }

    #[test]
    fn context_custom_thresholds() {
        let data = make_data(Some(55_000), Some(100_000));
        let cfg = Config {
            context_warning: Some(50),
            context_critical: Some(60),
            ..Default::default()
        };
        let out = render(&data, &cfg);
        assert_eq!(out, "[Opus] my-project  \x1b[33mctx 55%\x1b[0m");
    }

    #[test]
    fn context_integer_percentage() {
        let data = make_data(Some(33_333), Some(100_000));
        let cfg = Config {
            colors: Some(false),
            ..Default::default()
        };
        assert_eq!(render(&data, &cfg), "[Opus] my-project  ctx 33%");
    }

    #[test]
    fn context_double_space_separator() {
        let data = make_data(Some(10_000), Some(100_000));
        let cfg = Config {
            colors: Some(false),
            ..Default::default()
        };
        let out = render(&data, &cfg);
        assert!(out.contains("my-project  ctx"));
    }
}
