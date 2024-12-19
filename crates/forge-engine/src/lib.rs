pub mod error;
pub mod model;
mod runtime;
use model::Action;
pub use runtime::Runtime;
use tokio_stream::Stream;

pub type ActionStream = Box<dyn Stream<Item = Action> + Unpin>;
