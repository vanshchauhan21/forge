mod console;
mod fs;
pub mod shell;
#[allow(unused)]
mod mcp;
mod router;
mod think;
pub use router::*;

#[async_trait::async_trait]
pub(crate) trait ToolTrait {
    type Input;
    type Output;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String>;
}

trait Description {
    fn description() -> &'static str;
}
