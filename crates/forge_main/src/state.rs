use forge_api::{ConversationId, Usage};
use serde::Deserialize;

use crate::prompt::ForgePrompt;

// TODO: convert to a new type
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Plan,
    Help,
    #[default]
    Act,
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Plan => write!(f, "PLAN"),
            Mode::Help => write!(f, "HELP"),
            Mode::Act => write!(f, "ACT"),
        }
    }
}

/// State information for the UI
#[derive(Default, Clone)]
pub struct UIState {
    pub current_title: Option<String>,
    pub conversation_id: Option<ConversationId>,
    pub usage: Usage,
    pub mode: Mode,
    pub is_first: bool,
}

impl UIState {
    pub fn new(mode: Mode) -> Self {
        Self {
            current_title: Default::default(),
            conversation_id: Default::default(),
            usage: Default::default(),
            mode,
            is_first: true,
        }
    }
}

impl From<UIState> for ForgePrompt {
    fn from(state: UIState) -> Self {
        ForgePrompt {
            title: state.current_title,
            usage: Some(state.usage),
            mode: state.mode,
        }
    }
}
