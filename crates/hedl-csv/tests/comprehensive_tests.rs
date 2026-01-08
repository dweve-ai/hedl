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

//! Comprehensive tests for hedl-csv crate.
//!
//! Tests bidirectional CSV ↔ HEDL conversion including:
//! - All scalar types (null, bool, int, float, string)
//! - References (local and qualified)
//! - Expressions
//! - Config options (delimiter, headers, trim, quote_style)
//! - Error handling
//! - Edge cases and Unicode support

use hedl_core::{Document, Item, MatrixList, Node, Reference, Tensor, Value};
use hedl_csv::{
    from_csv, from_csv_with_config, to_csv, to_csv_with_config, FromCsvConfig, ToCsvConfig,
};
use hedl_test::{expr_value, fixtures};

// =============================================================================
// HEDL to CSV - Scalar Type Tests
// =============================================================================

#[test]
fn test_null_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Null],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("1,\n") || csv.contains("1,"));
}

#[test]
fn test_bool_true_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Bool(true)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("true"));
}

#[test]
fn test_bool_false_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Bool(false)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("false"));
}

#[test]
fn test_int_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Int(42)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("42"));
}

#[test]
fn test_negative_int_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Int(-123)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("-123"));
}

#[test]
fn test_float_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Float(1.23456)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("1.23456"));
}

#[test]
fn test_negative_float_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Float(-2.5)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("-2.5"));
}

#[test]
fn test_string_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::String("hello world".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("hello world"));
}

#[test]
fn test_empty_string_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::String("".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    // Empty string should be quoted to distinguish from null
    assert!(csv.contains("1,"));
}

// =============================================================================
// HEDL to CSV - Reference Tests
// =============================================================================

#[test]
fn test_local_reference_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "ref".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::Reference(Reference::local("target")),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("@target"));
}

#[test]
fn test_qualified_reference_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "ref".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::Reference(Reference::qualified("User", "alice")),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("@User:alice"));
}

// =============================================================================
// HEDL to CSV - Expression Tests
// =============================================================================

#[test]
fn test_expression_to_csv() {
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
}

#[test]
fn test_complex_expression_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "expr".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            expr_value("divide(multiply(add(a, b), subtract(c, d)), e)"),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("$(divide(multiply(add(a, b), subtract(c, d)), e))"));
}

// =============================================================================
// HEDL to CSV - Tensor Tests
// =============================================================================

#[test]
fn test_1d_tensor_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "tensor".to_string()]);
    let tensor = Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Tensor(tensor)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("[1,2,3]"));
}

#[test]
fn test_2d_tensor_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "tensor".to_string()]);
    // Create a 2x2 tensor: [[1, 2], [3, 4]]
    let tensor = Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Tensor(tensor)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("[[1,2],[3,4]]"));
}

#[test]
fn test_tensor_with_floats_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "tensor".to_string()]);
    let tensor = Tensor::Array(vec![
        Tensor::Scalar(1.5),
        Tensor::Scalar(2.7),
        Tensor::Scalar(3.25),
    ]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Tensor(tensor)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("[1.5,2.7,3.25]"));
}

#[test]
fn test_tensor_round_trip_1d() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "tensor".to_string()]);
    let tensor = Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::Tensor(tensor.clone()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(&csv, "Item", &["tensor"]).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
    if let Value::Tensor(t) = &list2.rows[0].fields[1] {
        assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0]);
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_tensor_round_trip_2d() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "tensor".to_string()]);
    let tensor = Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::Tensor(tensor.clone()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(&csv, "Item", &["tensor"]).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
    if let Value::Tensor(t) = &list2.rows[0].fields[1] {
        assert_eq!(t.shape(), vec![2, 2]);
        assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0, 4.0]);
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_csv_tensor_inference() {
    let csv_data = "id,tensor\n1,\"[1,2,3]\"\n";
    let doc = from_csv(csv_data, "Item", &["tensor"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    if let Value::Tensor(t) = &list.rows[0].fields[1] {
        assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0]);
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_csv_2d_tensor_inference() {
    let csv_data = "id,tensor\n1,\"[[1,2],[3,4]]\"\n";
    let doc = from_csv(csv_data, "Item", &["tensor"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    if let Value::Tensor(t) = &list.rows[0].fields[1] {
        assert_eq!(t.shape(), vec![2, 2]);
        assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0, 4.0]);
    } else {
        panic!("Expected tensor");
    }
}

// =============================================================================
// HEDL to CSV - Special Float Values
// =============================================================================

#[test]
fn test_nan_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Float(f64::NAN)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("NaN"));
}

#[test]
fn test_infinity_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Float(f64::INFINITY)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("Infinity"));
}

#[test]
fn test_neg_infinity_to_csv() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::Float(f64::NEG_INFINITY),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    assert!(csv.contains("-Infinity"));
}

// =============================================================================
// CSV to HEDL - Scalar Type Tests
// =============================================================================

#[test]
fn test_csv_null_inference() {
    let csv_data = "id,value\n1,\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Null);
}

#[test]
fn test_csv_tilde_null_inference() {
    let csv_data = "id,value\n1,~\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Null);
}

#[test]
fn test_csv_bool_true_inference() {
    let csv_data = "id,value\n1,true\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Bool(true));
}

#[test]
fn test_csv_bool_false_inference() {
    let csv_data = "id,value\n1,false\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Bool(false));
}

#[test]
fn test_csv_int_inference() {
    let csv_data = "id,value\n1,42\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Int(42));
}

#[test]
fn test_csv_negative_int_inference() {
    let csv_data = "id,value\n1,-999\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Int(-999));
}

#[test]
fn test_csv_float_inference() {
    let csv_data = "id,value\n1,3.25\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Float(3.25));
}

#[test]
fn test_csv_string_inference() {
    let csv_data = "id,value\n1,hello\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::String("hello".to_string()));
}

#[test]
fn test_csv_nan_inference() {
    let csv_data = "id,value\n1,NaN\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    if let Value::Float(f) = list.rows[0].fields[1] {
        assert!(f.is_nan());
    } else {
        panic!("Expected float NaN");
    }
}

#[test]
fn test_csv_infinity_inference() {
    let csv_data = "id,value\n1,Infinity\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Float(f64::INFINITY));
}

#[test]
fn test_csv_neg_infinity_inference() {
    let csv_data = "id,value\n1,-Infinity\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Float(f64::NEG_INFINITY));
}

// =============================================================================
// CSV to HEDL - Reference Tests
// =============================================================================

#[test]
fn test_csv_local_reference_inference() {
    let csv_data = "id,ref\n1,@target\n";
    let doc = from_csv(csv_data, "Item", &["ref"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    if let Value::Reference(r) = &list.rows[0].fields[1] {
        assert_eq!(r.id, "target");
        assert_eq!(r.type_name, None);
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_csv_qualified_reference_inference() {
    let csv_data = "id,ref\n1,@User:alice\n";
    let doc = from_csv(csv_data, "Item", &["ref"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();

    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    if let Value::Reference(r) = &list.rows[0].fields[1] {
        assert_eq!(r.id, "alice");
        assert_eq!(r.type_name, Some("User".to_string()));
    } else {
        panic!("Expected reference");
    }
}

// =============================================================================
// CSV to HEDL - Expression Tests
// =============================================================================

#[test]
fn test_csv_expression_inference() {
    let csv_data = "id,expr\n1,\"$(add(a, b))\"\n";
    let doc = from_csv(csv_data, "Item", &["expr"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], expr_value("add(a, b)"));
}

#[test]
fn test_csv_nested_expression_inference() {
    // Note: expression with comma must be quoted in CSV
    let csv_data = "id,expr\n1,\"$(func(x, y))\"\n";
    let doc = from_csv(csv_data, "Item", &["expr"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], expr_value("func(x, y)"));
}

// =============================================================================
// Config Tests - Delimiter
// =============================================================================

#[test]
fn test_semicolon_delimiter() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec!["id".to_string(), "a".to_string(), "b".to_string()],
    );
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::String("x".to_string()),
            Value::String("y".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToCsvConfig {
        delimiter: b';',
        ..Default::default()
    };
    let csv = to_csv_with_config(&doc, config).unwrap();
    assert!(csv.contains(';'));
    assert!(!csv.contains(','));

    let config = FromCsvConfig {
        delimiter: b';',
        ..Default::default()
    };
    let doc2 = from_csv_with_config(&csv, "Item", &["a", "b"], config).unwrap();
    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list2.rows[0].fields[1], Value::String("x".to_string()));
}

#[test]
fn test_pipe_delimiter() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec!["id".to_string(), "a".to_string(), "b".to_string()],
    );
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::String("x".to_string()),
            Value::String("y".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToCsvConfig {
        delimiter: b'|',
        ..Default::default()
    };
    let csv = to_csv_with_config(&doc, config).unwrap();
    assert!(csv.contains('|'));

    let config = FromCsvConfig {
        delimiter: b'|',
        ..Default::default()
    };
    let doc2 = from_csv_with_config(&csv, "Item", &["a", "b"], config).unwrap();
    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list2.rows[0].fields[1], Value::String("x".to_string()));
}

// =============================================================================
// Config Tests - Headers
// =============================================================================

#[test]
fn test_no_headers_output() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Int(42)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToCsvConfig {
        include_headers: false,
        ..Default::default()
    };
    let csv = to_csv_with_config(&doc, config).unwrap();
    assert!(!csv.contains("id,value"));
    assert!(csv.starts_with("1,42"));
}

#[test]
fn test_no_headers_input() {
    let csv_data = "1,hello\n2,world\n";
    let config = FromCsvConfig {
        has_headers: false,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv_data, "Item", &["value"], config).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows.len(), 2);
    assert_eq!(list.rows[0].id, "1");
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::String("hello".to_string()));
}

// =============================================================================
// Config Tests - Whitespace Trimming
// =============================================================================

#[test]
fn test_trim_whitespace() {
    let csv_data = "id,value\n1,  spaced  \n";
    let config = FromCsvConfig {
        trim: true,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv_data, "Item", &["value"], config).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::String("spaced".to_string()));
}

#[test]
fn test_no_trim_whitespace() {
    let csv_data = "id,value\n1,  spaced  \n";
    let config = FromCsvConfig {
        trim: false,
        ..Default::default()
    };
    let doc = from_csv_with_config(csv_data, "Item", &["value"], config).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("  spaced  ".to_string())
    );
}

// =============================================================================
// Round-trip Tests
// =============================================================================

#[test]
fn test_round_trip_all_scalar_types() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec![
            "id".to_string(),
            "null_val".to_string(),
            "bool_val".to_string(),
            "int_val".to_string(),
            "float_val".to_string(),
            "string_val".to_string(),
        ],
    );
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::Null,
            Value::Bool(true),
            Value::Int(42),
            Value::Float(3.25),
            Value::String("hello".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(
        &csv,
        "Item",
        &["null_val", "bool_val", "int_val", "float_val", "string_val"],
    )
    .unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field (parsed as int by CSV)
    assert_eq!(list2.rows[0].fields[1], Value::Null);
    assert_eq!(list2.rows[0].fields[2], Value::Bool(true));
    assert_eq!(list2.rows[0].fields[3], Value::Int(42));
    assert_eq!(list2.rows[0].fields[4], Value::Float(3.25));
    assert_eq!(list2.rows[0].fields[5], Value::String("hello".to_string()));
}

#[test]
fn test_round_trip_references() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec![
            "id".to_string(),
            "local_ref".to_string(),
            "qualified_ref".to_string(),
        ],
    );
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::Reference(Reference::local("target")),
            Value::Reference(Reference::qualified("User", "alice")),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(&csv, "Item", &["local_ref", "qualified_ref"]).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();

    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field (parsed as int by CSV)
    if let Value::Reference(r) = &list2.rows[0].fields[1] {
        assert_eq!(r.id, "target");
        assert_eq!(r.type_name, None);
    } else {
        panic!("Expected local reference");
    }

    if let Value::Reference(r) = &list2.rows[0].fields[2] {
        assert_eq!(r.id, "alice");
        assert_eq!(r.type_name, Some("User".to_string()));
    } else {
        panic!("Expected qualified reference");
    }
}

#[test]
fn test_round_trip_expression() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "expr".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            expr_value("add(multiply(a, b), c)"),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(&csv, "Item", &["expr"]).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list2.rows[0].fields[0],
        Value::Int(1) // ID field
    );
    assert_eq!(
        list2.rows[0].fields[1],
        expr_value("add(multiply(a, b), c)")
    );
}

#[test]
fn test_round_trip_special_floats() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Item",
        vec!["id".to_string(), "inf".to_string(), "neg_inf".to_string()],
    );
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::Float(f64::INFINITY),
            Value::Float(f64::NEG_INFINITY),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(&csv, "Item", &["inf", "neg_inf"]).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field (parsed as int by CSV)
    assert_eq!(list2.rows[0].fields[1], Value::Float(f64::INFINITY));
    assert_eq!(list2.rows[0].fields[2], Value::Float(f64::NEG_INFINITY));
}

#[test]
fn test_round_trip_multiple_rows() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Person",
        vec!["id".to_string(), "name".to_string(), "age".to_string()],
    );
    list.add_row(Node::new(
        "Person",
        "p1",
        vec![
            Value::String("p1".to_string()),
            Value::String("Alice".to_string()),
            Value::Int(30),
        ],
    ));
    list.add_row(Node::new(
        "Person",
        "p2",
        vec![
            Value::String("p2".to_string()),
            Value::String("Bob".to_string()),
            Value::Int(25),
        ],
    ));
    list.add_row(Node::new(
        "Person",
        "p3",
        vec![
            Value::String("p3".to_string()),
            Value::String("Charlie".to_string()),
            Value::Int(35),
        ],
    ));
    doc.root.insert("persons".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(&csv, "Person", &["name", "age"]).unwrap();

    let list2 = doc2.get("persons").unwrap().as_list().unwrap();
    assert_eq!(list2.rows.len(), 3);
    assert_eq!(list2.rows[0].id, "p1");
    assert_eq!(list2.rows[1].id, "p2");
    assert_eq!(list2.rows[2].id, "p3");
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_error_no_matrix_list() {
    let doc = Document::new((1, 0));
    let result = to_csv(&doc);
    assert!(result.is_err());
}

#[test]
fn test_error_empty_id() {
    let csv_data = "id,value\n,hello\n";
    let result = from_csv(csv_data, "Item", &["value"]);
    assert!(result.is_err());
}

#[test]
fn test_error_invalid_reference_empty() {
    let csv_data = "id,ref\n1,@\n";
    let result = from_csv(csv_data, "Item", &["ref"]);
    assert!(result.is_err());
}

#[test]
fn test_error_invalid_qualified_reference() {
    let csv_data = "id,ref\n1,@:\n";
    let result = from_csv(csv_data, "Item", &["ref"]);
    assert!(result.is_err());
}

// =============================================================================
// Edge Case Tests
// =============================================================================

#[test]
fn test_string_with_comma() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::String("hello, world".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    let doc2 = from_csv(&csv, "Item", &["value"]).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list2.rows[0].fields[0],
        Value::Int(1) // ID field
    );
    assert_eq!(
        list2.rows[0].fields[1],
        Value::String("hello, world".to_string())
    );
}

#[test]
fn test_string_with_quotes() {
    let csv_data = r#"id,value
1,"She said ""Hello"""
"#;
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[0],
        Value::Int(1) // ID field
    );
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("She said \"Hello\"".to_string())
    );
}

#[test]
fn test_string_with_newline() {
    let csv_data = "id,value\n1,\"line1\nline2\"\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[0],
        Value::Int(1) // ID field
    );
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("line1\nline2".to_string())
    );
}

#[test]
fn test_unicode_characters() {
    let csv_data = "id,value\n1,Hello 世界 🌍\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(
        list.rows[0].fields[0],
        Value::Int(1) // ID field
    );
    assert_eq!(
        list.rows[0].fields[1],
        Value::String("Hello 世界 🌍".to_string())
    );
}

#[test]
fn test_many_columns() {
    let mut doc = Document::new((1, 0));
    // Create schema with id + 20 columns for SPEC compliance
    let mut schema: Vec<String> = vec!["id".to_string()];
    schema.extend((0..20).map(|i| format!("col{}", i)));
    let mut list = MatrixList::new("Item", schema.clone());

    // Create values: id=1, col0=0, col1=1, ..., col19=19
    let mut values: Vec<Value> = vec![Value::Int(1)];
    values.extend((0..20).map(Value::Int));
    list.add_row(Node::new("Item", "1", values));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    // from_csv adds id, so we pass only col0-col19
    let schema_refs: Vec<String> = (0..20).map(|i| format!("col{}", i)).collect();
    let schema_refs_str: Vec<&str> = schema_refs.iter().map(|s| s.as_str()).collect();
    let doc2 = from_csv(&csv, "Item", &schema_refs_str).unwrap();

    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(list2.rows[0].fields.len(), 21); // id + 20 columns (SPEC-compliant)
    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // id field
    assert_eq!(list2.rows[0].fields[1], Value::Int(0)); // col0
    assert_eq!(list2.rows[0].fields[20], Value::Int(19)); // col19
}

#[test]
fn test_many_rows() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    for i in 0..100 {
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
    assert_eq!(list2.rows.len(), 100);
}

#[test]
fn test_numeric_string_preserved() {
    // A numeric-looking string that was quoted in the original should remain a string
    // This depends on how the original HEDL doc was created
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![
            Value::String("1".to_string()),
            Value::String("12345".to_string()),
        ],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let csv = to_csv(&doc).unwrap();
    // Note: CSV round-trip will infer it as an integer since CSV loses type info
    let doc2 = from_csv(&csv, "Item", &["value"]).unwrap();
    let list2 = doc2.get("items").unwrap().as_list().unwrap();
    // This becomes Int due to type inference
    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list2.rows[0].fields[1], Value::Int(12345));
}

#[test]
fn test_integer_max_values() {
    let csv_data = format!("id,value\n1,{}\n2,{}\n", i64::MAX, i64::MIN);
    let doc = from_csv(&csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Int(i64::MAX));
    assert_eq!(list.rows[1].fields[0], Value::Int(2)); // ID field
    assert_eq!(list.rows[1].fields[1], Value::Int(i64::MIN));
}

#[test]
fn test_float_precision() {
    let csv_data = "id,value\n1,1.7976931348623157e308\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Float(f64::MAX));
}

#[test]
fn test_scientific_notation() {
    let csv_data = "id,value\n1,1.5e10\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Float(1.5e10));
}

#[test]
fn test_zero_values() {
    let csv_data = "id,int,float\n1,0,0.0\n";
    let doc = from_csv(csv_data, "Item", &["int", "float"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Int(0));
    assert_eq!(list.rows[0].fields[2], Value::Float(0.0));
}

// =============================================================================
// Writer API Tests
// =============================================================================

#[test]
fn test_to_csv_writer() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Int(42)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let mut buffer = Vec::new();
    hedl_csv::to_csv_writer(&doc, &mut buffer).unwrap();

    let csv = String::from_utf8(buffer).unwrap();
    assert!(csv.contains("id,value"));
    assert!(csv.contains("1,42"));
}

#[test]
fn test_to_csv_writer_with_config() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "1",
        vec![Value::String("1".to_string()), Value::Int(42)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    let mut buffer = Vec::new();
    let config = ToCsvConfig {
        delimiter: b'\t',
        include_headers: false,
        ..Default::default()
    };
    hedl_csv::to_csv_writer_with_config(&doc, &mut buffer, config).unwrap();

    let csv = String::from_utf8(buffer).unwrap();
    assert!(csv.contains("1\t42"));
    assert!(!csv.contains("id"));
}

// =============================================================================
// Reader API Tests
// =============================================================================

#[test]
fn test_from_csv_reader() {
    let csv_data = b"id,value\n1,42\n";
    let doc = hedl_csv::from_csv_reader(&csv_data[..], "Item", &["value"]).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Int(42));
}

#[test]
fn test_from_csv_reader_with_config() {
    let csv_data = b"1\t42\n";
    let config = FromCsvConfig {
        delimiter: b'\t',
        has_headers: false,
        ..Default::default()
    };
    let doc =
        hedl_csv::from_csv_reader_with_config(&csv_data[..], "Item", &["value"], config).unwrap();
    let list = doc.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list.rows[0].fields[1], Value::Int(42));
}

// =============================================================================
// Schema Registration Tests
// =============================================================================

#[test]
fn test_schema_registered() {
    let csv_data = "id,name,age\n1,Alice,30\n";
    let doc = from_csv(csv_data, "Person", &["name", "age"]).unwrap();

    let schema = doc.get_schema("Person").unwrap();
    assert_eq!(schema, &["id", "name", "age"]);
}

#[test]
fn test_list_key_generated() {
    let csv_data = "id,value\n1,hello\n";
    let doc = from_csv(csv_data, "Item", &["value"]).unwrap();

    // Should create "items" (lowercase + 's')
    assert!(doc.get("items").is_some());
}

#[test]
fn test_type_name_preserved() {
    let csv_data = "id,value\n1,hello\n";
    let doc = from_csv(csv_data, "CustomType", &["value"]).unwrap();

    let list = doc.get("customtypes").unwrap().as_list().unwrap();
    assert_eq!(list.type_name, "CustomType");
}

// =============================================================================
// Shared Fixture Tests - Testing CSV with hedl-test fixtures
// =============================================================================

/// Test user_list fixture roundtrip through CSV.
///
/// The user_list fixture contains a MatrixList with User nodes.
/// CSV should preserve all fields: id, name, email.
/// Note: The user_list fixture schema is [id, name, email], but CSV export
/// automatically includes id, so we only need to specify the remaining fields.
#[test]
fn test_user_list_csv_roundtrip() {
    let doc = fixtures::user_list();

    // Export to CSV
    let csv = to_csv(&doc).unwrap();

    // Verify CSV contains expected data
    assert!(csv.contains("id,name,email")); // SPEC-compliant: schema is [id, name, email]
    assert!(csv.contains("alice"));
    assert!(csv.contains("Alice Smith"));
    assert!(csv.contains("alice@example.com"));
    assert!(csv.contains("bob"));
    assert!(csv.contains("Bob Jones"));
    assert!(csv.contains("charlie"));

    // Import back from CSV - from_csv adds id to schema, so we only pass remaining fields
    let doc2 = from_csv(&csv, "User", &["name", "email"]).unwrap();

    // Verify structure
    let list = doc2.get("users").unwrap().as_list().unwrap();
    assert_eq!(list.type_name, "User");
    assert_eq!(list.rows.len(), 3);

    // Verify first user - fields are [id, name, email] (SPEC-compliant)
    let alice = &list.rows[0];
    assert_eq!(alice.id, "alice");
    assert_eq!(alice.fields[0], Value::String("alice".to_string())); // id field
    assert_eq!(alice.fields[1], Value::String("Alice Smith".to_string())); // name field
    assert_eq!(
        alice.fields[2],
        Value::String("alice@example.com".to_string())
    ); // email field

    // Verify second user
    let bob = &list.rows[1];
    assert_eq!(bob.id, "bob");
    assert_eq!(bob.fields[0], Value::String("bob".to_string()));
    assert_eq!(bob.fields[1], Value::String("Bob Jones".to_string()));
    assert_eq!(bob.fields[2], Value::String("bob@example.com".to_string()));

    // Verify third user
    let charlie = &list.rows[2];
    assert_eq!(charlie.id, "charlie");
    assert_eq!(charlie.fields[0], Value::String("charlie".to_string()));
    assert_eq!(
        charlie.fields[1],
        Value::String("Charlie Brown".to_string())
    );
    assert_eq!(
        charlie.fields[2],
        Value::String("charlie@example.com".to_string())
    );
}

/// Test mixed_type_list fixture roundtrip through CSV.
///
/// This fixture contains various value types: string, int, float, bool, null.
/// CSV should handle type inference correctly on import.
/// Note: The fixture schema is [id, name, count, price, active, notes]
#[test]
fn test_mixed_types_csv_roundtrip() {
    let doc = fixtures::mixed_type_list();

    // Export to CSV
    let csv = to_csv(&doc).unwrap();

    // Verify CSV contains headers and data (SPEC-compliant)
    assert!(csv.contains("id,name,count,price,active,notes"));
    assert!(csv.contains("Widget"));
    assert!(csv.contains("100"));
    assert!(csv.contains("9.99"));
    assert!(csv.contains("true"));
    assert!(csv.contains("Gadget"));
    assert!(csv.contains("50"));
    assert!(csv.contains("19.99"));
    assert!(csv.contains("false"));

    // Import back from CSV - from_csv adds id, so we pass remaining fields
    let doc2 = from_csv(&csv, "Item", &["name", "count", "price", "active", "notes"]).unwrap();

    // Verify structure
    let list = doc2.get("items").unwrap().as_list().unwrap();
    assert_eq!(list.type_name, "Item");
    assert_eq!(list.rows.len(), 2);

    // Verify first item - fields are [id, name, count, price, active, notes] (SPEC-compliant)
    let item1 = &list.rows[0];
    assert_eq!(item1.id, "item1");
    assert_eq!(item1.fields[0], Value::String("item1".to_string())); // id
    assert_eq!(item1.fields[1], Value::String("Widget".to_string())); // name
    assert_eq!(item1.fields[2], Value::Int(100)); // count
    assert_eq!(item1.fields[3], Value::Float(9.99)); // price
    assert_eq!(item1.fields[4], Value::Bool(true)); // active
    assert_eq!(item1.fields[5], Value::String("Best seller".to_string())); // notes

    // Verify second item - including null value
    let item2 = &list.rows[1];
    assert_eq!(item2.id, "item2");
    assert_eq!(item2.fields[0], Value::String("item2".to_string()));
    assert_eq!(item2.fields[1], Value::String("Gadget".to_string()));
    assert_eq!(item2.fields[2], Value::Int(50));
    assert_eq!(item2.fields[3], Value::Float(19.99));
    assert_eq!(item2.fields[4], Value::Bool(false));
    assert_eq!(item2.fields[5], Value::Null);
}

/// Test with_references fixture CSV handling.
///
/// This fixture contains references between User and Post lists.
/// CSV should serialize references as @User:id syntax and parse them back correctly.
/// Note: Post schema is [id, title, author]
#[test]
fn test_references_csv_roundtrip() {
    let doc = fixtures::with_references();

    // CSV can only export one list at a time, so we need to extract the posts list
    // which contains references to users
    let posts_list = doc.get("posts").unwrap().as_list().unwrap();

    // Create a document with just the posts for CSV export
    let mut posts_doc = Document::new((1, 0));
    posts_doc
        .root
        .insert("posts".to_string(), Item::List(posts_list.clone()));

    // Export posts to CSV
    let csv = to_csv(&posts_doc).unwrap();

    // Verify CSV contains reference syntax (SPEC-compliant)
    assert!(csv.contains("id,title,author"));
    assert!(csv.contains("@User:alice"));
    assert!(csv.contains("@User:bob"));
    assert!(csv.contains("Hello World"));
    assert!(csv.contains("Rust is great"));

    // Import back from CSV - from_csv adds id, so we pass remaining fields
    let doc2 = from_csv(&csv, "Post", &["title", "author"]).unwrap();

    // Verify structure
    let list = doc2.get("posts").unwrap().as_list().unwrap();
    assert_eq!(list.type_name, "Post");
    assert_eq!(list.rows.len(), 3);

    // Verify first post with reference - fields are [id, title, author] (SPEC-compliant)
    let post1 = &list.rows[0];
    assert_eq!(post1.id, "post1");
    assert_eq!(post1.fields[0], Value::String("post1".to_string())); // id
    assert_eq!(post1.fields[1], Value::String("Hello World".to_string())); // title

    if let Value::Reference(ref r) = post1.fields[2] {
        // author
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "alice");
    } else {
        panic!("Expected reference for author field");
    }

    // Verify second post with different author
    let post2 = &list.rows[1];
    assert_eq!(post2.id, "post2");
    assert_eq!(post2.fields[0], Value::String("post2".to_string()));
    assert_eq!(post2.fields[1], Value::String("Rust is great".to_string()));

    if let Value::Reference(ref r) = post2.fields[2] {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "bob");
    } else {
        panic!("Expected reference for author field");
    }

    // Verify third post
    let post3 = &list.rows[2];
    assert_eq!(post3.id, "post3");
    assert_eq!(post3.fields[0], Value::String("post3".to_string()));

    if let Value::Reference(ref r) = post3.fields[2] {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "alice");
    } else {
        panic!("Expected reference for author field");
    }
}

/// Test comprehensive fixture - extract and test individual lists.
///
/// The comprehensive fixture has multiple lists. CSV can only handle one list
/// at a time, so we test each list individually.
/// Note: User schema is [id, name, email, age]
#[test]
fn test_comprehensive_users_csv() {
    let doc = fixtures::comprehensive();

    // Extract users list (note: comprehensive has NEST which CSV doesn't support)
    let users_list = doc.get("users").unwrap().as_list().unwrap();

    // Create a document with just the users for CSV export
    let mut users_doc = Document::new((1, 0));
    users_doc
        .root
        .insert("users".to_string(), Item::List(users_list.clone()));

    // Export to CSV
    let csv = to_csv(&users_doc).unwrap();

    // Verify CSV contains expected data (SPEC-compliant)
    assert!(csv.contains("id,name,email,age"));
    assert!(csv.contains("alice"));
    assert!(csv.contains("Alice Smith"));
    assert!(csv.contains("30"));
    assert!(csv.contains("bob"));
    assert!(csv.contains("Bob Jones"));
    assert!(csv.contains("25"));

    // Import back from CSV - from_csv adds id, so we pass remaining fields
    let doc2 = from_csv(&csv, "User", &["name", "email", "age"]).unwrap();

    // Verify structure
    let list = doc2.get("users").unwrap().as_list().unwrap();
    assert_eq!(list.type_name, "User");
    assert_eq!(list.rows.len(), 2);

    // Verify data types preserved - fields are [id, name, email, age] (SPEC-compliant)
    let alice = &list.rows[0];
    assert_eq!(alice.fields[3], Value::Int(30)); // age is 4th field

    let bob = &list.rows[1];
    assert_eq!(bob.fields[3], Value::Int(25)); // age is 4th field
}

/// Test comprehensive fixture - comments with multiple references.
/// Note: Comment schema is [id, text, author, post]
#[test]
fn test_comprehensive_comments_csv() {
    let doc = fixtures::comprehensive();

    // Extract comments list which has references to both User and Post
    let comments_list = doc.get("comments").unwrap().as_list().unwrap();

    // Create a document with just the comments for CSV export
    let mut comments_doc = Document::new((1, 0));
    comments_doc
        .root
        .insert("comments".to_string(), Item::List(comments_list.clone()));

    // Export to CSV
    let csv = to_csv(&comments_doc).unwrap();

    // Verify CSV contains reference syntax for both types (SPEC-compliant)
    assert!(csv.contains("id,text,author,post"));
    assert!(csv.contains("@User:bob"));
    assert!(csv.contains("@Post:p1"));
    assert!(csv.contains("Great article!"));

    // Import back from CSV - from_csv adds id, so we pass remaining fields
    let doc2 = from_csv(&csv, "Comment", &["text", "author", "post"]).unwrap();

    // Verify structure
    let list = doc2.get("comments").unwrap().as_list().unwrap();
    assert_eq!(list.type_name, "Comment");
    assert_eq!(list.rows.len(), 1);

    // Verify comment with multiple references - fields are [id, text, author, post] (SPEC-compliant)
    let comment = &list.rows[0];
    assert_eq!(comment.id, "c1");
    assert_eq!(comment.fields[0], Value::String("c1".to_string())); // id
    assert_eq!(
        comment.fields[1],
        Value::String("Great article!".to_string())
    ); // text

    // Verify author reference (3rd field)
    if let Value::Reference(ref r) = comment.fields[2] {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "bob");
    } else {
        panic!("Expected reference for author field");
    }

    // Verify post reference (4th field)
    if let Value::Reference(ref r) = comment.fields[3] {
        assert_eq!(r.type_name, Some("Post".to_string()));
        assert_eq!(r.id, "p1");
    } else {
        panic!("Expected reference for post field");
    }
}

/// Test comprehensive fixture - tags list (simple, no references).
/// Note: Tag schema is [id, name, color]
#[test]
fn test_comprehensive_tags_csv() {
    let doc = fixtures::comprehensive();

    // Extract tags list - simple string fields only
    let tags_list = doc.get("tags").unwrap().as_list().unwrap();

    // Create a document with just the tags for CSV export
    let mut tags_doc = Document::new((1, 0));
    tags_doc
        .root
        .insert("tags".to_string(), Item::List(tags_list.clone()));

    // Export to CSV
    let csv = to_csv(&tags_doc).unwrap();

    // Verify CSV structure (SPEC-compliant)
    assert!(csv.contains("id,name,color"));
    assert!(csv.contains("rust"));
    assert!(csv.contains("Rust"));
    assert!(csv.contains("#FF4500"));
    assert!(csv.contains("hedl"));
    assert!(csv.contains("HEDL"));
    assert!(csv.contains("#00BFFF"));

    // Import back from CSV - from_csv adds id, so we pass remaining fields
    let doc2 = from_csv(&csv, "Tag", &["name", "color"]).unwrap();

    // Verify structure
    let list = doc2.get("tags").unwrap().as_list().unwrap();
    assert_eq!(list.type_name, "Tag");
    assert_eq!(list.rows.len(), 2);

    // Verify tags - fields are [id, name, color] (SPEC-compliant)
    let rust_tag = &list.rows[0];
    assert_eq!(rust_tag.id, "rust");
    assert_eq!(rust_tag.fields[0], Value::String("rust".to_string())); // id
    assert_eq!(rust_tag.fields[1], Value::String("Rust".to_string())); // name
    assert_eq!(rust_tag.fields[2], Value::String("#FF4500".to_string())); // color

    let hedl_tag = &list.rows[1];
    assert_eq!(hedl_tag.id, "hedl");
    assert_eq!(hedl_tag.fields[0], Value::String("hedl".to_string()));
    assert_eq!(hedl_tag.fields[1], Value::String("HEDL".to_string()));
    assert_eq!(hedl_tag.fields[2], Value::String("#00BFFF".to_string()));
}

/// Test that CSV properly handles edge cases from fixtures.
///
/// CSV has limitations but should gracefully handle what it can.
#[test]
fn test_edge_cases_large_numbers() {
    let doc = fixtures::edge_cases();

    // Extract specific edge case values and create a MatrixList
    let mut list = MatrixList::new(
        "EdgeCase",
        vec![
            "id".to_string(),
            "large_int".to_string(),
            "small_int".to_string(),
        ],
    );

    if let Some(Item::Scalar(Value::Int(large))) = doc.root.get("large_int") {
        if let Some(Item::Scalar(Value::Int(small))) = doc.root.get("small_int") {
            list.add_row(Node::new(
                "EdgeCase",
                "1",
                vec![
                    Value::String("1".to_string()),
                    Value::Int(*large),
                    Value::Int(*small),
                ],
            ));
        }
    }

    let mut test_doc = Document::new((1, 0));
    test_doc
        .root
        .insert("edgecases".to_string(), Item::List(list));

    // Export to CSV
    let csv = to_csv(&test_doc).unwrap();

    // Verify extreme values are present
    assert!(csv.contains(&i64::MAX.to_string()));
    assert!(csv.contains(&i64::MIN.to_string()));

    // Import back
    let doc2 = from_csv(&csv, "EdgeCase", &["large_int", "small_int"]).unwrap();
    let list2 = doc2.get("edgecases").unwrap().as_list().unwrap();

    // Verify values preserved
    assert_eq!(list2.rows[0].fields[0], Value::Int(1)); // ID field
    assert_eq!(list2.rows[0].fields[1], Value::Int(i64::MAX));
    assert_eq!(list2.rows[0].fields[2], Value::Int(i64::MIN));
}

/// Test special strings fixture through CSV.
///
/// CSV should properly quote and escape strings with special characters.
#[test]
fn test_special_strings_csv() {
    let doc = fixtures::special_strings();

    // Create a MatrixList with various special strings
    let mut list = MatrixList::new("SpecialString", vec!["id".to_string(), "value".to_string()]);

    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("with_quotes") {
        list.add_row(Node::new(
            "SpecialString",
            "quotes",
            vec![
                Value::String("quotes".to_string()),
                Value::String(s.clone()),
            ],
        ));
    }

    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("with_newline") {
        list.add_row(Node::new(
            "SpecialString",
            "newline",
            vec![
                Value::String("newline".to_string()),
                Value::String(s.clone()),
            ],
        ));
    }

    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("with_unicode") {
        list.add_row(Node::new(
            "SpecialString",
            "unicode",
            vec![
                Value::String("unicode".to_string()),
                Value::String(s.clone()),
            ],
        ));
    }

    let mut test_doc = Document::new((1, 0));
    test_doc
        .root
        .insert("specialstrings".to_string(), Item::List(list));

    // Export to CSV
    let csv = to_csv(&test_doc).unwrap();

    // Import back
    let doc2 = from_csv(&csv, "SpecialString", &["value"]).unwrap();
    let list2 = doc2.get("specialstrings").unwrap().as_list().unwrap();

    // Verify special characters preserved
    assert_eq!(list2.rows.len(), 3);

    // Check quotes preserved
    if let Value::String(s) = &list2.rows[0].fields[1] {
        assert!(s.contains("\"hello\""));
    } else {
        panic!("Expected string value");
    }

    // Check newline preserved
    if let Value::String(s) = &list2.rows[1].fields[1] {
        assert!(s.contains('\n'));
    } else {
        panic!("Expected string value");
    }

    // Check unicode preserved
    if let Value::String(s) = &list2.rows[2].fields[1] {
        assert!(s.contains("日本語"));
        assert!(s.contains("🎉"));
    } else {
        panic!("Expected string value");
    }
}
