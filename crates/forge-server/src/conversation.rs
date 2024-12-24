use crate::broadcast::Broadcast;
use crate::EventStream;

#[derive(Default)]
pub struct Conversation {
    broadcast: Broadcast,
}

#[derive(Debug, serde::Serialize)]
enum Action {}

#[derive(serde::Deserialize)]
pub struct Request {
    // Add fields as needed, for example:
    pub prompt: String,
    pub model: Option<String>,
}

impl Conversation {
    pub async fn chat(&self, _request: Request) -> EventStream {
        Box::new(Box::pin(self.broadcast.as_stream().await))
    }
}
