use std::collections::HashSet;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tokio_stream::{Stream, StreamExt};

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Key {
    Esc,
    ControlC,
    None,
}

impl From<Key> for KeyEvent {
    fn from(key: Key) -> Self {
        match key {
            Key::Esc => KeyEvent::new(KeyCode::Esc, KeyModifiers::empty()),
            Key::ControlC => KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Key::None => KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
        }
    }
}

impl From<KeyEvent> for Key {
    fn from(key_event: KeyEvent) -> Self {
        match key_event.code {
            KeyCode::Esc => Key::Esc,
            KeyCode::Char(char)
                if char == 'c' && key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Key::ControlC
            }
            _ => Key::None,
        }
    }
}

pub struct KeyboardEvents<
    S: Stream<Item = std::io::Result<Event>> + Unpin + Send = event::EventStream,
> {
    reader: S,
    events: HashSet<Key>,
}

impl KeyboardEvents<event::EventStream> {
    pub fn new() -> Self {
        let reader = event::EventStream::new();
        Self { reader, events: HashSet::new() }
    }
}

impl<S: Stream<Item = std::io::Result<Event>> + Unpin + Send> KeyboardEvents<S> {
    pub fn register(&mut self, event: Key) {
        self.events.insert(event);
    }

    pub async fn is_pressed(&mut self) -> bool {
        #[cfg(not(test))]
        crossterm::terminal::enable_raw_mode().expect("Failed to enable raw mode");

        let result = if let Some(Ok(Event::Key(key))) = self.reader.next().await {
            let event = Key::from(key);
            self.events.contains(&event)
        } else {
            false
        };

        #[cfg(not(test))]
        crossterm::terminal::disable_raw_mode().expect("Failed to disable raw mode");

        result
    }
}

impl<S: Stream<Item = std::io::Result<Event>> + Unpin + Send> Drop for KeyboardEvents<S> {
    fn drop(&mut self) {
        // best effort to disable raw mode
        #[cfg(not(test))]
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

        fn send_key(&self, code: Key) {
            let event = Event::Key(KeyEvent::from(code));
            self.event_tx
                .send(Ok(event))
                .expect("Failed to send test event");
        }

        // Mock version that doesn't require raw mode
        async fn is_pressed(&mut self) -> bool {
            self.keyboard.is_pressed().await
        }

        fn register(&mut self, event: Key) {
            self.keyboard.register(event);
        }
    }

    #[tokio::test]
    async fn test_key_press() {
        let mut test_kb = TestKeyboard::new();
        test_kb.register(Key::Esc);
        test_kb.register(Key::ControlC);

        // Test ESC key press
        test_kb.send_key(Key::Esc);
        let is_pressed = tokio::time::timeout(Duration::from_millis(50), test_kb.is_pressed())
            .await
            .unwrap();
        assert!(is_pressed, "ESC key press should be detected");

        // Test Control + C key press
        test_kb.send_key(Key::ControlC);
        let is_pressed = tokio::time::timeout(Duration::from_millis(50), test_kb.is_pressed())
            .await
            .unwrap();
        assert!(is_pressed, "Control + C key press should be detected");
    }
}
