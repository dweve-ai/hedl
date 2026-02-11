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

//! Property-based tests for error handling consistency.
//!
//! # Invariants Tested
//!
//! 1. **Error Determinism**: Same malformed input always produces same error
//! 2. **No Panics**: Parser never panics, only returns Err
//! 3. **Error Message Quality**: All errors are informative and non-empty
//! 4. **Validation Consistency**: Same validation rules applied consistently

use hedl_core::parse;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Parsing same input twice produces identical success/failure.
    #[test]
    fn prop_error_determinism(content in "[a-zA-Z0-9 \n:,|@~\\.]{0,200}") {
        let result1 = parse(content.as_bytes());
        let result2 = parse(content.as_bytes());

        prop_assert_eq!(result1.is_ok(), result2.is_ok(),
            "Error determinism violated");

        if let (Err(e1), Err(e2)) = (result1, result2) {
            let msg1 = format!("{e1}");
            let msg2 = format!("{e2}");
            prop_assert_eq!(msg1, msg2, "Error messages differ");
        }
    }

    /// Property: Parser never panics on any input.
    #[test]
    fn prop_no_panic_on_any_input(content in "\\PC{0,500}") {
        let _result = parse(content.as_bytes());
        // If we get here without panic, test passes
    }

    /// Property: Missing VERSION header produces specific error.
    #[test]
    fn prop_missing_version_error(content in "[a-zA-Z0-9 \n]{0,100}") {
        let doc = format!("---\n{content}");

        let result = parse(doc.as_bytes());
        if result.is_err() {
            let err_msg = format!("{}", result.unwrap_err());
            prop_assert!(
                err_msg.contains("VERSION") ||err_msg.contains("version") ||err_msg.contains("expected"),
                "Error should mention VERSION: {}", err_msg
            );
        }
    }

    /// Property: Malformed VERSION produces error.
    #[test]
    fn prop_malformed_version_error(
        major in "\\PC{0,10}",
        minor in "\\PC{0,10}"
    ) {
        prop_assume!(!major.chars().all(|c|c.is_ascii_digit()));
        prop_assume!(!minor.chars().all(|c|c.is_ascii_digit()));

        let doc = format!("%VERSION: {major}.{minor}\n---\nvalue: 1\n");

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_err(), "Should reject malformed VERSION");
    }

    /// Property: Duplicate keys in same object produce error.
    #[test]
    fn prop_duplicate_key_error(
        key in "[a-z][a-z0-9_]{0,30}",
        val1 in -100_i64..100,
        val2 in -100_i64..100
    ) {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nobj:\n {key}: {val1}\n {key}: {val2}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_err(), "Should reject duplicate key '{}'", key);

        if let Err(e) = result {
            let err_msg = format!("{e}");
            prop_assert!(
                err_msg.contains("duplicate") ||err_msg.contains(&key) ||err_msg.contains("Duplicate"),
                "Error should mention duplicate or key name: {}", err_msg
            );
        }
    }

    /// Property: Valid 1-space indentation (any number of spaces = that level).
    #[test]
    fn prop_valid_one_space_indent(spaces in 1_usize..10) {
        let indent = " ".repeat(spaces);
        let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nobj:\n{indent}value: 1\n");

        let result = parse(doc.as_bytes());
        // With 1-space indentation, all positive indent levels are valid
        // Should not panic regardless
        if result.is_err() {
            let err_msg = format!("{}", result.unwrap_err());
            prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
        }
    }

    /// Property: Unresolved reference produces error in strict mode.
    #[test]
    fn prop_unresolved_reference_error(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id in "[a-z][a-z0-9_-]{1,30}",
        other_id in "[a-z][a-z0-9_-]{1,30}"
    ) {
        prop_assume!(id != other_id);

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id, ref]\n---\nitems:@{type_name}\n |{id}, @{other_id}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_err(),
            "Expected error for unresolved reference '{}' (only '{}' exists)", other_id, id);

        if let Err(e) = result {
            let err_msg = format!("{e}");
            prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
        }
    }

    /// Property: Duplicate IDs in same type produce error.
    #[test]
    fn prop_duplicate_id_error(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id in "[a-z][a-z0-9_-]{1,30}"
    ) {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id]\n---\nitems:@{type_name}\n |{id}\n |{id}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_err(), "Should reject duplicate ID '{}'", id);

        if let Err(e) = result {
            let err_msg = format!("{e}");
            prop_assert!(
                err_msg.contains("duplicate") ||err_msg.contains(&id) ||err_msg.contains("collision"),
                "Error should mention duplicate/collision: {}", err_msg
            );
        }
    }

    /// Property: All errors have non-empty messages.
    #[test]
    fn prop_errors_have_messages(malformed in "\\PC{0,200}") {
        let result = parse(malformed.as_bytes());

        if let Err(e) = result {
            let err_msg = format!("{e}");
            prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
            prop_assert!(err_msg.len() > 5, "Error message too short: '{}'", err_msg);
        }
    }

    /// Property: Errors don't contain internal panic messages.
    #[test]
    fn prop_no_internal_panic_messages(malformed in "\\PC{0,200}") {
        let result = parse(malformed.as_bytes());

        if let Err(e) = result {
            let err_msg = format!("{e}");
            prop_assert!(!err_msg.contains("unwrap"),
                "Error message contains 'unwrap': {}", err_msg);
            prop_assert!(!err_msg.contains("panic"),
                "Error message contains 'panic': {}", err_msg);
            prop_assert!(!err_msg.contains("thread"),
                "Error message contains 'thread': {}", err_msg);
        }
    }

    /// Property: Ditto in first row produces error.
    #[test]
    fn prop_ditto_first_row_error(type_name in "[A-Z][a-zA-Z0-9]{0,15}") {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id, value]\n---\nitems:@{type_name}\n |id1, ^\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_err(), "Ditto in first row should fail");

        if let Err(e) = result {
            let err_msg = format!("{e}");
            prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
        }
    }

    /// Property: Undefined struct type produces error.
    #[test]
    fn prop_undefined_struct_error(
        defined_type in "[A-Z][a-zA-Z0-9]{0,10}",
        undefined_type in "[A-Z][a-zA-Z0-9]{0,10}"
    ) {
        prop_assume!(defined_type != undefined_type);

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{defined_type}:[id]\n---\nitems:@{undefined_type}\n |id1\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_err(),
            "Should reject undefined struct type '{}'", undefined_type);
    }

    /// Property: Mismatched field count produces error or handles gracefully.
    #[test]
    fn prop_field_count_mismatch(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        field_count in 2_usize..10,
        value_count in 2_usize..10
    ) {
        prop_assume!(field_count != value_count);

        let fields = (0..field_count).map(|i|format!("f{i}")).collect::<Vec<_>>().join(", ");
        let values = (0..value_count).map(|i|i.to_string()).collect::<Vec<_>>().join(", ");

        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[{fields}]\n---\nitems:@{type_name}\n |{values}\n"
        );

        let result = parse(doc.as_bytes());
        // May succeed with padding or fail with error, but should not panic
        if result.is_err() {
            let err_msg = format!("{}", result.unwrap_err());
            prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
        }
    }
}

#[cfg(test)]
mod limit_violations {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property: Excessive nesting depth is handled.
        #[test]
        fn prop_deep_nesting_handled(depth in 100_usize..200) {
            let mut doc = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n");
            for d in 0..depth {
                let indent = " ".repeat(d);
                doc.push_str(&format!("{indent}level{d}:\n"));
            }
            let indent = " ".repeat(depth);
            doc.push_str(&format!("{indent}value: 1\n"));

            let result = parse(doc.as_bytes());
            // Should either succeed or produce a clear error
            if result.is_err() {
                let err_msg = format!("{}", result.unwrap_err());
                prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
            }
        }

        /// Property: Very long lines are handled.
        #[test]
        fn prop_long_line_handled(length in 1000_usize..10000) {
            let long_value = "x".repeat(length);
            let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: {long_value}\n");

            let result = parse(doc.as_bytes());
            // Should either succeed or produce a clear error
            if result.is_err() {
                let err_msg = format!("{}", result.unwrap_err());
                prop_assert!(!err_msg.is_empty(), "Error message should not be empty");
            }
        }
    }
}
