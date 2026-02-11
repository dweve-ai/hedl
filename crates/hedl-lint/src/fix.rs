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

//! Auto-fix functionality for lint violations
//!
//! This module provides safe, verifiable automatic fixing of lint violations.
//!
//! # Examples
//!
//! ```rust
//! use hedl_lint::{FixApplicator, FixConfig};
//! use hedl_core::parse;
//!
//! let source = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n---\nusers:@User\n |a,Alice\n";
//! let doc = parse(source.as_bytes()).unwrap();
//!
//! // Generate fixes (shown in provider implementations)
//! let fixes = vec![];
//!
//! let applicator = FixApplicator::new(FixConfig::default());
//! let result = applicator.apply_fixes(source, fixes);
//!
//! if let Some(fixed) = result.fixed_source {
//!     println!("Fixed: {}", fixed);
//! }
//! ```

/// Fix application engine.
pub mod applicator;
/// Configuration for fix behavior.
pub mod config;
/// Conflict detection and resolution for fixes.
pub mod conflict;
/// Context for fix generation.
pub mod context;
/// Diff generation for fix previews.
pub mod diff;
/// Error types for fix operations.
pub mod error;
/// Fix ordering for safe sequential application.
pub mod ordering;
/// Preview functionality for fixes.
pub mod preview;
/// Fix provider trait for lint rules.
pub mod provider;
/// Source position and range utilities for fix application.
pub mod range;
/// Statistics tracking for fix operations.
pub mod statistics;
/// Verification of fix safety and correctness.
pub mod verifier;

pub use applicator::FixApplicator;
pub use config::{ConflictStrategy, FixConfig};
pub use conflict::{ConflictDetector, ConflictResolution, ConflictType, FixConflict};
pub use context::FixContext;
pub use diff::DiffGenerator;
pub use error::FixError;
pub use ordering::FixOrderer;
pub use preview::FixPreview;
pub use provider::FixProvider;
pub use range::{SourcePosition, SourceRange};
pub use statistics::FixStatistics;
pub use verifier::FixVerifier;

use crate::diagnostic::Severity;
use uuid::Uuid;

/// Unique identifier for a fix
pub type FixId = Uuid;

/// Represents a single atomic fix for a lint violation
#[derive(Debug, Clone)]
pub struct Fix {
    /// Unique identifier for this fix
    pub id: FixId,
    /// Rule that generated this fix
    pub rule_id: String,
    /// Source range to be replaced
    pub range: SourceRange,
    /// Replacement text
    pub replacement: String,
    /// Human-readable description
    pub description: String,
    /// Severity of violation being fixed
    pub severity: Severity,
    /// Dependencies on other fixes (must be applied first)
    pub dependencies: Vec<FixId>,
    /// Whether this fix is safe to auto-apply
    pub is_safe: bool,
}

impl Fix {
    /// Create a new fix
    pub fn new(
        rule_id: impl Into<String>,
        range: SourceRange,
        replacement: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule_id: rule_id.into(),
            range,
            replacement: replacement.into(),
            description: description.into(),
            severity: Severity::Hint,
            dependencies: Vec::new(),
            is_safe: true,
        }
    }

    /// Set the severity level
    #[must_use]
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Mark as unsafe (requires explicit user approval)
    #[must_use]
    pub fn with_unsafe(mut self) -> Self {
        self.is_safe = false;
        self
    }

    /// Add a dependency on another fix
    #[must_use]
    pub fn with_dependency(mut self, dep: FixId) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Add multiple dependencies
    #[must_use]
    pub fn with_dependencies(mut self, deps: Vec<FixId>) -> Self {
        self.dependencies.extend(deps);
        self
    }
}

/// Result of fix application
#[derive(Debug, Clone)]
pub struct FixResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// The fixed source text (if successful)
    pub fixed_source: Option<String>,
    /// Errors encountered during application
    pub errors: Vec<FixError>,
    /// Conflicts detected
    pub conflicts: Vec<FixConflict>,
    /// Fixes that were successfully applied
    pub applied_fixes: Vec<Fix>,
}

impl FixResult {
    /// Create a successful result
    #[must_use]
    pub fn success(fixed_source: String, applied_fixes: Vec<Fix>) -> Self {
        Self {
            success: true,
            fixed_source: Some(fixed_source),
            errors: Vec::new(),
            conflicts: Vec::new(),
            applied_fixes,
        }
    }

    /// Create an error result
    #[must_use]
    pub fn error(error: FixError) -> Self {
        Self {
            success: false,
            fixed_source: None,
            errors: vec![error],
            conflicts: Vec::new(),
            applied_fixes: Vec::new(),
        }
    }

    /// Create a result with conflicts
    #[must_use]
    pub fn with_conflicts(conflicts: Vec<FixConflict>) -> Self {
        Self {
            success: false,
            fixed_source: None,
            errors: Vec::new(),
            conflicts,
            applied_fixes: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_creation() {
        let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
        let fix = Fix::new("test-rule", range.clone(), "replacement", "Test fix");

        assert_eq!(fix.rule_id, "test-rule");
        assert_eq!(fix.range, range);
        assert_eq!(fix.replacement, "replacement");
        assert_eq!(fix.description, "Test fix");
        assert_eq!(fix.severity, Severity::Hint);
        assert!(fix.dependencies.is_empty());
        assert!(fix.is_safe);
    }

    #[test]
    fn test_fix_with_severity() {
        let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
        let fix = Fix::new("test", range, "text", "desc").with_severity(Severity::Error);

        assert_eq!(fix.severity, Severity::Error);
    }

    #[test]
    fn test_fix_with_unsafe() {
        let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
        let fix = Fix::new("test", range, "text", "desc").with_unsafe();

        assert!(!fix.is_safe);
    }

    #[test]
    fn test_fix_with_dependency() {
        let dep_id = Uuid::new_v4();
        let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
        let fix = Fix::new("test", range, "text", "desc").with_dependency(dep_id);

        assert_eq!(fix.dependencies.len(), 1);
        assert_eq!(fix.dependencies[0], dep_id);
    }

    #[test]
    fn test_fix_with_dependencies() {
        let deps = vec![Uuid::new_v4(), Uuid::new_v4()];
        let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
        let fix = Fix::new("test", range, "text", "desc").with_dependencies(deps.clone());

        assert_eq!(fix.dependencies.len(), 2);
        assert_eq!(fix.dependencies, deps);
    }

    #[test]
    fn test_fix_unique_ids() {
        let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
        let fix1 = Fix::new("test", range.clone(), "text", "desc");
        let fix2 = Fix::new("test", range, "text", "desc");

        assert_ne!(fix1.id, fix2.id);
    }

    #[test]
    fn test_fix_result_success() {
        let result = FixResult::success("fixed text".to_string(), vec![]);

        assert!(result.success);
        assert_eq!(result.fixed_source, Some("fixed text".to_string()));
        assert!(result.errors.is_empty());
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_fix_result_error() {
        let error = FixError::InvalidRange("test".to_string());
        let result = FixResult::error(error);

        assert!(!result.success);
        assert!(result.fixed_source.is_none());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_fix_result_with_conflicts() {
        let conflicts = vec![];
        let result = FixResult::with_conflicts(conflicts);

        assert!(!result.success);
        assert!(result.fixed_source.is_none());
    }
}
