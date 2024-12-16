use async_openai::error::OpenAIError;
use derive_more::derive::{Display, From};

#[derive(From, Display, Debug)]
pub enum Error {
    // Custom display message for OpenAI error
    #[display("OpenAI Error: {}", _0)]
    OpenAI(OpenAIError),

    // Custom display message for EmptyResponse
    #[display("Empty Response from provider: {}", _0)]
    EmptyResponse(String),
}

pub type Result<T> = std::result::Result<T, Error>;
