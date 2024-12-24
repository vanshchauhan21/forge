use std::convert::Infallible;

use axum::response::sse::Event;
use futures::stream::{self, Stream};
use serde::Serialize;
use tokio::sync::broadcast;

use crate::Result;

// Shared state between HTTP server and CLI
#[derive(Clone)]
pub struct Broadcast {
    tx: broadcast::Sender<Event>,
}

impl Default for Broadcast {
    fn default() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx }
    }
}

impl Broadcast {
    #[allow(unused)]
    pub fn dispatch(&self, event: impl Serialize) -> Result<usize> {
        let json = serde_json::to_string(&event)?;
        Ok(self.tx.send(Event::default().data(json))?)
    }

    pub async fn as_stream(&self) -> impl Stream<Item = std::result::Result<Event, Infallible>> {
        let rx = self.tx.subscribe();

        stream::unfold(rx, |mut rx| async move {
            let event = rx.recv().await.expect("Broadcast channel closed");
            Some((Ok(event), rx))
        })
    }
}
