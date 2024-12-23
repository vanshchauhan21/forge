mod fs;

#[allow(unused)]
mod mcp;
mod outline;
mod router;
mod shell;
mod think;

pub use fs::*;
pub use outline::*;
pub use router::*;
pub use shell::*;

#[async_trait::async_trait]
pub(crate) trait ToolTrait {
    type Input;
    type Output;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String>;
}

trait Description {
    fn description() -> &'static str;
}
