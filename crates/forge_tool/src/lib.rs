mod ask;
mod fs;
mod outline;
mod shell;
mod think;
mod tool_engine;

pub use tool_engine::*;

#[async_trait::async_trait]
trait ToolCallService {
    type Input;
    type Output;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String>;
}

trait Description {
    fn description() -> &'static str;
}
