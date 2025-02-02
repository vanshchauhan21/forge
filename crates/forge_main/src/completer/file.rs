use std::path::PathBuf;

use forge_walker::Walker;
use reedline::{Completer, Span, Suggestion};
use tracing::info;

#[derive(Clone)]
pub struct FileCompleter {
    walker: Walker,
}

impl FileCompleter {
    pub fn new(cwd: PathBuf) -> Self {
        let walker = Walker::max_all().cwd(cwd).skip_binary(true);
        Self { walker }
    }
}

impl Completer for FileCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        info!("Completing line: '{}' pos: {}", line, pos);

        // Handle empty or whitespace-only input
        if line.trim().is_empty() {
            return Vec::new();
        }

        // Extract leading and trailing spaces
        let leading_spaces = line.len() - line.trim_start().len();
        let trailing_spaces = line.len() - line.trim_end().len();

        // Calculate span based on word position and spaces
        let (search_term, span) =
            if line[leading_spaces..line.len() - trailing_spaces].contains(' ') {
                // Multiple words after leading spaces
                let last_space_idx = line.rfind(' ').unwrap();
                let after_space = &line[last_space_idx + 1..];

                // Skip trailing spaces
                if after_space.trim().is_empty() {
                    return Vec::new();
                }

                (
                    after_space.trim(),
                    Span::new(last_space_idx + 1, line.len()),
                )
            } else {
                // Single word (with possible leading/trailing spaces)
                let term = line.trim();
                if term.is_empty() {
                    return Vec::new();
                }
                (term, Span::new(0, line.len()))
            };

        info!("Search term: '{}', Span: {:?}", search_term, span);
        
        let files = self.walker.get_blocking().unwrap_or_default();
        files
            .into_iter()
            .filter(|file| !file.is_dir())
            .filter_map(|file| {
                if let Some(file_name) = file.file_name.as_ref() {
                    if file_name.contains(search_term) {
                        Some(Suggestion {
                            value: file.path,
                            description: None,
                            style: None,
                            extra: None,
                            span,
                            append_whitespace: true,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_no_completion_for_regular_text() {
        let mut completer = FileCompleter::new(PathBuf::from("."));
        let suggestions = completer.complete("regular", 0);

        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_file_completion() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let mut completer = FileCompleter::new(dir.path().to_path_buf());
        let suggestions = completer.complete("open test", 0);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "test.txt");
        assert_eq!(suggestions[0].description, None);
        // Verify span starts after "open " and covers "test"
        assert_eq!(suggestions[0].span, Span::new(5, 9));
    }

    #[test]
    fn test_file_completion_empty() {
        let dir = tempdir().unwrap();
        let mut completer = FileCompleter::new(dir.path().to_path_buf());
        let suggestions = completer.complete("open ", 0);

        // Should list all files/directories in the empty temp directory
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_file_completion_multiple_words() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let mut completer = FileCompleter::new(dir.path().to_path_buf());
        let suggestions = completer.complete("some file test", 0);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "test.txt");
        assert_eq!(suggestions[0].description, None);
        // Verify span starts after "some file " and covers "test"
        assert_eq!(suggestions[0].span, Span::new(10, 14));
    }

    #[test]
    fn test_span_with_leading_spaces() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let mut completer = FileCompleter::new(dir.path().to_path_buf());
        let suggestions = completer.complete("   test", 0);

        assert_eq!(suggestions.len(), 1);
        // Should handle leading spaces correctly
        assert_eq!(suggestions[0].span, Span::new(0, 7));
    }

    #[test]
    fn test_span_with_multiple_spaces() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let mut completer = FileCompleter::new(dir.path().to_path_buf());
        let suggestions = completer.complete("open   test", 0);

        assert_eq!(suggestions.len(), 1);
        // Should handle multiple spaces correctly
        assert_eq!(suggestions[0].span, Span::new(7, 11));
    }

    #[test]
    fn test_span_with_trailing_spaces() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let mut completer = FileCompleter::new(dir.path().to_path_buf());
        let suggestions = completer.complete("test   ", 0);

        assert_eq!(suggestions.len(), 1);
        // Should handle trailing spaces correctly
        assert_eq!(suggestions[0].span, Span::new(0, 7));
    }

    #[test]
    fn test_span_with_partial_filename() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("testfile.txt");
        File::create(&file_path).unwrap();

        let mut completer = FileCompleter::new(dir.path().to_path_buf());
        let suggestions = completer.complete("open test", 0);

        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].value, "testfile.txt");
        // Should cover only the partial match
        assert_eq!(suggestions[0].span, Span::new(5, 9));
    }

    #[test]
    fn test_span_with_single_word() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let mut completer = FileCompleter::new(dir.path().to_path_buf());
        let suggestions = completer.complete("test", 0);

        assert_eq!(suggestions.len(), 1);
        // For single word, span should cover the entire input
        assert_eq!(suggestions[0].span, Span::new(0, 4));
    }

    #[test]
    fn test_span_with_different_lengths() {
        let dir = tempdir().unwrap();
        // Create files with names of different lengths
        File::create(dir.path().join("short.txt")).unwrap();
        File::create(dir.path().join("very_long_name.txt")).unwrap();

        let mut completer = FileCompleter::new(dir.path().to_path_buf());

        // Test for shorter filename
        let suggestions_short = completer.complete("open sho", 0);
        assert_eq!(suggestions_short.len(), 1);
        assert_eq!(suggestions_short[0].span, Span::new(5, 8));

        // Test for longer filename
        let suggestions_long = completer.complete("open ver", 0);
        assert_eq!(suggestions_long.len(), 1);
        assert_eq!(suggestions_long[0].span, Span::new(5, 8));
    }
}
