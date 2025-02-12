use std::path::PathBuf;

use forge_app::{EnvironmentService, Infrastructure};
use forge_domain::{Environment, ModelId};

use crate::file_read::ForgeFileReadService;

pub struct TestEnvironmentService {
    large_model_id: ModelId,
    small_model_id: ModelId,
}

impl TestEnvironmentService {
    pub fn new(large_model_id: ModelId, small_model_id: ModelId) -> Self {
        Self { large_model_id, small_model_id }
    }
}

impl EnvironmentService for TestEnvironmentService {
    fn get_environment(&self) -> Environment {
        dotenv::dotenv().ok();
        let cwd = std::env::current_dir().unwrap_or(PathBuf::from("."));
        let api_key = std::env::var("OPEN_ROUTER_KEY").expect("OPEN_ROUTER_KEY must be set");
        Environment {
            os: std::env::consts::OS.to_string(),
            cwd,
            shell: "/bin/sh".to_string(),
            api_key,
            large_model_id: self.large_model_id.clone(),
            small_model_id: self.small_model_id.clone(),
            base_path: dirs::config_dir()
                .map(|a| a.join("forge"))
                .unwrap_or(PathBuf::from(".").join(".forge")),
            home: dirs::home_dir(),
        }
    }
}

pub struct TestInfra {
    _file_read_service: ForgeFileReadService,
    _environment_service: TestEnvironmentService,
}

impl TestInfra {
    pub fn new(large_model_id: ModelId, small_model_id: ModelId) -> Self {
        Self {
            _file_read_service: ForgeFileReadService::new(),
            _environment_service: TestEnvironmentService::new(large_model_id, small_model_id),
        }
    }
}

impl Infrastructure for TestInfra {
    type EnvironmentService = TestEnvironmentService;
    type FileReadService = ForgeFileReadService;

    fn environment_service(&self) -> &Self::EnvironmentService {
        &self._environment_service
    }

    fn file_read_service(&self) -> &Self::FileReadService {
        &self._file_read_service
    }
}
