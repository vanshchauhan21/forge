use axum::response::{IntoResponse, Response};
use derive_more::derive::{Debug, From};
use rig::completion::PromptError;

use crate::cause::Cause;

#[derive(Debug, From)]
pub enum Error {
    IO {
        error: tokio::io::Error,
        resource: String,
    },
    Axum {
        error: axum::Error,
    },
    Serde {
        error: serde_json::Error,
    },
    Prompt {
        error: PromptError,
    },
}

pub type Result<A> = std::result::Result<A, Error>;

impl From<Error> for Cause {
    fn from(value: Error) -> Self {
        match value {
            Error::IO { error, resource } => {
                Cause::new(format!("IO Error: {}", resource)).cause(Cause::new(error.to_string()))
            }
            Error::Axum { error } => Cause::new("Axum Error").cause(Cause::new(error.to_string())),
            Error::Serde { error } => {
                Cause::new("Serde Error").cause(Cause::new(error.to_string()))
            }
            Error::Prompt { error } => {
                Cause::new("Prompt Error").cause(Cause::new(error.to_string()))
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Cause::from(self).into_response()
    }
}
