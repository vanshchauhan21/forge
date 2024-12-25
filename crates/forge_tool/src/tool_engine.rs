use std::collections::HashMap;

use inflector::Inflector;
use schemars::schema::RootSchema;
use schemars::{schema_for, JsonSchema};
use serde_json::Value;

use crate::shell::Shell;
use crate::think::Think;
use crate::{
    Description, FSFileInfo, FSList, FSRead, FSReplace, FSSearch, FSWrite, Outline, ToolTrait,
};

struct JsonTool<T>(T);

#[async_trait::async_trait]
impl<T: ToolTrait + Sync> ToolTrait for JsonTool<T>
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

struct ToolDefinition {
    executable: Box<dyn ToolTrait<Input = Value, Output = Value> + Send + Sync + 'static>,
    tool: Tool,
}

pub struct ToolEngine {
    tools: HashMap<ToolId, ToolDefinition>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Tool {
    pub id: ToolId,
    pub description: String,
    pub input_schema: RootSchema,
    pub output_schema: Option<RootSchema>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct ToolId(String);

impl ToolId {
    pub fn new(id: &str) -> Self {
        Self(id.to_string())
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ToolEngine {
    pub async fn call(&self, tool_id: &ToolId, input: Value) -> Result<Value, String> {
        match self.tools.get(tool_id) {
            Some(tool) => tool.executable.call(input).await,
            None => Err(format!("No such tool found: {}", tool_id.as_str())),
        }
    }

    pub fn list(&self) -> Vec<Tool> {
        self.tools.values().map(|tool| tool.tool.clone()).collect()
    }

    fn import<T>(tool: T) -> (ToolId, ToolDefinition)
    where
        T: ToolTrait + Description + Send + Sync + 'static,
        T::Input: serde::de::DeserializeOwned + JsonSchema,
        T::Output: serde::Serialize + JsonSchema,
    {
        let id = std::any::type_name::<T>()
            .split("::")
            .last()
            .unwrap()
            .to_snake_case();
        let executable = Box::new(JsonTool(tool));
        let tool = Tool {
            id: ToolId(id.clone()),
            description: T::description().to_string(),
            input_schema: schema_for!(T::Input),
            output_schema: Some(schema_for!(T::Output)),
        };

        (ToolId(id), ToolDefinition { executable, tool })
    }
}

impl Default for ToolEngine {
    fn default() -> Self {
        let tools: HashMap<ToolId, ToolDefinition> = HashMap::from([
            ToolEngine::import(FSRead),
            ToolEngine::import(FSSearch),
            ToolEngine::import(FSList),
            ToolEngine::import(FSFileInfo),
            ToolEngine::import(FSWrite),
            ToolEngine::import(FSReplace),
            ToolEngine::import(Outline),
            ToolEngine::import(Think::default()),
            ToolEngine::import(Shell::default()),
        ]);

        Self { tools }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_id() {
        assert!(ToolEngine::import(FSRead)
            .0
            .into_string()
            .ends_with("fs_read"));
        assert!(ToolEngine::import(FSSearch)
            .0
            .into_string()
            .ends_with("fs_search"));
        assert!(ToolEngine::import(FSList)
            .0
            .into_string()
            .ends_with("fs_list"));
        assert!(ToolEngine::import(FSFileInfo)
            .0
            .into_string()
            .ends_with("file_info"));
        assert!(ToolEngine::import(Think::default())
            .0
            .into_string()
            .ends_with("think"));
    }
}
