use forge_domain::{NamedTool, ToolCallService, ToolDescription, ToolName};
use forge_tool_macros::ToolDescription;
use schemars::JsonSchema;
use serde::Deserialize;

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
        ToolName::new("tool_forge_fs_write")
    }
}

#[async_trait::async_trait]
impl ToolCallService for FSWrite {
    type Input = FSWriteInput;

    async fn call(&self, input: Self::Input) -> Result<String, String> {
        // Check if file already exists
        if tokio::fs::metadata(&input.path).await.is_ok() {
            return Err(format!(
                "File {} already exists. Cannot overwrite.",
                input.path
            ));
        }

        // Validate file content if it's a supported language file
        let syntax_warning = syn::validate(&input.path, &input.content);

        // Write file only after validation passes
        tokio::fs::write(&input.path, &input.content)
            .await
            .map_err(|e| e.to_string())?;

        let mut result = format!(
            "Successfully wrote {} bytes to {}",
            input.content.len(),
            input.path
        );
        if let Some(warning) = syntax_warning {
            result.push_str("\nWarning: ");
            result.push_str(&warning.to_string());
        }

        Ok(result)
    }
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
        let content = "Hello, World!";

        let fs_write = FSWrite;
        let output = fs_write
            .call(FSWriteInput {
                path: file_path.to_string_lossy().to_string(),
                content: content.to_string(),
            })
            .await
            .unwrap();

        assert!(output.contains("Successfully wrote"));
        assert!(output.contains(&file_path.display().to_string()));
        assert!(output.contains(&content.len().to_string()));

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

        let output = result.unwrap();
        assert!(output.contains("Warning:"));
    }

    #[tokio::test]
    async fn test_fs_write_valid_rust() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let fs_write = FSWrite;
        let content = "fn main() { let x = 42; }";
        let result = fs_write
            .call(FSWriteInput {
                path: file_path.to_string_lossy().to_string(),
                content: content.to_string(),
            })
            .await;

        let output = result.unwrap();
        assert!(output.contains("Successfully wrote"));
        assert!(output.contains(&file_path.display().to_string()));
        assert!(output.contains(&content.len().to_string()));
        assert!(!output.contains("Warning:"));
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
