//! Handles normalization of console output with respect to newlines.
//!
//! This module provides utilities for normalizing text output, particularly
//! focusing on newline handling and buffering.

/// Handles normalization of console output with respect to newlines
#[derive(Debug, Default)]
pub struct NewLine {
    /// Count of consecutive newlines so far
    newline_count: u32,
}

impl NewLine {
    /// Creates a new ConsoleNormalizer instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Normalizes content to have at most two consecutive newlines
    pub fn normalize(&mut self, content: &str) -> String {
        let content = content.replace("\r\n", "\n");
        if content.is_empty() {
            return String::new();
        }

        // Replace multiple newlines with maximum of two
        let mut output = String::new();
        let mut newline_count = self.newline_count;

        for c in content.chars() {
            if c == '\n' {
                newline_count += 1;
                if newline_count <= 2 {
                    output.push(c);
                }
            } else {
                newline_count = 0;
                output.push(c);
            }
        }

        self.newline_count = newline_count;

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A writer used for testing newline normalization
    struct Writer {
        /// The newline normalizer
        normalizer: NewLine,
        /// The internal buffer for storing written text
        buffer: String,
    }

    impl Writer {
        /// Creates a new Writer instance
        fn new() -> Self {
            Self { normalizer: NewLine::new(), buffer: String::new() }
        }

        /// Writes text to the buffer, normalizing newlines
        fn write(&mut self, content: &str) {
            let normalized = self.normalizer.normalize(content);
            self.buffer.push_str(&normalized);
        }

        /// Returns the contents of the buffer
        fn buffer(&self) -> &str {
            &self.buffer
        }
    }

    #[test]
    fn write_and_buffer() {
        let mut writer = Writer::new();
        writer.write("Hello\n\n\n");
        assert_eq!(writer.buffer(), "Hello\n\n");

        writer.write("World\n\n\n");
        assert_eq!(writer.buffer(), "Hello\n\nWorld\n\n");
    }

    #[test]
    fn empty_write() {
        let mut writer = Writer::new();
        writer.write("");
        assert_eq!(writer.buffer(), "");
    }

    #[test]
    fn windows_line_endings() {
        let mut writer = Writer::new();
        writer.write("abc\r\n");
        assert_eq!(writer.buffer(), "abc\n");
        writer.write("\r\n\r\ndef");
        assert_eq!(writer.buffer(), "abc\n\ndef");
        writer.write("abc\r\n\r\n");
        assert_eq!(writer.buffer(), "abc\n\ndefabc\n\n");
    }

    #[test]
    fn mixed_line_endings() {
        let mut writer = Writer::new();
        writer.write("abc\r\n\n");
        assert_eq!(writer.buffer(), "abc\n\n");
        writer.write("\n\r\ndef");
        assert_eq!(writer.buffer(), "abc\n\ndef");
        writer.write("\r\n\n\r\n");
        assert_eq!(writer.buffer(), "abc\n\ndef\n\n");
    }

    #[test]
    fn preserve_content_between_newlines() {
        let mut writer = Writer::new();
        writer.write("abc\n\ndef\n\nghi");
        assert_eq!(writer.buffer(), "abc\n\ndef\n\nghi");

        writer.write("\n\njkl\n\nmno");
        assert_eq!(writer.buffer(), "abc\n\ndef\n\nghi\n\njkl\n\nmno");
    }

    #[test]
    fn only_newlines() {
        let mut writer = Writer::new();
        writer.write("\n\n\n");
        assert_eq!(writer.buffer(), "\n\n");
        writer.write("\n\n\n\n");
        assert_eq!(writer.buffer(), "\n\n");
    }

    #[test]
    fn complex_newline_sequence() {
        let mut writer = Writer::new();
        writer.write("abc\n\n");
        assert_eq!(writer.buffer(), "abc\n\n");
        writer.write("\n\n\ndef\n\n\n");
        assert_eq!(writer.buffer(), "abc\n\ndef\n\n");
        writer.write("\n\n\nghi\n\n\n");
        assert_eq!(writer.buffer(), "abc\n\ndef\n\nghi\n\n");
    }

    #[test]
    fn newlines_at_boundaries() {
        let mut writer = Writer::new();
        writer.write("abc\n\n");
        assert_eq!(writer.buffer(), "abc\n\n");
        writer.write("\n\ndef");
        assert_eq!(writer.buffer(), "abc\n\ndef");
        writer.write("\n\nghi");
        assert_eq!(writer.buffer(), "abc\n\ndef\n\nghi");
    }

    #[test]
    fn consecutive_outputs() {
        let mut writer = Writer::new();
        writer.write("abc\n");
        assert_eq!(writer.buffer(), "abc\n");
        writer.write("def");
        assert_eq!(writer.buffer(), "abc\ndef");
        writer.write("\n\n\nghi");
        assert_eq!(writer.buffer(), "abc\ndef\n\nghi");
    }

    #[test]
    fn newlines_between_outputs() {
        let mut writer = Writer::new();
        writer.write("abc\n");
        assert_eq!(writer.buffer(), "abc\n");
        writer.write("\n\n\ndef");
        assert_eq!(writer.buffer(), "abc\n\ndef");
        writer.write("\n\n\nghi");
        assert_eq!(writer.buffer(), "abc\n\ndef\n\nghi");
    }

    #[test]
    fn empty_newlines() {
        let mut writer = Writer::new();
        writer.write("abc\n");
        assert_eq!(writer.buffer(), "abc\n");
        writer.write("\n");
        assert_eq!(writer.buffer(), "abc\n\n");
        writer.write("\n");
        assert_eq!(writer.buffer(), "abc\n\n");
        writer.write("\n\n\ndef");
        assert_eq!(writer.buffer(), "abc\n\ndef");
    }
}
