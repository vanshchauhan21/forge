use async_openai::error::OpenAIError;
use derive_more::derive::{Display, From};

#[derive(From, Debug, Display)]
pub enum Error {
    OpenAI(OpenAIError)
}

pub type Result<T> = std::result::Result<T, Error>;