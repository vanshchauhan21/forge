use async_openai::error::OpenAIError;
use derive_more::derive::{Display, From};

#[derive(From, Debug, Display)]
pub enum Error {
    OpenAI(OpenAIError),

    // TODO: add name of the provider in this error
    EmptyResponse,
}

pub type Result<T> = std::result::Result<T, Error>;
