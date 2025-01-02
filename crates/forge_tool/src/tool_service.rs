use std::collections::{BTreeSet, HashMap};
use std::fmt::Display;

use inflector::Inflector;
use schemars::schema::RootSchema;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::fs::*;
use crate::outline::Outline;
use crate::shell::Shell;
use crate::think::Think;
use crate::{Description, Service, ToolCallService};

struct JsonTool<T>(T);

#[async_trait::async_trait]
impl<T: ToolCallService + Sync> ToolCallService for JsonTool<T>
where
    T::Input: serde::de::DeserializeOwned + JsonSchema,
    T::Output: serde::Serialize + JsonSchema,
{
    type Input = Value;
    type Output = Value;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let input: T::Input = serde_json::from_value(input).map_err(|e| e.to_string())?;
        let output: T::Output = self.0.call(input).await?;
        Ok(serde_json::to_value(output).map_err(|e| e.to_string())?)
    }
}

#[async_trait::async_trait]
pub trait ToolService: Send + Sync {
    async fn call(&self, name: &ToolName, input: Value) -> Result<Value, String>;
    fn list(&self) -> Vec<ToolDefinition>;
    fn usage_prompt(&self) -> String;
}

struct Live {
    tools: HashMap<ToolName, Tool>,
}

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

#[derive(Debug)]
pub struct UsagePrompt {
    tool_name: String,
    input_parameters: Vec<UsageParameterPrompt>,
    description: String,
}

impl Display for UsagePrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.tool_name)?;
        f.write_str("\n")?;
        f.write_str(&self.description)?;

        f.write_str("\n\nUsage:\n")?;
        f.write_str("<")?;
        f.write_str(&self.tool_name)?;
        f.write_str(">")?;

        for parameter in self.input_parameters.iter() {
            f.write_str("\n")?;
            parameter.fmt(f)?;
        }

        f.write_str("\n")?;
        f.write_str("</")?;
        f.write_str(&self.tool_name)?;
        f.write_str(">\n")?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct UsageParameterPrompt {
    pub parameter_name: String,
    pub parameter_type: String,
}

impl Display for UsageParameterPrompt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<")?;
        f.write_str(&self.parameter_name)?;
        f.write_str(">")?;
        f.write_str(&self.parameter_type)?;
        f.write_str("</")?;
        f.write_str(&self.parameter_name)?;
        f.write_str(">")?;

        Ok(())
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ToolName(String);

impl ToolName {
    pub fn new(value: impl ToString) -> Self {
        ToolName(value.to_string())
    }
}

impl ToolName {
    pub fn into_string(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Live {
    fn new() -> Self {
        let tools: HashMap<ToolName, Tool> = [
            Tool::new(FSRead),
            Tool::new(FSWrite),
            Tool::new(FSList),
            Tool::new(FSSearch),
            Tool::new(FSFileInfo),
            Tool::new(FSReplace),
            Tool::new(Outline),
            Tool::new(Shell::default()),
            // TODO: uncomment them later on, as of now we only need the above tools.
            Tool::new(Think::default()),
            // importer::import(AskFollowUpQuestion),
        ]
        .into_iter()
        .map(|tool| (tool.name.clone(), tool))
        .collect::<HashMap<_, _>>();

        Self { tools }
    }
}

#[async_trait::async_trait]
impl ToolService for Live {
    async fn call(&self, name: &ToolName, input: Value) -> Result<Value, String> {
        let output = match self.tools.get(name) {
            Some(tool) => tool.executable.call(input).await,
            None => Err(format!("No such tool found: {}", name.as_str())),
        };

        output
    }

    fn list(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|tool| tool.definition.clone())
            .collect()
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

struct Tool {
    name: ToolName,
    executable: Box<dyn ToolCallService<Input = Value, Output = Value> + Send + Sync + 'static>,
    definition: ToolDefinition,
}

impl Tool {
    fn new<T>(tool: T) -> Tool
    where
        T: ToolCallService + Description + Send + Sync + 'static,
        T::Input: serde::de::DeserializeOwned + JsonSchema,
        T::Output: serde::Serialize + JsonSchema,
    {
        let name = std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap()
            .to_snake_case();
        let executable = Box::new(JsonTool(tool));

        let input: RootSchema = schema_for!(T::Input);
        let output: RootSchema = schema_for!(T::Output);
        let mut description = T::description().to_string();

        description.push_str("\n\nParameters:");

        let required = input
            .schema
            .clone()
            .object
            .iter()
            .flat_map(|object| object.required.clone().into_iter())
            .collect::<BTreeSet<_>>();
        for (name, desc) in input
            .schema
            .object
            .clone()
            .into_iter()
            .flat_map(|object| object.properties.into_iter())
            .flat_map(|(name, props)| {
                props
                    .into_object()
                    .metadata
                    .into_iter()
                    .map(move |meta| (name.clone(), meta))
            })
            .flat_map(|(name, meta)| {
                meta.description
                    .into_iter()
                    .map(move |desc| (name.clone(), desc))
            })
        {
            description.push_str("\n- ");
            description.push_str(&name);

            if required.contains(&name) {
                description.push_str(" (required)");
            }

            description.push_str(": ");
            description.push_str(&desc);
        }

        let tool = ToolDefinition {
            name: ToolName(name.clone()),
            description,
            input_schema: input,
            output_schema: Some(output),
        };

        Tool { executable, definition: tool, name: ToolName(name) }
    }
}

impl Service {
    pub fn live() -> impl ToolService {
        Live::new()
    }
}

#[cfg(test)]
mod test {

    use insta::assert_snapshot;

    use super::*;
    use crate::fs::{FSFileInfo, FSSearch};

    #[test]
    fn test_id() {
        assert!(Tool::new(FSRead).name.into_string().ends_with("fs_read"));
        assert!(Tool::new(FSSearch)
            .name
            .into_string()
            .ends_with("fs_search"));
        assert!(Tool::new(FSList).name.into_string().ends_with("fs_list"));
        assert!(Tool::new(FSFileInfo)
            .name
            .into_string()
            .ends_with("file_info"));
    }

    #[test]
    fn test_usage_prompt() {
        let docs = Live::new().usage_prompt();

        assert_snapshot!(docs);
    }
}
