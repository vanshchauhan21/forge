use anyhow::Result;
use reqwest::Url;

#[derive(Clone)]
pub enum Provider {
    OpenAI(Url),
    OpenRouter(Url),
}

impl Provider {
    pub fn is_openai(&self) -> bool {
        matches!(self, Self::OpenAI(_))
    }

    pub fn is_open_router(&self) -> bool {
        matches!(self, Self::OpenRouter(_))
    }

    pub fn parse(base_url: &str) -> Result<Self> {
        match base_url {
            "https://api.openai.com/v1/" => Ok(Self::OpenAI(Url::parse(base_url)?)),
            "https://openrouter.ai/api/v1/" => Ok(Self::OpenRouter(Url::parse(base_url)?)),
            _ => Err(anyhow::anyhow!("Provider not supported yet!")),
        }
    }

    pub fn base_url(&self) -> &Url {
        match self {
            Self::OpenAI(url) | Self::OpenRouter(url) => url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_parser() {
        let open_ai_provider = Provider::parse("https://api.openai.com/v1/");
        assert!(open_ai_provider.is_ok());
        assert!(matches!(open_ai_provider.unwrap(), Provider::OpenAI(_)));

        let open_router_provider = Provider::parse("https://openrouter.ai/api/v1/");
        assert!(open_router_provider.is_ok());
        assert!(matches!(
            open_router_provider.unwrap(),
            Provider::OpenRouter(_)
        ));

        let groq_provider = Provider::parse("https://groq.com/api/v1/");
        assert!(groq_provider.is_err());
    }
}
