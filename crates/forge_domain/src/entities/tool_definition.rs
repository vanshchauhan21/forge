use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};

use crate::{ToolName, UsageParameterPrompt, UsagePrompt};

///
/// Refer to the specification over here:
/// https://glama.ai/blog/2024-11-25-model-context-protocol-quickstart#server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    pub name: ToolName,
    pub description: String,
    pub input_schema: RootSchema,
    pub output_schema: Option<RootSchema>,
}

impl ToolDefinition {
    pub fn usage_prompt(&self) -> UsagePrompt {
        let input_parameters = self
            .input_schema
            .schema
            .object
            .clone()
            .map(|object| {
                object
                    .properties
                    .keys()
                    .map(|name| UsageParameterPrompt {
                        parameter_name: name.to_string(),
                        parameter_type: "...".to_string(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        UsagePrompt {
            tool_name: self.name.clone().into_string(),
            input_parameters,
            description: self.description.to_string(),
        }
    }
}
