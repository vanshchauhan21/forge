use std::sync::Arc;

use forge_domain::{ChatRequest, ChatResponse, Context, ConversationRepository, ResultStream};
use tokio_stream::{once, StreamExt};
use tracing::debug;

use super::chat::ChatService;
use super::workflow_title::TitleService;
use crate::Service;

#[async_trait::async_trait]
pub trait UIService: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> ResultStream<ChatResponse, anyhow::Error>;
}

struct Live {
    conversation_service: Arc<dyn ConversationRepository>,
    chat_service: Arc<dyn ChatService>,
    title_service: Arc<dyn TitleService>,
}

impl Live {
    fn new(
        conversation_service: Arc<dyn ConversationRepository>,
        chat_service: Arc<dyn ChatService>,
        title_service: Arc<dyn TitleService>,
    ) -> Self {
        Self { conversation_service, chat_service, title_service }
    }
}

impl Service {
    pub fn ui_service(
        conversation_service: Arc<dyn ConversationRepository>,
        neo_chat_service: Arc<dyn ChatService>,
        title_service: Arc<dyn TitleService>,
    ) -> impl UIService {
        Live::new(conversation_service, neo_chat_service, title_service)
    }
}

#[async_trait::async_trait]
impl UIService for Live {
    async fn chat(&self, request: ChatRequest) -> ResultStream<ChatResponse, anyhow::Error> {
        let (conversation, is_new) = if let Some(conversation_id) = &request.conversation_id {
            let context = self.conversation_service.get(*conversation_id).await?;
            (context, false)
        } else {
            let conversation = self
                .conversation_service
                .insert(&Context::default(), None)
                .await?;
            (conversation, true)
        };

        debug!("Job {} started", conversation.id);
        let request = request.conversation_id(conversation.id);

        let mut stream = self
            .chat_service
            .chat(request.clone(), conversation.context)
            .await?;

        if is_new {
            let title_stream = self.title_service.get_title(request.clone()).await?;
            let id = conversation.id;
            stream = Box::pin(
                once(Ok(ChatResponse::ConversationStarted(id)))
                    .chain(title_stream)
                    .merge(stream),
            );
        }

        let conversation_service = self.conversation_service.clone();
        let stream = stream
            .then(move |message| {
                let conversation_service = conversation_service.clone();
                async move {
                    match &message {
                        Ok(ChatResponse::CompleteTitle(title)) => {
                            let conversation_id = request
                                .conversation_id
                                .expect("`conversation_id` must be set at this point.");
                            conversation_service
                                .set_title(&conversation_id, title.to_owned())
                                .await?;
                            message
                        }
                        Ok(ChatResponse::ModifyContext(context)) => {
                            conversation_service
                                .insert(context, request.conversation_id)
                                .await?;
                            message
                        }
                        _ => message,
                    }
                }
            })
            .filter(|message| !matches!(message, Ok(ChatResponse::ModifyContext { .. })));

        Ok(Box::pin(stream))
    }
}

#[cfg(test)]
mod tests {
    use forge_domain::ModelId;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::repo::test::TestConversationStorage;

    struct TestTitleService {
        events: Vec<ChatResponse>,
    }

    impl TestTitleService {
        fn single() -> Self {
            Self {
                events: vec![ChatResponse::CompleteTitle(
                    "test title generated".to_string(),
                )],
            }
        }
    }

    #[async_trait::async_trait]
    impl TitleService for TestTitleService {
        async fn get_title(&self, _: ChatRequest) -> ResultStream<ChatResponse, anyhow::Error> {
            Ok(Box::pin(tokio_stream::iter(
                self.events.clone().into_iter().map(Ok),
            )))
        }
    }

    struct TestChatService {
        events: Vec<ChatResponse>,
    }

    impl TestChatService {
        fn single() -> Self {
            Self { events: vec![ChatResponse::Text("test message".to_string())] }
        }
    }

    #[async_trait::async_trait]
    impl ChatService for TestChatService {
        async fn chat(
            &self,
            _: ChatRequest,
            _: Context,
        ) -> ResultStream<ChatResponse, anyhow::Error> {
            Ok(Box::pin(tokio_stream::iter(
                self.events.clone().into_iter().map(Ok),
            )))
        }
    }

    #[tokio::test]
    async fn test_chat_existing_conversation() {
        let conversation_service = Arc::new(TestConversationStorage::in_memory().unwrap());
        let service = Service::ui_service(
            conversation_service.clone(),
            Arc::new(TestChatService::single()),
            Arc::new(TestTitleService::single()),
        );

        let model_id = ModelId::new("gpt-3.5-turbo");
        let conversation = conversation_service
            .insert(&Context::default(), None)
            .await
            .unwrap();

        let request = ChatRequest::new(model_id, "test").conversation_id(conversation.id);
        let mut responses = service.chat(request).await.unwrap();

        if let Some(Ok(ChatResponse::Text(content))) = responses.next().await {
            assert_eq!(content, "test message");
        } else {
            panic!("Expected Text response");
        }

        assert!(responses.next().await.is_none(), "Expected end of stream");
    }
}
