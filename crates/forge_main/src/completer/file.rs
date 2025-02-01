use std::path::PathBuf;

use forge_walker::Walker;
use reedline::{Completer, Span, Suggestion};

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
    fn complete(&mut self, line: &str, _: usize) -> Vec<Suggestion> {
        if let Some(search_term) = line.split_whitespace().last() {
            let last_space_pos = line.rfind(' ').map(|a| a + 1).unwrap_or(0);
            let files = self.walker.get_blocking().unwrap_or_default();
            files
                .into_iter()
                .filter(|file| !file.is_dir())
                .filter_map(|file| {
                    if !search_term.is_empty()
                        && file
                            .file_name
                            .as_ref()
                            .map_or_else(|| false, |file| file.contains(search_term))
                    {
                        Some(Suggestion {
                            value: file.path,
                            description: None,
                            style: None,
                            extra: None,
                            span: Span::new(last_space_pos, line.len()),
                            append_whitespace: true,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            // No completion for other inputs
            Vec::new()
        }
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
    }
}
