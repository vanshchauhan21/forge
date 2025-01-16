use std::fmt;
use std::marker::PhantomData;
use std::path::PathBuf;

use async_trait::async_trait;
use derive_setters::Setters;
use handlebars::Handlebars;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::{Context, ContextMessage, ModelId};

/// Represents the contents of a prompt, which can either be direct text or a
/// file reference
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PromptContent {
    Text(String),
    File(PathBuf),
}

impl From<String> for PromptContent {
    fn from(s: String) -> Self {
        PromptContent::Text(s)
    }
}

impl From<&str> for PromptContent {
    fn from(s: &str) -> Self {
        PromptContent::Text(s.to_string())
    }
}

impl From<PathBuf> for PromptContent {
    fn from(p: PathBuf) -> Self {
        PromptContent::File(p)
    }
}

impl fmt::Display for PromptContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromptContent::Text(content) => write!(f, "{}", content),
            PromptContent::File(path) => write!(f, "@{}", path.display()),
        }
    }
}

impl PromptContent {
    pub fn new(content: impl Into<String>) -> Self {
        PromptContent::Text(content.into())
    }

    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Self {
        PromptContent::File(path.as_ref().to_path_buf())
    }
}

/// Represents an AI agent with specific system and user prompts, templated with
/// a context type
#[derive(Clone, Debug, Default, Deserialize, Serialize, Setters)]
#[serde(rename_all = "camelCase")]
#[setters(into, strip_option)]
pub struct Agent<C> {
    /// Name of the agent
    pub name: String,

    /// Description of what the agent does
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional model ID to use for this agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<ModelId>,

    /// The system prompt that defines the agent's behavior
    #[serde(rename = "systemPrompt", skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<PromptContent>,

    /// Optional user prompt template for consistent interactions
    #[serde(rename = "userPrompt", skip_serializing_if = "Option::is_none")]
    pub user_prompt: Option<PromptContent>,

    /// Phantom data for the context type
    #[serde(skip)]
    _context: PhantomData<C>,
}

impl<C> Agent<C>
where
    C: Serialize + DeserializeOwned,
{
    /// Creates a new agent
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            model_id: None,
            system_prompt: None,
            user_prompt: None,
            _context: PhantomData,
        }
    }

    fn render_system_prompt(&self, binding: &C) -> Result<Option<String>> {
        if let Some(system_prompt) = &self.system_prompt {
            let handlebars = Handlebars::new();
            let prompt = system_prompt.to_string();
            Ok(Some(handlebars.render_template(&prompt, binding)?))
        } else {
            Ok(None)
        }
    }

    fn render_user_prompt(&self, binding: &C) -> Result<Option<String>> {
        if let Some(user_prompt) = &self.user_prompt {
            let handlebars = Handlebars::new();
            let prompt = user_prompt.to_string();
            Ok(Some(handlebars.render_template(&prompt, binding)?))
        } else {
            Ok(None)
        }
    }

    /// Converts the agent to a Context by rendering its prompts with the
    /// provided template binding.
    ///
    /// The binding contains values for handlebars template variables in both
    /// system and user prompts. The resulting Context will contain the
    /// rendered prompts as messages in the correct order (system message
    /// first, if present, followed by user message if present).
    ///
    /// # Example
    /// ```rust,ignore
    /// let binding = CodeContext {
    ///     role: "helpful".to_string(),
    ///     language: "Rust".to_string()
    /// };
    /// let context = agent.to_context(&binding)?;
    /// ```
    pub fn to_context(&self, binding: &C) -> Result<Context> {
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(system_message) = self.render_system_prompt(binding)? {
            messages.push(ContextMessage::system(system_message));
        }

        // Add user message if present
        if let Some(user_message) = self.render_user_prompt(binding)? {
            messages.push(ContextMessage::user(user_message));
        }

        let model_id = self.model_id.clone().unwrap_or_default();

        Ok(Context::new(model_id.clone()).extend_messages(messages))
    }
}

/// Loader trait for loading Agent definitions. The loader should be able to
/// load all available agents from a source (e.g., configuration files).
#[async_trait]
pub trait AgentLoader<C>
where
    C: Send + Sync + DeserializeOwned,
{
    /// Load all available agents. Returns a Vec of agents in the order they
    /// were defined in the source.
    async fn load_agents(&self) -> anyhow::Result<Vec<Agent<C>>>;
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct CodeContext {
        role: String,
        language: String,
    }

    use std::path::Path;

    #[test]
    fn test_create_agent() {
        let agent: Agent<CodeContext> = Agent::new("Coder");

        assert_eq!(agent.name, "Coder");
        assert_eq!(agent.description, None);
        assert_eq!(agent.model_id, None);
        assert_eq!(agent.system_prompt, None);
        assert_eq!(agent.user_prompt, None);
    }

    #[test]
    fn test_create_with_prompts_and_model() {
        let agent: Agent<CodeContext> = Agent::new("Coder")
            .description("A coding assistant")
            .system_prompt("You are a helpful coding assistant")
            .model_id(ModelId::new("gpt-4"));

        assert_eq!(agent.name, "Coder");
        assert_eq!(agent.description, Some("A coding assistant".to_string()));
        assert_eq!(agent.model_id, Some(ModelId::new("gpt-4")));
        assert_eq!(
            agent.system_prompt,
            Some(PromptContent::Text(
                "You are a helpful coding assistant".to_string()
            ))
        );
        assert_eq!(agent.user_prompt, None);
    }

    #[test]
    fn test_agent_with_file_prompts() {
        let agent: Agent<CodeContext> = Agent::new("Coder")
            .description("A coding assistant")
            .system_prompt(PromptContent::from_file("prompts/system.md"))
            .user_prompt(PromptContent::from_file("prompts/user.md"));

        assert_eq!(
            agent.system_prompt,
            Some(PromptContent::File(
                Path::new("prompts/system.md").to_path_buf()
            ))
        );
        assert_eq!(
            agent.user_prompt,
            Some(PromptContent::File(
                Path::new("prompts/user.md").to_path_buf()
            ))
        );
    }

    #[test]
    fn test_render_prompts_with_context() {
        let agent: Agent<CodeContext> = Agent::new("Coder")
            .description("A coding assistant")
            .system_prompt("You are a {{role}} coding assistant")
            .user_prompt("How can I help with {{language}} code today?")
            .model_id(ModelId::new("gpt-4"));

        let binding = CodeContext { role: "helpful".to_string(), language: "Rust".to_string() };

        assert_eq!(
            agent.render_system_prompt(&binding).unwrap(),
            Some("You are a helpful coding assistant".to_string())
        );
        assert_eq!(
            agent.render_user_prompt(&binding).unwrap(),
            Some("How can I help with Rust code today?".to_string())
        );
    }

    #[test]
    fn test_to_context_with_model() {
        let agent: Agent<CodeContext> = Agent::new("Coder")
            .description("A coding assistant")
            .system_prompt("You are a {{role}} coding assistant")
            .user_prompt("How can I help with {{language}} code today?")
            .model_id(ModelId::new("gpt-4"));

        let binding = CodeContext { role: "helpful".to_string(), language: "Rust".to_string() };

        let result = agent.to_context(&binding).unwrap();
        assert_eq!(result.model, ModelId::new("gpt-4"));
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.model, ModelId::new("gpt-4"));
        assert_eq!(
            result.messages[0],
            ContextMessage::system("You are a helpful coding assistant")
        );
        assert_eq!(
            result.messages[1],
            ContextMessage::user("How can I help with Rust code today?")
        );
    }

    #[test]
    fn test_to_context_default_model() {
        let agent: Agent<CodeContext> = Agent::new("Coder")
            .system_prompt("You are a {{role}} coding assistant")
            // No model specified, should use default
            ;

        let binding = CodeContext { role: "helpful".to_string(), language: "Rust".to_string() };

        let result = agent.to_context(&binding).unwrap();
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.model, ModelId::default());
        assert_eq!(
            result.messages[0],
            ContextMessage::system("You are a helpful coding assistant")
        );
    }

    #[test]
    fn test_to_context_user_only() {
        let agent: Agent<CodeContext> =
            Agent::new("Coder").user_prompt("How can I help with {{language}} code today?");

        let binding = CodeContext { role: "helpful".to_string(), language: "Rust".to_string() };

        let result = agent.to_context(&binding).unwrap();
        assert_eq!(result.messages.len(), 1);
        assert_eq!(
            result.messages[0],
            ContextMessage::user("How can I help with Rust code today?")
        );
    }

    #[test]
    fn test_to_context_no_prompts() {
        let agent: Agent<CodeContext> = Agent::new("Coder");

        let binding = CodeContext { role: "helpful".to_string(), language: "Rust".to_string() };

        let result = agent.to_context(&binding).unwrap();
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn test_agent_roundtrip() {
        let agent: Agent<CodeContext> = Agent::new("Coder")
            .description("A coding assistant")
            .system_prompt(PromptContent::Text("System prompt".to_string()))
            .user_prompt("User prompt");

        let serialized = serde_json::to_string(&agent).unwrap();
        let deserialized: Agent<CodeContext> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.name, agent.name);
        assert_eq!(deserialized.description, agent.description);
        assert_eq!(deserialized.system_prompt, agent.system_prompt);
        assert_eq!(deserialized.user_prompt, agent.user_prompt);
    }
}
