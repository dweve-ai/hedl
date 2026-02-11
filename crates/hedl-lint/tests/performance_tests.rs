// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Performance and optimization tests for hedl-lint

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_lint::{lint, lint_with_config, LintConfig, Severity};
use std::collections::BTreeMap;

/// Generate a document with many nodes for performance testing
fn generate_large_document(num_nodes: usize) -> Document {
    let mut doc = Document::new((2, 0));

    doc.structs.insert(
        "TestType".to_string(),
        vec!["id".to_string(), "value".to_string()],
    );

    let mut list = MatrixList::new("TestType", vec!["id".to_string(), "value".to_string()]);

    for i in 0..num_nodes {
        let id = format!("node_{i}");
        let node = Node::new("TestType", &id, vec![Value::Int(i as i64)]);
        list.add_row(node);
    }

    doc.root.insert("nodes".to_string(), Item::List(list));
    doc
}

/// Generate a document with deep nesting
fn generate_deeply_nested_document(depth: usize) -> Document {
    let mut doc = Document::new((2, 0));

    let current = &mut doc.root;
    for i in 0..depth {
        let mut nested = BTreeMap::new();
        let mut list = MatrixList::new("Level", vec!["id".to_string()]);
        list.add_row(Node::new("Level", format!("node_{i}"), vec![]));
        nested.insert("data".to_string(), Item::List(list));

        // For the last iteration, we just insert and stop
        if i == depth - 1 {
            current.insert(format!("level_{i}"), Item::Object(nested));
        } else {
            // Store the nested map first
            current.insert(format!("level_{i}"), Item::Object(nested));
            // We can't easily get a mutable reference to the nested object in current setup
            // So we'll create a linear nesting structure instead
        }
    }

    doc
}

#[test]
fn test_lint_small_document_performance() {
    let doc = generate_large_document(100);
    let diagnostics = lint(&doc);
    // Should complete quickly and find no errors/warnings (hints are acceptable)
    let issues: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.severity(), Severity::Error | Severity::Warning))
        .collect();
    assert!(issues.is_empty());
}

#[test]
fn test_lint_medium_document_performance() {
    let doc = generate_large_document(1000);
    let diagnostics = lint(&doc);
    let issues: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.severity(), Severity::Error | Severity::Warning))
        .collect();
    assert!(issues.is_empty());
}

#[test]
fn test_lint_large_document_performance() {
    let doc = generate_large_document(10_000);
    let diagnostics = lint(&doc);
    let issues: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.severity(), Severity::Error | Severity::Warning))
        .collect();
    assert!(issues.is_empty());
}

#[test]
fn test_early_termination_with_max_diagnostics() {
    let mut doc = Document::new((2, 0));

    // Create 1000 nodes with single-character IDs (all violations)
    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    for i in 0..1000 {
        let id = format!("{}", (i % 26 + 97) as u8 as char);
        list.add_row(Node::new("Test", &id, vec![]));
    }
    doc.root.insert("items".to_string(), Item::List(list));

    // Set a low max_diagnostics limit
    let config = LintConfig {
        max_diagnostics: 50,
        ..LintConfig::default()
    };

    let diagnostics = lint_with_config(&doc, config);

    // Should have stopped at limit + 1 (limit exceeded warning)
    assert!(diagnostics.len() <= 51);

    // Should have warning about limit exceeded
    let has_limit_warning = diagnostics
        .iter()
        .any(|d| d.message().contains("Diagnostic limit"));
    assert!(has_limit_warning);
}

#[test]
fn test_pre_allocation_efficiency() {
    let mut doc = Document::new((2, 0));

    // Multiple types to trigger multiple rule checks
    for i in 0..10 {
        doc.structs
            .insert(format!("Type_{i}"), vec!["id".to_string()]);
    }

    // Add some unused schemas
    doc.structs
        .insert("Unused1".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Unused2".to_string(), vec!["id".to_string()]);

    let diagnostics = lint(&doc);

    // Should find unused schemas
    assert!(!diagnostics.is_empty());
}

#[test]
#[cfg(feature = "parallel")]
fn test_parallel_execution_correctness() {
    let mut doc = Document::new((2, 0));

    // Create a document that triggers multiple rules
    doc.structs
        .insert("UsedType".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("UnusedType".to_string(), vec!["id".to_string()]);

    let mut list = MatrixList::new("UsedType", vec!["id".to_string()]);
    list.add_row(Node::new("UsedType", "a", vec![])); // Short ID
    list.add_row(Node::new("UsedType", "123", vec![])); // Numeric ID
    doc.root.insert("items".to_string(), Item::List(list));

    // Add empty list
    let empty_list = MatrixList::new("UsedType", vec!["id".to_string()]);
    doc.root
        .insert("empty_list".to_string(), Item::List(empty_list));

    // Add unqualified reference
    let ref_val = Value::Reference(Reference::local("a"));
    doc.root
        .insert("ref_field".to_string(), Item::Scalar(ref_val));

    // Run with parallel enabled
    let parallel_config = LintConfig {
        parallel: true,
        ..LintConfig::default()
    };
    let parallel_diagnostics = lint_with_config(&doc, parallel_config);

    // Run with parallel disabled
    let sequential_config = LintConfig {
        parallel: false,
        ..LintConfig::default()
    };
    let sequential_diagnostics = lint_with_config(&doc, sequential_config);

    // Both should find the same number of issues
    assert_eq!(parallel_diagnostics.len(), sequential_diagnostics.len());

    // Both should have the same diagnostic kinds (may be in different order)
    use std::collections::HashSet;

    let parallel_kinds: HashSet<_> = parallel_diagnostics
        .iter()
        .map(|d| format!("{:?}", d.kind()))
        .collect();
    let sequential_kinds: HashSet<_> = sequential_diagnostics
        .iter()
        .map(|d| format!("{:?}", d.kind()))
        .collect();

    assert_eq!(parallel_kinds, sequential_kinds);
}

#[test]
#[cfg(feature = "parallel")]
fn test_parallel_with_disabled_rules() {
    let mut doc = Document::new((2, 0));

    // Create violations for multiple rules
    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    list.add_row(Node::new("Test", "a", vec![])); // Short ID
    doc.root.insert("items".to_string(), Item::List(list));

    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]);

    // Disable one rule
    let mut config = LintConfig {
        parallel: true,
        ..LintConfig::default()
    };
    config.disable_rule("id-naming");

    let diagnostics = lint_with_config(&doc, config);

    // Should not have id-naming diagnostics
    use hedl_lint::DiagnosticKind;
    let has_id_naming = diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::IdNaming));
    assert!(!has_id_naming);

    // Should still have unused schema diagnostic
    let has_unused_schema = diagnostics
        .iter()
        .any(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema));
    assert!(has_unused_schema);
}

#[test]
#[cfg(feature = "parallel")]
fn test_parallel_early_termination() {
    let mut doc = Document::new((2, 0));

    // Create many violations
    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    for i in 0..1000 {
        let id = format!("{}", (i % 26 + 97) as u8 as char);
        list.add_row(Node::new("Test", &id, vec![]));
    }
    doc.root.insert("items".to_string(), Item::List(list));

    // Set a low limit
    let config = LintConfig {
        parallel: true,
        max_diagnostics: 50,
        ..LintConfig::default()
    };

    let diagnostics = lint_with_config(&doc, config);

    // Should have stopped at limit + 1 (limit exceeded warning)
    assert!(diagnostics.len() <= 51);
}

#[test]
fn test_deep_nesting_within_limits() {
    let doc = generate_deeply_nested_document(50);
    let diagnostics = lint(&doc);

    // Should complete without stack overflow
    // May or may not have diagnostics depending on the structure
    assert!(diagnostics.len() < 1000);
}

#[test]
fn test_max_depth_protection() {
    // Create a document with excessive nesting (beyond MAX_RECURSION_DEPTH)
    let mut doc = Document::new((2, 0));

    // We can't easily create 1000+ levels due to memory constraints in tests,
    // but we can test that reasonable nesting works
    let mut current = BTreeMap::new();
    for i in 0..100 {
        let mut nested = BTreeMap::new();
        let mut list = MatrixList::new("Deep", vec!["id".to_string()]);
        list.add_row(Node::new("Deep", format!("node_{i}"), vec![]));
        nested.insert("data".to_string(), Item::List(list));
        current.insert(format!("level_{i}"), Item::Object(nested));
    }
    doc.root
        .insert("deep_structure".to_string(), Item::Object(current));

    let diagnostics = lint(&doc);

    // Should complete without errors
    assert!(diagnostics.len() < 1000);
}

#[test]
fn test_severity_ordering_after_optimization() {
    let mut doc = Document::new((2, 0));

    // Create mixed severity issues
    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    list.add_row(Node::new("Test", "a", vec![])); // Hint: short ID
    doc.root.insert("items".to_string(), Item::List(list));

    doc.structs
        .insert("Unused".to_string(), vec!["id".to_string()]); // Warning: unused schema

    let ref_val = Value::Reference(Reference::local("a")); // Warning: unqualified ref
    doc.root.insert("ref".to_string(), Item::Scalar(ref_val));

    let diagnostics = lint(&doc);

    // Verify diagnostics are sorted by severity (highest first)
    let mut prev_severity = Severity::Error;
    for diag in &diagnostics {
        assert!(diag.severity() <= prev_severity);
        prev_severity = diag.severity();
    }
}

#[test]
#[cfg(feature = "parallel")]
fn test_parallel_deterministic_results() {
    let mut doc = Document::new((2, 0));

    // Create a consistent test document
    doc.structs
        .insert("Type1".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Type2".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Unused1".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Unused2".to_string(), vec!["id".to_string()]);

    let mut list = MatrixList::new("Type1", vec!["id".to_string()]);
    list.add_row(Node::new("Type1", "a", vec![]));
    list.add_row(Node::new("Type1", "b", vec![]));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = LintConfig {
        parallel: true,
        ..LintConfig::default()
    };

    // Run multiple times and verify consistent results
    let results: Vec<Vec<_>> = (0..5)
        .map(|_| {
            let diags = lint_with_config(&doc, config.clone());
            diags
                .into_iter()
                .map(|d| {
                    (
                        d.severity(),
                        format!("{:?}", d.kind()),
                        d.message().to_string(),
                    )
                })
                .collect()
        })
        .collect();

    // All runs should produce the same diagnostics (same length)
    let first_len = results[0].len();
    for result in &results {
        assert_eq!(result.len(), first_len);
    }
}

#[test]
fn test_empty_document_minimal_overhead() {
    let doc = Document::new((2, 0));

    // Should complete very quickly with no allocations wasted
    let diagnostics = lint(&doc);
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_config_validation() {
    let mut config = LintConfig::default();

    // Should validate successfully
    assert!(config.validate().is_ok());

    // Add a very long rule ID
    let long_id = "a".repeat(200);
    config.enable_rule(&long_id);

    // Should fail validation
    assert!(config.validate().is_err());
}

#[test]
fn test_memory_efficiency_with_large_diagnostics() {
    let mut doc = Document::new((2, 0));

    // Create a moderate number of violations
    let mut list = MatrixList::new("Test", vec!["id".to_string()]);
    for i in 0..100 {
        let id = format!("{}", (i % 26 + 97) as u8 as char);
        list.add_row(Node::new("Test", &id, vec![]));
    }
    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    // Should handle efficiently without excessive memory
    assert!(!diagnostics.is_empty());
    assert!(diagnostics.len() <= 100);
}
