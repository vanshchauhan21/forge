use std::io::{self, Write};
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    /// Global console instance for standardized output handling
    pub static ref CONSOLE: Console = Console::new();
}

/// A specialized console that provides enhanced printing capabilities
pub struct Console {
    stdout: Mutex<io::Stdout>,
    /// Stores the last text written and a count of trailing newlines
    state: Mutex<ConsoleState>,
}

struct ConsoleState {
    last_text: String,
    trailing_newlines: usize,
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
            stdout: Mutex::new(io::stdout()),
            state: Mutex::new(ConsoleState {
                last_text: String::new(),
                trailing_newlines: 0,
            }),
        }
    }

    /// Writes the given content without a newline
    pub fn write(&self, content: impl AsRef<str>) -> io::Result<()> {
        let content = content.as_ref();
        let mut state = self.state.lock().unwrap();
        let mut stdout = self.stdout.lock().unwrap();
        
        if content.is_empty() {
            return Ok(());
        }

        let normalized = if state.trailing_newlines > 0 && content.starts_with('\n') {
            let additional_newlines = Self::count_trailing_newlines(content);
            let total_newlines = (state.trailing_newlines + additional_newlines).min(2);
            
            if total_newlines > state.trailing_newlines {
                let extra_newlines = total_newlines - state.trailing_newlines;
                "\n".repeat(extra_newlines) + content.trim_start_matches(&['\n', '\r'][..])
            } else {
                content.trim_start_matches(&['\n', '\r'][..]).to_string()
            }
        } else {
            Self::normalize_newlines(content)
        };

        write!(stdout, "{}", normalized)?;
        stdout.flush()?;
        
        state.last_text = content.to_string();
        state.trailing_newlines = Self::count_trailing_newlines(&normalized);
        
        Ok(())
    }

    /// Writes the given content with a newline
    pub fn writeln(&self, content: impl AsRef<str>) -> io::Result<()> {
        let content = content.as_ref();
        let mut state = self.state.lock().unwrap();
        let mut stdout = self.stdout.lock().unwrap();
        
        let mut normalized = Self::normalize_newlines(content);
        
        let current_newlines = Self::count_trailing_newlines(&normalized);
        let total_newlines = (current_newlines + 1).min(2);
        
        if current_newlines > 0 {
            normalized.truncate(normalized.len() - current_newlines);
        }
        
        normalized.push_str(&"\n".repeat(total_newlines));
        
        write!(stdout, "{}", normalized)?;
        stdout.flush()?;
        
        state.last_text = content.to_string();
        state.trailing_newlines = total_newlines;
        
        Ok(())
    }

    /// Returns the last text that was written
    pub fn last_text(&self) -> String {
        self.state.lock().unwrap().last_text.clone()
    }

    /// Returns the number of trailing newlines in a string
    fn count_trailing_newlines(s: &str) -> usize {
        s.as_bytes()
            .iter()
            .rev()
            .take_while(|&&b| b == b'\n' || b == b'\r')
            .filter(|&&b| b == b'\n')
            .count()
    }

    /// Normalizes consecutive newlines to ensure no more than 2 in a row
    fn normalize_newlines(content: &str) -> String {
        let mut result = String::with_capacity(content.len());
        let mut consecutive_newlines = 0;

        for c in content.chars() {
            if c == '\n' {
                consecutive_newlines += 1;
                if consecutive_newlines <= 2 {
                    result.push(c);
                }
            } else if c != '\r' { // Skip \r as we'll normalize to \n only
                consecutive_newlines = 0;
                result.push(c);
            }
        }
        result
    }
}