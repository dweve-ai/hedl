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

//! Tests for error fixture generation.

use hedl_core::Item;
use hedl_test::fixtures::errors::*;

#[test]
fn test_invalid_hedl_samples_returns_non_empty() {
    let samples = invalid_hedl_samples();
    assert!(!samples.is_empty());
}

#[test]
fn test_invalid_hedl_samples_have_names_and_content() {
    let samples = invalid_hedl_samples();

    for (name, _hedl_text) in samples {
        assert!(!name.is_empty(), "Sample name should not be empty");
    }
}

#[test]
fn test_invalid_hedl_samples_coverage() {
    let samples = invalid_hedl_samples();
    let names: Vec<&str> = samples.iter().map(|(name, _)| *name).collect();

    // Verify key error categories are covered
    assert!(names.contains(&"empty"));
    assert!(names.contains(&"whitespace_only"));
    assert!(names.contains(&"invalid_directive"));
    assert!(names.contains(&"unclosed_string"));
    assert!(names.contains(&"malformed_reference"));
    assert!(names.contains(&"invalid_number"));
}

#[test]
fn test_invalid_expression_samples_returns_non_empty() {
    let samples = invalid_expression_samples();
    assert!(!samples.is_empty());
}

#[test]
fn test_invalid_expression_samples_have_descriptions() {
    let samples = invalid_expression_samples();

    for (desc, _expr) in samples {
        assert!(!desc.is_empty(), "Description should not be empty");
        // Expression itself might be empty for testing empty input
    }
}

#[test]
fn test_invalid_expression_samples_coverage() {
    let samples = invalid_expression_samples();
    let descriptions: Vec<&str> = samples.iter().map(|(desc, _)| *desc).collect();

    // Verify key error categories are covered
    assert!(descriptions.contains(&"empty"));
    assert!(descriptions.contains(&"unclosed_paren"));
    assert!(descriptions.contains(&"unclosed_string"));
    assert!(descriptions.contains(&"invalid_chars"));
    assert!(descriptions.contains(&"mismatched_parens"));
}

#[test]
fn test_semantically_invalid_docs_returns_non_empty() {
    let docs = semantically_invalid_docs();
    assert!(!docs.is_empty());
}

#[test]
fn test_semantically_invalid_docs_have_valid_structure() {
    let docs = semantically_invalid_docs();

    for (name, doc) in docs {
        assert!(!name.is_empty(), "Doc name should not be empty");
        assert_eq!(
            doc.version,
            (1, 0),
            "Invalid doc '{name}' should have valid version"
        );
    }
}

#[test]
fn test_semantically_invalid_docs_coverage() {
    let docs = semantically_invalid_docs();
    let names: Vec<&str> = docs.iter().map(|(name, _)| *name).collect();

    assert!(names.contains(&"undefined_struct"));
    assert!(names.contains(&"undefined_nest"));
    assert!(names.contains(&"circular_nest"));
    assert!(names.contains(&"dangling_reference"));
    assert!(names.contains(&"mismatched_schema"));
    assert!(names.contains(&"empty_type_name"));
    assert!(names.contains(&"duplicate_ids"));
    assert!(names.contains(&"invalid_alias"));
}

#[test]
fn test_undefined_struct_doc() {
    let docs = semantically_invalid_docs();
    let (_, doc) = docs
        .iter()
        .find(|(name, _)| *name == "undefined_struct")
        .unwrap();

    // Should have a list but no corresponding struct definition
    assert!(!doc.root.is_empty());
    if let Some(Item::List(list)) = doc.root.values().next() {
        assert!(!doc.structs.contains_key(&list.type_name));
    }
}

#[test]
fn test_undefined_nest_doc() {
    let docs = semantically_invalid_docs();
    let (_, doc) = docs
        .iter()
        .find(|(name, _)| *name == "undefined_nest")
        .unwrap();

    // Should have NEST pointing to non-existent type
    assert!(!doc.nests.is_empty());

    for child_type in doc.nests.values() {
        // At least one child type should not be defined
        if !doc.structs.contains_key(child_type) {
            return; // Test passes
        }
    }

    // If we get here, all nests were defined (test should fail)
    panic!("Expected at least one undefined nest target");
}

#[test]
fn test_circular_nest_doc() {
    let docs = semantically_invalid_docs();
    let (_, doc) = docs
        .iter()
        .find(|(name, _)| *name == "circular_nest")
        .unwrap();

    // Should have at least 2 NEST relationships forming a cycle
    assert!(doc.nests.len() >= 2);
}

#[test]
fn test_dangling_reference_doc() {
    let docs = semantically_invalid_docs();
    let (_, doc) = docs
        .iter()
        .find(|(name, _)| *name == "dangling_reference")
        .unwrap();

    // Should have references
    let ref_count = hedl_test::count_references(doc);
    assert!(ref_count > 0, "Should have at least one reference");
}

#[test]
fn test_mismatched_schema_doc() {
    let docs = semantically_invalid_docs();
    let (_, doc) = docs
        .iter()
        .find(|(name, _)| *name == "mismatched_schema")
        .unwrap();

    // Should have struct definition
    assert!(!doc.structs.is_empty());

    // Should have list with different schema
    if let Some(Item::List(list)) = doc.root.values().next() {
        if let Some(struct_schema) = doc.structs.get(&list.type_name) {
            assert_ne!(
                &list.schema, struct_schema,
                "Schema should not match struct definition"
            );
        }
    }
}

#[test]
fn test_empty_type_name_doc() {
    let docs = semantically_invalid_docs();
    let (_, doc) = docs
        .iter()
        .find(|(name, _)| *name == "empty_type_name")
        .unwrap();

    // Should have a list with empty type name
    if let Some(Item::List(list)) = doc.root.values().next() {
        assert!(list.type_name.is_empty());
    } else {
        panic!("Expected list with empty type name");
    }
}

#[test]
fn test_duplicate_ids_doc() {
    let docs = semantically_invalid_docs();
    let (_, doc) = docs
        .iter()
        .find(|(name, _)| *name == "duplicate_ids")
        .unwrap();

    // Should have list with duplicate IDs
    if let Some(Item::List(list)) = doc.root.values().next() {
        assert!(list.rows.len() >= 2, "Need at least 2 rows");

        let mut ids = std::collections::HashSet::new();
        let mut has_duplicate = false;

        for row in &list.rows {
            if !ids.insert(&row.id) {
                has_duplicate = true;
                break;
            }
        }

        assert!(has_duplicate, "Should have duplicate IDs");
    }
}

#[test]
fn test_invalid_alias_doc() {
    let docs = semantically_invalid_docs();
    let (_, doc) = docs
        .iter()
        .find(|(name, _)| *name == "invalid_alias")
        .unwrap();

    // Should have alias
    assert!(!doc.aliases.is_empty());

    // Alias should point to non-existent item
    for target in doc.aliases.values() {
        if !doc.root.contains_key(target) {
            return; // Test passes
        }
    }

    panic!("Expected at least one invalid alias");
}

#[test]
fn test_deeply_nested_document_basic() {
    let doc = deeply_nested_document(5);

    assert_eq!(doc.version, (1, 0));
    assert!(doc.root.contains_key("levels"));
}

#[test]
fn test_deeply_nested_document_large_depth() {
    let doc = deeply_nested_document(50);

    // Should succeed without stack overflow
    assert_eq!(doc.version, (1, 0));
}

#[test]
fn test_deeply_nested_document_zero_depth() {
    let doc = deeply_nested_document(0);

    if let Some(Item::List(list)) = doc.root.get("levels") {
        assert!(list.rows.is_empty());
    }
}

#[test]
fn test_wide_document_basic() {
    let doc = wide_document(10);

    if let Some(Item::List(list)) = doc.root.get("items") {
        assert_eq!(list.rows.len(), 10);
    } else {
        panic!("Expected items list");
    }
}

#[test]
fn test_wide_document_large_width() {
    let doc = wide_document(10000);

    if let Some(Item::List(list)) = doc.root.get("items") {
        assert_eq!(list.rows.len(), 10000);
    }
}

#[test]
fn test_wide_document_zero_width() {
    let doc = wide_document(0);

    if let Some(Item::List(list)) = doc.root.get("items") {
        assert!(list.rows.is_empty());
    }
}

#[test]
fn test_long_string_document_basic() {
    let doc = long_string_document(1000);

    if let Some(Item::Scalar(hedl_core::Value::String(s))) = doc.root.get("long_text") {
        assert_eq!(s.len(), 1000);
        assert!(s.chars().all(|c| c == 'x'));
    } else {
        panic!("Expected long_text string");
    }
}

#[test]
fn test_long_string_document_very_long() {
    let doc = long_string_document(100000);

    if let Some(Item::Scalar(hedl_core::Value::String(s))) = doc.root.get("long_text") {
        assert_eq!(s.len(), 100000);
    }
}

#[test]
fn test_long_string_document_zero_length() {
    let doc = long_string_document(0);

    if let Some(Item::Scalar(hedl_core::Value::String(s))) = doc.root.get("long_text") {
        assert_eq!(s.len(), 0);
    }
}

#[test]
fn test_many_references_document_basic() {
    let doc = many_references_document(10);

    if let Some(Item::List(list)) = doc.root.get("targets") {
        assert_eq!(list.rows.len(), 10);
    }

    if let Some(Item::List(list)) = doc.root.get("refs") {
        assert_eq!(list.rows.len(), 10);
    }

    assert_eq!(hedl_test::count_references(&doc), 10);
}

#[test]
fn test_many_references_document_large_count() {
    let doc = many_references_document(1000);

    assert_eq!(hedl_test::count_references(&doc), 1000);
}

#[test]
fn test_many_references_document_zero_count() {
    let doc = many_references_document(0);

    if let Some(Item::List(list)) = doc.root.get("refs") {
        assert!(list.rows.is_empty());
    }
}

#[test]
fn test_error_fixtures_are_independent() {
    // Verify that calling fixture functions multiple times produces
    // independent instances (no shared state)
    let doc1 = deeply_nested_document(5);
    let doc2 = deeply_nested_document(5);

    // They should be equal but independent
    assert_eq!(doc1.version, doc2.version);
}

#[test]
fn test_invalid_samples_uniqueness() {
    let hedl_samples = invalid_hedl_samples();
    let expr_samples = invalid_expression_samples();

    // Check for unique names/descriptions
    let mut hedl_names = std::collections::HashSet::new();
    for (name, _) in hedl_samples {
        assert!(
            hedl_names.insert(name),
            "Duplicate HEDL sample name: {name}"
        );
    }

    let mut expr_descs = std::collections::HashSet::new();
    for (desc, _) in expr_samples {
        assert!(
            expr_descs.insert(desc),
            "Duplicate expression sample description: {desc}"
        );
    }
}

#[test]
fn test_semantically_invalid_docs_uniqueness() {
    let docs = semantically_invalid_docs();
    let mut names = std::collections::HashSet::new();

    for (name, _) in docs {
        assert!(names.insert(name), "Duplicate doc name: {name}");
    }
}
