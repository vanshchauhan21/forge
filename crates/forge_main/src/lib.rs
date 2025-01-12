pub mod console;
pub mod input;
mod normalize;
pub mod status;

pub use console::CONSOLE;
pub use input::UserInput;
pub use status::{StatusDisplay, StatusKind};
