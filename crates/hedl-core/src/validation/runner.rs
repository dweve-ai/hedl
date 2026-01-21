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

//! Validation runner and orchestration.

use crate::validation::registry::RuleConfig;
use crate::validation::{
    Diagnostic, Rule, RuleRegistry, Severity, ValidationContext, ValidationStats,
};
use crate::{Document, Item, Node};
use std::collections::HashMap;

/// Configuration for validation.
#[derive(Debug, Clone)]
pub struct LintConfig {
    /// Maximum recursion depth.
    pub max_depth: usize,
    /// Maximum diagnostics to collect.
    pub max_diagnostics: usize,
    /// Minimum severity to report.
    pub min_severity: Severity,
    /// Per-rule configuration.
    pub rule_config: HashMap<String, RuleConfig>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            max_depth: 1000,
            max_diagnostics: 10_000,
            min_severity: Severity::Hint,
            rule_config: HashMap::new(),
        }
    }
}

impl LintConfig {
    /// Disable a specific rule.
    pub fn disable_rule(&mut self, rule_id: impl Into<String>) {
        let config = self.rule_config.entry(rule_id.into()).or_default();
        config.enabled = false;
    }

    /// Set a rule to escalate to error.
    pub fn set_rule_error(&mut self, rule_id: impl Into<String>) {
        let config = self.rule_config.entry(rule_id.into()).or_default();
        config.escalate_to_error = true;
    }

    /// Enable a specific rule.
    pub fn enable_rule(&mut self, rule_id: impl Into<String>) {
        let config = self.rule_config.entry(rule_id.into()).or_default();
        config.enabled = true;
    }
}

/// Result of validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// All diagnostics produced.
    pub diagnostics: Vec<Diagnostic>,
    /// Performance statistics.
    pub stats: ValidationStats,
    /// Overall validation status (true if no errors).
    pub is_valid: bool,
}

impl ValidationResult {
    /// Get errors only.
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_error())
    }

    /// Get warnings only.
    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_warning())
    }

    /// Get hints only.
    pub fn hints(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_hint())
    }
}

/// Main validation runner.
///
/// Orchestrates validation rules, manages context, and produces validation results.
pub struct ValidationRunner {
    /// Rule registry.
    registry: RuleRegistry,
    /// Configuration.
    config: LintConfig,
}

impl ValidationRunner {
    /// Create a new validator with default built-in rules.
    pub fn new(config: LintConfig) -> Self {
        let mut registry = RuleRegistry::new();

        // Register built-in rules
        registry.register_all(crate::validation::rules::default_rules());

        // Apply configuration
        for (rule_id, rule_config) in &config.rule_config {
            registry.set_config(rule_id, rule_config.clone());
        }

        Self { registry, config }
    }

    /// Create a validator without built-in rules.
    pub fn empty(config: LintConfig) -> Self {
        let mut registry = RuleRegistry::new();

        // Apply configuration (same as new, just without built-in rules)
        for (rule_id, rule_config) in &config.rule_config {
            registry.set_config(rule_id, rule_config.clone());
        }

        Self { registry, config }
    }

    /// Add a custom rule.
    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.registry.register(rule);
    }

    /// Validate a document.
    pub fn validate(&self, doc: &Document) -> ValidationResult {
        let mut context =
            ValidationContext::with_limits(self.config.max_depth, self.config.max_diagnostics);
        let mut stats = ValidationStats::start();

        // Build symbol table
        build_symbol_table(doc, &mut context);

        // Get enabled rules sorted by cost
        let rules = self.registry.enabled_rules();

        // Execute rules
        for rule in rules {
            // Check if applicable
            if !rule.is_applicable(&context) {
                continue;
            }

            // Call before hook
            if let Err(e) = rule.before_document(&mut context) {
                // Log error but continue
                let _ = context.add_diagnostic(Diagnostic::error(
                    crate::validation::DiagnosticKind::Custom("rule-error".to_string()),
                    format!("Rule '{}' before_document failed: {}", rule.id(), e),
                    rule.id(),
                ));
                continue;
            }

            // Execute rule
            match rule.check(doc, &mut context) {
                Ok(mut diagnostics) => {
                    // Apply escalation if configured
                    if self.registry.should_escalate(rule.id()) {
                        for diag in &mut diagnostics {
                            if !diag.is_error() {
                                diag.escalate_to_error();
                            }
                        }
                    }

                    // Filter by min severity
                    diagnostics.retain(|d| d.severity() >= self.config.min_severity);

                    // Add to context
                    for diag in diagnostics {
                        if context.add_diagnostic(diag).is_err() {
                            break; // Hit diagnostic limit
                        }
                    }

                    stats.rules_executed += 1;
                }
                Err(e) => {
                    // Rule execution failed
                    let _ = context.add_diagnostic(Diagnostic::error(
                        crate::validation::DiagnosticKind::Custom("rule-error".to_string()),
                        format!("Rule '{}' failed: {}", rule.id(), e),
                        rule.id(),
                    ));
                }
            }

            // Call after hook
            let _ = rule.after_document(&mut context);
        }

        stats.finish();

        let diagnostics = context.take_diagnostics();
        let is_valid = !diagnostics.iter().any(|d| d.is_error());

        ValidationResult {
            diagnostics,
            stats,
            is_valid,
        }
    }

    /// Get the rule registry.
    pub fn registry(&self) -> &RuleRegistry {
        &self.registry
    }

    /// Get mutable access to the rule registry.
    pub fn registry_mut(&mut self) -> &mut RuleRegistry {
        &mut self.registry
    }
}

/// Build symbol table from document.
fn build_symbol_table(doc: &Document, context: &mut ValidationContext) {
    fn visit_items(
        items: &std::collections::BTreeMap<String, Item>,
        context: &mut ValidationContext,
    ) {
        for item in items.values() {
            match item {
                Item::List(list) => {
                    for node in &list.rows {
                        context.register_node(&list.type_name, node);
                        if let Some(children) = node.children() {
                            visit_children(children, context);
                        }
                    }
                }
                Item::Object(obj) => {
                    visit_items(obj, context);
                }
                _ => {}
            }
        }
    }

    fn visit_children(
        children: &std::collections::BTreeMap<String, Vec<Node>>,
        context: &mut ValidationContext,
    ) {
        for (type_name, nodes) in children {
            for node in nodes {
                context.register_node(type_name, node);
                if let Some(grandchildren) = node.children() {
                    visit_children(grandchildren, context);
                }
            }
        }
    }

    visit_items(&doc.root, context);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{DiagnosticKind, Rule, RuleCategory};
    use crate::{HedlError, MatrixList, Value};

    struct TestRule {
        id: String,
        diagnostics: Vec<Diagnostic>,
    }

    impl TestRule {
        fn new(id: impl Into<String>, diagnostics: Vec<Diagnostic>) -> Self {
            Self {
                id: id.into(),
                diagnostics,
            }
        }
    }

    impl Rule for TestRule {
        fn id(&self) -> &str {
            &self.id
        }
        fn description(&self) -> &str {
            "Test"
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
            Ok(self.diagnostics.clone())
        }
    }

    #[test]
    fn test_runner_new() {
        let runner = ValidationRunner::new(LintConfig::default());
        assert!(runner.registry().count() > 0); // Has built-in rules
    }

    #[test]
    fn test_runner_empty() {
        let runner = ValidationRunner::empty(LintConfig::default());
        assert_eq!(runner.registry().count(), 0);
    }

    #[test]
    fn test_validate_empty_document() {
        let runner = ValidationRunner::new(LintConfig::default());
        let doc = Document::new((1, 0));
        let result = runner.validate(&doc);

        assert!(result.is_valid);
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_validate_with_custom_rule() {
        let mut runner = ValidationRunner::empty(LintConfig::default());

        let rule = TestRule::new(
            "test",
            vec![Diagnostic::warning(
                DiagnosticKind::Custom("test".to_string()),
                "Test warning",
                "test",
            )],
        );
        runner.add_rule(Box::new(rule));

        let doc = Document::new((1, 0));
        let result = runner.validate(&doc);

        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.is_valid); // Warnings don't make it invalid
    }

    #[test]
    fn test_validate_with_error() {
        let mut runner = ValidationRunner::empty(LintConfig::default());

        let rule = TestRule::new(
            "test",
            vec![Diagnostic::error(
                DiagnosticKind::DuplicateKey,
                "Duplicate key",
                "test",
            )],
        );
        runner.add_rule(Box::new(rule));

        let doc = Document::new((1, 0));
        let result = runner.validate(&doc);

        assert!(!result.is_valid);
        assert_eq!(result.errors().count(), 1);
    }

    #[test]
    fn test_config_disable_rule() {
        let mut config = LintConfig::default();
        config.disable_rule("duplicate-key");

        let mut runner = ValidationRunner::empty(config);

        // Use "duplicate-key" as the rule id so it can be disabled
        let rule = TestRule::new(
            "duplicate-key",
            vec![Diagnostic::error(
                DiagnosticKind::DuplicateKey,
                "Error",
                "duplicate-key",
            )],
        );
        runner.add_rule(Box::new(rule));

        runner.registry_mut().disable_rule("duplicate-key");

        let doc = Document::new((1, 0));
        let result = runner.validate(&doc);

        // Rule is disabled, no diagnostics
        assert_eq!(result.diagnostics.len(), 0);
    }

    #[test]
    fn test_symbol_table_build() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
        list.add_row(Node::new(
            "User",
            "alice",
            vec![Value::String("Alice".to_string().into())],
        ));
        doc.root.insert("users".to_string(), Item::List(list));

        let mut context = ValidationContext::new();
        build_symbol_table(&doc, &mut context);

        let reference = crate::Reference::local("alice");
        assert!(context.resolve_reference(&reference).is_some());
    }

    #[test]
    fn test_result_filters() {
        let result = ValidationResult {
            diagnostics: vec![
                Diagnostic::error(DiagnosticKind::DuplicateKey, "Error", "test"),
                Diagnostic::warning(DiagnosticKind::TypeMismatch, "Warning", "test"),
                Diagnostic::hint(DiagnosticKind::UnusedReference, "Hint", "test"),
            ],
            stats: ValidationStats::default(),
            is_valid: false,
        };

        assert_eq!(result.errors().count(), 1);
        assert_eq!(result.warnings().count(), 1);
        assert_eq!(result.hints().count(), 1);
    }

    #[test]
    fn test_escalation_config_works() {
        // Create config that escalates test rule to error
        let mut config = LintConfig::default();
        config.set_rule_error("test-rule");

        let mut runner = ValidationRunner::empty(config);

        // Add a rule that produces a warning
        let rule = TestRule::new(
            "test-rule",
            vec![Diagnostic::warning(
                DiagnosticKind::Custom("test".to_string()),
                "Test warning that should be escalated",
                "test-rule",
            )],
        );
        runner.add_rule(Box::new(rule));

        let doc = Document::new((1, 0));
        let result = runner.validate(&doc);

        // The warning should have been escalated to error
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].is_error());
        assert!(!result.is_valid); // Should be invalid now
    }

    #[test]
    fn test_escalation_hint_to_error() {
        let mut config = LintConfig::default();
        config.set_rule_error("hint-rule");

        let mut runner = ValidationRunner::empty(config);

        let rule = TestRule::new(
            "hint-rule",
            vec![Diagnostic::hint(
                DiagnosticKind::Custom("hint".to_string()),
                "Hint that should be escalated",
                "hint-rule",
            )],
        );
        runner.add_rule(Box::new(rule));

        let doc = Document::new((1, 0));
        let result = runner.validate(&doc);

        // Hint should have been escalated to error
        assert_eq!(result.diagnostics.len(), 1);
        assert!(result.diagnostics[0].is_error());
    }
}
