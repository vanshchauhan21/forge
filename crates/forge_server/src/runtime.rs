use std::sync::Arc;

use forge_provider::ResultStream;
use futures::future::join_all;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

pub trait Application: Send + Sync + Sized + Clone {
    type Action: Send;
    type Error: Send;
    type Command: Send;
    fn run(
        self,
        action: impl Into<Self::Action>,
    ) -> std::result::Result<(Self, Vec<Self::Command>), Self::Error>;

    #[allow(unused)]
    fn run_seq(self, actions: Vec<Self::Action>) -> Result<(Self, Vec<Self::Command>), Self::Error>
    where
        Self::Action: Clone,
    {
        let mut this = self;
        let mut commands = Vec::new();
        for action in actions {
            let (s, c) = this.run(action.clone())?;
            this = s;
            commands.extend(c)
        }

        Ok((this, commands))
    }
}

#[derive(Clone)]
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

impl<A: Application + 'static> ApplicationRuntime<A> {
    #[async_recursion::async_recursion]
    pub async fn execute<'a>(
        &'a self,
        action: A::Action,
        executor: Arc<
            impl Executor<Command = A::Command, Action = A::Action, Error = A::Error> + 'static,
        >,
    ) -> std::result::Result<(), A::Error> {
        let mut guard = self.state.lock().await;
        let app = guard.clone();
        let (app, commands) = app.run(action)?;
        *guard = app;
        drop(guard);

        join_all(commands.into_iter().map(|command| {
            let executor = executor.clone();

            async move {
                let _: Result<(), A::Error> = async move {
                    let mut stream = executor.clone().execute(&command).await?;
                    while let Some(action) = stream.next().await {
                        let this = self.clone();
                        let executor = executor.clone();
                        // NOTE: The `execute` call needs to run sequentially. Executing it
                        // asynchronously would disrupt the order of `toolUse` content, leading to
                        // mixed-up.
                        this.execute(action?, executor).await?;
                    }

                    Ok(())
                }
                .await;
            }
        }))
        .await;

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
