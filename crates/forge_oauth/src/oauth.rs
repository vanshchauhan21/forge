//! A library for handling OAuth2 authentication with Clerk using OpenID
//! Connect.
//!
//! This crate provides functionality to authenticate users via Clerk's OAuth2
//! implementation with PKCE (Proof Key for Code Exchange) for enhanced security
//! using the OpenID Connect standard.

use anyhow::{Context as _, Result};
use openidconnect::core::{CoreClient, CoreResponseType};
use openidconnect::{
    AuthUrl, AuthenticationFlow, ClientId, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde_json::json;

use crate::callback::CallbackServer;
use crate::error::AuthError;
use crate::user_info::UserInfoClient;

/// Configuration for the OpenID Connect client
#[derive(Clone, Debug)]
pub struct ClerkConfig {
    pub client_id: String,
    pub redirect_url: String,
    pub auth_url: String,
    pub token_url: String,
    pub user_info_url: String,
    pub issuer_url: String,
    pub scope: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    key: String,
}

/// Contains all the state information needed for the OAuth authorization flow
#[derive(Debug)]
pub struct AuthFlowState {
    /// The authorization URL the user needs to visit
    auth_url: String,
    /// PKCE code verifier used to validate the code exchange
    pkce_verifier: PkceCodeVerifier,
    /// CSRF token used to prevent cross-site request forgery attacks
    csrf_token: CsrfToken,
}

impl AuthFlowState {
    pub fn url(&self) -> &str {
        &self.auth_url
    }
}

/// The main authentication client
#[derive(Clone)]
pub struct ClerkAuthClient {
    config: ClerkConfig,
    client: CoreClient,
    user_info_client: UserInfoClient,
    key_url: String,
}

impl ClerkAuthClient {
    /// Create a new client with the given configuration
    pub fn new(config: ClerkConfig, key_url: String) -> Result<Self> {
        // Set up the OpenID Connect client
        let client_id = ClientId::new(config.client_id.clone());
        let redirect_url = RedirectUrl::new(config.redirect_url.clone())
            .map_err(|e| AuthError::ConfigError(format!("Invalid redirect URL: {}", e)))?;

        // We'll create a client manually since we don't have full OpenID discovery
        let issuer_url = IssuerUrl::new(config.issuer_url.clone())
            .map_err(|e| AuthError::ConfigError(format!("Invalid issuer URL: {}", e)))?;

        let auth_url = AuthUrl::new(config.auth_url.clone())
            .map_err(|e| AuthError::ConfigError(format!("Invalid auth URL: {}", e)))?;
        let token_url = TokenUrl::new(config.token_url.clone())
            .map_err(|e| AuthError::ConfigError(format!("Invalid token URL: {}", e)))?;

        // Create the client
        let client = CoreClient::new(
            client_id,
            None, // No client secret for PKCE flow
            issuer_url,
            auth_url,
            Some(token_url),
            None,               // No user info endpoint
            Default::default(), // Default JWKS
        )
        .set_redirect_uri(redirect_url);

        let user_info_client = UserInfoClient::new(config.user_info_url.clone());

        Ok(Self { config, client, user_info_client, key_url })
    }

    /// Generate the authorization URL that the user needs to visit
    pub fn generate_auth_url(&self) -> AuthFlowState {
        // Generate PKCE code verifier and challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate a CSRF token and nonce
        let csrf_token = CsrfToken::new_random();
        let nonce = Nonce::new_random();

        let csrf_token_clone = csrf_token.clone();
        let nonce_clone = nonce.clone();

        // Generate the authorization URL
        let (auth_url, _, _) = self
            .client
            .authorize_url(
                AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
                move || csrf_token_clone.clone(),
                move || nonce_clone.clone(),
            )
            .add_scope(Scope::new(self.config.scope.clone()))
            .set_pkce_challenge(pkce_challenge)
            .url();

        let auth_url = auth_url.to_string();

        AuthFlowState { auth_url, pkce_verifier, csrf_token }
    }

    pub async fn complete_auth_flow(&self, auth_state: AuthFlowState) -> Result<()> {
        // Open browser for user authentication
        if let Err(e) = open::that(&auth_state.auth_url) {
            anyhow::bail!("Failed to open browser: {}", e);
        }

        // Start the callback server and wait for the response
        let callback_server = CallbackServer::default();
        // Get both the result and server handle
        let (callback_result, server_handle) = callback_server
            .wait_for_callback_with_handle(120)
            .await
            .map_err(AuthError::from)?;

        // Verify the state to prevent CSRF attacks
        if callback_result.state != *auth_state.csrf_token.secret() {
            // Explicitly shut down the server before returning error
            server_handle.shutdown().await;
            return Err(AuthError::StateMismatch.into());
        }

        // Exchange the code for a token
        let token_response = self
            .exchange_code_for_token(callback_result.code, auth_state.pkce_verifier)
            .await?;

        // Immediately shut down the server after we have the token
        // This is critical to ensure we don't leave servers running
        server_handle.shutdown().await;

        // Get user information
        let user_info = self
            .user_info_client
            .get_user_info(token_response.access_token())
            .await?;

        let token = token_response
            .id_token()
            .ok_or(AuthError::InvalidIDToken)?
            .to_string();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        let body = json!({
            "name": format!("{}-{}", user_info.name.clone().unwrap_or_default(), user_info.email.clone().unwrap_or_default()),
        });
        // Create a new key
        let client = reqwest::Client::new();
        let key: Key = client
            .put(self.key_url.clone())
            .headers(headers)
            .json(&body)
            .send()
            .await
            .context("Failed to send request to antinomy")?
            .error_for_status()?
            .json()
            .await
            .context("Failed to parse antinomy response")?;

        // Save the key to the keychain
        self.save_key_to_keychain(&key.key)?;

        Ok(())
    }

    /// Exchange the authorization code for an access token
    async fn exchange_code_for_token(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<openidconnect::core::CoreTokenResponse> {
        // Clone the client before moving it into the spawn_blocking task
        let client = self.client.clone();

        // Use a blocking task since the openidconnect library uses blocking requests
        tokio::task::spawn_blocking(move || {
            client
                .exchange_code(openidconnect::AuthorizationCode::new(code))
                .set_pkce_verifier(pkce_verifier)
                .request(openidconnect::reqwest::http_client)
                .map_err(|e| AuthError::TokenExchangeError(e.to_string()).into())
        })
        .await?
    }

    fn save_key_to_keychain(&self, key: &str) -> anyhow::Result<()> {
        // Create a keyring entry for the forge API token
        let keyring = keyring::Entry::new("code-forge", "forge_user")?;

        // Set the password (API token)
        keyring
            .set_password(key)
            .map_err(|e| anyhow::anyhow!("Failed to store token in secure storage: {}", e))?;

        Ok(())
    }

    /// Get the key from secure storage if it exists
    pub fn get_key_from_keychain(&self) -> Option<String> {
        // Create a keyring entry for the forge API token
        let keyring = match keyring::Entry::new("code-forge", "forge_user") {
            Ok(keyring) => keyring,
            Err(_) => return None,
        };

        // Try to get the password (API token)
        keyring.get_password().ok()
    }

    /// Delete the key from secure storage if it exists
    pub fn delete_key_from_keychain(&self) -> anyhow::Result<bool> {
        // Create a keyring entry for the forge API token
        let keyring = keyring::Entry::new("code-forge", "forge_user")?;

        // Check if we have a token first
        if keyring.get_password().is_ok() {
            // Try to delete the password (API token)
            keyring.delete_password().map_err(|e| {
                anyhow::anyhow!("Failed to delete token from secure storage: {}", e)
            })?;
            Ok(true)
        } else {
            // No token found, nothing to delete
            Ok(false)
        }
    }
}
