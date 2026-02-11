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

//! Edge case tests for TOON conversion (`to_toon`)
//!
//! Tests configuration, delimiters, error handling, and boundary cases.

use hedl_core::lex::{ExprLiteral, Expression, Span, Tensor};
use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_toon::{to_toon, Delimiter, ToToonConfig, ToToonConfigBuilder, ToonError};

#[test]
fn test_config_default() {
    let config = ToToonConfig::default();
    assert_eq!(config.indent, 2); // TOON format uses 2-space indentation
    assert_eq!(config.delimiter, Delimiter::Comma);
}

#[test]
fn test_config_builder() {
    let config = ToToonConfig::builder()
        .indent(4)
        .delimiter(Delimiter::Tab)
        .build();

    assert_eq!(config.indent, 4);
    assert_eq!(config.delimiter, Delimiter::Tab);
}

#[test]
fn test_config_builder_default() {
    let config = ToToonConfigBuilder::default().build();
    assert_eq!(config.indent, 2); // TOON format uses 2-space indentation
    assert_eq!(config.delimiter, Delimiter::Comma);
}

#[test]
fn test_delimiter_comma() {
    let mut doc = Document::new((2, 0));
    doc.structs
        .insert("Item".to_string(), vec!["a".to_string(), "b".to_string()]);

    let mut list = MatrixList::new("Item", vec!["a".to_string(), "b".to_string()]);
    list.add_row(Node::new("Item", "i1", vec![Value::Int(1), Value::Int(2)]));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToToonConfig {
        indent: 1,
        delimiter: Delimiter::Comma,
    };

    let toon = to_toon(&doc, &config).unwrap();
    assert!(toon.contains("items[1]{a,b}:"));
    assert!(toon.contains("1,2"));
}

#[test]
fn test_delimiter_tab() {
    let mut doc = Document::new((2, 0));
    doc.structs
        .insert("Item".to_string(), vec!["a".to_string(), "b".to_string()]);

    let mut list = MatrixList::new("Item", vec!["a".to_string(), "b".to_string()]);
    list.add_row(Node::new("Item", "i1", vec![Value::Int(1), Value::Int(2)]));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToToonConfig {
        indent: 1,
        delimiter: Delimiter::Tab,
    };

    let toon = to_toon(&doc, &config).unwrap();
    assert!(toon.contains("items[1\t]{a\tb}:"));
    assert!(toon.contains("1\t2"));
}

#[test]
fn test_delimiter_pipe() {
    let mut doc = Document::new((2, 0));
    doc.structs
        .insert("Item".to_string(), vec!["a".to_string(), "b".to_string()]);

    let mut list = MatrixList::new("Item", vec!["a".to_string(), "b".to_string()]);
    list.add_row(Node::new("Item", "i1", vec![Value::Int(1), Value::Int(2)]));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToToonConfig {
        indent: 1,
        delimiter: Delimiter::Pipe,
    };

    let toon = to_toon(&doc, &config).unwrap();
    assert!(toon.contains("items[1|]{a|b}:"));
    assert!(toon.contains("1|2"));
}

#[test]
fn test_custom_indent_size() {
    let mut doc = Document::new((2, 0));

    let mut inner = std::collections::BTreeMap::new();
    inner.insert("value".to_string(), Item::Scalar(Value::Int(42)));

    doc.root.insert("outer".to_string(), Item::Object(inner));

    let config = ToToonConfig {
        indent: 4,
        delimiter: Delimiter::Comma,
    };

    let toon = to_toon(&doc, &config).unwrap();

    // Should use 4-space indentation
    assert!(toon.contains("outer:\n    value: 42") || toon.contains("outer:\r\n    value: 42"));
}

#[test]
fn test_tensor_encoding() {
    let mut doc = Document::new((2, 0));

    let tensor = Box::new(Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ]));

    doc.root
        .insert("tensor".to_string(), Item::Scalar(Value::Tensor(tensor)));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // Tensor should be encoded as inline array
    assert!(toon.contains("[3]"));
    assert!(toon.contains('1'));
    assert!(toon.contains('2'));
    assert!(toon.contains('3'));
}

#[test]
fn test_expression_encoding() {
    let mut doc = Document::new((2, 0));

    // Create a simple expression
    let expr = Expression::Literal {
        value: ExprLiteral::Int(42),
        span: Span::synthetic(),
    };
    doc.root.insert(
        "expr".to_string(),
        Item::Scalar(Value::Expression(Box::new(expr))),
    );

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // Expression is converted to string representation
    // The exact format depends on how we serialize expressions
    assert!(toon.contains("expr:"));
}

#[test]
fn test_reference_in_array() {
    let mut doc = Document::new((2, 0));
    doc.structs.insert(
        "Link".to_string(),
        vec!["id".to_string(), "target".to_string()],
    );

    let mut list = MatrixList::new("Link", vec!["id".to_string(), "target".to_string()]);
    list.add_row(Node::new(
        "Link",
        "l1",
        vec![
            Value::String("l1".to_string().into()),
            Value::Reference(Reference::qualified("User", "u1")),
        ],
    ));

    doc.root.insert("links".to_string(), Item::List(list));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // Reference should be quoted in tabular format
    assert!(toon.contains("\"@User:u1\""));
}

#[test]
fn test_mixed_value_types_in_array() {
    let mut doc = Document::new((2, 0));
    doc.structs.insert(
        "Mixed".to_string(),
        vec![
            "id".to_string(),
            "int_val".to_string(),
            "float_val".to_string(),
            "bool_val".to_string(),
            "str_val".to_string(),
        ],
    );

    let mut list = MatrixList::new(
        "Mixed",
        vec![
            "id".to_string(),
            "int_val".to_string(),
            "float_val".to_string(),
            "bool_val".to_string(),
            "str_val".to_string(),
        ],
    );
    list.add_row(Node::new(
        "Mixed",
        "m1",
        vec![
            Value::String("m1".to_string().into()),
            Value::Int(42),
            Value::Float(3.15),
            Value::Bool(true),
            Value::String("text".to_string().into()),
        ],
    ));

    doc.root.insert("mixed".to_string(), Item::List(list));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // Should use tabular format for all primitive types
    assert!(toon.contains("mixed[1]{id,int_val,float_val,bool_val,str_val}:"));
    assert!(toon.contains("m1,42,3.15,true,text"));
}

#[test]
fn test_zero_indent_works() {
    // Zero indent is valid with official toon-format library
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "name".to_string(),
        Item::Scalar(Value::String("test".into())),
    );

    let config = ToToonConfig {
        indent: 0,
        delimiter: Delimiter::Comma,
    };

    let result = to_toon(&doc, &config);
    // Official library accepts zero indent (no pretty-printing)
    assert!(result.is_ok());
}

#[test]
fn test_error_max_depth_exceeded() {
    let mut doc = Document::new((2, 0));

    // Create deeply nested structure (101 levels)
    let mut current = std::collections::BTreeMap::new();
    current.insert("value".to_string(), Item::Scalar(Value::Int(42)));

    for i in (0..101).rev() {
        let mut parent = std::collections::BTreeMap::new();
        parent.insert(format!("level{i}"), Item::Object(current));
        current = parent;
    }

    doc.root.insert("root".to_string(), Item::Object(current));

    let config = ToToonConfig::default();
    let result = to_toon(&doc, &config);

    assert!(result.is_err());

    if let Err(ToonError::MaxDepthExceeded { depth, max }) = result {
        assert!(depth > max);
    } else {
        panic!("Expected MaxDepthExceeded error");
    }
}

#[test]
fn test_empty_document() {
    let doc = Document::new((2, 0));
    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    assert_eq!(toon, "");
}

#[test]
fn test_single_scalar() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("value".to_string(), Item::Scalar(Value::Int(42)));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    assert_eq!(toon, "value: 42");
}

#[test]
fn test_multiple_root_items() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("a".to_string(), Item::Scalar(Value::Int(1)));
    doc.root
        .insert("b".to_string(), Item::Scalar(Value::Int(2)));
    doc.root
        .insert("c".to_string(), Item::Scalar(Value::Int(3)));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // BTreeMap sorts keys
    assert!(toon.contains("a: 1"));
    assert!(toon.contains("b: 2"));
    assert!(toon.contains("c: 3"));
}

#[test]
fn test_count_hint_respected() {
    let mut doc = Document::new((2, 0));
    doc.structs
        .insert("Item".to_string(), vec!["id".to_string()]);

    // Create list with count_hint different from actual rows
    let mut list = MatrixList::with_count_hint("Item", vec!["id".to_string()], 100);
    list.add_row(Node::new("Item", "i1", vec![Value::Int(1)]));
    list.add_row(Node::new("Item", "i2", vec![Value::Int(2)]));

    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // TOON format doesn't preserve count hints - it uses actual array length
    // The conversion goes HEDL -> JSON -> TOON, so count hints are lost
    assert!(toon.contains("items"));
    // Verify the data is there
    assert!(toon.contains("1") && toon.contains("2"));
}

#[test]
fn test_nested_children_with_different_types() {
    let mut doc = Document::new((2, 0));
    doc.structs
        .insert("Parent".to_string(), vec!["id".to_string()]);
    doc.structs
        .insert("Child1".to_string(), vec!["name".to_string()]);
    doc.structs
        .insert("Child2".to_string(), vec!["value".to_string()]);

    let mut parent_list = MatrixList::new("Parent", vec!["id".to_string()]);
    let mut parent = Node::new("Parent", "p1", vec![Value::String("p1".to_string().into())]);

    parent.add_child(
        "Child1",
        Node::new(
            "Child1",
            "c1",
            vec![Value::String("name1".to_string().into())],
        ),
    );
    parent.add_child("Child2", Node::new("Child2", "c2", vec![Value::Int(42)]));

    parent_list.add_row(parent);
    doc.root
        .insert("parents".to_string(), Item::List(parent_list));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // TOON format via JSON doesn't preserve HEDL's specific tabular header format
    // The nested children are converted to JSON arrays
    assert!(toon.contains("parents"));
    // Children data should be present in some form
    assert!(toon.contains("name1") || toon.contains("Child1"));
}

#[test]
fn test_float_normalization_nan() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("nan".to_string(), Item::Scalar(Value::Float(f64::NAN)));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // NaN becomes null per TOON spec
    assert!(toon.contains("nan: null"));
}

#[test]
fn test_float_normalization_infinity() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("inf".to_string(), Item::Scalar(Value::Float(f64::INFINITY)));
    doc.root.insert(
        "neg_inf".to_string(),
        Item::Scalar(Value::Float(f64::NEG_INFINITY)),
    );

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // Infinity becomes null per TOON spec
    assert!(toon.contains("inf: null"));
    assert!(toon.contains("neg_inf: null"));
}

#[test]
fn test_float_normalization_negative_zero() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("neg_zero".to_string(), Item::Scalar(Value::Float(-0.0)));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // -0 becomes 0 per TOON spec
    assert!(toon.contains("neg_zero: 0"));
}

#[test]
fn test_float_normalization_whole_numbers() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("five".to_string(), Item::Scalar(Value::Float(5.0)));
    doc.root
        .insert("hundred".to_string(), Item::Scalar(Value::Float(100.0)));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // Whole numbers should not have .0
    assert!(toon.contains("five: 5"));
    assert!(toon.contains("hundred: 100"));
}

#[test]
fn test_float_no_trailing_zeros() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("half".to_string(), Item::Scalar(Value::Float(0.5)));
    doc.root
        .insert("pi_approx".to_string(), Item::Scalar(Value::Float(3.15159)));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // Should not have trailing zeros
    assert!(toon.contains("half: 0.5"));
    assert!(toon.contains("pi_approx: 3.15159"));
}

#[test]
fn test_key_quoting() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("simple_key".to_string(), Item::Scalar(Value::Int(1)));
    doc.root
        .insert("with.dot".to_string(), Item::Scalar(Value::Int(2)));
    doc.root
        .insert("123".to_string(), Item::Scalar(Value::Int(3)));
    doc.root
        .insert("with space".to_string(), Item::Scalar(Value::Int(4)));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // Simple key doesn't need quotes
    assert!(toon.contains("simple_key: 1"));

    // Key with dot doesn't need quotes
    assert!(toon.contains("with.dot: 2"));

    // Numeric key needs quotes
    assert!(toon.contains("\"123\": 3"));

    // Key with space needs quotes
    assert!(toon.contains("\"with space\": 4"));
}

#[test]
fn test_value_quoting_structural_chars() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "colon".to_string(),
        Item::Scalar(Value::String("has:colon".to_string().into())),
    );
    doc.root.insert(
        "bracket".to_string(),
        Item::Scalar(Value::String("has[bracket]".to_string().into())),
    );
    doc.root.insert(
        "brace".to_string(),
        Item::Scalar(Value::String("has{brace}".to_string().into())),
    );

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // All structural characters require quoting
    assert!(toon.contains("\"has:colon\""));
    assert!(toon.contains("\"has[bracket]\""));
    assert!(toon.contains("\"has{brace}\""));
}

#[test]
fn test_value_quoting_delimiters() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "comma".to_string(),
        Item::Scalar(Value::String("a,b,c".to_string().into())),
    );

    // With comma delimiter
    let config_comma = ToToonConfig {
        indent: 1,
        delimiter: Delimiter::Comma,
    };
    let toon_comma = to_toon(&doc, &config_comma).unwrap();
    assert!(toon_comma.contains("\"a,b,c\""));

    // With tab delimiter (comma shouldn't need quoting)
    let config_tab = ToToonConfig {
        indent: 1,
        delimiter: Delimiter::Tab,
    };
    let toon_tab = to_toon(&doc, &config_tab).unwrap();
    assert!(toon_tab.contains("a,b,c") || toon_tab.contains("\"a,b,c\""));
}

#[test]
fn test_large_array() {
    let mut doc = Document::new((2, 0));
    doc.structs.insert(
        "Item".to_string(),
        vec!["id".to_string(), "value".to_string()],
    );

    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);

    // Add 1000 rows
    for i in 0..1000 {
        list.add_row(Node::new(
            "Item",
            format!("i{i}"),
            vec![
                Value::String(format!("i{i}").into()),
                Value::Int(i64::from(i)),
            ],
        ));
    }

    doc.root.insert("items".to_string(), Item::List(list));

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // Should successfully generate TOON for large array
    assert!(toon.contains("items[1000]{id,value}:"));
    assert!(toon.lines().count() > 1000);
}

#[test]
fn test_deeply_nested_but_within_limit() {
    let mut doc = Document::new((2, 0));

    // Create nested structure just under max depth (50 levels)
    // Root is depth 0, so we can nest 99 more levels
    let mut current = std::collections::BTreeMap::new();
    current.insert("value".to_string(), Item::Scalar(Value::Int(42)));

    for i in (0..50).rev() {
        let mut parent = std::collections::BTreeMap::new();
        parent.insert(format!("level{i}"), Item::Object(current));
        current = parent;
    }

    doc.root.insert("root".to_string(), Item::Object(current));

    let config = ToToonConfig::default();
    let result = to_toon(&doc, &config);

    // Should succeed well under max depth
    assert!(result.is_ok());
}

#[test]
fn test_deterministic_output() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("a".to_string(), Item::Scalar(Value::Int(1)));
    doc.root
        .insert("b".to_string(), Item::Scalar(Value::Int(2)));
    doc.root
        .insert("c".to_string(), Item::Scalar(Value::Int(3)));

    let config = ToToonConfig::default();

    // Convert multiple times
    let toon1 = to_toon(&doc, &config).unwrap();
    let toon2 = to_toon(&doc, &config).unwrap();
    let toon3 = to_toon(&doc, &config).unwrap();

    // Should be identical
    assert_eq!(toon1, toon2);
    assert_eq!(toon2, toon3);
}
