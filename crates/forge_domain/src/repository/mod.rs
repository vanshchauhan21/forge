mod configuration;
mod context;
mod conversation;
mod environment;
mod learning;
mod user_interaction;

pub use configuration::ConfigurationRepository;
pub use context::ContextRepository;
pub use conversation::ConversationRepository;
pub use environment::EnvironmentRepository;
pub use learning::{Learning, LearningId, LearningRepository};
pub use user_interaction::UserInteractionPort;

/// Domain module trait that provides access to all repositories and ports
pub trait DomainModule: Send + Sync {
    type ConversationRepository: ConversationRepository + Send + Sync;
    type ConfigurationRepository: ConfigurationRepository + Send + Sync;
    type ContextRepository: ContextRepository + Send + Sync;
    type UserInteractionPort: UserInteractionPort + Send + Sync;
    type EnvironmentRepository: EnvironmentRepository + Send + Sync;
    type LearningRepository: LearningRepository + Send + Sync;

    /// Get the conversation repository implementation
    fn conversation_repository(&self) -> &Self::ConversationRepository;

    /// Get the configuration repository implementation
    fn configuration_repository(&self) -> &Self::ConfigurationRepository;

    /// Get the context repository implementation
    fn context_repository(&self) -> &Self::ContextRepository;

    /// Get the user interaction port implementation
    fn user_interaction_port(&self) -> &Self::UserInteractionPort;

    /// Get the environment repository implementation
    fn environment_repository(&self) -> &Self::EnvironmentRepository;

    /// Get the learning repository implementation
    fn learning_repository(&self) -> &Self::LearningRepository;
}
