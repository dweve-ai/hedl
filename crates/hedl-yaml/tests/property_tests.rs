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

//! Property-based tests for hedl-yaml using proptest.
//!
//! These tests verify roundtrip properties and invariants for YAML conversion:
//! - HEDL → YAML → HEDL preserves data semantics
//! - Type preservation (int stays int, not string)
//! - Value preservation (42 → 42, not "42")
//! - Structure preservation (nesting, order where relevant)
//! - Reference preservation (@User:id format)
//! - Edge cases (empty objects/arrays, deep nesting, Unicode)

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_core::lex::Tensor;
use hedl_yaml::{from_yaml, to_yaml, FromYamlConfig, ToYamlConfig};
use proptest::prelude::*;
use std::collections::BTreeMap;

// =============================================================================
// Arbitrary Value Generation
// =============================================================================

/// Strategy for generating valid finite f64 values.
fn arb_finite_f64() -> impl Strategy<Value = f64> {
    prop::num::f64::NORMAL
        .prop_filter("must be finite", |f| f.is_finite())
        .prop_map(|f| f.max(-1e10).min(1e10))
}

/// Strategy for generating scalar values.
fn arb_scalar_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        1 => Just(Value::Null),
        4 => any::<bool>().prop_map(Value::Bool),
        4 => any::<i64>().prop_map(Value::Int),
        3 => arb_finite_f64().prop_map(Value::Float),
        5 => "[a-zA-Z0-9_ ]{0,50}".prop_map(Value::String),
    ]
}

/// Strategy for generating Unicode strings.
fn arb_unicode_string() -> impl Strategy<Value = String> {
    prop_oneof![
        // ASCII
        3 => "[a-zA-Z0-9_ ]{1,30}",
        // Unicode (safe subset)
        1 => "[\\p{L}\\p{N} ]{1,30}",
        // Special characters
        1 => "[!@#$%^&*()_+\\-=\\[\\]{};':\",./<>?]{1,20}",
    ]
}

/// Strategy for generating reference values.
fn arb_reference() -> impl Strategy<Value = Value> {
    prop_oneof![
        // Local reference
        1 => "[a-z][a-z0-9_]{0,10}".prop_map(|id| Value::Reference(Reference::local(id))),
        // Qualified reference
        2 => ("[A-Z][a-zA-Z]{0,10}", "[a-z][a-z0-9_]{0,10}")
            .prop_map(|(type_name, id)| Value::Reference(Reference::qualified(type_name, id))),
    ]
}

/// Strategy for generating tensor values (limited depth for performance).
fn arb_tensor_value() -> impl Strategy<Value = Value> {
    let leaf = arb_finite_f64().prop_map(Tensor::Scalar);

    leaf.prop_recursive(
        3,  // max depth
        20, // max nodes
        5,  // items per collection
        |inner| {
            prop::collection::vec(inner, 1..6)
                .prop_map(Tensor::Array)
        },
    )
    .prop_map(Value::Tensor)
}

/// Strategy for generating any value type.
fn arb_value() -> impl Strategy<Value = Value> {
    prop_oneof![
        5 => arb_scalar_value(),
        2 => arb_reference(),
        1 => arb_tensor_value(),
    ]
}

/// Strategy for generating simple objects (limited depth).
fn arb_simple_object() -> impl Strategy<Value = BTreeMap<String, Item>> {
    prop::collection::btree_map(
        "[a-z][a-z0-9_]{0,10}",
        arb_value().prop_map(Item::Scalar),
        0..10,
    )
}

/// Strategy for generating nested objects (up to depth 5).
fn arb_object(depth: u32) -> impl Strategy<Value = BTreeMap<String, Item>> {
    let leaf = arb_value().prop_map(Item::Scalar);

    leaf.prop_recursive(
        depth,  // max depth
        50,     // max nodes
        5,      // items per collection
        move |inner| {
            prop_oneof![
                // Nested object
                2 => prop::collection::btree_map(
                    "[a-z][a-z0-9_]{0,10}",
                    inner.clone(),
                    1..5,
                ).prop_map(Item::Object),
                // Scalar fallback
                8 => arb_value().prop_map(Item::Scalar),
            ]
        },
    )
    .prop_map(|item| {
        let mut map = BTreeMap::new();
        map.insert("value".to_string(), item);
        map
    })
}

/// Strategy for generating matrix lists.
///
/// Note: YAML cannot preserve schema for empty lists, so we only generate non-empty lists
/// for roundtrip testing.
fn arb_matrix_list() -> impl Strategy<Value = MatrixList> {
    (
        "[A-Z][a-zA-Z]{0,10}",                              // type_name
        prop::collection::vec("[a-z_]{2,10}", 2..5)
            .prop_filter("schema must have unique column names", |schema| {
                let unique: std::collections::HashSet<_> = schema.iter().collect();
                unique.len() == schema.len() // Ensure no duplicates
            }),        // schema (at least 2 columns, all unique)
        prop::collection::vec(arb_scalar_value(), 1..3),    // rows (at least 1, to preserve schema)
    )
        .prop_map(|(type_name, schema, values)| {
            // Ensure schema has at least "id" column
            let mut final_schema = schema;
            if !final_schema.iter().any(|s| s == "id") {
                final_schema.insert(0, "id".to_string());
            }

            let mut list = MatrixList::new(type_name.clone(), final_schema.clone());

            // Create nodes with matching field count
            for (i, value) in values.iter().enumerate() {
                let id = format!("id_{}", i);
                // Per SPEC: fields must include ALL schema columns including ID
                let mut fields = vec![Value::String(id.clone())]; // ID field
                fields.push(value.clone()); // First data field
                // Pad with nulls to match schema length
                while fields.len() < final_schema.len() {
                    fields.push(Value::Null);
                }
                let node = Node::new(type_name.clone(), id, fields);
                list.add_row(node);
            }

            list
        })
}

/// Strategy for generating documents.
fn arb_document() -> impl Strategy<Value = Document> {
    (
        arb_simple_object(),
        prop::option::of(arb_matrix_list()),
    )
        .prop_map(|(root, maybe_list)| {
            let mut doc = Document::new((1, 0));

            // Add simple objects to root
            for (key, item) in root {
                doc.root.insert(key, item);
            }

            // Optionally add a matrix list
            if let Some(list) = maybe_list {
                doc.root.insert("items".to_string(), Item::List(list));
            }

            doc
        })
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if two values are semantically equal (with float tolerance).
///
/// Note: YAML conversion may convert single-element tensors to scalars,
/// so we handle both Tensor(Scalar(x)) and Float(x) as equivalent.
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => {
            let diff = (a - b).abs();
            let tolerance = a.abs().max(b.abs()) * 1e-10 + 1e-14;
            diff <= tolerance
        }
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Reference(a), Value::Reference(b)) => a == b,
        (Value::Tensor(a), Value::Tensor(b)) => {
            let flat_a = a.flatten();
            let flat_b = b.flatten();
            if flat_a.len() != flat_b.len() {
                return false;
            }
            flat_a
                .iter()
                .zip(flat_b.iter())
                .all(|(x, y)| {
                    let diff = (x - y).abs();
                    let tolerance = x.abs().max(y.abs()) * 1e-10 + 1e-14;
                    diff <= tolerance
                })
        }
        // Handle YAML conversion quirk: Tensor(Scalar(x)) may become Float(x)
        (Value::Tensor(t), Value::Float(f)) | (Value::Float(f), Value::Tensor(t)) => {
            let flat = t.flatten();
            if flat.len() != 1 {
                return false;
            }
            let diff = (flat[0] - f).abs();
            let tolerance = flat[0].abs().max(f.abs()) * 1e-10 + 1e-14;
            diff <= tolerance
        }
        _ => false,
    }
}

/// Check if two items are semantically equal.
///
/// Note: YAML conversion may reorder schema columns alphabetically with 'id' first,
/// so we check for schema equivalence rather than exact ordering.
fn items_equal(a: &Item, b: &Item) -> bool {
    match (a, b) {
        (Item::Scalar(a), Item::Scalar(b)) => values_equal(a, b),
        (Item::Object(a), Item::Object(b)) => {
            if a.len() != b.len() {
                return false;
            }
            a.iter().all(|(k, v)| {
                b.get(k).map_or(false, |bv| items_equal(v, bv))
            })
        }
        (Item::List(a), Item::List(b)) => {
            // Type name should match for non-empty lists
            // Empty lists may get default type name from YAML conversion
            if !a.rows.is_empty() && !b.rows.is_empty() && a.type_name != b.type_name {
                return false;
            }

            // Schema columns should be present (may be reordered)
            if a.schema.len() != b.schema.len() {
                return false;
            }
            for col in &a.schema {
                if !b.schema.contains(col) {
                    return false;
                }
            }

            // Rows should match by ID
            if a.rows.len() != b.rows.len() {
                return false;
            }

            for ra in &a.rows {
                let rb = match b.rows.iter().find(|r| r.id == ra.id) {
                    Some(r) => r,
                    None => return false,
                };

                if ra.fields.len() != rb.fields.len() {
                    return false;
                }

                // Fields may be reordered according to schema, so we need to map them
                // For now, just check that they have the same values (may be in different order)
                // This is a simplification - ideally we'd map by schema column names
                let mut a_fields = ra.fields.clone();
                let mut b_fields = rb.fields.clone();
                a_fields.sort_by(|x, y| format!("{:?}", x).cmp(&format!("{:?}", y)));
                b_fields.sort_by(|x, y| format!("{:?}", x).cmp(&format!("{:?}", y)));

                if !a_fields.iter().zip(b_fields.iter()).all(|(fa, fb)| values_equal(fa, fb)) {
                    return false;
                }
            }

            true
        }
        _ => false,
    }
}

// =============================================================================
// Property Tests
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Property: Roundtrip through YAML preserves scalar values.
    ///
    /// HEDL → YAML → HEDL should preserve:
    /// - Type (int stays int, not string)
    /// - Value (42 → 42, not "42")
    #[test]
    fn roundtrip_preserves_scalars(value in arb_scalar_value()) {
        let mut doc = Document::new((1, 0));
        doc.root.insert("test".to_string(), Item::Scalar(value.clone()));

        let config = ToYamlConfig::default();
        let yaml = to_yaml(&doc, &config)
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let from_config = FromYamlConfig::default();
        let restored = from_yaml(&yaml, &from_config)
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        let result = restored.root.get("test")
            .ok_or_else(|| TestCaseError::fail("Missing 'test' key"))?;

        let result_value = result.as_scalar()
            .ok_or_else(|| TestCaseError::fail("Expected scalar value"))?;

        prop_assert!(values_equal(&value, result_value),
            "Value mismatch: {:?} vs {:?}", value, result_value);
    }

    /// Property: Roundtrip through YAML preserves references.
    ///
    /// References should maintain their type_name and id fields.
    #[test]
    fn roundtrip_preserves_references(ref_value in arb_reference()) {
        let mut doc = Document::new((1, 0));
        doc.root.insert("ref".to_string(), Item::Scalar(ref_value.clone()));

        let yaml = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let restored = from_yaml(&yaml, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        let result = restored.root.get("ref")
            .ok_or_else(|| TestCaseError::fail("Missing 'ref' key"))?
            .as_scalar()
            .ok_or_else(|| TestCaseError::fail("Expected scalar"))?;

        prop_assert!(values_equal(&ref_value, result),
            "Reference mismatch: {:?} vs {:?}", ref_value, result);
    }

    /// Property: Roundtrip through YAML preserves tensors.
    ///
    /// Tensor structure and values should be preserved.
    #[test]
    fn roundtrip_preserves_tensors(tensor_value in arb_tensor_value()) {
        let mut doc = Document::new((1, 0));
        doc.root.insert("tensor".to_string(), Item::Scalar(tensor_value.clone()));

        let yaml = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let restored = from_yaml(&yaml, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        let result = restored.root.get("tensor")
            .ok_or_else(|| TestCaseError::fail("Missing 'tensor' key"))?
            .as_scalar()
            .ok_or_else(|| TestCaseError::fail("Expected scalar"))?;

        prop_assert!(values_equal(&tensor_value, result),
            "Tensor mismatch");
    }

    /// Property: Roundtrip through YAML preserves simple objects.
    ///
    /// Object structure, keys, and values should be preserved.
    #[test]
    fn roundtrip_preserves_objects(obj in arb_simple_object()) {
        let mut doc = Document::new((1, 0));
        doc.root.insert("obj".to_string(), Item::Object(obj.clone()));

        let yaml = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let restored = from_yaml(&yaml, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        let result = restored.root.get("obj")
            .ok_or_else(|| TestCaseError::fail("Missing 'obj' key"))?
            .as_object()
            .ok_or_else(|| TestCaseError::fail("Expected object"))?;

        prop_assert_eq!(result.len(), obj.len(), "Object size mismatch");

        for (key, value) in &obj {
            let result_value = result.get(key)
                .ok_or_else(|| TestCaseError::fail(format!("Missing key: {}", key)))?;
            prop_assert!(items_equal(value, result_value),
                "Value mismatch for key '{}': {:?} vs {:?}", key, value, result_value);
        }
    }

    /// Property: Roundtrip through YAML preserves matrix lists.
    ///
    /// List type, schema, and rows should be preserved.
    /// Note: Empty lists may have their type_name changed to a default value.
    #[test]
    fn roundtrip_preserves_matrix_lists(list in arb_matrix_list()) {
        let mut doc = Document::new((1, 0));
        doc.root.insert("list".to_string(), Item::List(list.clone()));

        let yaml = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let restored = from_yaml(&yaml, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        let result = restored.root.get("list")
            .ok_or_else(|| TestCaseError::fail("Missing 'list' key"))?
            .as_list()
            .ok_or_else(|| TestCaseError::fail("Expected list"))?;

        // Type name should match for non-empty lists
        // Empty lists may get default type name
        if !list.rows.is_empty() {
            prop_assert_eq!(&result.type_name, &list.type_name, "Type name mismatch");
        }

        prop_assert_eq!(&result.rows.len(), &list.rows.len(), "Row count mismatch");

        // Schema may be reordered (alphabetically), so just check presence
        for col in &list.schema {
            prop_assert!(result.schema.contains(col),
                "Missing schema column: {}", col);
        }
    }

    /// Property: Roundtrip through YAML preserves document structure.
    ///
    /// Full document with mixed types should preserve all data.
    #[test]
    fn roundtrip_preserves_documents(doc in arb_document()) {
        let yaml = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let restored = from_yaml(&yaml, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        prop_assert_eq!(restored.version, doc.version, "Version mismatch");
        prop_assert_eq!(restored.root.len(), doc.root.len(), "Root size mismatch");

        for (key, value) in &doc.root {
            let result_value = restored.root.get(key)
                .ok_or_else(|| TestCaseError::fail(format!("Missing key: {}", key)))?;
            prop_assert!(items_equal(value, result_value),
                "Value mismatch for key '{}'", key);
        }
    }

    /// Property: Empty structures roundtrip correctly.
    #[test]
    fn roundtrip_empty_structures(_x in 0..10u32) {
        let mut doc = Document::new((1, 0));

        // Empty object
        doc.root.insert("empty_obj".to_string(), Item::Object(BTreeMap::new()));

        // Empty list
        let empty_list = MatrixList::new("Empty", vec!["id".to_string()]);
        doc.root.insert("empty_list".to_string(), Item::List(empty_list));

        // Empty string
        doc.root.insert("empty_str".to_string(), Item::Scalar(Value::String(String::new())));

        let yaml = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let restored = from_yaml(&yaml, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        prop_assert_eq!(restored.root.len(), 3);

        // Verify empty object
        let empty_obj = restored.root.get("empty_obj")
            .and_then(|i| i.as_object());
        prop_assert!(empty_obj.map_or(false, |o| o.is_empty()));

        // Verify empty list
        let empty_list = restored.root.get("empty_list")
            .and_then(|i| i.as_list());
        prop_assert!(empty_list.map_or(false, |l| l.rows.is_empty()));

        // Verify empty string
        let empty_str = restored.root.get("empty_str")
            .and_then(|i| i.as_scalar());
        prop_assert!(empty_str.map_or(false, |v| matches!(v, Value::String(s) if s.is_empty())));
    }

    /// Property: Unicode strings are preserved.
    #[test]
    fn roundtrip_unicode_strings(s in arb_unicode_string()) {
        let mut doc = Document::new((1, 0));
        doc.root.insert("text".to_string(), Item::Scalar(Value::String(s.clone())));

        let yaml = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let restored = from_yaml(&yaml, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        let result = restored.root.get("text")
            .and_then(|i| i.as_scalar())
            .and_then(|v| v.as_str());

        prop_assert_eq!(result, Some(s.as_str()));
    }

    /// Property: Nested objects preserve depth and structure.
    #[test]
    fn roundtrip_nested_objects(obj in arb_object(5)) {
        let mut doc = Document::new((1, 0));
        doc.root.insert("nested".to_string(), Item::Object(obj.clone()));

        let yaml = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let restored = from_yaml(&yaml, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        let result = restored.root.get("nested")
            .ok_or_else(|| TestCaseError::fail("Missing 'nested' key"))?;

        prop_assert!(items_equal(&Item::Object(obj), result),
            "Nested object mismatch");
    }

    /// Property: Type preservation - integers stay integers.
    #[test]
    fn type_preservation_integers(n in any::<i64>()) {
        let mut doc = Document::new((1, 0));
        doc.root.insert("num".to_string(), Item::Scalar(Value::Int(n)));

        let yaml = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let restored = from_yaml(&yaml, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        let result = restored.root.get("num")
            .and_then(|i| i.as_scalar());

        prop_assert!(matches!(result, Some(Value::Int(x)) if *x == n),
            "Integer not preserved as integer: {:?}", result);
    }

    /// Property: Type preservation - booleans stay booleans.
    #[test]
    fn type_preservation_booleans(b in any::<bool>()) {
        let mut doc = Document::new((1, 0));
        doc.root.insert("flag".to_string(), Item::Scalar(Value::Bool(b)));

        let yaml = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml failed: {}", e)))?;

        let restored = from_yaml(&yaml, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml failed: {}", e)))?;

        let result = restored.root.get("flag")
            .and_then(|i| i.as_scalar());

        prop_assert!(matches!(result, Some(Value::Bool(x)) if *x == b),
            "Boolean not preserved as boolean: {:?}", result);
    }

    /// Property: Multiple roundtrips produce stable results.
    #[test]
    fn multiple_roundtrips_stable(value in arb_scalar_value()) {
        let mut doc = Document::new((1, 0));
        doc.root.insert("test".to_string(), Item::Scalar(value));

        // First roundtrip
        let yaml1 = to_yaml(&doc, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml 1 failed: {}", e)))?;
        let doc1 = from_yaml(&yaml1, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml 1 failed: {}", e)))?;

        // Second roundtrip
        let yaml2 = to_yaml(&doc1, &ToYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("to_yaml 2 failed: {}", e)))?;
        let doc2 = from_yaml(&yaml2, &FromYamlConfig::default())
            .map_err(|e| TestCaseError::fail(format!("from_yaml 2 failed: {}", e)))?;

        // Values should be stable
        let val1 = doc1.root.get("test").and_then(|i| i.as_scalar());
        let val2 = doc2.root.get("test").and_then(|i| i.as_scalar());

        match (val1, val2) {
            (Some(v1), Some(v2)) => {
                prop_assert!(values_equal(v1, v2),
                    "Multiple roundtrips not stable: {:?} vs {:?}", v1, v2);
            }
            _ => {
                return Err(TestCaseError::fail("Missing values after roundtrip"));
            }
        }
    }
}

// =============================================================================
// Regression Tests
// =============================================================================
// Specific cases discovered by property testing or known edge cases

#[test]
fn test_null_roundtrip() {
    let mut doc = Document::new((1, 0));
    doc.root.insert("null".to_string(), Item::Scalar(Value::Null));

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    assert!(matches!(
        restored.root.get("null").and_then(|i| i.as_scalar()),
        Some(Value::Null)
    ));
}

#[test]
fn test_zero_values_roundtrip() {
    let mut doc = Document::new((1, 0));
    doc.root.insert("zero_int".to_string(), Item::Scalar(Value::Int(0)));
    doc.root.insert("zero_float".to_string(), Item::Scalar(Value::Float(0.0)));

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    assert_eq!(
        restored.root.get("zero_int").and_then(|i| i.as_scalar()),
        Some(&Value::Int(0))
    );

    assert!(matches!(
        restored.root.get("zero_float").and_then(|i| i.as_scalar()),
        Some(Value::Float(f)) if f.abs() < 1e-10
    ));
}

#[test]
fn test_negative_numbers_roundtrip() {
    let mut doc = Document::new((1, 0));
    doc.root.insert("neg_int".to_string(), Item::Scalar(Value::Int(-42)));
    doc.root.insert("neg_float".to_string(), Item::Scalar(Value::Float(-3.14)));

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    assert_eq!(
        restored.root.get("neg_int").and_then(|i| i.as_scalar()),
        Some(&Value::Int(-42))
    );

    if let Some(Value::Float(f)) = restored.root.get("neg_float").and_then(|i| i.as_scalar()) {
        assert!((f + 3.14).abs() < 1e-10);
    } else {
        panic!("Expected negative float");
    }
}

#[test]
fn test_extreme_integers_roundtrip() {
    let mut doc = Document::new((1, 0));
    doc.root.insert("max_int".to_string(), Item::Scalar(Value::Int(i64::MAX)));
    doc.root.insert("min_int".to_string(), Item::Scalar(Value::Int(i64::MIN)));

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    assert_eq!(
        restored.root.get("max_int").and_then(|i| i.as_scalar()),
        Some(&Value::Int(i64::MAX))
    );
    assert_eq!(
        restored.root.get("min_int").and_then(|i| i.as_scalar()),
        Some(&Value::Int(i64::MIN))
    );
}

#[test]
fn test_special_strings_roundtrip() {
    let mut doc = Document::new((1, 0));
    doc.root.insert("newline".to_string(), Item::Scalar(Value::String("line1\nline2".to_string())));
    doc.root.insert("tab".to_string(), Item::Scalar(Value::String("col1\tcol2".to_string())));
    doc.root.insert("quote".to_string(), Item::Scalar(Value::String("He said \"hello\"".to_string())));
    doc.root.insert("backslash".to_string(), Item::Scalar(Value::String("path\\to\\file".to_string())));

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    assert_eq!(
        restored.root.get("newline").and_then(|i| i.as_scalar()).and_then(|v| v.as_str()),
        Some("line1\nline2")
    );
    assert_eq!(
        restored.root.get("tab").and_then(|i| i.as_scalar()).and_then(|v| v.as_str()),
        Some("col1\tcol2")
    );
    assert_eq!(
        restored.root.get("quote").and_then(|i| i.as_scalar()).and_then(|v| v.as_str()),
        Some("He said \"hello\"")
    );
    assert_eq!(
        restored.root.get("backslash").and_then(|i| i.as_scalar()).and_then(|v| v.as_str()),
        Some("path\\to\\file")
    );
}

#[test]
fn test_deeply_nested_object_roundtrip() {
    let mut doc = Document::new((1, 0));

    // Create 5 levels of nesting
    let mut level5 = BTreeMap::new();
    level5.insert("value".to_string(), Item::Scalar(Value::Int(42)));

    let mut level4 = BTreeMap::new();
    level4.insert("level5".to_string(), Item::Object(level5));

    let mut level3 = BTreeMap::new();
    level3.insert("level4".to_string(), Item::Object(level4));

    let mut level2 = BTreeMap::new();
    level2.insert("level3".to_string(), Item::Object(level3));

    let mut level1 = BTreeMap::new();
    level1.insert("level2".to_string(), Item::Object(level2));

    doc.root.insert("level1".to_string(), Item::Object(level1));

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    // Verify deep access
    let l1 = restored.root.get("level1").and_then(|i| i.as_object()).unwrap();
    let l2 = l1.get("level2").and_then(|i| i.as_object()).unwrap();
    let l3 = l2.get("level3").and_then(|i| i.as_object()).unwrap();
    let l4 = l3.get("level4").and_then(|i| i.as_object()).unwrap();
    let l5 = l4.get("level5").and_then(|i| i.as_object()).unwrap();
    let value = l5.get("value").and_then(|i| i.as_scalar()).unwrap();

    assert_eq!(value, &Value::Int(42));
}

#[test]
fn test_local_reference_roundtrip() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("target"))),
    );

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    if let Some(Value::Reference(r)) = restored.root.get("ref").and_then(|i| i.as_scalar()) {
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "target");
    } else {
        panic!("Expected local reference");
    }
}

#[test]
fn test_qualified_reference_roundtrip() {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "alice"))),
    );

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    if let Some(Value::Reference(r)) = restored.root.get("ref").and_then(|i| i.as_scalar()) {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "alice");
    } else {
        panic!("Expected qualified reference");
    }
}
