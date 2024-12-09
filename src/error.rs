use axum::response::{IntoResponse, Response};
use derive_more::derive::{Debug, From};

use crate::cause::Cause;

#[derive(Debug, From)]
pub enum Error {
    IO {
        error: tokio::io::Error,
        resource: String,
    },
}

pub type Result<A> = std::result::Result<A, Error>;

impl From<Error> for Cause {
    fn from(value: Error) -> Self {
        match value {
            Error::IO { error, resource } => {
                Cause::new(format!("IO Error: {}", resource)).cause(Cause::new(error.to_string()))
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Cause::from(self).into_response()
    }
}
