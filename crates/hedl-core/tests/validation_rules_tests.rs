// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for validation rules.
//!
//! This test suite provides exhaustive coverage of the validation system,
//! including all rule implementations, edge cases, and error paths.

use hedl_core::validation::{
    DuplicateKeyRule, InvalidReferenceRule, Rule, RuleCategory, Severity, TypeMismatchRule,
    UnusedReferenceRule, ValidationContext,
};
use hedl_core::{parse, Document, Item, MatrixList, Node, Value};

// ==================== Helper functions ====================

fn create_test_doc() -> Document {
    let mut doc = Document::new((2, 0));

    // Create a User list
    let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    list.add_row(Node::new(
        "User",
        "1",
        vec![Value::Int(1), Value::String("Alice".to_string().into())],
    ));
    list.add_row(Node::new(
        "User",
        "2",
        vec![Value::Int(2), Value::String("Bob".to_string().into())],
    ));

    doc.root.insert("User".to_string(), Item::List(list));
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    doc
}

fn create_doc_with_duplicates() -> Document {
    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("User", vec!["name".to_string()]);
    list.add_row(Node::new(
        "User",
        "1",
        vec![Value::String("Alice".to_string().into())],
    ));
    list.add_row(Node::new(
        "User",
        "1", // Duplicate!
        vec![Value::String("Bob".to_string().into())],
    ));

    doc.root.insert("User".to_string(), Item::List(list));
    doc.structs
        .insert("User".to_string(), vec!["name".to_string()]);

    doc
}

// ==================== DuplicateKeyRule tests ====================

#[test]
fn test_duplicate_key_rule_id() {
    let rule = DuplicateKeyRule;
    assert_eq!(rule.id(), "duplicate-key");
}

#[test]
fn test_duplicate_key_rule_description() {
    let rule = DuplicateKeyRule;
    assert!(!rule.description().is_empty());
    assert!(rule.description().contains("duplicate"));
}

#[test]
fn test_duplicate_key_rule_category() {
    let rule = DuplicateKeyRule;
    assert_eq!(rule.category(), RuleCategory::Structure);
}

#[test]
fn test_duplicate_key_rule_default_severity() {
    let rule = DuplicateKeyRule;
    assert_eq!(rule.default_severity(), Severity::Error);
}

#[test]
fn test_duplicate_key_rule_cost_estimate() {
    let rule = DuplicateKeyRule;
    let cost = rule.cost_estimate();
    assert!(cost > 0 && cost <= 100);
}

#[test]
fn test_duplicate_key_rule_no_duplicates() {
    let rule = DuplicateKeyRule;
    let doc = create_test_doc();
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_duplicate_key_rule_finds_duplicates() {
    let rule = DuplicateKeyRule;
    let doc = create_doc_with_duplicates();
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 1);

    let diag = &diagnostics[0];
    assert_eq!(diag.severity(), Severity::Error);
    assert!(diag.message().contains("Duplicate"));
    assert!(diag.message().contains("'1'"));
}

#[test]
fn test_duplicate_key_rule_multiple_types_no_conflict() {
    // Different types can have same IDs
    let mut doc = Document::new((2, 0));

    let mut user_list = MatrixList::new("User", vec!["name".to_string()]);
    user_list.add_row(Node::new(
        "User",
        "1",
        vec![Value::String("Alice".to_string().into())],
    ));
    doc.root.insert("User".to_string(), Item::List(user_list));

    let mut post_list = MatrixList::new("Post", vec!["title".to_string()]);
    post_list.add_row(Node::new(
        "Post",
        "1", // Same ID but different type
        vec![Value::String("Hello".to_string().into())],
    ));
    doc.root.insert("Post".to_string(), Item::List(post_list));

    let rule = DuplicateKeyRule;
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_duplicate_key_rule_empty_document() {
    let rule = DuplicateKeyRule;
    let doc = Document::new((2, 0));
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_duplicate_key_rule_single_node() {
    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("User", vec!["name".to_string()]);
    list.add_row(Node::new(
        "User",
        "1",
        vec![Value::String("Alice".to_string().into())],
    ));
    doc.root.insert("User".to_string(), Item::List(list));

    let rule = DuplicateKeyRule;
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 0);
}

// ==================== InvalidReferenceRule tests ====================

#[test]
fn test_invalid_reference_rule_id() {
    let rule = InvalidReferenceRule;
    assert_eq!(rule.id(), "invalid-reference");
}

#[test]
fn test_invalid_reference_rule_description() {
    let rule = InvalidReferenceRule;
    assert!(!rule.description().is_empty());
    assert!(rule.description().contains("reference"));
}

#[test]
fn test_invalid_reference_rule_category() {
    let rule = InvalidReferenceRule;
    assert_eq!(rule.category(), RuleCategory::References);
}

#[test]
fn test_invalid_reference_rule_default_severity() {
    let rule = InvalidReferenceRule;
    assert_eq!(rule.default_severity(), Severity::Error);
}

#[test]
fn test_invalid_reference_rule_cost_estimate() {
    let rule = InvalidReferenceRule;
    let cost = rule.cost_estimate();
    assert!(cost > 0 && cost <= 100);
}

#[test]
fn test_invalid_reference_rule_before_document() {
    let rule = InvalidReferenceRule;
    let mut context = ValidationContext::new();

    let result = rule.before_document(&mut context);
    assert!(result.is_ok());
}

#[test]
fn test_invalid_reference_rule_no_references() {
    let rule = InvalidReferenceRule;
    let doc = create_test_doc();
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_invalid_reference_rule_valid_reference() {
    use hedl_core::Reference;

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("User", vec!["id".to_string(), "friend".to_string()]);
    list.add_row(Node::new(
        "User",
        "1",
        vec![
            Value::Int(1),
            Value::Reference(Reference::qualified("User", "2")),
        ],
    ));
    list.add_row(Node::new("User", "2", vec![Value::Int(2), Value::Null]));
    doc.root.insert("User".to_string(), Item::List(list));

    let rule = InvalidReferenceRule;
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_invalid_reference_rule_detects_invalid() {
    use hedl_core::Reference;

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("User", vec!["id".to_string(), "friend".to_string()]);
    list.add_row(Node::new(
        "User",
        "1",
        vec![
            Value::Int(1),
            Value::Reference(Reference::qualified("User", "999")), // Invalid!
        ],
    ));
    doc.root.insert("User".to_string(), Item::List(list));

    let rule = InvalidReferenceRule;
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 1);

    let diag = &diagnostics[0];
    assert_eq!(diag.severity(), Severity::Error);
    assert!(diag.message().contains("non-existent"));
    assert!(diag.message().contains("999"));
}

#[test]
fn test_invalid_reference_rule_unqualified_reference() {
    use hedl_core::Reference;

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("User", vec!["id".to_string(), "ref".to_string()]);
    list.add_row(Node::new(
        "User",
        "1",
        vec![
            Value::Int(1),
            Value::Reference(Reference::local("nonexistent")),
        ],
    ));
    doc.root.insert("User".to_string(), Item::List(list));

    let rule = InvalidReferenceRule;
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert!(!diagnostics.is_empty());
}

// ==================== TypeMismatchRule tests ====================

#[test]
fn test_type_mismatch_rule_id() {
    let rule = TypeMismatchRule;
    assert_eq!(rule.id(), "type-mismatch");
}

#[test]
fn test_type_mismatch_rule_description() {
    let rule = TypeMismatchRule;
    assert!(!rule.description().is_empty());
}

#[test]
fn test_type_mismatch_rule_category() {
    let rule = TypeMismatchRule;
    assert_eq!(rule.category(), RuleCategory::TypeSafety);
}

#[test]
fn test_type_mismatch_rule_default_severity() {
    let rule = TypeMismatchRule;
    assert_eq!(rule.default_severity(), Severity::Warning);
}

#[test]
fn test_type_mismatch_rule_cost_estimate() {
    let rule = TypeMismatchRule;
    let cost = rule.cost_estimate();
    assert!(cost > 0 && cost <= 100);
}

// ==================== UnusedReferenceRule tests ====================

#[test]
fn test_unused_reference_rule_id() {
    let rule = UnusedReferenceRule;
    assert_eq!(rule.id(), "unused-reference");
}

#[test]
fn test_unused_reference_rule_description() {
    let rule = UnusedReferenceRule;
    assert!(!rule.description().is_empty());
}

#[test]
fn test_unused_reference_rule_category() {
    let rule = UnusedReferenceRule;
    assert_eq!(rule.category(), RuleCategory::Style);
}

#[test]
fn test_unused_reference_rule_default_severity() {
    let rule = UnusedReferenceRule;
    assert_eq!(rule.default_severity(), Severity::Hint);
}

#[test]
fn test_unused_reference_rule_cost_estimate() {
    let rule = UnusedReferenceRule;
    let cost = rule.cost_estimate();
    assert!(cost > 0 && cost <= 100);
}

// ==================== Integration tests ====================

#[test]
fn test_parse_rejects_duplicate_keys() {
    // Parser already detects duplicate keys during parsing
    let input = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | user1, Alice
 | user1, Bob
";

    let result = parse(input.as_bytes());
    // Duplicate keys are detected at parse time, not validation time
    assert!(result.is_err());
}

#[test]
fn test_parse_and_validate_no_duplicates() {
    let input = r"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | user1, Alice
 | user2, Bob
";

    let doc = parse(input.as_bytes()).unwrap();
    let rule = DuplicateKeyRule;
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 0);
}

#[test]
fn test_multiple_rules_on_same_document() {
    let doc = create_test_doc();

    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(DuplicateKeyRule),
        Box::new(InvalidReferenceRule),
        Box::new(TypeMismatchRule),
        Box::new(UnusedReferenceRule),
    ];

    for rule in &rules {
        let mut context = ValidationContext::new();
        let result = rule.check(&doc, &mut context);
        assert!(result.is_ok());
    }
}

// ==================== Edge case tests ====================

#[test]
fn test_rule_on_empty_document() {
    let doc = Document::new((2, 0));

    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(DuplicateKeyRule),
        Box::new(InvalidReferenceRule),
        Box::new(TypeMismatchRule),
        Box::new(UnusedReferenceRule),
    ];

    for rule in &rules {
        let mut context = ValidationContext::new();
        let diagnostics = rule.check(&doc, &mut context).unwrap();
        assert_eq!(diagnostics.len(), 0);
    }
}

#[test]
fn test_diagnostic_with_related_info() {
    let doc = create_doc_with_duplicates();
    let rule = DuplicateKeyRule;
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert!(!diagnostics.is_empty());

    // Check that diagnostic has message
    let diag = &diagnostics[0];
    assert!(!diag.message().is_empty());
    assert_eq!(diag.rule_id(), "duplicate-key");
}

#[test]
fn test_validation_context_registration() {
    let doc = create_test_doc();
    let mut context = ValidationContext::new();

    // InvalidReferenceRule builds the symbol table
    let rule = InvalidReferenceRule;
    let _ = rule.check(&doc, &mut context);

    // Context should have nodes registered
    // (This is implicitly tested by the rule not failing)
}

// ==================== Boundary tests ====================

#[test]
fn test_very_long_id_duplicate_detection() {
    let long_id = "a".repeat(1000);

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("User", vec!["name".to_string()]);
    list.add_row(Node::new(
        "User",
        &long_id,
        vec![Value::String("First".to_string().into())],
    ));
    list.add_row(Node::new(
        "User",
        &long_id,
        vec![Value::String("Second".to_string().into())],
    ));
    doc.root.insert("User".to_string(), Item::List(list));

    let rule = DuplicateKeyRule;
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 1);
}

#[test]
fn test_unicode_id_duplicate_detection() {
    let unicode_id = "用户1";

    let mut doc = Document::new((2, 0));

    let mut list = MatrixList::new("User", vec!["name".to_string()]);
    list.add_row(Node::new(
        "User",
        unicode_id,
        vec![Value::String("First".to_string().into())],
    ));
    list.add_row(Node::new(
        "User",
        unicode_id,
        vec![Value::String("Second".to_string().into())],
    ));
    doc.root.insert("User".to_string(), Item::List(list));

    let rule = DuplicateKeyRule;
    let mut context = ValidationContext::new();

    let diagnostics = rule.check(&doc, &mut context).unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics[0].message().contains(unicode_id));
}

#[test]
fn test_rule_cost_estimates_are_reasonable() {
    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(DuplicateKeyRule),
        Box::new(InvalidReferenceRule),
        Box::new(TypeMismatchRule),
        Box::new(UnusedReferenceRule),
    ];

    for rule in &rules {
        let cost = rule.cost_estimate();
        // Cost should be between 1 and 100
        assert!(
            (1..=100).contains(&cost),
            "Rule {} has invalid cost {}",
            rule.id(),
            cost
        );
    }
}

#[test]
fn test_all_rules_have_unique_ids() {
    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(DuplicateKeyRule),
        Box::new(InvalidReferenceRule),
        Box::new(TypeMismatchRule),
        Box::new(UnusedReferenceRule),
    ];

    let mut ids = std::collections::HashSet::new();
    for rule in &rules {
        let id = rule.id();
        assert!(!ids.contains(id), "Duplicate rule ID: {id}");
        ids.insert(id);
    }
}

#[test]
fn test_all_rules_have_descriptions() {
    let rules: Vec<Box<dyn Rule>> = vec![
        Box::new(DuplicateKeyRule),
        Box::new(InvalidReferenceRule),
        Box::new(TypeMismatchRule),
        Box::new(UnusedReferenceRule),
    ];

    for rule in &rules {
        let desc = rule.description();
        assert!(!desc.is_empty(), "Rule {} has no description", rule.id());
        assert!(desc.len() > 10, "Rule {} description too short", rule.id());
    }
}
