use std::io::{self, Write};
use std::sync::Mutex;

use lazy_static::lazy_static;

mod normalize;
use normalize::ConsoleNormalizer;

lazy_static! {
    /// Global console instance for standardized output handling
    pub static ref CONSOLE: Console = Console::new();
}

/// Console state containing both the output stream and normalizer
struct ConsoleState {
    stdout: io::Stdout,
    normalizer: ConsoleNormalizer,
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
            state: Mutex::new(ConsoleState {
                stdout: io::stdout(),
                normalizer: ConsoleNormalizer::new(),
            }),
        }
    }

    /// Writes the given content without a newline
    pub fn write(&self, content: impl AsRef<str>) -> io::Result<()> {
        self.write_internal(content, false)
    }

    /// Writes the given content with a newline
    pub fn writeln(&self, content: impl AsRef<str>) -> io::Result<()> {
        self.write_internal(content, true)
    }

    /// Internal write implementation that handles both write and writeln cases
    fn write_internal(&self, content: impl AsRef<str>, add_newline: bool) -> io::Result<()> {
        let content = content.as_ref();
        let mut state = self.state.lock().unwrap();

        // Handle empty string cases
        if content.is_empty() {
            if !add_newline {
                state.normalizer.reset();
                return Ok(());
            }
            state.normalizer.reset();
            return Ok(());
        }

        // Normalize the content
        let normalized = state.normalizer.normalize(content, add_newline);

        // Write and flush
        write!(state.stdout, "{}", normalized)?;
        state.stdout.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Console {
        fn last_output(&self) -> String {
            self.state
                .lock()
                .unwrap()
                .normalizer
                .last_text()
                .to_string()
        }
    }

    mod basic_operations {
        use super::*;

        #[test]
        fn write() {
            let console = Console::new();
            console.write("Hello").unwrap();
            assert_eq!(console.last_output(), "Hello");
        }

        #[test]
        fn writeln() {
            let console = Console::new();
            console.writeln("World").unwrap();
            assert_eq!(console.last_output(), "World\n");
        }

        #[test]
        fn write_empty() {
            let console = Console::new();
            console.write("").unwrap();
            assert_eq!(console.last_output(), "");
        }

        #[test]
        fn writeln_empty() {
            let console = Console::new();
            console.writeln("").unwrap();
            assert_eq!(console.last_output(), "");
        }
    }

    mod sequences {
        use super::*;

        #[test]
        fn multiple_writes() {
            let console = Console::new();
            console.write("line1").unwrap();
            console.write("line2").unwrap();
            assert_eq!(console.last_output(), "line2");
        }

        #[test]
        fn write_then_writeln() {
            let console = Console::new();
            console.write("Hello").unwrap();
            console.writeln("World").unwrap();
            assert_eq!(console.last_output(), "World\n");
        }

        #[test]
        fn multiple_writeln() {
            let console = Console::new();
            console.writeln("line1").unwrap();
            console.writeln("line2").unwrap();
            assert_eq!(console.last_output(), "line2\n");
        }
    }

    mod newline_handling {
        use super::*;

        #[test]
        fn write_with_newlines() {
            let console = Console::new();
            console.write("line1\n\n").unwrap();
            assert_eq!(console.last_output(), "line1\n\n");
        }

        #[test]
        fn writeln_with_newlines() {
            let console = Console::new();
            console.writeln("line1\n").unwrap();
            assert_eq!(console.last_output(), "line1\n\n");
        }

        #[test]
        fn leading_newlines() {
            let console = Console::new();
            console.write("\n\ntext").unwrap();
            assert_eq!(console.last_output(), "\n\ntext");
        }
    }

    mod state_handling {
        use super::*;

        #[test]
        fn consecutive_empty_writes() {
            let console = Console::new();
            console.write("").unwrap();
            console.write("").unwrap();
            assert_eq!(console.last_output(), "");
        }

        #[test]
        fn write_after_newlines() {
            let console = Console::new();
            console.write("text\n\n").unwrap();
            console.write("more").unwrap();
            assert_eq!(console.last_output(), "more");
        }

        #[test]
        fn writeln_after_newlines() {
            let console = Console::new();
            console.write("text\n\n").unwrap();
            console.writeln("more").unwrap();
            assert_eq!(console.last_output(), "more\n");
        }
    }
}
