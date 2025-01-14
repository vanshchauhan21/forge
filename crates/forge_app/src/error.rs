use std::fmt::Debug;

use derive_more::derive::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, derive_more::From)]
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
