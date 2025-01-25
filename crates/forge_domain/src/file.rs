use std::path::PathBuf;

use anyhow::Result;

/// A service for reading files from the filesystem.
///
/// This trait provides an abstraction over file reading operations, allowing
/// for both real file system access and test mocking.
///
/// # Example
/// ```rust,no_run
/// use std::path::PathBuf;
/// use forge_domain::FileReadService;
///
/// # async fn example(file_service: impl FileReadService) -> anyhow::Result<()> {
/// let content = file_service.read(PathBuf::from("config.toml")).await?;
/// println!("File content: {}", content);
/// # Ok(())
/// # }
/// ```
#[async_trait::async_trait]
pub trait FileReadService: Send + Sync {
    /// Reads the content of a file at the specified path.
    ///
    /// # Arguments
    /// * `path` - The path to the file to read
    ///
    /// # Returns
    /// * `Result<String>` - The content of the file if successful, or an error
    ///   if the file cannot be read
    async fn read(&self, path: PathBuf) -> Result<String>;
}
