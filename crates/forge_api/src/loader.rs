use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use forge_app::{FileReadService, Infrastructure};
use forge_domain::{Prompt, Workflow};

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
    pub async fn load(&self, path: &Path) -> anyhow::Result<Workflow> {
        let workflow_content = self.0.file_read_service().read(path).await?;
        let workflow: Workflow = workflow_content.parse()?;

        let workflow_dir = path
            .parent()
            .with_context(|| {
                format!(
                    "Failed to get parent directory of workflow file: {:?}",
                    path
                )
            })?
            .to_path_buf();

        self.resolve(workflow_dir, workflow).await
    }

    /// given an workflow, it resolves all the internal paths specified in
    /// workflow.
    async fn resolve(&self, path: PathBuf, mut workflow: Workflow) -> Result<Workflow> {
        for agent in workflow.agents.iter_mut() {
            agent.system_prompt = self.resolve_prompt(&path, &agent.system_prompt).await?;
            agent.user_prompt = self.resolve_prompt(&path, &agent.user_prompt).await?;
        }
        Ok(workflow)
    }

    /// if prompt is a file path, then it reads the file and returns the
    /// content. if the file path is relative, it resolves it with the given
    /// path. otherwise, it returns the prompt as it is.
    async fn resolve_prompt<V: Clone>(&self, path: &Path, prompt: &Prompt<V>) -> Result<Prompt<V>> {
        if let Some(file_path) = Self::is_file_path(path, prompt.template.as_str()) {
            let path = if file_path.is_absolute() {
                file_path
            } else {
                path.join(file_path)
            };
            Ok(Prompt::new(
                self.0.file_read_service().read(path.as_path()).await?,
            ))
        } else {
            Ok(prompt.clone())
        }
    }

    /// checks if given content is valid file path by checking it's existence.
    fn is_file_path(workflow_dir: &Path, content: &str) -> Option<PathBuf> {
        let path = Path::new(content);
        if path.exists() || workflow_dir.join(path).exists() {
            return Some(path.to_path_buf());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
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
            Self {
                temp_dir,
                loader,
                workflow_path: PathBuf::from("workflow.toml"),
            }
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
                .load(&self.temp_dir.path().join(self.workflow_path.clone()))
                .await
        }

        async fn create_prompt_file(&self, path: &Path, content: &str) -> Result<()> {
            let file_path = self.temp_dir.path().join(path);
            if let Some(parent) = file_path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            tokio::fs::write(file_path, content).await?;
            Ok(())
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
    async fn test_load_workflow_with_file_prompt() -> Result<()> {
        let fixture = Fixture::default();
        // create system prompt file
        let system_prompt = "You are a software developer assistant, good at solving complex software engineering problems";
        let system_prompt_path = Path::new("nested/system_prompt.md");
        fixture
            .create_prompt_file(system_prompt_path, system_prompt)
            .await?;

        let workflow = fixture
            .run("nested/system_prompt.md", "<task>{{event.value}}</task>")
            .await?;
        insta::assert_snapshot!(serde_json::to_string_pretty(&workflow)?);
        Ok(())
    }

    #[tokio::test]
    async fn test_load_workflow_with_missing_prompt_file() -> Result<()> {
        let workflow = Fixture::default()
            .run("nested/system_prompt.md", "<task>{{event.value}}</task>")
            .await?;
        insta::assert_snapshot!(serde_json::to_string_pretty(&workflow)?);
        Ok(())
    }
}
