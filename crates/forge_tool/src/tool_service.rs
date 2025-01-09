use std::collections::HashMap;

use forge_domain::{Tool, ToolCallFull, ToolDefinition, ToolName, ToolResult, ToolService};
use serde_json::Value;
use tracing::info;

use crate::fs::*;
use crate::outline::Outline;
use crate::shell::Shell;
use crate::think::Think;
use crate::Service;

struct Live {
    tools: HashMap<ToolName, Tool>,
}

impl FromIterator<Tool> for Live {
    fn from_iter<T: IntoIterator<Item = Tool>>(iter: T) -> Self {
        let tools: HashMap<ToolName, Tool> = iter
            .into_iter()
            .map(|tool| (tool.definition.name.clone(), tool))
            .collect::<HashMap<_, _>>();

        Self { tools }
    }
}

#[async_trait::async_trait]
impl ToolService for Live {
    async fn call(&self, call: ToolCallFull) -> ToolResult {
        let name = call.name.clone();
        let input = call.arguments.clone();
        info!("Calling tool: {}", name.as_str());
        let available_tools = self
            .tools
            .keys()
            .map(|name| name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let output = match self.tools.get(&name) {
            Some(tool) => tool.executable.call(input).await,
            None => Err(format!(
                "No tool with name '{}' was found. Please try again with one of these tools {}",
                name.as_str(),
                available_tools
            )),
        };

        match output {
            Ok(output) => ToolResult::from(call).content(output),
            Err(error) => {
                ToolResult::from(call).content(Value::from(format!("<error>{}</error>", error)))
            }
        }
    }

    fn list(&self) -> Vec<ToolDefinition> {
        let mut tools: Vec<_> = self
            .tools
            .values()
            .map(|tool| tool.definition.clone())
            .collect();

        // Sorting is required to ensure system prompts are exactly the same
        tools.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));

        tools
    }

    fn usage_prompt(&self) -> String {
        let mut tools: Vec<_> = self.tools.values().collect();
        tools.sort_by(|a, b| a.definition.name.as_str().cmp(b.definition.name.as_str()));

        tools
            .iter()
            .enumerate()
            .fold("".to_string(), |mut acc, (i, tool)| {
                acc.push('\n');
                acc.push_str((i + 1).to_string().as_str());
                acc.push_str(". ");
                acc.push_str(tool.definition.usage_prompt().to_string().as_str());
                acc
            })
    }
}

impl Service {
    pub fn tool_service() -> impl ToolService {
        Live::from_iter([
            Tool::new(FSRead),
            Tool::new(FSWrite),
            Tool::new(FSList),
            Tool::new(FSSearch),
            Tool::new(FSFileInfo),
            Tool::new(FSReplace),
            Tool::new(Outline),
            Tool::new(Shell::default()),
            Tool::new(Think::default()),
        ])
    }
}

#[cfg(test)]
mod test {
    use insta::assert_snapshot;

    use super::*;
    use crate::fs::{FSFileInfo, FSSearch};

    #[test]
    fn test_id() {
        assert!(Tool::new(FSRead)
            .definition
            .name
            .into_string()
            .ends_with("fs_read"));
        assert!(Tool::new(FSSearch)
            .definition
            .name
            .into_string()
            .ends_with("fs_search"));
        assert!(Tool::new(FSList)
            .definition
            .name
            .into_string()
            .ends_with("fs_list"));
        assert!(Tool::new(FSFileInfo)
            .definition
            .name
            .into_string()
            .ends_with("file_info"));
    }

    #[test]
    fn test_usage_prompt() {
        let docs = Service::tool_service().usage_prompt();

        assert_snapshot!(docs);
    }

    #[test]
    fn test_tool_definition() {
        let tools = Service::tool_service().list();

        assert_snapshot!(serde_json::to_string_pretty(&tools).unwrap());
    }
}
