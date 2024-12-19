use crate::{
    error::Error,
    state::{Action, Command},
};

#[async_trait::async_trait]
pub trait Runtime {
    async fn run(&self, command: Command) -> Result<Action, Error>;
}
