use crate::types::{Config, StdinData};

pub fn render(data: &StdinData, _config: &Config) -> String {
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

    format!("[{model_name}] {project_name}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Model, StdinData};

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
}
