use std::collections::HashMap;

use inflector::Inflector;
use schemars::JsonSchema;
use serde_json::Value;

use crate::console::{ReadLine, WriteLine};
use crate::fs::{FSFileInfo, FSList, FSRead, FSSearch};
use crate::think::Think;
use crate::ToolTrait;

struct JsonTool<T>(T);

#[async_trait::async_trait]
impl<T: ToolTrait + Sync> ToolTrait for JsonTool<T>
where
    T::Input: serde::de::DeserializeOwned + JsonSchema,
    T::Output: serde::Serialize + JsonSchema,
{
    type Input = Value;
    type Output = Value;

    fn description(&self) -> String {
        self.0.description()
    }

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let input: T::Input = serde_json::from_value(input).map_err(|e| e.to_string())?;
        let output: T::Output = self.0.call(input).await?;
        Ok(serde_json::to_value(output).map_err(|e| e.to_string())?)
    }
}

impl<T> JsonTool<T> {
    fn id(&self) -> ToolId {
        let id = std::any::type_name::<T>();
        let out = id
            .split("::")
            .map(|v| v.to_snake_case())
            .collect::<Vec<_>>()
            .join("/");
        ToolId(out)
    }
}

pub struct Router {
    tools: HashMap<ToolId, Box<dyn ToolTrait<Input = Value, Output = Value>>>,
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub id: ToolId,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ToolId(String);

impl ToolId {
    pub fn into_string(self) -> String {
        self.0
    }
}

impl Router {
    pub async fn call(&self, tool_id: ToolId, input: Value) -> Result<Value, String> {
        match self.tools.get(&tool_id) {
            Some(tool) => tool.call(input).await,
            None => Err(format!("No such tool found: {}", tool_id.into_string())),
        }
    }

    pub fn list(&self) -> Vec<Tool> {
        todo!()
    }

    fn import<T>(tool: T) -> (ToolId, Box<dyn ToolTrait<Input = Value, Output = Value>>)
    where
        T: ToolTrait + Send + Sync + 'static,
        T::Input: serde::de::DeserializeOwned + JsonSchema,
        T::Output: serde::Serialize + JsonSchema,
    {
        let id = std::any::type_name::<T>()
            .split("::")
            .map(|v| v.to_snake_case())
            .collect::<Vec<_>>()
            .join("/");
        let json = Box::new(JsonTool(tool));
        (ToolId(id), json)
    }
}

impl Default for Router {
    fn default() -> Self {
        let tools: HashMap<ToolId, Box<dyn ToolTrait<Input = Value, Output = Value>>> =
            HashMap::from([
                Router::import(FSRead),
                Router::import(FSSearch),
                Router::import(FSList),
                Router::import(FSFileInfo),
                Router::import(Think::default()),
                Router::import(ReadLine),
                Router::import(WriteLine),
            ]);

        Self { tools }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_id() {
        assert!(Router::import(FSRead)
            .0
            .into_string()
            .ends_with("fs/fs_read"));
        assert!(Router::import(FSSearch)
            .0
            .into_string()
            .ends_with("fs/fs_search"));
        assert!(Router::import(FSList)
            .0
            .into_string()
            .ends_with("fs/fs_list"));
        assert!(Router::import(FSFileInfo)
            .0
            .into_string()
            .ends_with("fs/fs_file_info"));
        assert!(Router::import(Think::default())
            .0
            .into_string()
            .ends_with("/think"));
    }
}
