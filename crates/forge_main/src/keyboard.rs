use std::collections::HashSet;

use crossterm::event::{self, Event, KeyCode};
use tokio_stream::{Stream, StreamExt};

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum KeyEvent {
    Esc,
    None,
}

impl From<KeyCode> for KeyEvent {
    fn from(code: KeyCode) -> Self {
        match code {
            KeyCode::Esc => KeyEvent::Esc,
            _ => KeyEvent::None,
        }
    }
}

pub struct KeyboardEvents<
    S: Stream<Item = std::io::Result<Event>> + Unpin + Send = event::EventStream,
> {
    reader: S,
    events: HashSet<KeyEvent>,
}

impl KeyboardEvents<event::EventStream> {
    pub fn new() -> Self {
        Self { reader: event::EventStream::new(), events: HashSet::new() }
    }
}

impl<S: Stream<Item = std::io::Result<Event>> + Unpin + Send> KeyboardEvents<S> {
    pub fn register(&mut self, event: KeyEvent) {
        self.events.insert(event);
    }

    pub async fn is_pressed(&mut self) -> bool {
        crossterm::terminal::enable_raw_mode().expect("Failed to enable raw mode");
        let result = if let Some(Ok(Event::Key(key))) = self.reader.next().await {
            let event = KeyEvent::from(key.code);
            self.events.contains(&event)
        } else {
            false
        };
        crossterm::terminal::disable_raw_mode().expect("Failed to enable raw mode");
        result
    }
}

impl<S: Stream<Item = std::io::Result<Event>> + Unpin + Send> Drop for KeyboardEvents<S> {
    fn drop(&mut self) {
        // best effort to disable raw mode
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::mpsc;
    use tokio_stream::wrappers::UnboundedReceiverStream;

    use super::*;

    struct TestKeyboard {
        keyboard: KeyboardEvents<UnboundedReceiverStream<std::io::Result<Event>>>,
        event_tx: mpsc::UnboundedSender<std::io::Result<Event>>,
    }

    impl TestKeyboard {
        fn new() -> Self {
            let (tx, rx) = mpsc::unbounded_channel();
            let stream = UnboundedReceiverStream::new(rx);

            Self {
                keyboard: KeyboardEvents { reader: stream, events: HashSet::new() },
                event_tx: tx,
            }
        }

        fn send_key(&self, code: KeyCode) {
            use crossterm::event::{KeyEvent, KeyModifiers};
            let event = Event::Key(KeyEvent::new(code, KeyModifiers::empty()));
            self.event_tx
                .send(Ok(event))
                .expect("Failed to send test event");
        }

        async fn is_pressed(&mut self) -> bool {
            self.keyboard.is_pressed().await
        }

        fn register(&mut self, event: KeyEvent) {
            self.keyboard.register(event);
        }
    }

    #[tokio::test]
    async fn test_key_press() {
        let mut test_kb = TestKeyboard::new();
        test_kb.register(KeyEvent::Esc);

        // Test ESC key press
        test_kb.send_key(KeyCode::Esc);
        let is_pressed = tokio::time::timeout(Duration::from_millis(50), test_kb.is_pressed())
            .await
            .unwrap();
        assert!(is_pressed, "ESC key press should be detected");
    }
}
