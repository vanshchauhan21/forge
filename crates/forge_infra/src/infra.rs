use forge_app::{EnvironmentService, Infrastructure};

use crate::embedding::ForgeEmbeddingService;
use crate::env::ForgeEnvironmentService;
use crate::file_read::ForgeFileReadService;
use crate::knowledge::QdrantKnowledgeRepository;

pub struct ForgeInfra {
    _file_read_service: ForgeFileReadService,
    _environment_service: ForgeEnvironmentService,
    _information_repo: QdrantKnowledgeRepository,
    _embedding_service: ForgeEmbeddingService,
}

impl ForgeInfra {
    pub fn new(restricted: bool) -> Self {
        let _environment_service = ForgeEnvironmentService::new(restricted);
        let env = _environment_service.get_environment();
        Self {
            _file_read_service: ForgeFileReadService::new(),
            _environment_service,
            _information_repo: QdrantKnowledgeRepository::new(env, "user_feedback"),
            _embedding_service: ForgeEmbeddingService::new(),
        }
    }
}

impl Infrastructure for ForgeInfra {
    type EnvironmentService = ForgeEnvironmentService;
    type FileReadService = ForgeFileReadService;
    type KnowledgeRepository = QdrantKnowledgeRepository;
    type EmbeddingService = ForgeEmbeddingService;

    fn environment_service(&self) -> &Self::EnvironmentService {
        &self._environment_service
    }

    fn file_read_service(&self) -> &Self::FileReadService {
        &self._file_read_service
    }

    fn textual_knowledge_repo(&self) -> &Self::KnowledgeRepository {
        &self._information_repo
    }

    fn embedding_service(&self) -> &Self::EmbeddingService {
        &self._embedding_service
    }
}
