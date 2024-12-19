use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tree_sitter::{Language, Parser, Query, QueryCursor};

mod error;
mod queries;

use error::{Error, Result};
use queries::{javascript, python, rust};

fn load_language_parser(language_name: &str) -> Result<Language> {
    match language_name {
        "rust" => Ok(tree_sitter_rust::language()),
        "javascript" => Ok(tree_sitter_javascript::language()),
        "python" => Ok(tree_sitter_python::language()),
        x => Err(Error::LanguageError(format!("Unsupported language: {}", x))),
    }
}

fn load_queries() -> HashMap<&'static str, &'static str> {
    let mut queries = HashMap::new();
    queries.insert("rust", rust::QUERY);
    queries.insert("javascript", javascript::QUERY);
    queries.insert("python", python::QUERY);
    queries
}

fn parse_file(_file: &Path, content: &str, parser: &mut Parser, query: &Query) -> Option<String> {
    let tree = parser.parse(content, None)?;

    let mut cursor = QueryCursor::new();
    let mut captures: Vec<_> = cursor
        .matches(query, tree.root_node(), content.as_bytes())
        .flat_map(|m| m.captures)
        .filter_map(|capture| {
            let capture_name = &query.capture_names()[capture.index as usize];
            if !capture_name.contains("name") {
                return None;
            }
            let node = capture.node;
            let parent = node.parent()?;
            // Skip impl blocks and other non-definition nodes
            if parent.kind() == "impl_item" {
                return None;
            }
            Some((node.start_position().row, parent))
        })
        .collect();

    // Sort captures by their start position
    captures.sort_by_key(|&(row, _)| row);

    let lines: Vec<&str> = content.lines().collect();
    let mut formatted_output = String::new();
    let mut last_line = -1;
    let mut seen_lines = HashSet::new();

    for (start_line, _) in captures {
        // Skip if we've already processed this line
        if !seen_lines.insert(start_line) {
            continue;
        }

        // Add separator if there's a gap between captures
        if last_line != -1 && start_line as i32 > last_line + 1 {
            formatted_output.push_str("|----\n");
        }

        // Only add the first line of the definition
        if let Some(line) = lines.get(start_line) {
            formatted_output.push_str(&format!("│{}\n", line.trim()));
        }

        last_line = start_line as i32;
    }

    if formatted_output.is_empty() {
        None
    } else {
        Some(formatted_output)
    }
}

/// Parse source code files provided as a vector of tuples containing file paths and their contents
pub fn parse_source_code_for_definitions(files: Vec<(PathBuf, String)>) -> Result<String> {
    let extensions_to_languages =
        HashMap::from([("rs", "rust"), ("js", "javascript"), ("py", "python")]);

    let queries = load_queries();
    let mut parsers: HashMap<&str, (Parser, Query)> = HashMap::new();
    let mut result = String::new();

    for (file, content) in files {
        if let Some(ext) = file.extension().and_then(|e| e.to_str()) {
            if let Some(&lang_name) = extensions_to_languages.get(ext.to_lowercase().as_str()) {
                if !parsers.contains_key(lang_name) {
                    let language = load_language_parser(lang_name)?;
                    let mut parser = Parser::new();
                    parser
                        .set_language(language)
                        .map_err(|e| Error::LanguageError(e.to_string()))?;
                    let query = Query::new(language, queries[lang_name])
                        .map_err(|e| Error::QueryError(e.to_string()))?;
                    parsers.insert(lang_name, (parser, query));
                }

                if let Some((parser, query)) = parsers.get_mut(lang_name) {
                    if let Some(file_output) = parse_file(&file, &content, parser, query) {
                        if !result.is_empty() {
                            result.push_str("|----\n");
                        }
                        result.push_str(&format!(
                            "{}\n",
                            file.file_name().unwrap().to_string_lossy()
                        ));
                        result.push_str(&file_output);
                    }
                }
            }
        }
    }

    if result.is_empty() {
        Ok("No source code definitions found.".into())
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_snapshot;

    #[test]
    fn test_empty_directory() {
        let files = Vec::new();
        let result = parse_source_code_for_definitions(files).unwrap();
        assert_snapshot!(result);
    }

    #[test]
    fn test_unsupported_files() {
        let mut files = Vec::new();
        files.push((PathBuf::from("test.txt"), "Some content".to_string()));
        let result = parse_source_code_for_definitions(files).unwrap();
        assert_snapshot!(result);
    }

    #[test]
    fn test_rust_definitions() {
        let mut files = Vec::new();
        let rust_content = r#"
                                    struct User {
                                        name: String,
                                        age: u32,
                                    }

                                    fn calculate_age(birth_year: u32) -> u32 {
                                        2024 - birth_year
                                    }

                                    impl User {
                                        fn new(name: String, age: u32) -> Self {
                                            User { name, age }
                                        }
                                    }
                                    "#;
        files.push((PathBuf::from("test.rs"), rust_content.to_string()));
        let result = parse_source_code_for_definitions(files).unwrap();
        assert_snapshot!(result);
    }

    #[test]
    fn test_javascript_definitions() {
        let mut files = Vec::new();
        let js_content = r#"
            function calculateTotal(items) {
                return items.reduce((sum, item) => sum + item.price, 0);
            }

            function formatPrice(price) {
                return `$${price.toFixed(2)}`;
            }
            "#;
        files.push((PathBuf::from("test.js"), js_content.to_string()));
        let result = parse_source_code_for_definitions(files).unwrap();
        assert_snapshot!(result);
    }

    #[test]
    fn test_multiple_file_types() {
        let mut files = Vec::new();
        files.push((
            PathBuf::from("test.rs"),
            "fn test_function() {}".to_string(),
        ));
        files.push((
            PathBuf::from("test.js"),
            "function jsFunction() {}".to_string(),
        ));
        files.push((PathBuf::from("test.txt"), "plain text".to_string()));

        let result = parse_source_code_for_definitions(files).unwrap();
        assert_snapshot!(result);
    }
}
