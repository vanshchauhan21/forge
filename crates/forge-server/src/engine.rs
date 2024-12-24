
use crate::broadcast::Broadcast;
use crate::EventStream;

#[derive(Default)]
pub struct Engine {
    broadcast: Broadcast,
}

#[derive(Debug, serde::Serialize)]
enum Action {}

impl Engine {
    pub async fn as_stream(&self) -> EventStream {
        Box::new(Box::pin(self.broadcast.as_stream().await))
    }
}
