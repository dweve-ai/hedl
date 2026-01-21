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

//! Authentication and authorization module for HEDL MCP server.
//!
//! This module provides enterprise-grade security features including:
//!
//! - Multiple authentication schemes (API Key, JWT, `OAuth2`)
//! - Secure credential storage with encryption
//! - Session management with expiration
//! - Fine-grained authorization policies
//! - Audit logging for security events

mod crypto;
mod error;
mod types;

pub mod api_key;
pub mod authorization;
pub mod jwt;
pub mod oauth2;
pub mod session;

pub use crypto::{secure_write, ApiKeyHasher, CredentialStore};
pub use error::{AuthError, AuthResult};
pub use types::{AuthenticationScheme, ClientMetadata, RateLimit, SecretString, SessionId};

// Re-export authentication traits
pub use api_key::{ApiKeyAuth, ApiKeyInfo, ApiKeyStore, InMemoryApiKeyStore};
pub use authorization::{
    Action, AuthResource, AuthorizationPolicy, Condition, PathPattern, Permission, PolicyRule,
    SubjectMatcher,
};
pub use jwt::{JwtAuth, JwtClaims};
pub use oauth2::{OAuth2Auth, OAuth2Provider};
pub use session::{Session, SessionConfig, SessionManager};
