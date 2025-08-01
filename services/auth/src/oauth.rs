//! OAuth2 integration for Google and Apple providers

use anyhow::Result;
use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl,
    RevocationUrl, Scope, TokenResponse, TokenUrl, basic::BasicClient,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;
use uuid::Uuid;

use crate::{AppState, jwt::Claims, models::User};

/// OAuth2 provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OAuthProvider {
    Google,
    Apple,
}

impl OAuthProvider {
    /// Get the provider name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::Apple => "apple",
        }
    }
}

/// OAuth2 configuration for a provider
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub auth_url: String,
    pub token_url: String,
    pub revocation_url: Option<String>,
}

/// OAuth2 client wrapper
#[derive(Clone)]
pub struct OAuthClient {
    provider: OAuthProvider,
    client: BasicClient,
    config: OAuthConfig,
}

impl OAuthClient {
    /// Create a new OAuth2 client for Google
    pub fn new_google(config: OAuthConfig) -> Result<Self> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        );

        Ok(Self {
            provider: OAuthProvider::Google,
            client,
            config,
        })
    }

    /// Create a new OAuth2 client for Apple
    pub fn new_apple(config: OAuthConfig) -> Result<Self> {
        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new(config.auth_url.clone())?,
            Some(TokenUrl::new(config.token_url.clone())?),
        );

        Ok(Self {
            provider: OAuthProvider::Apple,
            client,
            config,
        })
    }

    /// Generate authorization URL with PKCE
    pub fn generate_auth_url(
        &self,
        scopes: &[&str],
    ) -> Result<(String, CsrfToken, PkceCodeVerifier)> {
        info!("Generating authorization URL for {:?}", self.provider);

        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Build the authorization request
        let mut request = self
            .client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);

        // Add scopes
        for scope in scopes {
            request = request.add_scope(Scope::new(scope.to_string()));
        }

        let (auth_url, csrf_token) = request.url();

        Ok((auth_url.to_string(), csrf_token, pkce_verifier))
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(
        &self,
        code: String,
        pkce_verifier: PkceCodeVerifier,
    ) -> Result<
        oauth2::StandardTokenResponse<oauth2::EmptyExtraTokenFields, oauth2::basic::BasicTokenType>,
    > {
        info!(
            "Exchanging authorization code for access token for {:?}",
            self.provider
        );

        let token_response = self
            .client
            .exchange_code(oauth2::AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await?;

        Ok(token_response)
    }

    /// Get user profile information from the provider
    pub async fn get_user_profile(&self, access_token: &str) -> Result<OAuthUserProfile> {
        info!("Getting user profile for {:?}", self.provider);

        match self.provider {
            OAuthProvider::Google => self.get_google_user_profile(access_token).await,
            OAuthProvider::Apple => self.get_apple_user_profile(access_token).await,
        }
    }

    /// Get Google user profile
    async fn get_google_user_profile(&self, access_token: &str) -> Result<OAuthUserProfile> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await?;

        if response.status().is_success() {
            let google_user: GoogleUser = response.json().await?;
            Ok(OAuthUserProfile {
                id: google_user.id,
                email: google_user.email,
                name: Some(format!(
                    "{} {}",
                    google_user.given_name, google_user.family_name
                )),
                verified_email: google_user.verified_email,
                provider: OAuthProvider::Google,
            })
        } else {
            Err(anyhow::anyhow!(
                "Failed to get Google user profile: {}",
                response.status()
            ))
        }
    }

    /// Get Apple user profile
    async fn get_apple_user_profile(&self, access_token: &str) -> Result<OAuthUserProfile> {
        // Apple doesn't provide a user info endpoint, so we'll need to extract
        // user information from the ID token or the initial authorization response
        // For now, we'll return a basic profile with just the token info
        Ok(OAuthUserProfile {
            id: access_token.to_string(),                // This is a placeholder
            email: "apple_user@example.com".to_string(), // This would come from the authorization response
            name: None,
            verified_email: true,
            provider: OAuthProvider::Apple,
        })
    }

    /// Get the provider
    pub fn provider(&self) -> &OAuthProvider {
        &self.provider
    }
}

/// Google user profile response
#[derive(Debug, Deserialize)]
struct GoogleUser {
    id: String,
    email: String,
    verified_email: bool,
    given_name: String,
    family_name: String,
}

/// OAuth user profile information
#[derive(Debug, Clone)]
pub struct OAuthUserProfile {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub verified_email: bool,
    pub provider: OAuthProvider,
}

/// OAuth session data stored in Redis
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthSession {
    pub csrf_token: String,
    pub pkce_verifier: String,
    pub provider: OAuthProvider,
    pub created_at: u64,
}

impl OAuthSession {
    /// Create a new OAuth session
    pub fn new(
        csrf_token: String,
        pkce_verifier: String,
        provider: OAuthProvider,
        created_at: u64,
    ) -> Self {
        Self {
            csrf_token,
            pkce_verifier,
            provider,
            created_at,
        }
    }
}
