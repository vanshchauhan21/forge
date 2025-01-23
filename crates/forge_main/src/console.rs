use std::io::{self, Write};
use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::normalize::NewLine;

lazy_static! {
    /// Global console instance for standardized output handling
    pub static ref CONSOLE: Console = Console::new();
}

/// Console state containing both the output stream and normalizer
struct ConsoleState {
    stdout: io::Stdout,
    normalizer: NewLine,
}

/// A specialized console that provides enhanced printing capabilities
pub struct Console {
    /// Combined state under a single mutex
    state: Mutex<ConsoleState>,
}

impl Default for Console {
    fn default() -> Self {
        Self::new()
    }
}

impl Console {
    /// Creates a new Console instance
    pub fn new() -> Self {
        Self {
            state: Mutex::new(ConsoleState { stdout: io::stdout(), normalizer: NewLine::new() }),
        }
    }

    /// Writes the given content
    pub fn write(&self, content: impl AsRef<str>) -> io::Result<()> {
        // Disable raw mode to prevent terminal issues
        #[cfg(not(test))]
        crossterm::terminal::disable_raw_mode().expect("Failed to enable raw mode");

        let content = content.as_ref();
        let mut state = self.state.lock().unwrap();
        if content.is_empty() {
            return Ok(());
        }

        let normalized = state.normalizer.normalize(content);
        write!(state.stdout, "{}", normalized)?;
        state.stdout.flush()
    }

    /// Writes the given content with a newline
    pub fn writeln(&self, content: impl AsRef<str>) -> io::Result<()> {
        let content = format!("{}\n", content.as_ref());
        self.write(content)
    }

    /// Writes a newline
    pub fn newline(&self) -> io::Result<()> {
        self.write("\n")
    }
}
