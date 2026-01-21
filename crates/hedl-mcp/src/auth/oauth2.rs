// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! OAuth 2.0 authentication implementation.

use crate::auth::{AuthError, ClientMetadata};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

/// OAuth 2.0 provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Provider {
    /// Issuer URL.
    pub issuer: String,

    /// Authorization endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_endpoint: Option<String>,

    /// Token endpoint.
    pub token_endpoint: String,

    /// Userinfo endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userinfo_endpoint: Option<String>,

    /// JWKS URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,

    /// Introspection endpoint.
    pub introspection_endpoint: String,
}

/// OAuth 2.0 introspection response.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IntrospectionResponse {
    /// Whether the token is active.
    pub active: bool,

    /// Client ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// Scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,

    /// Expiration time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,

    /// Issued at time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,

    /// Subject.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
}

/// Token validation cache entry.
struct CachedTokenValidation {
    client_metadata: ClientMetadata,
    validated_at: Instant,
    expires_at: Option<Instant>,
}

/// OAuth 2.0 authentication.
pub struct OAuth2Auth {
    /// OAuth 2.0 provider configuration.
    provider: OAuth2Provider,

    /// Client credentials for introspection.
    client_id: String,

    /// Client secret (never exposed in logs/errors).
    client_secret: secrecy::SecretBox<String>,

    /// Introspection cache.
    cache: Arc<DashMap<String, CachedTokenValidation>>,

    /// Cache TTL in seconds.
    cache_ttl_seconds: u64,
}

impl OAuth2Auth {
    /// Create a new OAuth 2.0 authentication handler.
    ///
    /// # Arguments
    ///
    /// * `provider` - OAuth 2.0 provider configuration
    /// * `client_id` - Client ID for introspection
    /// * `client_secret` - Client secret for introspection
    /// * `cache_ttl_seconds` - Cache TTL for introspection results
    #[must_use]
    pub fn new(
        provider: OAuth2Provider,
        client_id: String,
        client_secret: String,
        cache_ttl_seconds: u64,
    ) -> Self {
        Self {
            provider,
            client_id,
            client_secret: secrecy::SecretBox::new(Box::new(client_secret)),
            cache: Arc::new(DashMap::new()),
            cache_ttl_seconds,
        }
    }

    /// Authenticate an OAuth 2.0 access token.
    ///
    /// # Arguments
    ///
    /// * `token` - The access token
    ///
    /// # Returns
    ///
    /// Client metadata if authentication succeeds.
    pub async fn authenticate(&self, token: &str) -> Result<ClientMetadata, AuthError> {
        let token_hash = self.hash_token(token);

        // Check cache first
        if let Some(cached) = self.cache.get(&token_hash) {
            if cached.validated_at.elapsed().as_secs() < self.cache_ttl_seconds {
                if let Some(exp) = cached.expires_at {
                    if Instant::now() < exp {
                        return Ok(cached.client_metadata.clone());
                    }
                } else {
                    return Ok(cached.client_metadata.clone());
                }
            }
        }

        // Perform introspection
        let metadata = self.introspect(token).await?;

        // Cache the result
        let expires_at = metadata.expires_at.map(|dt| {
            let timestamp = dt.timestamp();
            std::time::Instant::now()
                + std::time::Duration::from_secs(
                    timestamp.saturating_sub(Utc::now().timestamp()) as u64
                )
        });

        self.cache.insert(
            token_hash,
            CachedTokenValidation {
                client_metadata: metadata.clone(),
                validated_at: Instant::now(),
                expires_at,
            },
        );

        Ok(metadata)
    }

    /// Perform token introspection.
    async fn introspect(&self, token: &str) -> Result<ClientMetadata, AuthError> {
        let client = reqwest::Client::new();

        let response = client
            .post(&self.provider.introspection_endpoint)
            .basic_auth(&self.client_id, Some(self.client_secret.expose_secret()))
            .form(&[("token", token)])
            .send()
            .await
            .map_err(|e| AuthError::OAuth2(format!("Request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(AuthError::OAuth2(format!(
                "Introspection failed: {}",
                response.status()
            )));
        }

        let introspection: IntrospectionResponse = response
            .json()
            .await
            .map_err(|e| AuthError::OAuth2(format!("Invalid response: {e}")))?;

        if !introspection.active {
            return Err(AuthError::InvalidCredentials);
        }

        let scopes = introspection
            .scope
            .unwrap_or_default()
            .split(' ')
            .map(String::from)
            .filter(|s| !s.is_empty())
            .collect();

        Ok(ClientMetadata {
            client_id: introspection
                .client_id
                .unwrap_or_else(|| introspection.sub.unwrap_or_else(|| "unknown".to_string())),
            scopes,
            rate_limit: None,
            created_at: introspection
                .iat
                .and_then(|ts| DateTime::from_timestamp(ts, 0))
                .unwrap_or_else(Utc::now),
            expires_at: introspection
                .exp
                .and_then(|ts| DateTime::from_timestamp(ts, 0)),
            metadata: None,
        })
    }

    /// Hash a token for caching.
    fn hash_token(&self, token: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Cleanup expired cache entries.
    pub fn cleanup_cache(&self) {
        let now = Instant::now();
        self.cache.retain(|_, entry| {
            let cache_valid = entry.validated_at.elapsed().as_secs() < self.cache_ttl_seconds;
            let token_valid = entry.expires_at.map_or(true, |exp| now < exp);
            cache_valid && token_valid
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth2_provider_deserialize() {
        let json = r#"{
            "issuer": "https://auth.example.com",
            "token_endpoint": "https://auth.example.com/oauth/token",
            "introspection_endpoint": "https://auth.example.com/oauth/introspect"
        }"#;

        let provider: OAuth2Provider = serde_json::from_str(json).unwrap();
        assert_eq!(provider.issuer, "https://auth.example.com");
        assert_eq!(
            provider.token_endpoint,
            "https://auth.example.com/oauth/token"
        );
        assert_eq!(
            provider.introspection_endpoint,
            "https://auth.example.com/oauth/introspect"
        );
    }

    #[test]
    fn test_token_hash_different() {
        let provider = OAuth2Provider {
            issuer: "test".to_string(),
            token_endpoint: "test".to_string(),
            introspection_endpoint: "test".to_string(),
            authorization_endpoint: None,
            userinfo_endpoint: None,
            jwks_uri: None,
        };

        let auth = OAuth2Auth::new(provider, "client".to_string(), "secret".to_string(), 300);

        let hash1 = auth.hash_token("token1");
        let hash2 = auth.hash_token("token2");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_token_hash_same() {
        let provider = OAuth2Provider {
            issuer: "test".to_string(),
            token_endpoint: "test".to_string(),
            introspection_endpoint: "test".to_string(),
            authorization_endpoint: None,
            userinfo_endpoint: None,
            jwks_uri: None,
        };

        let auth = OAuth2Auth::new(provider, "client".to_string(), "secret".to_string(), 300);

        let hash1 = auth.hash_token("token1");
        let hash2 = auth.hash_token("token1");

        assert_eq!(hash1, hash2);
    }
}
