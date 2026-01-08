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

//! Property-based tests for parse → canonicalize → parse roundtrip.
//!
//! These tests verify that:
//! - Parsing is deterministic (same input always produces same output)
//! - Canonicalization is idempotent (canonical(canonical(x)) == canonical(x))
//! - Roundtrip preserves semantics (parse → canonicalize → parse produces equivalent document)
//!
//! # Properties Tested
//!
//! 1. **Parse Determinism**: Parsing the same input twice produces identical ASTs
//! 2. **Canonicalization Idempotency**: Canonicalizing canonical output is a no-op
//! 3. **Semantic Preservation**: Roundtrip preserves all semantic information
//! 4. **Value Stability**: Values maintain their types and data through roundtrips
//! 5. **Reference Preservation**: References remain valid after canonicalization
//! 6. **Schema Stability**: Matrix schemas are preserved through roundtrips

use hedl_core::parse;
use hedl_c14n::canonicalize;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Parsing the same document twice produces identical results.
    ///
    /// This tests that the parser is deterministic and doesn't rely on
    /// non-deterministic factors like hash map iteration order.
    #[test]
    fn prop_parse_determinism(
        key in "[a-z][a-z0-9_]{0,20}",
        value in -10000_i64..10000
    ) {
        let doc = format!("%VERSION: 1.0\n---\n{}: {}\n", key, value);

        let result1 = parse(doc.as_bytes());
        let result2 = parse(doc.as_bytes());

        prop_assert!(result1.is_ok(), "First parse failed");
        prop_assert!(result2.is_ok(), "Second parse failed");

        let parsed1 = result1.unwrap();
        let parsed2 = result2.unwrap();

        // Documents should be identical
        prop_assert_eq!(parsed1, parsed2, "Parse non-deterministic");
    }

    /// Property: Canonicalizing a canonical document is idempotent.
    ///
    /// This verifies that canonical(canonical(x)) == canonical(x).
    #[test]
    fn prop_canonicalization_idempotent(
        key in "[a-z][a-z0-9_]{0,20}",
        value in -1000_i64..1000
    ) {
        let doc = format!("%VERSION: 1.0\n---\n{}: {}\n", key, value);

        let parsed = parse(doc.as_bytes()).unwrap();
        let canon1 = canonicalize(&parsed).unwrap();

        let parsed2 = parse(canon1.as_bytes()).unwrap();
        let canon2 = canonicalize(&parsed2).unwrap();

        // Second canonicalization should produce identical output
        prop_assert_eq!(canon1, canon2, "Canonicalization not idempotent");
    }

    /// Property: Integer values roundtrip through parse → canonicalize → parse.
    #[test]
    fn prop_integer_roundtrip(
        key in "[a-z][a-z0-9_]{0,20}",
        value in i64::MIN/1000..i64::MAX/1000
    ) {
        let doc = format!("%VERSION: 1.0\n---\n{}: {}\n", key, value);

        // Parse original
        let parsed1 = parse(doc.as_bytes()).unwrap();

        // Canonicalize
        let canon = canonicalize(&parsed1).unwrap();

        // Parse canonical
        let parsed2 = parse(canon.as_bytes()).unwrap();

        // Values should be identical
        let val1 = parsed1.get(&key).unwrap().as_scalar().unwrap();
        let val2 = parsed2.get(&key).unwrap().as_scalar().unwrap();

        prop_assert_eq!(val1.as_int(), val2.as_int(), "Integer value changed in roundtrip");
    }

    /// Property: Float values roundtrip (within floating point precision).
    #[test]
    fn prop_float_roundtrip(
        key in "[a-z][a-z0-9_]{0,20}",
        value in -1000.0_f64..1000.0
    ) {
        // Skip NaN and infinity for this test
        prop_assume!(value.is_finite());

        let doc = format!("%VERSION: 1.0\n---\n{}: {}\n", key, value);

        let parsed1 = parse(doc.as_bytes()).unwrap();
        let canon = canonicalize(&parsed1).unwrap();
        let parsed2 = parse(canon.as_bytes()).unwrap();

        let val1 = parsed1.get(&key).unwrap().as_scalar().unwrap();
        let val2 = parsed2.get(&key).unwrap().as_scalar().unwrap();

        if let (Some(f1), Some(f2)) = (val1.as_float(), val2.as_float()) {
            let epsilon = 1e-10_f64.max(f1.abs() * 1e-10);
            prop_assert!((f1 - f2).abs() < epsilon,
                "Float value changed: {} -> {}", f1, f2);
        } else {
            prop_assert!(false, "Expected float values");
        }
    }

    /// Property: Boolean values roundtrip exactly.
    #[test]
    fn prop_bool_roundtrip(
        key in "[a-z][a-z0-9_]{0,20}",
        value: bool
    ) {
        let doc = format!("%VERSION: 1.0\n---\n{}: {}\n", key, value);

        let parsed1 = parse(doc.as_bytes()).unwrap();
        let canon = canonicalize(&parsed1).unwrap();
        let parsed2 = parse(canon.as_bytes()).unwrap();

        let val1 = parsed1.get(&key).unwrap().as_scalar().unwrap();
        let val2 = parsed2.get(&key).unwrap().as_scalar().unwrap();

        prop_assert_eq!(val1.as_bool(), val2.as_bool(),
            "Boolean value changed in roundtrip");
    }

    /// Property: String values roundtrip exactly.
    /// Note: Value must start with a letter to avoid numeric inference.
    #[test]
    fn prop_string_roundtrip(
        key in "[a-z][a-z0-9_]{0,20}",
        value in "[a-zA-Z][a-zA-Z0-9 _-]{0,99}"
    ) {
        let doc = format!("%VERSION: 1.0\n---\n{}: {}\n", key, value);

        let parsed1 = parse(doc.as_bytes()).unwrap();
        let canon = canonicalize(&parsed1).unwrap();
        let parsed2 = parse(canon.as_bytes()).unwrap();

        let val1 = parsed1.get(&key).unwrap().as_scalar().unwrap();
        let val2 = parsed2.get(&key).unwrap().as_scalar().unwrap();

        prop_assert_eq!(val1.as_str(), val2.as_str(),
            "String value changed in roundtrip");
    }

    /// Property: Null values roundtrip exactly.
    #[test]
    fn prop_null_roundtrip(key in "[a-z][a-z0-9_]{0,20}") {
        let doc = format!("%VERSION: 1.0\n---\n{}: ~\n", key);

        let parsed1 = parse(doc.as_bytes()).unwrap();
        let canon = canonicalize(&parsed1).unwrap();
        let parsed2 = parse(canon.as_bytes()).unwrap();

        let val1 = parsed1.get(&key).unwrap().as_scalar().unwrap();
        let val2 = parsed2.get(&key).unwrap().as_scalar().unwrap();

        prop_assert!(val1.is_null(), "Original value not null");
        prop_assert!(val2.is_null(), "Roundtrip value not null");
    }

    /// Property: References roundtrip with correct ID and type.
    #[test]
    fn prop_reference_roundtrip(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id in "[a-z][a-z0-9_]{0,20}"
    ) {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id]\n---\nitems: @{}\n  | {}\nref: @{}:{}\n",
            type_name, type_name, id, type_name, id
        );

        let parsed1 = parse(doc.as_bytes()).unwrap();
        let canon = canonicalize(&parsed1).unwrap();
        let parsed2 = parse(canon.as_bytes()).unwrap();

        let val1 = parsed1.get("ref").unwrap().as_scalar().unwrap();
        let val2 = parsed2.get("ref").unwrap().as_scalar().unwrap();

        let ref1 = val1.as_reference().unwrap();
        let ref2 = val2.as_reference().unwrap();

        prop_assert_eq!(&ref1.id, &ref2.id, "Reference ID changed");
        prop_assert_eq!(&ref1.type_name, &ref2.type_name, "Reference type changed");
    }

    /// Property: Matrix lists roundtrip with same structure.
    #[test]
    fn prop_matrix_list_roundtrip(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        count in 1_usize..50
    ) {
        let mut doc = format!(
            "%VERSION: 1.0\n\
             %STRUCT: {}: [id, value]\n\
             ---\n\
             items: @{}\n",
            type_name, type_name
        );

        for i in 0..count {
            doc.push_str(&format!("  | id{}, {}\n", i, i * 10));
        }

        let parsed1 = parse(doc.as_bytes()).unwrap();
        let canon = canonicalize(&parsed1).unwrap();
        let parsed2 = parse(canon.as_bytes()).unwrap();

        let list1 = parsed1.get("items").unwrap().as_list().unwrap();
        let list2 = parsed2.get("items").unwrap().as_list().unwrap();

        prop_assert_eq!(&list1.type_name, &list2.type_name, "Type name changed");
        prop_assert_eq!(list1.rows.len(), list2.rows.len(), "Row count changed");
        prop_assert_eq!(&list1.schema, &list2.schema, "Schema changed");

        // Check each row
        for i in 0..count {
            prop_assert_eq!(&list1.rows[i].id, &list2.rows[i].id,
                "Row {} ID changed", i);
            prop_assert_eq!(list1.rows[i].fields.len(), list2.rows[i].fields.len(),
                "Row {} field count changed", i);
        }
    }

    /// Property: Nested objects roundtrip with same structure.
    #[test]
    fn prop_nested_object_roundtrip(depth in 1_usize..10) {
        let mut doc = String::from("%VERSION: 1.0\n---\n");

        for d in 0..depth {
            let indent = "  ".repeat(d);
            doc.push_str(&format!("{}level{}:\n", indent, d));
        }

        let indent = "  ".repeat(depth);
        doc.push_str(&format!("{}value: 42\n", indent));

        let parsed1 = parse(doc.as_bytes()).unwrap();
        let canon = canonicalize(&parsed1).unwrap();
        let parsed2 = parse(canon.as_bytes()).unwrap();

        // Navigate to deepest level in both
        let mut obj1 = &parsed1.root;
        let mut obj2 = &parsed2.root;

        for d in 0..depth {
            let key = format!("level{}", d);
            obj1 = obj1.get(&key).unwrap().as_object().unwrap();
            obj2 = obj2.get(&key).unwrap().as_object().unwrap();
        }

        // Check final value
        let val1 = obj1.get("value").unwrap().as_scalar().unwrap();
        let val2 = obj2.get("value").unwrap().as_scalar().unwrap();

        prop_assert_eq!(val1.as_int(), Some(42), "Original value wrong");
        prop_assert_eq!(val2.as_int(), Some(42), "Roundtrip value wrong");
    }

    /// Property: Multiple key-value pairs roundtrip in canonical order.
    #[test]
    fn prop_multiple_keys_roundtrip(count in 1_usize..20) {
        let mut doc = String::from("%VERSION: 1.0\n---\n");
        let mut keys = Vec::new();

        for i in 0..count {
            let key = format!("key{:02}", i);
            keys.push(key.clone());
            doc.push_str(&format!("{}: {}\n", key, i));
        }

        let parsed1 = parse(doc.as_bytes()).unwrap();
        let canon = canonicalize(&parsed1).unwrap();
        let parsed2 = parse(canon.as_bytes()).unwrap();

        // All keys should be present in both
        for key in &keys {
            let val1 = parsed1.get(key).unwrap().as_scalar().unwrap();
            let val2 = parsed2.get(key).unwrap().as_scalar().unwrap();

            prop_assert_eq!(val1, val2, "Value changed for key {}", key);
        }
    }

    /// Property: Nested structures roundtrip with correct indentation.
    /// Note: HEDL requires exactly 2 spaces per nesting level.
    #[test]
    fn prop_whitespace_normalization(depth in 1_usize..5) {
        // Build a nested structure with correct 2-space indentation
        let mut doc = String::from("%VERSION: 1.0\n---\n");
        for d in 0..depth {
            let indent = "  ".repeat(d);
            doc.push_str(&format!("{}level{}:\n", indent, d));
        }
        let indent = "  ".repeat(depth);
        doc.push_str(&format!("{}value: 42\n", indent));

        let parsed1 = parse(doc.as_bytes()).unwrap();
        let canon = canonicalize(&parsed1).unwrap();
        let parsed2 = parse(canon.as_bytes()).unwrap();

        // Navigate to deepest level in both
        let mut obj1 = &parsed1.root;
        let mut obj2 = &parsed2.root;

        for d in 0..depth {
            let key = format!("level{}", d);
            obj1 = obj1.get(&key).unwrap().as_object().unwrap();
            obj2 = obj2.get(&key).unwrap().as_object().unwrap();
        }

        let val1 = obj1.get("value").unwrap().as_scalar().unwrap();
        let val2 = obj2.get("value").unwrap().as_scalar().unwrap();

        prop_assert_eq!(val1.as_int(), val2.as_int(), "Value changed in roundtrip");
    }
}

/// Additional roundtrip tests for edge cases.
#[cfg(test)]
mod edge_cases {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// Property: Simple objects with one child roundtrip correctly.
        /// Note: HEDL requires objects to have at least one child.
        #[test]
        fn prop_simple_object_roundtrip(key in "[a-z][a-z0-9_]{0,20}") {
            let doc = format!("%VERSION: 1.0\n---\n{}:\n  child: value\n", key);

            let parsed1 = parse(doc.as_bytes()).unwrap();
            let canon = canonicalize(&parsed1).unwrap();
            let parsed2 = parse(canon.as_bytes()).unwrap();

            let obj1 = parsed1.get(&key).unwrap().as_object().unwrap();
            let obj2 = parsed2.get(&key).unwrap().as_object().unwrap();

            prop_assert!(obj1.contains_key("child"), "Original missing child");
            prop_assert!(obj2.contains_key("child"), "Roundtrip missing child");
        }

        /// Property: Unicode strings roundtrip correctly.
        #[test]
        fn prop_unicode_roundtrip(
            key in "[a-z][a-z0-9_]{0,20}",
            value in "[\u{4E00}-\u{9FFF}]{1,20}"  // Chinese characters
        ) {
            let doc = format!("%VERSION: 1.0\n---\n{}: \"{}\"\n", key, value);

            let parsed1 = parse(doc.as_bytes()).unwrap();
            let canon = canonicalize(&parsed1).unwrap();
            let parsed2 = parse(canon.as_bytes()).unwrap();

            let val1 = parsed1.get(&key).unwrap().as_scalar().unwrap();
            let val2 = parsed2.get(&key).unwrap().as_scalar().unwrap();

            prop_assert_eq!(val1.as_str(), val2.as_str(), "Unicode string changed");
        }

        /// Property: Mixed value types in matrix lists roundtrip.
        #[test]
        fn prop_mixed_types_roundtrip(
            type_name in "[A-Z][a-zA-Z0-9]{0,15}",
            count in 1_usize..20
        ) {
            let mut doc = format!(
                "%VERSION: 1.0\n\
                 %STRUCT: {}: [id, int_val, str_val, bool_val]\n\
                 ---\n\
                 items: @{}\n",
                type_name, type_name
            );

            for i in 0..count {
                let bool_val = if i % 2 == 0 { "true" } else { "false" };
                doc.push_str(&format!("  | id{}, {}, val{}, {}\n", i, i * 10, i, bool_val));
            }

            let parsed1 = parse(doc.as_bytes()).unwrap();
            let canon = canonicalize(&parsed1).unwrap();
            let parsed2 = parse(canon.as_bytes()).unwrap();

            let list1 = parsed1.get("items").unwrap().as_list().unwrap();
            let list2 = parsed2.get("items").unwrap().as_list().unwrap();

            prop_assert_eq!(list1.rows.len(), list2.rows.len(), "Row count changed");

            for i in 0..count {
                // Check each field type is preserved
                let fields1 = &list1.rows[i].fields;
                let fields2 = &list2.rows[i].fields;

                prop_assert_eq!(fields1.len(), fields2.len(), "Field count changed at row {}", i);

                // Int field
                prop_assert_eq!(fields1[0].as_int(), fields2[0].as_int(),
                    "Int field changed at row {}", i);

                // String field
                prop_assert_eq!(fields1[1].as_str(), fields2[1].as_str(),
                    "String field changed at row {}", i);

                // Bool field
                prop_assert_eq!(fields1[2].as_bool(), fields2[2].as_bool(),
                    "Bool field changed at row {}", i);
            }
        }
    }
}
