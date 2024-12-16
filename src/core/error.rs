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
            error: ProviderError::EmptyResponse,
        }
    }
}

#[derive(Debug)]
pub enum ProviderError {
    // Custom display message for OpenAI error
    OpenAI(OpenAIError),

    // Custom display message for EmptyResponse
    EmptyResponse,
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
