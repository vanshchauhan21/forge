use std::sync::Arc;

use anyhow::Result;
use forge_domain::{EnvironmentService, File, Services};
use forge_services::Infrastructure;
use forge_walker::Walker;

pub struct ForgeSuggestionService<F> {
    domain: Arc<F>,
}

impl<F: Services> ForgeSuggestionService<F> {
    pub fn new(domain: Arc<F>) -> Self {
        Self { domain }
    }
}

impl<F: Infrastructure> ForgeSuggestionService<F> {
    pub async fn suggestions(&self) -> Result<Vec<File>> {
        let cwd = self
            .domain
            .environment_service()
            .get_environment()
            .cwd
            .clone();
        let walker = Walker::max_all().cwd(cwd);

        let files = walker.get().await?;
        Ok(files
            .into_iter()
            .map(|file| File { path: file.path.clone(), is_dir: file.is_dir() })
            .collect())
    }
}
