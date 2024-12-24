use forge_provider::MessageStream;

use crate::error::Result;

pub struct Engine;

enum Action {}

impl Engine {
    pub async fn as_stream(&self) -> Result<MessageStream<Action>> {
        todo!()
    }
}
