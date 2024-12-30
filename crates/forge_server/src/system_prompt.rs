use crate::Result;
use derive_setters::Setters;
use forge_env::Environment;

pub struct SystemPrompt {
    template: String,
    config: Config,
}

#[derive(Setters)]
pub struct Config {
    pub use_tool: bool,
    pub env: Environment,
}

impl Config {
    pub fn new(env: Environment) -> Self {
        Self { use_tool: true, env }
    }
}

impl SystemPrompt {
    pub fn new(config: Config) -> Self {
        let template = include_str!("./prompts/system.md").to_string();
        Self { template, config }
    }
}

impl SystemPrompt {
    pub fn render(&self) -> Result<String> {
        Ok(self.config.env.render(&self.template)?)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;

    fn test_env() -> Environment {
        Environment {
            os: "linux".to_string(),
            cwd: "/home/user/project".to_string(),
            shell: "/bin/bash".to_string(),
            home: Some("/home/user".to_string()),
            files: vec!["file1.txt".to_string(), "file2.txt".to_string()],
        }
    }

    #[test]
    fn test_tool_supported() {
        let config = Config::new(test_env());
        let prompt = SystemPrompt::new(config).render().unwrap();
        assert_snapshot!(prompt);
    }
}
