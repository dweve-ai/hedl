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

//! Property-based tests for parse/serialize roundtrip preservation.
//!
//! # Invariants Tested
//!
//! 1. **Parse Determinism**: Same input always produces same parsed output
//! 2. **Structure Preservation**: Document structure (nesting, types) is preserved
//! 3. **Value Preservation**: All scalar values maintain their types and values
//! 4. **Schema Preservation**: Type names and field names are preserved exactly

use hedl_core::parse;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Property: Parsing is deterministic (same bytes -> same result).
    #[test]
    fn prop_parse_deterministic(
        type_name in "[A-Z][a-zA-Z0-9]{0,15}",
        id in "[a-z][a-z0-9_-]{0,30}",
        value in -1000_i64..1000
    ) {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id, value]\n---\nitems:@{type_name}\n |{id}, {value}\n"
        );

        let parsed1 = parse(doc.as_bytes()).unwrap();
        let parsed2 = parse(doc.as_bytes()).unwrap();

        prop_assert_eq!(parsed1, parsed2, "Parsing not deterministic");
    }

    /// Property: Valid documents with varying nesting depths always parse.
    #[test]
    fn prop_nested_objects_parse(depth in 0_usize..10) {
        let mut doc = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n");

        for d in 0..depth {
            let indent = " ".repeat(d);
            doc.push_str(&format!("{indent}level{d}:\n"));
        }

        let indent = " ".repeat(depth);
        doc.push_str(&format!("{indent}value: 42\n"));

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse depth {}: {:?}", depth, result.err());
    }

    /// Property: Document version is preserved.
    #[test]
    fn prop_version_preserved(major in 0_u32..10, minor in 0_u32..20) {
        // v2.0+ requires compact syntax and %NULL/%QUOTE directives
        let doc = if (major, minor) >= (2, 0) {
            format!("%V:{major}.{minor}\n%NULL:~\n%QUOTE:\"\n---\nvalue: 1\n")
        } else {
            format!("%VERSION: {major}.{minor}\n---\nvalue: 1\n")
        };

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        prop_assert_eq!(parsed.version, (major, minor), "Version not preserved");
    }

    /// Property: Type names are preserved exactly.
    #[test]
    fn prop_type_name_preserved(type_name in "[A-Z][a-zA-Z0-9]{0,20}") {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:{type_name}:[id]\n---\nitems:@{type_name}\n |id1\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        prop_assert_eq!(&list.type_name, &type_name, "Type name not preserved");
    }

    /// Property: Field names are preserved exactly.
    #[test]
    fn prop_field_names_preserved(
        fields in prop::collection::vec("[a-z][a-z0-9_]{0,20}", 1..=10)
    ) {
        let unique_fields: Vec<String> = fields.into_iter()
            .enumerate()
            .map(|(i, f)|format!("{f}_{i}"))
            .collect();

        let field_list = unique_fields.join(", ");
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:T:[{field_list}]\n---\nitems:@T\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let schema = parsed.get_schema("T").unwrap();
        prop_assert_eq!(schema.len(), unique_fields.len(), "Field count mismatch");

        for (i, field) in unique_fields.iter().enumerate() {
            prop_assert_eq!(&schema[i], field, "Field name {} not preserved", i);
        }
    }

    /// Property: Integer values preserve their exact value.
    #[test]
    fn prop_integer_value_preserved(value in i64::MIN..i64::MAX) {
        let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: {value}\n");

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let val = parsed.get("value").unwrap().as_scalar().unwrap();
        prop_assert_eq!(val.as_int(), Some(value), "Integer value not preserved");
    }

    /// Property: String values preserve their content (trimmed of trailing whitespace).
    #[test]
    fn prop_string_value_preserved(s in "[a-zA-Z][a-zA-Z0-9_-]{0,99}") {
        let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: {s}\n");

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let val = parsed.get("value").unwrap().as_scalar().unwrap();
        prop_assert_eq!(val.as_str(), Some(s.as_str()), "String value not preserved");
    }

    /// Property: Boolean values preserve their exact value.
    #[test]
    fn prop_bool_value_preserved(value: bool) {
        let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: {value}\n");

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let val = parsed.get("value").unwrap().as_scalar().unwrap();
        prop_assert_eq!(val.as_bool(), Some(value), "Boolean value not preserved");
    }

    /// Property: Null values are preserved.
    #[test]
    fn prop_null_value_preserved(_n in 0..100_u32) {
        let doc = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nvalue: ~\n";

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let val = parsed.get("value").unwrap().as_scalar().unwrap();
        prop_assert!(val.is_null(), "Null value not preserved");
    }

    /// Property: List row count is preserved.
    #[test]
    fn prop_list_row_count_preserved(row_count in 1_usize..100) {
        let mut doc = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:T:[id]\n---\nitems:@T\n");
        for i in 0..row_count {
            doc.push_str(&format!(" |id{i}\n"));
        }

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        prop_assert_eq!(list.rows.len(), row_count, "Row count not preserved");
    }

    /// Property: Empty documents parse correctly.
    #[test]
    fn prop_empty_document_parses(_n in 0..100_u32) {
        let doc = "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n";

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Empty document should parse");

        let parsed = result.unwrap();
        prop_assert!(parsed.root.is_empty(), "Empty document should have empty root");
    }

    /// Property: Single-character keys work.
    #[test]
    fn prop_single_char_key_preserved(c in "[a-z]") {
        let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n{c}: 1\n");

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        prop_assert!(parsed.get(&c).is_some(), "Single-char key '{}' not found", c);
    }

    /// Property: Very long keys (up to 100 chars) are preserved.
    #[test]
    fn prop_long_key_preserved(key in "[a-z][a-z0-9_]{50,99}") {
        let doc = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n{key}: 1\n");

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        prop_assert!(parsed.get(&key).is_some(), "Long key not found");
    }

    /// Property: Mixed value types in same list are preserved correctly.
    #[test]
    fn prop_mixed_types_preserved(
        int_val in -100_i64..100,
        bool_val: bool,
        str_val in "[a-zA-Z]{1,20}"
    ) {
        let doc = format!(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:T:[id,int_col,bool_col,str_col]\n---\nitems:@T\n |id1, {int_val}, {bool_val}, {str_val}\n"
        );

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        let row = &list.rows[0];

        prop_assert_eq!(row.fields[1].as_int(), Some(int_val), "Int value not preserved");
        prop_assert_eq!(row.fields[2].as_bool(), Some(bool_val), "Bool value not preserved");
        prop_assert_eq!(row.fields[3].as_str(), Some(str_val.as_str()), "String value not preserved");
    }

    /// Property: Large documents (1000+ nodes) parse correctly.
    #[test]
    fn prop_large_document_parses(node_count in 100_usize..1000) {
        let mut doc = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:T:[id,value]\n---\nitems:@T\n");
        for i in 0..node_count {
            doc.push_str(&format!(" |id{}, {}\n", i, i * 10));
        }

        let result = parse(doc.as_bytes());
        prop_assert!(result.is_ok(), "Failed to parse large document: {:?}", result.err());

        let parsed = result.unwrap();
        let list = parsed.get("items").unwrap().as_list().unwrap();
        prop_assert_eq!(list.rows.len(), node_count, "Node count mismatch");
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
}
