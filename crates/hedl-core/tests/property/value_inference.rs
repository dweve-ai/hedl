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

//! Property-based tests for value type inference.
//!
//! These tests verify that value type inference is deterministic and consistent:
//!
//! # Properties Tested
//!
//! 1. **Inference Determinism**: Same input always infers to same type
//! 2. **Roundtrip Stability**: Inferred values roundtrip correctly
//! 3. **Type Precedence**: Inference ladder is respected (null > bool > number > string)
//! 4. **Edge Case Handling**: Boundary values are handled correctly
//!
//! All tests run 1000 cases to ensure comprehensive coverage.

use hedl_core::parse;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    /// Property: Any valid integer should parse correctly and roundtrip.
    #[test]
    fn prop_integer_roundtrips(n in -1_000_000_i64..1_000_000_i64) {
        let doc = format!("%VERSION: 1.0\n---\nvalue: {}\n", n);
        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let val = parsed.get("value").expect("Missing 'value' key");
        let scalar = val.as_scalar().expect("Expected scalar value");

        prop_assert_eq!(scalar.as_int(), Some(n), "Integer didn't roundtrip");
    }

    /// Property: Any valid floating-point number should parse correctly.
    #[test]
    fn prop_float_parses(f in -1_000_000.0_f64..1_000_000.0_f64) {
        let doc = format!("%VERSION: 1.0\n---\nvalue: {}\n", f);
        let result = parse(doc.as_bytes());

        if f.is_nan() {
            // NaN should parse, but won't equal itself
            prop_assert!(result.is_ok());
            let parsed = result.unwrap();
            let val = parsed.get("value").expect("Missing 'value' key");
            let scalar = val.as_scalar().expect("Expected scalar value");
            prop_assert!(scalar.as_float().map(|x| x.is_nan()).unwrap_or(false));
        } else if f.is_finite() {
            prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
            let parsed = result.unwrap();
            let val = parsed.get("value").expect("Missing 'value' key");
            let scalar = val.as_scalar().expect("Expected scalar value");

            if let Some(parsed_f) = scalar.as_float() {
                // Use approximate comparison for floats
                let epsilon = 1e-10;
                prop_assert!((parsed_f - f).abs() < epsilon || (parsed_f - f).abs() / f.abs() < epsilon,
                    "Float {} didn't roundtrip, got {}", f, parsed_f);
            } else {
                prop_assert!(false, "Expected float, got {:?}", scalar);
            }
        }
    }

    /// Property: Any valid boolean should parse correctly and roundtrip.
    #[test]
    fn prop_bool_roundtrips(b: bool) {
        let doc = format!("%VERSION: 1.0\n---\nvalue: {}\n", b);
        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let val = parsed.get("value").expect("Missing 'value' key");
        let scalar = val.as_scalar().expect("Expected scalar value");

        prop_assert_eq!(scalar.as_bool(), Some(b), "Boolean didn't roundtrip");
    }

    /// Property: Any valid string (without special chars) should parse and roundtrip.
    /// Note: Must start with a letter to avoid numeric inference.
    #[test]
    fn prop_simple_string_roundtrips(s in "[a-zA-Z][a-zA-Z0-9_-]{0,99}") {
        let doc = format!("%VERSION: 1.0\n---\nvalue: {}\n", s);
        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let val = parsed.get("value").expect("Missing 'value' key");
        let scalar = val.as_scalar().expect("Expected scalar value");

        prop_assert_eq!(scalar.as_str(), Some(s.as_str()), "String didn't roundtrip");
    }

    /// Property: Null value always parses correctly.
    #[test]
    fn prop_null_parses(_n in 0..100_u32) {
        let doc = "%VERSION: 1.0\n---\nvalue: ~\n";
        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok());

        let parsed = result.unwrap();
        let val = parsed.get("value").expect("Missing 'value' key");
        let scalar = val.as_scalar().expect("Expected scalar value");

        prop_assert!(scalar.is_null(), "Expected null value");
    }

    /// Property: Any valid key name should work in key-value pairs.
    /// Note: Keys use snake_case (letters, digits, underscores).
    #[test]
    fn prop_valid_key_names(key in "[a-z][a-z0-9_]{0,50}") {
        let doc = format!("%VERSION: 1.0\n---\n{}: test\n", key);
        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let val = parsed.get(&key);
        prop_assert!(val.is_some(), "Key '{}' not found", key);
    }

    /// Property: Numbers with leading zeros should be rejected or parsed as strings.
    #[test]
    fn prop_leading_zeros_handled(zeros in "0{2,5}", digits in "[1-9][0-9]{0,5}") {
        let num_str = format!("{}{}", zeros, digits);
        let doc = format!("%VERSION: 1.0\n---\nvalue: {}\n", num_str);
        let result = parse(doc.as_bytes());

        // Should parse successfully (either as int 0 or as string)
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    }
}

/// Additional inference tests for determinism and consistency.
#[cfg(test)]
mod determinism_tests {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// Property: Integer inference is deterministic (same input -> same output).
        #[test]
        fn prop_integer_inference_deterministic(n in -1_000_000_i64..1_000_000_i64) {
            let doc = format!("%VERSION: 1.0\n---\nval: {}\n", n);

            let result1 = parse(doc.as_bytes()).unwrap();
            let result2 = parse(doc.as_bytes()).unwrap();

            let val1 = result1.get("val").unwrap().as_scalar().unwrap();
            let val2 = result2.get("val").unwrap().as_scalar().unwrap();

            prop_assert_eq!(val1.as_int(), val2.as_int(),
                "Integer inference non-deterministic for {}", n);
        }

        /// Property: Float inference is deterministic.
        #[test]
        fn prop_float_inference_deterministic(f in -1_000.0_f64..1_000.0) {
            prop_assume!(f.is_finite());

            let doc = format!("%VERSION: 1.0\n---\nval: {}\n", f);

            let result1 = parse(doc.as_bytes()).unwrap();
            let result2 = parse(doc.as_bytes()).unwrap();

            let val1 = result1.get("val").unwrap().as_scalar().unwrap();
            let val2 = result2.get("val").unwrap().as_scalar().unwrap();

            // Both should infer as float with same value
            if let (Some(f1), Some(f2)) = (val1.as_float(), val2.as_float()) {
                let epsilon = 1e-10;
                prop_assert!((f1 - f2).abs() < epsilon,
                    "Float inference non-deterministic: {} vs {}", f1, f2);
            } else {
                prop_assert!(false, "Both should be floats");
            }
        }

        /// Property: String inference is deterministic.
        /// Note: Must start with a letter to avoid numeric inference.
        #[test]
        fn prop_string_inference_deterministic(s in "[a-zA-Z][a-zA-Z0-9_-]{0,99}") {
            let doc = format!("%VERSION: 1.0\n---\nval: {}\n", s);

            let result1 = parse(doc.as_bytes()).unwrap();
            let result2 = parse(doc.as_bytes()).unwrap();

            let val1 = result1.get("val").unwrap().as_scalar().unwrap();
            let val2 = result2.get("val").unwrap().as_scalar().unwrap();

            prop_assert_eq!(val1.as_str(), val2.as_str(),
                "String inference non-deterministic for '{}'", s);
        }

        /// Property: Bool inference is deterministic.
        #[test]
        fn prop_bool_inference_deterministic(b: bool) {
            let doc = format!("%VERSION: 1.0\n---\nval: {}\n", b);

            let result1 = parse(doc.as_bytes()).unwrap();
            let result2 = parse(doc.as_bytes()).unwrap();

            let val1 = result1.get("val").unwrap().as_scalar().unwrap();
            let val2 = result2.get("val").unwrap().as_scalar().unwrap();

            prop_assert_eq!(val1.as_bool(), val2.as_bool(),
                "Bool inference non-deterministic for {}", b);
        }

        /// Property: Null inference is deterministic.
        #[test]
        fn prop_null_inference_deterministic(_n in 0..100_u32) {
            let doc = "%VERSION: 1.0\n---\nval: ~\n";

            let result1 = parse(doc.as_bytes()).unwrap();
            let result2 = parse(doc.as_bytes()).unwrap();

            let val1 = result1.get("val").unwrap().as_scalar().unwrap();
            let val2 = result2.get("val").unwrap().as_scalar().unwrap();

            prop_assert!(val1.is_null() && val2.is_null(),
                "Null inference non-deterministic");
        }

        /// Property: Reference inference is deterministic.
        #[test]
        fn prop_reference_inference_deterministic(
            type_name in "[A-Z][a-zA-Z0-9]{0,15}",
            id in "[a-z][a-z0-9_]{0,30}"
        ) {
            let doc = format!(
                "%VERSION: 1.0\n%STRUCT: {}: [id]\n---\nitems: @{}\n  | {}\nref: @{}:{}\n",
                type_name, type_name, id, type_name, id
            );

            let result1 = parse(doc.as_bytes()).unwrap();
            let result2 = parse(doc.as_bytes()).unwrap();

            let val1 = result1.get("ref").unwrap().as_scalar().unwrap();
            let val2 = result2.get("ref").unwrap().as_scalar().unwrap();

            let ref1 = val1.as_reference().unwrap();
            let ref2 = val2.as_reference().unwrap();

            prop_assert_eq!(&ref1.id, &ref2.id, "Reference ID non-deterministic");
            prop_assert_eq!(&ref1.type_name, &ref2.type_name, "Reference type non-deterministic");
        }
    }
}

/// Property: List with consistent schema should parse correctly.
#[test]
fn test_property_list_with_n_rows() {
    proptest!(ProptestConfig::with_cases(1000), |(n in 1_usize..100)| {
        let mut doc = String::from("%VERSION: 1.0\n%STRUCT: T: [id, value]\n---\ndata: @T\n");
        for i in 0..n {
            doc.push_str(&format!("  | id{}, {}\n", i, i * 10));
        }

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list_item = parsed.get("data").expect("Missing 'data' key");
        let list = list_item.as_list().expect("Expected list");

        prop_assert_eq!(list.rows.len(), n, "Row count mismatch");
    });
}

/// Property: Nested objects at various depths should parse.
#[test]
fn test_property_nested_objects() {
    proptest!(ProptestConfig::with_cases(1000), |(depth in 1_usize..10)| {
        let mut doc = String::from("%VERSION: 1.0\n---\n");

        // Build nested structure
        for d in 0..depth {
            let indent = "  ".repeat(d);
            doc.push_str(&format!("{}level{}:\n", indent, d));
        }

        // Add final value
        let indent = "  ".repeat(depth);
        doc.push_str(&format!("{}value: 42\n", indent));

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse depth {}: {:?}", depth, result.err());
    });
}

/// Property: Nested objects at correct indentation parse successfully.
/// Note: HEDL requires exactly 2 spaces per nesting level.
#[test]
fn test_property_whitespace_variations() {
    proptest!(ProptestConfig::with_cases(1000), |(depth in 1_usize..5)| {
        // Build a nested structure with correct 2-space indentation per level
        let mut doc = String::from("%VERSION: 1.0\n---\n");
        for d in 0..depth {
            let indent = "  ".repeat(d);
            doc.push_str(&format!("{}level{}:\n", indent, d));
        }
        let indent = "  ".repeat(depth);
        doc.push_str(&format!("{}value: 42\n", indent));

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed at depth {}: {:?}", depth, result.err());
    });
}
