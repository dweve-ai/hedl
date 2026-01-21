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

//! Property-based test generators for hedl-lint
//!
//! This module provides proptest strategies for generating valid and invalid
//! HEDL documents to test lint rules systematically.
//!
//! # Generator Strategy
//!
//! We use separate generators for different violation scenarios:
//! - `valid_*` generators: Produce documents that should NOT trigger diagnostics
//! - `*_violation` generators: Produce documents designed to trigger specific rules
//! - `random_*` generators: Produce arbitrary valid documents for general testing
//!
//! # Example Generated Documents
//!
//! ## Short ID Violation
//! ```text
//! %VERSION: 1.0
//! ---
//! items: @User[id]
//!   | a
//!   | b
//! ```
//!
//! ## Valid Document
//! ```text
//! %VERSION: 1.0
//! ---
//! users: @User[id, name]
//!   | alice_smith, "Alice Smith"
//!   | bob_jones, "Bob Jones"
//! ```

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use proptest::prelude::*;
use std::collections::BTreeMap;

/// Generate valid HEDL identifiers (lowercase, underscore, digits)
///
/// Properties:
/// - Starts with lowercase letter or underscore
/// - Contains only [a-z0-9_]
/// - Length: 1-50 characters (reasonable range)
///
/// # Examples
/// - `user_id`
/// - `_private`
/// - `item123`
pub fn valid_id() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z_][a-z0-9_]{0,49}")
        .expect("valid_id regex failed")
        .prop_filter("Exclude double underscore prefix", |s| !s.starts_with("__"))
}

/// Generate short IDs that should trigger id-naming hints
///
/// Single-character IDs are considered non-descriptive and should
/// trigger the id-naming rule with Hint severity.
///
/// # Examples
/// - `a`
/// - `x`
/// - `_`
pub fn short_id() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z_]").expect("short_id regex failed")
}

/// Generate numeric-only IDs that should trigger id-naming hints
///
/// IDs like "123" or "42" are considered non-descriptive and should
/// trigger the id-naming rule. Note that mixed alphanumeric like "user123"
/// does NOT trigger this rule.
///
/// # Examples
/// - `123`
/// - `42`
/// - `1_2_3`
pub fn numeric_id() -> impl Strategy<Value = String> {
    prop::string::string_regex("[0-9_]{1,20}")
        .expect("numeric_id regex failed")
        .prop_filter("Must contain at least one digit", |s| {
            s.chars().any(|c| c.is_ascii_digit())
        })
}

/// Generate descriptive IDs that should NOT trigger hints
///
/// These are properly formatted, multi-character, alphanumeric IDs
/// that follow HEDL best practices.
///
/// # Examples
/// - `alice_smith`
/// - `user123`
/// - `item_abc`
pub fn good_id() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{2,30}")
        .expect("good_id regex failed")
        .prop_filter("Must have at least one letter after first char", |s| {
            s.chars().skip(1).any(|c| c.is_ascii_lowercase())
        })
}

/// Generate valid HEDL type names (`PascalCase`)
///
/// Type names must start with an uppercase letter and contain only
/// alphanumeric characters.
///
/// # Examples
/// - `User`
/// - `Product`
/// - `OrderItem`
pub fn type_name() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-zA-Z0-9]{1,30}").expect("type_name regex failed")
}

/// Generate HEDL string values (printable ASCII, no control chars)
///
/// Generates safe string values suitable for HEDL documents.
///
/// # Examples
/// - `"Alice Smith"`
/// - `"Product 123"`
/// - `"Hello, World!"`
pub fn hedl_string() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9 .,!?()\-_]{0,100}").expect("hedl_string regex failed")
}

/// Generate simple document with one scalar field
///
/// Produces minimal valid HEDL documents with a single key-value pair.
/// Useful for testing basic lint functionality.
pub fn simple_document() -> impl Strategy<Value = Document> {
    (valid_id(), hedl_string()).prop_map(|(key, value)| {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert(key, Item::Scalar(Value::String(value.into())));
        doc
    })
}

/// Generate document with multiple scalar fields (no duplicates)
///
/// Produces documents with 1-20 unique key-value pairs. Keys are
/// guaranteed to be unique.
pub fn multi_field_document() -> impl Strategy<Value = Document> {
    prop::collection::hash_set((valid_id(), hedl_string()), 1..20).prop_map(|fields| {
        let mut doc = Document::new((1, 0));
        for (key, value) in fields {
            doc.root
                .insert(key, Item::Scalar(Value::String(value.into())));
        }
        doc
    })
}

/// Generate document with matrix list containing GOOD IDs (should NOT trigger hints)
///
/// This generator creates documents with descriptive, multi-character IDs
/// that conform to best practices and should NOT trigger the id-naming rule.
pub fn valid_list_document() -> impl Strategy<Value = Document> {
    (type_name(), prop::collection::hash_set(good_id(), 1..10)).prop_map(|(type_name, ids)| {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new(&type_name, vec!["id".to_string()]);

        for id in ids {
            list.add_row(Node::new(&type_name, &id, vec![]));
        }

        doc.root.insert("items".to_string(), Item::List(list));
        doc
    })
}

/// Generate document with matrix list containing SHORT IDs (triggers hints)
///
/// This generator creates documents with single-character IDs that should
/// trigger the id-naming rule with Hint severity.
pub fn short_id_list_document() -> impl Strategy<Value = Document> {
    (type_name(), prop::collection::hash_set(short_id(), 1..20)).prop_map(|(type_name, ids)| {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new(&type_name, vec!["id".to_string()]);

        for id in ids {
            list.add_row(Node::new(&type_name, &id, vec![]));
        }

        doc.root.insert("items".to_string(), Item::List(list));
        doc
    })
}

/// Generate document with matrix list containing NUMERIC IDs (triggers hints)
///
/// This generator creates documents with numeric-only IDs like "123" that
/// should trigger the id-naming rule.
pub fn numeric_id_list_document() -> impl Strategy<Value = Document> {
    (type_name(), prop::collection::hash_set(numeric_id(), 1..10)).prop_map(|(type_name, ids)| {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new(&type_name, vec!["id".to_string()]);

        for id in ids {
            list.add_row(Node::new(&type_name, &id, vec![]));
        }

        doc.root.insert("items".to_string(), Item::List(list));
        doc
    })
}

/// Generate document with UNUSED schemas
///
/// This generator creates documents with schema definitions that are never
/// used in any matrix list, triggering the unused-schema rule.
pub fn unused_schema_document() -> impl Strategy<Value = Document> {
    (
        prop::collection::hash_set(type_name(), 1..10), // unused schemas
        prop::collection::hash_set(type_name(), 1..5),  // used schemas
    )
        .prop_map(|(unused, used)| {
            let mut doc = Document::new((1, 0));

            // Define unused schemas
            for schema in unused {
                doc.structs.insert(schema, vec!["id".to_string()]);
            }

            // Define and use schemas
            for schema in &used {
                doc.structs.insert(schema.clone(), vec!["id".to_string()]);

                let mut list = MatrixList::new(schema, vec!["id".to_string()]);
                list.add_row(Node::new(schema, "item1", vec![]));

                doc.root
                    .insert(format!("{}_items", schema.to_lowercase()), Item::List(list));
            }

            doc
        })
}

/// Generate document with EMPTY matrix lists
///
/// This generator creates documents with matrix lists that have columns
/// defined but no rows, triggering the empty-list rule.
pub fn empty_list_document() -> impl Strategy<Value = Document> {
    (type_name(), 1..5usize).prop_map(|(type_name, count)| {
        let mut doc = Document::new((1, 0));

        for i in 0..count {
            let list = MatrixList::new(&type_name, vec!["id".to_string()]);
            doc.root.insert(format!("list{i}"), Item::List(list));
        }

        doc
    })
}

/// Generate document with UNQUALIFIED references in key-value context
///
/// This generator creates documents with local references like `@id` instead
/// of qualified references like `@Type:id`, triggering the unqualified-kv-ref rule.
pub fn unqualified_ref_document() -> impl Strategy<Value = Document> {
    (valid_id(), valid_id()).prop_map(|(key, target_id)| {
        let mut doc = Document::new((1, 0));

        // Add unqualified reference
        doc.root.insert(
            key,
            Item::Scalar(Value::Reference(Reference::local(&target_id))),
        );

        doc
    })
}

/// Generate document with QUALIFIED references (should NOT trigger warnings)
///
/// This generator creates documents with properly qualified references like
/// `@User:alice`, which should NOT trigger the unqualified-kv-ref rule.
pub fn qualified_ref_document() -> impl Strategy<Value = Document> {
    (valid_id(), type_name(), valid_id()).prop_map(|(key, type_name, target_id)| {
        let mut doc = Document::new((1, 0));

        // Add qualified reference
        doc.root.insert(
            key,
            Item::Scalar(Value::Reference(Reference::qualified(
                &type_name, &target_id,
            ))),
        );

        doc
    })
}

/// Generate document with nested objects
///
/// Creates documents with nested object structures to test recursion
/// depth handling and nested diagnostics.
pub fn nested_object_document() -> impl Strategy<Value = Document> {
    (1..5usize, good_id()).prop_map(|(depth, id)| {
        let mut doc = Document::new((1, 0));

        let mut current = &mut doc.root;
        for i in 0..depth {
            let mut nested = BTreeMap::new();
            if i == depth - 1 {
                // Leaf: add a list with good ID
                let mut list = MatrixList::new("Test", vec!["id".to_string()]);
                list.add_row(Node::new("Test", &id, vec![]));
                nested.insert("items".to_string(), Item::List(list));
            }
            let key = format!("level{i}");
            current.insert(key.clone(), Item::Object(nested));
            current = if let Some(Item::Object(ref mut obj)) = current.get_mut(&key) {
                obj
            } else {
                unreachable!()
            };
        }

        doc
    })
}

/// Generate document with nested objects containing violations
///
/// Creates documents with deeply nested structures that contain violations
/// (short IDs) to test recursive violation detection.
pub fn nested_violation_document() -> impl Strategy<Value = Document> {
    (1..5usize, short_id()).prop_map(|(depth, id)| {
        let mut doc = Document::new((1, 0));

        let mut current = &mut doc.root;
        for i in 0..depth {
            let mut nested = BTreeMap::new();
            if i == depth - 1 {
                // Leaf: add a list with short ID (violation)
                let mut list = MatrixList::new("Test", vec!["id".to_string()]);
                list.add_row(Node::new("Test", &id, vec![]));
                nested.insert("items".to_string(), Item::List(list));
            }
            let key = format!("level{i}");
            current.insert(key.clone(), Item::Object(nested));
            current = if let Some(Item::Object(ref mut obj)) = current.get_mut(&key) {
                obj
            } else {
                unreachable!()
            };
        }

        doc
    })
}

/// Generate well-formed document with NO violations
///
/// This generator creates documents that follow all best practices:
/// - Descriptive IDs (multi-character, alphanumeric)
/// - All schemas are used
/// - No empty lists
/// - Qualified references
///
/// These documents should produce ZERO diagnostics.
pub fn well_formed_document() -> impl Strategy<Value = Document> {
    (
        prop::collection::hash_set(type_name(), 1..3),
        prop::collection::hash_set(good_id(), 2..5),
    )
        .prop_map(|(type_names, ids)| {
            let mut doc = Document::new((1, 0));
            let ids_vec: Vec<_> = ids.into_iter().collect();

            for type_name in type_names {
                // Define schema
                doc.structs
                    .insert(type_name.clone(), vec!["id".to_string()]);

                // Use schema with non-empty list
                let mut list = MatrixList::new(&type_name, vec!["id".to_string()]);
                for id in &ids_vec {
                    list.add_row(Node::new(&type_name, id, vec![]));
                }

                doc.root.insert(
                    format!("{}_items", type_name.to_lowercase()),
                    Item::List(list),
                );

                // Add qualified reference
                if let Some(first_id) = ids_vec.first() {
                    doc.root.insert(
                        format!("{}_ref", type_name.to_lowercase()),
                        Item::Scalar(Value::Reference(Reference::qualified(&type_name, first_id))),
                    );
                }
            }

            doc
        })
}

/// Generate document with MIXED violations (multiple rule violations)
///
/// This generator creates documents that violate multiple lint rules
/// simultaneously, useful for testing diagnostic aggregation and ordering.
///
/// Violations include:
/// - Short IDs (id-naming)
/// - Empty lists (empty-list)
/// - Unused schemas (unused-schema)
/// - Unqualified references (unqualified-kv-ref)
pub fn mixed_violation_document() -> impl Strategy<Value = Document> {
    (
        prop::collection::hash_set(short_id(), 1..5),
        prop::collection::hash_set(type_name(), 1..3),
        type_name(),
    )
        .prop_map(|(short_ids, unused_schemas, used_type)| {
            let mut doc = Document::new((1, 0));

            // Add short IDs (id-naming violation)
            let mut list = MatrixList::new(&used_type, vec!["id".to_string()]);
            for id in short_ids {
                list.add_row(Node::new(&used_type, &id, vec![]));
            }
            doc.root.insert("items".to_string(), Item::List(list));

            // Add empty list (empty-list violation)
            let empty_list = MatrixList::new("Empty", vec!["id".to_string()]);
            doc.root
                .insert("empty_items".to_string(), Item::List(empty_list));

            // Add unused schemas (unused-schema violation)
            for schema in unused_schemas {
                doc.structs.insert(schema, vec!["id".to_string()]);
            }

            // Define used type
            doc.structs
                .insert(used_type.clone(), vec!["id".to_string()]);
            doc.structs
                .insert("Empty".to_string(), vec!["id".to_string()]);

            // Add unqualified reference (unqualified-kv-ref violation)
            doc.root.insert(
                "ref".to_string(),
                Item::Scalar(Value::Reference(Reference::local("some_id"))),
            );

            doc
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that generators produce valid documents
    #[test]
    fn test_generators_produce_valid_documents() {
        proptest!(|(
            doc1 in simple_document(),
            doc2 in multi_field_document(),
            doc3 in valid_list_document(),
            doc4 in short_id_list_document(),
            doc5 in numeric_id_list_document(),
            doc6 in unused_schema_document(),
            doc7 in empty_list_document(),
            doc8 in unqualified_ref_document(),
            doc9 in qualified_ref_document(),
            doc10 in well_formed_document(),
            doc11 in mixed_violation_document(),
            doc12 in nested_object_document(),
            doc13 in nested_violation_document()
        )| {
            // All generators should produce valid Document structures
            // (no panics, no invalid states)
            assert_eq!(doc1.version, (1, 0));
            assert_eq!(doc2.version, (1, 0));
            assert_eq!(doc3.version, (1, 0));
            assert_eq!(doc4.version, (1, 0));
            assert_eq!(doc5.version, (1, 0));
            assert_eq!(doc6.version, (1, 0));
            assert_eq!(doc7.version, (1, 0));
            assert_eq!(doc8.version, (1, 0));
            assert_eq!(doc9.version, (1, 0));
            assert_eq!(doc10.version, (1, 0));
            assert_eq!(doc11.version, (1, 0));
            assert_eq!(doc12.version, (1, 0));
            assert_eq!(doc13.version, (1, 0));
        });
    }

    /// Test ID generators produce correct formats
    #[test]
    fn test_id_generators() {
        proptest!(|(
            vid in valid_id(),
            sid in short_id(),
            nid in numeric_id(),
            gid in good_id()
        )| {
            // Valid IDs match pattern
            assert!(vid.chars().next().unwrap().is_ascii_lowercase() || vid.starts_with('_'));
            assert!(vid.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));

            // Short IDs are single character
            assert_eq!(sid.len(), 1);

            // Numeric IDs contain at least one digit
            assert!(nid.chars().any(|c| c.is_ascii_digit()));
            assert!(nid.chars().all(|c| c.is_ascii_digit() || c == '_'));

            // Good IDs are multi-character with letters
            assert!(gid.len() >= 3);
            assert!(gid.chars().skip(1).any(|c| c.is_ascii_lowercase()));
        });
    }
}
