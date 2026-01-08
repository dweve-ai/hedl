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

//! Integration tests for hedl-csv.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_csv::{from_csv, to_csv, FromCsvConfig, ToCsvConfig};
use hedl_test::expr_value;

#[test]
fn test_complete_round_trip() {
    // Create a document with various value types
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Record",
        vec![
            "id".to_string(),
            "string".to_string(),
            "int".to_string(),
            "float".to_string(),
            "bool".to_string(),
            "null".to_string(),
            "ref".to_string(),
        ],
    );

    list.add_row(Node::new(
        "Record",
        "r1",
        vec![
            Value::String("r1".to_string()),
            Value::String("hello".to_string()),
            Value::Int(42),
            Value::Float(3.25),
            Value::Bool(true),
            Value::Null,
            Value::Reference(hedl_core::Reference::local("ref1")),
        ],
    ));

    doc.root.insert("records".to_string(), Item::List(list));

    // Convert to CSV
    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("hello"));
    assert!(csv.contains("42"));
    assert!(csv.contains("3.25"));
    assert!(csv.contains("true"));
    assert!(csv.contains("@ref1"));

    // Convert back to HEDL
    let doc2 = from_csv(
        &csv,
        "Record",
        &["string", "int", "float", "bool", "null", "ref"],
    )
    .unwrap();

    // Verify the data
    let list2 = doc2.get("records").unwrap().as_list().unwrap();
    let row = &list2.rows[0];

    assert_eq!(row.id, "r1");
    assert_eq!(row.fields[0], Value::String("r1".to_string())); // ID field (stays string)
    assert_eq!(row.fields[1], Value::String("hello".to_string()));
    assert_eq!(row.fields[2], Value::Int(42));
    assert_eq!(row.fields[3], Value::Float(3.25));
    assert_eq!(row.fields[4], Value::Bool(true));
    assert_eq!(row.fields[5], Value::Null);
    assert!(matches!(row.fields[6], Value::Reference(_)));
}

#[test]
fn test_large_dataset() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    // Create 1000 rows
    for i in 0..1000 {
        list.add_row(Node::new(
            "Item",
            format!("id_{}", i),
            vec![Value::String(format!("id_{}", i)), Value::Int(i)],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(&csv, "Item", &["value"]).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(list2.rows.len(), 1000);

    // Verify first and last
    assert_eq!(list2.rows[0].id, "id_0");
    assert_eq!(list2.rows[0].fields[0], Value::String("id_0".to_string())); // ID field
    assert_eq!(list2.rows[0].fields[1], Value::Int(0));
    assert_eq!(list2.rows[999].id, "id_999");
    assert_eq!(
        list2.rows[999].fields[0],
        Value::String("id_999".to_string())
    ); // ID field
    assert_eq!(list2.rows[999].fields[1], Value::Int(999));
}

#[test]
fn test_csv_with_special_characters() {
    let csv_data = r#"id,name,description
1,"Alice, Bob",A person with comma
2,"Charlie ""The Great""",A person with quotes
3,"Eve
Newline",A person with newline
"#;

    let doc = from_csv(csv_data, "Person", &["name", "description"]).unwrap();
    let list = doc.get("persons").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 3);
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Alice, Bob".to_string())
    );
    assert_eq!(list.rows[1].fields[0], Value::Int(2)); // ID field
    assert_eq!(
        list.rows[1].fields[1],
        Value::String("Charlie \"The Great\"".to_string())
    );
    assert_eq!(list.rows[2].fields[0], Value::Int(3)); // ID field
    assert_eq!(
        list.rows[2].fields[1],
        Value::String("Eve\nNewline".to_string())
    );
}

#[test]
fn test_tsv_format() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Data",
        vec!["id".to_string(), "col1".to_string(), "col2".to_string()],
    );

    list.add_row(Node::new(
        "Data",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::String("a".to_string()),
            Value::String("b".to_string()),
        ],
    ));

    doc.root.insert("data".to_string(), Item::List(list));

    // Convert to TSV
    let config = ToCsvConfig {
        delimiter: b'\t',
        ..Default::default()
    };
    let tsv = hedl_csv::to_csv_with_config(&doc, config).unwrap();
    assert!(tsv.contains('\t'));
    assert!(!tsv.contains(','));

    // Parse TSV
    let config = FromCsvConfig {
        delimiter: b'\t',
        ..Default::default()
    };
    let doc2 = hedl_csv::from_csv_with_config(&tsv, "Data", &["col1", "col2"], config).unwrap();

    let list2 = doc2.get("datas").unwrap().as_list().unwrap();
    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list2.rows[0].fields[1], Value::String("a".to_string()));
    assert_eq!(list2.rows[0].fields[2], Value::String("b".to_string()));
}

#[test]
fn test_empty_values_and_null() {
    let csv_data = "id,value\n1,\n2,~\n3,null\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 3);
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Null); // Empty string
    assert_eq!(list.rows[1].fields[0], Value::Int(2)); // ID field
    assert_eq!(list.rows[1].fields[1], Value::Null); // Tilde
    assert_eq!(list.rows[2].fields[0], Value::Int(3)); // ID field
    assert_eq!(list.rows[2].fields[1], Value::String("null".to_string())); // String "null"
}

#[test]
fn test_numeric_edge_cases() {
    let csv_data = "id,int,float\n1,0,-0.0\n2,9223372036854775807,1.7976931348623157e308\n3,-9223372036854775808,-1.7976931348623157e308\n";
    let doc = from_csv(csv_data, "Number", &["int", "float"]).unwrap();
    let list = doc.get("numbers").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 3);

    // Zero values
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Int(0));
    assert_eq!(list.rows[0].fields[2], Value::Float(-0.0));

    // Max values
    assert_eq!(list.rows[1].fields[0], Value::Int(2)); // ID field
    assert_eq!(list.rows[1].fields[1], Value::Int(i64::MAX));
    assert_eq!(list.rows[1].fields[2], Value::Float(f64::MAX));

    // Min values
    assert_eq!(list.rows[2].fields[0], Value::Int(3)); // ID field
    assert_eq!(list.rows[2].fields[1], Value::Int(i64::MIN));
    assert_eq!(list.rows[2].fields[2], Value::Float(f64::MIN));
}

#[test]
fn test_qualified_references() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Link", vec!["id".to_string(), "target".to_string()]);

    list.add_row(Node::new(
        "Link",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::Reference(hedl_core::Reference::qualified("User", "alice")),
        ],
    ));

    doc.root.insert("links".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("@User:alice"));

    let doc2 = from_csv(&csv, "Link", &["target"]).unwrap();
    let list2 = doc2.get("links").unwrap().as_list().unwrap();

    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
    if let Value::Reference(ref r) = list2.rows[0].fields[1] {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "alice");
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_expressions() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Formula", vec!["id".to_string(), "expr".to_string()]);

    list.add_row(Node::new(
        "Formula",
        "1",
        vec![
            Value::String("1".to_string()),
            expr_value("add(x, multiply(y, 2))"),
        ],
    ));

    doc.root.insert("formulas".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("$(add(x, multiply(y, 2)))"));

    let doc2 = from_csv(&csv, "Formula", &["expr"]).unwrap();
    let list2 = doc2.get("formulas").unwrap().as_list().unwrap();

    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(
        list2.rows[0].fields[1],
        expr_value("add(x, multiply(y, 2))")
    );
}

#[test]
fn test_unicode_data() {
    let csv_data = "id,text\n1,Hello 世界\n2,Привет мир\n3,مرحبا العالم\n";
    let doc = from_csv(csv_data, "Text", &["text"]).unwrap();
    let list = doc.get("texts").unwrap().as_list().unwrap();

    assert_eq!(list.rows.len(), 3);
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Hello 世界".to_string())
    );
    assert_eq!(list.rows[1].fields[0], Value::Int(2)); // ID field
    assert_eq!(
        list.rows[1].fields[1],
        Value::String("Привет мир".to_string())
    );
    assert_eq!(list.rows[2].fields[0], Value::Int(3)); // ID field
    assert_eq!(
        list.rows[2].fields[1],
        Value::String("مرحبا العالم".to_string())
    );
}
