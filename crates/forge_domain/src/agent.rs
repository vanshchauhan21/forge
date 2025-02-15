use derive_more::derive::Display;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use crate::prompt::Prompt;
use crate::{Environment, ModelId, ToolName, UserContext};

#[derive(Debug, Default, Setters, Clone, Serialize, Deserialize)]
#[setters(strip_option)]
pub struct SystemContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Environment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_information: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_supported: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

#[derive(Debug, Display, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(String);
impl AgentId {
    pub fn new(id: impl ToString) -> Self {
        Self(id.to_string())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<ToolName> for AgentId {
    fn from(value: ToolName) -> Self {
        Self(value.into_string())
    }
}

fn is_true(value: &bool) -> bool {
    *value
}

fn truth() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub model: ModelId,
    pub description: Option<String>,
    pub system_prompt: Prompt<SystemContext>,
    pub user_prompt: Prompt<UserContext>,

    /// Suggests if the agent needs to maintain its state for the lifetime of
    /// the program.    
    #[serde(skip_serializing_if = "is_true", default = "truth")]
    pub ephemeral: bool,

    /// Tools that the agent can use    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolName>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transforms: Vec<Transform>,

    /// Used to specify the events the agent is interested in    
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subscribe: Vec<String>,

    /// Maximum number of turns the agent can take    
    pub max_turns: u64,
}

/// Transformations that can be applied to the agent's context before sending it
/// upstream to the provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Transform {
    /// Compresses multiple assistant messages into a single message
    Assistant {
        input: String,
        output: String,
        agent_id: AgentId,
        token_limit: usize,
    },

    /// Works on the user prompt by enriching it with additional information
    User { agent_id: AgentId, output: String },

    /// Intercepts the context and performs an operation without changing the
    /// context
    PassThrough { agent_id: AgentId, input: String },
}
