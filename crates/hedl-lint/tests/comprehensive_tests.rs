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

//! Comprehensive tests for hedl-lint
//!
//! Tests for all lint rules and configurations.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_lint::{lint, lint_with_config, Diagnostic, DiagnosticKind, LintConfig, Severity};
use std::collections::BTreeMap;

// =============================================================================
// ID Naming Rule Tests
// =============================================================================

#[test]
fn test_single_char_id_hint() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new("Item", "a", vec![Value::Int(1)]));
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let id_hints: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .collect();

    assert!(!id_hints.is_empty());
    assert!(id_hints.iter().all(|d| d.severity() == Severity::Hint));
}

#[test]
fn test_numeric_only_id_hint() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new("Item", "123", vec![Value::Int(1)]));
    list.add_row(Node::new("Item", "456", vec![Value::Int(2)]));
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let id_hints: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .filter(|d| d.message().contains("only numbers"))
        .collect();

    assert_eq!(id_hints.len(), 2);
}

#[test]
fn test_good_id_no_hint() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list.add_row(Node::new(
        "User",
        "user_alice",
        vec![Value::String("Alice".to_string())],
    ));
    list.add_row(Node::new(
        "User",
        "user_bob",
        vec![Value::String("Bob".to_string())],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let id_hints: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .collect();

    assert!(id_hints.is_empty());
}

#[test]
fn test_nested_child_ids_checked() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    let mut parent = Node::new("Parent", "parent_1", vec![]);
    let child = Node::new("Child", "x", vec![]); // Short ID in child
    parent.children.insert("Child".to_string(), vec![child]);

    list.add_row(parent);
    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let id_hints: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .filter(|d| d.message().contains("'x'"))
        .collect();

    assert!(!id_hints.is_empty());
}

// =============================================================================
// Unused Schema Rule Tests
// =============================================================================

#[test]
fn test_unused_schema_warning() {
    let mut doc = Document::new((1, 0));

    // Define a schema that's never used
    doc.structs
        .insert("UnusedType".to_string(), vec!["id".to_string()]);

    // Define and use another schema
    doc.structs
        .insert("UsedType".to_string(), vec!["id".to_string()]);
    let mut list = MatrixList::new("UsedType", vec!["id".to_string()]);
    list.add_row(Node::new("UsedType", "item1", vec![]));
    doc.root.insert("used_items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let unused_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert_eq!(unused_warnings.len(), 1);
    assert!(unused_warnings[0].message().contains("UnusedType"));
    assert_eq!(unused_warnings[0].severity(), Severity::Warning);
}

#[test]
fn test_used_schema_no_warning() {
    let mut doc = Document::new((1, 0));

    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list.add_row(Node::new(
        "User",
        "user1",
        vec![Value::String("Alice".to_string())],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let unused_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert!(unused_warnings.is_empty());
}

#[test]
fn test_multiple_unused_schemas() {
    let mut doc = Document::new((1, 0));

    doc.structs
        .insert("Unused1".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Unused2".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Unused3".to_string(), vec!["id".to_string()]);

    let diagnostics = lint(&doc);

    let unused_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert_eq!(unused_warnings.len(), 3);
}

#[test]
fn test_no_schemas_no_warning() {
    let doc = Document::new((1, 0));
    let diagnostics = lint(&doc);

    let unused_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert!(unused_warnings.is_empty());
}

// =============================================================================
// Empty List Rule Tests
// =============================================================================

#[test]
fn test_empty_list_hint() {
    let mut doc = Document::new((1, 0));

    let list = MatrixList::new("EmptyType", vec!["id".to_string()]);
    doc.root.insert("empty_items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let empty_hints: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();

    assert_eq!(empty_hints.len(), 1);
    assert!(empty_hints[0].message().contains("empty_items"));
    assert_eq!(empty_hints[0].severity(), Severity::Hint);
}

#[test]
fn test_non_empty_list_no_hint() {
    let mut doc = Document::new((1, 0));

    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "item1", vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let empty_hints: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();

    assert!(empty_hints.is_empty());
}

#[test]
fn test_nested_empty_list_hint() {
    let mut doc = Document::new((1, 0));

    let mut outer = BTreeMap::new();
    let list = MatrixList::new("EmptyNested", vec!["id".to_string()]);
    outer.insert("nested_empty".to_string(), Item::List(list));
    doc.root
        .insert("container".to_string(), Item::Object(outer));

    let diagnostics = lint(&doc);

    let empty_hints: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();

    assert_eq!(empty_hints.len(), 1);
}

// =============================================================================
// Unqualified Reference Rule Tests
// =============================================================================

#[test]
fn test_unqualified_kv_reference_warning() {
    let mut doc = Document::new((1, 0));

    doc.root.insert(
        "ref_field".to_string(),
        Item::Scalar(Value::Reference(Reference::local("some_id"))),
    );

    let diagnostics = lint(&doc);

    let unqualified_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();

    assert_eq!(unqualified_warnings.len(), 1);
    assert_eq!(unqualified_warnings[0].severity(), Severity::Warning);
    assert!(unqualified_warnings[0].suggestion().is_some());
}

#[test]
fn test_qualified_kv_reference_no_warning() {
    let mut doc = Document::new((1, 0));

    doc.root.insert(
        "ref_field".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "user1"))),
    );

    let diagnostics = lint(&doc);

    let unqualified_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();

    assert!(unqualified_warnings.is_empty());
}

#[test]
fn test_nested_unqualified_reference_warning() {
    let mut doc = Document::new((1, 0));

    let mut inner = BTreeMap::new();
    inner.insert(
        "nested_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target"))),
    );
    doc.root.insert("outer".to_string(), Item::Object(inner));

    let diagnostics = lint(&doc);

    let unqualified_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();

    assert_eq!(unqualified_warnings.len(), 1);
}

#[test]
fn test_multiple_unqualified_references() {
    let mut doc = Document::new((1, 0));

    doc.root.insert(
        "ref1".to_string(),
        Item::Scalar(Value::Reference(Reference::local("id1"))),
    );
    doc.root.insert(
        "ref2".to_string(),
        Item::Scalar(Value::Reference(Reference::local("id2"))),
    );
    doc.root.insert(
        "ref3".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("Type", "id3"))),
    );

    let diagnostics = lint(&doc);

    let unqualified_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();

    // Only 2 unqualified references
    assert_eq!(unqualified_warnings.len(), 2);
}

// =============================================================================
// Config Tests
// =============================================================================

#[test]
fn test_disable_specific_rule() {
    let mut doc = Document::new((1, 0));

    let list = MatrixList::new("EmptyType", vec!["id".to_string()]);
    doc.root.insert("empty_items".to_string(), Item::List(list));

    // Disable empty-list rule
    let mut config = LintConfig::default();
    config.disable_rule("empty-list");

    let diagnostics = lint_with_config(&doc, config);

    let empty_hints: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();

    assert!(empty_hints.is_empty());
}

#[test]
fn test_disable_multiple_rules() {
    let mut doc = Document::new((1, 0));

    // Create conditions for multiple rules
    let list = MatrixList::new("EmptyType", vec!["id".to_string()]);
    doc.root.insert("empty".to_string(), Item::List(list));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("id"))),
    );

    let mut config = LintConfig::default();
    config.disable_rule("empty-list");
    config.disable_rule("unqualified-kv-ref");

    let diagnostics = lint_with_config(&doc, config);

    let empty_hints: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();
    let unqualified_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();

    assert!(empty_hints.is_empty());
    assert!(unqualified_warnings.is_empty());
}

#[test]
fn test_default_config_all_rules_enabled() {
    let mut doc = Document::new((1, 0));

    // Trigger multiple rules
    let list = MatrixList::new("EmptyType", vec!["id".to_string()]);
    doc.root.insert("empty".to_string(), Item::List(list));
    doc.structs
        .insert("UnusedType".to_string(), vec!["id".to_string()]);

    let diagnostics = lint(&doc);

    // Should have diagnostics from both rules
    let empty_hints: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();
    let unused_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert!(!empty_hints.is_empty());
    assert!(!unused_warnings.is_empty());
}

// =============================================================================
// Diagnostic Tests
// =============================================================================

#[test]
fn test_diagnostic_hint_creation() {
    let diag = Diagnostic::hint(DiagnosticKind::IdNaming, "Test hint message", "test-rule");

    assert_eq!(diag.severity(), Severity::Hint);
    assert!(diag.message().contains("Test hint message"));
    assert_eq!(diag.rule_id(), "test-rule");
}

#[test]
fn test_diagnostic_warning_creation() {
    let diag = Diagnostic::warning(
        DiagnosticKind::UnusedSchema,
        "Test warning message",
        "test-rule",
    );

    assert_eq!(diag.severity(), Severity::Warning);
    assert!(diag.message().contains("Test warning message"));
}

#[test]
fn test_diagnostic_error_creation() {
    let diag = Diagnostic::error(
        DiagnosticKind::DuplicateKey,
        "Test error message",
        "test-rule",
    );

    assert_eq!(diag.severity(), Severity::Error);
    assert!(diag.message().contains("Test error message"));
}

#[test]
fn test_diagnostic_with_line() {
    let diag = Diagnostic::warning(DiagnosticKind::UnusedSchema, "Test", "test").with_line(42);

    assert_eq!(diag.line(), Some(42));
}

#[test]
fn test_diagnostic_with_suggestion() {
    let diag = Diagnostic::warning(DiagnosticKind::UnqualifiedKvReference, "Test", "test")
        .with_suggestion("Use @Type:id".to_string());

    assert_eq!(diag.suggestion(), Some("Use @Type:id"));
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
fn test_diagnostic_display_with_line() {
    let diag = Diagnostic::error(DiagnosticKind::DuplicateKey, "Duplicate found", "dup-key")
        .with_line(100);

    let display = format!("{}", diag);
    assert!(display.contains("line 100"));
}

// =============================================================================
// Clean Document Tests
// =============================================================================

#[test]
fn test_empty_document_no_diagnostics() {
    let doc = Document::new((1, 0));
    let diagnostics = lint(&doc);
    assert!(diagnostics.is_empty());
}

#[test]
fn test_well_formed_document_minimal_diagnostics() {
    let mut doc = Document::new((1, 0));

    // Use good IDs and qualified references
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list.add_row(Node::new(
        "User",
        "user_alice",
        vec![Value::String("Alice".to_string())],
    ));
    list.add_row(Node::new(
        "User",
        "user_bob",
        vec![Value::String("Bob".to_string())],
    ));
    doc.root.insert("users".to_string(), Item::List(list));

    doc.root.insert(
        "admin".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "user_alice"))),
    );

    let diagnostics = lint(&doc);

    // Should have no warnings or errors (only possible hints)
    let warnings_and_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity() != Severity::Hint)
        .collect();

    assert!(warnings_and_errors.is_empty());
}

// =============================================================================
// Combination Tests
// =============================================================================

#[test]
fn test_multiple_rule_violations() {
    let mut doc = Document::new((1, 0));

    // Trigger ID naming rule
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "a", vec![])); // Short ID

    doc.root.insert("items".to_string(), Item::List(list));

    // Trigger unused schema rule
    doc.structs
        .insert("UnusedType".to_string(), vec!["id".to_string()]);

    // Trigger unqualified reference rule
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target"))),
    );

    let diagnostics = lint(&doc);

    // Should have diagnostics from multiple rules
    assert!(diagnostics.len() >= 3);

    // Check each type is present
    assert!(diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::IdNaming)));
    assert!(diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema)));
    assert!(diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference)));
}

#[test]
fn test_deeply_nested_structure() {
    let mut doc = Document::new((1, 0));

    let mut level3 = BTreeMap::new();
    level3.insert(
        "deep_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("deep"))),
    );

    let mut level2 = BTreeMap::new();
    level2.insert("level3".to_string(), Item::Object(level3));

    let mut level1 = BTreeMap::new();
    level1.insert("level2".to_string(), Item::Object(level2));

    doc.root.insert("level1".to_string(), Item::Object(level1));

    let diagnostics = lint(&doc);

    // Should find the deeply nested unqualified reference
    let unqualified_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();

    assert!(!unqualified_warnings.is_empty());
}
