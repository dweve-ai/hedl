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

//! Authorization system for fine-grained access control.

use crate::auth::{AuthError, AuthResult, ClientMetadata};
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Action types for authorization.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    /// Read access.
    Read,

    /// Write access.
    Write,

    /// Execute tool.
    Execute,

    /// Validate.
    Validate,

    /// Convert.
    Convert,

    /// Delete.
    Delete,
}

impl std::str::FromStr for Action {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "read" => Ok(Self::Read),
            "write" => Ok(Self::Write),
            "execute" => Ok(Self::Execute),
            "validate" => Ok(Self::Validate),
            "convert" => Ok(Self::Convert),
            "delete" => Ok(Self::Delete),
            _ => Err(AuthError::Configuration(format!("Invalid action: {s}"))),
        }
    }
}

/// Resource types for authorization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthResource {
    /// All resources.
    All,

    /// Specific tool.
    Tool {
        /// Tool name.
        name: String,
    },

    /// File path pattern.
    Path {
        /// Glob pattern for path matching.
        pattern: String,
    },

    /// Entity type.
    EntityType {
        /// Entity type name.
        name: String,
    },
}

impl AuthResource {
    /// Check if this resource matches another.
    #[must_use]
    pub fn matches(&self, other: &AuthResource) -> bool {
        match (self, other) {
            (AuthResource::All, _) => true,
            (AuthResource::Tool { name: a }, AuthResource::Tool { name: b }) => a == b,
            (AuthResource::Path { pattern: a }, AuthResource::Path { pattern: b }) => {
                // Use glob pattern matching
                if let Ok(pattern) = Pattern::new(a) {
                    pattern.matches(b)
                } else {
                    a == b
                }
            }
            (AuthResource::EntityType { name: a }, AuthResource::EntityType { name: b }) => a == b,
            _ => false,
        }
    }

    /// Create a tool resource.
    #[must_use]
    pub fn tool(name: &str) -> Self {
        Self::Tool {
            name: name.to_string(),
        }
    }

    /// Create a path resource.
    #[must_use]
    pub fn path(pattern: &str) -> Self {
        Self::Path {
            pattern: pattern.to_string(),
        }
    }
}

/// Subject matcher for policy rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SubjectMatcher {
    /// Match by exact client ID.
    ClientId {
        /// Exact client ID to match.
        id: String,
    },

    /// Match by client ID pattern (glob).
    ClientPattern {
        /// Glob pattern for client ID matching.
        pattern: String,
    },

    /// Match by scope.
    Scope {
        /// Scope name to match.
        scope: String,
    },

    /// Match by group.
    Group {
        /// Group name to match.
        name: String,
    },
}

impl SubjectMatcher {
    /// Check if this subject matches a client.
    #[must_use]
    pub fn matches(&self, client: &ClientMetadata) -> bool {
        match self {
            SubjectMatcher::ClientId { id } => &client.client_id == id,
            SubjectMatcher::ClientPattern { pattern } => {
                if let Ok(pattern) = Pattern::new(pattern) {
                    pattern.matches(&client.client_id)
                } else {
                    false
                }
            }
            SubjectMatcher::Scope { scope } => client.scopes.iter().any(|s| s == scope),
            SubjectMatcher::Group { name } => {
                // Check metadata for group membership
                client
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("groups"))
                    .and_then(|g| g.as_array())
                    .is_some_and(|groups| groups.iter().any(|g| g.as_str() == Some(name)))
            }
        }
    }
}

/// Condition for policy rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Condition {
    /// Time-based access control.
    TimeWindow {
        /// Start time (HH:MM:SS format).
        start: String,
        /// End time (HH:MM:SS format).
        end: String,
    },

    /// IP whitelist.
    IpWhitelist {
        /// Allowed IP addresses.
        ips: Vec<String>,
    },

    /// Rate limit.
    RateLimit {
        /// Maximum requests allowed.
        requests: u32,
        /// Time window in seconds.
        window_seconds: u32,
    },

    /// Custom condition.
    Custom {
        /// Custom condition name.
        name: String,
        /// Custom condition configuration.
        config: serde_json::Value,
    },
}

impl Condition {
    /// Check if this condition is satisfied.
    pub fn check(&self) -> AuthResult<()> {
        match self {
            Condition::TimeWindow { start, end } => {
                let now = chrono::Utc::now().time();
                let start_time = chrono::NaiveTime::parse_from_str(start, "%H:%M:%S")
                    .map_err(|_| AuthError::Configuration("Invalid time format".to_string()))?;
                let end_time = chrono::NaiveTime::parse_from_str(end, "%H:%M:%S")
                    .map_err(|_| AuthError::Configuration("Invalid time format".to_string()))?;

                if now < start_time || now > end_time {
                    return Err(AuthError::Forbidden);
                }
                Ok(())
            }
            Condition::IpWhitelist { .. } => {
                // IP-based conditions are checked at the network layer
                Ok(())
            }
            Condition::RateLimit { .. } => {
                // Rate limiting is handled separately
                Ok(())
            }
            Condition::Custom { .. } => {
                // Custom conditions are evaluated by the application
                Ok(())
            }
        }
    }
}

/// Authorization policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Subject matcher.
    pub subject: SubjectMatcher,

    /// Resource matcher.
    pub resource: AuthResource,

    /// Allowed actions.
    pub actions: Vec<Action>,

    /// Optional conditions.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
}

/// Authorization policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationPolicy {
    /// Policy rules.
    pub rules: Vec<PolicyRule>,

    /// Default policy (allow or deny).
    #[serde(default)]
    pub default_policy: DefaultPolicy,
}

/// Default policy when no rules match.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DefaultPolicy {
    /// Deny by default (secure).
    #[default]
    Deny,

    /// Allow by default (permissive).
    Allow,
}

impl AuthorizationPolicy {
    /// Create a new authorization policy.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default_policy: DefaultPolicy::Deny,
        }
    }

    /// Add a policy rule.
    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
    }

    /// Check if a client is authorized for an action on a resource.
    ///
    /// # Arguments
    ///
    /// * `client` - Client metadata
    /// * `resource` - Resource to access
    /// * `action` - Action to perform
    ///
    /// # Returns
    ///
    /// Ok(()) if authorized, Err otherwise.
    pub fn check(
        &self,
        client: &ClientMetadata,
        resource: &AuthResource,
        action: &Action,
    ) -> AuthResult<()> {
        for rule in &self.rules {
            if rule.subject.matches(client)
                && rule.resource.matches(resource)
                && rule.actions.contains(action)
            {
                // Check conditions
                for condition in &rule.conditions {
                    condition.check()?;
                }
                return Ok(());
            }
        }

        // No matching rule, apply default policy
        match self.default_policy {
            DefaultPolicy::Allow => Ok(()),
            DefaultPolicy::Deny => Err(AuthError::Forbidden),
        }
    }

    /// Check if a client can execute a specific tool.
    pub fn check_tool(&self, client: &ClientMetadata, tool_name: &str) -> AuthResult<()> {
        self.check(client, &AuthResource::tool(tool_name), &Action::Execute)
    }

    /// Check if a client can read a file path.
    pub fn check_path_read(&self, client: &ClientMetadata, path: &Path) -> AuthResult<()> {
        let path_str = path.to_string_lossy().to_string();
        self.check(client, &AuthResource::path(&path_str), &Action::Read)
    }

    /// Check if a client can write to a file path.
    pub fn check_path_write(&self, client: &ClientMetadata, path: &Path) -> AuthResult<()> {
        let path_str = path.to_string_lossy().to_string();
        self.check(client, &AuthResource::path(&path_str), &Action::Write)
    }
}

impl Default for AuthorizationPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission represents a granted capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Resource this permission applies to.
    pub resource: AuthResource,

    /// Actions allowed on this resource.
    pub actions: Vec<Action>,
}

/// Path pattern for file-based authorization.
#[derive(Debug, Clone)]
pub struct PathPattern {
    pattern: glob::Pattern,
}

impl PathPattern {
    /// Create a new path pattern.
    pub fn new(pattern: &str) -> Result<Self, AuthError> {
        Ok(Self {
            pattern: Pattern::new(pattern).map_err(|_| {
                AuthError::Configuration(format!("Invalid glob pattern: {pattern}"))
            })?,
        })
    }

    /// Check if a path matches this pattern.
    #[must_use]
    pub fn matches(&self, path: &Path) -> bool {
        self.pattern.matches_path(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_action_from_str() {
        assert_eq!(Action::from_str("read").unwrap(), Action::Read);
        assert_eq!(Action::from_str("WRITE").unwrap(), Action::Write);
        assert!(Action::from_str("invalid").is_err());
    }

    #[test]
    fn test_resource_matching() {
        let all = AuthResource::All;
        let tool_read = AuthResource::tool("hedl_read");
        let tool_write = AuthResource::tool("hedl_write");

        assert!(all.matches(&tool_read));
        assert!(tool_read.matches(&AuthResource::tool("hedl_read")));
        assert!(!tool_read.matches(&tool_write));
    }

    #[test]
    fn test_subject_matching() {
        let client = ClientMetadata {
            client_id: "admin-client".to_string(),
            scopes: vec!["admin".to_string(), "read".to_string()],
            ..Default::default()
        };

        let id_matcher = SubjectMatcher::ClientId {
            id: "admin-client".to_string(),
        };
        assert!(id_matcher.matches(&client));

        let scope_matcher = SubjectMatcher::Scope {
            scope: "admin".to_string(),
        };
        assert!(scope_matcher.matches(&client));

        let pattern_matcher = SubjectMatcher::ClientPattern {
            pattern: "admin-*".to_string(),
        };
        assert!(pattern_matcher.matches(&client));
    }

    #[test]
    fn test_authorization_policy_allow() {
        let mut policy = AuthorizationPolicy::new();

        policy.add_rule(PolicyRule {
            subject: SubjectMatcher::Scope {
                scope: "admin".to_string(),
            },
            resource: AuthResource::All,
            actions: vec![Action::Read, Action::Write, Action::Execute],
            conditions: vec![],
        });

        let client = ClientMetadata {
            client_id: "test-client".to_string(),
            scopes: vec!["admin".to_string()],
            ..Default::default()
        };

        assert!(policy
            .check(&client, &AuthResource::tool("hedl_read"), &Action::Execute)
            .is_ok());
    }

    #[test]
    fn test_authorization_policy_deny() {
        let policy = AuthorizationPolicy::new();

        let client = ClientMetadata {
            client_id: "test-client".to_string(),
            scopes: vec![],
            ..Default::default()
        };

        let result = policy.check(&client, &AuthResource::tool("hedl_read"), &Action::Execute);
        assert!(matches!(result, Err(AuthError::Forbidden)));
    }

    #[test]
    fn test_authorization_policy_default_allow() {
        let mut policy = AuthorizationPolicy::new();
        policy.default_policy = DefaultPolicy::Allow;

        let client = ClientMetadata {
            client_id: "test-client".to_string(),
            scopes: vec![],
            ..Default::default()
        };

        assert!(policy
            .check(&client, &AuthResource::tool("hedl_read"), &Action::Execute)
            .is_ok());
    }

    #[test]
    fn test_path_pattern() {
        let pattern = PathPattern::new("**/*.hedl").unwrap();

        assert!(pattern.matches(Path::new("test.hedl")));
        assert!(pattern.matches(Path::new("data/test.hedl")));
        assert!(!pattern.matches(Path::new("test.txt")));
    }

    #[test]
    fn test_condition_time_window() {
        let condition = Condition::TimeWindow {
            start: "00:00:00".to_string(),
            end: "23:59:59".to_string(),
        };

        // Current time should be within this window
        assert!(condition.check().is_ok());
    }
}
