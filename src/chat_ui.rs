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
        Self {
            input: String::new(),
            cursor_position: 0,
            messages: Vec::new(),
            streaming_response: None,
        }
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
}

pub struct ChatUI {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: Arc<Mutex<ChatState>>,
}

impl ChatUI {
    pub fn new() -> Result<Self, Box<dyn Error>> {
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

    fn render_chat_ui(frame: &mut Frame, state: &ChatState) -> Result<(), Box<dyn Error>> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Messages area
                Constraint::Length(3), // Input area
            ])
            .split(frame.size());

        // Messages area with scrolling
        let messages_text = state.messages.iter()
            .chain(state.streaming_response.iter())
            .cloned()
            .collect::<Vec<String>>()
            .join("\n");

        let messages_block = Paragraph::new(messages_text)
            .block(Block::default().borders(Borders::ALL).title("Chat Messages"))
            .wrap(Wrap { trim: true })
            .scroll((state.messages.len() as u16, 0));

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

    pub async fn run<S>(
        &mut self,
        mut response_stream: S,
    ) -> Result<(), Box<dyn Error>>
    where
        S: Stream<Item = String> + Unpin,
    {
        let (state_tx, mut state_rx) = mpsc::channel(100);
        let (input_tx, mut input_rx) = mpsc::channel(100);
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        // Set up Ctrl+C handler
        let shutdown_tx_clone = shutdown_tx.clone();
        tokio::spawn(async move {
            if let Ok(()) = tokio::signal::ctrl_c().await {
                let _ = shutdown_tx_clone.send(()).await;
            }
        });

        // Spawn input handling task
        let state_clone = Arc::clone(&self.state);
        let shutdown_tx_clone = shutdown_tx.clone();
        let input_handle = tokio::spawn(async move {
            loop {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Enter => {
                                let mut state = state_clone.lock().await;
                                let input = state.input.clone();
                                if !input.is_empty() {
                                    state.messages.push(format!("You: {}", input));
                                    let _ = input_tx.send(input).await;
                                }
                                state.input.clear();
                                state.cursor_position = 0;
                                drop(state);
                                let _ = state_tx.send(()).await;
                            },
                            KeyCode::Char(c) => {
                                let mut state = state_clone.lock().await;
                                state.insert_char(c);
                                drop(state);
                                let _ = state_tx.send(()).await;
                            },
                            KeyCode::Backspace => {
                                let mut state = state_clone.lock().await;
                                state.backspace();
                                drop(state);
                                let _ = state_tx.send(()).await;
                            },
                            KeyCode::Delete => {
                                let mut state = state_clone.lock().await;
                                state.delete();
                                drop(state);
                                let _ = state_tx.send(()).await;
                            },
                            KeyCode::Left => {
                                let mut state = state_clone.lock().await;
                                if state.cursor_position > 0 {
                                    state.cursor_position -= 1;
                                }
                                drop(state);
                                let _ = state_tx.send(()).await;
                            },
                            KeyCode::Right => {
                                let mut state = state_clone.lock().await;
                                if state.cursor_position < state.input.len() {
                                    state.cursor_position += 1;
                                }
                                drop(state);
                                let _ = state_tx.send(()).await;
                            },
                            KeyCode::Home => {
                                let mut state = state_clone.lock().await;
                                state.cursor_position = 0;
                                drop(state);
                                let _ = state_tx.send(()).await;
                            },
                            KeyCode::End => {
                                let mut state = state_clone.lock().await;
                                state.cursor_position = state.input.len();
                                drop(state);
                                let _ = state_tx.send(()).await;
                            },
                            KeyCode::Esc => {
                                let _ = shutdown_tx_clone.send(()).await;
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        // Process responses
        loop {
            tokio::select! {
                Some(response) = response_stream.next() => {
                    let mut state = self.state.lock().await;
                    state.messages.push(format!("Assistant: {}", response));
                    let state_snapshot = state.clone();
                    drop(state);
                    
                    self.terminal.draw(|frame| {
                        Self::render_chat_ui(frame, &state_snapshot).unwrap()
                    })?;
                }
                Some(_) = state_rx.recv() => {
                    let state_snapshot = self.state.lock().await.clone();
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

        // Clean up input handling task
        input_handle.abort();

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
