use derive_more::derive::From;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::tool_call_parser::parse;
use crate::{Error, Result, ToolName};

/// Unique identifier for a using a tool
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ToolCallId(pub(crate) String);

impl ToolCallId {
    pub fn new(value: impl ToString) -> Self {
        ToolCallId(value.to_string())
    }
}

/// Contains a part message for using a tool. This is received as a part of the
/// response from the model only when streaming is enabled.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Setters)]
#[setters(strip_option, into)]
pub struct ToolCallPart {
    /// Optional unique identifier that represents a single call to the tool
    /// use. NOTE: Not all models support a call ID for using a tool
    pub call_id: Option<ToolCallId>,
    pub name: Option<ToolName>,

    /// Arguments that need to be passed to the tool. NOTE: Not all tools
    /// require input
    pub arguments_part: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, From)]
pub enum ToolCall {
    Full(ToolCallFull),
    Part(ToolCallPart),
}

/// Contains the full information about using a tool. This is received as a part
/// of the response from the model when streaming is disabled.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Setters)]
#[setters(strip_option, into)]
pub struct ToolCallFull {
    pub name: ToolName,
    pub call_id: Option<ToolCallId>,
    pub arguments: Value,
}

impl ToolCallFull {
    pub fn new(tool_name: ToolName) -> Self {
        Self { name: tool_name, call_id: None, arguments: Value::default() }
    }
    pub fn try_from_parts(parts: &[ToolCallPart]) -> Result<Self> {
        let mut tool_name = None;
        let mut tool_call_id = None;

        let mut input = String::new();
        for part in parts.iter() {
            if let Some(value) = &part.name {
                tool_name = Some(value);
            }

            if let Some(value) = &part.call_id {
                tool_call_id = Some(value);
            }

            input.push_str(&part.arguments_part);
        }

        if let Some(tool_name) = tool_name {
            Ok(ToolCallFull {
                name: tool_name.clone(),
                call_id: tool_call_id.cloned(),
                arguments: serde_json::from_str(&input)?,
            })
        } else {
            Err(Error::ToolCallMissingName)
        }
    }

    /// Parse multiple tool calls from XML format.
    pub fn try_from_xml(input: &str) -> std::result::Result<Vec<Self>, String> {
        parse(input)
    }
}
