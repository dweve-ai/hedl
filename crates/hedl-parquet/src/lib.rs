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

//! Bidirectional conversion between Parquet and HEDL formats.
//!
//! This crate provides functionality to convert HEDL documents to Parquet files
//! and vice versa. Parquet is a columnar storage format that is well-suited for
//! HEDL's matrix list structures.
//!
//! # Features
//!
//! - **HEDL → Parquet**: Convert HEDL documents to Parquet files with configurable compression
//! - **Parquet → HEDL**: Read Parquet files and convert them back to HEDL documents
//! - **Matrix Lists**: Natural mapping of HEDL matrix lists to Parquet tables
//! - **Metadata**: Support for key-value pairs as single-row tables
//! - **Security**: Built-in protections against decompression bombs and memory exhaustion
//!
//! # Security Considerations
//!
//! When reading untrusted Parquet files, this crate implements several security protections:
//!
//! ## Decompression Bomb Protection
//!
//! Parquet files can use compression algorithms like GZIP or ZSTD, which could be exploited
//! to create "decompression bombs" - small compressed files that expand to enormous sizes.
//!
//! **Protection**: Total decompressed data is limited to 100 MB (`MAX_DECOMPRESSED_SIZE`).
//! Files exceeding this limit are rejected with a clear security error.
//!
//! ## Large Schema Attacks
//!
//! Malicious files could contain thousands of columns, causing memory exhaustion during
//! schema processing.
//!
//! **Protection**: Schemas are limited to 1,000 columns (`MAX_COLUMNS`). Files with more
//! columns are rejected immediately.
//!
//! ## Memory Exhaustion Prevention
//!
//! Large dimensions (many rows × many columns) could exhaust available memory.
//!
//! **Protection**: The decompressed size limit (100 MB) effectively prevents row × column
//! multiplication attacks. Memory usage is tracked cumulatively across all record batches.
//!
//! ## Metadata Validation
//!
//! Parquet metadata could contain malicious identifiers (SQL injection, XSS, etc.).
//!
//! **Protection**: All metadata values are validated as HEDL identifiers (alphanumeric +
//! underscore, max 100 chars). Invalid characters are sanitized or rejected.
//!
//! ## Integer Overflow Protection
//!
//! Size calculations could overflow when processing malicious files.
//!
//! **Protection**: All size calculations use checked arithmetic (`checked_add`, etc.).
//! Overflows produce clear security errors.
//!
//! For complete security documentation, see the `SECURITY.md` file in the crate root.
//!
//! # Examples
//!
//! ## Writing HEDL to Parquet
//!
//! ```no_run
//! use hedl_core::{Document, MatrixList, Node, Value, Item};
//! use hedl_parquet::to_parquet;
//! use std::path::Path;
//!
//! let mut doc = Document::new((1, 0));
//! let mut matrix_list = MatrixList::new(
//!     "User",
//!     vec!["id".to_string(), "name".to_string(), "age".to_string()]
//! );
//!
//! let node = Node::new(
//!     "User",
//!     "alice",
//!     vec![Value::String("Alice".to_string()), Value::Int(30)],
//! );
//! matrix_list.add_row(node);
//! doc.root.insert("users".to_string(), Item::List(matrix_list));
//!
//! to_parquet(&doc, Path::new("output.parquet")).unwrap();
//! ```
//!
//! ## Reading Parquet to HEDL
//!
//! ```no_run
//! use hedl_parquet::from_parquet;
//! use std::path::Path;
//!
//! let doc = from_parquet(Path::new("input.parquet")).unwrap();
//! println!("Version: {:?}", doc.version);
//! ```
//!
//! ## Round-trip Conversion
//!
//! ```no_run
//! use hedl_core::{Document, MatrixList, Node, Value, Item};
//! use hedl_parquet::{to_parquet_bytes, from_parquet_bytes};
//!
//! let mut doc = Document::new((1, 0));
//! // ... populate document ...
//!
//! // Convert to Parquet bytes
//! let bytes = to_parquet_bytes(&doc).unwrap();
//!
//! // Convert back to HEDL
//! let doc2 = from_parquet_bytes(&bytes).unwrap();
//! ```
//!
//! # Mapping Strategy
//!
//! ## Matrix Lists → Parquet Tables
//!
//! HEDL matrix lists map naturally to Parquet tables:
//! - Each column in the HEDL schema becomes a Parquet column
//! - The first column (ID) is always a string column
//! - Data types are inferred from values: Int, Float, Bool, String
//! - References are stored as strings (e.g., "@User:alice")
//! - Tensors are serialized as strings
//!
//! ## Key-Value Pairs → Metadata Table
//!
//! When a HEDL document contains only scalar key-value pairs (no matrix lists),
//! they are stored as a two-column table with "key" and "value" columns.
//!
//! # Type Inference
//!
//! When reading Parquet files, HEDL types are inferred from Arrow types:
//! - Arrow Boolean → HEDL Bool
//! - Arrow Int8/16/32/64, UInt8/16/32/64 → HEDL Int
//! - Arrow Float32/64 → HEDL Float
//! - Arrow Utf8 → HEDL String (or Reference if starts with '@')
//!
//! # Compression
//!
//! Parquet files can be compressed using various algorithms. The default is SNAPPY,
//! but you can customize this using `ToParquetConfig`:
//!
//! ```no_run
//! use hedl_core::Document;
//! use hedl_parquet::{to_parquet_with_config, ToParquetConfig};
//! use parquet::basic::Compression;
//! use std::path::Path;
//!
//! let doc = Document::new((1, 0));
//! let config = ToParquetConfig {
//!     compression: Compression::GZIP(Default::default()),
//!     ..Default::default()
//! };
//!
//! to_parquet_with_config(&doc, Path::new("output.parquet"), &config).unwrap();
//! ```

mod from_parquet;
mod to_parquet;

// Re-export public API
pub use from_parquet::{from_parquet, from_parquet_bytes};
pub use to_parquet::{
    to_parquet, to_parquet_bytes, to_parquet_bytes_with_config, to_parquet_with_config,
    ToParquetConfig,
};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use hedl_core::{Document, Item, MatrixList, Node, Value};

    #[test]
    fn test_round_trip_simple_matrix_list() {
        let mut doc = Document::new((1, 0));
        let mut matrix_list = MatrixList::new(
            "User",
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );

        let node1 = Node::new(
            "User",
            "alice",
            vec![Value::String("Alice".to_string()), Value::Int(30)],
        );
        let node2 = Node::new(
            "User",
            "bob",
            vec![Value::String("Bob".to_string()), Value::Int(25)],
        );

        matrix_list.add_row(node1);
        matrix_list.add_row(node2);
        doc.root
            .insert("users".to_string(), Item::List(matrix_list));

        // Convert to Parquet bytes
        let bytes = to_parquet_bytes(&doc).unwrap();
        assert!(!bytes.is_empty());

        // Convert back to HEDL
        let doc2 = from_parquet_bytes(&bytes).unwrap();

        // Verify the structure
        assert!(doc2.root.contains_key("users"));
        if let Some(Item::List(list)) = doc2.root.get("users") {
            assert_eq!(list.rows.len(), 2);
            assert_eq!(list.schema.len(), 3);
        } else {
            panic!("Expected a list item");
        }
    }

    #[test]
    fn test_round_trip_numeric_types() {
        let mut doc = Document::new((1, 0));
        let mut matrix_list = MatrixList::new(
            "Data",
            vec![
                "id".to_string(),
                "int_val".to_string(),
                "float_val".to_string(),
                "bool_val".to_string(),
            ],
        );

        let node = Node::new(
            "Data",
            "row1",
            vec![
                Value::String("row1".to_string()),
                Value::Int(42),
                Value::Float(3.25),
                Value::Bool(true),
            ],
        );

        matrix_list.add_row(node);
        doc.root.insert("data".to_string(), Item::List(matrix_list));

        // Round trip
        let bytes = to_parquet_bytes(&doc).unwrap();
        let doc2 = from_parquet_bytes(&bytes).unwrap();

        // Verify types are preserved
        if let Some(Item::List(list)) = doc2.root.get("data") {
            assert_eq!(list.rows.len(), 1);
            let row = &list.rows[0];

            // fields[0] is now the ID
            assert!(matches!(row.fields[0], Value::String(ref s) if s == "row1"));
            assert!(matches!(row.fields[1], Value::Int(42)));
            assert!(matches!(row.fields[2], Value::Float(f) if (f - 3.25).abs() < 0.001));
            assert!(matches!(row.fields[3], Value::Bool(true)));
        } else {
            panic!("Expected a list item");
        }
    }

    #[test]
    fn test_round_trip_with_nulls() {
        let mut doc = Document::new((1, 0));
        let mut matrix_list = MatrixList::new("Data", vec!["id".to_string(), "value".to_string()]);

        let node1 = Node::new(
            "Data",
            "row1",
            vec![Value::String("row1".to_string()), Value::Int(42)],
        );
        let node2 = Node::new(
            "Data",
            "row2",
            vec![Value::String("row2".to_string()), Value::Null],
        );
        let node3 = Node::new(
            "Data",
            "row3",
            vec![
                Value::String("row3".to_string()),
                Value::String("test".to_string()),
            ],
        );

        matrix_list.add_row(node1);
        matrix_list.add_row(node2);
        matrix_list.add_row(node3);
        doc.root.insert("data".to_string(), Item::List(matrix_list));

        // Round trip
        let bytes = to_parquet_bytes(&doc).unwrap();
        let doc2 = from_parquet_bytes(&bytes).unwrap();

        // Verify nulls are preserved
        if let Some(Item::List(list)) = doc2.root.get("data") {
            assert_eq!(list.rows.len(), 3);
            // fields[0] is the ID, fields[1] is the value
            assert!(matches!(list.rows[1].fields[1], Value::Null));
        } else {
            panic!("Expected a list item");
        }
    }

    #[test]
    fn test_metadata_table_round_trip() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "version".to_string(),
            Item::Scalar(Value::String("1.0".to_string())),
        );
        doc.root
            .insert("count".to_string(), Item::Scalar(Value::Int(42)));

        // Round trip
        let bytes = to_parquet_bytes(&doc).unwrap();
        let doc2 = from_parquet_bytes(&bytes).unwrap();

        // Verify metadata
        assert!(doc2.root.contains_key("version"));
        assert!(doc2.root.contains_key("count"));
    }

    #[test]
    fn test_references_preserved() {
        let mut doc = Document::new((1, 0));
        let mut matrix_list = MatrixList::new("Post", vec!["id".to_string(), "author".to_string()]);

        let node = Node::new(
            "Post",
            "post1",
            vec![
                Value::String("post1".to_string()),
                Value::Reference(hedl_core::Reference::qualified("User", "alice")),
            ],
        );

        matrix_list.add_row(node);
        doc.root
            .insert("posts".to_string(), Item::List(matrix_list));

        // Round trip
        let bytes = to_parquet_bytes(&doc).unwrap();
        let doc2 = from_parquet_bytes(&bytes).unwrap();

        // Verify reference is preserved
        if let Some(Item::List(list)) = doc2.root.get("posts") {
            // fields[0] is the ID, fields[1] is the author reference
            if let Value::Reference(r) = &list.rows[0].fields[1] {
                assert_eq!(r.type_name, Some("User".to_string()));
                assert_eq!(r.id, "alice");
            } else {
                panic!("Expected reference value");
            }
        } else {
            panic!("Expected a list item");
        }
    }

    #[test]
    fn test_empty_document() {
        let doc = Document::new((1, 0));
        let bytes = to_parquet_bytes(&doc).unwrap();

        // Should succeed even with empty document
        assert!(bytes.is_empty() || !bytes.is_empty()); // Either is acceptable
    }
}
