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

//! Tests for inline child list lint rules.

use hedl_core::{Document, Item, MatrixList, Node};
use hedl_lint::{lint, DiagnosticKind, Severity};

// =============================================================================
// InlineChildExceedsMax Rule Tests
// =============================================================================

#[test]
fn test_inline_child_within_limit_passes() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Create node with 10 children (maximum allowed per SPEC v2.0 line 58)
    let mut parent = Node::with_child_count("Parent", "p1", vec![], 10);
    for i in 1..=10 {
        parent.add_child("Child", Node::new("Child", format!("c{}", i), vec![]));
    }
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let exceeds_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineChildExceedsMax))
        .collect();

    assert!(
        exceeds_errors.is_empty(),
        "10 children should be allowed (SPEC v2.0 limit)"
    );
}

#[test]
fn test_inline_child_exceeds_max_error() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Create node with 11 children (exceeds v2.0 maximum of 10)
    let mut parent = Node::with_child_count("Parent", "p1", vec![], 11);
    for i in 1..=11 {
        parent.add_child("Child", Node::new("Child", format!("c{}", i), vec![]));
    }
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let exceeds_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineChildExceedsMax))
        .collect();

    assert_eq!(exceeds_errors.len(), 1);
    assert_eq!(exceeds_errors[0].severity(), Severity::Warning);
    assert!(exceeds_errors[0].message().contains("11 entries"));
    assert!(exceeds_errors[0]
        .message()
        .contains("recommended maximum is 10"));
}

#[test]
fn test_inline_child_exceeds_max_multiple_nodes() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // First node with 12 children (exceeds v2.0 limit of 10)
    let mut parent1 = Node::with_child_count("Parent", "p1", vec![], 12);
    for i in 1..=12 {
        parent1.add_child("Child", Node::new("Child", format!("c1_{}", i), vec![]));
    }
    list.add_row(parent1);

    // Second node with 15 children (exceeds v2.0 limit of 10)
    let mut parent2 = Node::with_child_count("Parent", "p2", vec![], 15);
    for i in 1..=15 {
        parent2.add_child("Child", Node::new("Child", format!("c2_{}", i), vec![]));
    }
    list.add_row(parent2);

    // Third node with 3 children (valid)
    let mut parent3 = Node::with_child_count("Parent", "p3", vec![], 3);
    for i in 1..=3 {
        parent3.add_child("Child", Node::new("Child", format!("c3_{}", i), vec![]));
    }
    list.add_row(parent3);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let exceeds_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineChildExceedsMax))
        .collect();

    // Should have 2 warnings (p1 and p2)
    assert_eq!(exceeds_errors.len(), 2);
}

#[test]
fn test_inline_child_no_hint_no_error() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Create node with many children but NO count hint
    let mut parent = Node::new("Parent", "p1", vec![]);
    for i in 1..=20 {
        parent.add_child("Child", Node::new("Child", format!("c{}", i), vec![]));
    }
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let exceeds_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineChildExceedsMax))
        .collect();

    // No count hint means not using inline syntax, so no error
    assert!(exceeds_errors.is_empty());
}

// =============================================================================
// InlineCountMismatch Rule Tests
// =============================================================================

#[test]
fn test_inline_count_matches_passes() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Declare 3 children, add exactly 3
    let mut parent = Node::with_child_count("Parent", "p1", vec![], 3);
    parent.add_child("Child", Node::new("Child", "c1", vec![]));
    parent.add_child("Child", Node::new("Child", "c2", vec![]));
    parent.add_child("Child", Node::new("Child", "c3", vec![]));
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let mismatch_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineCountMismatch))
        .collect();

    assert!(mismatch_errors.is_empty());
}

#[test]
fn test_inline_count_mismatch_fewer_children() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Declare 5 children, but only add 3
    let mut parent = Node::with_child_count("Parent", "p1", vec![], 5);
    parent.add_child("Child", Node::new("Child", "c1", vec![]));
    parent.add_child("Child", Node::new("Child", "c2", vec![]));
    parent.add_child("Child", Node::new("Child", "c3", vec![]));
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let mismatch_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineCountMismatch))
        .collect();

    assert_eq!(mismatch_errors.len(), 1);
    assert_eq!(mismatch_errors[0].severity(), Severity::Error);
    assert!(mismatch_errors[0].message().contains("declares 5"));
    assert!(mismatch_errors[0].message().contains("has 3"));
}

#[test]
fn test_inline_count_mismatch_more_children() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Declare 2 children, but add 4
    let mut parent = Node::with_child_count("Parent", "p1", vec![], 2);
    parent.add_child("Child", Node::new("Child", "c1", vec![]));
    parent.add_child("Child", Node::new("Child", "c2", vec![]));
    parent.add_child("Child", Node::new("Child", "c3", vec![]));
    parent.add_child("Child", Node::new("Child", "c4", vec![]));
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let mismatch_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineCountMismatch))
        .collect();

    assert_eq!(mismatch_errors.len(), 1);
    assert!(mismatch_errors[0].message().contains("declares 2"));
    assert!(mismatch_errors[0].message().contains("has 4"));
}

#[test]
fn test_inline_count_mismatch_multiple_child_types() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Declare 5 children total, add 2 Child + 2 Tag = 4 total
    let mut parent = Node::with_child_count("Parent", "p1", vec![], 5);
    parent.add_child("Child", Node::new("Child", "c1", vec![]));
    parent.add_child("Child", Node::new("Child", "c2", vec![]));
    parent.add_child("Tag", Node::new("Tag", "t1", vec![]));
    parent.add_child("Tag", Node::new("Tag", "t2", vec![]));
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let mismatch_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineCountMismatch))
        .collect();

    assert_eq!(mismatch_errors.len(), 1);
    assert!(mismatch_errors[0].message().contains("declares 5"));
    assert!(mismatch_errors[0].message().contains("has 4"));
}

#[test]
fn test_inline_count_zero_hint_with_children() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // No count hint (child_count = 0) but has children
    let mut parent = Node::new("Parent", "p1", vec![]);
    parent.add_child("Child", Node::new("Child", "c1", vec![]));
    parent.add_child("Child", Node::new("Child", "c2", vec![]));
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let mismatch_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineCountMismatch))
        .collect();

    // No hint means no mismatch error (different rule handles missing hints)
    assert!(mismatch_errors.is_empty());
}

// =============================================================================
// MissingCountHint Rule Tests
// =============================================================================

#[test]
fn test_missing_count_hint_suggestion() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Node with multiple children of same type but no count hint
    let mut parent = Node::new("Parent", "p1", vec![]);
    parent.add_child("Child", Node::new("Child", "c1", vec![]));
    parent.add_child("Child", Node::new("Child", "c2", vec![]));
    parent.add_child("Child", Node::new("Child", "c3", vec![]));
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let hint_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::MissingCountHint))
        .collect();

    assert_eq!(hint_warnings.len(), 1);
    assert_eq!(hint_warnings[0].severity(), Severity::Hint);
    assert!(hint_warnings[0].message().contains("3 'Child' children"));
    assert!(hint_warnings[0].message().contains("no count hint"));
}

#[test]
fn test_missing_count_hint_single_child_no_warning() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Single child doesn't need inline syntax
    let mut parent = Node::new("Parent", "p1", vec![]);
    parent.add_child("Child", Node::new("Child", "c1", vec![]));
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let hint_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::MissingCountHint))
        .collect();

    assert!(hint_warnings.is_empty());
}

#[test]
fn test_missing_count_hint_exceeds_max_no_warning() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // More than 5 children shouldn't suggest inline syntax
    let mut parent = Node::new("Parent", "p1", vec![]);
    for i in 1..=10 {
        parent.add_child("Child", Node::new("Child", format!("c{}", i), vec![]));
    }
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let hint_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::MissingCountHint))
        .collect();

    // Shouldn't suggest inline syntax for >5 children
    assert!(hint_warnings.is_empty());
}

#[test]
fn test_missing_count_hint_multiple_types() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Multiple child types, each with multiple children
    let mut parent = Node::new("Parent", "p1", vec![]);
    parent.add_child("Child", Node::new("Child", "c1", vec![]));
    parent.add_child("Child", Node::new("Child", "c2", vec![]));
    parent.add_child("Tag", Node::new("Tag", "t1", vec![]));
    parent.add_child("Tag", Node::new("Tag", "t2", vec![]));
    parent.add_child("Tag", Node::new("Tag", "t3", vec![]));
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let hint_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::MissingCountHint))
        .collect();

    // Should suggest hints for both Child (2) and Tag (3)
    assert_eq!(hint_warnings.len(), 2);
}

#[test]
fn test_missing_count_hint_has_hint_no_warning() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Node with count hint should not trigger warning
    let mut parent = Node::with_child_count("Parent", "p1", vec![], 3);
    parent.add_child("Child", Node::new("Child", "c1", vec![]));
    parent.add_child("Child", Node::new("Child", "c2", vec![]));
    parent.add_child("Child", Node::new("Child", "c3", vec![]));
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let hint_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::MissingCountHint))
        .collect();

    assert!(hint_warnings.is_empty());
}

// =============================================================================
// Combined Rule Tests
// =============================================================================

#[test]
fn test_multiple_inline_issues() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Node 1: Exceeds max (12 children with hint, exceeds v2.0 limit of 10)
    let mut p1 = Node::with_child_count("Parent", "p1", vec![], 12);
    for i in 1..=12 {
        p1.add_child("Child", Node::new("Child", format!("c1_{}", i), vec![]));
    }
    list.add_row(p1);

    // Node 2: Count mismatch (declares 3, has 5)
    let mut p2 = Node::with_child_count("Parent", "p2", vec![], 3);
    for i in 1..=5 {
        p2.add_child("Child", Node::new("Child", format!("c2_{}", i), vec![]));
    }
    list.add_row(p2);

    // Node 3: Missing count hint (4 children, no hint)
    let mut p3 = Node::new("Parent", "p3", vec![]);
    for i in 1..=4 {
        p3.add_child("Child", Node::new("Child", format!("c3_{}", i), vec![]));
    }
    list.add_row(p3);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let exceeds: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineChildExceedsMax))
        .collect();
    let mismatch: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineCountMismatch))
        .collect();
    let missing: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::MissingCountHint))
        .collect();

    assert_eq!(exceeds.len(), 1, "p1 should trigger exceeds-max");
    assert_eq!(mismatch.len(), 1, "p2 should trigger count-mismatch");
    assert_eq!(missing.len(), 1, "p3 should trigger missing-hint");
}

#[test]
fn test_nested_inline_children() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Level1", vec!["id".to_string()]);

    // Level 1 node with inline children
    let mut l1 = Node::with_child_count("Level1", "l1", vec![], 2);

    // Level 2 child with its own inline children (exceeds v2.0 max of 10)
    let mut l2a = Node::with_child_count("Level2", "l2a", vec![], 11);
    for i in 1..=11 {
        l2a.add_child("Level3", Node::new("Level3", format!("l3a_{}", i), vec![]));
    }

    // Level 2 child with count mismatch
    let mut l2b = Node::with_child_count("Level2", "l2b", vec![], 2);
    l2b.add_child("Level3", Node::new("Level3", "l3b_1", vec![]));

    l1.add_child("Level2", l2a);
    l1.add_child("Level2", l2b);
    list.add_row(l1);

    doc.root.insert("items".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let exceeds: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineChildExceedsMax))
        .collect();
    let mismatch: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineCountMismatch))
        .collect();

    assert_eq!(exceeds.len(), 1, "l2a should trigger exceeds-max");
    assert_eq!(mismatch.len(), 1, "l2b should trigger count-mismatch");
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_document_no_inline_warnings() {
    let doc = Document::new((2, 0));
    let diagnostics = lint(&doc);

    let inline_diagnostics: Vec<_> = diagnostics
        .iter()
        .filter(|d| {
            matches!(
                d.kind(),
                DiagnosticKind::InlineChildExceedsMax
                    | DiagnosticKind::InlineCountMismatch
                    | DiagnosticKind::MissingCountHint
            )
        })
        .collect();

    assert!(inline_diagnostics.is_empty());
}

#[test]
fn test_node_with_no_children_no_warnings() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Node with no children at all
    let parent = Node::new("Parent", "p1", vec![]);
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let inline_diagnostics: Vec<_> = diagnostics
        .iter()
        .filter(|d| {
            matches!(
                d.kind(),
                DiagnosticKind::InlineChildExceedsMax
                    | DiagnosticKind::InlineCountMismatch
                    | DiagnosticKind::MissingCountHint
            )
        })
        .collect();

    assert!(inline_diagnostics.is_empty());
}

#[test]
fn test_inline_hint_exactly_at_boundary() {
    let mut doc = Document::new((2, 0));
    let mut list = MatrixList::new("Parent", vec!["id".to_string()]);

    // Exactly 10 children (maximum allowed for inline per SPEC v2.0)
    let mut parent = Node::with_child_count("Parent", "p1", vec![], 10);
    for i in 1..=10 {
        parent.add_child("Child", Node::new("Child", format!("c{}", i), vec![]));
    }
    list.add_row(parent);

    doc.root.insert("parents".to_string(), Item::List(list));

    let diagnostics = lint(&doc);

    let exceeds: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineChildExceedsMax))
        .collect();
    let mismatch: Vec<_> = diagnostics
        .iter()
        .filter(|d| matches!(d.kind(), DiagnosticKind::InlineCountMismatch))
        .collect();

    assert!(
        exceeds.is_empty(),
        "10 children is at the v2.0 limit, should pass"
    );
    assert!(mismatch.is_empty(), "count matches exactly");
}
