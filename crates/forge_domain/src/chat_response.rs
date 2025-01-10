use serde::Serialize;

use crate::{Context, ConversationId, Errata, ToolCallFull, ToolName, ToolResult};

/// Events that are emitted by the agent for external consumption. This includes
/// events for all internal state changes.
#[derive(Debug, Clone, Serialize, PartialEq, derive_more::From)]
#[serde(rename_all = "camelCase")]
pub enum ChatResponse {
    #[from(ignore)]
    Text(String),
    ToolCallDetected(ToolName),
    ToolCallArgPart(String),
    ToolCallStart(ToolCallFull),
    ToolCallEnd(ToolResult),
    ConversationStarted(ConversationId),
    ModifyContext(Context),
    Complete,
    #[from(ignore)]
    PartialTitle(String),
    #[from(ignore)]
    CompleteTitle(String),
    Error(Errata),
}
