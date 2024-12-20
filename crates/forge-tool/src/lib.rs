mod console;
mod fs;
mod mcp;
pub mod model;
mod prompt_parser;
mod router;
mod think;
pub use model::*;
pub use router::*;

#[async_trait::async_trait]
pub(crate) trait ToolTrait {
    type Input;
    type Output;

    fn description(&self) -> String;
    async fn call(&self, input: Self::Input) -> Result<Self::Output, String>;
}

trait Description {
    fn description() -> &'static str;
}
