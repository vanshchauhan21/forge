use schemars::JsonSchema;
use serde::Deserialize;

use crate::{Description, ToolTrait};
use forge_tool_macros::Description as DescriptionDerive;

#[derive(Deserialize, JsonSchema)]
pub struct FSReplaceInput {
    pub path: String,
    pub diff: String,
}

/// Replace content in a file using SEARCH/REPLACE blocks. Each block defines
/// exact changes to make to specific parts of the file. Supports multiple
/// blocks for complex changes while preserving file formatting and structure.
#[derive(DescriptionDerive)]
pub struct FSReplace;

#[async_trait::async_trait]
impl ToolTrait for FSReplace {
    type Input = FSReplaceInput;
    type Output = String;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let content = tokio::fs::read_to_string(&input.path)
            .await
            .map_err(|e| e.to_string())?;

        let mut result = content;

        // Process each block
        for block in input.diff.split(">>>>>>> REPLACE").filter(|b| !b.trim().is_empty()) {
            let parts: Vec<&str> = block.split("=======").collect();
            if parts.len() != 2 {
                continue;
            }

            let search = parts[0].trim_start_matches("<<<<<<< SEARCH").trim();
            let replace = parts[1].trim();

            // Convert search and replace content to lines
            let search_lines: Vec<&str> = search.lines().collect();
            let replace_lines: Vec<&str> = replace.lines().collect();

            // Convert current content to lines
            let mut lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();
            let mut new_lines = Vec::new();
            let mut i = 0;

            while i < lines.len() {
                if i + search_lines.len() <= lines.len() {
                    let mut matches = true;
                    for (j, search_line) in search_lines.iter().enumerate() {
                        if lines[i + j] != *search_line {
                            matches = false;
                            break;
                        }
                    }
                    if matches {
                        new_lines.extend(replace_lines.iter().map(|s| s.to_string()));
                        i += search_lines.len();
                        continue;
                    }
                }
                new_lines.push(lines[i].clone());
                i += 1;
            }

            // Update result with new content
            result = if new_lines.is_empty() {
                String::new()
            } else {
                new_lines.join("\n")
            };
        }
        tokio::fs::write(&input.path, &result)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("Successfully replaced content in {}", input.path))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_fs_replace_single_block() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello World\nTest Line\nGoodbye World";

        fs::write(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: "<<<<<<< SEARCH\nHello World\n=======\nHi World\n>>>>>>> REPLACE".to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully replaced"));

        let new_content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(new_content, "Hi World\nTest Line\nGoodbye World");
    }

    #[tokio::test]
    async fn test_fs_replace_multiple_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello World\nTest Line\nGoodbye World";

        fs::write(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let diff = "<<<<<<< SEARCH\nHello World\n=======\nHi World\n>>>>>>> REPLACE\n\n<<<<<<< SEARCH\nGoodbye World\n=======\nBye World\n>>>>>>> REPLACE".to_string();

        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff,
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully replaced"));

        let new_content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(new_content, "Hi World\nTest Line\nBye World");
    }

    #[tokio::test]
    async fn test_fs_replace_no_match() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello World\nTest Line\nGoodbye World";

        fs::write(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: "<<<<<<< SEARCH\nNo Match\n=======\nReplacement\n>>>>>>> REPLACE".to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully replaced"));

        let new_content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(new_content, content);
    }

    #[tokio::test]
    async fn test_fs_replace_empty_block() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Hello World\nTest Line\nGoodbye World";

        fs::write(&file_path, content).await.unwrap();

        let fs_replace = FSReplace;
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: "<<<<<<< SEARCH\nTest Line\n=======\n>>>>>>> REPLACE".to_string(),
            })
            .await
            .unwrap();

        assert!(result.contains("Successfully replaced"));

        let new_content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(new_content, "Hello World\n\nGoodbye World");
    }
}
