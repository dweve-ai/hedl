// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive round-trip tests to ensure YAML -> HEDL -> YAML conversion
//! preserves data integrity across various edge cases and complex scenarios.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_yaml::{from_yaml, to_yaml, FromYamlConfig, ToYamlConfig};
use std::collections::BTreeMap;

// ==================== Basic Round-trip Tests ====================

#[test]
fn test_roundtrip_all_scalar_types() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut root = BTreeMap::new();
    root.insert("null_val".to_string(), Item::Scalar(Value::Null));
    root.insert("bool_true".to_string(), Item::Scalar(Value::Bool(true)));
    root.insert("bool_false".to_string(), Item::Scalar(Value::Bool(false)));
    root.insert("int_positive".to_string(), Item::Scalar(Value::Int(42)));
    root.insert("int_negative".to_string(), Item::Scalar(Value::Int(-100)));
    root.insert("int_zero".to_string(), Item::Scalar(Value::Int(0)));
    root.insert(
        "float_positive".to_string(),
        Item::Scalar(Value::Float(3.15)),
    );
    root.insert(
        "float_negative".to_string(),
        Item::Scalar(Value::Float(-2.72)),
    );
    root.insert("float_zero".to_string(), Item::Scalar(Value::Float(0.0)));
    root.insert(
        "string_simple".to_string(),
        Item::Scalar(Value::String("hello".into())),
    );
    root.insert(
        "string_empty".to_string(),
        Item::Scalar(Value::String("".into())),
    );

    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    assert_eq!(restored.root.len(), doc.root.len());
    // from_yaml creates v2.0 documents by default (version not preserved in YAML)
    assert_eq!(restored.version, (2, 0));
}

#[test]
fn test_roundtrip_unicode_strings() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut root = BTreeMap::new();
    root.insert(
        "japanese".to_string(),
        Item::Scalar(Value::String("こんにちは".into())),
    );
    root.insert(
        "emoji".to_string(),
        Item::Scalar(Value::String("🎉🚀💻".into())),
    );
    root.insert(
        "arabic".to_string(),
        Item::Scalar(Value::String("مرحبا".into())),
    );
    root.insert(
        "russian".to_string(),
        Item::Scalar(Value::String("Привет".into())),
    );
    root.insert(
        "mixed".to_string(),
        Item::Scalar(Value::String("Hello 世界 🌍".into())),
    );

    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::Scalar(Value::String(s))) = restored.root.get("japanese") {
        assert_eq!(s.as_ref(), "こんにちは");
    }

    if let Some(Item::Scalar(Value::String(s))) = restored.root.get("emoji") {
        assert_eq!(s.as_ref(), "🎉🚀💻");
    }
}

#[test]
fn test_roundtrip_special_characters() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut root = BTreeMap::new();
    root.insert(
        "quotes".to_string(),
        Item::Scalar(Value::String("He said \"hello\"".into())),
    );
    root.insert(
        "newlines".to_string(),
        Item::Scalar(Value::String("line1\nline2\nline3".into())),
    );
    root.insert(
        "tabs".to_string(),
        Item::Scalar(Value::String("col1\tcol2\tcol3".into())),
    );
    root.insert(
        "backslash".to_string(),
        Item::Scalar(Value::String("path\\to\\file".into())),
    );

    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::Scalar(Value::String(s))) = restored.root.get("quotes") {
        assert!(s.contains("\"hello\""));
    }

    if let Some(Item::Scalar(Value::String(s))) = restored.root.get("newlines") {
        assert!(s.contains("line1"));
        assert!(s.contains("line2"));
    }
}

#[test]
fn test_roundtrip_nested_objects() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut inner1 = BTreeMap::new();
    inner1.insert("x".to_string(), Item::Scalar(Value::Int(10)));
    inner1.insert("y".to_string(), Item::Scalar(Value::Int(20)));

    let mut inner2 = BTreeMap::new();
    inner2.insert("a".to_string(), Item::Scalar(Value::String("test".into())));
    inner2.insert("b".to_string(), Item::Object(inner1));

    let mut root = BTreeMap::new();
    root.insert("nested".to_string(), Item::Object(inner2));

    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    // Verify structure is preserved
    if let Some(Item::Object(nested)) = restored.root.get("nested") {
        assert!(nested.contains_key("a"));
        assert!(nested.contains_key("b"));
    } else {
        panic!("Expected nested object");
    }
}

#[test]
fn test_roundtrip_matrix_list_simple() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut list = MatrixList::new(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string(), "age".to_string()],
    );

    list.add_row(Node::new(
        "User",
        "u1",
        vec![
            Value::String("u1".into()),
            Value::String("Alice".into()),
            Value::Int(30),
        ],
    ));

    list.add_row(Node::new(
        "User",
        "u2",
        vec![
            Value::String("u2".into()),
            Value::String("Bob".into()),
            Value::Int(25),
        ],
    ));

    let mut root = BTreeMap::new();
    root.insert("users".to_string(), Item::List(list));
    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::List(restored_list)) = restored.root.get("users") {
        assert_eq!(restored_list.rows.len(), 2);
        assert_eq!(restored_list.type_name, "User");
        assert_eq!(restored_list.schema.len(), 3);
    } else {
        panic!("Expected matrix list");
    }
}

#[test]
fn test_roundtrip_empty_collections() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut root = BTreeMap::new();
    root.insert("empty_obj".to_string(), Item::Object(BTreeMap::new()));
    root.insert(
        "empty_list".to_string(),
        Item::List(MatrixList::new("Item".to_string(), vec!["id".to_string()])),
    );

    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::Object(obj)) = restored.root.get("empty_obj") {
        assert!(obj.is_empty());
    }

    if let Some(Item::List(list)) = restored.root.get("empty_list") {
        assert!(list.rows.is_empty());
    }
}

#[test]
fn test_roundtrip_tensor_1d() {
    use hedl_core::lex::Tensor;

    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let tensor = Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
        Tensor::Scalar(4.0),
    ]);

    let mut root = BTreeMap::new();
    root.insert(
        "vector".to_string(),
        Item::Scalar(Value::Tensor(Box::new(tensor))),
    );
    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::Scalar(Value::Tensor(t))) = restored.root.get("vector") {
        if let Tensor::Array(items) = t.as_ref() {
            assert_eq!(items.len(), 4);
        }
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_roundtrip_tensor_2d() {
    use hedl_core::lex::Tensor;

    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let tensor = Tensor::Array(vec![
        Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
            Tensor::Scalar(3.0),
        ]),
        Tensor::Array(vec![
            Tensor::Scalar(4.0),
            Tensor::Scalar(5.0),
            Tensor::Scalar(6.0),
        ]),
    ]);

    let mut root = BTreeMap::new();
    root.insert(
        "matrix".to_string(),
        Item::Scalar(Value::Tensor(Box::new(tensor))),
    );
    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::Scalar(Value::Tensor(t))) = restored.root.get("matrix") {
        if let Tensor::Array(rows) = t.as_ref() {
            assert_eq!(rows.len(), 2);
        }
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_roundtrip_references() {
    use hedl_core::Reference;

    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut root = BTreeMap::new();
    root.insert(
        "local_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("item1"))),
    );
    root.insert(
        "qualified_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "user1"))),
    );

    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("local_ref") {
        assert_eq!(r.id.as_ref(), "item1");
        assert_eq!(r.type_name, None);
    } else {
        panic!("Expected local reference");
    }

    if let Some(Item::Scalar(Value::Reference(r))) = restored.root.get("qualified_ref") {
        assert_eq!(r.id.as_ref(), "user1");
        assert_eq!(r.type_name.as_deref(), Some("User"));
    } else {
        panic!("Expected qualified reference");
    }
}

#[test]
fn test_roundtrip_expressions() {
    use hedl_core::lex::{ExprLiteral, Expression, Span};

    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let expr = Expression::Call {
        name: "add".to_string(),
        args: vec![
            Expression::Identifier {
                name: "x".to_string(),
                span: Span::default(),
            },
            Expression::Literal {
                value: ExprLiteral::Int(10),
                span: Span::default(),
            },
        ],
        span: Span::default(),
    };

    let mut root = BTreeMap::new();
    root.insert(
        "calculation".to_string(),
        Item::Scalar(Value::Expression(Box::new(expr))),
    );
    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::Scalar(Value::Expression(e))) = restored.root.get("calculation") {
        let expr_str = e.to_string();
        assert!(expr_str.contains("add"));
    } else {
        panic!("Expected expression");
    }
}

#[test]
fn test_roundtrip_float_special_values() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut root = BTreeMap::new();
    root.insert(
        "infinity".to_string(),
        Item::Scalar(Value::Float(f64::INFINITY)),
    );
    root.insert(
        "neg_infinity".to_string(),
        Item::Scalar(Value::Float(f64::NEG_INFINITY)),
    );
    root.insert("nan".to_string(), Item::Scalar(Value::Float(f64::NAN)));

    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::Scalar(Value::Float(f))) = restored.root.get("infinity") {
        assert!(f.is_infinite() && f.is_sign_positive());
    }

    if let Some(Item::Scalar(Value::Float(f))) = restored.root.get("neg_infinity") {
        assert!(f.is_infinite() && f.is_sign_negative());
    }

    if let Some(Item::Scalar(Value::Float(f))) = restored.root.get("nan") {
        assert!(f.is_nan());
    }
}

#[test]
fn test_roundtrip_large_numbers() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut root = BTreeMap::new();
    root.insert("max_int".to_string(), Item::Scalar(Value::Int(i64::MAX)));
    root.insert("min_int".to_string(), Item::Scalar(Value::Int(i64::MIN)));
    root.insert(
        "large_float".to_string(),
        Item::Scalar(Value::Float(1.7976931348623157e308)),
    );

    doc.root = root;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::Scalar(Value::Int(n))) = restored.root.get("max_int") {
        assert_eq!(*n, i64::MAX);
    }

    if let Some(Item::Scalar(Value::Int(n))) = restored.root.get("min_int") {
        assert_eq!(*n, i64::MIN);
    }
}

#[test]
fn test_roundtrip_complex_nested_structure() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    // Build complex nested structure
    let mut level3 = BTreeMap::new();
    level3.insert(
        "value".to_string(),
        Item::Scalar(Value::String("deep".into())),
    );

    let mut level2 = BTreeMap::new();
    level2.insert("nested".to_string(), Item::Object(level3));
    level2.insert("count".to_string(), Item::Scalar(Value::Int(42)));

    let mut level1 = BTreeMap::new();
    level1.insert("data".to_string(), Item::Object(level2));

    doc.root = level1;

    let to_config = ToYamlConfig::default();
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    // Verify structure
    assert!(restored.root.contains_key("data"));
}

#[test]
fn test_roundtrip_metadata_preserved() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut list = MatrixList::new(
        "Person".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    list.add_row(Node::new(
        "Person",
        "p1",
        vec![Value::String("p1".into()), Value::String("Alice".into())],
    ));

    doc.root.insert("people".to_string(), Item::List(list));

    // With metadata enabled
    let to_config = ToYamlConfig {
        include_metadata: true,
        flatten_lists: false,
        include_children: true,
    };
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    if let Some(Item::List(restored_list)) = restored.root.get("people") {
        assert_eq!(restored_list.type_name, "Person");
        assert_eq!(restored_list.schema, vec!["id", "name"]);
    } else {
        panic!("Expected matrix list");
    }
}

#[test]
fn test_roundtrip_flattened_lists() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    let mut list = MatrixList::new("Item".to_string(), vec!["id".to_string()]);
    list.add_row(Node::new("Item", "i1", vec![Value::String("i1".into())]));

    doc.root.insert("items".to_string(), Item::List(list));

    // With flattened lists
    let to_config = ToYamlConfig {
        include_metadata: false,
        flatten_lists: true,
        include_children: true,
    };
    let yaml = to_yaml(&doc, &to_config).unwrap();

    let from_config = FromYamlConfig::default();
    let restored = from_yaml(&yaml, &from_config).unwrap();

    // Should still parse as list
    assert!(matches!(restored.root.get("items"), Some(Item::List(_))));
}

#[test]
fn test_roundtrip_multiple_iterations() {
    let mut doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        root: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        schema_versions: BTreeMap::new(),
    };

    doc.root.insert(
        "test".to_string(),
        Item::Scalar(Value::String("value".into())),
    );

    let to_config = ToYamlConfig::default();
    let from_config = FromYamlConfig::default();

    // Round-trip multiple times
    let yaml1 = to_yaml(&doc, &to_config).unwrap();
    let doc1 = from_yaml(&yaml1, &from_config).unwrap();

    let yaml2 = to_yaml(&doc1, &to_config).unwrap();
    let doc2 = from_yaml(&yaml2, &from_config).unwrap();

    let yaml3 = to_yaml(&doc2, &to_config).unwrap();
    let doc3 = from_yaml(&yaml3, &from_config).unwrap();

    // Should remain stable
    assert_eq!(doc3.root.len(), 1);
}
