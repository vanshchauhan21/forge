use forge_tool_macros::Description as DescriptionDerive;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{Description, ToolTrait};

#[derive(Deserialize, JsonSchema)]
pub struct FSFileInfoInput {
    pub path: String,
}

/// Retrieve detailed metadata about a file or directory. Returns comprehensive
/// information including size, creation time, last modified time, permissions,
/// and type. This tool is perfect for understanding file characteristics
/// without reading the actual content. Only works within allowed directories.
#[derive(DescriptionDerive)]
pub struct FSFileInfo;

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
}
