use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::Path;

use dissimilar::Chunk;
use forge_domain::{Description, ToolCallService};
use forge_tool_macros::Description;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use tracing::{debug, error};

use crate::fs::syn;

fn persist_changes<P: AsRef<Path>>(
    temp_file: NamedTempFile,
    path: P,
    backup_path: impl AsRef<Path>,
) -> Result<(), String> {
    // Persist changes atomically
    match temp_file.persist(&path) {
        Ok(_) => {
            debug!("Successfully persisted changes to {:?}", path.as_ref());
            // Remove backup file on success
            if backup_path.as_ref().exists() {
                if let Err(e) = fs::remove_file(&backup_path) {
                    error!("Failed to remove backup file: {}", e);
                }
            }
            Ok(())
        }
        Err(e) => {
            error!("Failed to persist changes: {}", e);
            // Restore from backup if persist failed
            if backup_path.as_ref().exists() {
                if let Err(e) = fs::rename(&backup_path, &path) {
                    error!("Failed to restore from backup: {}", e);
                }
            }
            Err(e.to_string())
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct FSReplaceInput {
    /// File path relative to the current working directory
    pub path: String,
    /// SEARCH/REPLACE blocks defining changes
    pub diff: String,
}

/// Replace sections in a file using SEARCH/REPLACE blocks for precise
/// modifications.
///
/// <<<<<<< SEARCH
/// [exact content to find]
/// =======
/// [new content to replace with]
/// >>>>>>> REPLACE
///
/// Rules:
/// 1. SEARCH must match exactly (whitespace, indentation, line endings)
/// 2. Each block replaces first match only
/// 3. Keep blocks minimal - include only changing lines plus needed context
/// 4. Complete lines only - no truncation
/// 5. For moves: use 2 blocks (delete + insert)
/// 6. For deletes: use empty REPLACE section
///
/// Example:
/// <<<<<<< SEARCH
/// def old_function(x):
///     return x + 1
/// =======
/// def new_function(x, y=0):
///     return x + y
/// >>>>>>> REPLACE
#[derive(Description)]
pub struct FSReplace;

struct Block {
    search: String,
    replace: String,
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

    while let Some(search_start) = diff[pos..].find("<<<<<<< SEARCH") {
        let search_start = pos + search_start + "<<<<<<< SEARCH".len();

        // Include the newline after SEARCH marker in the position
        let search_start = match diff[search_start..].find('\n') {
            Some(nl) => search_start + nl + 1,
            None => {
                return Err("Invalid diff format: Missing newline after SEARCH marker".to_string())
            }
        };

        let Some(separator) = diff[search_start..].find("=======") else {
            return Err("Invalid diff format: Missing separator".to_string());
        };
        let separator = search_start + separator;

        // Include the newline after separator in the position
        let separator_end = separator + "=======".len();
        let separator_end = match diff[separator_end..].find('\n') {
            Some(nl) => separator_end + nl + 1,
            None => return Err("Invalid diff format: Missing newline after separator".to_string()),
        };

        let Some(replace_end) = diff[separator_end..].find(">>>>>>> REPLACE") else {
            return Err("Invalid diff format: Missing end marker".to_string());
        };
        let replace_end = separator_end + replace_end;

        let search = &diff[search_start..separator];
        let replace = &diff[separator_end..replace_end];

        blocks.push(Block { search: search.to_string(), replace: replace.to_string() });

        pos = replace_end + ">>>>>>> REPLACE".len();
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

fn apply_changes<P: AsRef<Path>>(path: P, blocks: Vec<Block>) -> Result<String, String> {
    let mut content = String::new();
    let mut result = String::new();
    let backup_path = path.as_ref().with_extension("bak");

    // Handle new file or empty file case
    if !path.as_ref().exists() || blocks[0].search.is_empty() {
        let mut temp_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        if !blocks[0].replace.is_empty() {
            // Validate content before writing for new file
            syn::validate(path.as_ref(), &blocks[0].replace)?;
            write!(temp_file, "{}", blocks[0].replace).map_err(|e| e.to_string())?;
            result = blocks[0].replace.clone();
        }
        persist_changes(temp_file, path, backup_path)?;
        return Ok(result);
    }

    // Create backup and read existing file
    fs::copy(&path, &backup_path).map_err(|e| {
        error!("Failed to create backup: {}", e);
        e.to_string()
    })?;
    debug!("Created backup at {:?}", backup_path);

    let file = File::open(&path).map_err(|e| {
        error!("Failed to open source file: {}", e);
        e.to_string()
    })?;

    BufReader::new(file)
        .read_to_string(&mut content)
        .map_err(|e| {
            error!("Failed to read file content: {}", e);
            e.to_string()
        })?;

    result = content.clone();
    let mut temp_file = NamedTempFile::new().map_err(|e| e.to_string())?;

    // Apply each block sequentially
    for block in blocks {
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

    // Validate the final content before writing
    syn::validate(path.as_ref(), &result)?;

    // Write the modified content
    write!(temp_file, "{}", result).map_err(|e| e.to_string())?;
    persist_changes(temp_file, path, backup_path)?;

    Ok(result)
}

#[derive(Serialize, JsonSchema)]
pub struct FSReplaceOutput {
    pub path: String,
    pub content: String,
}

#[async_trait::async_trait]
impl ToolCallService for FSReplace {
    type Input = FSReplaceInput;
    type Output = FSReplaceOutput;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let blocks = parse_blocks(&input.diff)?;
        let content = apply_changes(&input.path, blocks)?;
        Ok(FSReplaceOutput { path: input.path, content })
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use tempfile::TempDir;

    use super::*;

    async fn write_test_file(path: impl AsRef<Path>, content: &str) -> Result<(), String> {
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(content.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(())
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
                diff: "<<<<<<< SEARCH\n    Hello World    \n=======\n    Hi World    \n>>>>>>> REPLACE\n"
                    .to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            result.content,
            "    Hi World    \n  Test Line  \n   Goodbye World   \n"
        );
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
                diff: "<<<<<<< SEARCH\n=======\nNew content\n>>>>>>> REPLACE\n".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.content, "New content\n");
    }

    #[tokio::test]
    async fn test_multiple_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "    First Line    \n  Middle Line  \n    Last Line    \n";

        write_test_file(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let diff = "<<<<<<< SEARCH\n    First Line    \n=======\n    New First    \n>>>>>>> REPLACE\n<<<<<<< SEARCH\n    Last Line    \n=======\n    New Last    \n>>>>>>> REPLACE\n".to_string();

        let result = fs_replace
            .call(FSReplaceInput { path: file_path.to_string_lossy().to_string(), diff })
            .await
            .unwrap();

        assert_eq!(
            result.content,
            "    New First    \n  Middle Line  \n    New Last    \n"
        );
    }

    #[tokio::test]
    async fn test_empty_block() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "    First Line    \n  Middle Line  \n    Last Line    \n";

        write_test_file(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: "<<<<<<< SEARCH\n  Middle Line  \n=======\n>>>>>>> REPLACE\n".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.content, "    First Line    \n    Last Line    \n");
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
        let result1 = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: "<<<<<<< SEARCH\n    let x = 1;\n\n\n    console.log(x);\n=======\n    let y = 2;\n\n\n    console.log(y);\n>>>>>>> REPLACE\n".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            result1.content,
            "\n\n// Header comment\n\n\nfunction test() {\n    // Inside comment\n\n    let y = 2;\n\n\n    console.log(y);\n}\n\n// Footer comment\n\n\n"
        );

        // Test 2: Replace block with different newline pattern
        let result2 = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: "<<<<<<< SEARCH\n\n// Footer comment\n\n\n=======\n\n\n\n// Updated footer\n\n>>>>>>> REPLACE\n".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            result2.content,
            "\n\n// Header comment\n\n\nfunction test() {\n    // Inside comment\n\n    let y = 2;\n\n\n    console.log(y);\n}\n\n\n\n// Updated footer\n\n"
        );

        // Test 3: Replace with empty lines preservation
        let result3 = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: "<<<<<<< SEARCH\n\n\n// Header comment\n\n\n=======\n\n\n\n// New header\n\n\n\n>>>>>>> REPLACE\n".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            result3.content,
            "\n\n\n// New header\n\n\n\nfunction test() {\n    // Inside comment\n\n    let y = 2;\n\n\n    console.log(y);\n}\n\n\n\n// Updated footer\n\n"
        );
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
                diff: r#"<<<<<<< SEARCH
  for (const itm of items) {
    total += itm.price;
=======
  for (const item of items) {
    total += item.price * item.quantity;
>>>>>>> REPLACE
"#
                .to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            result.content,
            r#"function calculateTotal(items) {
  let total = 0;
  for (const item of items) {
    total += item.price * item.quantity;
  }
  return total;
}
"#
        );

        // Test fuzzy matching with more variations
        let result2 = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: r#"<<<<<<< SEARCH
function calculateTotal(items) {
  let total = 0;
=======
function computeTotal(items, tax = 0) {
  let total = 0.0;
>>>>>>> REPLACE
"#
                .to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            result2.content,
            r#"function computeTotal(items, tax = 0) {
  let total = 0.0;
  for (const item of items) {
    total += item.price * item.quantity;
  }
  return total;
}
"#
        );
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
                diff: r#"<<<<<<< SEARCH
  async getUserById(userId) {
    const user = await db.findOne({ id: userId });
=======
  async findUser(id, options = {}) {
    const user = await this.db.findOne({ userId: id, ...options });
>>>>>>> REPLACE
"#
                .to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            result.content,
            r#"class UserManager {
  async findUser(id, options = {}) {
    const user = await this.db.findOne({ userId: id, ...options });
    if (!user) throw new Error('User not found');
    return user;
  }
}
"#
        );

        // Test fuzzy matching with error handling changes
        let result2 = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: r#"<<<<<<< SEARCH
    if (!user) throw new Error('User not found');
    return user;
=======
    if (!user) {
      throw new UserNotFoundError(id);
    }
    return this.sanitizeUser(user);
>>>>>>> REPLACE
"#
                .to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            result2.content,
            r#"class UserManager {
  async findUser(id, options = {}) {
    const user = await this.db.findOne({ userId: id, ...options });
    if (!user) {
      throw new UserNotFoundError(id);
    }
    return this.sanitizeUser(user);
  }
}
"#
        );
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
                diff: "<<<<<<< SEARCH\nfn main() { let x = 42; }\n=======\nfn main() { let x = \n>>>>>>> REPLACE\n".to_string(),
            })
            .await;

        assert!(result.is_err());
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
                diff: "<<<<<<< SEARCH\nfn main() { let x = 42; }\n=======\nfn main() { let x = 42; let y = x * 2; }\n>>>>>>> REPLACE\n".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.content, "fn main() { let x = 42; let y = x * 2; }\n");
    }
}
