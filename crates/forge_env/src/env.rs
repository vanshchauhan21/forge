use handlebars::{Handlebars, RenderError};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Environment {
    pub os: Option<String>,
    pub cwd: Option<String>,
    pub shell: Option<String>,
    pub home: Option<String>,
}

impl Environment {
    pub fn from_env() -> Self {
        Environment {
            os: Some(std::env::consts::OS.to_string()),
            cwd: std::env::current_dir()
                .ok()
                .map(|p| p.display().to_string()),
            shell: if cfg!(windows) {
                std::env::var("COMSPEC").ok().map(String::from)
            } else {
                std::env::var("SHELL").ok().or(Some("/bin/sh".to_string()))
            },
            home: dirs::home_dir().map(|a| a.display().to_string()),
        }
    }
}

impl Environment {
    pub fn render(&self, template: &str) -> Result<String, RenderError> {
        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        hb.render_template(template, &self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // use crate::default_ctx for unit test in the project.

    fn test_env() -> Environment {
        Environment {
            cwd: Some("/Users/test".into()),
            os: Some("TestOS".into()),
            shell: Some("ZSH".into()),
            home: Some("/Users".into()),
        }
    }

    #[test]
    fn test_render_with_custom_context() {
        let result = test_env().render("OS: {{os}}, CWD: {{cwd}}").unwrap();
        assert_eq!(result, "OS: TestOS, CWD: /Users/test");
    }
}
