use forge_tool_macros::Description;
use schemars::JsonSchema;
use serde::Deserialize;
use tracing::debug;

use crate::{Description, ToolTrait};

#[derive(Deserialize, JsonSchema)]
pub struct FSReadInput {
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct FSSearchInput {
    pub dir: String,
    pub pattern: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct FSListInput {
    pub path: String,
    pub recursive: Option<bool>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FSFileInfoInput {
    pub path: String,
}

/// Read the complete contents of a file from the file system. Handles various
/// text encodings and provides detailed error messages if the file cannot be
/// read. Use this tool when you need to examine the contents of a single file.
/// Only works within allowed directories.
#[derive(Description)]
pub(crate) struct FSRead;
/// Recursively search for files and directories matching a pattern. Searches
/// through all subdirectories from the starting path. The search is
/// case-insensitive and matches partial names. Returns full paths to all
/// matching items. Great for finding files when you don't know their exact
/// location. Only searches within allowed directories.
#[derive(Description)]
pub(crate) struct FSSearch;
/// Get a detailed listing of all files and directories in a specified path.
/// Results clearly distinguish between files and directories with [FILE] and
/// [DIR] prefixes. When recursive is true, lists contents of all
/// subdirectories. When recursive is false or not provided, only lists
/// top-level contents. This tool is essential for understanding directory
/// structure and finding specific files within a directory. Only works within
/// allowed directories.
#[derive(Description)]
pub(crate) struct FSList;
/// Retrieve detailed metadata about a file or directory. Returns comprehensive
/// information including size, creation time, last modified time, permissions,
/// and type. This tool is perfect for understanding file characteristics
/// without reading the actual content. Only works within allowed directories.
#[derive(Description)]
pub(crate) struct FSFileInfo;
/// Write the provided content to a file. This tool is useful for creating new
/// files or overwriting existing files with new content. Only works within
/// allowed directories.
#[derive(Description)]
pub(crate) struct FSWrite;

#[derive(Deserialize, JsonSchema)]
pub struct FSWriteInput {
    pub path: String,
    pub content: String,
}
#[async_trait::async_trait]
impl ToolTrait for FSWrite {
    type Input = FSWriteInput;
    type Output = String;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        tokio::fs::write(&input.path, &input.content)
            .await
            .map_err(|e| e.to_string())?;
        Ok("Write successful".to_string())
    }
}

#[async_trait::async_trait]
impl ToolTrait for FSRead {
    type Input = FSReadInput;
    type Output = String;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let content = tokio::fs::read_to_string(&input.path)
            .await
            .map_err(|e| e.to_string())?;
        Ok(content)
    }
}

#[async_trait::async_trait]
impl ToolTrait for FSSearch {
    type Input = FSSearchInput;
    type Output = Vec<String>;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let pattern = input.pattern.to_lowercase();

        async fn search(dir: &std::path::Path, pattern: &str) -> Result<Vec<String>, String> {
            let mut matches = Vec::new();
            let mut walker = tokio::fs::read_dir(dir).await.map_err(|e| e.to_string())?;

            while let Some(entry) = walker.next_entry().await.map_err(|e| e.to_string())? {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name = name.to_string_lossy().to_lowercase();
                    if name.contains(pattern) {
                        matches.push(path.to_string_lossy().to_string());
                    }
                }

                if path.is_dir() {
                    matches.extend(Box::pin(search(&path, pattern)).await?);
                }
            }
            Ok(matches)
        }

        Ok(Box::pin(search(std::path::Path::new(&input.dir), &pattern)).await?)
    }
}

#[async_trait::async_trait]
impl ToolTrait for FSList {
    type Input = FSListInput;
    type Output = Vec<String>;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        async fn list_dir(dir: &std::path::Path, recursive: bool) -> Result<Vec<String>, String> {
            let mut paths = Vec::new();
            let mut walker = tokio::fs::read_dir(dir).await.map_err(|e| e.to_string())?;

            while let Some(entry) = walker.next_entry().await.map_err(|e| e.to_string())? {
                let file_type = entry.file_type().await.map_err(|e| e.to_string())?;
                let prefix = if file_type.is_dir() {
                    "[DIR]"
                } else {
                    "[FILE]"
                };
                paths.push(format!("{} {}", prefix, entry.path().display()));

                if recursive && file_type.is_dir() {
                    paths.extend(Box::pin(list_dir(&entry.path(), true)).await?);
                }
            }
            Ok(paths)
        }

        let dir = std::path::Path::new(&input.path);
        let paths = list_dir(dir, input.recursive.unwrap_or(false)).await?;
        debug!("Found items {}", paths.join("\n"));
        Ok(paths)
    }
}

#[async_trait::async_trait]
impl ToolTrait for FSFileInfo {
    type Input = FSFileInfoInput;
    type Output = String;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let meta = tokio::fs::metadata(input.path)
            .await
            .map_err(|e| e.to_string())?;
        Ok(format!("{:?}", meta))
    }
}

#[cfg(test)]
mod test {
    use tempfile::TempDir;
    use tokio::fs;

    use super::*;

    #[tokio::test]
    async fn test_fs_read_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let test_content = "Hello, World!";
        fs::write(&file_path, test_content).await.unwrap();

        let fs_read = FSRead;
        let result = fs_read
            .call(FSReadInput { path: file_path.to_string_lossy().to_string() })
            .await
            .unwrap();

        assert_eq!(result, test_content);
    }

    #[tokio::test]
    async fn test_fs_read_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");

        let fs_read = FSRead;
        let result = fs_read
            .call(FSReadInput { path: nonexistent_file.to_string_lossy().to_string() })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fs_read_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.txt");
        fs::write(&file_path, "").await.unwrap();

        let fs_read = FSRead;
        let result = fs_read
            .call(FSReadInput { path: file_path.to_string_lossy().to_string() })
            .await
            .unwrap();

        assert_eq!(result, "");
    }

    #[tokio::test]
    async fn test_fs_file_info_on_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test content").await.unwrap();

        let fs_info = FSFileInfo;
        let result = fs_info
            .call(FSFileInfoInput { path: file_path.to_string_lossy().to_string() })
            .await
            .unwrap();

        assert!(result.contains("FileType"));
        assert!(result.contains("permissions"));
        assert!(result.contains("modified"));
    }

    #[tokio::test]
    async fn test_fs_file_info_on_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");
        fs::create_dir(&dir_path).await.unwrap();

        let fs_info = FSFileInfo;
        let result = fs_info
            .call(FSFileInfoInput { path: dir_path.to_string_lossy().to_string() })
            .await
            .unwrap();

        assert!(result.contains("FileType"));
        assert!(result.contains("permissions"));
        assert!(result.contains("modified"));
    }

    #[tokio::test]
    async fn test_fs_file_info_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");

        let fs_info = FSFileInfo;
        let result = fs_info
            .call(FSFileInfoInput { path: nonexistent_path.to_string_lossy().to_string() })
            .await;

        assert!(result.is_err());
    }

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

        assert!(result.is_empty());
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

        assert_eq!(result.len(), 4);

        let files: Vec<_> = result.iter().filter(|p| p.starts_with("[FILE]")).collect();
        let dirs: Vec<_> = result.iter().filter(|p| p.starts_with("[DIR]")).collect();

        assert_eq!(files.len(), 2);
        assert_eq!(dirs.len(), 2);

        assert!(result.iter().any(|p| p.contains("file1.txt")));
        assert!(result.iter().any(|p| p.contains("file2.txt")));
        assert!(result.iter().any(|p| p.contains("dir1")));
        assert!(result.iter().any(|p| p.contains("dir2")));
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

        assert_eq!(result.len(), 3);
        assert!(result.iter().any(|p| p.contains("regular.txt")));
        assert!(result.iter().any(|p| p.contains(".hidden")));
        assert!(result.iter().any(|p| p.contains(".hidden_dir")));
    }

    #[tokio::test]
    async fn test_fs_search_basic() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test1.txt"), "")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("test2.txt"), "")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("other.txt"), "")
            .await
            .unwrap();

        let fs_search = FSSearch;
        let result = fs_search
            .call(FSSearchInput {
                dir: temp_dir.path().to_string_lossy().to_string(),
                pattern: "test".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|p| p.ends_with("test1.txt")));
        assert!(result.iter().any(|p| p.ends_with("test2.txt")));
    }

    #[tokio::test]
    async fn test_fs_search_recursive() {
        let temp_dir = TempDir::new().unwrap();

        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).await.unwrap();

        fs::write(temp_dir.path().join("test1.txt"), "")
            .await
            .unwrap();
        fs::write(sub_dir.join("test2.txt"), "").await.unwrap();
        fs::write(sub_dir.join("other.txt"), "").await.unwrap();

        let fs_search = FSSearch;
        let result = fs_search
            .call(FSSearchInput {
                dir: temp_dir.path().to_string_lossy().to_string(),
                pattern: "test".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|p| p.ends_with("test1.txt")));
        assert!(result.iter().any(|p| p.ends_with("test2.txt")));
    }

    #[tokio::test]
    async fn test_fs_search_case_insensitive() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("TEST.txt"), "")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("TeSt2.txt"), "")
            .await
            .unwrap();

        let fs_search = FSSearch;
        let result = fs_search
            .call(FSSearchInput {
                dir: temp_dir.path().to_string_lossy().to_string(),
                pattern: "test".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|p| p.ends_with("TEST.txt")));
        assert!(result.iter().any(|p| p.ends_with("TeSt2.txt")));
    }

    #[tokio::test]
    async fn test_fs_search_empty_pattern() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test.txt"), "")
            .await
            .unwrap();

        let fs_search = FSSearch;
        let result = fs_search
            .call(FSSearchInput {
                dir: temp_dir.path().to_string_lossy().to_string(),
                pattern: "".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert!(result.iter().any(|p| p.ends_with("test.txt")));
    }

    #[tokio::test]
    async fn test_fs_search_nonexistent_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_dir = temp_dir.path().join("nonexistent");

        let fs_search = FSSearch;
        let result = fs_search
            .call(FSSearchInput {
                dir: nonexistent_dir.to_string_lossy().to_string(),
                pattern: "test".to_string(),
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fs_search_directory_names() {
        let temp_dir = TempDir::new().unwrap();

        fs::create_dir(temp_dir.path().join("test_dir"))
            .await
            .unwrap();
        fs::create_dir(temp_dir.path().join("test_dir").join("nested"))
            .await
            .unwrap();
        fs::create_dir(temp_dir.path().join("other_dir"))
            .await
            .unwrap();

        let fs_search = FSSearch;
        let result = fs_search
            .call(FSSearchInput {
                dir: temp_dir.path().to_string_lossy().to_string(),
                pattern: "test".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert!(result.iter().any(|p| p.ends_with("test_dir")));
    }

    #[test]
    fn test_description() {
        assert!(FSRead::description().len() > 100)
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

        assert_eq!(result.len(), 5); // root.txt, dir1, file1.txt, subdir, file2.txt
        assert!(result.iter().any(|p| p.contains("root.txt")));
        assert!(result.iter().any(|p| p.contains("dir1")));
        assert!(result.iter().any(|p| p.contains("file1.txt")));
        assert!(result.iter().any(|p| p.contains("subdir")));
        assert!(result.iter().any(|p| p.contains("file2.txt")));

        // Test non-recursive listing of same structure
        let result = fs_list
            .call(FSListInput {
                path: temp_dir.path().to_string_lossy().to_string(),
                recursive: Some(false),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 2); // Only root.txt and dir1
        assert!(result.iter().any(|p| p.contains("root.txt")));
        assert!(result.iter().any(|p| p.contains("dir1")));
        assert!(!result.iter().any(|p| p.contains("file1.txt")));
        assert!(!result.iter().any(|p| p.contains("subdir")));
        assert!(!result.iter().any(|p| p.contains("file2.txt")));
    }

    #[tokio::test]
    async fn test_fs_write_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let fs_write = FSWrite;
        let _ = fs_write
            .call(FSWriteInput {
                path: file_path.to_string_lossy().to_string(),
                content: "Hello, World!".to_string(),
            })
            .await
            .unwrap();
        let s = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(s, "Hello, World!")
    }
}
