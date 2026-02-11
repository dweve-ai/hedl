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

//! Integration tests for lint rules

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_lint::{lint, DiagnosticKind, LintConfig, LintRunner, Severity};
use std::collections::BTreeMap;

// =============================================================================
// Deep nesting and recursion limit tests
// =============================================================================

#[test]
fn test_moderate_nesting_allowed() {
    let mut doc = Document::new((2, 0));

    // Create nested objects within reasonable limit
    let mut current = BTreeMap::new();
    for i in 0..50 {
        let mut next = BTreeMap::new();
        let mut list = MatrixList::new("Level", vec!["id".to_string()]);
        list.add_row(Node::new("Level", format!("level_{i}"), vec![]));
        next.insert("inner".to_string(), Item::List(list));
        current.insert(format!("level_{i}"), Item::Object(next.clone()));
        current = next;
    }

    doc.root = current;

    let diagnostics = lint(&doc);
    // Should not have max-depth-exceeded warning for moderate nesting
    assert!(!diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::Custom(s) if s == "max-depth-exceeded")));
}

#[test]
fn test_nested_children_within_limit() {
    let mut doc = Document::new((2, 0));

    // Create nested node children within reasonable limit
    let mut list = MatrixList::new("Root", vec!["id".to_string()]);
    let root = Node::new("Root", "root", vec![]);

    // Create a few levels of nesting
    let mut parent = root.clone();
    for i in 0..10 {
        let mut child = Node::new("Child", format!("child_{i}"), vec![]);
        if i < 9 {
            child.add_child(
                "Child",
                Node::new("Child", format!("child_{i}_inner"), vec![]),
            );
        }
        parent.add_child("Child", child);
    }

    list.add_row(parent);
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);
    // Should work fine with moderate nesting
    let depth_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::Custom(s) if s.contains("depth")))
        .collect();
    assert!(depth_warnings.is_empty());
}

// =============================================================================
// Unused schema with complex type usage tests
// =============================================================================

#[test]
fn test_unused_schema_used_in_nested_reference() {
    let mut doc = Document::new((2, 0));

    // Define schemas
    doc.structs
        .insert("User".to_string(), vec!["id".to_string()]);
    doc.structs.insert(
        "Post".to_string(),
        vec!["id".to_string(), "author".to_string()],
    );

    // User is used in Post's reference field
    let mut post_list = MatrixList::new("Post", vec!["id".to_string(), "author".to_string()]);
    post_list.add_row(Node::new(
        "Post",
        "post1",
        vec![Value::Reference(Reference::qualified("User", "alice"))],
    ));
    doc.root.insert("posts".to_string(), Item::List(post_list));

    let diagnostics = lint(&doc);

    // Neither User nor Post should be reported as unused
    let unused_schemas: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert!(unused_schemas.is_empty());
}

#[test]
fn test_unused_schema_multiple_levels_deep() {
    let mut doc = Document::new((2, 0));

    // Define schemas
    doc.structs.insert("A".to_string(), vec!["id".to_string()]);
    doc.structs.insert("B".to_string(), vec!["id".to_string()]);
    doc.structs.insert("C".to_string(), vec!["id".to_string()]);
    doc.structs.insert("D".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]);

    // Create hierarchy: A > B > C > D
    let mut a_list = MatrixList::new("A", vec!["id".to_string()]);
    let mut a_node = Node::new("A", "a1", vec![]);

    let mut b_node = Node::new("B", "b1", vec![]);
    let mut c_node = Node::new("C", "c1", vec![]);
    let d_node = Node::new("D", "d1", vec![]);

    c_node.add_child("D", d_node);
    b_node.add_child("C", c_node);
    a_node.add_child("B", b_node);
    a_list.add_row(a_node);

    doc.root.insert("items".to_string(), Item::List(a_list));

    let diagnostics = lint(&doc);

    // Only "Unused" should be reported
    let unused_schemas: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert_eq!(unused_schemas.len(), 1);
    assert!(unused_schemas[0].message().contains("Unused"));
}

#[test]
fn test_unused_schema_used_in_multiple_contexts() {
    let mut doc = Document::new((2, 0));

    doc.structs
        .insert("User".to_string(), vec!["id".to_string()]);

    // Used as list type
    let mut user_list = MatrixList::new("User", vec!["id".to_string()]);
    user_list.add_row(Node::new("User", "alice", vec![]));
    doc.root.insert("users".to_string(), Item::List(user_list));

    // Used in reference
    doc.root.insert(
        "admin".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "alice"))),
    );

    let diagnostics = lint(&doc);

    // User should not be reported as unused
    let unused_schemas: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert!(unused_schemas.is_empty());
}

// =============================================================================
// Edge cases for ID naming
// =============================================================================

#[test]
fn test_id_naming_with_special_characters() {
    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "user-alice", vec![])); // Hyphen
    list.add_row(Node::new("Item", "user_bob", vec![])); // Underscore
    list.add_row(Node::new("Item", "user.charlie", vec![])); // Dot
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    // None of these should trigger id-naming warnings
    let id_naming_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .collect();

    assert!(id_naming_diags.is_empty());
}

#[test]
fn test_id_naming_only_underscores() {
    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "___", vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    // Underscores only (no digits) should not trigger numeric check
    // But might trigger short ID check depending on length
    let id_naming_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .filter(|d| d.message().contains("numbers"))
        .collect();

    assert!(id_naming_diags.is_empty());
}

#[test]
fn test_id_naming_mixed_case() {
    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "UserAlice", vec![]));
    list.add_row(Node::new("Item", "user_Bob", vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    // Mixed case should not trigger warnings
    let id_naming_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .collect();

    assert!(id_naming_diags.is_empty());
}

// =============================================================================
// Diagnostic limit enforcement tests
// =============================================================================

#[test]
fn test_diagnostic_limit_with_many_short_ids() {
    let config = LintConfig {
        max_diagnostics: 10,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);

    // Add 100 nodes with short IDs
    for i in 0..100 {
        list.add_row(Node::new("Item", format!("{}", i % 10), vec![]));
    }
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Should stop at limit (might be 10 or 11 with limit-exceeded warning)
    assert!(diagnostics.len() <= 11);

    // Should have diagnostic-limit-exceeded warning OR be at the max
    let has_limit_warning = diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::Custom(s) if s == "diagnostic-limit-exceeded"));
    let at_limit = diagnostics.len() == 10;

    assert!(has_limit_warning || at_limit);
}

#[test]
fn test_diagnostic_limit_zero() {
    let config = LintConfig {
        max_diagnostics: 0,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "a", vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Should have limit exceeded warning only
    assert!(diagnostics.len() <= 1);
}

#[test]
fn test_diagnostic_limit_with_multiple_rule_types() {
    let config = LintConfig {
        max_diagnostics: 5,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));

    // Create violations for multiple rules
    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    for i in 0..10 {
        list.add_row(Node::new("Test", format!("{i}"), vec![]));
    }
    doc.root.insert("items".to_string(), Item::List(list));

    // Add empty lists
    for i in 0..5 {
        doc.root.insert(
            format!("empty_{i}"),
            Item::List(MatrixList::new("Empty", vec!["id".to_string()])),
        );
    }

    // Add unused schemas
    for i in 0..5 {
        doc.structs
            .insert(format!("Unused{i}"), vec!["id".to_string()]);
    }

    let diagnostics = runner.run(&doc);

    // Should stop at limit
    assert!(diagnostics.len() <= 6); // 5 + limit exceeded warning
}

// =============================================================================
// Empty list in different contexts
// =============================================================================

#[test]
fn test_empty_list_nested_in_objects() {
    let mut doc = Document::new((2, 0));

    let mut nested = BTreeMap::new();
    let empty_list = MatrixList::new("Empty", vec!["id".to_string()]);
    nested.insert("empty".to_string(), Item::List(empty_list));

    doc.root
        .insert("container".to_string(), Item::Object(nested));

    let diagnostics = lint(&doc);

    let empty_list_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();

    assert_eq!(empty_list_diags.len(), 1);
}

#[test]
fn test_empty_list_multiple_levels() {
    let mut doc = Document::new((2, 0));

    // Level 1
    let mut level1 = BTreeMap::new();
    let empty1 = MatrixList::new("Empty1", vec!["id".to_string()]);
    level1.insert("empty1".to_string(), Item::List(empty1));

    // Level 2
    let mut level2 = BTreeMap::new();
    let empty2 = MatrixList::new("Empty2", vec!["id".to_string()]);
    level2.insert("empty2".to_string(), Item::List(empty2));
    level1.insert("nested".to_string(), Item::Object(level2));

    doc.root
        .insert("container".to_string(), Item::Object(level1));

    let diagnostics = lint(&doc);

    let empty_list_diags: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();

    assert_eq!(empty_list_diags.len(), 2);
}

// =============================================================================
// Unqualified reference edge cases
// =============================================================================

#[test]
fn test_unqualified_reference_deeply_nested() {
    let mut doc = Document::new((2, 0));

    let mut level1 = BTreeMap::new();
    let mut level2 = BTreeMap::new();

    level2.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("some_id"))),
    );
    level1.insert("nested".to_string(), Item::Object(level2));
    doc.root
        .insert("container".to_string(), Item::Object(level1));

    let diagnostics = lint(&doc);

    let unqualified_refs: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();

    assert_eq!(unqualified_refs.len(), 1);
}

#[test]
fn test_qualified_reference_no_warning() {
    let mut doc = Document::new((2, 0));

    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "alice"))),
    );

    let diagnostics = lint(&doc);

    let unqualified_refs: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();

    assert!(unqualified_refs.is_empty());
}

#[test]
fn test_non_reference_scalars_no_warning() {
    let mut doc = Document::new((2, 0));

    doc.root.insert(
        "string".to_string(),
        Item::Scalar(Value::String("test".to_string().into())),
    );
    doc.root
        .insert("int".to_string(), Item::Scalar(Value::Int(42)));
    doc.root
        .insert("float".to_string(), Item::Scalar(Value::Float(3.5)));
    doc.root
        .insert("bool".to_string(), Item::Scalar(Value::Bool(true)));

    let diagnostics = lint(&doc);

    let unqualified_refs: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
        .collect();

    assert!(unqualified_refs.is_empty());
}

// =============================================================================
// Min severity filtering tests
// =============================================================================

#[test]
fn test_min_severity_filters_hints() {
    let config = LintConfig {
        min_severity: Severity::Warning,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));

    // Create hint: short ID
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "a", vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Should not have hints
    assert!(!diagnostics.iter().any(|d| d.severity() == Severity::Hint));
}

#[test]
fn test_min_severity_allows_warnings_and_errors() {
    let config = LintConfig {
        min_severity: Severity::Warning,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));

    // Create warning: unqualified reference
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("id"))),
    );

    // Create warning: unused schema
    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]);

    let diagnostics = runner.run(&doc);

    // Should have warnings
    assert!(diagnostics
        .iter()
        .any(|d| d.severity() == Severity::Warning));
}

// =============================================================================
// Rule escalation edge cases
// =============================================================================

#[test]
fn test_escalate_warning_to_error() {
    let mut config = LintConfig::default();
    config.set_rule_error("unused-schema");

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]);

    let diagnostics = runner.run(&doc);

    let escalated: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert!(!escalated.is_empty());
    assert!(escalated.iter().all(|d| d.severity() == Severity::Error));
}

#[test]
fn test_escalate_multiple_rules() {
    let mut config = LintConfig::default();
    config.set_rule_error("id-naming");
    config.set_rule_error("empty-list");

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "a", vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    doc.root.insert(
        "empty".to_string(),
        Item::List(MatrixList::new("Empty", vec!["id".to_string()])),
    );

    let diagnostics = runner.run(&doc);

    // All should be errors
    let id_naming: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
        .collect();
    let empty_list: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
        .collect();

    assert!(id_naming.iter().all(|d| d.severity() == Severity::Error));
    assert!(empty_list.iter().all(|d| d.severity() == Severity::Error));
}
