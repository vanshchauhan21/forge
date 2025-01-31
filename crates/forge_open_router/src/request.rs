use derive_more::derive::Display;
use derive_setters::Setters;
use forge_domain::{
    Context, ContextMessage, ModelId, Role, ToolCallFull, ToolCallId, ToolDefinition, ToolName,
};
use serde::{Deserialize, Serialize};

use super::response::{FunctionCall, OpenRouterToolCall};
use super::tool_choice::{FunctionType, ToolChoice};

// NOTE: only some of the anthropic models support caching.
const CLAUDE_CACHE_SUPPORTED_MODELS: &[&str] = &[
    "anthropic/claude-3.5-sonnet",
    "anthropic/claude-3.5-haiku",
    "anthropic/claude-3-haiku",
    "anthropic/claude-3-opus",
];

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TextContent {
    // TODO: could be an enum
    pub r#type: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageContentPart {
    pub r#type: String,
    pub image_url: ImageUrl,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenRouterMessage {
    pub role: OpenRouterRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ToolName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<ToolCallId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenRouterToolCall>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

impl MessageContent {
    pub fn cached(self) -> Self {
        match self {
            MessageContent::Text(text) => MessageContent::Parts(vec![ContentPart::Text {
                text,
                cache_control: Some(CacheControl { type_: CacheControlType::Ephemeral }),
            }]),
            _ => self,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    ImageUrl {
        image_url: ImageUrl,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub type_: CacheControlType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CacheControlType {
    Ephemeral,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionDescription {
    pub description: Option<String>,
    pub name: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenRouterTool {
    // TODO: should be an enum
    pub r#type: FunctionType,
    pub function: FunctionDescription,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponseFormat {
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Prediction {
    pub r#type: String,
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProviderPreferences {
    // Define fields as necessary
}

#[derive(Debug, Deserialize, Serialize, Clone, Setters)]
#[setters(strip_option)]
pub struct OpenRouterRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<OpenRouterMessage>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenRouterTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<std::collections::HashMap<u32, f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_a: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<Prediction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<Transform>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,
}

/// ref: https://openrouter.ai/docs/transforms
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub enum Transform {
    #[default]
    #[serde(rename = "middle-out")]
    MiddleOut,
}

impl From<ToolDefinition> for OpenRouterTool {
    fn from(value: ToolDefinition) -> Self {
        OpenRouterTool {
            r#type: FunctionType,
            function: FunctionDescription {
                description: Some(value.description),
                name: value.name.into_string(),
                parameters: serde_json::to_value(value.input_schema).unwrap(),
            },
        }
    }
}

impl From<Context> for OpenRouterRequest {
    fn from(request: Context) -> Self {
        OpenRouterRequest {
            messages: {
                let messages = request
                    .messages
                    .into_iter()
                    .map(OpenRouterMessage::from)
                    .collect::<Vec<_>>();

                Some(messages)
            },
            tools: {
                let tools = request
                    .tools
                    .into_iter()
                    .map(OpenRouterTool::from)
                    .collect::<Vec<_>>();
                if tools.is_empty() {
                    None
                } else {
                    Some(tools)
                }
            },
            model: None,
            prompt: Default::default(),
            response_format: Default::default(),
            stop: Default::default(),
            stream: Default::default(),
            max_tokens: Default::default(),
            temperature: Default::default(),
            tool_choice: request.tool_choice.map(|tc| tc.into()),
            seed: Default::default(),
            top_p: Default::default(),
            top_k: Default::default(),
            frequency_penalty: Default::default(),
            presence_penalty: Default::default(),
            repetition_penalty: Default::default(),
            logit_bias: Default::default(),
            top_logprobs: Default::default(),
            min_p: Default::default(),
            top_a: Default::default(),
            prediction: Default::default(),
            transforms: Some(vec![Transform::default()]),
            models: Default::default(),
            route: Default::default(),
            provider: Default::default(),
        }
    }
}

impl From<ToolCallFull> for OpenRouterToolCall {
    fn from(value: ToolCallFull) -> Self {
        Self {
            id: value.call_id,
            r#type: FunctionType,
            function: FunctionCall {
                arguments: serde_json::to_string(&value.arguments).unwrap(),
                name: Some(value.name),
            },
        }
    }
}

impl From<ContextMessage> for OpenRouterMessage {
    fn from(value: ContextMessage) -> Self {
        match value {
            ContextMessage::ContentMessage(chat_message) => OpenRouterMessage {
                role: chat_message.role.into(),
                content: Some(MessageContent::Text(chat_message.content)),
                name: None,
                tool_call_id: None,
                tool_calls: chat_message.tool_calls.map(|tool_calls| {
                    tool_calls
                        .into_iter()
                        .map(OpenRouterToolCall::from)
                        .collect()
                }),
            },
            ContextMessage::ToolMessage(tool_result) => OpenRouterMessage {
                role: OpenRouterRole::Tool,
                content: Some(MessageContent::Text(tool_result.to_string())),
                name: Some(tool_result.name),
                tool_call_id: tool_result.call_id,
                tool_calls: None,
            },
        }
    }
}

impl OpenRouterRequest {
    /// Inserts cache control information into the last system or user message
    /// if model supports it.
    /// NOTE: This helps reduce context window usage
    /// by caching only the most recent system/user message
    pub fn cache(mut self) -> Self {
        if let (Some(mut messages), Some(model)) = (self.messages.take(), self.model.take()) {
            let model_id = model.as_str();
            let should_cache = !model_id.contains("anthropic")
                || CLAUDE_CACHE_SUPPORTED_MODELS
                    .iter()
                    .any(|m| model_id.contains(m));

            if should_cache {
                if let Some(msg) = messages
                    .iter_mut()
                    .rev()
                    .find(|msg| matches!(msg.role, OpenRouterRole::System | OpenRouterRole::User))
                {
                    msg.content = msg.content.take().map(|content| content.cached());
                }
            }

            self.messages = Some(messages);
            self.model = Some(model);
        }
        self
    }
}

impl From<Role> for OpenRouterRole {
    fn from(role: Role) -> Self {
        match role {
            Role::System => OpenRouterRole::System,
            Role::User => OpenRouterRole::User,
            Role::Assistant => OpenRouterRole::Assistant,
        }
    }
}

#[derive(Debug, Deserialize, Display, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OpenRouterRole {
    System,
    User,
    Assistant,
    Tool,
}

#[cfg(test)]
mod tests {
    use forge_domain::{
        ContentMessage, ContextMessage, Role, ToolCallFull, ToolCallId, ToolName, ToolResult,
    };
    use insta::assert_json_snapshot;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_user_message_conversion() {
        let user_message = ContextMessage::ContentMessage(ContentMessage {
            role: Role::User,
            content: "Hello".to_string(),
            tool_calls: None,
        });
        let router_message = OpenRouterMessage::from(user_message);
        assert_json_snapshot!(router_message);
    }

    #[test]
    fn test_message_with_special_chars() {
        let xml_content = r#"Here's some XML content:
<task>
    <id>123</id>
    <description>Test <special> characters</description>
    <data key="value">
        <item>1</item>
        <item>2</item>
    </data>
</task>"#;

        let message = ContextMessage::ContentMessage(ContentMessage {
            role: Role::User,
            content: xml_content.to_string(),
            tool_calls: None,
        });
        let router_message = OpenRouterMessage::from(message);
        assert_json_snapshot!(router_message);
    }

    #[test]
    fn test_assistant_message_with_tool_call_conversion() {
        let tool_call = ToolCallFull {
            call_id: Some(ToolCallId::new("123")),
            name: ToolName::new("test_tool"),
            arguments: json!({"key": "value"}),
        };

        let assistant_message = ContextMessage::ContentMessage(ContentMessage {
            role: Role::Assistant,
            content: "Using tool".to_string(),
            tool_calls: Some(vec![tool_call]),
        });
        let router_message = OpenRouterMessage::from(assistant_message);
        assert_json_snapshot!(router_message);
    }

    #[test]
    fn test_tool_message_conversion() {
        let tool_result = ToolResult::new(ToolName::new("test_tool"))
            .call_id(ToolCallId::new("123"))
            .success(
                r#"{
               "user": "John",
               "age": 30,
               "address": [{"city": "New York"}, {"city": "San Francisco"}]
            }"#,
            );

        let tool_message = ContextMessage::ToolMessage(tool_result);
        let router_message = OpenRouterMessage::from(tool_message);
        assert_json_snapshot!(router_message);
    }

    #[test]
    fn test_tool_message_with_special_chars() {
        let tool_result = ToolResult::new(ToolName::new("html_tool"))
            .call_id(ToolCallId::new("456"))
            .success(
                r#"{
                "html": "<div class=\"container\"><p>Hello <World></p></div>",
                "elements": ["<span>", "<br/>", "<hr>"],
                "attributes": {
                    "style": "color: blue; font-size: 12px;",
                    "data-test": "<test>&value</test>"
                }
            }"#,
            );

        let tool_message = ContextMessage::ToolMessage(tool_result);
        let router_message = OpenRouterMessage::from(tool_message);
        assert_json_snapshot!(router_message);
    }

    #[test]
    fn test_tool_message_typescript_code() {
        let tool_result = ToolResult::new(ToolName::new("rust_tool"))
            .call_id(ToolCallId::new("456"))
            .success(r#"{ "code": "fn main<T>(gt: T) {let b = &gt; }"}"#);

        let tool_message = ContextMessage::ToolMessage(tool_result);
        let router_message = OpenRouterMessage::from(tool_message);
        assert_json_snapshot!(router_message);
    }

    #[test]
    fn test_message_caching() {
        let context = Context {
            messages: vec![
                ContextMessage::ContentMessage(ContentMessage {
                    role: Role::System,
                    content: "First system message".to_string(),
                    tool_calls: None,
                }),
                ContextMessage::ContentMessage(ContentMessage {
                    role: Role::User,
                    content: "Last user message".to_string(),
                    tool_calls: None,
                }),
                ContextMessage::ContentMessage(ContentMessage {
                    role: Role::Assistant,
                    content: "Assistant message".to_string(),
                    tool_calls: None,
                }),
            ],
            tools: vec![],
            tool_choice: None,
        };

        let request = OpenRouterRequest::from(context)
            .model(ModelId::new("anthropic/claude-3.5-sonnet"))
            .cache();
        let messages = request.messages.unwrap();

        // Verify first system message is NOT cached (it's not the last system/user
        // message)
        if let Some(MessageContent::Text(_)) = &messages[0].content {
            // System message should be plain text (not cached)
        } else {
            panic!("First system message should not be cached");
        }

        // Verify last user message IS cached
        if let Some(MessageContent::Parts(parts)) = &messages[1].content {
            assert!(matches!(
                parts[0],
                ContentPart::Text { cache_control: Some(_), .. }
            ));
        } else {
            panic!("Last user message should be cached");
        }

        // Verify assistant message is not cached
        if let Some(MessageContent::Text(_)) = &messages[2].content {
            // Assistant message remains as Text, not converted to Parts with
            // caching
        } else {
            panic!("Assistant message should not be cached");
        }

        // Test with only system messages
        let context_system_only = Context {
            messages: vec![
                ContextMessage::ContentMessage(ContentMessage {
                    role: Role::System,
                    content: "First system message".to_string(),
                    tool_calls: None,
                }),
                ContextMessage::ContentMessage(ContentMessage {
                    role: Role::System,
                    content: "Last system message".to_string(),
                    tool_calls: None,
                }),
            ],
            tools: vec![],
            tool_choice: None,
        };

        let request = OpenRouterRequest::from(context_system_only)
            .model(ModelId::new("anthropic/claude-3.5-sonnet"))
            .cache();
        let messages = request.messages.unwrap();

        // Verify first system message is NOT cached
        if let Some(MessageContent::Text(_)) = &messages[0].content {
            // First system message should be plain text (not cached)
        } else {
            panic!("First system message should not be cached");
        }

        // Verify last system message IS cached
        if let Some(MessageContent::Parts(parts)) = &messages[1].content {
            assert!(matches!(
                parts[0],
                ContentPart::Text { cache_control: Some(_), .. }
            ));
        } else {
            panic!("Last system message should be cached");
        }
    }

    #[test]
    fn test_should_not_cache_when_model_doesnt_support() {
        let context = Context {
            messages: vec![
                ContextMessage::ContentMessage(ContentMessage {
                    role: Role::System,
                    content: "First system message".to_string(),
                    tool_calls: None,
                }),
                ContextMessage::ContentMessage(ContentMessage {
                    role: Role::User,
                    content: "Last user message".to_string(),
                    tool_calls: None,
                }),
            ],
            tools: vec![],
            tool_choice: None,
        };

        let request = OpenRouterRequest::from(context)
            .model(ModelId::new("anthropic/claude-3-sonnet"))
            .cache();

        let messages = request.messages.unwrap();
        for msg in messages {
            assert!(matches!(msg.content, Some(MessageContent::Text(_))));
        }
    }

    #[test]
    fn test_transform_display() {
        assert_eq!(
            serde_json::to_string(&Transform::MiddleOut).unwrap(),
            "\"middle-out\""
        );
    }
}
