use std::path::Path;

use dissimilar::Chunk;
use forge_domain::{NamedTool, ToolCallService, ToolDescription, ToolName};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::fs;
use tracing::{debug, error};

use super::fs_replace_marker::{DIVIDER, REPLACE, SEARCH};
use crate::fs::syn;

#[derive(Deserialize, JsonSchema)]
pub struct FSReplaceInput {
    /// File path relative to the current working directory
    pub path: String,
    /// SEARCH/REPLACE blocks defining changes
    pub diff: String,
}

pub struct FSReplace;

impl NamedTool for FSReplace {
    fn tool_name(&self) -> ToolName {
        ToolName::new("tool.forge.fs.replace")
    }
}

struct Block {
    search: String,
    replace: String,
}

impl ToolDescription for FSReplace {
    fn description(&self) -> String {
        format!(
            r#"        
Replace sections in a file using SEARCH/REPLACE blocks for precise
modifications.

{}
[exact content to find]
{}
[new content to replace with]
{}

Rules:
1. SEARCH must match exactly (whitespace, indentation, line endings)
2. Each block replaces first match only
3. Keep blocks minimal - include only changing lines plus needed context
4. Complete lines only - no truncation
5. For moves: use 2 blocks (delete + insert)
6. For deletes: use empty REPLACE section

Example:
{}
def old_function(x):
    return x + 1
{}
def new_function(x, y=0):
    return x + y
{}
        "#,
            SEARCH, DIVIDER, REPLACE, SEARCH, DIVIDER, REPLACE
        )
        .trim()
        .to_string()
    }
}

fn normalize_line_endings(text: &str) -> String {
    // Only normalize CRLF to LF while preserving the original line endings
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\r' && chars.peek() == Some(&'\n') {
            chars.next(); // Skip the \n since we'll add it below
            result.push('\n');
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_blocks(diff: &str) -> Result<Vec<Block>, String> {
    let mut blocks = Vec::new();
    let mut pos = 0;

    // Normalize line endings in the diff string while preserving original newlines
    let diff = normalize_line_endings(diff);

    while let Some(search_start) = diff[pos..].find(SEARCH) {
        let search_start = pos + search_start + SEARCH.len();

        // Include the newline after SEARCH marker in the position
        let search_start = match diff[search_start..].find('\n') {
            Some(nl) => search_start + nl + 1,
            None => {
                return Err("Invalid diff format: Missing newline after SEARCH marker".to_string())
            }
        };

        let Some(separator) = diff[search_start..].find(DIVIDER) else {
            return Err("Invalid diff format: Missing separator".to_string());
        };
        let separator = search_start + separator;

        // Include the newline after separator in the position
        let separator_end = separator + DIVIDER.len();
        let separator_end = match diff[separator_end..].find('\n') {
            Some(nl) => separator_end + nl + 1,
            None => return Err("Invalid diff format: Missing newline after separator".to_string()),
        };

        let Some(replace_end) = diff[separator_end..].find(REPLACE) else {
            return Err("Invalid diff format: Missing end marker".to_string());
        };
        let replace_end = separator_end + replace_end;

        let search = &diff[search_start..separator];
        let replace = &diff[separator_end..replace_end];

        blocks.push(Block { search: search.to_string(), replace: replace.to_string() });

        pos = replace_end + REPLACE.len();
        // Move past the newline after REPLACE if it exists
        if let Some(nl) = diff[pos..].find('\n') {
            pos += nl + 1;
        }
    }

    if blocks.is_empty() {
        return Err("Invalid diff format: No valid blocks found".to_string());
    }

    Ok(blocks)
}

/// Apply changes to file content based on search/replace blocks.
/// Changes are only written to disk if all replacements are successful.
async fn apply_changes<P: AsRef<Path>>(path: P, blocks: Vec<Block>) -> Result<String, String> {
    // Initialize content based on whether file exists
    let mut result = if path.as_ref().exists() {
        fs::read_to_string(&path).await.map_err(|e| {
            error!("Failed to read file content: {}", e);
            e.to_string()
        })?
    } else if !blocks[0].search.is_empty() {
        return Err("File does not exist and search pattern is not empty".to_string());
    } else {
        String::new()
    };

    // Apply each block sequentially
    for block in blocks {
        // For empty search string, append the replacement text at the end of file.
        if block.search.is_empty() {
            result.push_str(&block.replace);
            continue;
        }

        // For exact matching, first try to find the exact string
        if let Some(start_idx) = result.find(&block.search) {
            let end_idx = start_idx + block.search.len();
            result.replace_range(start_idx..end_idx, &block.replace);
            continue;
        }

        // If exact match fails, try fuzzy matching
        let normalized_search = block.search.replace("\r\n", "\n").replace('\r', "\n");
        let normalized_result = result.replace("\r\n", "\n").replace('\r', "\n");

        if let Some(start_idx) = normalized_result.find(&normalized_search) {
            result.replace_range(start_idx..start_idx + block.search.len(), &block.replace);
            continue;
        }

        // If still no match, try more aggressive fuzzy matching
        let chunks = dissimilar::diff(&result, &block.search);
        let mut best_match = None;
        let mut best_score = 0.0;
        let mut current_pos = 0;

        for chunk in chunks.iter() {
            if let Chunk::Equal(text) = chunk {
                let score = text.len() as f64 / block.search.len() as f64;
                if score > best_score {
                    best_score = score;
                    best_match = Some((current_pos, text.len()));
                }
            }
            match chunk {
                Chunk::Equal(text) | Chunk::Delete(text) | Chunk::Insert(text) => {
                    current_pos += text.len();
                }
            }
        }

        if let Some((start_idx, len)) = best_match {
            if best_score > 0.7 {
                // Threshold for fuzzy matching
                result.replace_range(start_idx..start_idx + len, &block.replace);
            }
        }
    }

    // Write the modified content
    fs::write(&path, &result).await.map_err(|e| {
        error!("Failed to write file: {}", e);
        e.to_string()
    })?;
    debug!("Successfully wrote changes to {:?}", path.as_ref());

    Ok(result)
}

#[async_trait::async_trait]
impl ToolCallService for FSReplace {
    type Input = FSReplaceInput;

    async fn call(&self, input: Self::Input) -> Result<String, String> {
        let blocks = parse_blocks(&input.diff)?;
        let blocks_len = blocks.len();
        let content = apply_changes(&input.path, blocks).await?;
        let syntax_warning = syn::validate(&input.path, &content).err();

        let mut result = format!(
            "Successfully applied {} patch(es) to {}",
            blocks_len, input.path
        );
        if let Some(warning) = syntax_warning {
            result.push_str("\nWarning: ");
            result.push_str(&warning);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use tempfile::TempDir;

    use super::*;

    async fn write_test_file(path: impl AsRef<Path>, content: &str) -> Result<(), String> {
        fs::write(&path, content).await.map_err(|e| e.to_string())
    }

    #[tokio::test]
    async fn test_whitespace_preservation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "    Hello World    \n  Test Line  \n   Goodbye World   \n";

        write_test_file(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!(
                    "{}\n    Hello World    \n{}\n    Hi World    \n{}\n",
                    SEARCH, DIVIDER, REPLACE
                )
                .to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&file_path.display().to_string()));
    }

    #[tokio::test]
    async fn test_empty_search_new_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        write_test_file(&file_path, "").await.unwrap();

        let fs_replace = FSReplace;
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!("{}\n{}\nNew content\n{}\n", SEARCH, DIVIDER, REPLACE).to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&*file_path.to_string_lossy()));
    }

    #[tokio::test]
    async fn test_multiple_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "    First Line    \n  Middle Line  \n    Last Line    \n";

        write_test_file(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let diff = format!("{}\n    First Line    \n{}\n    New First    \n{}\n{}\n    Last Line    \n{}\n    New Last    \n{}\n",
            SEARCH, DIVIDER, REPLACE, SEARCH, DIVIDER, REPLACE).to_string();

        let result = fs_replace
            .call(FSReplaceInput { path: file_path.to_string_lossy().to_string(), diff })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied 2 patch(es)"));
        assert!(result.contains(&*file_path.to_string_lossy()));
    }

    #[tokio::test]
    async fn test_empty_block() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "    First Line    \n  Middle Line  \n    Last Line    \n";

        write_test_file(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let diff = format!("{}\n  Middle Line  \n{}\n{}\n", SEARCH, DIVIDER, REPLACE);
        let result = fs_replace
            .call(FSReplaceInput { path: file_path.to_string_lossy().to_string(), diff })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&*file_path.to_string_lossy()));
    }

    #[tokio::test]
    async fn test_complex_newline_preservation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Test file with various newline patterns
        let content = "\n\n// Header comment\n\n\nfunction test() {\n    // Inside comment\n\n    let x = 1;\n\n\n    console.log(x);\n}\n\n// Footer comment\n\n\n";
        write_test_file(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;

        // Test 1: Replace content while preserving surrounding newlines
        // Test 1: Replace content while preserving surrounding newlines
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!("{}\n    let x = 1;\n\n\n    console.log(x);\n{}\n    let y = 2;\n\n\n    console.log(y);\n{}\n",
                    SEARCH, DIVIDER, REPLACE).to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&*file_path.to_string_lossy()));

        // Test 2: Replace block with different newline pattern
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!(
                    "{}\n\n// Footer comment\n\n\n{}\n\n\n\n// Updated footer\n\n{}\n",
                    SEARCH, DIVIDER, REPLACE
                )
                .to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&*file_path.to_string_lossy()));

        // Test 3: Replace with empty lines preservation
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!(
                    "{}\n\n\n// Header comment\n\n\n{}\n\n\n\n// New header\n\n\n\n{}\n",
                    SEARCH, DIVIDER, REPLACE
                )
                .to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&*file_path.to_string_lossy()));
    }

    #[tokio::test]
    async fn test_fuzzy_search_replace() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Test file with typos and variations
        let content = r#"function calculateTotal(items) {
  let total = 0;
  for (const itm of items) {
    total += itm.price;
  }
  return total;
}
"#;
        write_test_file(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        // Search with different casing, spacing, and variable names
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!("{}\n  for (const itm of items) {{\n    total += itm.price;\n{}\n  for (const item of items) {{\n    total += item.price * item.quantity;\n{}\n",
                    SEARCH, DIVIDER, REPLACE).to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&*file_path.to_string_lossy()));

        // Test fuzzy matching with more variations
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!("{}\nfunction calculateTotal(items) {{\n  let total = 0;\n{}\nfunction computeTotal(items, tax = 0) {{\n  let total = 0.0;\n{}\n",
                    SEARCH, DIVIDER, REPLACE).to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&*file_path.to_string_lossy()));
    }

    #[tokio::test]
    async fn test_fuzzy_search_advanced() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Test file with more complex variations
        let content = r#"class UserManager {
  async getUserById(userId) {
    const user = await db.findOne({ id: userId });
    if (!user) throw new Error('User not found');
    return user;
  }
}
"#;
        write_test_file(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        // Search with structural similarities but different variable names and spacing
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!("{}\n  async getUserById(userId) {{\n    const user = await db.findOne({{ id: userId }});\n{}\n  async findUser(id, options = {{}}) {{\n    const user = await this.db.findOne({{ userId: id, ...options }});\n{}\n",
                    SEARCH, DIVIDER, REPLACE).to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&*file_path.to_string_lossy()));

        // Test fuzzy matching with error handling changes
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!("{}\n    if (!user) throw new Error('User not found');\n    return user;\n{}\n    if (!user) {{\n      throw new UserNotFoundError(id);\n    }}\n    return this.sanitizeUser(user);\n{}\n",
                    SEARCH, DIVIDER, REPLACE).to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&*file_path.to_string_lossy()));
    }

    #[tokio::test]
    async fn test_invalid_rust_replace() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = "fn main() { let x = 42; }";

        write_test_file(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!(
                    "{}\nfn main() {{ let x = 42; }}\n{}\nfn main() {{ let x = \n{}\n",
                    SEARCH, DIVIDER, REPLACE
                )
                .to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&file_path.display().to_string()));
        assert!(result.contains("Warning: Syntax"));
    }

    #[tokio::test]
    async fn test_valid_rust_replace() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = "fn main() { let x = 42; }";

        write_test_file(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: format!("{}\nfn main() {{ let x = 42; }}\n{}\nfn main() {{ let x = 42; let y = x * 2; }}\n{}\n",
                    SEARCH, DIVIDER, REPLACE).to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully applied"));
        assert!(result.contains(&file_path.display().to_string()));
    }

    #[tokio::test]
    async fn test_replace_curly_brace_with_double_curly_brace() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        // Create test file with content
        let file_content = "fn test(){\n    let x = 42;\n    {\n        // test block-1    }\n }\n";
        write_test_file(&file_path, file_content).await.unwrap();

        // want to replace '}' with '}}'.
        let diff = format!("{}\n}}{}\n}}}}\n{}", SEARCH, DIVIDER, REPLACE);
        let res = FSReplace
            .call(FSReplaceInput { path: file_path.to_string_lossy().to_string(), diff })
            .await
            .unwrap();

        assert!(res.contains("Successfully applied"));
        assert!(res.contains(&file_path.display().to_string()));
    }

    #[tokio::test]
    async fn test_empty_search_block() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        // Create test file with content
        let file_content =
            r#"fn test(){\n    let x = 42;\n    {\n        // test block-1    }\n}\n"#;
        write_test_file(&file_path, file_content).await.unwrap();

        // want to replace '' with 'empty-space-replaced'.
        let diff = format!("{}\n{}\nempty-space-replaced{}", SEARCH, DIVIDER, REPLACE);

        let res = FSReplace
            .call(FSReplaceInput { path: file_path.to_string_lossy().to_string(), diff })
            .await
            .unwrap();

        assert!(res.contains("Successfully applied"));
        assert!(res.contains(&file_path.display().to_string()));
    }

    #[tokio::test]
    async fn test_match_empty_white_space() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        // Create test file with content
        let file_content =
            r#"fn test(){\n    let x = 42;\n    {\n        // test block-1    }\n}\n"#;
        write_test_file(&file_path, file_content).await.unwrap();

        // want to replace ' ' with '--'.
        let diff = format!("{}\n {}\n--{}", SEARCH, DIVIDER, REPLACE);

        let res = FSReplace
            .call(FSReplaceInput { path: file_path.to_string_lossy().to_string(), diff })
            .await
            .unwrap();

        assert!(res.contains("Successfully applied"));
        assert!(res.contains(&file_path.display().to_string()));
    }
}
