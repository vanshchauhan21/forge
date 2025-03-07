use thiserror::Error;

use crate::callback::CallbackError;

/// Error types specific to the authentication process
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Failed to start local server: {0}")]
    ServerStartError(String),

    #[error("Failed to exchange code for token: {0}")]
    TokenExchangeError(String),

    #[error("Failed to retrieve user information: {0}")]
    UserInfoError(String),

    #[error("Authorization was denied: {0}")]
    AuthorizationDenied(String),

    #[error("State parameter mismatch")]
    StateMismatch,

    #[error("No code received in callback")]
    NoCodeReceived,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Timeout")]
    Timeout,
    #[error("Invalid ID token")]
    InvalidIDToken,
}

impl From<CallbackError> for AuthError {
    fn from(e: CallbackError) -> Self {
        match e {
            CallbackError::Timeout(_) => AuthError::Timeout,
            CallbackError::ServerError(e) => AuthError::ServerStartError(e),
        }
    }
}
