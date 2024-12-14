use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

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
};
use tokio::sync::mpsc;
use tokio_stream;

#[derive(Debug, Clone)]
struct Agent {}

impl Agent {
    fn new() -> Agent {
        Agent {}
    }

    /// Method to write a file (added back to maintain original functionality)
    fn write_file(&self, path: &str, content: &str) -> Result<(), std::io::Error> {
        let mut file = fs::File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Simulate a streaming response from an AI agent
    async fn stream_response(&self) -> impl Stream<Item = String> {
        let (tx, rx) = mpsc::channel(5);

        tokio::spawn(async move {
            let response_parts = vec![
                "Hello, ",
                "this is ",
                "a simulated ",
                "streaming response ",
                "from the AI agent.",
            ];

            for part in response_parts {
                tokio::time::sleep(Duration::from_millis(200)).await;
                tx.send(part.to_string()).await.unwrap();
            }
        });

        tokio_stream::wrappers::ReceiverStream::new(rx)
    }
}

/// UI state for the chat interface
#[derive(Clone)]
struct ChatState {
    input: String,
    messages: Vec<String>,
    streaming_response: Option<String>,
}

impl ChatState {
    fn new() -> Self {
        Self {
            input: String::new(),
            messages: Vec::new(),
            streaming_response: None,
        }
    }
}

/// Render the chat UI
fn render_chat_ui(frame: &mut Frame, state: &ChatState) -> Result<(), Box<dyn Error>> {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Messages area
            Constraint::Length(3), // Input area
        ])
        .split(frame.size());

    // Messages area
    let messages_block = Paragraph::new(
        state
            .messages
            .iter()
            .chain(state.streaming_response.iter())
            .cloned()
            .collect::<Vec<String>>()
            .join("\n"),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Chat Messages"),
    )
    .wrap(Wrap { trim: true });
    frame.render_widget(messages_block, layout[0]);

    // Input area
    let input_block = Paragraph::new(state.input.clone())
        .block(Block::default().borders(Borders::ALL).title("Input"));
    frame.render_widget(input_block, layout[1]);

    Ok(())
}

/// Main chat loop handling UI and interactions
async fn chat_loop() -> Result<(), Box<dyn Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Chat state and channels for async communication
    let chat_state_main = Arc::new(tokio::sync::Mutex::new(ChatState::new()));
    let (input_tx, mut input_rx) = mpsc::channel(100);
    let (state_tx, mut state_rx) = mpsc::channel(100);

    // Spawn a task to handle input events

    let chat_state = chat_state_main.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Enter => {
                            // Safely get and clear input
                            let input = {
                                let mut state = chat_state.lock().await;
                                let input = state.input.clone();
                                state.input.clear();
                                input
                            };

                            // Send input and signal state update
                            if !input.is_empty() {
                                input_tx.send(input).await.unwrap();
                                state_tx.send(()).await.unwrap();
                            }
                        }
                        KeyCode::Char(c) => {
                            let mut state = chat_state.lock().await;
                            state.input.push(c);
                            state_tx.send(()).await.unwrap();
                        }
                        KeyCode::Backspace => {
                            let mut state = chat_state.lock().await;
                            state.input.pop();
                            state_tx.send(()).await.unwrap();
                        }
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    // Main rendering and message processing loop
    let leader = Agent::new();

    // Example of file manipulation (from original code)
    match leader.write_file("example.txt", "Hello, file manipulation!") {
        Ok(_) => println!("File created successfully"),
        Err(e) => eprintln!("Error creating file: {}", e),
    }

    while let Some(message) = input_rx.recv().await {
        // Add user message to chat
        {
            let mut state = chat_state_main.lock().await;
            state.messages.push(format!("User: {}", message));
        }

        // Clear any previous streaming response
        {
            let mut state = chat_state_main.lock().await;
            state.streaming_response = None;
        }

        // Simulate streaming AI response
        let mut response_stream = leader.stream_response().await;

        // Render loop for streaming response
        while let Some(part) = response_stream.next().await {
            {
                let mut state = chat_state_main.lock().await;
                state.streaming_response = Some(part);
            }

            // Render terminal
            let content = &chat_state_main.lock().await;
            terminal.draw(|frame| {
                render_chat_ui(frame, &content).unwrap();
            })?;
        }

        // Add full response to messages
        {
            let mut state = chat_state_main.lock().await;
            if let Some(full_response) = state.streaming_response.take() {
                state.messages.push(format!("AI: {}", full_response));
            }
        }

        // Consume any state update signals
        while state_rx.try_recv().is_ok() {}
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Start the chat interface
    chat_loop().await?;

    Ok(())
}
