use std::sync::Arc;

use forge_provider::ResultStream;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

pub trait Application: Send + Sync + Sized + Clone {
    type Action: Send;
    type Error: Send;
    type Command: Send;
    fn update(
        self,
        action: Self::Action,
    ) -> std::result::Result<(Self, Self::Command), Self::Error>;
}

pub struct ApplicationRuntime<A: Application> {
    state: Arc<Mutex<A>>,
}

impl<A: Application> ApplicationRuntime<A> {
    pub fn new(app: A) -> Self {
        Self { state: Arc::new(Mutex::new(app)) }
    }

    pub async fn state(&self) -> A {
        self.state.lock().await.clone()
    }
}

impl<A: Application> ApplicationRuntime<A> {
    #[async_recursion::async_recursion]
    pub async fn execute(
        &self,
        a: A::Action,
        executor: &impl Executor<Command = A::Command, Action = A::Action, Error = A::Error>,
    ) -> std::result::Result<(), A::Error> {
        let mut guard = self.state.lock().await;
        let app = guard.clone();
        let (app, command) = app.update(a)?;
        *guard = app;
        let mut stream = executor.execute(&command).await?;

        while let Some(result) = stream.next().await {
            self.execute(result?, executor).await?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait Executor: Send + Sync {
    type Command;
    type Action;
    type Error;
    async fn execute(&self, command: &Self::Command) -> ResultStream<Self::Action, Self::Error>;
}
