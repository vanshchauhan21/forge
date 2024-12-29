//! The model is built on top of Open AI's API specification
//! Specification: https://platform.openai.com/docs/api-reference/chat/create

mod message;
mod request;
mod response;
mod tool;

pub use message::*;
pub use request::*;
pub use response::*;
pub use tool::*;
