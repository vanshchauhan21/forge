mod api_service;
mod chat_service;
mod completion_service;
mod neo_chat_service;
mod system_prompt_service;
pub use api_service::*;
pub use chat_service::*;
pub use completion_service::*;
pub use system_prompt_service::*;

pub struct Service;
