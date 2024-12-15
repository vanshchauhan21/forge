use std::error::Error;
use std::io::{self};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::stream::Stream;
use futures::StreamExt;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
    style::{Style, Modifier},
};
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;

#[derive(Clone)]
pub struct ChatState {
    input: String,
    cursor_position: usize,
    messages: Vec<String>,
    streaming_response: Option<String>,
}

impl ChatState {
    pub fn new() -> Self {
        let mut state = Self {
            input: String::new(),
            cursor_position: 0,
            messages: Vec::new(),
            streaming_response: None,
        };
        
        // Add welcome message to initial state
        state.messages.push(
            "Welcome to Code Forge Chat, powered by Claude 3 Sonnet.\n\
            I'm your AI programming assistant, specializing in software development and technical topics.\n\
            Feel free to ask questions about programming, architecture, best practices, or any tech-related topics.\n\
            Type your message and press Enter to send. Press Ctrl+C or Esc to exit.".to_string()
        );
        
        state
    }

    fn insert_char(&mut self, c: char) {
        let pos = self.cursor_position;
        if pos == self.input.len() {
            self.input.push(c);
        } else {
            self.input.insert(pos, c);
        }
        self.cursor_position += 1;
    }

    fn backspace(&mut self) {
        if self.cursor_position > 0 {
            let pos = self.cursor_position - 1;
            self.input.remove(pos);
            self.cursor_position = pos;
        }
    }

    fn delete(&mut self) {
        if self.cursor_position < self.input.len() {
            self.input.remove(self.cursor_position);
        }
    }

    fn start_response(&mut self) {
        self.streaming_response = Some(String::from("Assistant: "));
    }

    fn append_to_response(&mut self, text: &str) {
        // Skip "Sending response part:" messages
        if text.starts_with("Sending response part:") {
            return;
        }

        if let Some(ref mut response) = self.streaming_response {
            response.push_str(text);
        } else {
            self.streaming_response = Some(format!("Assistant: {}", text));
        }
    }

    fn complete_response(&mut self) {
        if let Some(response) = self.streaming_response.take() {
            // Format the response with proper line breaks
            let formatted = response
                .lines()
                .map(|line| line.trim())
                .collect::<Vec<_>>()
                .join("\n");
            self.messages.push(formatted);
        }
    }
}

pub struct ChatUI {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: Arc<Mutex<ChatState>>,
}

impl ChatUI {
    pub fn new() -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Terminal setup
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            state: Arc::new(Mutex::new(ChatState::new())),
        })
    }

    fn render_chat_ui(frame: &mut Frame, state: &ChatState) -> Result<(), Box<dyn Error + Send + Sync>> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Messages area
                Constraint::Length(3), // Input area
            ])
            .split(frame.size());

        // Messages area with scrolling
        let mut display_messages = state.messages.clone();
        if let Some(ref current) = state.streaming_response {
            display_messages.push(current.clone());
        }

        let messages_text = display_messages.join("\n\n");

        let messages_block = Paragraph::new(messages_text)
            .block(Block::default().borders(Borders::ALL).title("Chat Messages"))
            .wrap(Wrap { trim: true })
            .scroll((display_messages.len().saturating_sub(1) as u16, 0));

        frame.render_widget(messages_block, layout[0]);

        // Input area with cursor
        let input_len = state.input.len();
        let cursor_style = Style::default().add_modifier(Modifier::REVERSED);
        
        let input_text = if state.cursor_position < input_len {
            let (before, at_cursor) = state.input.split_at(state.cursor_position);
            let (at_cursor, after) = at_cursor.split_at(1);
            Line::from(vec![
                Span::raw(before),
                Span::styled(at_cursor, cursor_style),
                Span::raw(after),
            ])
        } else if input_len > 0 {
            Line::from(vec![
                Span::raw(&state.input),
                Span::styled(" ", cursor_style),
            ])
        } else {
            Line::from(vec![Span::styled(" ", cursor_style)])
        };

        let input_block = Paragraph::new(input_text)
            .block(Block::default().borders(Borders::ALL).title("Input"));
        frame.render_widget(input_block, layout[1]);

        Ok(())
    }

    async fn handle_key_event(&mut self, key: KeyCode, input_tx: &mpsc::Sender<String>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let mut state = self.state.lock().await;
        match key {
            KeyCode::Enter => {
                let input = state.input.clone();
                if !input.is_empty() {
                    state.messages.push(format!("You: {}", input));
                    state.start_response();
                    state.input.clear();
                    state.cursor_position = 0;
                    input_tx.send(input).await?;
                }
            },
            KeyCode::Char(c) => {
                state.insert_char(c);
            },
            KeyCode::Backspace => {
                state.backspace();
            },
            KeyCode::Delete => {
                state.delete();
            },
            KeyCode::Left => {
                if state.cursor_position > 0 {
                    state.cursor_position -= 1;
                }
            },
            KeyCode::Right => {
                if state.cursor_position < state.input.len() {
                    state.cursor_position += 1;
                }
            },
            KeyCode::Home => {
                state.cursor_position = 0;
            },
            KeyCode::End => {
                state.cursor_position = state.input.len();
            },
            KeyCode::Esc => {
                return Ok(true);
            }
            _ => {}
        }
        
        let state_snapshot = state.clone();
        drop(state);
        self.terminal.draw(|frame| {
            Self::render_chat_ui(frame, &state_snapshot).unwrap()
        })?;

        Ok(false)
    }

    pub async fn run<S>(
        &mut self,
        mut response_stream: S,
        input_tx: mpsc::Sender<String>,
    ) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        S: Stream<Item = String> + Unpin,
    {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        let (event_tx, mut event_rx) = mpsc::channel(100);

        // Set up Ctrl+C handler
        let shutdown_tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            if let Ok(()) = tokio::signal::ctrl_c().await {
                let _ = shutdown_tx_clone.send(()).await;
            }
        });

        // Spawn event reading task
        let event_tx_clone = event_tx.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(event) = event::read() {
                    if let Err(_) = event_tx_clone.send(event).await {
                        break;
                    }
                }
            }
        });

        // Initial render
        let state_snapshot = self.state.lock().await.clone();
        self.terminal.draw(|frame| {
            Self::render_chat_ui(frame, &state_snapshot).unwrap()
        })?;

        // Main event loop
        loop {
            tokio::select! {
                Some(event) = event_rx.recv() => {
                    if let Event::Key(key) = event {
                        if key.kind == KeyEventKind::Press {
                            if self.handle_key_event(key.code, &input_tx).await? {
                                break;
                            }
                        }
                    }
                }
                Some(response) = response_stream.next() => {
                    let mut state = self.state.lock().await;
                    if response == "\n" {
                        state.complete_response();
                    } else {
                        state.append_to_response(&response);
                    }
                    let state_snapshot = state.clone();
                    drop(state);
                    
                    self.terminal.draw(|frame| {
                        Self::render_chat_ui(frame, &state_snapshot).unwrap()
                    })?;
                }
                Some(_) = shutdown_rx.recv() => {
                    break;
                }
                else => break,
            }
        }

        Ok(())
    }
}

impl Drop for ChatUI {
    fn drop(&mut self) {
        // Cleanup terminal
        disable_raw_mode().unwrap();
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).unwrap();
        self.terminal.show_cursor().unwrap();
    }
}
