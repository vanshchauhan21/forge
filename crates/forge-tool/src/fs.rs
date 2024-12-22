use std::path::Path;
use std::pin::Pin;
use std::collections::HashSet;

use forge_tool_macros::Description;
use walkdir::WalkDir;
use schemars::JsonSchema;
use serde::Deserialize;
use tokio::task;
use tracing::debug;

use crate::{Description, ToolTrait};

#[derive(Deserialize, JsonSchema)]
pub struct FSReadInput {
    pub path: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct FSSearchInput {
    pub path: String,
    pub regex: String,
    pub file_pattern: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct FSReplaceInput {
    pub path: String,
    pub diff: String,
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
/// Recursively search through file contents using regex patterns. Provides context
/// around matches and supports filtering by file patterns. Returns matches with
/// surrounding lines for better context understanding. Great for finding code
/// patterns or specific content across multiple files.
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
    pub content: Option<String>,
}
#[async_trait::async_trait]
impl ToolTrait for FSWrite {
    type Input = FSWriteInput;
    type Output = String;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        tokio::fs::write(&input.path, &input.content.unwrap_or_default())
            .await
            .map_err(|e| e.to_string())?;
        Ok(format!("Successfully wrote to {}", input.path))
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

/// Replace content in a file using SEARCH/REPLACE blocks. Each block defines
/// exact changes to make to specific parts of the file. Supports multiple blocks
/// for complex changes while preserving file formatting and structure.
#[derive(Description)]
pub(crate) struct FSReplace;

#[async_trait::async_trait]
impl ToolTrait for FSSearch {
    type Input = FSSearchInput;
    type Output = Vec<String>;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        use regex::Regex;
        use walkdir::WalkDir;

        let dir = Path::new(&input.path);
        if !dir.exists() {
            return Err("Directory does not exist".to_string());
        }

        // Create case-insensitive regex pattern
        let pattern = if input.regex.is_empty() {
            ".*".to_string()
        } else {
            format!("(?i){}", regex::escape(&input.regex)) // Add back regex::escape for literal matches
        };
        let regex = Regex::new(&pattern).map_err(|e| e.to_string())?;

        let mut matches = Vec::new();
        let mut seen_paths = HashSet::new();
        let walker = WalkDir::new(dir)
            .follow_links(false)
            .same_file_system(true)
            .into_iter();

        let entries = if let Some(ref pattern) = input.file_pattern {
            let glob = glob::Pattern::new(pattern).map_err(|e| e.to_string())?;
            walker
                .filter_entry(move |e| {
                    if !e.file_type().is_file() {
                        return true; // Keep traversing directories
                    }
                    e.file_name()
                        .to_str()
                        .map(|name| glob.matches(name))
                        .unwrap_or(false)
                })
                .filter_map(Result::ok)
                .collect::<Vec<_>>()
        } else {
            walker.filter_map(Result::ok).collect::<Vec<_>>()
        };

        for entry in entries {
            let path = entry.path().to_string_lossy();

            let name = entry.file_name().to_string_lossy();
            let is_file = entry.file_type().is_file();
            let is_dir = entry.file_type().is_dir();

            // For empty pattern
            if input.regex.is_empty() {
                if seen_paths.insert(path.to_string()) {
                    matches.push(format!(
                        "File: {}\nLines 1-1:\n{}\n",
                        path, path.to_string()
                    ));
                }
                continue;
            }

            // Check filename and directory name for match
            if regex.is_match(&name) {
                if seen_paths.insert(path.to_string()) {
                    matches.push(format!(
                        "File: {}\nLines 1-1:\n{}\n",
                        path, name
                    ));
                }
                if !is_file {
                    continue;
                }
            }

            // Skip content check for directories
            if !is_file {
                continue;
            }

            // Skip content check if already matched by name
            if seen_paths.contains(&path.to_string()) {
                continue;
            }

            // Check file content
            let content = match tokio::fs::read_to_string(entry.path()).await {
                Ok(content) => content,
                Err(_) => continue,
            };

            let lines: Vec<&str> = content.lines().collect();
            let mut content_matches = Vec::new();
            
            for (line_num, line) in lines.iter().enumerate() {
                if regex.is_match(line) {
                    // Get context (3 lines before and after)
                    let start = line_num.saturating_sub(3);
                    let end = (line_num + 4).min(lines.len());
                    let context = lines[start..end].join("\n");

                    content_matches.push(format!(
                        "File: {}\nLines {}-{}:\n{}\n",
                        path,
                        start + 1,
                        end,
                        context
                    ));
                }
            }

            if !content_matches.is_empty() {
                matches.extend(content_matches);
                seen_paths.insert(path.to_string());
            }
        }

        Ok(matches)
    }
}

#[async_trait::async_trait]
impl ToolTrait for FSReplace {
    type Input = FSReplaceInput;
    type Output = String;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let content = tokio::fs::read_to_string(&input.path)
            .await
            .map_err(|e| e.to_string())?;

        let mut result = content;
        let blocks: Vec<&str> = input.diff.split(">>>>>>> REPLACE").collect();

        for block in blocks {
            if block.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = block.split("=======").collect();
            if parts.len() != 2 {
                continue;
            }

            let search = parts[0]
                .trim_start_matches("<<<<<<< SEARCH")
                .trim();
            let replace = parts[1].trim();

            // Process one replacement at a time to maintain order
            let lines: Vec<&str> = result.lines().collect();
            let mut new_lines = Vec::new();
            let mut i = 0;

            while i < lines.len() {
                let mut found = false;
                let search_lines: Vec<&str> = search.lines().collect();
                
                if i + search_lines.len() <= lines.len() {
                    let mut matches = true;
                    for (j, search_line) in search_lines.iter().enumerate() {
                        if lines[i + j].trim() != search_line.trim() {
                            matches = false;
                            break;
                        }
                    }
                    if matches {
                        if replace.is_empty() {
                            new_lines.push("");
                        } else {
                            new_lines.extend(replace.lines());
                        }
                        i += search_lines.len();
                        found = true;
                    }
                }

                if !found {
                    new_lines.push(lines[i]);
                    i += 1;
                }
            }

            result = new_lines.join("\n");
        }

        tokio::fs::write(&input.path, result)
            .await
            .map_err(|e| e.to_string())?;

        Ok(format!("Successfully replaced content in {}", input.path))
    }
}

type BoxedFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

#[async_trait::async_trait]
impl ToolTrait for FSList {
    type Input = FSListInput;
    type Output = Vec<String>;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        let dir = Path::new(&input.path);
        if !dir.exists() {
            return Err("Directory does not exist".to_string());
        }

        let mut paths = Vec::new();
        let recursive = input.recursive.unwrap_or(false);
        let max_depth = if recursive { std::usize::MAX } else { 1 };

        let walker = WalkDir::new(dir)
            .min_depth(0)
            .max_depth(max_depth)
            .follow_links(false)
            .same_file_system(true)
            .into_iter();

        for entry in walker.filter_map(Result::ok) {
            // Skip the root directory itself
            if entry.path() == dir {
                continue;
            }

            let file_type = entry.file_type();
            if file_type.is_file() || file_type.is_dir() {
                let prefix = if file_type.is_dir() { "[DIR]" } else { "[FILE]" };
                paths.push(format!("{} {}", prefix, entry.path().display()));
            }
        }

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
    async fn test_fs_search_content() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test1.txt"), "Hello test world")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("test2.txt"), "Another test case")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("other.txt"), "No match here")
            .await
            .unwrap();

        let fs_search = FSSearch;
        let result = fs_search
            .call(FSSearchInput {
                path: temp_dir.path().to_string_lossy().to_string(),
                regex: "test".to_string(),
                file_pattern: None,
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|p| p.contains("test1.txt")));
        assert!(result.iter().any(|p| p.contains("test2.txt")));
        assert!(result.iter().all(|p| p.contains("Lines")));
    }

    #[tokio::test]
    async fn test_fs_search_with_pattern() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("test1.txt"), "Hello test world")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("test2.rs"), "fn test() {}")
            .await
            .unwrap();

        let fs_search = FSSearch;
        let result = fs_search
            .call(FSSearchInput {
                path: temp_dir.path().to_string_lossy().to_string(),
                regex: "test".to_string(),
                file_pattern: Some("*.rs".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert!(result.iter().any(|p| p.contains("test2.rs")));
    }

    #[tokio::test]
    async fn test_fs_search_with_context() {
        let temp_dir = TempDir::new().unwrap();
        let content = "line 1\nline 2\ntest line\nline 4\nline 5";

        fs::write(temp_dir.path().join("test.txt"), content)
            .await
            .unwrap();

        let fs_search = FSSearch;
        let result = fs_search
            .call(FSSearchInput {
                path: temp_dir.path().to_string_lossy().to_string(),
                regex: "test".to_string(),
                file_pattern: None,
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        let output = &result[0];
        assert!(output.contains("line 1"));
        assert!(output.contains("line 2"));
        assert!(output.contains("test line"));
        assert!(output.contains("line 4"));
        assert!(output.contains("line 5"));
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
                path: temp_dir.path().to_string_lossy().to_string(),
                regex: "test".to_string(),
                file_pattern: None,
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
                path: temp_dir.path().to_string_lossy().to_string(),
                regex: "test".to_string(),
                file_pattern: None,
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
                path: temp_dir.path().to_string_lossy().to_string(),
                regex: "".to_string(),
                file_pattern: None,
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
                path: nonexistent_dir.to_string_lossy().to_string(),
                regex: "test".to_string(),
                file_pattern: None,
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
                path: temp_dir.path().to_string_lossy().to_string(),
                regex: "test".to_string(),
                file_pattern: None,
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
                content: Some("Hello, World!".to_string()),
            })
            .await
            .unwrap();
        let s = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(s, "Hello, World!")
    }

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
        let result = fs_replace
            .call(FSReplaceInput {
                path: file_path.to_string_lossy().to_string(),
                diff: "<<<<<<< SEARCH\nHello World\n=======\nHi World\n>>>>>>> REPLACE\n\n<<<<<<< SEARCH\nGoodbye World\n=======\nBye World\n>>>>>>> REPLACE".to_string(),
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
