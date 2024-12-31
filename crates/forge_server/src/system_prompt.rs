use std::sync::Arc;

use forge_env::Environment;
use forge_tool::ToolEngine;
use handlebars::Handlebars;
use serde::Serialize;

use crate::Result;

#[derive(Clone, Serialize)]
struct Context {
    env: Environment,
    tool_information: String,
    use_tool: bool,
}

#[derive(Clone)]
pub struct SystemPrompt {
    ctx: Context,
}

impl SystemPrompt {
    pub fn new(env: Environment, tools: Arc<ToolEngine>) -> Self {
        let tool_information = tools.usage_prompt();

        Self { ctx: Context { env, tool_information, use_tool: true } }
    }
    pub fn use_tool(mut self, use_tool: bool) -> Self {
        self.ctx.use_tool = use_tool;
        self
    }
}

impl SystemPrompt {
    pub fn render(&self) -> Result<String> {
        let template = include_str!("./prompts/system.md").to_string();

        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        hb.register_escape_fn(|str| str.to_string());

        Ok(hb.render_template(template.as_str(), &self.ctx)?)
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
        let env = test_env();
        let tools = Arc::new(ToolEngine::new());
        let prompt = SystemPrompt::new(env, tools).render().unwrap();
        assert_snapshot!(prompt);
    }

    #[test]
    fn test_tool_unsupported() {
        let env = test_env();
        let tools = Arc::new(ToolEngine::new());
        let prompt = SystemPrompt::new(env, tools)
            .use_tool(true)
            .render()
            .unwrap();
        assert_snapshot!(prompt);
    }
}
