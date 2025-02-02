use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Result;
use forge_domain::Environment;

use super::Service;

pub trait EnvironmentService {
    fn get(&self) -> Result<Environment>;
}

impl Service {
    pub fn environment_service(base_dir: Option<PathBuf>) -> impl EnvironmentService {
        Live::new(base_dir)
    }
}

struct Live {
    env: Mutex<Option<Environment>>,
    base_dir: Option<PathBuf>,
}

impl Live {
    pub fn new(base_dir: Option<PathBuf>) -> Self {
        Self { env: Mutex::new(None), base_dir }
    }

    fn from_env(cwd: Option<PathBuf>) -> Result<Environment> {
        dotenv::dotenv().ok();
        let api_key = std::env::var("FORGE_KEY").expect("FORGE_KEY must be set");
        let large_model_id =
            std::env::var("FORGE_LARGE_MODEL").unwrap_or("anthropic/claude-3.5-sonnet".to_owned());
        let small_model_id =
            std::env::var("FORGE_SMALL_MODEL").unwrap_or("anthropic/claude-3.5-haiku".to_owned());

        let cwd = if let Some(cwd) = cwd {
            cwd
        } else {
            std::env::current_dir()?
        };

        Ok(Environment {
            os: std::env::consts::OS.to_string(),
            cwd,
            shell: if cfg!(windows) {
                std::env::var("COMSPEC")?
            } else {
                std::env::var("SHELL").unwrap_or("/bin/sh".to_string())
            },
            api_key,
            large_model_id,
            small_model_id,
            base_path: base_path(),
            home: dirs::home_dir(),
        })
    }
}

fn base_path() -> PathBuf {
    dirs::config_dir()
        .map(|a| a.join("forge"))
        .unwrap_or(PathBuf::from(".").join(".forge"))
}

impl EnvironmentService for Live {
    fn get(&self) -> Result<Environment> {
        let mut guard = self.env.lock().unwrap();

        if let Some(env) = guard.as_ref() {
            Ok(env.clone())
        } else {
            *guard = Some(Live::from_env(self.base_dir.clone())?);
            Ok(guard.as_ref().unwrap().clone())
        }
    }
}
