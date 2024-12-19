use error::Error;
use forge_engine::*;
use model::{Action, Command};

pub struct Runtime {}

#[async_trait::async_trait]
impl forge_engine::Runtime for Runtime {
    async fn run(&self, command: Command) -> Result<Action, Error> {
        todo!()
    }
}
