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

//! Property-based tests for ditto marker expansion.

use hedl_core::parse;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Ditto copies integer values correctly.
    #[test]
    fn prop_ditto_copies_integer(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        value in -1000_i64..1000
    ) {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id, value]\n---\nitems: @{}\n  | id1, {}\n  | id2, ^\n",
            type_name, type_name, value
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();

        let row1_val = &list.rows[0].fields[1];
        let row2_val = &list.rows[1].fields[1];

        prop_assert_eq!(row1_val, row2_val, "Ditto didn't copy integer");
    }

    /// Property: Ditto copies string values correctly.
    #[test]
    fn prop_ditto_copies_string(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        value in "[a-zA-Z0-9]{1,50}"
    ) {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id, value]\n---\nitems: @{}\n  | id1, {}\n  | id2, ^\n",
            type_name, type_name, value
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();

        let row1_val = &list.rows[0].fields[1];
        let row2_val = &list.rows[1].fields[1];

        prop_assert_eq!(row1_val, row2_val, "Ditto didn't copy string");
    }

    /// Property: Ditto copies boolean values correctly.
    #[test]
    fn prop_ditto_copies_boolean(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        value in proptest::bool::ANY
    ) {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id, value]\n---\nitems: @{}\n  | id1, {}\n  | id2, ^\n",
            type_name, type_name, value
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();

        let row1_val = &list.rows[0].fields[1];
        let row2_val = &list.rows[1].fields[1];

        prop_assert_eq!(row1_val, row2_val, "Ditto didn't copy boolean");
    }

    /// Property: Ditto copies null values correctly.
    #[test]
    fn prop_ditto_copies_null(type_name in "[A-Z][a-zA-Z0-9]{0,15}") {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id, value]\n---\nitems: @{}\n  | id1, ~\n  | id2, ^\n",
            type_name, type_name
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();

        let row1_val = &list.rows[0].fields[1];
        let row2_val = &list.rows[1].fields[1];

        prop_assert!(row1_val.is_null(), "First row should be null");
        prop_assert!(row2_val.is_null(), "Ditto didn't copy null");
    }

    /// Property: Ditto copies reference values correctly.
    #[test]
    fn prop_ditto_copies_reference(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        ref_id in "[a-z][a-z0-9_-]{0,20}"
    ) {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id, ref]\n---\nitems: @{}\n  | {}, @{}\n  | id2, ^\n",
            type_name, type_name, ref_id, ref_id
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();

        let row1_val = &list.rows[0].fields[1];
        let row2_val = &list.rows[1].fields[1];

        prop_assert_eq!(row1_val, row2_val, "Ditto didn't copy reference");
    }

    /// Property: Ditto in first row produces an error.
    #[test]
    fn prop_ditto_first_row_error(type_name in "[A-Z][a-zA-Z0-9]{0,15}") {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id, value]\n---\nitems: @{}\n  | id1, ^\n",
            type_name, type_name
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_err(), "Ditto in first row should fail");
    }

    /// Property: Ditto chain works (multiple dittos in sequence).
    #[test]
    fn prop_ditto_chain(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        value in -100_i64..100
    ) {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id, value]\n---\nitems: @{}\n  | id1, {}\n  | id2, ^\n  | id3, ^\n",
            type_name, type_name, value
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();

        let val1 = &list.rows[0].fields[1];
        let val2 = &list.rows[1].fields[1];
        let val3 = &list.rows[2].fields[1];

        prop_assert_eq!(val1, val2, "Second row ditto failed");
        prop_assert_eq!(val2, val3, "Third row ditto failed");
    }

    /// Property: Partial ditto copies only specified columns.
    #[test]
    fn prop_partial_ditto(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        val1 in -100_i64..100,
        val2 in -100_i64..100
    ) {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id, col1, col2]\n---\nitems: @{}\n  | id1, {}, {}\n  | id2, ^, 999\n",
            type_name, type_name, val1, val2
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();

        let row1_col1 = &list.rows[0].fields[1];
        let row2_col1 = &list.rows[1].fields[1];

        prop_assert_eq!(row1_col1, row2_col1, "Ditto didn't copy col1");
    }

    /// Property: Multiple dittos in same row work independently.
    #[test]
    fn prop_multiple_dittos(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        val1 in -100_i64..100,
        val2 in -100_i64..100
    ) {
        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id, col1, col2]\n---\nitems: @{}\n  | id1, {}, {}\n  | id2, ^, ^\n",
            type_name, type_name, val1, val2
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();

        let row1_col1 = &list.rows[0].fields[1];
        let row1_col2 = &list.rows[0].fields[2];
        let row2_col1 = &list.rows[1].fields[1];
        let row2_col2 = &list.rows[1].fields[2];

        prop_assert_eq!(row1_col1, row2_col1, "Ditto didn't copy col1");
        prop_assert_eq!(row1_col2, row2_col2, "Ditto didn't copy col2");
    }

    /// Property: Ditto copies previous row only, not earlier rows.
    #[test]
    fn prop_ditto_copies_previous_only(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        val1 in -100_i64..100,
        val2 in -100_i64..100
    ) {
        prop_assume!(val1 != val2);

        let doc = format!(
            "%VERSION: 1.0\n%STRUCT: {}: [id, value]\n---\nitems: @{}\n  | id1, {}\n  | id2, {}\n  | id3, ^\n",
            type_name, type_name, val1, val2
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();

        let row2_val = &list.rows[1].fields[1];
        let row3_val = &list.rows[2].fields[1];

        prop_assert_eq!(row2_val, row3_val, "Ditto should copy row 2, not row 1");
    }
}

/// Edge case tests for ditto markers.
#[cfg(test)]
mod edge_cases {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// Property: Ditto with float values.
        #[test]
        fn prop_ditto_copies_float(
            type_name in "[A-Z][a-zA-Z0-9]{0,15}",
            value in -1000.0_f64..1000.0
        ) {
            let doc = format!(
                "%VERSION: 1.0\n%STRUCT: {}: [id, value]\n---\nitems: @{}\n  | id1, {}\n  | id2, ^\n",
                type_name, type_name, value
            );

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

            let parsed = result.unwrap();
            let list = parsed.get("items").unwrap().as_list().unwrap();

            let row1_val = &list.rows[0].fields[1];
            let row2_val = &list.rows[1].fields[1];

            prop_assert_eq!(row1_val, row2_val, "Ditto didn't copy float");
        }

        /// Property: Long ditto chain (10+ rows).
        #[test]
        fn prop_long_ditto_chain(
            type_name in "[A-Z][a-zA-Z0-9]{0,15}",
            value in -100_i64..100,
            count in 2_usize..20
        ) {
            let mut doc = format!(
                "%VERSION: 1.0\n%STRUCT: {}: [id, value]\n---\nitems: @{}\n  | id0, {}\n",
                type_name, type_name, value
            );

            for i in 1..=count {
                doc.push_str(&format!("  | id{}, ^\n", i));
            }

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

            let parsed = result.unwrap();
            let list = parsed.get("items").unwrap().as_list().unwrap();

            // All rows should have the same value
            for i in 0..=count {
                let val = &list.rows[i].fields[1];
                prop_assert_eq!(val, &list.rows[0].fields[1],
                    "Row {} value mismatch", i);
            }
        }

        /// Property: Ditto with mixed types in row.
        #[test]
        fn prop_ditto_mixed_types(
            type_name in "[A-Z][a-zA-Z0-9]{0,15}",
            int_val in -100_i64..100,
            bool_val in proptest::bool::ANY
        ) {
            let doc = format!(
                "%VERSION: 1.0\n%STRUCT: {}: [id, int_col, bool_col, null_col]\n---\nitems: @{}\n  | id1, {}, {}, ~\n  | id2, ^, ^, ^\n",
                type_name, type_name, int_val, bool_val
            );

            let result = parse(doc.as_bytes());
            prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

            let parsed = result.unwrap();
            let list = parsed.get("items").unwrap().as_list().unwrap();

            for col_idx in 1..4 {
                let row1_val = &list.rows[0].fields[col_idx];
                let row2_val = &list.rows[1].fields[col_idx];
                prop_assert_eq!(row1_val, row2_val,
                    "Ditto didn't copy column {}", col_idx);
            }
        }
    }
}
