use forge_app::{EnvironmentService, Infrastructure};

use crate::auth::ForgeAuthService;
use crate::embedding::OpenAIEmbeddingService;
use crate::env::ForgeEnvironmentService;
use crate::file_read::ForgeFileReadService;
use crate::qdrant::QdrantVectorIndex;

pub struct ForgeInfra {
    auth_service: ForgeAuthService,
    file_read_service: ForgeFileReadService,
    environment_service: ForgeEnvironmentService,
    information_repo: QdrantVectorIndex,
    embedding_service: OpenAIEmbeddingService,
}

impl ForgeInfra {
    pub fn new(restricted: bool) -> Self {
        let _environment_service = ForgeEnvironmentService::new(restricted);
        let env = _environment_service.get_environment();
        Self {
            auth_service: ForgeAuthService::new(),
            file_read_service: ForgeFileReadService::new(),
            environment_service: _environment_service,
            information_repo: QdrantVectorIndex::new(env.clone(), "user_feedback"),
            embedding_service: OpenAIEmbeddingService::new(env),
        }
    }
}

impl Infrastructure for ForgeInfra {
    type CredentialRepository = ForgeAuthService;
    type EnvironmentService = ForgeEnvironmentService;
    type FileReadService = ForgeFileReadService;
    type VectorIndex = QdrantVectorIndex;
    type EmbeddingService = OpenAIEmbeddingService;

    fn credential_repository(&self) -> &Self::CredentialRepository {
        &self.auth_service
    }

    fn environment_service(&self) -> &Self::EnvironmentService {
        &self.environment_service
    }

    fn file_read_service(&self) -> &Self::FileReadService {
        &self.file_read_service
    }

    fn vector_index(&self) -> &Self::VectorIndex {
        &self.information_repo
    }

    fn embedding_service(&self) -> &Self::EmbeddingService {
        &self.embedding_service
    }
}
