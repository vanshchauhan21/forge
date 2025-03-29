use std::collections::HashSet;

#[derive(
    Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Hash,
)]
pub struct Attachment {
    pub content: String,
    pub path: String,
    pub content_type: ContentType,
}

#[derive(
    Debug, schemars::JsonSchema, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq, Hash,
)]
pub enum ContentType {
    Image,
    Text,
}

impl Attachment {
    /// Parses a string and extracts all file paths prefixed with "@".
    /// File paths can contain spaces and are considered to extend until the
    /// next whitespace. When a file path contains spaces, the entire path
    /// should be wrapped in quotes.
    pub fn parse_all<T: ToString>(v: T) -> HashSet<String> {
        let v = v.to_string();
        let mut paths = HashSet::new();
        let mut i = 0;

        while i < v.len() {
            let remaining = &v[i..];

            if let Some(pos) = remaining.find('@') {
                i += pos + 1; // Move past the '@'

                if i >= v.len() {
                    break;
                }

                let path_start = i;
                let path_end;

                // Check if the path is quoted (for paths with spaces)
                if i < v.len() && v[i..].starts_with('\"') {
                    i += 1; // Move past the opening quote
                    let path_start_after_quote = i;

                    // Find the closing quote
                    if let Some(end_quote) = v[i..].find('\"') {
                        path_end = i + end_quote;
                        let file_path = v[path_start_after_quote..path_end].to_string();

                        // Add the file path to the set
                        paths.insert(file_path);

                        i = path_end + 1; // Move past the closing quote
                    } else {
                        // If no closing quote, consider the rest of the string as path
                        path_end = v.len();
                        let file_path = v[path_start_after_quote..path_end].to_string();

                        // Add the file path to the set
                        paths.insert(file_path);

                        i = path_end;
                    }
                    continue; // Skip the common path handling code since we've
                              // already added the attachment
                } else {
                    // For unquoted paths, the path extends until the next whitespace
                    if let Some(end_pos) = v[i..].find(char::is_whitespace) {
                        path_end = i + end_pos;
                        i = path_end; // Move to the whitespace
                    } else {
                        // If no whitespace, consider the rest of the string as path
                        path_end = v.len();
                        i = path_end;
                    }
                }

                let file_path = if path_start < path_end {
                    v[path_start..path_end].to_string()
                } else {
                    continue;
                };

                // Add the file path to the set
                paths.insert(file_path);
            } else {
                break;
            }
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_parse_all_empty() {
        let text = String::from("No attachments here");
        let attachments = Attachment::parse_all(text);
        assert!(attachments.is_empty());
    }

    #[test]
    fn test_attachment_parse_all_simple() {
        let text = String::from("Check this file @/path/to/file.txt");
        let paths = Attachment::parse_all(text);
        assert_eq!(paths.len(), 1);

        let path_found = paths.iter().next().unwrap();
        assert_eq!(path_found, "/path/to/file.txt");
    }

    #[test]
    fn test_attachment_parse_all_with_spaces() {
        let text = String::from("Check this file @\"/path/with spaces/file.txt\"");
        let paths = Attachment::parse_all(text);
        assert_eq!(paths.len(), 1);

        let path_found = paths.iter().next().unwrap();
        assert_eq!(path_found, "/path/with spaces/file.txt");
    }

    #[test]
    fn test_attachment_parse_all_multiple() {
        let text = String::from(
            "Check @/file1.txt and also @\"/path/with spaces/file2.txt\" and @/file3.txt",
        );
        let paths = Attachment::parse_all(text);
        assert_eq!(paths.len(), 3);

        assert!(paths.contains("/file1.txt"));
        assert!(paths.contains("/path/with spaces/file2.txt"));
        assert!(paths.contains("/file3.txt"));
    }

    #[test]
    fn test_attachment_parse_all_at_end() {
        let text = String::from("Check this file @");
        let paths = Attachment::parse_all(text);
        assert_eq!(paths.len(), 0);
    }

    #[test]
    fn test_attachment_parse_all_unclosed_quote() {
        let text = String::from("Check this file @\"/path/with spaces/unclosed");
        let paths = Attachment::parse_all(text);
        assert_eq!(paths.len(), 1);

        let path_found = paths.iter().next().unwrap();
        assert_eq!(path_found, "/path/with spaces/unclosed");
    }

    #[test]
    fn test_attachment_parse_all_with_multibyte_chars() {
        let text = String::from(
            "Check this file @\"🚀/path/with spaces/file.txt🔥\" and also @🌟simple_path",
        );
        let paths = Attachment::parse_all(text);
        assert_eq!(paths.len(), 2);

        assert!(paths.contains("🚀/path/with spaces/file.txt🔥"));
        assert!(paths.contains("🌟simple_path"));
    }
}
