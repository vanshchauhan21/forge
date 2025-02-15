use std::sync::Arc;

use forge_domain::App;

use crate::conversation::ForgeConversationService;
use crate::provider::ForgeProviderService;
use crate::tool_service::ForgeToolService;
use crate::Infrastructure;

pub struct ForgeApp<F> {
    infra: Arc<F>,
    _tool_service: ForgeToolService,
    _provider_service: ForgeProviderService,
    _conversation_service: ForgeConversationService,
}

impl<F: Infrastructure> ForgeApp<F> {
    pub fn new(infra: Arc<F>) -> Self {
        Self {
            infra: infra.clone(),
            _tool_service: ForgeToolService::new(infra.clone()),
            _provider_service: ForgeProviderService::new(infra.clone()),
            _conversation_service: ForgeConversationService::new(),
        }
    }
}

impl<F: Infrastructure> App for ForgeApp<F> {
    type ToolService = ForgeToolService;
    type ProviderService = ForgeProviderService;
    type ConversationService = ForgeConversationService;

    fn tool_service(&self) -> &Self::ToolService {
        &self._tool_service
    }

    fn provider_service(&self) -> &Self::ProviderService {
        &self._provider_service
    }

    fn conversation_service(&self) -> &Self::ConversationService {
        &self._conversation_service
    }
}

impl<F: Infrastructure> Infrastructure for ForgeApp<F> {
    type EnvironmentService = F::EnvironmentService;
    type FileReadService = F::FileReadService;

    fn environment_service(&self) -> &Self::EnvironmentService {
        self.infra.environment_service()
    }

    fn file_read_service(&self) -> &Self::FileReadService {
        self.infra.file_read_service()
    }
}
