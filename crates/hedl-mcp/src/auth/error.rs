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

//! Authentication and authorization error types.

use thiserror::Error;

/// Authentication and authorization error type.
#[derive(Error, Debug)]
pub enum AuthError {
    /// Invalid credentials provided.
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// Token has expired.
    #[error("Token expired")]
    TokenExpired,

    /// Token signature is invalid.
    #[error("Invalid token signature")]
    InvalidSignature,

    /// Token is malformed or missing required fields.
    #[error("Invalid token format: {0}")]
    InvalidTokenFormat(String),

    /// Authentication scheme not supported.
    #[error("Unsupported authentication scheme: {0}")]
    UnsupportedScheme(String),

    /// Authorization failed - forbidden.
    #[error("Access forbidden")]
    Forbidden,

    /// Session not found or expired.
    #[error("Session not found or expired")]
    SessionNotFound,

    /// Session expired due to timeout.
    #[error("Session expired")]
    SessionExpired,

    /// Session idle timeout exceeded.
    #[error("Session idle timeout exceeded")]
    SessionIdle,

    /// Session request limit exceeded.
    #[error("Session request limit exceeded")]
    SessionRequestLimitExceeded,

    /// Rate limit exceeded.
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Encryption operation failed.
    #[error("Encryption failed")]
    EncryptionFailed,

    /// Decryption operation failed.
    #[error("Decryption failed")]
    DecryptionFailed,

    /// Invalid ciphertext.
    #[error("Invalid ciphertext")]
    InvalidCiphertext,

    /// Password hashing failed.
    #[error("Hashing failed")]
    HashingFailed,

    /// Invalid password hash format.
    #[error("Invalid hash format")]
    InvalidHash,

    /// IO error during credential storage operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Base64 encoding/decoding error.
    #[error("Base64 error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// JWT encoding/decoding error.
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    /// `OAuth2` error.
    #[error("OAuth2 error: {0}")]
    OAuth2(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for authentication operations.
pub type AuthResult<T> = Result<T, AuthError>;

impl AuthError {
    /// Get the MCP error code for this error.
    #[must_use]
    pub fn code(&self) -> i32 {
        match self {
            Self::InvalidCredentials => -32401,
            Self::TokenExpired => -32402,
            Self::InvalidSignature => -32403,
            Self::InvalidTokenFormat(_) => -32404,
            Self::UnsupportedScheme(_) => -32405,
            Self::Forbidden => -32403,
            Self::SessionNotFound => -32406,
            Self::SessionExpired => -32407,
            Self::SessionIdle => -32408,
            Self::SessionRequestLimitExceeded => -32409,
            Self::RateLimitExceeded => -32410,
            Self::EncryptionFailed
            | Self::DecryptionFailed
            | Self::InvalidCiphertext
            | Self::HashingFailed
            | Self::InvalidHash => -32411,
            Self::Io(_) => -32002,
            Self::Json(_) => -32700,
            Self::Base64(_) => -32412,
            Self::Jwt(_) => -32413,
            Self::OAuth2(_) => -32414,
            Self::Configuration(_) => -32415,
            Self::Internal(_) => -32603,
        }
    }

    /// Check if this error should be logged as a security event.
    #[must_use]
    pub fn is_security_event(&self) -> bool {
        matches!(
            self,
            Self::InvalidCredentials
                | Self::Forbidden
                | Self::InvalidSignature
                | Self::RateLimitExceeded
        )
    }
}
