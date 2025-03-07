use forge_app::CredentialRepository;
use forge_oauth::{AuthFlowState, ClerkAuthClient, ClerkConfig};

pub struct ForgeAuthService {
    auth_client: ClerkAuthClient,
}

impl ForgeAuthService {
    pub fn new() -> Self {
        // Create configuration for Clerk OAuth
        let config = ClerkConfig {
            client_id: "wmVgjvoQ3hUcGbk8".to_string(),
            redirect_url: "http://localhost:8080/callback".to_string(),
            auth_url: "https://clerk.antinomy.ai/oauth/authorize".to_string(),
            token_url: "https://clerk.antinomy.ai/oauth/token".to_string(),
            user_info_url: "https://clerk.antinomy.ai/oauth/userinfo".to_string(),
            issuer_url: "https://clerk.antinomy.ai".to_string(),
            scope: "email".to_string(),
        };
        let key_url = "https://antinomy.ai/api/v1/key".to_string();

        // Initialize the auth client
        let auth_client = ClerkAuthClient::new(config, key_url)
            .expect("Failed to initialize authentication client");

        Self { auth_client }
    }
}

#[async_trait::async_trait]
impl CredentialRepository for ForgeAuthService {
    fn create(&self) -> AuthFlowState {
        // Generate the authorization URL
        self.auth_client.generate_auth_url()
    }
    async fn authenticate(&self, auth_flow_state: AuthFlowState) -> anyhow::Result<()> {
        // Perform the OAuth flow which will store the token in the keychain
        self.auth_client.complete_auth_flow(auth_flow_state).await
    }

    fn delete(&self) -> anyhow::Result<bool> {
        // Delete the token from the keychain
        self.auth_client.delete_key_from_keychain()
    }

    fn credentials(&self) -> Option<String> {
        // Get the token from the keychain
        self.auth_client.get_key_from_keychain()
    }
}

impl Default for ForgeAuthService {
    fn default() -> Self {
        Self::new()
    }
}
