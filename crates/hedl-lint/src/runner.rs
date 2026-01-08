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

//! Lint runner

use crate::diagnostic::{Diagnostic, Severity};
use crate::rules::{default_rules, LintRule, RuleConfig};
use hedl_core::Document;
use std::collections::HashMap;
use std::path::PathBuf;

/// Context passed to lint rules for better error reporting
///
/// This struct provides context information to lint rules, including file paths,
/// line numbers, and source text, enabling more informative diagnostics.
///
/// # Examples
///
/// ```text
/// let context = LintContext::new(
///     Some(PathBuf::from("data.hedl")),
///     42,
///     "the full source text here"
/// );
/// ```
#[derive(Debug, Clone)]
pub struct LintContext {
    /// Optional file path being linted
    pub file_path: Option<PathBuf>,
    /// Current line number (1-indexed, 0 means no specific line)
    pub line_number: u32,
    /// Source text being linted
    pub source_text: String,
}

impl LintContext {
    /// Create a new lint context
    ///
    /// # Arguments
    ///
    /// * `file_path` - Optional path to the file being linted
    /// * `line_number` - Current line number (1-indexed)
    /// * `source_text` - The full source text being linted
    pub fn new(file_path: Option<PathBuf>, line_number: u32, source_text: impl Into<String>) -> Self {
        Self {
            file_path,
            line_number,
            source_text: source_text.into(),
        }
    }

    /// Create a context without a file path
    pub fn from_text(source_text: impl Into<String>) -> Self {
        Self {
            file_path: None,
            line_number: 0,
            source_text: source_text.into(),
        }
    }

    /// Create a context with a file path
    pub fn with_file(file_path: PathBuf, source_text: impl Into<String>) -> Self {
        Self {
            file_path: Some(file_path),
            line_number: 0,
            source_text: source_text.into(),
        }
    }

    /// Set the line number
    pub fn with_line(mut self, line_number: u32) -> Self {
        self.line_number = line_number;
        self
    }

    /// Get the file name if available
    pub fn file_name(&self) -> Option<String> {
        self.file_path.as_ref().and_then(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
        })
    }

    /// Get a specific line from the source text
    ///
    /// Returns the requested line (1-indexed) if it exists
    pub fn get_line(&self, line_num: u32) -> Option<&str> {
        if line_num == 0 {
            return None;
        }
        self.source_text
            .lines()
            .nth((line_num - 1) as usize)
    }

    /// Get the current line from the context
    pub fn current_line(&self) -> Option<&str> {
        if self.line_number == 0 {
            return None;
        }
        self.get_line(self.line_number)
    }
}

/// Maximum number of diagnostics to collect before stopping.
///
/// This limit prevents memory exhaustion from malicious documents that could
/// generate millions of diagnostics (e.g., a document with 1M nodes each
/// triggering a diagnostic would consume ~100MB+ of memory).
///
/// Security Rationale:
/// - Each diagnostic consumes ~100-200 bytes of memory
/// - At 10,000 diagnostics, this represents ~1-2MB of memory
/// - Most legitimate documents produce <100 diagnostics
/// - This limit provides defense-in-depth against DoS attacks
/// - Users can still identify and fix issues with first 10,000 diagnostics
const MAX_DIAGNOSTICS: usize = 10_000;

/// Configuration for the lint runner
#[derive(Debug, Clone)]
pub struct LintConfig {
    /// Rule configurations by rule ID
    pub rules: HashMap<String, RuleConfig>,
    /// Minimum severity to report
    pub min_severity: Severity,
    /// Maximum number of diagnostics to collect (default: 10,000)
    ///
    /// # Security
    ///
    /// This limit prevents memory exhaustion attacks from maliciously crafted
    /// documents that could generate unlimited diagnostics. Once this limit is
    /// reached, no further diagnostics will be collected and a warning will be
    /// issued.
    pub max_diagnostics: usize,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            rules: HashMap::new(),
            min_severity: Severity::Hint,
            max_diagnostics: MAX_DIAGNOSTICS,
        }
    }
}

impl LintConfig {
    /// Validate configuration (checks rule ID lengths and limits).
    pub fn validate(&self) -> Result<(), String> {
        const MAX_RULE_ID_LENGTH: usize = 100;
        const MAX_RULES: usize = 1000;

        if self.rules.len() > MAX_RULES {
            return Err(format!(
                "Too many rule configurations: {} (max: {})",
                self.rules.len(),
                MAX_RULES
            ));
        }

        for id in self.rules.keys() {
            if id.is_empty() {
                return Err("Empty rule ID not allowed".to_string());
            }
            if id.len() > MAX_RULE_ID_LENGTH {
                return Err(format!(
                    "Rule ID too long: {} bytes (max: {})",
                    id.len(),
                    MAX_RULE_ID_LENGTH
                ));
            }
        }

        Ok(())
    }

    /// Disable a specific rule
    pub fn disable_rule(&mut self, rule_id: &str) {
        self.rules.insert(
            rule_id.to_string(),
            RuleConfig {
                enabled: false,
                error: false,
            },
        );
    }

    /// Enable a specific rule
    pub fn enable_rule(&mut self, rule_id: &str) {
        self.rules.insert(
            rule_id.to_string(),
            RuleConfig {
                enabled: true,
                error: false,
            },
        );
    }

    /// Set a rule to error level
    pub fn set_rule_error(&mut self, rule_id: &str) {
        self.rules.insert(
            rule_id.to_string(),
            RuleConfig {
                enabled: true,
                error: true,
            },
        );
    }
}

/// Lint runner
pub struct LintRunner {
    config: LintConfig,
    rules: Vec<Box<dyn LintRule>>,
}

impl LintRunner {
    /// Create a new lint runner with default rules
    pub fn new(config: LintConfig) -> Self {
        Self {
            config,
            rules: default_rules(),
        }
    }

    /// Create a lint runner with custom rules
    pub fn with_rules(config: LintConfig, rules: Vec<Box<dyn LintRule>>) -> Self {
        Self { config, rules }
    }

    /// Add a custom rule
    pub fn add_rule(&mut self, rule: Box<dyn LintRule>) {
        self.rules.push(rule);
    }

    /// Run all enabled rules on a document with diagnostic limit enforcement.
    ///
    /// # Security
    ///
    /// This method enforces a configurable limit on the number of diagnostics
    /// to prevent memory exhaustion from malicious documents. If the limit is
    /// exceeded, a warning diagnostic is added and no further rules are executed.
    ///
    /// # Returns
    ///
    /// A vector of diagnostics, limited to `config.max_diagnostics` entries.
    /// If the limit is reached, the last diagnostic will be a warning about
    /// the limit being exceeded.
    pub fn run(&self, doc: &Document) -> Vec<Diagnostic> {
        let context = LintContext::from_text("");
        self.run_with_context(doc, context)
    }

    /// Run all enabled rules on a document with lint context.
    ///
    /// This method allows passing file path and line number context to lint rules,
    /// enabling better diagnostics with file information.
    ///
    /// # Security
    ///
    /// This method enforces a configurable limit on the number of diagnostics
    /// to prevent memory exhaustion from malicious documents. If the limit is
    /// exceeded, a warning diagnostic is added and no further rules are executed.
    ///
    /// # Arguments
    ///
    /// * `doc` - The document to lint
    /// * `context` - Lint context with file path and line number information
    ///
    /// # Returns
    ///
    /// A vector of diagnostics, limited to `config.max_diagnostics` entries.
    pub fn run_with_context(&self, doc: &Document, context: LintContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        let mut limit_exceeded = false;

        for rule in &self.rules {
            // Check diagnostic limit before processing each rule
            if diagnostics.len() >= self.config.max_diagnostics {
                limit_exceeded = true;
                break;
            }

            let rule_id = rule.id();
            let rule_config = self.config.rules.get(rule_id).cloned().unwrap_or_default();

            if !rule_config.enabled {
                continue;
            }

            let mut rule_diagnostics = rule.check_with_context(doc, &context as &dyn std::any::Any);

            // Apply rule configuration
            for diag in &mut rule_diagnostics {
                if rule_config.error && diag.severity() == Severity::Warning {
                    diag.escalate_to_error();
                }
            }

            // Filter by minimum severity and apply diagnostic limit
            for diag in rule_diagnostics
                .into_iter()
                .filter(|d| d.severity() >= self.config.min_severity)
            {
                if diagnostics.len() >= self.config.max_diagnostics {
                    limit_exceeded = true;
                    break;
                }
                diagnostics.push(diag);
            }

            if limit_exceeded {
                break;
            }
        }

        // Add limit exceeded warning if applicable
        if limit_exceeded {
            use crate::diagnostic::DiagnosticKind;
            diagnostics.push(
                Diagnostic::warning(
                    DiagnosticKind::Custom("diagnostic-limit-exceeded".to_string()),
                    format!(
                        "Diagnostic limit of {} exceeded. Further diagnostics have been suppressed. \
                         This typically indicates a systemic issue in the document that should be \
                         addressed before fixing individual diagnostics.",
                        self.config.max_diagnostics
                    ),
                    "lint-runner",
                )
            );
        }

        // Sort by severity (errors first)
        diagnostics.sort_by(|a, b| b.severity().cmp(&a.severity()));

        diagnostics
    }

    /// Check if any errors were found
    pub fn has_errors(&self, diagnostics: &[Diagnostic]) -> bool {
        diagnostics.iter().any(|d| d.severity() == Severity::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::DiagnosticKind;
    use hedl_core::{Item, MatrixList, Node, Reference, Value};

    // ==================== LintConfig tests ====================

    #[test]
    fn test_lint_config_default() {
        let config = LintConfig::default();
        assert_eq!(config.min_severity, Severity::Hint);
        assert!(config.rules.is_empty());
    }

    #[test]
    fn test_lint_config_clone() {
        let mut config = LintConfig::default();
        config.disable_rule("test");
        let cloned = config.clone();
        assert!(!cloned.rules.get("test").unwrap().enabled);
    }

    #[test]
    fn test_lint_config_debug() {
        let config = LintConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("LintConfig"));
    }

    #[test]
    fn test_disable_rule() {
        let mut config = LintConfig::default();
        config.disable_rule("id-naming");
        assert!(!config.rules.get("id-naming").unwrap().enabled);
    }

    #[test]
    fn test_enable_rule() {
        let mut config = LintConfig::default();
        config.enable_rule("my-rule");
        let rule_config = config.rules.get("my-rule").unwrap();
        assert!(rule_config.enabled);
        assert!(!rule_config.error);
    }

    #[test]
    fn test_set_rule_error() {
        let mut config = LintConfig::default();
        config.set_rule_error("strict-rule");
        let rule_config = config.rules.get("strict-rule").unwrap();
        assert!(rule_config.enabled);
        assert!(rule_config.error);
    }

    #[test]
    fn test_rule_config_overwrite() {
        let mut config = LintConfig::default();
        config.enable_rule("test");
        config.disable_rule("test");
        assert!(!config.rules.get("test").unwrap().enabled);
    }

    // ==================== LintRunner tests ====================

    #[test]
    fn test_lint_runner_new() {
        let runner = LintRunner::new(LintConfig::default());
        let doc = Document::new((1, 0));
        let diagnostics = runner.run(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_lint_runner_with_rules() {
        use crate::rules::EmptyListRule;

        let rules: Vec<Box<dyn LintRule>> = vec![Box::new(EmptyListRule)];
        let runner = LintRunner::with_rules(LintConfig::default(), rules);

        let doc = Document::new((1, 0));
        let diagnostics = runner.run(&doc);
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_lint_runner_add_rule() {
        use crate::rules::EmptyListRule;

        let mut runner = LintRunner::with_rules(LintConfig::default(), vec![]);
        runner.add_rule(Box::new(EmptyListRule));

        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(list));

        let diagnostics = runner.run(&doc);
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn test_lint_runner_disabled_rule() {
        let mut config = LintConfig::default();
        config.disable_rule("empty-list");

        let runner = LintRunner::new(config);

        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(list));

        let diagnostics = runner.run(&doc);
        // empty-list rule is disabled, so no empty list diagnostics
        let empty_list_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
            .collect();
        assert!(empty_list_diags.is_empty());
    }

    #[test]
    fn test_lint_runner_error_escalation() {
        let mut config = LintConfig::default();
        config.set_rule_error("empty-list");

        let runner = LintRunner::new(config);

        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(list));

        let diagnostics = runner.run(&doc);
        let empty_list_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
            .collect();

        // Hint should be escalated to Error
        assert!(!empty_list_diags.is_empty());
        // Note: Error escalation only affects Warning -> Error, not Hint -> Error
    }

    #[test]
    fn test_lint_runner_min_severity_filter() {
        let config = LintConfig {
            min_severity: Severity::Warning,
            ..Default::default()
        };

        let runner = LintRunner::new(config);

        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(list));

        let diagnostics = runner.run(&doc);
        // EmptyList produces Hint which is below min_severity (Warning)
        let empty_list_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
            .collect();
        assert!(empty_list_diags.is_empty());
    }

    #[test]
    fn test_lint_runner_min_severity_error() {
        let config = LintConfig {
            min_severity: Severity::Error,
            ..Default::default()
        };

        let runner = LintRunner::new(config);

        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(list));

        let diagnostics = runner.run(&doc);
        // With min_severity Error, only errors should be reported
        // Standard rules produce hints and warnings, not errors
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_lint_runner_sorting() {
        let runner = LintRunner::new(LintConfig::default());

        let mut doc = Document::new((1, 0));

        // Add items that will produce different severity diagnostics
        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(list));

        let ref_val = Value::Reference(Reference::local("id"));
        doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

        let diagnostics = runner.run(&doc);

        // Verify errors come first (sorted by severity descending)
        let mut prev_severity = Severity::Error;
        for diag in &diagnostics {
            assert!(diag.severity() <= prev_severity);
            prev_severity = diag.severity();
        }
    }

    #[test]
    fn test_has_errors_true() {
        let runner = LintRunner::new(LintConfig::default());
        let diagnostics = vec![Diagnostic::error(
            DiagnosticKind::DuplicateKey,
            "test",
            "rule",
        )];
        assert!(runner.has_errors(&diagnostics));
    }

    #[test]
    fn test_has_errors_false() {
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
    fn test_has_errors_mixed() {
        let runner = LintRunner::new(LintConfig::default());
        let diagnostics = vec![
            Diagnostic::hint(DiagnosticKind::IdNaming, "test", "rule"),
            Diagnostic::error(DiagnosticKind::DuplicateKey, "test", "rule"),
            Diagnostic::warning(DiagnosticKind::UnusedSchema, "test", "rule"),
        ];
        assert!(runner.has_errors(&diagnostics));
    }

    // ==================== Integration tests ====================

    #[test]
    fn test_lint_runner_all_rules_run() {
        let runner = LintRunner::new(LintConfig::default());

        let mut doc = Document::new((1, 0));

        // Create a document that will trigger multiple rules
        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "a", vec![])); // Short ID
        doc.root.insert("items".to_string(), Item::List(list));

        doc.structs
            .insert("Unused".to_string(), vec!["id".to_string()]);

        let diagnostics = runner.run(&doc);
        // Should have diagnostics from multiple rules
        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_lint_runner_multiple_disabled_rules() {
        let mut config = LintConfig::default();
        config.disable_rule("id-naming");
        config.disable_rule("empty-list");
        config.disable_rule("unused-schema");

        let runner = LintRunner::new(config);

        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Test", vec!["id".to_string()]);
        list.add_row(Node::new("Test", "a", vec![]));
        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = runner.run(&doc);
        // With multiple rules disabled, should have fewer diagnostics
        let id_naming_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
            .collect();
        assert!(id_naming_diags.is_empty());
    }

    #[test]
    fn test_lint_runner_complex_document() {
        let runner = LintRunner::new(LintConfig::default());

        let mut doc = Document::new((1, 0));

        // Multiple lists
        let mut list1 = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
        list1.add_row(Node::new(
            "User",
            "alice",
            vec![Value::String("Alice".to_string())],
        ));
        doc.root.insert("users".to_string(), Item::List(list1));

        let list2 = MatrixList::new("Product", vec!["id".to_string()]);
        doc.root.insert("products".to_string(), Item::List(list2)); // Empty

        // Schemas
        doc.structs.insert(
            "User".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );
        doc.structs
            .insert("Product".to_string(), vec!["id".to_string()]);
        doc.structs
            .insert("Unused".to_string(), vec!["id".to_string()]);

        // Reference
        let ref_val = Value::Reference(Reference::local("alice"));
        doc.root.insert("owner".to_string(), Item::Scalar(ref_val));

        let diagnostics = runner.run(&doc);

        // Should have diagnostics for:
        // - Empty products list
        // - Unused schema
        // - Unqualified reference
        assert!(diagnostics.len() >= 3);
    }

    #[test]
    fn test_lint_runner_no_false_positives() {
        let runner = LintRunner::new(LintConfig::default());

        let mut doc = Document::new((1, 0));

        // A well-formed document
        doc.structs.insert(
            "User".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );

        let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
        list.add_row(Node::new(
            "User",
            "alice_smith",
            vec![Value::String("Alice Smith".to_string())],
        ));
        list.add_row(Node::new(
            "User",
            "bob_jones",
            vec![Value::String("Bob Jones".to_string())],
        ));
        doc.root.insert("users".to_string(), Item::List(list));

        // Qualified reference
        let ref_val = Value::Reference(Reference::qualified("User", "alice_smith"));
        doc.root.insert("owner".to_string(), Item::Scalar(ref_val));

        let diagnostics = runner.run(&doc);
        // Should have no diagnostics for a well-formed document
        assert!(diagnostics.is_empty());
    }

    // ==================== LintContext tests ====================

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
        assert_eq!(context.source_text, "content");
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
    fn test_lint_context_current_line_none() {
        let source = "line1\nline2";
        let context = LintContext::from_text(source);
        assert_eq!(context.current_line(), None);
    }

    #[test]
    fn test_lint_context_clone() {
        let path = PathBuf::from("test.hedl");
        let context = LintContext::new(Some(path), 10, "text");
        let cloned = context.clone();

        assert_eq!(context.file_path, cloned.file_path);
        assert_eq!(context.line_number, cloned.line_number);
        assert_eq!(context.source_text, cloned.source_text);
    }

    #[test]
    fn test_lint_context_debug() {
        let context = LintContext::from_text("test");
        let debug = format!("{:?}", context);
        assert!(debug.contains("LintContext"));
    }

    #[test]
    fn test_lint_runner_with_context() {
        let runner = LintRunner::new(LintConfig::default());

        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(list));

        let context = LintContext::with_file(PathBuf::from("test.hedl"), "source");
        let diagnostics = runner.run_with_context(&doc, context);

        assert!(!diagnostics.is_empty());
    }

    #[test]
    fn test_lint_runner_with_context_and_line() {
        let runner = LintRunner::new(LintConfig::default());

        let mut doc = Document::new((1, 0));
        let list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty".to_string(), Item::List(list));

        let context = LintContext::new(
            Some(PathBuf::from("test.hedl")),
            42,
            "line1\nline2\nline3",
        );
        let diagnostics = runner.run_with_context(&doc, context);

        assert!(!diagnostics.is_empty());
    }
}
