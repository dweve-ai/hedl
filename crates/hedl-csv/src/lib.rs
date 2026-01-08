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

//! CSV file ↔ HEDL format bidirectional conversion.
//!
//! This crate provides functionality to convert between CSV files and HEDL documents.
//! It handles both reading CSV data into HEDL structures and writing HEDL data to CSV format.
//!
//! # Features
//!
//! - **Bidirectional conversion**: Convert HEDL → CSV and CSV → HEDL
//! - **Type inference**: Automatically infer types when reading CSV (null, bool, int, float, string, references)
//! - **Configurable**: Support for custom delimiters, quote styles, and header options
//! - **Matrix lists**: CSV tables map naturally to HEDL matrix lists
//! - **Error handling**: Comprehensive error reporting with context
//!
//! # Examples
//!
//! ## Converting HEDL to CSV
//!
//! ```no_run
//! use hedl_core::{Document, Item, MatrixList, Node, Value};
//! use hedl_csv::to_csv;
//!
//! let mut doc = Document::new((1, 0));
//! let mut list = MatrixList::new("Person", vec!["name".to_string(), "age".to_string()]);
//!
//! list.add_row(Node::new(
//!     "Person",
//!     "1",
//!     vec![Value::String("Alice".to_string()), Value::Int(30)],
//! ));
//!
//! doc.root.insert("people".to_string(), Item::List(list));
//!
//! let csv_string = to_csv(&doc).unwrap();
//! println!("{}", csv_string);
//! // Output:
//! // id,name,age
//! // 1,Alice,30
//! ```
//!
//! ## Converting CSV to HEDL
//!
//! ```no_run
//! use hedl_csv::from_csv;
//!
//! let csv_data = r#"
//! id,name,age,active
//! 1,Alice,30,true
//! 2,Bob,25,false
//! "#;
//!
//! let doc = from_csv(csv_data, "Person", &["name", "age", "active"]).unwrap();
//!
//! // Access the matrix list
//! let item = doc.get("persons").unwrap();
//! let list = item.as_list().unwrap();
//! assert_eq!(list.rows.len(), 2);
//! ```
//!
//! ## Custom Configuration
//!
//! ```no_run
//! use hedl_csv::{from_csv_with_config, to_csv_with_config, FromCsvConfig, ToCsvConfig};
//!
//! // Reading CSV with custom delimiter
//! let csv_data = "id\tname\tage\n1\tAlice\t30";
//! let config = FromCsvConfig {
//!     delimiter: b'\t',
//!     has_headers: true,
//!     trim: true,
//!     ..Default::default()
//! };
//! let doc = from_csv_with_config(csv_data, "Person", &["name", "age"], config).unwrap();
//!
//! // Writing CSV without headers
//! let config = ToCsvConfig {
//!     include_headers: false,
//!     ..Default::default()
//! };
//! let csv_string = to_csv_with_config(&doc, config).unwrap();
//! ```
//!
//! ## Custom List Keys (Irregular Plurals)
//!
//! ```no_run
//! use hedl_csv::{from_csv_with_config, FromCsvConfig};
//!
//! let csv_data = "id,name,age\n1,Alice,30\n2,Bob,25";
//!
//! // Use "people" instead of default "persons" for Person type
//! let config = FromCsvConfig {
//!     list_key: Some("people".to_string()),
//!     ..Default::default()
//! };
//! let doc = from_csv_with_config(csv_data, "Person", &["name", "age"], config).unwrap();
//!
//! // Access using the custom plural form
//! let list = doc.get("people").unwrap().as_list().unwrap();
//! assert_eq!(list.rows.len(), 2);
//! ```
//!
//! ## Selective List Export
//!
//! When a document contains multiple lists, you can export each one independently
//! without converting the entire document:
//!
//! ```no_run
//! use hedl_core::Document;
//! use hedl_csv::to_csv_list;
//!
//! let doc = Document::new((1, 0));
//! // Export only the "people" list
//! let csv_people = to_csv_list(&doc, "people").unwrap();
//! // Export only the "items" list
//! let csv_items = to_csv_list(&doc, "items").unwrap();
//! ```
//!
//! This is useful when you want to export specific tables from multi-list documents
//! without exporting everything.
//!
//! ## Round-trip Conversion
//!
//! ```no_run
//! use hedl_csv::{from_csv, to_csv};
//!
//! let original_csv = "id,name,age\n1,Alice,30\n2,Bob,25\n";
//! let doc = from_csv(original_csv, "Person", &["name", "age"]).unwrap();
//! let converted_csv = to_csv(&doc).unwrap();
//!
//! // The structure is preserved
//! assert_eq!(original_csv, converted_csv);
//! ```
//!
//! # Type Inference
//!
//! When reading CSV data, values are automatically inferred as:
//!
//! - Empty string or `~` → `Value::Null`
//! - `true` or `false` → `Value::Bool`
//! - Integer pattern → `Value::Int`
//! - Float pattern → `Value::Float`
//! - `@id` or `@Type:id` → `Value::Reference`
//! - `$(expr)` → `Value::Expression`
//! - Otherwise → `Value::String`
//!
//! Special float values are supported: `NaN`, `Infinity`, `-Infinity`

mod error;
mod from_csv;
mod to_csv;

// Re-export public API
pub use error::{CsvError, Result};
pub use from_csv::{
    from_csv, from_csv_reader, from_csv_reader_with_config, from_csv_with_config, FromCsvConfig,
};
pub use to_csv::{
    to_csv, to_csv_list, to_csv_list_with_config, to_csv_list_writer, to_csv_list_writer_with_config,
    to_csv_with_config, to_csv_writer, to_csv_writer_with_config, ToCsvConfig,
};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use hedl_core::{Document, Item, MatrixList, Node, Value};
    use hedl_test::expr_value;

    /// Test round-trip conversion: HEDL → CSV → HEDL
    #[test]
    fn test_round_trip_conversion() {
        // Create original document
        let mut doc = Document::new((1, 0));
        // Per SPEC.md: MatrixList.schema includes all column names with ID first
        let mut list = MatrixList::new(
            "Person",
            vec![
                "id".to_string(),
                "name".to_string(),
                "age".to_string(),
                "score".to_string(),
                "active".to_string(),
            ],
        );

        // Per SPEC.md: Node.fields contains ALL values including ID (first column)
        list.add_row(Node::new(
            "Person",
            "1",
            vec![
                Value::String("1".to_string()),
                Value::String("Alice".to_string()),
                Value::Int(30),
                Value::Float(95.5),
                Value::Bool(true),
            ],
        ));

        list.add_row(Node::new(
            "Person",
            "2",
            vec![
                Value::String("2".to_string()),
                Value::String("Bob".to_string()),
                Value::Int(25),
                Value::Float(87.3),
                Value::Bool(false),
            ],
        ));

        doc.root.insert("people".to_string(), Item::List(list));

        // Convert to CSV
        let csv = to_csv(&doc).unwrap();

        // Convert back to HEDL
        let doc2 = from_csv(&csv, "Person", &["name", "age", "score", "active"]).unwrap();

        // Verify structure
        let list2 = doc2.get("persons").unwrap().as_list().unwrap();
        assert_eq!(list2.rows.len(), 2);

        // Verify first row
        let row1 = &list2.rows[0];
        assert_eq!(row1.id, "1");
        assert_eq!(row1.fields[0], Value::Int(1)); // ID field
        assert_eq!(row1.fields[1], Value::String("Alice".to_string()));
        assert_eq!(row1.fields[2], Value::Int(30));
        assert_eq!(row1.fields[3], Value::Float(95.5));
        assert_eq!(row1.fields[4], Value::Bool(true));

        // Verify second row
        let row2 = &list2.rows[1];
        assert_eq!(row2.id, "2");
        assert_eq!(row2.fields[0], Value::Int(2)); // ID field
        assert_eq!(row2.fields[1], Value::String("Bob".to_string()));
        assert_eq!(row2.fields[2], Value::Int(25));
        assert_eq!(row2.fields[3], Value::Float(87.3));
        assert_eq!(row2.fields[4], Value::Bool(false));
    }

    /// Test handling of null values
    #[test]
    fn test_null_values() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

        list.add_row(Node::new(
            "Item",
            "1",
            vec![Value::String("1".to_string()), Value::Null],
        ));
        doc.root.insert("items".to_string(), Item::List(list));

        let csv = to_csv(&doc).unwrap();
        let doc2 = from_csv(&csv, "Item", &["value"]).unwrap();

        let list2 = doc2.get("items").unwrap().as_list().unwrap();
        assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
        assert_eq!(list2.rows[0].fields[1], Value::Null);
    }

    /// Test handling of references
    #[test]
    fn test_references() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Item", vec!["id".to_string(), "ref".to_string()]);

        list.add_row(Node::new(
            "Item",
            "1",
            vec![
                Value::String("1".to_string()),
                Value::Reference(hedl_core::Reference::local("user1")),
            ],
        ));

        list.add_row(Node::new(
            "Item",
            "2",
            vec![
                Value::String("2".to_string()),
                Value::Reference(hedl_core::Reference::qualified("User", "user2")),
            ],
        ));

        doc.root.insert("items".to_string(), Item::List(list));

        let csv = to_csv(&doc).unwrap();
        let doc2 = from_csv(&csv, "Item", &["ref"]).unwrap();

        let list2 = doc2.get("items").unwrap().as_list().unwrap();

        // Check local reference
        assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
        let ref1 = list2.rows[0].fields[1].as_reference().unwrap();
        assert_eq!(ref1.id, "user1");
        assert_eq!(ref1.type_name, None);

        // Check qualified reference
        assert_eq!(list2.rows[1].fields[0], Value::Int(2)); // ID field
        let ref2 = list2.rows[1].fields[1].as_reference().unwrap();
        assert_eq!(ref2.id, "user2");
        assert_eq!(ref2.type_name, Some("User".to_string()));
    }

    /// Test handling of mixed types
    #[test]
    fn test_mixed_types() {
        let csv_data = r#"
id,value
1,42
2,3.25
3,true
4,hello
5,@ref1
6,
"#;

        let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
        let list = doc.get("items").unwrap().as_list().unwrap();

        assert_eq!(list.rows.len(), 6);
        assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
        assert_eq!(list.rows[0].fields[1], Value::Int(42));
        assert_eq!(list.rows[1].fields[0], Value::Int(2)); // ID field
        assert_eq!(list.rows[1].fields[1], Value::Float(3.25));
        assert_eq!(list.rows[2].fields[0], Value::Int(3)); // ID field
        assert_eq!(list.rows[2].fields[1], Value::Bool(true));
        assert_eq!(list.rows[3].fields[0], Value::Int(4)); // ID field
        assert_eq!(list.rows[3].fields[1], Value::String("hello".to_string()));
        assert_eq!(list.rows[4].fields[0], Value::Int(5)); // ID field
        assert!(matches!(list.rows[4].fields[1], Value::Reference(_)));
        assert_eq!(list.rows[5].fields[0], Value::Int(6)); // ID field
        assert_eq!(list.rows[5].fields[1], Value::Null);
    }

    /// Test expressions
    #[test]
    fn test_expressions() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Item", vec!["id".to_string(), "expr".to_string()]);

        list.add_row(Node::new(
            "Item",
            "1",
            vec![Value::String("1".to_string()), expr_value("add(x, y)")],
        ));

        doc.root.insert("items".to_string(), Item::List(list));

        let csv = to_csv(&doc).unwrap();
        assert!(csv.contains("$(add(x, y))"));

        let doc2 = from_csv(&csv, "Item", &["expr"]).unwrap();
        let list2 = doc2.get("items").unwrap().as_list().unwrap();

        assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
        assert_eq!(list2.rows[0].fields[1], expr_value("add(x, y)"));
    }
}
