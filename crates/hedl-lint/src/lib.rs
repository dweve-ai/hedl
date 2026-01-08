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

//! HEDL Linting
//!
//! Provides extensible linting and best practices validation for HEDL documents.
//!
//! ## Quick Start
//!
//! ```rust
//! use hedl_core::Document;
//! use hedl_lint::{lint, Severity};
//!
//! let doc = Document::new((1, 0));
//! let diagnostics = lint(&doc);
//!
//! for diag in &diagnostics {
//!     if diag.severity() == Severity::Error {
//!         eprintln!("{}", diag);
//!     }
//! }
//! ```
//!
//! ## Custom Configuration
//!
//! ```rust
//! use hedl_lint::{lint_with_config, LintConfig, Severity};
//! use hedl_core::Document;
//!
//! let doc = Document::new((1, 0));
//!
//! let mut config = LintConfig::default();
//! config.disable_rule("id-naming");
//! config.set_rule_error("unused-schema");
//! config.min_severity = Severity::Warning;
//!
//! let diagnostics = lint_with_config(&doc, config);
//! ```
//!
//! ## Custom Rules
//!
//! ```rust
//! use hedl_lint::{LintRule, Diagnostic, DiagnosticKind, LintRunner, LintConfig};
//! use hedl_core::Document;
//!
//! struct MyCustomRule;
//!
//! impl LintRule for MyCustomRule {
//!     fn id(&self) -> &str { "my-custom-rule" }
//!     fn description(&self) -> &str { "Custom validation logic" }
//!     fn check(&self, doc: &Document) -> Vec<Diagnostic> {
//!         vec![]
//!     }
//! }
//!
//! let mut runner = LintRunner::new(LintConfig::default());
//! runner.add_rule(Box::new(MyCustomRule));
//!
//! let doc = Document::new((1, 0));
//! let diagnostics = runner.run(&doc);
//! ```
//!
//! ## Using LintContext
//!
//! ```rust
//! use hedl_lint::{LintContext, LintRunner, LintConfig};
//! use hedl_core::Document;
//! use std::path::PathBuf;
//!
//! let doc = Document::new((1, 0));
//! let runner = LintRunner::new(LintConfig::default());
//!
//! // Create context with file path and source text
//! let context = LintContext::with_file(
//!     PathBuf::from("data.hedl"),
//!     "source content here"
//! ).with_line(42);
//!
//! let diagnostics = runner.run_with_context(&doc, context);
//! ```

mod diagnostic;
mod rules;
mod runner;

pub use diagnostic::{Diagnostic, DiagnosticKind, Severity};
pub use rules::{LintRule, RuleConfig};
pub use runner::{LintConfig, LintContext, LintRunner};

use hedl_core::Document;

/// Run all default lint rules on a document
pub fn lint(doc: &Document) -> Vec<Diagnostic> {
    let runner = LintRunner::new(LintConfig::default());
    runner.run(doc)
}

/// Run lint with custom configuration
pub fn lint_with_config(doc: &Document, config: LintConfig) -> Vec<Diagnostic> {
    let runner = LintRunner::new(config);
    runner.run(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Item, MatrixList, Node, Reference, Value};

    #[test]
    fn test_lint_empty_document() {
        let doc = Document::new((1, 0));
        let diagnostics = lint(&doc);
        assert_eq!(diagnostics.len(), 0);
    }

    #[test]
    fn test_lint_short_ids() {
        let mut doc = Document::new((1, 0));

        // Create a matrix list with short IDs
        let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
        let node1 = Node::new("User", "a", vec![Value::String("Alice".to_string())]);
        let node2 = Node::new("User", "b", vec![Value::String("Bob".to_string())]);
        list.add_row(node1);
        list.add_row(node2);

        doc.root.insert("users".to_string(), Item::List(list));

        let diagnostics = lint(&doc);

        // Should have 2 hints for short IDs
        let short_id_hints: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
            .collect();
        assert_eq!(short_id_hints.len(), 2);
        assert!(short_id_hints.iter().all(|d| d.severity() == Severity::Hint));
    }

    #[test]
    fn test_lint_numeric_ids() {
        let mut doc = Document::new((1, 0));

        let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
        let node = Node::new("Item", "123", vec![Value::Int(100)]);
        list.add_row(node);

        doc.root.insert("items".to_string(), Item::List(list));

        let diagnostics = lint(&doc);

        let numeric_id_hints: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
            .collect();
        assert!(!numeric_id_hints.is_empty());
    }

    #[test]
    fn test_lint_unused_schema() {
        let mut doc = Document::new((1, 0));

        // Define a schema that's never used
        doc.structs.insert(
            "UnusedType".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );

        // Add a used schema
        doc.structs
            .insert("UsedType".to_string(), vec!["id".to_string()]);
        let mut list = MatrixList::new("UsedType", vec!["id".to_string()]);
        list.add_row(Node::new("UsedType", "test", vec![]));
        doc.root.insert("used".to_string(), Item::List(list));

        let diagnostics = lint(&doc);

        let unused_schema_warnings: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
            .collect();
        assert_eq!(unused_schema_warnings.len(), 1);
        assert_eq!(unused_schema_warnings[0].severity(), Severity::Warning);
    }

    #[test]
    fn test_lint_empty_list() {
        let mut doc = Document::new((1, 0));

        let list = MatrixList::new("EmptyType", vec!["id".to_string()]);
        doc.root.insert("empty_list".to_string(), Item::List(list));

        let diagnostics = lint(&doc);

        let empty_list_hints: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
            .collect();
        assert_eq!(empty_list_hints.len(), 1);
        assert_eq!(empty_list_hints[0].severity(), Severity::Hint);
    }

    #[test]
    fn test_lint_unqualified_reference() {
        let mut doc = Document::new((1, 0));

        // Add an unqualified reference in key-value context
        let ref_value = Value::Reference(Reference::local("some_id"));
        doc.root
            .insert("ref_field".to_string(), Item::Scalar(ref_value));

        let diagnostics = lint(&doc);

        let unqualified_ref_warnings: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
            .collect();
        assert_eq!(unqualified_ref_warnings.len(), 1);
        assert_eq!(unqualified_ref_warnings[0].severity(), Severity::Warning);
        assert!(unqualified_ref_warnings[0].suggestion().is_some());
    }

    #[test]
    fn test_lint_config_disable_rule() {
        let mut doc = Document::new((1, 0));

        let list = MatrixList::new("EmptyType", vec!["id".to_string()]);
        doc.root.insert("empty_list".to_string(), Item::List(list));

        // Disable the empty-list rule
        let mut config = LintConfig::default();
        config.disable_rule("empty-list");

        let diagnostics = lint_with_config(&doc, config);

        let empty_list_hints: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
            .collect();
        assert_eq!(empty_list_hints.len(), 0);
    }

    #[test]
    fn test_diagnostic_display() {
        let diag = Diagnostic::warning(DiagnosticKind::UnusedSchema, "Test message", "test-rule");
        let display = format!("{}", diag);
        assert!(display.contains("warning"));
        assert!(display.contains("Test message"));
        assert!(display.contains("test-rule"));
    }

    #[test]
    fn test_diagnostic_with_line() {
        let diag = Diagnostic::error(
            DiagnosticKind::DuplicateKey,
            "Duplicate key found",
            "dup-key",
        )
        .with_line(42);

        let display = format!("{}", diag);
        assert!(display.contains("line 42"));
    }
}
