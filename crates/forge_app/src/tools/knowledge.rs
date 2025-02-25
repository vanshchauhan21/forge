use std::sync::Arc;

use forge_domain::{
    ExecutableTool, NamedTool, Suggestion, SuggestionService, ToolDescription, ToolName,
};
use schemars::JsonSchema;

pub struct RecallSuggestions<F> {
    infra: Arc<F>,
}

impl<F> ToolDescription for RecallSuggestions<F> {
    fn description(&self) -> String {
        "Get Suggestion from the app".to_string()
    }
}

impl<F> RecallSuggestions<F> {
    pub fn new(infra: Arc<F>) -> Self {
        Self { infra }
    }
}

#[derive(serde::Deserialize, JsonSchema)]
pub struct GetSuggestionInput {
    pub query: String,
}

#[async_trait::async_trait]
impl<F: SuggestionService> ExecutableTool for RecallSuggestions<F> {
    type Input = GetSuggestionInput;

    async fn call(&self, input: Self::Input) -> anyhow::Result<String> {
        let out = self
            .infra
            .search(&input.query)
            .await?
            .into_iter()
            .fold(String::new(), |a, b| format!("{}\n{}", a, b.suggestion));

        Ok(out.trim().to_string())
    }
}

impl<F> NamedTool for RecallSuggestions<F> {
    fn tool_name() -> ToolName {
        ToolName::new("tool_forge_suggestion_get".to_string())
    }
}

pub struct StoreSuggestion<F> {
    suggestion: Arc<F>,
}

impl<F> StoreSuggestion<F> {
    pub fn new(suggestion: Arc<F>) -> Self {
        Self { suggestion }
    }
}

impl<F> ToolDescription for StoreSuggestion<F> {
    fn description(&self) -> String {
        "Set suggestions".to_string()
    }
}

#[derive(serde::Deserialize, JsonSchema)]
pub struct StoreSuggestionInput {
    /// The use case where the suggestion is applicable.
    pub use_case: String,

    /// The suggestion for the above use-case
    pub suggestion: String,
}

#[async_trait::async_trait]
impl<F: SuggestionService> ExecutableTool for StoreSuggestion<F> {
    type Input = StoreSuggestionInput;

    async fn call(&self, input: Self::Input) -> anyhow::Result<String> {
        self.suggestion
            .insert(Suggestion { use_case: input.use_case, suggestion: input.suggestion })
            .await?;

        Ok("Suggestion stored".to_string())
    }
}

impl<F> NamedTool for StoreSuggestion<F> {
    fn tool_name() -> ToolName {
        ToolName::new("tool_forge_suggestion_set".to_string())
    }
}
