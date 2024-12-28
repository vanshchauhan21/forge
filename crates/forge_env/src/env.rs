use derive_setters::Setters;
use forge_walker::Walker;
use handlebars::Handlebars;
use serde::Serialize;

use crate::Result;

#[derive(Default, Serialize, Debug, Setters, Clone)]
#[serde(rename_all = "camelCase")]
#[setters(strip_option)]
pub struct Environment {
    pub os: String,
    pub cwd: String,
    pub shell: String,
    pub home: Option<String>,
    pub files: Vec<String>,
}

impl Environment {
    pub async fn from_env() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let files = match Walker::new(cwd.clone()).get().await {
            Ok(files) => files
                .into_iter()
                .filter(|f| !f.is_dir)
                .map(|f| f.path)
                .collect(),
            Err(_) => vec![],
        };

        Ok(Environment {
            os: std::env::consts::OS.to_string(),
            cwd: cwd.display().to_string(),
            shell: if cfg!(windows) {
                std::env::var("COMSPEC")?
            } else {
                std::env::var("SHELL").unwrap_or("/bin/sh".to_string())
            },
            home: dirs::home_dir().map(|a| a.display().to_string()),
            files,
        })
    }

    pub fn render(&self, template: &str) -> Result<String> {
        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        Ok(hb.render_template(template, &self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // use crate::default_ctx for unit test in the project.

    fn test_env() -> Environment {
        Environment {
            cwd: "/Users/test".into(),
            os: "TestOS".into(),
            shell: "ZSH".into(),
            home: Some("/Users".into()),
            files: vec!["test.txt".into()],
        }
    }

    #[test]
    fn test_render_with_custom_context() {
        let result = test_env().render("OS: {{os}}, CWD: {{cwd}}").unwrap();
        assert_eq!(result, "OS: TestOS, CWD: /Users/test");
    }
}
