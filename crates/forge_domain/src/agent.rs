use std::path::PathBuf;

use derive_more::derive::Display;
use derive_setters::Setters;
use handlebars::Handlebars;
use schemars::schema::RootSchema;
use serde::{Deserialize, Serialize};

use crate::variables::Variables;
use crate::{Environment, Error, ModelId, Provider, ToolDefinition, ToolName};

#[derive(Default, Setters, Clone, Serialize, Deserialize)]
#[setters(strip_option)]
pub struct SystemContext {
    pub env: Option<Environment>,
    pub tool_information: Option<String>,
    pub tool_supported: Option<bool>,
    pub custom_instructions: Option<String>,
    pub files: Vec<String>,
}

pub enum PromptContent {
    Text(String),
    File(PathBuf),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Prompt<V> {
    pub template: PromptTemplate,
    pub variables: Schema<V>,
}

impl<V: Serialize> Prompt<V> {
    pub fn render(&self, ctx: &V) -> crate::Result<String> {
        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        hb.register_escape_fn(|str| str.to_string());

        hb.render_template(self.template.as_str(), &ctx)
            .map_err(Error::Template)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema<S> {
    pub schema: RootSchema,
    _marker: std::marker::PhantomData<S>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromptTemplate(String);
impl PromptTemplate {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Display, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(String);

impl From<ToolName> for AgentId {
    fn from(value: ToolName) -> Self {
        Self(value.into_string())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub provider: Provider,
    pub model: ModelId,
    pub description: String,
    pub system_prompt: Prompt<SystemContext>,
    pub user_prompt: Prompt<Variables>,

    /// Suggests if the agent needs to maintain its state for the lifetime of
    /// the program.
    pub ephemeral: bool,

    /// Tools that the agent can use
    pub tools: Vec<ToolName>,
    pub transforms: Vec<Transform>,

    /// Downstream agents that this agent can handover to
    pub handovers: Vec<Downstream>,

    /// Represents that the agent is the entry point to the workflow
    pub entry: bool,
}

impl From<Agent> for ToolDefinition {
    fn from(value: Agent) -> Self {
        ToolDefinition {
            name: ToolName::new(value.id.0),
            description: value.description,
            input_schema: value.user_prompt.variables.schema,
            output_schema: None,
        }
    }
}

/// Transformations that can be applied to the agent's context before sending it
/// upstream to the provider.
#[derive(Clone, Serialize, Deserialize)]
pub enum Transform {
    /// Compresses multiple assistant messages into a single message
    Assistant {
        input: String,
        output: String,
        agent_id: AgentId,
        token_limit: usize,
    },

    /// Works on the user prompt by enriching it with additional information
    User {
        agent_id: AgentId,
        input: String,
        output: String,
    },

    /// Intercepts the context and performs an operation without changing the
    /// context
    Tap { agent_id: AgentId, input: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downstream {
    pub agent: AgentId,
    pub wait: bool,
}
