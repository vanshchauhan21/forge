use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Missing command parameter: {0}")]
    MissingParameter(String),

    #[error("Unsupported command parameter: {0}")]
    UnsupportedParameter(String),

    #[error("Invalid argument: {0}")]
    MissingParameterValue(String),
}

pub type Result<A> = std::result::Result<A, Error>;
