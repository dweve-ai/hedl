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

//! Comprehensive escalation tests for HEDL linting.
//!
//! Tests severity escalation, configuration handling, security limits,
//! and edge cases for the lint system.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_lint::{
    lint, lint_with_config, Diagnostic, DiagnosticKind, LintConfig, LintContext, LintRule,
    LintRunner, RuleConfig, Severity,
};
use std::collections::BTreeMap;
use std::path::PathBuf;

// =============================================================================
// SEVERITY ESCALATION TESTS
// =============================================================================

#[test]
fn test_severity_ordering() {
    assert!(Severity::Hint < Severity::Warning);
    assert!(Severity::Warning < Severity::Error);
    assert!(Severity::Hint < Severity::Error);

    // Equality
    assert_eq!(Severity::Hint, Severity::Hint);
    assert_eq!(Severity::Warning, Severity::Warning);
    assert_eq!(Severity::Error, Severity::Error);
}

#[test]
fn test_severity_sorting() {
    let mut severities = vec![Severity::Warning, Severity::Hint, Severity::Error];
    severities.sort();
    assert_eq!(
        severities,
        vec![Severity::Hint, Severity::Warning, Severity::Error]
    );
}

#[test]
fn test_severity_copy() {
    let sev = Severity::Error;
    let copied = sev; // Copy semantics
    assert_eq!(sev, copied);
}

#[test]
fn test_diagnostic_escalate_to_error() {
    let mut diag = Diagnostic::warning(DiagnosticKind::UnusedSchema, "Test", "rule");
    assert_eq!(diag.severity(), Severity::Warning);

    diag.escalate_to_error();
    assert_eq!(diag.severity(), Severity::Error);
}

#[test]
fn test_diagnostic_escalate_hint_to_error() {
    let mut diag = Diagnostic::hint(DiagnosticKind::IdNaming, "Test", "rule");
    assert_eq!(diag.severity(), Severity::Hint);

    diag.escalate_to_error();
    assert_eq!(diag.severity(), Severity::Error);
}

#[test]
fn test_diagnostic_escalate_error_to_error() {
    // Escalating an error should remain error
    let mut diag = Diagnostic::error(
        DiagnosticKind::Custom("duplicate-key".to_string()),
        "Test",
        "rule",
    );
    assert_eq!(diag.severity(), Severity::Error);

    diag.escalate_to_error();
    assert_eq!(diag.severity(), Severity::Error);
}

#[test]
fn test_diagnostic_escalate_multiple_times() {
    let mut diag = Diagnostic::hint(DiagnosticKind::EmptyList, "Test", "rule");
    diag.escalate_to_error();
    diag.escalate_to_error();
    diag.escalate_to_error();
    assert_eq!(diag.severity(), Severity::Error);
}

// =============================================================================
// CONFIG-BASED ESCALATION TESTS
// =============================================================================

#[test]
fn test_config_set_rule_error() {
    let mut config = LintConfig::default();
    config.set_rule_error("unused-schema");

    let rule_config = config.rules.get("unused-schema").unwrap();
    assert!(rule_config.enabled);
    assert!(rule_config.error);
}

#[test]
fn test_config_escalation_affects_warnings() {
    let mut config = LintConfig::default();
    config.set_rule_error("unqualified-kv-ref");

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let ref_val = Value::Reference(Reference::local("some_id"));
    doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

    let diagnostics = runner.run(&doc);

    // UnqualifiedKvReference produces Warning, should be escalated to Error
    let ref_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();
    assert!(!ref_diags.is_empty());
    assert!(ref_diags.iter().all(|d| d.severity() == Severity::Error));
}

#[test]
fn test_config_escalation_does_not_affect_hints() {
    // Note: When set_rule_error is called, ALL severities are escalated to Error
    // This test verifies that a rule WITHOUT set_rule_error keeps hints as hints
    let config = LintConfig::default(); // No escalation

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let list = MatrixList::new("Empty", vec!["id".to_string()]);
    doc.root.insert("empty".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // EmptyList produces Hint by default (without escalation)
    let empty_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();
    // The diagnostic should exist
    assert!(!empty_diags.is_empty());
    // Hint should remain a Hint (no escalation was configured)
    assert!(empty_diags.iter().all(|d| d.severity() == Severity::Hint));
}

#[test]
fn test_config_enable_rule() {
    let mut config = LintConfig::default();
    config.enable_rule("test-rule");

    let rule_config = config.rules.get("test-rule").unwrap();
    assert!(rule_config.enabled);
    assert!(!rule_config.error);
}

#[test]
fn test_config_disable_rule() {
    let mut config = LintConfig::default();
    config.disable_rule("id-naming");

    let rule_config = config.rules.get("id-naming").unwrap();
    assert!(!rule_config.enabled);
    assert!(!rule_config.error);
}

#[test]
fn test_config_disable_then_escalate() {
    let mut config = LintConfig::default();
    config.disable_rule("unused-schema");
    config.set_rule_error("unused-schema");

    // set_rule_error should override disable
    let rule_config = config.rules.get("unused-schema").unwrap();
    assert!(rule_config.enabled);
    assert!(rule_config.error);
}

#[test]
fn test_config_escalate_then_disable() {
    let mut config = LintConfig::default();
    config.set_rule_error("unused-schema");
    config.disable_rule("unused-schema");

    // disable_rule should override set_rule_error
    let rule_config = config.rules.get("unused-schema").unwrap();
    assert!(!rule_config.enabled);
    assert!(!rule_config.error);
}

// =============================================================================
// MIN SEVERITY FILTER TESTS
// =============================================================================

#[test]
fn test_min_severity_hint() {
    let config = LintConfig {
        min_severity: Severity::Hint,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let list = MatrixList::new("Empty", vec!["id".to_string()]);
    doc.root.insert("empty".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Hints should be included
    let empty_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();
    assert!(!empty_diags.is_empty());
}

#[test]
fn test_min_severity_warning() {
    let config = LintConfig {
        min_severity: Severity::Warning,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let list = MatrixList::new("Empty", vec!["id".to_string()]);
    doc.root.insert("empty".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Hints should be excluded
    let empty_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();
    assert!(empty_diags.is_empty());
}

#[test]
fn test_min_severity_error() {
    let config = LintConfig {
        min_severity: Severity::Error,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let ref_val = Value::Reference(Reference::local("id"));
    doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

    let diagnostics = runner.run(&doc);

    // Warnings should be excluded when min_severity is Error
    let ref_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();
    assert!(ref_diags.is_empty());
}

#[test]
fn test_min_severity_with_escalation() {
    // Test interaction between min_severity and escalation
    let mut config = LintConfig {
        min_severity: Severity::Error,
        ..Default::default()
    };
    config.set_rule_error("unqualified-kv-ref");

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let ref_val = Value::Reference(Reference::local("id"));
    doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

    let diagnostics = runner.run(&doc);

    // Warning escalated to Error should pass the Error filter
    let ref_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();
    assert!(!ref_diags.is_empty());
    assert!(ref_diags.iter().all(|d| d.severity() == Severity::Error));
}

// =============================================================================
// DIAGNOSTIC LIMIT TESTS
// =============================================================================

#[test]
fn test_diagnostic_limit_default() {
    let config = LintConfig::default();
    assert_eq!(config.max_diagnostics, 10_000);
}

#[test]
fn test_diagnostic_limit_custom() {
    let config = LintConfig {
        max_diagnostics: 5,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    // Create many violations
    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    for i in 0..100 {
        list.add_row(Node::new("Test", format!("{i}"), vec![])); // Numeric IDs
    }
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Should be limited to max_diagnostics + 1 (for the limit warning)
    assert!(diagnostics.len() <= 6);

    // Should have a diagnostic-limit-exceeded warning
    let limit_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| {
            if let DiagnosticKind::Custom(name) = d.kind() {
                name == "diagnostic-limit-exceeded"
            } else {
                false
            }
        })
        .collect();
    assert!(!limit_diags.is_empty());
}

#[test]
fn test_diagnostic_limit_zero() {
    let config = LintConfig {
        max_diagnostics: 0,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let list = MatrixList::new("Empty", vec!["id".to_string()]);
    doc.root.insert("empty".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // When max_diagnostics is 0, no diagnostics are returned at all
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_diagnostic_limit_exactly_reached() {
    let config = LintConfig {
        max_diagnostics: 3,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    for i in 0..10 {
        list.add_row(Node::new("Test", format!("{i}"), vec![]));
    }
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Should have exactly 3 diagnostics + 1 limit warning
    assert!(diagnostics.len() <= 4);
}

// =============================================================================
// CONFIG VALIDATION TESTS
// =============================================================================

#[test]
fn test_config_validation_valid() {
    let config = LintConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validation_with_rules() {
    let mut config = LintConfig::default();
    config.disable_rule("id-naming");
    config.set_rule_error("unused-schema");
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validation_empty_rule_id() {
    let mut config = LintConfig::default();
    config.rules.insert(
        String::new(),
        RuleConfig {
            enabled: true,
            error: false,
        },
    );
    assert!(config.validate().is_err());
    assert!(config.validate().unwrap_err().contains("Empty rule ID"));
}

#[test]
fn test_config_validation_rule_id_too_long() {
    let mut config = LintConfig::default();
    let long_id = "x".repeat(101);
    config.rules.insert(
        long_id,
        RuleConfig {
            enabled: true,
            error: false,
        },
    );
    assert!(config.validate().is_err());
    assert!(config.validate().unwrap_err().contains("too long"));
}

#[test]
fn test_config_validation_many_rules() {
    let mut config = LintConfig::default();
    for i in 0..1001 {
        config.rules.insert(
            format!("rule-{i}"),
            RuleConfig {
                enabled: true,
                error: false,
            },
        );
    }
    assert!(config.validate().is_err());
    assert!(config.validate().unwrap_err().contains("Too many"));
}

#[test]
fn test_config_validation_boundary_rule_count() {
    let mut config = LintConfig::default();
    for i in 0..1000 {
        config.rules.insert(
            format!("rule-{i}"),
            RuleConfig {
                enabled: true,
                error: false,
            },
        );
    }
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validation_boundary_rule_id_length() {
    let mut config = LintConfig::default();
    let max_length_id = "x".repeat(100);
    config.rules.insert(
        max_length_id,
        RuleConfig {
            enabled: true,
            error: false,
        },
    );
    assert!(config.validate().is_ok());
}

// =============================================================================
// DIAGNOSTIC SORTING TESTS
// =============================================================================

#[test]
fn test_diagnostics_sorted_by_severity() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Create violations of different severities
    let list = MatrixList::new("Empty", vec!["id".to_string()]);
    doc.root.insert("empty".to_string(), Item::List(list));

    let ref_val = Value::Reference(Reference::local("id"));
    doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]);

    let diagnostics = runner.run(&doc);

    // Verify sorted by severity (errors first)
    let mut prev_severity = Severity::Error;
    for diag in &diagnostics {
        assert!(
            diag.severity() <= prev_severity,
            "Diagnostics not sorted by severity"
        );
        prev_severity = diag.severity();
    }
}

// =============================================================================
// HAS_ERRORS TESTS
// =============================================================================

#[test]
fn test_has_errors_with_error() {
    let runner = LintRunner::new(LintConfig::default());
    let diagnostics = vec![Diagnostic::error(
        DiagnosticKind::Custom("duplicate-key".to_string()),
        "test",
        "rule",
    )];
    assert!(runner.has_errors(&diagnostics));
}

#[test]
fn test_has_errors_without_error() {
    let runner = LintRunner::new(LintConfig::default());
    let diagnostics = vec![
        Diagnostic::warning(DiagnosticKind::UnusedSchema, "test", "rule"),
        Diagnostic::hint(DiagnosticKind::IdNaming, "test", "rule"),
    ];
    assert!(!runner.has_errors(&diagnostics));
}

#[test]
fn test_has_errors_empty() {
    let runner = LintRunner::new(LintConfig::default());
    let diagnostics: Vec<Diagnostic> = vec![];
    assert!(!runner.has_errors(&diagnostics));
}

#[test]
fn test_has_errors_mixed_with_escalation() {
    let mut config = LintConfig::default();
    config.set_rule_error("unqualified-kv-ref");

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let ref_val = Value::Reference(Reference::local("id"));
    doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

    let diagnostics = runner.run(&doc);
    assert!(runner.has_errors(&diagnostics));
}

// =============================================================================
// LINT CONTEXT TESTS
// =============================================================================

#[test]
fn test_lint_context_new() {
    let path = Some(PathBuf::from("test.hedl"));
    let context = LintContext::new(path.clone(), 42, "source text");

    assert_eq!(context.file_path, path);
    assert_eq!(context.line_number, 42);
    assert_eq!(context.source_text, "source text");
}

#[test]
fn test_lint_context_from_text() {
    let context = LintContext::from_text("some source");

    assert!(context.file_path.is_none());
    assert_eq!(context.line_number, 0);
    assert_eq!(context.source_text, "some source");
}

#[test]
fn test_lint_context_with_file() {
    let path = PathBuf::from("data.hedl");
    let context = LintContext::with_file(path.clone(), "content");

    assert_eq!(context.file_path, Some(path));
    assert_eq!(context.line_number, 0);
}

#[test]
fn test_lint_context_with_line() {
    let context = LintContext::from_text("line1\nline2\nline3").with_line(2);
    assert_eq!(context.line_number, 2);
}

#[test]
fn test_lint_context_file_name() {
    let path = PathBuf::from("data.hedl");
    let context = LintContext::with_file(path, "");
    assert_eq!(context.file_name(), Some("data.hedl".to_string()));
}

#[test]
fn test_lint_context_file_name_with_path() {
    let path = PathBuf::from("/path/to/data.hedl");
    let context = LintContext::with_file(path, "");
    assert_eq!(context.file_name(), Some("data.hedl".to_string()));
}

#[test]
fn test_lint_context_file_name_none() {
    let context = LintContext::from_text("");
    assert!(context.file_name().is_none());
}

#[test]
fn test_lint_context_get_line() {
    let source = "line1\nline2\nline3";
    let context = LintContext::from_text(source);

    assert_eq!(context.get_line(1), Some("line1"));
    assert_eq!(context.get_line(2), Some("line2"));
    assert_eq!(context.get_line(3), Some("line3"));
    assert_eq!(context.get_line(4), None);
}

#[test]
fn test_lint_context_get_line_zero() {
    let source = "line1\nline2";
    let context = LintContext::from_text(source);
    assert_eq!(context.get_line(0), None);
}

#[test]
fn test_lint_context_current_line() {
    let source = "line1\nline2\nline3";
    let context = LintContext::from_text(source).with_line(2);
    assert_eq!(context.current_line(), Some("line2"));
}

#[test]
fn test_lint_context_current_line_unset() {
    let source = "line1\nline2";
    let context = LintContext::from_text(source);
    assert_eq!(context.current_line(), None);
}

// =============================================================================
// CUSTOM RULE TESTS
// =============================================================================

struct AlwaysErrorRule;

impl LintRule for AlwaysErrorRule {
    fn id(&self) -> &'static str {
        "always-error"
    }

    fn description(&self) -> &'static str {
        "Always produces an error"
    }

    fn check(&self, _doc: &Document) -> Vec<Diagnostic> {
        vec![Diagnostic::error(
            DiagnosticKind::Custom("always-error".to_string()),
            "This is always an error",
            "always-error",
        )]
    }
}

struct AlwaysWarningRule;

impl LintRule for AlwaysWarningRule {
    fn id(&self) -> &'static str {
        "always-warning"
    }

    fn description(&self) -> &'static str {
        "Always produces a warning"
    }

    fn check(&self, _doc: &Document) -> Vec<Diagnostic> {
        vec![Diagnostic::warning(
            DiagnosticKind::Custom("always-warning".to_string()),
            "This is always a warning",
            "always-warning",
        )]
    }
}

#[test]
fn test_custom_rule_error() {
    let config = LintConfig::default();
    let mut runner = LintRunner::with_rules(config, vec![]);
    runner.add_rule(Box::new(AlwaysErrorRule));

    let doc = Document::new((2, 0));
    let diagnostics = runner.run(&doc);

    assert!(runner.has_errors(&diagnostics));
}

#[test]
fn test_custom_rule_warning_escalation() {
    let mut config = LintConfig::default();
    config.set_rule_error("always-warning");

    let mut runner = LintRunner::with_rules(config, vec![]);
    runner.add_rule(Box::new(AlwaysWarningRule));

    let doc = Document::new((2, 0));
    let diagnostics = runner.run(&doc);

    assert!(runner.has_errors(&diagnostics));
}

#[test]
fn test_custom_rule_disabled() {
    let mut config = LintConfig::default();
    config.disable_rule("always-error");

    let mut runner = LintRunner::with_rules(config, vec![]);
    runner.add_rule(Box::new(AlwaysErrorRule));

    let doc = Document::new((2, 0));
    let diagnostics = runner.run(&doc);

    assert!(!runner.has_errors(&diagnostics));
}

// =============================================================================
// MULTIPLE RULES INTERACTION TESTS
// =============================================================================

#[test]
fn test_multiple_rules_all_enabled() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    list.add_row(Node::new("Test", "a", vec![])); // Short ID
    doc.root.insert("items".to_string(), Item::List(list));

    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]);

    let ref_val = Value::Reference(Reference::local("id"));
    doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

    let diagnostics = runner.run(&doc);

    // Should have diagnostics from multiple rules
    assert!(diagnostics.len() >= 3);
}

#[test]
fn test_multiple_rules_some_disabled() {
    let mut config = LintConfig::default();
    config.disable_rule("id-naming");
    config.disable_rule("empty-list");

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    list.add_row(Node::new("Test", "a", vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]);

    let diagnostics = runner.run(&doc);

    // Should have UnusedSchema but not IdNaming or EmptyList
    let id_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .collect();
    let empty_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();
    let unused_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert!(id_diags.is_empty());
    assert!(empty_diags.is_empty());
    assert!(!unused_diags.is_empty());
}

#[test]
fn test_multiple_rules_all_escalated() {
    let mut config = LintConfig::default();
    config.set_rule_error("unused-schema");
    config.set_rule_error("unqualified-kv-ref");

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));

    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]);

    let ref_val = Value::Reference(Reference::local("id"));
    doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

    let diagnostics = runner.run(&doc);

    // Both should be errors
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity() == Severity::Error)
        .collect();
    assert!(errors.len() >= 2);
}

// =============================================================================
// DIAGNOSTIC KIND TESTS
// =============================================================================

#[test]
fn test_diagnostic_kind_custom() {
    let kind = DiagnosticKind::Custom("my-rule".to_string());
    if let DiagnosticKind::Custom(name) = kind {
        assert_eq!(name, "my-rule");
    } else {
        panic!("Expected Custom variant");
    }
}

#[test]
fn test_diagnostic_kind_equality() {
    assert_eq!(DiagnosticKind::IdNaming, DiagnosticKind::IdNaming);
    assert_ne!(DiagnosticKind::IdNaming, DiagnosticKind::UnusedSchema);

    let kind1 = DiagnosticKind::Custom("rule".to_string());
    let kind2 = DiagnosticKind::Custom("rule".to_string());
    let kind3 = DiagnosticKind::Custom("other".to_string());

    assert_eq!(kind1, kind2);
    assert_ne!(kind1, kind3);
}

// =============================================================================
// DIAGNOSTIC BUILDER TESTS
// =============================================================================

#[test]
fn test_diagnostic_with_line() {
    let diag = Diagnostic::warning(DiagnosticKind::EmptyList, "msg", "rule").with_line(42);
    assert_eq!(diag.line(), Some(42));
}

#[test]
fn test_diagnostic_with_suggestion() {
    let diag = Diagnostic::warning(DiagnosticKind::UnqualifiedKvReference, "msg", "rule")
        .with_suggestion("Use @Type:id");
    assert_eq!(diag.suggestion(), Some("Use @Type:id"));
}

#[test]
fn test_diagnostic_chained_builders() {
    let diag = Diagnostic::error(DiagnosticKind::IdNaming, "msg", "rule")
        .with_line(100)
        .with_suggestion("Fix it");

    assert_eq!(diag.severity(), Severity::Error);
    assert_eq!(diag.line(), Some(100));
    assert_eq!(diag.suggestion(), Some("Fix it"));
}

#[test]
fn test_diagnostic_display() {
    let diag = Diagnostic::warning(DiagnosticKind::UnusedSchema, "Test message", "test-rule");
    let display = format!("{diag}");
    assert!(display.contains("warning"));
    assert!(display.contains("Test message"));
    assert!(display.contains("test-rule"));
}

#[test]
fn test_diagnostic_display_with_line() {
    let diag = Diagnostic::error(
        DiagnosticKind::Custom("duplicate-key".to_string()),
        "Duplicate key found",
        "dup-key",
    )
    .with_line(42);
    let display = format!("{diag}");
    assert!(display.contains("line 42"));
}

#[test]
fn test_diagnostic_display_with_suggestion() {
    let diag = Diagnostic::warning(DiagnosticKind::IdNaming, "Short ID", "id")
        .with_suggestion("Use longer name");
    let display = format!("{diag}");
    assert!(display.contains("Use longer name"));
}

// =============================================================================
// LINT API TESTS
// =============================================================================

#[test]
fn test_lint_function() {
    let doc = Document::new((2, 0));
    let diagnostics = lint(&doc);
    assert!(diagnostics.is_empty());
}

#[test]
fn test_lint_with_config_function() {
    let mut config = LintConfig::default();
    config.disable_rule("id-naming");

    let doc = Document::new((2, 0));
    let diagnostics = lint_with_config(&doc, config);
    assert!(diagnostics.is_empty());
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[test]
fn test_empty_document() {
    let runner = LintRunner::new(LintConfig::default());
    let doc = Document::new((2, 0));
    let diagnostics = runner.run(&doc);
    assert!(diagnostics.is_empty());
}

#[test]
fn test_deeply_nested_objects() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Create nested structure
    let mut innermost = BTreeMap::new();
    innermost.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("id"))),
    );

    let mut middle = BTreeMap::new();
    middle.insert("inner".to_string(), Item::Object(innermost));

    let mut outer = BTreeMap::new();
    outer.insert("middle".to_string(), Item::Object(middle));

    doc.root.insert("outer".to_string(), Item::Object(outer));

    let diagnostics = runner.run(&doc);

    // Should find the nested unqualified reference
    let ref_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();
    assert!(!ref_diags.is_empty());
}

#[test]
fn test_unicode_in_ids() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    list.add_row(Node::new("Test", "user_alice_你好", vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Unicode ID should be fine
    let id_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .collect();
    assert!(id_diags.is_empty());
}

#[test]
fn test_very_long_id() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    let long_id = "user_".to_string() + &"a".repeat(1000);
    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    list.add_row(Node::new("Test", &long_id, vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Very long ID should be fine (not short or numeric)
    let id_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .collect();
    assert!(id_diags.is_empty());
}

// =============================================================================
// RULE CONFIG TESTS
// =============================================================================

#[test]
fn test_rule_config_default() {
    let config = RuleConfig::default();
    assert!(config.enabled);
    assert!(!config.error);
}

#[test]
fn test_rule_config_clone() {
    let config = RuleConfig {
        enabled: false,
        error: true,
    };
    let cloned = config.clone();
    assert!(!cloned.enabled);
    assert!(cloned.error);
}

// =============================================================================
// CONCURRENCY TESTS
// =============================================================================

#[test]
fn test_lint_rule_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    // All rules should be Send + Sync for thread safety
    assert_send_sync::<AlwaysErrorRule>();
    assert_send_sync::<AlwaysWarningRule>();
}

#[test]
fn test_lint_runner_thread_safe() {
    use std::sync::Arc;
    use std::thread;

    let runner = Arc::new(LintRunner::new(LintConfig::default()));

    let mut handles = vec![];
    for _ in 0..4 {
        let runner = Arc::clone(&runner);
        let handle = thread::spawn(move || {
            let doc = Document::new((2, 0));
            runner.run(&doc)
        });
        handles.push(handle);
    }

    for handle in handles {
        let diagnostics = handle.join().unwrap();
        assert!(diagnostics.is_empty());
    }
}

// =============================================================================
// STRESS TESTS
// =============================================================================

#[test]
fn test_many_violations() {
    let config = LintConfig {
        max_diagnostics: 100,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));

    // Create many violations
    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    for i in 0..500 {
        list.add_row(Node::new("Test", format!("{i}"), vec![])); // Numeric IDs
    }
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Should be limited
    assert!(diagnostics.len() <= 101);
}

#[test]
fn test_many_empty_lists() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    for i in 0..50 {
        let list = MatrixList::new(format!("Type{i}"), vec!["id".to_string()]);
        doc.root.insert(format!("list{i}"), Item::List(list));
    }

    let diagnostics = runner.run(&doc);

    let empty_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();
    assert_eq!(empty_diags.len(), 50);
}

#[test]
fn test_many_unused_schemas() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    for i in 0..50 {
        doc.structs
            .insert(format!("UnusedType{i}"), vec!["id".to_string()]);
    }

    let diagnostics = runner.run(&doc);

    let unused_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();
    assert_eq!(unused_diags.len(), 50);
}

#[test]
fn test_many_unqualified_references() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    for i in 0..50 {
        let ref_val = Value::Reference(Reference::local(format!("id{i}")));
        doc.root.insert(format!("ref{i}"), Item::Scalar(ref_val));
    }

    let diagnostics = runner.run(&doc);

    let ref_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();
    assert_eq!(ref_diags.len(), 50);
}
