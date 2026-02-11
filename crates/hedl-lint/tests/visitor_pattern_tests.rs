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

//! Comprehensive tests for visitor pattern and single-pass execution

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_lint::{DiagnosticKind, LintConfig, LintRunner, Severity};
use std::collections::BTreeMap;

// =============================================================================
// Single-pass vs sequential equivalence tests
// =============================================================================

#[test]
fn test_single_pass_sequential_equivalence_complex() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Complex document with all violation types
    doc.structs
        .insert("User".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Post".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("UnusedType".to_string(), vec!["id".to_string()]);

    let mut user_list = MatrixList::new("User", vec!["id".to_string()]);
    user_list.add_row(Node::new("User", "a", vec![])); // Short ID
    user_list.add_row(Node::new("User", "123", vec![])); // Numeric ID
    user_list.add_row(Node::new("User", "alice_smith", vec![])); // Good ID
    doc.root.insert("users".to_string(), Item::List(user_list));

    let post_list = MatrixList::new("Post", vec!["id".to_string()]);
    doc.root.insert("posts".to_string(), Item::List(post_list)); // Empty list

    doc.root.insert(
        "admin".to_string(),
        Item::Scalar(Value::Reference(Reference::local("alice"))), // Unqualified ref
    );

    let sequential = runner.run(&doc);
    let single_pass = runner.run_single_pass(&doc);

    // Should produce same diagnostic counts
    assert_eq!(sequential.len(), single_pass.len());

    // Should have same diagnostic kinds
    use std::collections::HashMap;
    let count_by_kind = |diagnostics: &[hedl_lint::Diagnostic]| {
        let mut counts = HashMap::new();
        for diag in diagnostics {
            let key = format!("{:?}", diag.kind());
            *counts.entry(key).or_insert(0) += 1;
        }
        counts
    };

    let seq_counts = count_by_kind(&sequential);
    let sp_counts = count_by_kind(&single_pass);

    assert_eq!(seq_counts, sp_counts);
}

#[test]
fn test_single_pass_preserves_severity() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Create violations of different severities
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "a", vec![])); // Hint
    doc.root.insert("items".to_string(), Item::List(list));

    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]); // Warning

    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("id"))), // Warning
    );

    let sequential = runner.run(&doc);
    let single_pass = runner.run_single_pass(&doc);

    // Count severities
    let count_severities = |diagnostics: &[hedl_lint::Diagnostic]| {
        let mut hints = 0;
        let mut warnings = 0;
        let mut errors = 0;
        for diag in diagnostics {
            match diag.severity() {
                Severity::Hint => hints += 1,
                Severity::Warning => warnings += 1,
                Severity::Error => errors += 1,
            }
        }
        (hints, warnings, errors)
    };

    let seq_counts = count_severities(&sequential);
    let sp_counts = count_severities(&single_pass);

    assert_eq!(seq_counts, sp_counts);
}

#[test]
fn test_single_pass_respects_rule_disable() {
    let mut config = LintConfig::default();
    config.disable_rule("id-naming");
    config.disable_rule("empty-list");

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "a", vec![])); // Would trigger id-naming
    doc.root.insert("items".to_string(), Item::List(list));

    let empty = MatrixList::new("Empty", vec!["id".to_string()]);
    doc.root.insert("empty".to_string(), Item::List(empty)); // Would trigger empty-list

    let diagnostics = runner.run_single_pass(&doc);

    // Should not have disabled rule diagnostics
    assert!(!diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::IdNaming)));
    assert!(!diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::EmptyList)));
}

#[test]
fn test_single_pass_nested_structure_traversal() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Create multi-level nested structure
    let mut level1 = BTreeMap::new();
    let mut level2 = BTreeMap::new();
    let mut level3 = BTreeMap::new();

    let mut list = MatrixList::new("Deep", vec!["id".to_string()]);
    list.add_row(Node::new("Deep", "x", vec![])); // Short ID at depth 3
    level3.insert("deepest".to_string(), Item::List(list));

    level2.insert("level3".to_string(), Item::Object(level3));
    level1.insert("level2".to_string(), Item::Object(level2));
    doc.root.insert("level1".to_string(), Item::Object(level1));

    let diagnostics = runner.run_single_pass(&doc);

    // Should find the short ID at depth 3
    assert!(diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::IdNaming)));
}

#[test]
fn test_single_pass_collects_types_from_nested_children() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Define schemas
    doc.structs
        .insert("Parent".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Child".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("GrandChild".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]);

    // Create hierarchy: Parent > Child > GrandChild
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);
    let mut parent = Node::new("Parent", "p1", vec![]);

    let mut child = Node::new("Child", "c1", vec![]);
    let grandchild = Node::new("GrandChild", "gc1", vec![]);

    child.add_child("GrandChild", grandchild);
    parent.add_child("Child", child);
    list.add_row(parent);

    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run_single_pass(&doc);

    // Only Unused should be reported
    let unused: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert_eq!(unused.len(), 1);
    assert!(unused[0].message().contains("Unused"));
}

#[test]
fn test_single_pass_diagnostic_limit_early_termination() {
    let config = LintConfig {
        max_diagnostics: 5,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);

    // Add many violations
    for i in 0..50 {
        list.add_row(Node::new("Item", format!("{}", i % 10), vec![]));
    }
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run_single_pass(&doc);

    // Should terminate early (allow for limit-exceeded warning)
    assert!(diagnostics.len() <= 6);
}

// =============================================================================
// Visitor with different document structures
// =============================================================================

#[test]
fn test_visitor_handles_empty_nested_objects() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    let mut nested = BTreeMap::new();
    let inner = BTreeMap::new();
    nested.insert("empty_object".to_string(), Item::Object(inner));

    doc.root
        .insert("container".to_string(), Item::Object(nested));

    let diagnostics = runner.run_single_pass(&doc);

    // Empty objects shouldn't cause issues
    assert!(diagnostics.is_empty());
}

#[test]
fn test_visitor_handles_mixed_item_types() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Mix of scalars, lists, and objects
    doc.root.insert(
        "string".to_string(),
        Item::Scalar(Value::String("test".to_string().into())),
    );
    doc.root
        .insert("int".to_string(), Item::Scalar(Value::Int(42)));

    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    list.add_row(Node::new("Item", "good_id", vec![]));
    doc.root.insert("list".to_string(), Item::List(list));

    let mut nested = BTreeMap::new();
    nested.insert("value".to_string(), Item::Scalar(Value::Bool(true)));
    doc.root.insert("object".to_string(), Item::Object(nested));

    let diagnostics = runner.run_single_pass(&doc);

    // Well-formed document should have no diagnostics
    assert!(diagnostics.is_empty());
}

#[test]
fn test_visitor_processes_multiple_lists() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));
    doc.structs
        .insert("Type1".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Type2".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Type3".to_string(), vec!["id".to_string()]);

    // Create multiple lists
    for i in 1..=3 {
        let mut list = MatrixList::new(format!("Type{i}"), vec!["id".to_string()]);
        list.add_row(Node::new(format!("Type{i}"), format!("item_{i}"), vec![]));
        doc.root.insert(format!("list{i}"), Item::List(list));
    }

    let diagnostics = runner.run_single_pass(&doc);

    // All types should be marked as used
    let unused: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
        .collect();

    assert!(unused.is_empty());
}

// =============================================================================
// Edge cases for visitor pattern
// =============================================================================

#[test]
fn test_visitor_handles_nodes_with_no_children() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    // Nodes without children
    list.add_row(Node::new("Item", "item1", vec![]));
    list.add_row(Node::new("Item", "item2", vec![]));
    list.add_row(Node::new("Item", "item3", vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run_single_pass(&doc);

    // Should work fine
    assert!(diagnostics.is_empty());
}

#[test]
fn test_visitor_handles_wide_trees() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Create wide tree (many children at same level)
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);
    let mut parent = Node::new("Parent", "parent_node", vec![]);

    for i in 0..100 {
        let child = Node::new("Child", format!("child_{i}"), vec![]);
        parent.add_child("Child", child);
    }

    list.add_row(parent);
    doc.root.insert("tree".to_string(), Item::List(list));

    let diagnostics = runner.run_single_pass(&doc);

    // Should handle wide trees without crashing
    // The exact number of diagnostics depends on implementation but should complete
    let _ = diagnostics;
}

#[test]
fn test_visitor_path_tracking() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Create nested structure to test path tracking
    let mut level1 = BTreeMap::new();
    let mut level2 = BTreeMap::new();

    level2.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("id"))),
    );

    level1.insert("inner".to_string(), Item::Object(level2));
    doc.root.insert("outer".to_string(), Item::Object(level1));

    let diagnostics = runner.run_single_pass(&doc);

    // Should find the unqualified reference
    assert!(diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference)));
}

// =============================================================================
// Parallel execution tests (when feature enabled)
// =============================================================================

#[cfg(feature = "parallel")]
#[test]
fn test_parallel_execution_produces_same_results() {
    let config = LintConfig {
        parallel: true,
        ..Default::default()
    };

    let runner_parallel = LintRunner::new(config);

    let config_sequential = LintConfig {
        parallel: false,
        ..Default::default()
    };

    let runner_sequential = LintRunner::new(config_sequential);

    let mut doc = Document::new((2, 0));

    // Complex document
    for i in 0..10 {
        doc.structs
            .insert(format!("Type{i}"), vec!["id".to_string()]);
    }

    for i in 0..5 {
        let mut list = MatrixList::new(format!("Type{i}"), vec!["id".to_string()]);
        list.add_row(Node::new(format!("Type{i}"), format!("item_{i}"), vec![]));
        doc.root.insert(format!("list{i}"), Item::List(list));
    }

    let parallel = runner_parallel.run(&doc);
    let sequential = runner_sequential.run(&doc);

    // Should produce same number of diagnostics
    assert_eq!(parallel.len(), sequential.len());
}

#[cfg(feature = "parallel")]
#[test]
fn test_parallel_execution_with_diagnostic_limit() {
    let config = LintConfig {
        parallel: true,
        max_diagnostics: 10,
        ..Default::default()
    };

    let runner = LintRunner::new(config);

    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);

    for i in 0..100 {
        list.add_row(Node::new("Item", format!("{}", i % 10), vec![]));
    }
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = runner.run(&doc);

    // Should respect limit even in parallel
    assert!(diagnostics.len() <= 11); // 10 + limit exceeded warning
}

// =============================================================================
// Performance characteristics tests
// =============================================================================

#[test]
fn test_single_pass_performance_on_large_document() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Create moderately large document
    doc.structs
        .insert("Item".to_string(), vec!["id".to_string()]);

    let mut list = MatrixList::new("Item", vec!["id".to_string()]);
    for i in 0..1000 {
        list.add_row(Node::new("Item", format!("item_{i}"), vec![]));
    }
    doc.root.insert("items".to_string(), Item::List(list));

    let start = std::time::Instant::now();
    let _diagnostics = runner.run_single_pass(&doc);
    let duration = start.elapsed();

    // Should complete in reasonable time (< 100ms for 1000 nodes)
    assert!(
        duration.as_millis() < 100,
        "Single pass took too long: {duration:?}"
    );
}

#[test]
fn test_sequential_vs_single_pass_produce_identical_results() {
    let runner = LintRunner::new(LintConfig::default());

    let mut doc = Document::new((2, 0));

    // Create diverse document with all rule triggers
    doc.structs.insert("A".to_string(), vec!["id".to_string()]);
    doc.structs.insert("B".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Unused1".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Unused2".to_string(), vec!["id".to_string()]);

    let mut list_a = MatrixList::new("A", vec!["id".to_string()]);
    list_a.add_row(Node::new("A", "a", vec![])); // Short
    list_a.add_row(Node::new("A", "123", vec![])); // Numeric
    list_a.add_row(Node::new("A", "good_id", vec![]));
    doc.root.insert("a_items".to_string(), Item::List(list_a));

    let list_b = MatrixList::new("B", vec!["id".to_string()]);
    doc.root.insert("b_items".to_string(), Item::List(list_b)); // Empty

    doc.root.insert(
        "ref1".to_string(),
        Item::Scalar(Value::Reference(Reference::local("id1"))),
    );
    doc.root.insert(
        "ref2".to_string(),
        Item::Scalar(Value::Reference(Reference::local("id2"))),
    );

    let sequential = runner.run(&doc);
    let single_pass = runner.run_single_pass(&doc);

    // Convert to comparable format
    use std::collections::HashMap;
    let to_map = |diagnostics: Vec<hedl_lint::Diagnostic>| {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for diag in diagnostics {
            let key = format!("{:?}", diag.kind());
            map.entry(key).or_default().push(diag.message().to_string());
        }
        for messages in map.values_mut() {
            messages.sort();
        }
        map
    };

    let seq_map = to_map(sequential);
    let sp_map = to_map(single_pass);

    assert_eq!(
        seq_map, sp_map,
        "Sequential and single-pass should produce identical diagnostics"
    );
}
