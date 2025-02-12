use derive_builder::Builder;
use derive_more::derive::Display;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};

use crate::prompt::Prompt;
use crate::{Context, Environment, ModelId, ToolName, UserContext};

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Default, Setters, Clone, Serialize, Deserialize)]
#[setters(strip_option)]
pub struct SystemContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<Environment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_information: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_supported: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_instructions: Option<String>,
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

#[derive(Clone, Serialize, Deserialize, Builder)]
#[builder(setter(into), pattern = "immutable")]
pub struct Agent {
    pub id: AgentId,
    pub model: ModelId,
    #[builder(default)]
    pub description: Option<String>,
    pub system_prompt: Prompt<SystemContext>,
    pub user_prompt: Prompt<UserContext>,

    /// Suggests if the agent needs to maintain its state for the lifetime of
    /// the program.
    #[builder(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub ephemeral: bool,

    /// Tools that the agent can use
    #[builder(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolName>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub transforms: Vec<Transform>,

    /// Used to specify the events the agent is interested in
    #[builder(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subscribe: Vec<String>,

    /// Maximum number of turns the agent can take
    #[builder(default = "1024")]
    pub max_turns: u64,

    /// Internal state of the agent
    #[serde(skip)]
    #[builder(default)]
    pub(crate) state: AgentState,
}

#[derive(Clone, Default)]
pub struct AgentState {
    pub turn_count: u64,
    pub context: Option<Context>,
}

/// Transformations that can be applied to the agent's context before sending it
/// upstream to the provider.
#[derive(Clone, Serialize, Deserialize)]
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
