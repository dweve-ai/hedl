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

//! JWT (JSON Web Token) authentication implementation.

use crate::auth::{AuthError, ClientMetadata};
use chrono::Utc;
use dashmap::DashMap;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

/// JWT claims structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (client/user ID).
    pub sub: String,

    /// Issuer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,

    /// Audience.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,

    /// Expiration time (Unix timestamp).
    pub exp: i64,

    /// Not before time (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<i64>,

    /// Issued at time (Unix timestamp).
    pub iat: i64,

    /// Custom: scopes/permissions.
    #[serde(default)]
    pub scopes: Vec<String>,

    /// Custom: client metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl From<JwtClaims> for ClientMetadata {
    fn from(claims: JwtClaims) -> Self {
        Self {
            client_id: claims.sub,
            scopes: claims.scopes,
            rate_limit: None,
            created_at: Utc::now(),
            // Handle invalid timestamps gracefully by falling back to current time + 1 hour
            expires_at: chrono::DateTime::from_timestamp(claims.exp, 0)
                .or_else(|| Some(Utc::now() + chrono::Duration::hours(1))),
            metadata: claims.metadata,
        }
    }
}

/// Token validation cache entry.
struct CachedValidation {
    client_metadata: ClientMetadata,
    validated_at: Instant,
    expires_at: Option<Instant>,
}

/// Token validation cache.
pub struct TokenValidationCache {
    cache: Arc<DashMap<String, CachedValidation>>,
    ttl_seconds: u64,
}

impl TokenValidationCache {
    /// Create a new token validation cache.
    ///
    /// # Arguments
    ///
    /// * `ttl_seconds` - Time-to-live for cache entries in seconds
    #[must_use]
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            ttl_seconds,
        }
    }

    /// Get a cached validation result.
    ///
    /// # Arguments
    ///
    /// * `token_hash` - Hash of the token
    ///
    /// # Returns
    ///
    /// Cached client metadata if valid and not expired.
    #[must_use]
    pub fn get(&self, token_hash: &str) -> Option<ClientMetadata> {
        let entry = self.cache.get(token_hash)?;

        // Check if cache entry expired
        if entry.validated_at.elapsed().as_secs() > self.ttl_seconds {
            self.cache.remove(token_hash);
            return None;
        }

        // Check if token itself expired
        if let Some(exp) = entry.expires_at {
            if Instant::now() > exp {
                self.cache.remove(token_hash);
                return None;
            }
        }

        Some(entry.client_metadata.clone())
    }

    /// Insert a validation result into the cache.
    ///
    /// # Arguments
    ///
    /// * `token_hash` - Hash of the token
    /// * `metadata` - Client metadata to cache
    /// * `expires_at` - Optional token expiration time
    pub fn insert(
        &self,
        token_hash: String,
        metadata: ClientMetadata,
        expires_at: Option<Instant>,
    ) {
        self.cache.insert(
            token_hash,
            CachedValidation {
                client_metadata: metadata,
                validated_at: Instant::now(),
                expires_at,
            },
        );
    }

    /// Clear expired entries from the cache.
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.cache.retain(|_, entry| {
            let cache_valid = entry.validated_at.elapsed().as_secs() <= self.ttl_seconds;
            let token_valid = entry.expires_at.map_or(true, |exp| now < exp);
            cache_valid && token_valid
        });
    }
}

/// JWT authentication configuration.
pub struct JwtAuthConfig {
    /// Clock skew tolerance for exp/nbf validation (in seconds).
    pub leeway_seconds: i64,

    /// Issuer validation (iss claim).
    pub issuer: Option<String>,

    /// Audience validation (aud claim).
    pub audience: Option<String>,

    /// Cache TTL for token validation (in seconds).
    pub cache_ttl_seconds: u64,
}

impl Default for JwtAuthConfig {
    fn default() -> Self {
        Self {
            leeway_seconds: 60,
            issuer: None,
            audience: None,
            cache_ttl_seconds: 300,
        }
    }
}

/// JWT authentication using HMAC-SHA256 (HS256).
pub struct JwtAuth {
    /// Secret key for HMAC-based signing.
    encoding_key: EncodingKey,

    /// Decoding key (same as encoding for HMAC).
    decoding_key: DecodingKey,

    /// Configuration.
    config: JwtAuthConfig,

    /// Token validation cache.
    cache: Option<TokenValidationCache>,
}

impl JwtAuth {
    /// Create a new JWT auth handler with HS256.
    ///
    /// # Arguments
    ///
    /// * `secret` - Secret key for HMAC signing (base64url encoded or raw)
    /// * `config` - Optional configuration
    #[must_use]
    pub fn new_with_secret(secret: &str, config: Option<JwtAuthConfig>) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        let config = config.unwrap_or_default();

        let cache = if config.cache_ttl_seconds > 0 {
            Some(TokenValidationCache::new(config.cache_ttl_seconds))
        } else {
            None
        };

        Self {
            encoding_key,
            decoding_key,
            config,
            cache,
        }
    }

    /// Authenticate a JWT token.
    ///
    /// # Arguments
    ///
    /// * `token` - The JWT token string
    ///
    /// # Returns
    ///
    /// Client metadata if authentication succeeds.
    pub fn authenticate(&self, token: &str) -> Result<ClientMetadata, AuthError> {
        // Check cache first
        let token_hash = self.hash_token(token);
        if let Some(cache) = &self.cache {
            if let Some(metadata) = cache.get(&token_hash) {
                return Ok(metadata);
            }
        }

        // Decode and validate token
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.leeway = self.config.leeway_seconds as u64;

        if let Some(ref iss) = self.config.issuer {
            validation.set_issuer(&[iss]);
        }

        if let Some(ref aud) = self.config.audience {
            validation.set_audience(&[aud]);
        }

        let token_data =
            decode::<JwtClaims>(token, &self.decoding_key, &validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                        AuthError::InvalidSignature
                    }
                    _ => AuthError::InvalidTokenFormat(e.to_string()),
                }
            })?;

        let claims = token_data.claims;
        let metadata: ClientMetadata = claims.clone().into();

        // Cache the result
        if let Some(cache) = &self.cache {
            let expires_at = metadata.expires_at.map(|dt| {
                let timestamp = dt.timestamp();
                std::time::Instant::now()
                    + std::time::Duration::from_secs(
                        (timestamp.saturating_sub(Utc::now().timestamp())) as u64,
                    )
            });

            cache.insert(token_hash, metadata.clone(), expires_at);
        }

        Ok(metadata)
    }

    /// Create a new JWT token.
    ///
    /// # Arguments
    ///
    /// * `claims` - JWT claims to encode
    ///
    /// # Returns
    ///
    /// Encoded JWT token string.
    pub fn create_token(&self, claims: &JwtClaims) -> Result<String, AuthError> {
        encode(&Header::default(), claims, &self.encoding_key).map_err(AuthError::Jwt)
    }

    /// Hash a token for caching.
    fn hash_token(&self, token: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        token.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get cache statistics.
    #[must_use]
    pub fn cache_size(&self) -> usize {
        self.cache.as_ref().map_or(0, |c| c.cache.len())
    }

    /// Cleanup expired cache entries.
    pub fn cleanup_cache(&self) {
        if let Some(cache) = &self.cache {
            cache.cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_jwt_create_and_verify() {
        let auth = JwtAuth::new_with_secret("test_secret", None);

        let claims = JwtClaims {
            sub: "user-123".to_string(),
            iss: None,
            aud: None,
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            nbf: Some(Utc::now().timestamp()),
            iat: Utc::now().timestamp(),
            scopes: vec!["read".to_string(), "write".to_string()],
            metadata: None,
        };

        let token = auth.create_token(&claims).unwrap();
        let metadata = auth.authenticate(&token).unwrap();

        assert_eq!(metadata.client_id, "user-123");
        assert_eq!(metadata.scopes, vec!["read", "write"]);
    }

    #[test]
    fn test_jwt_expired_token() {
        let auth = JwtAuth::new_with_secret("test_secret", None);

        let claims = JwtClaims {
            sub: "user-123".to_string(),
            iss: None,
            aud: None,
            exp: (Utc::now() - Duration::hours(1)).timestamp(),
            nbf: None,
            iat: Utc::now().timestamp(),
            scopes: vec![],
            metadata: None,
        };

        let token = auth.create_token(&claims).unwrap();
        let result = auth.authenticate(&token);

        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[test]
    fn test_jwt_invalid_signature() {
        let auth1 = JwtAuth::new_with_secret("secret1", None);
        let auth2 = JwtAuth::new_with_secret("secret2", None);

        let claims = JwtClaims {
            sub: "user-123".to_string(),
            iss: None,
            aud: None,
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            nbf: None,
            iat: Utc::now().timestamp(),
            scopes: vec![],
            metadata: None,
        };

        let token = auth1.create_token(&claims).unwrap();
        let result = auth2.authenticate(&token);

        assert!(matches!(result, Err(AuthError::InvalidSignature)));
    }

    #[test]
    fn test_jwt_cache_hit() {
        let config = JwtAuthConfig {
            cache_ttl_seconds: 300,
            ..Default::default()
        };
        let auth = JwtAuth::new_with_secret("test_secret", Some(config));

        let claims = JwtClaims {
            sub: "user-123".to_string(),
            iss: None,
            aud: None,
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            nbf: None,
            iat: Utc::now().timestamp(),
            scopes: vec![],
            metadata: None,
        };

        let token = auth.create_token(&claims).unwrap();

        // First call
        auth.authenticate(&token).unwrap();
        assert_eq!(auth.cache_size(), 1);

        // Second call should hit cache
        auth.authenticate(&token).unwrap();
        assert_eq!(auth.cache_size(), 1);
    }

    #[test]
    fn test_jwt_issuer_validation() {
        let config = JwtAuthConfig {
            issuer: Some("hedl-auth".to_string()),
            ..Default::default()
        };
        let auth = JwtAuth::new_with_secret("test_secret", Some(config));

        let claims = JwtClaims {
            sub: "user-123".to_string(),
            iss: Some("wrong-issuer".to_string()),
            aud: None,
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            nbf: None,
            iat: Utc::now().timestamp(),
            scopes: vec![],
            metadata: None,
        };

        let token = auth.create_token(&claims).unwrap();
        let result = auth.authenticate(&token);

        assert!(matches!(result, Err(AuthError::InvalidTokenFormat(_))));
    }
}
