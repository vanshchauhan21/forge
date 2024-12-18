use tokio_stream::StreamExt;

use crate::error::Error;
use crate::runtime::Runtime;
use crate::Combine;
use std::cell::Cell;
use std::sync::{Arc, Mutex};

pub struct App<A, S, C> {
    _app: Box<dyn for<'a> Fn(&'a A, &'a mut S) -> C>,
}

impl<A, S, C: Default> Default for App<A, S, C> {
    fn default() -> Self {
        Self {
            _app: Box::new(|_, _| C::default()),
        }
    }
}

impl<A, S, C> App<A, S, C> {
    pub fn update(f: impl Fn(&A, S) -> S + 'static) -> Self
    where
        C: Default,
    {
        Self {
            _app: Box::new(|_, _| C::default()),
        }
    }

    pub fn command(f: impl for<'a> Fn(&'a A, &'a mut S) -> C + 'static) -> Self {
        Self { _app: Box::new(f) }
    }

    pub async fn execute(self, runtime: impl Runtime<A, C> + 'static) -> Result<(), Error>
    where
        A: Default,
        S: Default,
    {
        Sink::new(self, runtime, S::default())
            .drain(&A::default())
            .await
    }
}

struct Sink<A, S, C> {
    app: App<A, S, C>,
    runtime: Box<dyn Runtime<A, C>>,
    state: Arc<Mutex<Cell<S>>>,
}

impl<A, S, C> Sink<A, S, C> {
    fn new(app: App<A, S, C>, runtime: impl Runtime<A, C> + 'static, state: S) -> Self {
        Self {
            app,
            runtime: Box::new(runtime),
            state: Arc::new(Mutex::new(Cell::new(state))),
        }
    }

    async fn drain(&self, action: &A) -> Result<(), Error> {
        let mut state = self.state.lock().unwrap();
        let command = (self.app._app)(action, state.get_mut());

        // Execute the generated stream of actions
        let mut actions = self.runtime.run(command).await?;
        while let Some(action) = actions.next().await {
            self.drain(&action).await?;
        }

        Ok(())
    }
}

impl<A: 'static, S: 'static, C: Combine + 'static> Combine for App<A, S, C> {
    fn combine(self, other: Self) -> Self {
        Self {
            _app: Box::new(move |a, s| (self._app)(a, s).combine((other._app)(a, s))),
        }
    }
}
