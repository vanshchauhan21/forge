use std::fmt::{Debug, Display, Formatter};

use derive_more::derive::Display;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};

#[derive(Display, derive_more::From)]
pub enum Error {
    Diesel(diesel::result::Error),
    DieselConnection(diesel::ConnectionError),
    DieselR2D2(diesel::r2d2::Error),
    Domain(forge_domain::Error),
    EmptyResponse,
    Handlebars(handlebars::RenderError),
    IO(std::io::Error),
    Provider(forge_provider::Error),
    R2D2(r2d2::Error),
    Serde(serde_json::Error),
    StdError(Box<dyn std::error::Error + Send + Sync>),
    ToolCallMissingName,
    Var(std::env::VarError),
    Walk(forge_walker::Error),
}

impl Error {
    pub fn from_std_error<T: std::error::Error + Send + Sync + 'static>(err: T) -> Self {
        Error::StdError(Box::new(err))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: InnerErrorResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InnerErrorResponse {
    pub code: u32,
    pub message: String,
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&Errata::from(self), f)
    }
}

#[derive(Clone, Setters, Serialize, PartialEq, Eq)]
pub struct Errata {
    pub message: String,
    #[setters(strip_option)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u32>,
    #[setters(strip_option, into)]
    pub description: Option<String>,
}

impl Errata {
    pub fn new(title: impl Into<String>) -> Self {
        Self { message: title.into(), description: None, code: None }
    }
}

impl Display for Errata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Debug for Errata {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(desc) = &self.description {
            if !desc.trim().is_empty() {
                write!(f, "\n{}", desc)?;
            }
        }
        Ok(())
    }
}

impl From<&Error> for Errata {
    fn from(error: &Error) -> Self {
        match error {
            Error::Provider(forge_provider::Error::Provider {
                provider,
                error: forge_provider::ProviderError::UpstreamError(value),
            }) => {
                let value: ErrorResponse = serde_json::from_value(value.clone()).unwrap();
                Self {
                    message: format!("{}: {}", provider, value.error.message),
                    code: Some(value.error.code),
                    description: value
                        .error
                        .metadata
                        .map(|m| serde_json::to_string(&m).unwrap()),
                }
            }
            _ => Errata::new(error.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_simple_error() {
        let error = Errata::new("Something went wrong");
        assert_eq!(format!("{:?}", error), "Something went wrong");
    }

    #[test]
    fn test_error_with_description() {
        let error = Errata::new("Invalid input").description("Expected a number");
        assert_eq!(format!("{:?}", error), "Invalid input\nExpected a number");
    }
}
