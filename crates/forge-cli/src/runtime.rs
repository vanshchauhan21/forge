use error::Error;
use forge_engine::*;
use state::{Action, Command};

#[derive(Default)]
pub struct Runtime {}

#[async_trait::async_trait]
impl forge_engine::Runtime for Runtime {
    async fn run(&self, command: Command) -> Result<Action, Error> {
        todo!()
    }
}
