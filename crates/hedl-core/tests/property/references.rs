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

//! Property-based tests for reference resolution.

use hedl_core::parse;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Any valid ID should work in references.
    #[test]
    fn prop_valid_reference_ids(id in "[a-z][a-z0-9_-]{0,50}") {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: T: [id, ref]\n---\ndata: @T\n  | {}, @{}\n",
            id, id
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse with ID '{}': {:?}", id, result.err());

        let parsed = result.unwrap();
        let list_item = parsed.get("data").expect("Missing 'data' key");
        let list = list_item.as_list().expect("Expected list");

        prop_assert_eq!(list.rows.len(), 1, "Expected one row");
        prop_assert_eq!(&list.rows[0].id, &id, "ID mismatch");
    }

    /// Property: Qualified references with valid type names should parse.
    #[test]
    fn prop_qualified_references(type_name in "[A-Z][a-zA-Z0-9]{0,20}", id in "[a-z][a-z0-9_-]{0,30}") {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id]\n%STRUCT: Other: [id, ref]\n---\nitems: @{}\n  | {}\nothers: @Other\n  | other1, @{}:{}\n",
            type_name, type_name, id, type_name, id
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let items = parsed.get("items").expect("Missing 'items' key");
        let items_list = items.as_list().expect("Expected list");

        prop_assert_eq!(&items_list.type_name, &type_name, "Type name mismatch");
    }

    /// Property: Multiple nodes with unique IDs should all be registered.
    #[test]
    fn prop_multiple_unique_ids(count in 1_usize..20) {
        let mut ids = Vec::new();
        for i in 0..count {
            ids.push(format!("id{}", i));
        }

        let mut doc = String::from("%VERSION: 1.0\n%STRUCT: T: [id]\n---\ndata: @T\n");
        for id in &ids {
            doc.push_str(&format!("  | {}\n", id));
        }

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list_item = parsed.get("data").expect("Missing 'data' key");
        let list = list_item.as_list().expect("Expected list");

        prop_assert_eq!(list.rows.len(), count, "Row count mismatch");

        for (i, row) in list.rows.iter().enumerate() {
            prop_assert_eq!(&row.id, &ids[i], "ID mismatch at index {}", i);
        }
    }

    /// Property: Self-references should work (node referring to itself).
    #[test]
    fn prop_self_reference(id in "[a-z][a-z0-9_-]{0,30}") {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: T: [id, self_ref]\n---\ndata: @T\n  | {}, @{}\n",
            id, id
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse self-reference: {:?}", result.err());

        let parsed = result.unwrap();
        let list_item = parsed.get("data").expect("Missing 'data' key");
        let list = list_item.as_list().expect("Expected list");

        prop_assert_eq!(list.rows.len(), 1, "Expected one row");

        let ref_field = &list.rows[0].fields[1];
        prop_assert!(ref_field.is_reference(), "Expected reference value");
    }
}

/// Additional reference resolution consistency tests.
#[cfg(test)]
mod consistency_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// Property: Reference resolution is deterministic.
        #[test]
        fn prop_reference_resolution_deterministic(
            type_name in "[A-Z][a-zA-Z0-9]{0,15}",
            id in "[a-z][a-z0-9_-]{0,30}"
        ) {
            let doc = format!(
                "%VERSION: 1.0\n%STRUCT: {}: [id, ref]\n---\nitems: @{}\n  | {}, @{}\n",
                type_name, type_name, id, id
            );

            let result1 = parse(doc.as_bytes());
            let result2 = parse(doc.as_bytes());

            prop_assert_eq!(result1.is_ok(), result2.is_ok(),
                "Reference resolution non-deterministic");

            if let (Ok(doc1), Ok(doc2)) = (result1, result2) {
                let list1 = doc1.get("items").unwrap().as_list().unwrap();
                let list2 = doc2.get("items").unwrap().as_list().unwrap();

                prop_assert_eq!(&list1.rows[0].id, &list2.rows[0].id,
                    "Resolved ID differs");
            }
        }

        /// Property: Cross-type references work correctly.
        #[test]
        fn prop_cross_type_references(
            type1 in "[A-Z][a-zA-Z0-9]{0,10}",
            type2 in "[A-Z][a-zA-Z0-9]{0,10}",
            id in "[a-z][a-z0-9_-]{0,20}"
        ) {
            prop_assume!(type1 != type2);

            let doc = format!(
                "%VERSION: 1.0\n%STRUCT: {}: [id]\n%STRUCT: {}: [id, ref]\n---\nitems1: @{}\n  | {}\nitems2: @{}\n  | other, @{}:{}\n",
                type1, type2, type1, id, type2, type1, id
            );

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(), "Cross-type reference failed: {:?}", result.err());

            let parsed = result.unwrap();
            let list2 = parsed.get("items2").unwrap().as_list().unwrap();
            let ref_field = &list2.rows[0].fields[1];

            prop_assert!(ref_field.is_reference(), "Expected reference");
            let ref_val = ref_field.as_reference().unwrap();
            prop_assert_eq!(&ref_val.id, &id, "Reference ID wrong");
            prop_assert_eq!(ref_val.type_name.as_deref(), Some(type1.as_str()),
                "Reference type wrong");
        }

        /// Property: Circular references are allowed (no cycle detection).
        #[test]
        fn prop_circular_references_allowed(
            type_name in "[A-Z][a-zA-Z0-9]{0,15}",
            id1 in "[a-z][a-z0-9_-]{1,15}",
            id2 in "[a-z][a-z0-9_-]{1,15}"
        ) {
            prop_assume!(id1 != id2);

            let doc = format!(
                "%VERSION: 1.0\n%STRUCT: {}: [id, ref]\n---\nitems: @{}\n  | {}, @{}\n  | {}, @{}\n",
                type_name, type_name, id1, id2, id2, id1
            );

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(), "Circular references failed: {:?}", result.err());
        }

        /// Property: Many-to-one references work (multiple refs to same target).
        #[test]
        fn prop_many_to_one_references(
            type_name in "[A-Z][a-zA-Z0-9]{0,15}",
            target_id in "[a-z][a-z0-9_-]{1,20}",
            count in 2_usize..20
        ) {
            let mut doc = format!(
                "%VERSION: 1.0\n%STRUCT: {}: [id, ref]\n---\nitems: @{}\n  | {}, ~\n",
                type_name, type_name, target_id
            );

            for i in 0..count {
                doc.push_str(&format!("  | ref{}, @{}\n", i, target_id));
            }

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(), "Many-to-one refs failed: {:?}", result.err());

            let parsed = result.unwrap();
            let list = parsed.get("items").unwrap().as_list().unwrap();

            for i in 1..=count {
                let ref_val = &list.rows[i].fields[1];
                prop_assert!(ref_val.is_reference(), "Row {} not a reference", i);
                let r = ref_val.as_reference().unwrap();
                prop_assert_eq!(&r.id, &target_id, "Row {} wrong target", i);
            }
        }

        /// Property: Reference preservation in nested structures.
        #[test]
        fn prop_nested_reference_resolution(
            type_name in "[A-Z][a-zA-Z0-9]{0,15}",
            id in "[a-z][a-z0-9_-]{0,20}"
        ) {
            let doc = format!(
                "%VERSION: 1.0\n%STRUCT: {}: [id]\n---\nitems: @{}\n  | {}\nnested:\n  ref: @{}:{}\n",
                type_name, type_name, id, type_name, id
            );

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(), "Nested reference failed: {:?}", result.err());

            let parsed = result.unwrap();
            let nested = parsed.get("nested").unwrap().as_object().unwrap();
            let ref_val = nested.get("ref").unwrap().as_scalar().unwrap();

            prop_assert!(ref_val.is_reference(), "Expected reference in nested");
        }
    }
}

/// Property: Duplicate IDs in same type should be detected.
#[test]
fn test_property_duplicate_id_detection() {
    proptest!(ProptestConfig::with_cases(1000), |(id in "[a-z][a-z0-9_-]{1,30}")| {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: T: [id, value]\n---\ndata: @T\n  | {}, val1\n  | {}, val2\n",
            id, id
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_err(), "Expected collision error for duplicate ID '{}'", id);
    });
}

/// Property: References to non-existent IDs should be detected in strict mode.
#[test]
fn test_property_unresolved_reference_detection() {
    proptest!(ProptestConfig::with_cases(1000), |(id in "[a-z][a-z0-9_-]{1,30}", other_id in "[a-z][a-z0-9_-]{1,30}")| {
        prop_assume!(id != other_id);

        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: T: [id, ref]\n---\ndata: @T\n  | {}, @{}\n",
            id, other_id
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_err(),
            "Expected reference error for unresolved reference '{}' (only '{}' exists)",
            other_id, id);
    });
}

/// Property: Forward references should work (reference before definition).
#[test]
fn test_property_forward_references() {
    proptest!(ProptestConfig::with_cases(1000), |(id1 in "[a-z][a-z0-9_-]{1,20}", id2 in "[a-z][a-z0-9_-]{1,20}")| {
        prop_assume!(id1 != id2);

        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: T: [id, ref]\n---\ndata: @T\n  | {}, @{}\n  | {}, ~\n",
            id1, id2, id2
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Forward reference should work: {:?}", result.err());
    });
}
