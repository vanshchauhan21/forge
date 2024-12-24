mod fs;

#[allow(unused)]
mod mcp;
mod outline;
mod router;
mod shell;
mod think;
mod ask;

pub use fs::*;
pub use outline::*;
pub use router::*;
pub use shell::*;
pub use ask::*;

#[async_trait::async_trait]
pub trait ToolTrait {
    type Input;
    type Output;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String>;
}

pub trait Description {
    fn description() -> &'static str;
}
