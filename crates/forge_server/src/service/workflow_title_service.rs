use std::sync::Arc;

use forge_domain::{Context, ContextMessage, ResultStream};
use forge_provider::ProviderService;
use handlebars::Handlebars;
use serde::Serialize;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use super::{ChatRequest, ChatResponse, Service};
use crate::{Error, Result};

impl Service {
    pub fn title_service(provider: Arc<dyn ProviderService>) -> impl TitleService {
        Live::new(provider)
    }
}

#[async_trait::async_trait]
pub trait TitleService: Send + Sync {
    async fn get_title(&self, content: ChatRequest) -> ResultStream<ChatResponse, Error>;
}

#[derive(Clone)]
struct Live {
    provider: Arc<dyn ProviderService>,
}

#[derive(Serialize)]
struct UserPromptContext {
    task: String,
}

impl Live {
    fn new(provider: Arc<dyn ProviderService>) -> Self {
        Self { provider }
    }

    fn system_prompt(&self) -> Result<String> {
        let template = include_str!("../prompts/title/system.md");
        Ok(template.to_string())
    }

    fn user_prompt(&self, task: &str) -> Result<String> {
        let template = include_str!("../prompts/title/user_task.md");

        let mut hb = Handlebars::new();
        hb.set_strict_mode(true);
        hb.register_escape_fn(|str| str.to_string());

        let ctx = UserPromptContext { task: task.to_string() };

        Ok(hb.render_template(template, &ctx)?)
    }

    async fn execute(
        &self,
        request: Context,
        tx: tokio::sync::mpsc::Sender<Result<ChatResponse>>,
    ) -> Result<()> {
        let mut response = self.provider.chat(request).await?;
        let mut title = String::new();
        while let Some(chunk) = response.next().await {
            let message = chunk?;
            if let Some(ref content) = message.content {
                if !content.is_empty() {
                    tx.send(Ok(ChatResponse::PartialTitle(content.as_str().to_string())))
                        .await
                        .unwrap();
                    title.push_str(content.as_str());
                }
            }
        }

        // send the complete the title, so that we can save it in the database.
        tx.send(Ok(ChatResponse::CompleteTitle(title)))
            .await
            .unwrap();

        Ok(())
    }
}

#[async_trait::async_trait]
impl TitleService for Live {
    async fn get_title(&self, chat: ChatRequest) -> ResultStream<ChatResponse, Error> {
        let system_prompt = self.system_prompt()?;
        let user_prompt = self.user_prompt(&chat.content)?;
        let request = Context::default()
            .set_system_message(system_prompt)
            .add_message(ContextMessage::user(user_prompt))
            .model(chat.model);
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        let that = self.clone();
        tokio::spawn(async move {
            match that.execute(request, tx.clone()).await {
                Ok(_) => {}
                Err(e) => tx.send(Err(e)).await.unwrap(),
            };
        });
        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::vec;

    use derive_setters::Setters;
    use forge_domain::{ChatCompletionMessage, FinishReason};
    use tokio_stream::StreamExt;

    use super::{ChatRequest, Live, TitleService};
    use crate::service::tests::TestProvider;
    use crate::service::{ChatResponse, ConversationId};

    #[derive(Default, Setters)]
    #[setters(into, strip_option)]
    struct Fixture {
        assistant_responses: Vec<Vec<ChatCompletionMessage>>,
    }

    impl Fixture {
        pub async fn run(&self, request: ChatRequest) -> Vec<ChatResponse> {
            let provider =
                Arc::new(TestProvider::default().with_messages(self.assistant_responses.clone()));
            let chat = Live::new(provider.clone());

            chat.get_title(request)
                .await
                .unwrap()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .map(|message| message.unwrap())
                .collect::<Vec<_>>()
        }
    }

    #[tokio::test]
    async fn test_title_generation() {
        let mock_llm_responses = vec![vec![ChatCompletionMessage::default()
            .content_part("Fibonacci Sequence in Rust")
            .finish_reason(FinishReason::Stop)]];

        let actual = Fixture::default()
            .assistant_responses(mock_llm_responses)
            .run(
                ChatRequest::new("write an rust program to generate an fibo seq.")
                    .conversation_id(ConversationId::new("5af97419-0277-410a-8ca6-0e2a252152c5")),
            )
            .await;

        assert!(actual.contains(&ChatResponse::CompleteTitle(
            "Fibonacci Sequence in Rust".to_string()
        )));
    }

    #[tokio::test]
    async fn test_user_prompt() {
        let provider = Arc::new(TestProvider::default());
        let chat = Live::new(provider);

        insta::assert_snapshot!(chat
            .user_prompt("write an rust program to generate an fibo seq.")
            .unwrap());
    }

    #[tokio::test]
    async fn test_system_prompt() {
        let provider = Arc::new(TestProvider::default());
        let chat = Live::new(provider);

        insta::assert_snapshot!(chat.system_prompt().unwrap());
    }
}
