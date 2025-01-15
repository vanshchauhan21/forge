mod error;
mod open_router;
mod provider;

pub use error::*;
use forge_domain::ProviderService;

pub struct Service;

impl Service {
    /// Creates a new OpenRouter provider instance
    pub fn open_router(api_key: impl ToString) -> impl ProviderService {
        open_router::provider::OpenRouter::new(api_key).into_provider()
    }
}
