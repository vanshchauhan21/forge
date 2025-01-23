use std::path::Path;

use anyhow::Context;
use forge_domain::{NamedTool, ToolCallService, ToolDescription, ToolName};
use forge_tool_macros::ToolDescription;
use forge_walker::Walker;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
pub struct FSListInput {
    /// The path of the directory to list contents for (relative to the current
    /// working directory)
    pub path: String,
    /// Whether to list files recursively. Use true for recursive listing, false
    /// or omit for top-level only.
    pub recursive: Option<bool>,
}

/// Request to list files and directories within the specified directory. If
/// recursive is true, it will list all files and directories recursively. If
/// recursive is false or not provided, it will only list the top-level
/// contents. Do not use this tool to confirm the existence of files you may
/// have created, as the user will let you know if the files were created
/// successfully or not.
#[derive(ToolDescription)]
pub struct FSList;

impl NamedTool for FSList {
    fn tool_name(&self) -> ToolName {
        ToolName::new("tool_forge_fs_list")
    }
}

#[async_trait::async_trait]
impl ToolCallService for FSList {
    type Input = FSListInput;

    async fn call(&self, input: Self::Input) -> Result<String, String> {
        let dir = Path::new(&input.path);
        if !dir.exists() {
            return Err(format!("Directory '{}' does not exist", input.path));
        }

        let mut paths = Vec::new();
        let recursive = input.recursive.unwrap_or(false);
        let max_depth = if recursive { usize::MAX } else { 1 };
        let walker = Walker::new(dir.to_path_buf()).with_max_depth(max_depth);

        let files = walker
            .get()
            .await
            .with_context(|| format!("Failed to read directory contents from '{}'", input.path))
            .map_err(|e| e.to_string())?;

        for entry in files {
            // Skip the root directory itself
            if entry.path == dir.to_string_lossy() {
                continue;
            }

            if !entry.path.is_empty() {
                let prefix = if entry.is_dir { "[DIR]" } else { "[FILE]" };
                paths.push(format!("{} {}", prefix, entry.path));
            }
        }

        if paths.is_empty() {
            Ok("No files found".to_string())
        } else {
            Ok(paths.join("\n"))
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;
    use tokio::fs;

    use super::*;

    #[tokio::test]
    async fn test_fs_list_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let fs_list = FSList;
        let result = fs_list
            .call(FSListInput {
                path: temp_dir.path().to_string_lossy().to_string(),
                recursive: None,
            })
            .await
            .unwrap();

        assert_eq!(result, "No files found");
    }

    #[tokio::test]
    async fn test_fs_list_with_files_and_dirs() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("file1.txt"), "content1")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2")
            .await
            .unwrap();
        fs::create_dir(temp_dir.path().join("dir1")).await.unwrap();
        fs::create_dir(temp_dir.path().join("dir2")).await.unwrap();

        let fs_list = FSList;
        let result = fs_list
            .call(FSListInput {
                path: temp_dir.path().to_string_lossy().to_string(),
                recursive: None,
            })
            .await
            .unwrap();

        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 4);

        let files: Vec<_> = lines.iter().filter(|p| p.starts_with("[FILE]")).collect();
        let dirs: Vec<_> = lines.iter().filter(|p| p.starts_with("[DIR]")).collect();

        assert_eq!(files.len(), 2);
        assert_eq!(dirs.len(), 2);

        assert!(result.contains("file1.txt"));
        assert!(result.contains("file2.txt"));
        assert!(result.contains("dir1"));
        assert!(result.contains("dir2"));
    }

    #[tokio::test]
    async fn test_fs_list_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_dir = temp_dir.path().join("nonexistent");

        let fs_list = FSList;
        let result = fs_list
            .call(FSListInput {
                path: nonexistent_dir.to_string_lossy().to_string(),
                recursive: None,
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fs_list_with_hidden_files() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("regular.txt"), "content")
            .await
            .unwrap();
        fs::write(temp_dir.path().join(".hidden"), "content")
            .await
            .unwrap();
        fs::create_dir(temp_dir.path().join(".hidden_dir"))
            .await
            .unwrap();

        let fs_list = FSList;
        let result = fs_list
            .call(FSListInput {
                path: temp_dir.path().to_string_lossy().to_string(),
                recursive: None,
            })
            .await
            .unwrap();

        assert!(result.contains("regular.txt"));
        assert!(!result.contains(".hidden"));
        assert!(!result.contains(".hidden_dir"));
    }

    #[tokio::test]
    async fn test_fs_list_recursive() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested directory structure
        fs::create_dir(temp_dir.path().join("dir1")).await.unwrap();
        fs::write(temp_dir.path().join("dir1/file1.txt"), "content1")
            .await
            .unwrap();
        fs::create_dir(temp_dir.path().join("dir1/subdir"))
            .await
            .unwrap();
        fs::write(temp_dir.path().join("dir1/subdir/file2.txt"), "content2")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("root.txt"), "content3")
            .await
            .unwrap();

        let fs_list = FSList;

        // Test recursive listing
        let result = fs_list
            .call(FSListInput {
                path: temp_dir.path().to_string_lossy().to_string(),
                recursive: Some(true),
            })
            .await
            .unwrap();

        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 5); // root.txt, dir1, file1.txt, subdir, file2.txt
        assert!(result.contains("root.txt"));
        assert!(result.contains("dir1"));
        assert!(result.contains("file1.txt"));
        assert!(result.contains("subdir"));
        assert!(result.contains("file2.txt"));

        // Test non-recursive listing of same structure
        let result = fs_list
            .call(FSListInput {
                path: temp_dir.path().to_string_lossy().to_string(),
                recursive: Some(false),
            })
            .await
            .unwrap();

        let lines: Vec<_> = result.lines().collect();
        assert_eq!(lines.len(), 2); // Only root.txt and dir1
        assert!(result.contains("root.txt"));
        assert!(result.contains("dir1"));
        assert!(!result.contains("file1.txt"));
        assert!(!result.contains("subdir"));
        assert!(!result.contains("file2.txt"));
    }
}
