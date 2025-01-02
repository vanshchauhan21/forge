use std::sync::Arc;

use futures::future::join_all;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

use crate::ChatService;

pub trait Application: Send + Sync + Sized + 'static {
    type Action: Send;
    type Error: Send;
    type Command: Send;

    fn run(
        &mut self,
        action: &Self::Action,
    ) -> std::result::Result<Vec<Self::Command>, Self::Error> {
        self.dispatch().run(self, action)
    }

    fn dispatch(&self) -> Channel<Self, Self::Action, Self::Command, Self::Error>;

    #[allow(unused)]
    fn run_seq(
        &mut self,
        actions: impl IntoIterator<Item = Self::Action>,
    ) -> Result<Vec<Self::Command>, Self::Error> {
        let mut commands = Vec::new();
        for action in actions.into_iter() {
            commands.extend(self.run(&action)?);
        }

        Ok(commands)
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

    pub async fn state(&self) -> A
    where
        A: Clone,
    {
        self.state.lock().await.clone()
    }
}

impl<A: Application + 'static> ApplicationRuntime<A> {
    #[async_recursion::async_recursion]
    pub async fn execute<'a>(
        &'a self,
        action: A::Action,
        executor: Arc<
            dyn ChatService<Command = A::Command, Action = A::Action, Error = A::Error> + 'static,
        >,
    ) -> std::result::Result<(), A::Error> {
        let mut guard = self.state.lock().await;
        let commands = guard.run(&action)?;
        drop(guard);

        join_all(commands.into_iter().map(|command| {
            let executor = executor.clone();

            async move {
                let _: Result<(), A::Error> = async move {
                    let mut stream = executor.clone().execute(&command).await?;
                    while let Some(action) = stream.next().await {
                        // NOTE: The `execute` call needs to run sequentially. Executing it
                        // asynchronously would disrupt the order of `toolUse` content, leading to
                        // mixed-up.
                        self.execute(action?, executor.clone()).await?;
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

type Type<State, In, Out, Error> = Box<dyn Fn(&mut State, &In) -> Result<Vec<Out>, Error>>;

pub struct Channel<State, In, Out, Error>(Type<State, In, Out, Error>);

impl<State: 'static, In: 'static, Out: 'static, Error: 'static> Channel<State, In, Out, Error> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut State, &In) -> Result<Vec<Out>, Error> + 'static,
    {
        Self(Box::new(f))
    }

    pub fn zip(self, other: Channel<State, In, Out, Error>) -> Self {
        Self::new(move |state: &mut State, action: &In| {
            let mut commands = self.0(state, action)?;
            commands.extend(other.0(state, action)?);
            Ok(commands)
        })
    }

    pub fn and_then<F, Outer: 'static>(self, f: F) -> Channel<State, In, Outer, Error>
    where
        F: Fn(&mut State, &Out) -> Result<Vec<Outer>, Error> + 'static,
    {
        {
            let other = Channel::new(f);
            Channel::new(move |state: &mut State, action: &In| {
                let mut out0: Vec<Outer> = Vec::new();
                for out in self.0(state, action)?.into_iter() {
                    out0.extend(other.0(state, &out)?);
                }

                Ok(out0)
            })
        }
    }

    pub fn run(&self, state: &mut State, action: &In) -> Result<Vec<Out>, Error> {
        (self.0)(state, action)
    }
}

impl<State, Action, Command, Error> Default for Channel<State, Action, Command, Error> {
    fn default() -> Self {
        Self(Box::new(|_, _| Ok(Vec::new())))
    }
}
