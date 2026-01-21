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

//! Core authentication types.

use crate::auth::AuthError;
use chrono::{DateTime, Utc};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Client metadata extracted from authentication credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMetadata {
    /// Unique client identifier.
    pub client_id: String,

    /// Scopes/permissions granted to this client.
    pub scopes: Vec<String>,

    /// Optional rate limit configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimit>,

    /// When this client was created.
    pub created_at: DateTime<Utc>,

    /// When this client's credentials expire (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    /// Optional metadata for custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl Default for ClientMetadata {
    fn default() -> Self {
        Self {
            client_id: String::new(),
            scopes: Vec::new(),
            rate_limit: None,
            created_at: Utc::now(),
            expires_at: None,
            metadata: None,
        }
    }
}

/// Rate limit configuration for a client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum requests per time window.
    pub max_requests: u32,

    /// Time window in seconds.
    pub window_seconds: u32,
}

/// Secure string wrapper that zeroizes memory on drop.
pub struct SecretString(secrecy::SecretBox<String>);

impl Clone for SecretString {
    fn clone(&self) -> Self {
        Self::new(self.0.expose_secret().clone())
    }
}

impl SecretString {
    /// Create a new secret string.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(secrecy::SecretBox::new(Box::new(value)))
    }

    /// Expose the secret value (use with caution).
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }

    /// Create from a string slice.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn from_str(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl<'de> Deserialize<'de> for SecretString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecretString")
            .field("value", &"***REDACTED***")
            .finish()
    }
}

/// Unique session identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Generate a new random session ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from a string.
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    /// Get the underlying string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Authentication scheme types supported by the MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationScheme {
    /// No authentication (development mode).
    None,

    /// API key authentication.
    ApiKey,

    /// JWT (JSON Web Token) authentication.
    Jwt,

    /// OAuth 2.0 authentication.
    OAuth2,

    /// Mutual TLS authentication.
    Mtls,
}

impl AuthenticationScheme {
    /// Parse from string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, AuthError> {
        match s.to_lowercase().as_str() {
            "none" | "disabled" => Ok(Self::None),
            "api_key" | "apikey" => Ok(Self::ApiKey),
            "jwt" => Ok(Self::Jwt),
            "oauth2" | "oauth" => Ok(Self::OAuth2),
            "mtls" | "tls" => Ok(Self::Mtls),
            _ => Err(AuthError::UnsupportedScheme(s.to_string())),
        }
    }

    /// Convert to string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::ApiKey => "api_key",
            Self::Jwt => "jwt",
            Self::OAuth2 => "oauth2",
            Self::Mtls => "mtls",
        }
    }
}

impl fmt::Display for AuthenticationScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_string_debug() {
        let secret = SecretString::new("password123".to_string());
        let debug_str = format!("{secret:?}");
        assert!(!debug_str.contains("password123"));
        assert!(debug_str.contains("REDACTED"));
    }

    #[test]
    fn test_secret_string_expose() {
        let secret = SecretString::new("password123".to_string());
        assert_eq!(secret.expose_secret(), "password123");
    }

    #[test]
    fn test_session_id_unique() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_session_id_from_str() {
        let id = SessionId::from_str("test-session");
        assert_eq!(id.as_str(), "test-session");
    }

    #[test]
    fn test_authentication_scheme_parse() {
        assert_eq!(
            AuthenticationScheme::from_str("api_key").unwrap(),
            AuthenticationScheme::ApiKey
        );
        assert_eq!(
            AuthenticationScheme::from_str("JWT").unwrap(),
            AuthenticationScheme::Jwt
        );
        assert!(matches!(
            AuthenticationScheme::from_str("invalid"),
            Err(AuthError::UnsupportedScheme(_))
        ));
    }

    #[test]
    fn test_client_metadata_default() {
        let meta = ClientMetadata::default();
        assert!(meta.client_id.is_empty());
        assert!(meta.scopes.is_empty());
        assert!(meta.expires_at.is_none());
    }
}
