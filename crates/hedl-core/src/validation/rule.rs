// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Core trait and types for validation rules.

use crate::validation::{Diagnostic, Severity, ValidationContext};
use crate::{Document, HedlError, Node, Value};

/// Core trait for validation rules.
///
/// ValidationRule provides a flexible interface for implementing both built-in
/// and custom validation logic. Rules can operate at various granularities
/// (document, node, field) and can share state via ValidationContext.
///
/// # Thread Safety
///
/// Rules must be `Send + Sync` to enable parallel validation in the future.
///
/// # Performance
///
/// Rules should implement `cost_estimate()` to help the validator optimize
/// execution order. Cheap rules run first to enable early exit on invalid documents.
pub trait Rule: Send + Sync {
    /// Unique identifier for this rule (e.g., "required-fields").
    fn id(&self) -> &str;

    /// Human-readable description of what this rule validates.
    fn description(&self) -> &str;

    /// Rule category for organization and filtering.
    fn category(&self) -> RuleCategory;

    /// Default severity if validation fails.
    fn default_severity(&self) -> Severity;

    /// Validate a document and return diagnostics.
    ///
    /// The rule should traverse the document and emit diagnostics for any
    /// violations. Use the context to access shared state like type registries
    /// and to respect recursion limits.
    ///
    /// # Errors
    ///
    /// Returns an error if the rule encounters an unrecoverable condition
    /// (e.g., exceeding recursion limits, internal errors). Validation failures
    /// should be returned as diagnostics, not errors.
    fn check(
        &self,
        doc: &Document,
        context: &mut ValidationContext,
    ) -> Result<Vec<Diagnostic>, HedlError>;

    /// Validate a single node (optional override for node-level rules).
    ///
    /// Default implementation returns empty diagnostics. Override for rules
    /// that can validate individual nodes efficiently.
    fn check_node(
        &self,
        _node: &Node,
        _context: &mut ValidationContext,
    ) -> Result<Vec<Diagnostic>, HedlError> {
        Ok(vec![])
    }

    /// Validate a field value (optional override for field-level rules).
    ///
    /// Default implementation returns empty diagnostics. Override for rules
    /// that validate specific field values.
    fn check_field(
        &self,
        _value: &Value,
        _field_name: &str,
        _context: &mut ValidationContext,
    ) -> Result<Vec<Diagnostic>, HedlError> {
        Ok(vec![])
    }

    /// Check if this rule is applicable in the current context.
    ///
    /// Allows rules to skip execution based on document properties,
    /// configuration, or context state. Used for conditional validation.
    ///
    /// Default implementation always returns true.
    fn is_applicable(&self, _context: &ValidationContext) -> bool {
        true
    }

    /// Estimate computational cost (0-100, higher = more expensive).
    ///
    /// Used by validator to order rules efficiently. Cheap rules run first
    /// to enable early exit. Guidelines:
    ///
    /// - 0-25: Very cheap (simple field checks, O(1) operations)
    /// - 26-50: Medium cost (single-pass traversal, O(n))
    /// - 51-75: Expensive (multiple passes, complex analysis)
    /// - 76-100: Very expensive (graph algorithms, external calls)
    ///
    /// Default is 50 (medium cost).
    fn cost_estimate(&self) -> u8 {
        50
    }

    /// Lifecycle hook called before validating a document.
    ///
    /// Use this to initialize rule-specific state in the context.
    /// Default implementation does nothing.
    fn before_document(&self, _context: &mut ValidationContext) -> Result<(), HedlError> {
        Ok(())
    }

    /// Lifecycle hook called after validating a document.
    ///
    /// Use this to clean up rule-specific state or perform final checks.
    /// Default implementation does nothing.
    fn after_document(&self, _context: &mut ValidationContext) -> Result<(), HedlError> {
        Ok(())
    }

    /// Check if this rule supports automatic fixes.
    ///
    /// Default implementation returns false. Override to true if the rule
    /// can provide [`DiagnosticFix`] suggestions.
    fn supports_auto_fix(&self) -> bool {
        false
    }
}

/// Rule categories for organization and filtering.
///
/// Categories help users enable/disable groups of related rules and provide
/// context about what aspect of the document is being validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuleCategory {
    /// Type system validation (type mismatches, coercion failures).
    TypeSafety,
    /// Structural constraints (required fields, nesting limits).
    Structure,
    /// Referential integrity (dangling references, circular deps).
    References,
    /// Schema compliance (field count, type definitions).
    Schema,
    /// Business logic rules (domain-specific constraints).
    BusinessLogic,
    /// Performance and resource limits.
    Performance,
    /// Style and conventions (naming, formatting).
    Style,
    /// Security constraints (injection, DoS protection).
    Security,
}

impl RuleCategory {
    /// Get human-readable name for this category.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TypeSafety => "type-safety",
            Self::Structure => "structure",
            Self::References => "references",
            Self::Schema => "schema",
            Self::BusinessLogic => "business-logic",
            Self::Performance => "performance",
            Self::Style => "style",
            Self::Security => "security",
        }
    }
}

impl std::fmt::Display for RuleCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRule;

    impl Rule for TestRule {
        fn id(&self) -> &str {
            "test-rule"
        }
        fn description(&self) -> &str {
            "Test rule"
        }
        fn category(&self) -> RuleCategory {
            RuleCategory::Structure
        }
        fn default_severity(&self) -> Severity {
            Severity::Warning
        }
        fn check(
            &self,
            _doc: &Document,
            _context: &mut ValidationContext,
        ) -> Result<Vec<Diagnostic>, HedlError> {
            Ok(vec![])
        }
    }

    #[test]
    fn test_rule_defaults() {
        let rule = TestRule;
        assert_eq!(rule.id(), "test-rule");
        assert_eq!(rule.category(), RuleCategory::Structure);
        assert_eq!(rule.cost_estimate(), 50);
        assert!(!rule.supports_auto_fix());
    }

    #[test]
    fn test_rule_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<TestRule>();
    }

    #[test]
    fn test_category_display() {
        assert_eq!(format!("{}", RuleCategory::TypeSafety), "type-safety");
        assert_eq!(format!("{}", RuleCategory::References), "references");
    }

    #[test]
    fn test_category_equality() {
        assert_eq!(RuleCategory::Schema, RuleCategory::Schema);
        assert_ne!(RuleCategory::Schema, RuleCategory::Security);
    }
}
