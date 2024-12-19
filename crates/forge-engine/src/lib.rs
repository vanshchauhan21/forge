pub mod error;
pub mod model;
mod runtime;
pub use runtime::Runtime;

pub type Stream<A> = Box<dyn tokio_stream::Stream<Item = A> + Unpin>;
