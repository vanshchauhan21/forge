use forge_domain::{NamedTool, ToolCallService, ToolDescription, ToolName};
use forge_tool_macros::ToolDescription;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::fs::syn;

#[derive(Deserialize, JsonSchema)]
pub struct FSWriteInput {
    /// The path of the file to write to (relative to the current working
    /// directory)
    pub path: String,
    /// The content to write to the file. ALWAYS provide the COMPLETE intended
    /// content of the file, without any truncation or omissions. You MUST
    /// include ALL parts of the file, even if they haven't been modified.
    pub content: String,
}

/// Use it to create a new file at a specified path with the provided content.
/// If the file already exists, the tool will return an error to prevent
/// overwriting. The tool automatically handles the creation of any missing
/// directories in the specified path, ensuring that the new file can be created
/// seamlessly. Use this tool only when creating files that do not yet exist.
#[derive(ToolDescription)]
pub struct FSWrite;

impl NamedTool for FSWrite {
    fn tool_name(&self) -> ToolName {
        ToolName::new("write_file")
    }
}

#[async_trait::async_trait]
impl ToolCallService for FSWrite {
    type Input = FSWriteInput;
    type Output = FSWriteOutput;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        // Check if file already exists
        if tokio::fs::metadata(&input.path).await.is_ok() {
            return Err(format!(
                "File {} already exists. Cannot overwrite.",
                input.path
            ));
        }

        // Validate file content if it's a supported language file
        let syntax_checker = syn::validate(&input.path, &input.content).err();

        // Write file only after validation passes
        tokio::fs::write(&input.path, &input.content)
            .await
            .map_err(|e| e.to_string())?;

        Ok(FSWriteOutput { path: input.path, syntax_checker, content: input.content })
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct FSWriteOutput {
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub syntax_checker: Option<String>,
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;
    use tokio::fs;

    use super::*;

    #[tokio::test]
    async fn test_fs_write_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let fs_write = FSWrite;
        let output = fs_write
            .call(FSWriteInput {
                path: file_path.to_string_lossy().to_string(),
                content: "Hello, World!".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(output.path, file_path.to_string_lossy().to_string());
        assert_eq!(output.content, "Hello, World!");

        // Verify file was actually written
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Hello, World!")
    }

    #[tokio::test]
    async fn test_fs_write_invalid_rust() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let fs_write = FSWrite;
        let result = fs_write
            .call(FSWriteInput {
                path: file_path.to_string_lossy().to_string(),
                content: "fn main() { let x = ".to_string(),
            })
            .await;

        assert!(result.unwrap().syntax_checker.is_some());
    }

    #[tokio::test]
    async fn test_fs_write_valid_rust() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let fs_write = FSWrite;
        let result = fs_write
            .call(FSWriteInput {
                path: file_path.to_string_lossy().to_string(),
                content: "fn main() { let x = 42; }".to_string(),
            })
            .await;

        assert!(result.is_ok());
        // Verify file contains valid Rust code
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "fn main() { let x = 42; }");
    }

    #[tokio::test]
    async fn test_fs_write_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // Create the file first
        fs::write(&file_path, "Existing content").await.unwrap();

        let fs_write = FSWrite;
        let result = fs_write
            .call(FSWriteInput {
                path: file_path.to_string_lossy().to_string(),
                content: "New content".to_string(),
            })
            .await;

        // Check that the result is an error
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));

        // Verify original content remains unchanged
        let content = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(content, "Existing content");
    }
}
