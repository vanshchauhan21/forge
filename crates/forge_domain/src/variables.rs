use std::collections::HashMap;

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{NamedTool, ToolCallFull, ToolDefinition, ToolName};

#[derive(Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Variables(HashMap<String, Value>);

impl Variables {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn merge(self, other: Self) -> Self {
        let mut merged = self;
        merged.0.extend(other.0);
        merged
    }

    pub fn default_key() -> &'static str {
        "value"
    }

    pub fn new_pair(key: impl Into<String>, value: impl Into<Value>) -> Self {
        let mut variables = Self::default();
        variables.set(key, value);
        variables
    }
}

impl From<Vec<Variables>> for Variables {
    fn from(value: Vec<Variables>) -> Self {
        value
            .into_iter()
            .reduce(|a, b| a.merge(b))
            .unwrap_or_default()
    }
}

impl From<Value> for Variables {
    fn from(value: Value) -> Self {
        let mut variables = Variables::default();
        match value {
            Value::Null => {}
            Value::Bool(value) => {
                variables.set(Self::default_key(), value.to_string());
            }
            Value::Number(value) => {
                variables.set(Self::default_key(), value.to_string());
            }
            Value::String(value) => {
                variables.set(Self::default_key(), value);
            }
            Value::Array(values) => {
                variables.set(Self::default_key(), values);
            }
            Value::Object(map) => {
                for (key, value) in map {
                    variables.set(key, value);
                }
            }
        };

        variables
    }
}

#[derive(Debug, JsonSchema, Deserialize)]
pub struct ReadVariable {
    pub name: String,
}

impl ReadVariable {
    pub fn tool_definition() -> ToolDefinition {
        ToolDefinition {
            name: ToolName::new("forge_read_variable"),
            description: "Reads a global workflow variable".to_string(),
            input_schema: schema_for!(Self),
            output_schema: None,
        }
    }

    pub fn parse(tool_call: &ToolCallFull) -> Option<Self> {
        if tool_call.name != Self::tool_definition().name {
            return None;
        }
        serde_json::from_value(tool_call.arguments.clone()).ok()
    }
}

impl NamedTool for ReadVariable {
    fn tool_name() -> ToolName {
        Self::tool_definition().name
    }
}

#[derive(Debug, JsonSchema, Deserialize)]
pub struct WriteVariable {
    pub name: String,
    pub value: String,
}

impl WriteVariable {
    pub fn tool_definition() -> ToolDefinition {
        ToolDefinition {
            name: ToolName::new("forge_write_variable"),
            description: "Writes a global workflow variable".to_string(),
            input_schema: schema_for!(Self),
            output_schema: None,
        }
    }

    pub fn parse(tool_call: &ToolCallFull) -> Option<Self> {
        if tool_call.name != Self::tool_definition().name {
            return None;
        }
        serde_json::from_value(tool_call.arguments.clone()).ok()
    }
}

impl NamedTool for WriteVariable {
    fn tool_name() -> ToolName {
        Self::tool_definition().name
    }
}
