use std::path::Path;
use std::sync::Arc;

use forge_app::{FileReadService, Infrastructure};
use forge_domain::Workflow;

// Default forge.toml content embedded in the binary
const DEFAULT_FORGE_TOML: &str = include_str!("../../../forge.toml");

/// A workflow loader to load the workflow from the given path.
/// It also resolves the internal paths specified in the workflow.
pub struct ForgeLoaderService<F>(Arc<F>);

impl<F> ForgeLoaderService<F> {
    pub fn new(app: Arc<F>) -> Self {
        Self(app)
    }
}

impl<F: Infrastructure> ForgeLoaderService<F> {
    /// loads the workflow from the given path.
    /// Loads the workflow from the given path if provided, otherwise tries to
    /// read from current directory's forge.toml, and falls back to embedded
    /// default if neither exists.
    pub async fn load(&self, path: Option<&Path>) -> anyhow::Result<Workflow> {
        let content = match path {
            Some(p) => self.0.file_read_service().read(p).await?,
            None => {
                let current_dir_config = Path::new("forge.toml");
                if current_dir_config.exists() {
                    self.0.file_read_service().read(current_dir_config).await?
                } else {
                    DEFAULT_FORGE_TOML.to_string()
                }
            }
        };

        let workflow: Workflow = content.parse()?;
        Ok(workflow)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use anyhow::Result;
    use forge_domain::Workflow;
    use forge_infra::ForgeInfra;
    use tempfile::TempDir;

    use super::ForgeLoaderService;

    const BASE_WORKFLOW: &str = r#"
[[agents]]
id = "developer"
model = "anthropic/claude-3.5-sonnet"
tools = ["tool_forge_fs_read", "tool_forge_fs_create"]
subscribe = ["user_task"]
max_turns = 1024"#;

    struct Fixture {
        temp_dir: TempDir,
        workflow_path: PathBuf,
        loader: ForgeLoaderService<ForgeInfra>,
    }

    impl Default for Fixture {
        fn default() -> Self {
            let temp_dir = tempfile::tempdir().unwrap();
            let loader = ForgeLoaderService::new(Arc::new(ForgeInfra::new(true)));
            Self { temp_dir, loader, workflow_path: PathBuf::from("forge.toml") }
        }
    }

    impl Fixture {
        async fn run(self, system_prompt: &str, user_prompt: &str) -> Result<Workflow> {
            let workflow = format!(
                "{}\n\n[agents.system_prompt]\ntemplate = \"{}\"\n\n[agents.user_prompt]\ntemplate = \"{}\"",
                BASE_WORKFLOW, system_prompt, user_prompt
            );
            let workflow_path = self.temp_dir.path().join(self.workflow_path.clone());
            tokio::fs::write(&workflow_path, workflow).await?;
            self.loader
                .load(Some(&self.temp_dir.path().join(self.workflow_path.clone())))
                .await
        }
    }

    #[tokio::test]
    async fn test_load_workflow_with_string_literals() -> Result<()> {
        let workflow = Fixture::default()
            .run(
                "You are a software developer assistant",
                "<task>{{event.value}}</task>",
            )
            .await?;
        insta::assert_snapshot!(serde_json::to_string_pretty(&workflow)?);
        Ok(())
    }

    #[tokio::test]
    async fn test_load_workflow_from_default() -> Result<()> {
        let loader = ForgeLoaderService::new(Arc::new(ForgeInfra::new(true)));
        let workflow = loader.load(None).await?;
        // Verify that the default workflow contains expected content
        assert!(serde_json::to_string(&workflow)?.contains("developer"));
        Ok(())
    }

    #[tokio::test]
    async fn test_load_workflow_from_current_dir() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        std::env::set_current_dir(&temp_dir)?;

        // Create a forge.toml in the current directory
        let workflow_content = format!(
            "{}\n\n[agents.system_prompt]\ntemplate = \"test\"\n\n[agents.user_prompt]\ntemplate = \"test\"",
            BASE_WORKFLOW
        );
        tokio::fs::write("forge.toml", workflow_content).await?;

        let loader = ForgeLoaderService::new(Arc::new(ForgeInfra::new(true)));
        let workflow = loader.load(None).await?;

        // Verify the workflow was loaded from the current directory
        assert!(serde_json::to_string(&workflow)?.contains("test"));
        Ok(())
    }
}
