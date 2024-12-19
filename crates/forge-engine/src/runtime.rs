use crate::{error::Error, model::Command, ActionStream};

#[async_trait::async_trait]
pub trait Runtime {
    async fn run(&self, command: Command) -> Result<ActionStream, Error>;
}
