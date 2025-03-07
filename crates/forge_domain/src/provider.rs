use std::fmt::Display;

use derive_more::derive::From;
use serde::{Deserialize, Serialize};
use url::Url;

const OPEN_ROUTER_URL: &str = "https://openrouter.ai/api/v1/";
const OPENAI_URL: &str = "https://api.openai.com/v1/";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/";
const ANTINOMY_URL: &str = "https://antinomy.ai/api/v1/";

/// OpenAI Compatible providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OpenAiCompat {
    OpenRouter,
    OpenAI,
    Antinomy,
}

impl OpenAiCompat {
    pub fn to_base_url(&self) -> Url {
        match self {
            OpenAiCompat::OpenRouter => Url::parse(OPEN_ROUTER_URL).unwrap(),
            OpenAiCompat::OpenAI => Url::parse(OPENAI_URL).unwrap(),
            OpenAiCompat::Antinomy => Url::parse(ANTINOMY_URL).unwrap(),
        }
    }
}

/// Providers that can be used.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, From)]
pub enum Provider {
    OpenAiCompat(OpenAiCompat),
    Anthropic,
}

impl Display for OpenAiCompat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAiCompat::OpenRouter => write!(f, "OpenRouter"),
            OpenAiCompat::OpenAI => write!(f, "OpenAI"),
            OpenAiCompat::Antinomy => write!(f, "Antinomy"),
        }
    }
}

impl Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::OpenAiCompat(compat) => write!(f, "{}", compat),
            Provider::Anthropic => write!(f, "Anthropic"),
        }
    }
}

impl Provider {
    // detects the active provider from environment variables
    pub fn from_env() -> Option<Self> {
        match (
            std::env::var("FORCE_ANTINOMY"),
            std::env::var("FORGE_KEY"),
            std::env::var("OPENROUTER_API_KEY"),
            std::env::var("OPENAI_API_KEY"),
            std::env::var("ANTHROPIC_API_KEY"),
        ) {
            (Ok(a), _, _, _, _) if a == "true" => Self::from_url(ANTINOMY_URL),

            (_, Ok(_), _, _, _) => {
                // note: if we're using FORGE_KEY, we need FORGE_PROVIDER_URL to be set.
                let provider_url = std::env::var("FORGE_PROVIDER_URL").ok()?;
                Self::from_url(&provider_url)
            }
            (_, _, Ok(_), _, _) => Some(Self::OpenAiCompat(OpenAiCompat::OpenRouter)),
            (_, _, _, Ok(_), _) => Some(Self::OpenAiCompat(OpenAiCompat::OpenAI)),
            (_, _, _, _, Ok(_)) => Some(Self::Anthropic),
            (Ok(a), _, _, _, _) if a == "false" => None,
            (Ok(_), Err(_), Err(_), Err(_), Err(_)) => None,
            (Err(_), Err(_), Err(_), Err(_), Err(_)) => None,
        }
    }

    /// converts the provider to it's base URL
    pub fn to_base_url(&self) -> Url {
        match self {
            Provider::OpenAiCompat(compat) => compat.to_base_url(),
            Provider::Anthropic => Url::parse(ANTHROPIC_URL).unwrap(),
        }
    }

    /// detects the active provider from base URL
    pub fn from_url(url: &str) -> Option<Self> {
        match url {
            OPENAI_URL => Some(Self::OpenAiCompat(OpenAiCompat::OpenAI)),
            OPEN_ROUTER_URL => Some(Self::OpenAiCompat(OpenAiCompat::OpenRouter)),
            ANTHROPIC_URL => Some(Self::Anthropic),
            ANTINOMY_URL => Some(Self::OpenAiCompat(OpenAiCompat::Antinomy)),
            _ => None,
        }
    }

    pub fn is_open_router(&self) -> bool {
        matches!(self, Self::OpenAiCompat(OpenAiCompat::OpenRouter))
    }

    pub fn is_open_ai(&self) -> bool {
        matches!(self, Self::OpenAiCompat(OpenAiCompat::OpenAI))
    }
}
