use std::sync::{Arc, Mutex};

use forge_provider::ResultStream;
use tokio_stream::StreamExt;

pub trait Application: Sized + Default + Clone {
    type Action;
    type Error;
    type Command;
    fn update(
        self,
        action: Self::Action,
    ) -> std::result::Result<(Self, Self::Command), Self::Error>;
}

pub struct ApplicationRuntime<A: Application> {
    app: A,
    executor: Box<dyn Executor<Command = A::Command, Action = A::Action, Error = A::Error>>,
    state: Arc<Mutex<A>>,
}

impl<A: Application> ApplicationRuntime<A> {
    pub async fn execute(&self, a: A::Action) -> std::result::Result<(), A::Error> {
        let mut guard = self.state.lock().unwrap();
        let app = guard.clone();
        let (app, command) = app.update(a)?;
        *guard = app;
        let mut stream = self.executor.execute(&command).await?;

        while let Some(result) = stream.next().await {
            self.execute(result?).await?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait Executor {
    type Command;
    type Action;
    type Error;
    async fn execute(&self, command: &Self::Command) -> ResultStream<Self::Action, Self::Error>;
}
