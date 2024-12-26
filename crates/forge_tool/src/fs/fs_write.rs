use forge_tool_macros::Description as DescriptionDerive;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{Description, ToolTrait};

#[derive(Deserialize, JsonSchema)]
pub struct FSWriteInput {
    pub path: String,
    pub content: String,
}

/// Write the provided content to a file. This tool is useful for creating new
/// files or overwriting existing files with new content. Only works within
/// allowed directories.
#[derive(DescriptionDerive)]
pub struct FSWrite;

#[async_trait::async_trait]
impl ToolTrait for FSWrite {
    type Input = FSWriteInput;
    type Output = String;

    async fn call(&self, input: Self::Input) -> Result<Self::Output, String> {
        tokio::fs::write(&input.path, &input.content)
            .await
            .map_err(|e| e.to_string())?;
        Ok(format!("Successfully wrote to {}", input.path))
    }
}

#[cfg(test)]
mod test {
    use tempfile::TempDir;
    use tokio::fs;

    use super::*;

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
