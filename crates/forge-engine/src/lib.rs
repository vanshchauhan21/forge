pub mod error;
mod runtime;
pub mod state;
use std::sync::{Arc, Mutex};

use error::Result;
pub use runtime::Runtime;
use state::State;

pub type Stream<A> = Box<dyn tokio_stream::Stream<Item = A> + Unpin>;

#[derive(Default, Clone)]
pub struct CodeForge {
    state: Arc<Mutex<State>>,
}

impl CodeForge {
    pub async fn run(&self, runtime: impl Runtime) -> Result<()> {
        todo!()
    }
}
