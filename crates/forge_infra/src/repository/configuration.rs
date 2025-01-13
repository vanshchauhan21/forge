use anyhow::Result;
use async_trait::async_trait;
use forge_domain::{Config, ConfigurationRepository};

pub struct SqliteConfigurationRepository {
    // Implementation details will be added later
}

impl Default for SqliteConfigurationRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl SqliteConfigurationRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ConfigurationRepository for SqliteConfigurationRepository {
    async fn get_configuration(&self) -> Result<Config> {
        // Implementation will be added later
        todo!()
    }

    async fn save_configuration(&self, _config: &Config) -> Result<()> {
        // Implementation will be added later
        todo!()
    }
}
