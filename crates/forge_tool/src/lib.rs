mod ask;
mod fs;
mod outline;
mod shell;
mod think;
mod tool_engine;

pub use ask::*;
pub use fs::*;
pub use outline::*;
pub use shell::*;
pub use tool_engine::*;

#[async_trait::async_trait]
pub trait ToolTrait {
    type Input;
    type Output;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String>;
}

pub trait Description {
    fn description() -> &'static str;
}
