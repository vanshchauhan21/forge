use std::sync::Arc;

use anyhow::Result;
use forge_domain::{ChatRequest, FileReadService};
use forge_prompt::Prompt;
use handlebars::Handlebars;
use serde::Serialize;

use super::{PromptService, Service};

impl Service {
    pub fn user_prompt_service(file_read: Arc<dyn FileReadService>) -> impl PromptService {
        Live { file_read }
    }
}

struct Live {
    file_read: Arc<dyn FileReadService>,
}

#[derive(Serialize)]
struct Context {
    task: String,
    files: Vec<FileRead>,
}

#[derive(Serialize)]
struct FileRead {
    path: String,
    content: String,
}

#[async_trait::async_trait]
impl PromptService for Live {
    async fn get(&self, request: &ChatRequest) -> Result<String> {
        let template = include_str!("../prompts/coding/user_task.md");
        let parsed_task = Prompt::parse(request.content.to_string());

        let mut file_contents = vec![];
        for file_path in parsed_task.files() {
            let content = self.file_read.read(file_path.clone().into()).await?;
            file_contents.push(FileRead { path: file_path, content });
        }

        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        hb.register_escape_fn(|str| str.to_string());

        let ctx = Context { task: request.content.to_string(), files: file_contents };

        Ok(hb.render_template(template, &ctx)?)
    }
}

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::service::test::TestFileReadService;

    #[tokio::test]
    async fn test_render_user_prompt() {
        let file_read = Arc::new(
            TestFileReadService::default()
                .add("foo.txt", "Hello World - Foo")
                .add("bar.txt", "Hello World - Bar"),
        );

        let request = ChatRequest::new(
            forge_domain::ModelId::new("gpt-3.5-turbo"),
            "read this file content from @foo.txt and @bar.txt",
        );
        let rendered_prompt = Service::user_prompt_service(file_read)
            .get(&request)
            .await
            .unwrap();
        insta::assert_snapshot!(rendered_prompt);
    }
}
