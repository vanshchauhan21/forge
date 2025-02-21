mod app;
mod conversation;
mod prompt;
mod provider;
mod tool_service;
mod tools;

use std::path::Path;

pub use app::*;
use forge_domain::{Knowledge, Query};
use serde_json::Value;

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
    async fn read(&self, path: &Path) -> anyhow::Result<String>;
}

#[async_trait::async_trait]
pub trait KnowledgeRepository<T>: Send + Sync {
    async fn store(&self, information: Vec<Knowledge<T>>) -> anyhow::Result<()>;
    async fn search(&self, query: Query) -> anyhow::Result<Vec<Value>>;
}

#[async_trait::async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
}

pub trait Infrastructure: Send + Sync + 'static {
    type EnvironmentService: EnvironmentService;
    type FileReadService: FileReadService;
    type KnowledgeRepository: KnowledgeRepository<Value>;
    type EmbeddingService: EmbeddingService;

    fn environment_service(&self) -> &Self::EnvironmentService;
    fn file_read_service(&self) -> &Self::FileReadService;
    fn textual_knowledge_repo(&self) -> &Self::KnowledgeRepository;
    fn embedding_service(&self) -> &Self::EmbeddingService;
}
