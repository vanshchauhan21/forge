use std::path::Path;

use tree_sitter::{Language, Parser};

/// Maps file extensions to their corresponding Tree-sitter language parsers.
///
/// This function takes a file extension as input and returns the appropriate
/// Tree-sitter language parser if supported.
///
/// # Arguments
/// * `ext` - The file extension to get a language parser for
///
/// # Returns
/// * `Some(Language)` - If the extension is supported
/// * `None` - If the extension is not supported
///
/// # Supported Languages
/// * Rust (.rs)
/// * JavaScript/TypeScript (.js, .jsx, .ts, .tsx)
/// * Python (.py)
pub fn extension(ext: &str) -> Option<Language> {
    match ext.to_lowercase().as_str() {
        "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
        "js" | "jsx" | "ts" | "tsx" => Some(tree_sitter_javascript::LANGUAGE.into()),
        "py" => Some(tree_sitter_python::LANGUAGE.into()),
        _ => None,
    }
}

/// Validates source code content using Tree-sitter parsers.
///
/// This function attempts to parse the provided content using a Tree-sitter
/// parser appropriate for the file's extension. It checks for syntax errors in
/// the parsed abstract syntax tree.
///
/// # Arguments
/// * `path` - The path to the file being validated (used to determine language)
/// * `content` - The source code content to validate
///
/// # Returns
/// * `Ok(())` - If the content is valid for the given language
/// * `Err(String)` - If validation fails, contains error description
///
/// # Note
/// Files with unsupported extensions are considered valid and will return
/// Ok(()). Files with no extension will return an error.
pub fn validate(path: impl AsRef<Path>, content: &str) -> Result<(), String> {
    let path = path.as_ref();

    // Get file extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| "File has no extension".to_string())?;

    // Get language for the extension
    let language = if let Some(lang) = extension(ext) {
        lang
    } else {
        // If we don't support the language, consider it valid
        return Ok(());
    };

    // Initialize parser
    let mut parser = Parser::new();
    parser.set_language(&language).map_err(|e| e.to_string())?;

    // Try parsing the content
    let tree = parser
        .parse(content, None)
        .ok_or_else(|| format!("Failed to parse {} as {}", path.display(), ext))?;

    // Find syntax errors in the tree
    let root_node = tree.root_node();
    if root_node.has_error() || root_node.is_error() {
        Err(format!(
            "Syntax error found in {} while parsing",
            path.display(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    // Include language samples
    const RUST_VALID: &str = include_str!("lang/rust/valid.rs");
    const RUST_INVALID: &str = include_str!("lang/rust/invalid.rs");
    const JAVASCRIPT_VALID: &str = include_str!("lang/javascript/valid.js");
    const JAVASCRIPT_INVALID: &str = include_str!("lang/javascript/invalid.js");
    const PYTHON_VALID: &str = include_str!("lang/python/valid.py");
    const PYTHON_INVALID: &str = include_str!("lang/python/invalid.py");

    #[test]
    fn test_rust_valid() {
        let path = PathBuf::from("test.rs");
        assert!(validate(&path, RUST_VALID).is_ok());
    }

    #[test]
    fn test_rust_invalid() {
        let path = PathBuf::from("test.rs");
        assert!(validate(&path, RUST_INVALID).is_err());
    }

    #[test]
    fn test_javascript_valid() {
        let path = PathBuf::from("test.js");
        assert!(validate(&path, JAVASCRIPT_VALID).is_ok());
    }

    #[test]
    fn test_javascript_invalid() {
        let path = PathBuf::from("test.js");
        assert!(validate(&path, JAVASCRIPT_INVALID).is_err());
    }

    #[test]
    fn test_python_valid() {
        let path = PathBuf::from("test.py");
        assert!(validate(&path, PYTHON_VALID).is_ok());
    }

    #[test]
    fn test_python_invalid() {
        let path = PathBuf::from("test.py");
        assert!(validate(&path, PYTHON_INVALID).is_err());
    }

    #[test]
    fn test_unsupported_extension() {
        let content = "Some random content";
        let path = PathBuf::from("test.txt");
        assert!(validate(&path, content).is_ok());
    }

    #[test]
    fn test_no_extension() {
        let content = "Some random content";
        let path = PathBuf::from("test");
        assert!(validate(&path, content).is_err());
    }
}
