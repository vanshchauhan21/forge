use std::sync::Arc;

use forge_domain::{Context, ResultStream};
use tokio_stream::{once, StreamExt};
use tracing::info;

use super::chat_service::ChatService;
use super::{ChatRequest, ChatResponse, ConversationService};
use crate::{Error, Service};

#[async_trait::async_trait]
pub trait UIService: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> ResultStream<ChatResponse, Error>;
}

struct Live {
    conversation_service: Arc<dyn ConversationService>,
    chat_service: Arc<dyn ChatService>,
}

impl Live {
    fn new(
        conversation_service: Arc<dyn ConversationService>,
        chat_service: Arc<dyn ChatService>,
    ) -> Self {
        Self { conversation_service, chat_service }
    }
}

impl Service {
    pub fn ui_service(
        conversation_service: Arc<dyn ConversationService>,
        neo_chat_service: Arc<dyn ChatService>,
    ) -> impl UIService {
        Live::new(conversation_service, neo_chat_service)
    }
}

#[async_trait::async_trait]
impl UIService for Live {
    async fn chat(&self, request: ChatRequest) -> ResultStream<ChatResponse, Error> {
        let (conversation, is_new) = if let Some(conversation_id) = &request.conversation_id {
            let context = self
                .conversation_service
                .get_conversation(*conversation_id)
                .await?;
            (context, false)
        } else {
            let conversation = self
                .conversation_service
                .set_conversation(&Context::default(), None)
                .await?;
            (conversation, true)
        };

        info!("Job {} started", conversation.id.as_uuid());
        let request = request.conversation_id(conversation.id);

        let mut stream = self
            .chat_service
            .chat(request.clone(), conversation.context)
            .await?;

        if is_new {
            let id = conversation.id;
            stream = Box::pin(once(Ok(ChatResponse::ConversationStarted(id))).chain(stream));
        }

        let conversation_service = self.conversation_service.clone();
        let stream = stream
            .then(move |message| {
                let conversation_service = conversation_service.clone();
                async move {
                    if let Ok(ChatResponse::ModifyContext(context)) = &message {
                        conversation_service
                            .set_conversation(context, request.conversation_id)
                            .await?;
                        message
                    } else {
                        message
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

    use super::super::conversation_service::tests::TestStorage;
    use super::*;

    #[async_trait::async_trait]
    impl ChatService for TestStorage {
        async fn chat(&self, _: ChatRequest, _: Context) -> ResultStream<ChatResponse, Error> {
            Ok(Box::pin(once(Ok(ChatResponse::Text(
                "test message".to_string(),
            )))))
        }
    }

    impl Default for ChatRequest {
        fn default() -> Self {
            Self {
                content: "test".to_string(),
                model: ModelId::default(),
                conversation_id: None,
            }
        }
    }

    #[tokio::test]
    async fn test_chat_new_conversation() {
        let storage = Arc::new(TestStorage);
        let conversation_service = Arc::new(TestStorage::in_memory().unwrap());
        let service = Service::ui_service(conversation_service.clone(), storage.clone());
        let request = ChatRequest::default();

        let mut responses = service.chat(request).await.unwrap();

        if let Some(Ok(ChatResponse::ConversationStarted(_))) = responses.next().await {
        } else {
            panic!("Expected ConversationStarted response");
        }

        if let Some(Ok(ChatResponse::Text(content))) = responses.next().await {
            assert_eq!(content, "test message");
        } else {
            panic!("Expected Text response");
        }
    }

    #[tokio::test]
    async fn test_chat_existing_conversation() {
        let storage = Arc::new(TestStorage);
        let conversation_service = Arc::new(TestStorage::in_memory().unwrap());
        let service = Service::ui_service(conversation_service.clone(), storage.clone());

        let conversation = conversation_service
            .set_conversation(&Context::default(), None)
            .await
            .unwrap();

        let request = ChatRequest::default().conversation_id(conversation.id);
        let mut responses = service.chat(request).await.unwrap();

        if let Some(Ok(ChatResponse::Text(content))) = responses.next().await {
            assert_eq!(content, "test message");
        } else {
            panic!("Expected Text response");
        }
    }
}
