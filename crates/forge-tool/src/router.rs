use crate::console::{ReadLine, WriteLine};
use crate::fs::{FSFileInfo, FSList, FSRead, FSSearch};
use crate::think::Think;
use crate::ToolTrait;
use inflector::Inflector;
use serde_json::Value;
use std::collections::HashMap;

struct JsonTool<T>(T);

impl<T> JsonTool<T> {
    fn import(tool: T) -> Box<dyn ToolTrait<Input = Value, Output = Value> + Sync + 'static>
    where
        T: ToolTrait + Sync + 'static,
        T::Input: serde::de::DeserializeOwned,
        T::Output: serde::Serialize,
    {
        Box::new(Self(tool))
    }
}

#[async_trait::async_trait]
impl<T: ToolTrait + Sync> ToolTrait for JsonTool<T>
where
    T::Input: serde::de::DeserializeOwned,
    T::Output: serde::Serialize,
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
pub struct JsonSchema(Value);

impl JsonSchema {
    pub(crate) fn from_value(value: Value) -> Self {
        JsonSchema(value)
    }

    pub fn into_value(self) -> Value {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Tool {
    pub id: ToolId,
    pub description: String,
    pub input_schema: JsonSchema,
    pub output_schema: Option<JsonSchema>,
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
        todo!()
    }

    pub fn list(&self) -> Vec<Tool> {
        todo!()
    }

    fn import<T>(tool: T) -> (ToolId, Box<dyn ToolTrait<Input = Value, Output = Value>>)
    where
        T: ToolTrait + Send + Sync + 'static,
        T::Input: serde::de::DeserializeOwned,
        T::Output: serde::Serialize,
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
