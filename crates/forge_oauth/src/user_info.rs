use anyhow::Result;
use openidconnect::AccessToken;
use serde::Deserialize;

use crate::error::AuthError;

/// User information returned by the OAuth provider
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

/// Client for fetching user information
#[derive(Clone)]
pub struct UserInfoClient {
    user_info_url: String,
}

impl UserInfoClient {
    /// Create a new user info client
    pub fn new(user_info_url: String) -> Self {
        Self { user_info_url }
    }

    /// Retrieve user information using the access token
    pub async fn get_user_info(&self, token: &AccessToken) -> Result<UserInfo> {
        let client = reqwest::Client::new();
        let response = client
            .get(&self.user_info_url)
            .header("Authorization", format!("Bearer {}", token.secret()))
            .send()
            .await
            .map_err(|e| AuthError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".into());
            return Err(AuthError::UserInfoError(error_text).into());
        }

        let user_info: UserInfo = response
            .json()
            .await
            .map_err(|e| AuthError::UserInfoError(e.to_string()))?;

        Ok(user_info)
    }
}
