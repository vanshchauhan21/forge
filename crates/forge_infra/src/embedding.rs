use anyhow::Context;
use forge_app::EmbeddingService;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use tokio::task;

pub struct ForgeEmbeddingService {}

impl Default for ForgeEmbeddingService {
    fn default() -> Self {
        Self::new()
    }
}

impl ForgeEmbeddingService {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl EmbeddingService for ForgeEmbeddingService {
    async fn embed(&self, sentence: &str) -> anyhow::Result<Vec<f32>> {
        let model = task::spawn_blocking(|| {
            SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
                .create_model()
                .context("Failed to initialize embedding model")
        })
        .await??;
        Ok(model.encode(&[sentence])?.concat())
    }
}
