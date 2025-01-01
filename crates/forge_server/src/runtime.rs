use std::sync::Arc;

use forge_provider::ResultStream;
use futures::future::join_all;
use tokio::sync::Mutex;
use tokio_stream::StreamExt;

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

    fn dispatch(&self) -> Dispatch<Self, Self::Action, Self::Command, Self::Error>;

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
            impl Executor<Command = A::Command, Action = A::Action, Error = A::Error> + 'static,
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

#[async_trait::async_trait]
pub trait Executor: Send + Sync {
    type Command;
    type Action;
    type Error;
    async fn execute(&self, command: &Self::Command) -> ResultStream<Self::Action, Self::Error>;
}

type Type<State, Action, Command, Error> =
    Box<dyn Fn(&mut State, &Action) -> Result<Vec<Command>, Error>>;

pub struct Dispatch<State, Action, Command, Error>(Type<State, Action, Command, Error>);

#[allow(unused)]
impl<State: 'static, Action: 'static, Command: 'static, Error: 'static>
    Dispatch<State, Action, Command, Error>
{
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut State, &Action) -> Result<Vec<Command>, Error> + 'static,
    {
        Self(Box::new(f))
    }

    pub fn select<F, S, Action0>(s: S, f: F) -> Self
    where
        S: Fn(&Action) -> Option<&Action0> + 'static,
        F: Fn(&mut State, &Action0) -> Result<Vec<Command>, Error> + 'static,
    {
        Self(Box::new(move |state, action| match s(action) {
            None => Ok(vec![]),
            Some(action) => f(state, &action),
        }))
    }

    pub fn and(self, other: Self) -> Self {
        let f = move |state: &mut State, action: &Action| {
            let mut commands = self.0(state, action)?;
            commands.extend(other.0(state, action)?);
            Ok(commands)
        };
        Self(Box::new(f))
    }

    pub fn try_command<F>(self, f: F) -> Self
    where
        F: Fn(&mut State, &Action) -> Result<Vec<Command>, Error> + 'static,
    {
        self.and(Dispatch::new(f))
    }

    pub fn command<F>(self, f: F) -> Self
    where
        F: Fn(&mut State, &Action) -> Vec<Command> + 'static,
    {
        self.try_command(move |state, action| Ok(f(state, action)))
    }

    pub fn try_update<F>(self, f: F) -> Self
    where
        F: Fn(&mut State, &Action) -> Result<(), Error> + 'static,
    {
        self.try_command(move |state, action| {
            f(state, action)?;
            Ok(Vec::new())
        })
    }

    pub fn update<F>(self, f: F) -> Self
    where
        F: Fn(&mut State, &Action) + 'static,
    {
        self.try_update(move |state, action| {
            f(state, action);
            Ok(())
        })
    }

    pub fn run(&self, state: &mut State, action: &Action) -> Result<Vec<Command>, Error> {
        (self.0)(state, action)
    }

    pub fn when<F>(self, f: F) -> Self
    where
        F: Fn(&State, &Action) -> bool + 'static,
    {
        let f = move |state: &mut State, action: &Action| {
            if f(state, action) {
                self.run(state, action)
            } else {
                Ok(Vec::new())
            }
        };
        Self(Box::new(f))
    }

    pub fn pipe(state: &mut State, action: &Action) -> Self {
        todo!()
    }
}

impl<State, Action, Command, Error> Default for Dispatch<State, Action, Command, Error> {
    fn default() -> Self {
        Self(Box::new(|_, _| Ok(Vec::new())))
    }
}
