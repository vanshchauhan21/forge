mod app;
mod attachment;
mod conversation;
mod provider;
mod template;
mod tool_service;
mod tools;

use std::path::Path;

pub use app::*;
use bytes::Bytes;
use forge_domain::{Point, Query, Suggestion};
use forge_oauth::AuthFlowState;

#[async_trait::async_trait]
pub trait CredentialRepository: Send + Sync + 'static {
    /// Returns the current authentication state
    fn create(&self) -> AuthFlowState;

    /// Authenticates the user and stores credentials
    async fn authenticate(&self, state: AuthFlowState) -> anyhow::Result<()>;

    /// Logs out the user by removing stored credentials
    /// Returns true if credentials were found and removed, false otherwise
    fn delete(&self) -> anyhow::Result<bool>;

    /// Retrieves the current authentication token if available
    /// Returns the token as a string if found, or an error if not authenticated
    fn credentials(&self) -> Option<String>;
}

/// Repository for accessing system environment information
#[async_trait::async_trait]
pub trait EnvironmentService {
    /// Get the current environment information including:
    /// - Operating system
    /// - Current working directory
    /// - Home directory
    /// - Default shell
    fn get_environment(&self) -> forge_domain::Environment;
}

/// A service for reading files from the filesystem.
///
/// This trait provides an abstraction over file reading operations, allowing
/// for both real file system access and test mocking.
#[async_trait::async_trait]
pub trait FileReadService: Send + Sync {
    /// Reads the content of a file at the specified path.
    async fn read(&self, path: &Path) -> anyhow::Result<Bytes>;
}

#[async_trait::async_trait]
pub trait VectorIndex<T>: Send + Sync {
    async fn store(&self, point: Point<T>) -> anyhow::Result<()>;
    async fn search(&self, query: Query) -> anyhow::Result<Vec<Point<T>>>;
}

#[async_trait::async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
}

pub trait Infrastructure: Send + Sync + 'static {
    type CredentialRepository: CredentialRepository;
    type EnvironmentService: EnvironmentService;
    type FileReadService: FileReadService;
    type VectorIndex: VectorIndex<Suggestion>;
    type EmbeddingService: EmbeddingService;

    fn credential_repository(&self) -> &Self::CredentialRepository;
    fn environment_service(&self) -> &Self::EnvironmentService;
    fn file_read_service(&self) -> &Self::FileReadService;
    fn vector_index(&self) -> &Self::VectorIndex;
    fn embedding_service(&self) -> &Self::EmbeddingService;
}
