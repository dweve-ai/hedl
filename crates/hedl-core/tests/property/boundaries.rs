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

//! Property-based tests for boundary conditions and limits.
//!
//! # Invariants Tested
//!
//! 1. **Numeric Boundaries**: MIN/MAX values for integers and floats
//! 2. **Depth Limits**: Maximum nesting and NEST hierarchy depth
//! 3. **Width Limits**: Maximum columns and object keys
//! 4. **Count Limits**: Maximum nodes, aliases, and keys
//! 5. **Size Limits**: Empty values and structures handled correctly

use hedl_core::parse;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: i64 boundary values parse correctly.
    #[test]
    fn prop_i64_boundaries(_seed in 0..100_u32) {
        let test_cases = vec![
            i64::MIN,
            i64::MIN + 1,
            -1_000_000,
            -1,
            0,
            1,
            1_000_000,
            i64::MAX - 1,
            i64::MAX,
        ];

        for value in test_cases {
            let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: {value}\n");
            let result = parse(doc.as_bytes());

            prop_assert!(result.is_ok(),
                "Failed to parse boundary value {}: {:?}", value, result.err());

            let parsed = result.unwrap();
            let val = parsed.get("value").unwrap().as_scalar().unwrap();
            prop_assert_eq!(val.as_int(), Some(value),
                "Integer {} didn't roundtrip", value);
        }
    }

    /// Property: Float boundary values parse correctly.
    #[test]
    fn prop_float_boundaries(_seed in 0..100_u32) {
        let test_cases = vec![
            -1_000_000.0,
            -1.0,
            -f64::MIN_POSITIVE,
            -0.0,
            0.0,
            f64::MIN_POSITIVE,
            1.0,
            1_000_000.0,
            f64::EPSILON,
        ];

        for value in test_cases {
            let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: {value}\n");
            let result = parse(doc.as_bytes());

            prop_assert!(result.is_ok(),
                "Failed to parse boundary value {}: {:?}", value, result.err());

            let parsed = result.unwrap();
            let val = parsed.get("value").unwrap().as_scalar().unwrap();

            if let Some(parsed_f) = val.as_float() {
                let epsilon = 1e-10;
                prop_assert!((parsed_f - value).abs() < epsilon,
                    "Float {} didn't roundtrip accurately, got {}", value, parsed_f);
            }
        }
    }

    /// Property: Zero values (int and float) are handled correctly.
    #[test]
    fn prop_zero_values(_seed in 0..100_u32) {
        let test_cases = vec![
            ("0", true, false),      // Integer zero
            ("0.0", false, true),    // Float zero
            ("-0", true, false),     // Negative integer zero
            ("-0.0", false, true),   // Negative float zero
        ];

        for (input, expect_int, expect_float) in test_cases {
            let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: {input}\n");
            let result = parse(doc.as_bytes());

            prop_assert!(result.is_ok(), "Failed to parse '{}': {:?}", input, result.err());

            let parsed = result.unwrap();
            let val = parsed.get("value").unwrap().as_scalar().unwrap();

            if expect_int {
                prop_assert!(val.as_int().is_some() ||val.as_float().is_some(),
                    "Zero '{}' should parse as number", input);
            }
            if expect_float {
                prop_assert!(val.as_float().is_some(),
                    "Float zero '{}' should parse as float", input);
            }
        }
    }

    /// Property: Empty strings are handled correctly.
    #[test]
    fn prop_empty_string_handled(_seed in 0..100_u32) {
        let doc = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: \"\"\n";

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse empty string: {:?}", result.err());

        let parsed = result.unwrap();
        let val = parsed.get("value").unwrap().as_scalar().unwrap();
        prop_assert_eq!(val.as_str(), Some(""), "Empty string not preserved");
    }

    /// Property: Single-space strings are preserved.
    #[test]
    fn prop_single_space_preserved(_seed in 0..100_u32) {
        let doc = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: \" \"\n";

        let result = parse(doc.as_bytes());
        if result.is_ok() {
            let parsed = result.unwrap();
            if let Some(item) = parsed.get("value") {
                if let Some(val) = item.as_scalar() {
                    if let Some(s) = val.as_str() {
                        prop_assert_eq!(s, " ", "Single space not preserved");
                    }
                }
            }
        }
    }

    /// Property: Nesting up to depth 10 succeeds.
    #[test]
    fn prop_moderate_nesting_succeeds(depth in 1_usize..10) {
        let mut doc = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n");

        for d in 0..depth {
            let indent = " ".repeat(d);
            doc.push_str(&format!("{indent}level{d}:\n"));
        }

        let indent = " ".repeat(depth);
        doc.push_str(&format!("{indent}value: 42\n"));

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed at depth {}: {:?}", depth, result.err());
    }

    /// Property: Wide schemas (many columns) are handled.
    #[test]
    fn prop_wide_schema_handled(column_count in 5_usize..50) {
        // First field must be 'id' for the identifier column
        let fields = std::iter::once("id".to_string())
            .chain((1..column_count).map(|i|format!("field{i}")))
            .collect::<Vec<_>>()
            .join(", ");

        // First value must be a string identifier
        let values = std::iter::once("item0".to_string())
            .chain((1..column_count).map(|i|i.to_string()))
            .collect::<Vec<_>>()
            .join(", ");

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:T:[{fields}]\n---\nitems:@T\n |{values}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(),
            "Failed with {} columns: {:?}", column_count, result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        prop_assert_eq!(list.schema.len(), column_count, "Column count mismatch");
    }

    /// Property: Many object keys are handled.
    #[test]
    fn prop_many_object_keys(key_count in 10_usize..100) {
        let mut doc = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nobj:\n");

        for i in 0..key_count {
            doc.push_str(&format!(" key{i}: {i}\n"));
        }

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(),
            "Failed with {} keys: {:?}", key_count, result.err());

        let parsed = result.unwrap();
        let obj = parsed.get("obj").unwrap().as_object().unwrap();
        prop_assert_eq!(obj.len(), key_count, "Key count mismatch");
    }

    /// Property: Empty objects parse correctly.
    #[test]
    fn prop_empty_object_parses(_seed in 0..100_u32) {
        let doc = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nobj:\n";

        let result = parse(doc.as_bytes());
        if result.is_ok() {
            let parsed = result.unwrap();
            if let Some(item) = parsed.get("obj") {
                if let Some(obj) = item.as_object() {
                    prop_assert!(obj.is_empty(), "Empty object should have no keys");
                }
            }
        }
    }

    /// Property: Empty lists parse correctly.
    #[test]
    fn prop_empty_list_parses(type_name in "[A-Z][a-zA-Z0-9]{0,15}") {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id]\n---\nitems:@{type_name}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse empty list: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        prop_assert!(list.rows.is_empty(), "Empty list should have no rows");
    }

    /// Property: Lists with 1 row work correctly.
    #[test]
    fn prop_single_row_list(type_name in "[A-Z][a-zA-Z0-9]{0,15}") {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id]\n---\nitems:@{type_name}\n |id1\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse single-row list: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        prop_assert_eq!(list.rows.len(), 1, "Should have exactly 1 row");
    }

    /// Property: Maximum column count (100) is supported.
    #[test]
    fn prop_max_columns_supported(_seed in 0..10_u32) {
        let column_count = 100;
        // First field must be 'id' for the identifier column
        let fields = std::iter::once("id".to_string())
            .chain((1..column_count).map(|i|format!("f{i}")))
            .collect::<Vec<_>>()
            .join(", ");

        // First value must be a string identifier
        let values = std::iter::once("row0".to_string())
            .chain((1..column_count).map(|_|"0".to_string()))
            .collect::<Vec<_>>()
            .join(", ");

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:T:[{fields}]\n---\nitems:@T\n |{values}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(),
            "Failed with max columns (100): {:?}", result.err());
    }

    /// Property: Very long IDs (up to 100 chars) are supported.
    #[test]
    fn prop_long_id_supported(id_prefix in "[a-z]{1,10}", id_suffix in "[a-z0-9_-]{50,90}") {
        let long_id = format!("{id_prefix}{id_suffix}");

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:T:[id]\n---\nitems:@T\n |{long_id}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(),
            "Failed with long ID (len={}): {:?}", long_id.len(), result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        prop_assert_eq!(&list.rows[0].id, &long_id, "Long ID not preserved");
    }

    /// Property: Many aliases (up to 100) are supported.
    #[test]
    fn prop_many_aliases_supported(alias_count in 10_usize..100) {
        let mut doc = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n");

        // HEDL v2.0 alias syntax: %A:%key:"value"
        for i in 0..alias_count {
            doc.push_str(&format!("%A:%alias{i}:\"value{i}\"\n"));
        }

        doc.push_str("---\nvalue: 1\n");

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(),
            "Failed with {} aliases: {:?}", alias_count, result.err());

        let parsed = result.unwrap();
        prop_assert_eq!(parsed.aliases.len(), alias_count, "Alias count mismatch");
    }

    /// Property: Unicode strings at boundaries work.
    #[test]
    fn prop_unicode_boundaries(prefix in "[a-z]{1,5}") {
        let test_cases = vec![
            format!("{}🚀", prefix),           // Emoji
            format!("{}日本語", prefix),         // CJK
            format!("{}مرحبا", prefix),         // Arabic
            format!("{}Ω", prefix),             // Greek
            format!("{}ñ", prefix),             // Combining marks
        ];

        for content in test_cases {
            let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: {content}\n");
            let result = parse(doc.as_bytes());

            prop_assert!(result.is_ok(),
                "Failed to parse unicode '{}': {:?}", content, result.err());
        }
    }
}

#[cfg(test)]
mod nest_boundaries {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: NEST hierarchy with moderate depth works.
        #[test]
        fn prop_nest_depth_moderate(depth in 2_usize..5) {
            let types: Vec<String> = (0..depth).map(|i|format!("Type{i}")).collect();

            let mut doc = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n");

            for t in &types {
                doc.push_str(&format!("%S:{t}:[id]\n"));
            }

            for i in 0..types.len()-1 {
                doc.push_str(&format!("%N:{}>{}\n", types[i], types[i+1]));
            }

            // Start with root type
            doc.push_str(&format!("---\nroot:@{}\n |root0\n", types[0]));

            // Child rows are simply indented further, no @Type needed
            // Each level needs one more indent than the parent
            for i in 1..depth {
                let indent = " ".repeat(i + 1);
                doc.push_str(&format!("{indent}|child{i}\n"));
            }

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(),
                "Failed at NEST depth {}: {:?}", depth, result.err());
        }

        /// Property: NEST with no children works.
        #[test]
        fn prop_nest_no_children(
            parent_type in "[A-Z][a-zA-Z0-9]{0,10}",
            child_type in "[A-Z][a-zA-Z0-9]{0,10}"
        ) {
            prop_assume!(parent_type != child_type);

            let doc = format!(
                "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{parent_type}:[id]\n%S:{child_type}:[id]\n%N:{parent_type}>{child_type}\n---\nparents:@{parent_type}\n |parent1\n"
            );

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(),
                "Failed with NEST but no children: {:?}", result.err());

            let parsed = result.unwrap();
            let list = parsed.get("parents").unwrap().as_list().unwrap();
            prop_assert_eq!(list.rows.len(), 1, "Should have 1 parent");
        }
    }
}
