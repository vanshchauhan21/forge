use async_openai::error::OpenAIError;

#[derive(Debug)]
pub enum Error {
    // Custom display message for provider error
    Provider {
        provider: String,
        error: ProviderError,
    },
}

impl Error {
    pub fn empty_response(provider: impl Into<String>) -> Self {
        Error::Provider {
            provider: provider.into(),
            error: ProviderError::EmptyResponse("No content received".to_string()),
        }
    }
}

#[derive(Debug)]
pub enum ProviderError {
    // Custom display message for OpenAI error
    OpenAI(OpenAIError),

    // Custom display message for EmptyResponse
    EmptyResponse(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::OpenAI(err) => write!(f, "{}", err),
            ProviderError::EmptyResponse(msg) => write!(f, "{}", msg),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<OpenAIError> for Error {
    fn from(error: OpenAIError) -> Self {
        Error::Provider {
            provider: "OpenAI".to_string(),
            error: ProviderError::OpenAI(error),
        }
    }
}
