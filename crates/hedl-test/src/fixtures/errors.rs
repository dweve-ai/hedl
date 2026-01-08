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

//! Error test fixtures for HEDL.
//!
//! This module provides common error scenarios and invalid documents
//! for testing error handling across all converters.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use std::collections::BTreeMap;

/// Invalid HEDL text samples for parser error testing.
///
/// Each tuple contains (name, hedl_text, expected_error_kind).
pub fn invalid_hedl_samples() -> Vec<(&'static str, &'static str)> {
    vec![
        ("empty", ""),
        ("whitespace_only", "   \t\n  "),
        ("invalid_directive", "%INVALID: 1.0\n---\n"),
        ("missing_separator", "%VERSION: 1.0\nfield: value"),
        ("invalid_version", "%VERSION: 99.99\n---\n"),
        ("malformed_struct", "%STRUCT: InvalidFormat\n---\n"),
        ("unclosed_string", "field: \"unclosed"),
        ("invalid_escape", "field: \"invalid\\x\""),
        ("malformed_reference", "ref: @"),
        ("invalid_tensor", "tensor: [[["),
        ("mismatched_brackets", "tensor: [1, 2]]"),
        ("invalid_number", "num: 123.456.789"),
        ("malformed_expression", "expr: $(unclosed("),
        ("invalid_identifier", "123invalid: value"),
        ("duplicate_directive", "%VERSION: 1.0\n%VERSION: 1.0\n---\n"),
    ]
}

/// Invalid expression strings for expression parser testing.
///
/// Returns a list of (description, expression_string) tuples.
pub fn invalid_expression_samples() -> Vec<(&'static str, &'static str)> {
    vec![
        ("empty", ""),
        ("whitespace_only", "   "),
        ("unclosed_paren", "func("),
        ("unclosed_string", "\"unclosed"),
        ("invalid_chars", "!!!"),
        ("double_dot", "obj..field"),
        ("trailing_dot", "obj."),
        ("leading_dot", ".field"),
        ("mismatched_parens", "func())"),
        ("empty_call", "()"),
        ("invalid_identifier", "123invalid"),
        ("special_chars_only", "@#$%"),
        ("nested_unclosed", "outer(inner("),
        ("comma_without_args", "func(,)"),
        ("trailing_comma", "func(a,)"),
    ]
}

/// Documents that are structurally valid but semantically invalid.
///
/// Useful for testing validation logic.
pub fn semantically_invalid_docs() -> Vec<(&'static str, Document)> {
    vec![
        ("undefined_struct", undefined_struct()),
        ("undefined_nest", undefined_nest()),
        ("circular_nest", circular_nest()),
        ("dangling_reference", dangling_reference()),
        ("mismatched_schema", mismatched_schema()),
        ("empty_type_name", empty_type_name()),
        ("duplicate_ids", duplicate_ids()),
        ("invalid_alias", invalid_alias()),
    ]
}

/// Document with undefined struct reference.
///
/// Has a MatrixList with a type_name not defined in structs.
fn undefined_struct() -> Document {
    let mut doc = Document::new((1, 0));

    let list = MatrixList {
        type_name: "UndefinedType".to_string(),
        schema: vec!["id".to_string(), "name".to_string()],
        rows: vec![],
        count_hint: None,
    };

    doc.root.insert("items".to_string(), Item::List(list));
    // Note: structs map is empty - UndefinedType not defined

    doc
}

/// Document with undefined NEST reference.
///
/// Has a NEST directive referencing a non-existent child type.
fn undefined_nest() -> Document {
    let mut doc = Document::new((1, 0));

    doc.structs.insert(
        "Parent".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    // NEST references non-existent child type
    doc.nests.insert("Parent".to_string(), "NonExistentChild".to_string());

    doc
}

/// Document with circular NEST references.
///
/// Parent nests Child, Child nests Parent (circular).
fn circular_nest() -> Document {
    let mut doc = Document::new((1, 0));

    doc.structs.insert(
        "TypeA".to_string(),
        vec!["id".to_string()],
    );
    doc.structs.insert(
        "TypeB".to_string(),
        vec!["id".to_string()],
    );

    // Circular NEST: A -> B -> A
    doc.nests.insert("TypeA".to_string(), "TypeB".to_string());
    doc.nests.insert("TypeB".to_string(), "TypeA".to_string());

    doc
}

/// Document with dangling references.
///
/// References point to non-existent nodes.
fn dangling_reference() -> Document {
    let mut doc = Document::new((1, 0));

    let mut list = MatrixList::new(
        "Item",
        vec!["id".to_string(), "related".to_string()],
    );

    list.add_row(Node::new(
        "Item",
        "item1",
        vec![
            Value::String("item1".to_string()),
            Value::Reference(hedl_core::Reference {
                type_name: Some("Item".to_string()),
                id: "nonexistent".to_string(), // Dangling reference
            }),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));
    doc.structs.insert(
        "Item".to_string(),
        vec!["id".to_string(), "related".to_string()],
    );

    doc
}

/// Document with mismatched schema.
///
/// MatrixList schema doesn't match the defined struct.
fn mismatched_schema() -> Document {
    let mut doc = Document::new((1, 0));

    // Define struct with certain fields
    doc.structs.insert(
        "Person".to_string(),
        vec!["id".to_string(), "name".to_string(), "age".to_string()],
    );

    // Create list with different schema
    let list = MatrixList {
        type_name: "Person".to_string(),
        schema: vec!["id".to_string(), "name".to_string()], // Missing 'age'
        rows: vec![],
        count_hint: None,
    };

    doc.root.insert("people".to_string(), Item::List(list));

    doc
}

/// Document with empty type name.
///
/// MatrixList has an empty string as type_name.
fn empty_type_name() -> Document {
    let mut doc = Document::new((1, 0));

    let list = MatrixList {
        type_name: String::new(), // Empty type name
        schema: vec!["id".to_string()],
        rows: vec![],
        count_hint: None,
    };

    doc.root.insert("items".to_string(), Item::List(list));

    doc
}

/// Document with duplicate IDs.
///
/// Multiple nodes in the same list have the same ID.
fn duplicate_ids() -> Document {
    let mut doc = Document::new((1, 0));

    let mut list = MatrixList::new("Item", vec!["id".to_string(), "name".to_string()]);

    // Add two nodes with the same ID
    list.add_row(Node::new(
        "Item",
        "duplicate",
        vec![
            Value::String("duplicate".to_string()),
            Value::String("First".to_string()),
        ],
    ));

    list.add_row(Node::new(
        "Item",
        "duplicate", // Duplicate ID
        vec![
            Value::String("duplicate".to_string()),
            Value::String("Second".to_string()),
        ],
    ));

    doc.root.insert("items".to_string(), Item::List(list));
    doc.structs.insert(
        "Item".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    doc
}

/// Document with invalid alias.
///
/// Alias references a non-existent identifier.
fn invalid_alias() -> Document {
    let mut doc = Document::new((1, 0));

    // Alias points to non-existent root item
    doc.aliases.insert("my_alias".to_string(), "nonexistent_item".to_string());

    doc
}

/// Edge case: Document with extremely deep nesting.
///
/// Tests stack overflow and recursion limits.
pub fn deeply_nested_document(depth: usize) -> Document {
    let mut doc = Document::new((1, 0));

    // Create a deeply nested structure
    let mut current_children = BTreeMap::new();

    // Build from the bottom up
    for i in (0..depth).rev() {
        let node = Node {
            type_name: "Level".to_string(),
            id: format!("node_{}", i),
            fields: vec![
                Value::String(format!("node_{}", i)),
                Value::Int(i as i64),
            ],
            children: current_children.clone(),
            child_count: None,
        };

        current_children = BTreeMap::new();
        current_children.insert("nested".to_string(), vec![node]);
    }

    // Create the root list
    let mut list = MatrixList::new("Level", vec!["id".to_string(), "level".to_string()]);

    if let Some(nodes) = current_children.get("nested") {
        for node in nodes {
            list.rows.push(node.clone());
        }
    }

    doc.root.insert("levels".to_string(), Item::List(list));
    doc.structs.insert(
        "Level".to_string(),
        vec!["id".to_string(), "level".to_string()],
    );
    doc.nests.insert("Level".to_string(), "Level".to_string());

    doc
}

/// Edge case: Document with extremely wide structure.
///
/// Tests memory and performance limits.
pub fn wide_document(width: usize) -> Document {
    let mut doc = Document::new((1, 0));

    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    for i in 0..width {
        list.add_row(Node::new(
            "Item",
            format!("item_{}", i),
            vec![
                Value::String(format!("item_{}", i)),
                Value::Int(i as i64),
            ],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));
    doc.structs.insert(
        "Item".to_string(),
        vec!["id".to_string(), "value".to_string()],
    );

    doc
}

/// Edge case: Document with very long strings.
///
/// Tests string handling and buffer limits.
pub fn long_string_document(length: usize) -> Document {
    let mut doc = Document::new((1, 0));

    let long_string = "x".repeat(length);

    doc.root.insert(
        "long_text".to_string(),
        Item::Scalar(Value::String(long_string)),
    );

    doc
}

/// Edge case: Document with many references.
///
/// Tests reference resolution performance.
pub fn many_references_document(count: usize) -> Document {
    let mut doc = Document::new((1, 0));

    // Create target nodes
    let mut targets = MatrixList::new("Target", vec!["id".to_string(), "name".to_string()]);

    for i in 0..count {
        targets.add_row(Node::new(
            "Target",
            format!("target_{}", i),
            vec![
                Value::String(format!("target_{}", i)),
                Value::String(format!("Target {}", i)),
            ],
        ));
    }

    // Create referencing nodes
    let mut refs = MatrixList::new("Ref", vec!["id".to_string(), "target".to_string()]);

    for i in 0..count {
        refs.add_row(Node::new(
            "Ref",
            format!("ref_{}", i),
            vec![
                Value::String(format!("ref_{}", i)),
                Value::Reference(hedl_core::Reference {
                    type_name: Some("Target".to_string()),
                    id: format!("target_{}", i % count),
                }),
            ],
        ));
    }

    doc.root.insert("targets".to_string(), Item::List(targets));
    doc.root.insert("refs".to_string(), Item::List(refs));

    doc.structs.insert(
        "Target".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    doc.structs.insert(
        "Ref".to_string(),
        vec!["id".to_string(), "target".to_string()],
    );

    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_hedl_samples_non_empty() {
        let samples = invalid_hedl_samples();
        assert!(!samples.is_empty(), "Should have invalid HEDL samples");

        for (name, _text) in samples {
            assert!(!name.is_empty(), "Sample name should not be empty");
        }
    }

    #[test]
    fn test_invalid_expression_samples_non_empty() {
        let samples = invalid_expression_samples();
        assert!(!samples.is_empty(), "Should have invalid expression samples");

        for (desc, _expr) in samples {
            assert!(!desc.is_empty(), "Description should not be empty");
        }
    }

    #[test]
    fn test_semantically_invalid_docs() {
        let docs = semantically_invalid_docs();
        assert!(!docs.is_empty(), "Should have semantically invalid docs");

        for (name, doc) in docs {
            assert!(!name.is_empty(), "Doc name should not be empty");
            assert_eq!(doc.version, (1, 0), "Should have valid version");
        }
    }

    #[test]
    fn test_deeply_nested_document() {
        let doc = deeply_nested_document(10);
        assert_eq!(doc.version, (1, 0));
        assert!(doc.root.contains_key("levels"));
    }

    #[test]
    fn test_wide_document() {
        let doc = wide_document(100);
        assert_eq!(doc.version, (1, 0));

        if let Some(Item::List(list)) = doc.root.get("items") {
            assert_eq!(list.rows.len(), 100);
        } else {
            panic!("Expected items list");
        }
    }

    #[test]
    fn test_long_string_document() {
        let doc = long_string_document(10000);

        if let Some(Item::Scalar(Value::String(s))) = doc.root.get("long_text") {
            assert_eq!(s.len(), 10000);
        } else {
            panic!("Expected long_text string");
        }
    }

    #[test]
    fn test_many_references_document() {
        let doc = many_references_document(50);

        if let Some(Item::List(list)) = doc.root.get("refs") {
            assert_eq!(list.rows.len(), 50);
        } else {
            panic!("Expected refs list");
        }
    }
}
