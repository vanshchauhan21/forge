/// Handles normalization of console output with respect to newlines
#[derive(Debug, Default)]
pub struct NewLine {
    /// The last normalized output
    pub(crate) last_output: String,
    trailing_newlines: usize,
}

impl NewLine {
    /// Creates a new ConsoleNormalizer instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Resets the normalizer state
    pub(crate) fn reset(&mut self) {
        self.last_output.clear();
        self.trailing_newlines = 0;
    }

    /// Returns the last normalized output
    #[cfg(test)]
    pub fn last_text(&self) -> &str {
        &self.last_output
    }

    /// Count the number of trailing newlines in a string
    pub(crate) fn count_trailing_newlines(s: &str) -> usize {
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
            } else if c != '\r' {
                // Skip \r as we'll normalize to \n only
                consecutive_newlines = 0;
                result.push(c);
            }
        }
        result
    }

    /// Normalizes content and updates internal state
    pub fn normalize(&mut self, content: &str, add_newline: bool) -> String {
        let mut result = if self.trailing_newlines > 0 && content.starts_with('\n') {
            // If we already have trailing newlines and content starts with newline,
            // we need to consider the existing newlines
            let additional_newlines = Self::count_trailing_newlines(content);
            let total_newlines = (self.trailing_newlines + additional_newlines).min(2);

            if total_newlines > self.trailing_newlines {
                // Only add the difference in newlines
                let extra_newlines = total_newlines - self.trailing_newlines;
                format!(
                    "{}{}",
                    "\n".repeat(extra_newlines),
                    content.trim_start_matches(&['\n', '\r'][..])
                )
            } else {
                content.trim_start_matches(&['\n', '\r'][..]).to_string()
            }
        } else {
            Self::normalize_newlines(content)
        };

        if add_newline {
            let current_newlines = Self::count_trailing_newlines(&result);
            if current_newlines > 0 {
                // Remove existing trailing newlines first
                result.truncate(result.len() - current_newlines);
            }
            // Add the correct number of newlines (including the one we're adding)
            let total_newlines = (current_newlines + 1).min(2);
            result.push_str(&"\n".repeat(total_newlines));
        }

        // Update state
        self.last_output = result.clone();
        self.trailing_newlines = Self::count_trailing_newlines(&result);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod newline_normalization {
        use super::*;

        #[test]
        fn basic_text() {
            assert_eq!(NewLine::normalize_newlines("abc"), "abc");
            assert_eq!(NewLine::normalize_newlines("abc\n"), "abc\n");
            assert_eq!(NewLine::normalize_newlines("abc\r\n"), "abc\n");
        }

        #[test]
        fn consecutive_newlines() {
            // Should limit to maximum of 2 newlines
            assert_eq!(NewLine::normalize_newlines("abc\n\n\n"), "abc\n\n");
            assert_eq!(NewLine::normalize_newlines("abc\n\n\n\n\n"), "abc\n\n");
            assert_eq!(NewLine::normalize_newlines("\n\n\n"), "\n\n");
        }

        #[test]
        fn mixed_content() {
            assert_eq!(
                NewLine::normalize_newlines("line1\n\nline2\n\n\nline3"),
                "line1\n\nline2\n\nline3"
            );
            assert_eq!(
                NewLine::normalize_newlines("line1\r\n\r\nline2"),
                "line1\n\nline2"
            );
        }

        #[test]
        fn leading_and_trailing() {
            assert_eq!(NewLine::normalize_newlines("\n\nabc"), "\n\nabc");
            assert_eq!(NewLine::normalize_newlines("abc\n\n"), "abc\n\n");
            assert_eq!(NewLine::normalize_newlines("\n\nabc\n\n"), "\n\nabc\n\n");
        }
    }

    mod state_tracking {
        use super::*;

        #[test]
        fn basic_state() {
            let normalizer = NewLine::new();
            assert_eq!(normalizer.trailing_newlines, 0);
            assert_eq!(normalizer.last_text(), "");
        }

        #[test]
        fn state_after_normalize() {
            let mut normalizer = NewLine::new();

            // Single line
            normalizer.normalize("abc", false);
            assert_eq!(normalizer.trailing_newlines, 0);
            assert_eq!(normalizer.last_text(), "abc");

            // With newlines
            normalizer.normalize("def\n\n", false);
            assert_eq!(normalizer.trailing_newlines, 2);
            assert_eq!(normalizer.last_text(), "def\n\n");
        }

        #[test]
        fn state_after_writeln() {
            let mut normalizer = NewLine::new();

            // Normal writeln
            normalizer.normalize("abc", true);
            assert_eq!(normalizer.trailing_newlines, 1);
            assert_eq!(normalizer.last_text(), "abc\n");

            // Writeln with existing newlines
            normalizer.normalize("def\n", true);
            assert_eq!(normalizer.trailing_newlines, 2);
            assert_eq!(normalizer.last_text(), "def\n\n");
        }

        #[test]
        fn state_reset() {
            let mut normalizer = NewLine::new();

            normalizer.normalize("abc\n\n", false);
            assert_eq!(normalizer.trailing_newlines, 2);

            normalizer.reset();
            assert_eq!(normalizer.trailing_newlines, 0);
            assert_eq!(normalizer.last_text(), "");
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn empty_content() {
            let mut normalizer = NewLine::new();

            let output = normalizer.normalize("", false);
            assert_eq!(output, "");
            assert_eq!(normalizer.last_text(), "");
            assert_eq!(normalizer.trailing_newlines, 0);
        }

        #[test]
        fn only_newlines() {
            let mut normalizer = NewLine::new();

            let output = normalizer.normalize("\n\n\n", false);
            assert_eq!(output, "\n\n");
            assert_eq!(normalizer.last_text(), "\n\n");
            assert_eq!(normalizer.trailing_newlines, 2);
        }

        #[test]
        fn carriage_returns() {
            let mut normalizer = NewLine::new();

            let output = normalizer.normalize("text\r\n\r\n", false);
            assert_eq!(output, "text\n\n");
            assert_eq!(normalizer.last_text(), "text\n\n");
            assert_eq!(normalizer.trailing_newlines, 2);
        }

        #[test]
        fn max_newlines() {
            let mut normalizer = NewLine::new();

            normalizer.normalize("abc\n\n", false);
            assert_eq!(normalizer.trailing_newlines, 2);

            // When at max newlines, writeln should still add one newline
            let output = normalizer.normalize("def", true);
            assert_eq!(output, "def\n");
            assert_eq!(normalizer.trailing_newlines, 1);
        }
    }

    mod trailing_newlines {
        use super::*;

        #[test]
        fn count_basic() {
            assert_eq!(NewLine::count_trailing_newlines("abc"), 0);
            assert_eq!(NewLine::count_trailing_newlines("abc\n"), 1);
        }

        #[test]
        fn count_multiple() {
            assert_eq!(NewLine::count_trailing_newlines("abc\n\n"), 2);
            assert_eq!(NewLine::count_trailing_newlines("abc\n\n\n"), 3);
        }

        #[test]
        fn count_with_carriage_returns() {
            assert_eq!(NewLine::count_trailing_newlines("abc\r\n"), 1);
            assert_eq!(NewLine::count_trailing_newlines("abc\n\r\n"), 2);
            assert_eq!(NewLine::count_trailing_newlines("abc\r\n\r\n"), 2);
        }

        #[test]
        fn count_mixed() {
            assert_eq!(NewLine::count_trailing_newlines("abc\n\r"), 1);
            assert_eq!(NewLine::count_trailing_newlines("abc\r\n\n"), 2);
            assert_eq!(NewLine::count_trailing_newlines("abc\n\r\n\r"), 2);
        }
    }
}
