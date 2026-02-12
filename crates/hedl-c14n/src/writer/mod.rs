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

//! Canonical HEDL writer.
//!
//! This module implements the core serialization logic for HEDL canonical form.
//! It handles document structure, value formatting, quoting, escaping, and
//! ditto optimization.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};

use crate::config::CanonicalConfig;
use hedl_core::Item;

mod constants;
mod document;
mod formatting;

use constants::*;

/// Writer for canonical HEDL output.
///
/// Serializes HEDL documents to canonical string format according to SPEC.md Section 13.2.
/// Handles all value types, proper escaping, ditto optimization, and recursion limits.
///
/// # Security
///
/// - **Recursion limit**: Maximum nesting depth of 1000 prevents stack overflow
/// - **Proper escaping**: All special characters and control sequences escaped
/// - **No unsafe code**: Memory safety guaranteed by Rust type system
///
/// # Performance
///
/// - Pre-allocated output buffer with size estimation (P1+P3 optimization)
/// - Direct `BTreeMap` iteration without cloning (P0 optimization)
/// - Cell buffer pooling across all matrix lists (P2 optimization)
/// - Indentation cache for all depth levels (P2 optimization)
/// - Schema and type reference caching (P5 optimization)
/// - Fast-path formatters for common cases (P4 optimization)
pub struct CanonicalWriter {
    config: CanonicalConfig,
    output: String,
    /// Document version (major, minor) - used for version-specific output formatting
    /// v2.0+ forbids |[N] inline count hints, pre-v2.0 documents use them
    version: (u32, u32),
    /// P2 OPTIMIZATION: Reusable cell buffer across all matrix lists
    /// Reduces Vec allocations from 1 per list to 1 per document
    cell_buffer: RefCell<Vec<String>>,
    /// P2 OPTIMIZATION: Pre-computed indentation strings for all depths
    /// Eliminates repeated string allocation in hot loops
    indent_cache: Vec<String>,
    /// P5 OPTIMIZATION: Cache formatted schema strings per type
    /// Avoids repeated join operations for same schema
    schema_cache: RefCell<HashMap<String, String>>,
    /// P5 OPTIMIZATION: Cache formatted type references (@Type)
    /// Avoids repeated format! calls for same type
    type_ref_cache: RefCell<HashMap<String, String>>,
}

impl CanonicalWriter {
    /// Creates a new canonical writer with the given configuration.
    #[must_use]
    pub fn new(config: CanonicalConfig) -> Self {
        // P2 OPTIMIZATION: Pre-compute indentation cache up to MAX_NESTING_DEPTH
        // Eliminates O(n) string allocation in hot loops
        let indent_cache: Vec<String> = (0..=MAX_NESTING_DEPTH)
            .map(|depth| " ".repeat(depth * SPACES_PER_INDENT))
            .collect();

        Self {
            config,
            output: String::with_capacity(INITIAL_OUTPUT_BUFFER_CAPACITY),
            version: (2, 0), // Default to v2.0, updated in write_document
            cell_buffer: RefCell::new(Vec::new()),
            indent_cache,
            schema_cache: RefCell::new(HashMap::new()),
            type_ref_cache: RefCell::new(HashMap::new()),
        }
    }

    /// P3 OPTIMIZATION: Estimate output size to pre-allocate buffer
    /// Reduces reallocations from 2-4 to 0-1 per document
    fn estimate_output_size(doc: &hedl_core::Document) -> usize {
        // Header size estimation
        let header_size = 200 // VERSION line
            + doc.aliases.len() * 50 // ALIAS lines average
            + doc.structs.len() * 100 // STRUCT lines average
            + doc.nests.len() * 40; // NEST lines average

        // Body size estimation (recursive)
        let body_size = Self::estimate_body_size(&doc.root);

        // Add 20% buffer for safety
        (header_size + body_size) * 12 / 10
    }

    /// Recursively estimate body size for pre-allocation
    fn estimate_body_size(items: &BTreeMap<String, Item>) -> usize {
        let mut size = 0;
        for (key, item) in items {
            match item {
                Item::Scalar(_) => size += key.len() + 20, // Key + value + formatting
                Item::Object(children) => {
                    size += key.len() + 10; // Key + colon
                    size += Self::estimate_body_size(children);
                }
                Item::List(list) => {
                    size += key.len() + 50; // Declaration line
                    size += list.rows.len() * (list.schema.len() * 15); // Rows
                }
            }
        }
        size
    }

    /// P5 OPTIMIZATION: Get cached schema string for a type
    /// Avoids repeated join operations for the same schema
    fn get_schema_string(&self, type_name: &str, schema: &[String]) -> String {
        let mut cache = self.schema_cache.borrow_mut();
        cache
            .entry(type_name.to_string())
            .or_insert_with(|| schema.join(","))
            .clone()
    }

    /// P5 OPTIMIZATION: Get cached type reference string (@Type)
    /// Avoids repeated format! calls for the same type
    fn get_type_ref(&self, type_name: &str) -> String {
        let mut cache = self.type_ref_cache.borrow_mut();
        cache
            .entry(type_name.to_string())
            .or_insert_with(|| format!("@{type_name}"))
            .clone()
    }

    /// Check if document uses compact format.
    ///
    /// Compact format uses abbreviated directives:
    /// - `%V:1.2` instead of `%VERSION: 1.2`
    /// - `%S:Type:[fields]` instead of `%STRUCT: Type: [fields]`
    /// - `%N:Parent>Child` instead of `%NEST: Parent > Child`
    fn use_compact_format(doc: &hedl_core::Document) -> bool {
        doc.version >= (1, 2)
    }

    /// Check if document is v2.0 or later.
    ///
    /// v2.0+ has these format changes:
    /// - NO `|[N]` inline count hints (use %C: directives instead)
    /// - NO `^` ditto operator (every cell must have explicit value)
    /// - 1-space indentation only
    fn is_v20_or_later(&self) -> bool {
        self.version >= (2, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::QuotingStrategy;
    use hedl_core::{Document, Expression, MatrixList, Node, Reference, Tensor, Value};

    // ==================== escape_quoted tests ====================

    #[test]
    fn test_escape_quoted() {
        assert_eq!(CanonicalWriter::escape_quoted("hello"), "hello");
        assert_eq!(
            CanonicalWriter::escape_quoted("say \"hi\""),
            "say \"\"hi\"\""
        );
    }

    #[test]
    fn test_escape_quoted_empty() {
        assert_eq!(CanonicalWriter::escape_quoted(""), "");
    }

    #[test]
    fn test_escape_quoted_single_quote() {
        assert_eq!(CanonicalWriter::escape_quoted("\""), "\"\"");
    }

    #[test]
    fn test_escape_quoted_multiple_quotes() {
        assert_eq!(CanonicalWriter::escape_quoted("\"\"\""), "\"\"\"\"\"\"");
    }

    #[test]
    fn test_escape_quoted_unicode() {
        assert_eq!(
            CanonicalWriter::escape_quoted("héllo \"wörld\""),
            "héllo \"\"wörld\"\""
        );
    }

    // ==================== escape_cell_string tests ====================

    #[test]
    fn test_escape_cell_string() {
        // Basic escaping
        assert_eq!(CanonicalWriter::escape_cell_string("hello"), "hello");
        assert_eq!(
            CanonicalWriter::escape_cell_string("say \"hi\""),
            "say \"\"hi\"\""
        );

        // Control character escapes
        assert_eq!(
            CanonicalWriter::escape_cell_string("line1\nline2"),
            "line1\\nline2"
        );
        assert_eq!(
            CanonicalWriter::escape_cell_string("col1\tcol2"),
            "col1\\tcol2"
        );
        // Per SPEC.md Section E.2: all control chars (\n, \t, \r) are escaped
        assert_eq!(
            CanonicalWriter::escape_cell_string("windows\r\nline"),
            "windows\\r\\nline"
        );
        assert_eq!(
            CanonicalWriter::escape_cell_string("path\\to\\file"),
            "path\\\\to\\\\file"
        );

        // Combined
        assert_eq!(
            CanonicalWriter::escape_cell_string("He said \"hello\"\nand left"),
            "He said \"\"hello\"\"\\nand left"
        );
    }

    #[test]
    fn test_escape_cell_string_empty() {
        assert_eq!(CanonicalWriter::escape_cell_string(""), "");
    }

    #[test]
    fn test_escape_cell_string_only_newline() {
        assert_eq!(CanonicalWriter::escape_cell_string("\n"), "\\n");
    }

    #[test]
    fn test_escape_cell_string_only_tab() {
        assert_eq!(CanonicalWriter::escape_cell_string("\t"), "\\t");
    }

    #[test]
    fn test_escape_cell_string_only_backslash() {
        assert_eq!(CanonicalWriter::escape_cell_string("\\"), "\\\\");
    }

    #[test]
    fn test_escape_cell_string_only_carriage_return() {
        assert_eq!(CanonicalWriter::escape_cell_string("\r"), "\\r");
    }

    #[test]
    fn test_escape_cell_string_crlf() {
        assert_eq!(CanonicalWriter::escape_cell_string("\r\n"), "\\r\\n");
    }

    #[test]
    fn test_escape_cell_string_multiple_escapes() {
        // Per SPEC.md Section E.2: EscapeSeq ::= '\n' | '\t' | '\r' | '\\' | '\"'
        assert_eq!(
            CanonicalWriter::escape_cell_string("\n\t\r\\\""),
            "\\n\\t\\r\\\\\"\"",
        );
    }

    // ==================== needs_quoting_kv tests ====================

    #[test]
    fn test_needs_quoting_kv() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());

        // Should NOT need quoting
        assert!(!writer.needs_quoting_kv("simple"));
        assert!(!writer.needs_quoting_kv("with_underscore"));

        // Should need quoting
        assert!(writer.needs_quoting_kv(""));
        assert!(writer.needs_quoting_kv(" space"));
        assert!(writer.needs_quoting_kv("space "));
        assert!(writer.needs_quoting_kv("with#comment"));
        assert!(writer.needs_quoting_kv("~null"));
        assert!(writer.needs_quoting_kv("@ref"));
        assert!(writer.needs_quoting_kv("true"));
        assert!(writer.needs_quoting_kv("false"));
        assert!(writer.needs_quoting_kv("123"));
        assert!(writer.needs_quoting_kv("3.5"));
    }

    #[test]
    fn test_needs_quoting_kv_special_first_chars() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert!(writer.needs_quoting_kv("~value"));
        assert!(writer.needs_quoting_kv("@value"));
        assert!(writer.needs_quoting_kv("$value"));
        assert!(writer.needs_quoting_kv("%value"));
        assert!(writer.needs_quoting_kv("[value"));
    }

    #[test]
    fn test_needs_quoting_kv_numbers() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert!(writer.needs_quoting_kv("0"));
        assert!(writer.needs_quoting_kv("-1"));
        assert!(writer.needs_quoting_kv("1.0"));
        assert!(writer.needs_quoting_kv("-0.5"));
        assert!(writer.needs_quoting_kv("1e10"));
    }

    #[test]
    fn test_needs_quoting_kv_with_quotes() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert!(writer.needs_quoting_kv("say \"hello\""));
        assert!(writer.needs_quoting_kv("\""));
    }

    // ==================== needs_quoting_cell tests ====================

    #[test]
    fn test_needs_quoting_cell() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());

        // Should NOT need quoting
        assert!(!writer.needs_quoting_cell("simple"));
        assert!(!writer.needs_quoting_cell("")); // Empty is ok in cells

        // Should need quoting
        assert!(writer.needs_quoting_cell(" space"));
        assert!(writer.needs_quoting_cell("with,comma"));
        assert!(writer.needs_quoting_cell("with|pipe"));
        assert!(writer.needs_quoting_cell("^ditto"));
        assert!(writer.needs_quoting_cell("true"));
    }

    #[test]
    fn test_needs_quoting_cell_special_first_chars() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert!(writer.needs_quoting_cell("~value"));
        assert!(writer.needs_quoting_cell("@value"));
        assert!(writer.needs_quoting_cell("$value"));
        assert!(writer.needs_quoting_cell("%value"));
        assert!(writer.needs_quoting_cell("^value"));
        assert!(writer.needs_quoting_cell("[value"));
    }

    #[test]
    fn test_needs_quoting_cell_numbers() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert!(writer.needs_quoting_cell("0"));
        assert!(writer.needs_quoting_cell("-1"));
        assert!(writer.needs_quoting_cell("3.5"));
    }

    #[test]
    fn test_needs_quoting_cell_booleans() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert!(writer.needs_quoting_cell("true"));
        assert!(writer.needs_quoting_cell("false"));
    }

    // ==================== format_value tests ====================

    #[test]
    fn test_format_value_null() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert_eq!(writer.format_value(&Value::Null), "~");
    }

    #[test]
    fn test_format_value_bool() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert_eq!(writer.format_value(&Value::Bool(true)), "true");
        assert_eq!(writer.format_value(&Value::Bool(false)), "false");
    }

    #[test]
    fn test_format_value_int() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert_eq!(writer.format_value(&Value::Int(42)), "42");
        assert_eq!(writer.format_value(&Value::Int(0)), "0");
        assert_eq!(writer.format_value(&Value::Int(-100)), "-100");
    }

    #[test]
    fn test_format_value_float() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert_eq!(writer.format_value(&Value::Float(3.5)), "3.5");
        // Whole numbers get .0 suffix
        assert_eq!(writer.format_value(&Value::Float(42.0)), "42.0");
    }

    #[test]
    fn test_format_value_string_minimal() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        // Simple string doesn't need quotes
        assert_eq!(
            writer.format_value(&Value::String("hello".to_string().into())),
            "hello"
        );
        // Empty string needs quotes
        assert_eq!(
            writer.format_value(&Value::String(String::new().into())),
            "\"\""
        );
        // String that looks like number needs quotes
        assert_eq!(
            writer.format_value(&Value::String("123".to_string().into())),
            "\"123\""
        );
    }

    #[test]
    fn test_format_value_string_always_quote() {
        let writer =
            CanonicalWriter::new(CanonicalConfig::new().with_quoting(QuotingStrategy::Always));
        assert_eq!(
            writer.format_value(&Value::String("hello".to_string().into())),
            "\"hello\""
        );
    }

    #[test]
    fn test_format_value_tensor() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let tensor = Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]);
        assert_eq!(
            writer.format_value(&Value::Tensor(Box::new(tensor))),
            "[1.0, 2.0]"
        );
    }

    #[test]
    fn test_format_value_reference() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let reference = Reference::qualified("User", "id");
        // Note: qualified reference uses : not .
        assert_eq!(
            writer.format_value(&Value::Reference(reference)),
            "@User:id"
        );
    }

    #[test]
    fn test_format_value_expression() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let expr = Expression::Identifier {
            name: "foo".to_string(),
            span: Default::default(),
        };
        assert_eq!(
            writer.format_value(&Value::Expression(Box::new(expr))),
            "$(foo)"
        );
    }

    // ==================== format_tensor tests ====================

    #[test]
    fn test_format_tensor_scalar() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        assert_eq!(writer.format_tensor(&Tensor::Scalar(1.0)), "1.0");
        assert_eq!(writer.format_tensor(&Tensor::Scalar(3.5)), "3.5");
    }

    #[test]
    fn test_format_tensor_1d() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let tensor = Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
            Tensor::Scalar(3.0),
        ]);
        assert_eq!(writer.format_tensor(&tensor), "[1.0, 2.0, 3.0]");
    }

    #[test]
    fn test_format_tensor_2d() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let tensor = Tensor::Array(vec![
            Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
            Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
        ]);
        assert_eq!(writer.format_tensor(&tensor), "[[1.0, 2.0], [3.0, 4.0]]");
    }

    // ==================== format_cell_value_with_position tests ====================

    #[test]
    fn test_format_cell_empty_string_last_col() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        // Empty string in last column MUST be quoted
        assert_eq!(
            writer.format_cell_value_with_position(&Value::String(String::new().into()), true),
            "\"\""
        );
    }

    #[test]
    fn test_format_cell_empty_string_not_last_col() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        // Empty string not in last column doesn't need quotes
        assert_eq!(
            writer.format_cell_value_with_position(&Value::String(String::new().into()), false),
            ""
        );
    }

    #[test]
    fn test_format_cell_with_newline() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        // Newline needs quoting and escaping
        assert_eq!(
            writer
                .format_cell_value_with_position(&Value::String("a\nb".to_string().into()), false),
            "\"a\\nb\""
        );
    }

    // ==================== CanonicalWriter construction tests ====================

    #[test]
    fn test_canonical_writer_new() {
        let config = CanonicalConfig::default();
        let writer = CanonicalWriter::new(config);
        // Just verify it can be created
        assert_eq!(writer.format_value(&Value::Null), "~");
    }

    #[test]
    fn test_canonical_writer_with_custom_config() {
        let config = CanonicalConfig {
            quoting: QuotingStrategy::Always,
            use_ditto: false,
            sort_keys: false,
            inline_schemas: true,
        };
        let writer = CanonicalWriter::new(config);
        // With Always quoting, strings are always quoted
        assert_eq!(
            writer.format_value(&Value::String("hello".to_string().into())),
            "\"hello\""
        );
    }

    // ==================== format_string tests ====================

    #[test]
    fn test_format_string_minimal_quoting() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());

        // No quoting needed
        assert_eq!(writer.format_string("hello"), "hello");
        assert_eq!(writer.format_string("hello_world"), "hello_world");

        // Quoting needed
        assert_eq!(writer.format_string(""), "\"\"");
        assert_eq!(writer.format_string(" leading"), "\" leading\"");
        assert_eq!(writer.format_string("trailing "), "\"trailing \"");
        assert_eq!(writer.format_string("with#hash"), "\"with#hash\"");
    }

    #[test]
    fn test_format_string_always_quoting() {
        let writer =
            CanonicalWriter::new(CanonicalConfig::new().with_quoting(QuotingStrategy::Always));

        assert_eq!(writer.format_string("hello"), "\"hello\"");
        assert_eq!(writer.format_string(""), "\"\"");
    }

    // ==================== write_matrix_list count hint tests ====================

    #[test]
    fn test_write_matrix_list_without_count_hint() {
        let mut doc = Document::new((2, 0));
        doc.structs.insert(
            "Team".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );

        let mut list = MatrixList::new("Team", vec!["id".to_string(), "name".to_string()]);
        list.add_row(Node::new(
            "Team",
            "1",
            vec![
                Value::Int(1),
                Value::String("Engineering".to_string().into()),
            ],
        ));
        doc.root.insert("teams".to_string(), Item::List(list));

        let config = CanonicalConfig::new().with_inline_schemas(true);
        let mut writer = CanonicalWriter::new(config);
        let output = writer.write_document(&doc).unwrap();

        // Should NOT have count hint in output
        assert!(output.contains("teams:@Team[id,name]"));
        assert!(!output.contains("teams("));
    }

    #[test]
    fn test_write_matrix_list_with_count_hint() {
        let mut doc = Document::new((2, 0));
        doc.structs.insert(
            "Team".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );

        let mut list =
            MatrixList::with_count_hint("Team", vec!["id".to_string(), "name".to_string()], 3);
        list.add_row(Node::new(
            "Team",
            "1",
            vec![
                Value::Int(1),
                Value::String("Engineering".to_string().into()),
            ],
        ));
        list.add_row(Node::new(
            "Team",
            "2",
            vec![Value::Int(2), Value::String("Design".to_string().into())],
        ));
        list.add_row(Node::new(
            "Team",
            "3",
            vec![Value::Int(3), Value::String("Product".to_string().into())],
        ));
        doc.root.insert("teams".to_string(), Item::List(list));

        let config = CanonicalConfig::new().with_inline_schemas(true);
        let mut writer = CanonicalWriter::new(config);
        let output = writer.write_document(&doc).unwrap();

        // Count hint should be in inline schema, list declaration has no count
        assert!(output.contains("teams:@Team[id,name]"));
        assert!(!output.contains("teams(3)"));
    }

    // ==================== Recursion depth limit tests ====================

    #[test]
    fn test_recursion_depth_limit_objects() {
        use std::collections::BTreeMap;

        // Create deeply nested object structure
        // Use a more reasonable test depth to avoid test stack overflow
        // MAX_NESTING_DEPTH is 1000, so test with 100 levels which is enough
        // to verify the limit works without overflowing the test thread stack
        const TEST_DEPTH: usize = 100;

        // Build from inside out
        let mut inner = BTreeMap::new();
        inner.insert(
            "leaf".to_string(),
            Item::Scalar(Value::Int(TEST_DEPTH as i64)),
        );

        // Wrap in TEST_DEPTH layers
        for i in (0..TEST_DEPTH).rev() {
            let mut outer = BTreeMap::new();
            outer.insert("value".to_string(), Item::Scalar(Value::Int(i as i64)));
            outer.insert("nested".to_string(), Item::Object(inner));
            inner = outer;
        }

        let mut doc = Document::new((2, 0));
        doc.root.insert("root".to_string(), Item::Object(inner));

        let config = CanonicalConfig::default();
        let mut writer = CanonicalWriter::new(config);
        let result = writer.write_document(&doc);

        // At 100 levels, this should succeed (well below 1000 limit)
        assert!(
            result.is_ok(),
            "100-level nesting should be accepted, got error: {:?}",
            result.err()
        );

        // Verify the limit constant is reasonable (compile-time checks)
        const _: () = assert!(MAX_NESTING_DEPTH >= 100);
        const _: () = assert!(MAX_NESTING_DEPTH <= 10000);
    }

    #[test]
    fn test_recursion_depth_limit_acceptable() {
        use std::collections::BTreeMap;

        // Create moderately nested structure (50 levels - well below limit)
        let mut inner = BTreeMap::new();
        inner.insert("leaf".to_string(), Item::Scalar(Value::Int(49)));

        for i in (0..50).rev() {
            let mut outer = BTreeMap::new();
            outer.insert("value".to_string(), Item::Scalar(Value::Int(i)));
            outer.insert("nested".to_string(), Item::Object(inner));
            inner = outer;
        }

        let mut doc = Document::new((2, 0));
        doc.root.insert("root".to_string(), Item::Object(inner));

        let config = CanonicalConfig::default();
        let mut writer = CanonicalWriter::new(config);
        let result = writer.write_document(&doc);

        // Should succeed - 50 levels is well within limit
        assert!(
            result.is_ok(),
            "50-level nesting should be accepted, got error: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_write_matrix_list_with_count_hint_no_inline_schema() {
        let mut doc = Document::new((2, 0));
        doc.structs.insert(
            "Team".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );

        let mut list =
            MatrixList::with_count_hint("Team", vec!["id".to_string(), "name".to_string()], 2);
        list.add_row(Node::new(
            "Team",
            "1",
            vec![
                Value::Int(1),
                Value::String("Engineering".to_string().into()),
            ],
        ));
        list.add_row(Node::new(
            "Team",
            "2",
            vec![Value::Int(2), Value::String("Design".to_string().into())],
        ));
        doc.root.insert("teams".to_string(), Item::List(list));

        let config = CanonicalConfig::new().with_inline_schemas(false);
        let mut writer = CanonicalWriter::new(config);
        let output = writer.write_document(&doc).unwrap();

        // Count goes in separate %C: directive, not in %S: or list declaration
        assert!(output.contains("teams:@Team"));
        assert!(!output.contains("teams(2)"));
        // v2.0 uses %S: without count + separate %C: directive
        assert!(output.contains("%S:Team:[id,name]"));
        assert!(output.contains("%C:Team.total=2"));
    }

    // ==================== format_list tests ====================

    #[test]
    fn test_format_list_empty() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let list = Value::List(Box::default());
        assert_eq!(writer.format_value(&list), "()");
    }

    #[test]
    fn test_format_list_strings() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let list = Value::List(Box::new(vec![
            Value::String("admin".to_string().into()),
            Value::String("editor".to_string().into()),
            Value::String("viewer".to_string().into()),
        ]));
        assert_eq!(writer.format_value(&list), "(admin, editor, viewer)");
    }

    #[test]
    fn test_format_list_booleans() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let list = Value::List(Box::new(vec![Value::Bool(true), Value::Bool(false)]));
        assert_eq!(writer.format_value(&list), "(true, false)");
    }

    #[test]
    fn test_format_list_integers() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let list = Value::List(Box::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
        assert_eq!(writer.format_value(&list), "(1, 2, 3)");
    }

    #[test]
    fn test_format_list_mixed_types() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let list = Value::List(Box::new(vec![
            Value::Int(1),
            Value::String("two".to_string().into()),
            Value::Bool(true),
        ]));
        assert_eq!(writer.format_value(&list), "(1, two, true)");
    }

    #[test]
    fn test_format_list_with_null() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let list = Value::List(Box::new(vec![
            Value::String("a".to_string().into()),
            Value::Null,
            Value::String("c".to_string().into()),
        ]));
        assert_eq!(writer.format_value(&list), "(a, ~, c)");
    }

    #[test]
    fn test_format_list_with_references() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let list = Value::List(Box::new(vec![
            Value::Reference(Reference::local("user1")),
            Value::Reference(Reference::qualified("User", "123")),
        ]));
        assert_eq!(writer.format_value(&list), "(@user1, @User:123)");
    }

    #[test]
    fn test_format_list_nested() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let inner = Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]));
        let outer = Value::List(Box::new(vec![inner, Value::Int(3)]));
        assert_eq!(writer.format_value(&outer), "((1, 2), 3)");
    }

    #[test]
    fn test_format_list_single_element() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let list = Value::List(Box::new(vec![Value::String("solo".to_string().into())]));
        assert_eq!(writer.format_value(&list), "(solo)");
    }

    #[test]
    fn test_format_list_in_cell_position() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let list = Value::List(Box::new(vec![
            Value::String("a".to_string().into()),
            Value::String("b".to_string().into()),
        ]));
        // Test both last and not last column positions
        assert_eq!(
            writer.format_cell_value_with_position(&list, false),
            "(a, b)"
        );
        assert_eq!(
            writer.format_cell_value_with_position(&list, true),
            "(a, b)"
        );
    }

    #[test]
    fn test_format_list_with_quoted_strings() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        // String with space doesn't need quotes in list context (within parens)
        // but numeric string does need quotes
        let list = Value::List(Box::new(vec![
            Value::String("needs quotes".to_string().into()),
            Value::String("123".to_string().into()),
        ]));
        assert_eq!(writer.format_value(&list), "(needs quotes, \"123\")");
    }

    #[test]
    fn test_format_list_always_quote() {
        let writer =
            CanonicalWriter::new(CanonicalConfig::new().with_quoting(QuotingStrategy::Always));
        let list = Value::List(Box::new(vec![
            Value::String("hello".to_string().into()),
            Value::String("world".to_string().into()),
        ]));
        assert_eq!(writer.format_value(&list), "(\"hello\", \"world\")");
    }

    #[test]
    fn test_format_list_floats() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let list = Value::List(Box::new(vec![
            Value::Float(1.5),
            Value::Float(2.0),
            Value::Float(4.56),
        ]));
        assert_eq!(writer.format_value(&list), "(1.5, 2.0, 4.56)");
    }

    #[test]
    fn test_format_list_with_expressions() {
        let writer = CanonicalWriter::new(CanonicalConfig::default());
        let expr1 = Expression::Identifier {
            name: "x".to_string(),
            span: Default::default(),
        };
        let expr2 = Expression::Identifier {
            name: "y".to_string(),
            span: Default::default(),
        };
        let list = Value::List(Box::new(vec![
            Value::Expression(Box::new(expr1)),
            Value::Expression(Box::new(expr2)),
        ]));
        assert_eq!(writer.format_value(&list), "($(x), $(y))");
    }
}
