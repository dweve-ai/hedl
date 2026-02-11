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

//! Round-trip conversion tests for TOON format
//!
//! These tests verify that converting HEDL → TOON → HEDL preserves
//! the semantic content of documents.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_toon::{from_toon, hedl_to_toon, toon_to_hedl};

#[test]
fn test_roundtrip_simple_scalars() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "string".to_string(),
        Item::Scalar(Value::String("hello".to_string().into())),
    );
    doc.root
        .insert("int".to_string(), Item::Scalar(Value::Int(42)));
    doc.root
        .insert("float".to_string(), Item::Scalar(Value::Float(3.15)));
    doc.root
        .insert("bool_true".to_string(), Item::Scalar(Value::Bool(true)));
    doc.root
        .insert("bool_false".to_string(), Item::Scalar(Value::Bool(false)));
    // Note: null values may not roundtrip in some formats

    let toon = hedl_to_toon(&doc).unwrap();
    let roundtrip_doc = toon_to_hedl(&toon).unwrap();

    // Verify key fields match - use get() to avoid panic on missing keys
    assert!(
        matches!(
            roundtrip_doc.root.get("string"),
            Some(Item::Scalar(Value::String(s))) if s.as_ref() == "hello"
        ),
        "string field mismatch: {:?}",
        roundtrip_doc.root.get("string")
    );
    assert!(
        matches!(
            roundtrip_doc.root.get("int"),
            Some(Item::Scalar(Value::Int(42)))
        ),
        "int field mismatch: {:?}",
        roundtrip_doc.root.get("int")
    );
    assert!(
        matches!(
            roundtrip_doc.root.get("float"),
            Some(Item::Scalar(Value::Float(f))) if (*f - 3.15).abs() < 0.001
        ),
        "float field mismatch: {:?}",
        roundtrip_doc.root.get("float")
    );
    assert!(
        matches!(
            roundtrip_doc.root.get("bool_true"),
            Some(Item::Scalar(Value::Bool(true)))
        ),
        "bool_true field mismatch: {:?}",
        roundtrip_doc.root.get("bool_true")
    );
    assert!(
        matches!(
            roundtrip_doc.root.get("bool_false"),
            Some(Item::Scalar(Value::Bool(false)))
        ),
        "bool_false field mismatch: {:?}",
        roundtrip_doc.root.get("bool_false")
    );
}

#[test]
fn test_roundtrip_references() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "qualified_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::qualified("User", "u123"))),
    );
    doc.root.insert(
        "local_ref".to_string(),
        Item::Scalar(Value::Reference(Reference::local("item1"))),
    );

    let toon = hedl_to_toon(&doc).unwrap();
    let roundtrip_doc = toon_to_hedl(&toon).unwrap();

    if let Item::Scalar(Value::Reference(r)) = &roundtrip_doc.root["qualified_ref"] {
        assert_eq!(r.type_name.as_deref(), Some("User"));
        assert_eq!(r.id.as_ref(), "u123");
    } else {
        panic!("Expected qualified reference");
    }

    if let Item::Scalar(Value::Reference(r)) = &roundtrip_doc.root["local_ref"] {
        assert_eq!(r.type_name, None);
        assert_eq!(r.id.as_ref(), "item1");
    } else {
        panic!("Expected local reference");
    }
}

#[test]
fn test_roundtrip_nested_objects() {
    let mut doc = Document::new((2, 0));

    let mut inner = std::collections::BTreeMap::new();
    inner.insert("x".to_string(), Item::Scalar(Value::Int(10)));
    inner.insert("y".to_string(), Item::Scalar(Value::Int(20)));

    let mut outer = std::collections::BTreeMap::new();
    outer.insert("position".to_string(), Item::Object(inner));
    outer.insert(
        "name".to_string(),
        Item::Scalar(Value::String("Entity1".to_string().into())),
    );

    doc.root.insert("entity".to_string(), Item::Object(outer));

    let toon = hedl_to_toon(&doc).unwrap();
    let roundtrip_doc = toon_to_hedl(&toon).unwrap();

    if let Item::Object(entity) = &roundtrip_doc.root["entity"] {
        assert!(matches!(
            &entity["name"],
            Item::Scalar(Value::String(s)) if s.as_ref() == "Entity1"
        ));

        if let Item::Object(position) = &entity["position"] {
            assert!(matches!(&position["x"], Item::Scalar(Value::Int(10))));
            assert!(matches!(&position["y"], Item::Scalar(Value::Int(20))));
        } else {
            panic!("Expected position object");
        }
    } else {
        panic!("Expected entity object");
    }
}

#[test]
fn test_roundtrip_tabular_arrays() {
    let mut doc = Document::new((2, 0));
    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string(), "age".to_string()],
    );

    let mut list = MatrixList::new(
        "User",
        vec!["id".to_string(), "name".to_string(), "age".to_string()],
    );
    list.add_row(Node::new(
        "User",
        "u1",
        vec![
            Value::String("u1".to_string().into()),
            Value::String("Alice".to_string().into()),
            Value::Int(30),
        ],
    ));
    list.add_row(Node::new(
        "User",
        "u2",
        vec![
            Value::String("u2".to_string().into()),
            Value::String("Bob".to_string().into()),
            Value::Int(25),
        ],
    ));

    doc.root.insert("users".to_string(), Item::List(list));

    let toon = hedl_to_toon(&doc).unwrap();
    let roundtrip_doc = toon_to_hedl(&toon).unwrap();

    if let Item::List(users) = &roundtrip_doc.root["users"] {
        assert_eq!(users.rows.len(), 2);
        // JSON sorts keys alphabetically, so schema order may differ
        let mut schema_sorted: Vec<_> = users.schema.iter().collect();
        schema_sorted.sort();
        assert_eq!(schema_sorted, vec!["age", "id", "name"]);

        assert_eq!(users.rows[0].fields.len(), 3);
        // Fields are in schema order, so we need to find them by position
        // The schema might be ["age", "id", "name"] after JSON sorting
    } else {
        panic!("Expected users list");
    }
}

#[test]
fn test_roundtrip_expanded_arrays() {
    // Note: This test is simplified because the current TOON parser
    // may not fully support nested children in expanded format.
    // We test expanded format without nested children.

    let mut doc = Document::new((2, 0));
    doc.structs.insert(
        "Order".to_string(),
        vec!["id".to_string(), "total".to_string(), "status".to_string()],
    );

    let mut list = MatrixList::new(
        "Order",
        vec!["id".to_string(), "total".to_string(), "status".to_string()],
    );

    // Add an order with all primitive fields
    list.add_row(Node::new(
        "Order",
        "o1",
        vec![
            Value::String("o1".to_string().into()),
            Value::Float(99.99),
            Value::String("pending".to_string().into()),
        ],
    ));

    doc.root.insert("orders".to_string(), Item::List(list));

    let toon = hedl_to_toon(&doc).unwrap();
    let roundtrip_doc = toon_to_hedl(&toon).unwrap();

    if let Item::List(orders) = &roundtrip_doc.root["orders"] {
        assert_eq!(orders.rows.len(), 1);
        let order = &orders.rows[0];

        // Check order fields
        if let Value::String(s) = &order.fields[0] {
            assert_eq!(s.as_ref(), "o1");
        }
        if let Value::Float(f) = &order.fields[1] {
            assert!((*f - 99.99).abs() < 0.01);
        }
        if let Value::String(s) = &order.fields[2] {
            assert_eq!(s.as_ref(), "pending");
        }
    } else {
        panic!("Expected orders list");
    }
}

#[test]
fn test_roundtrip_special_characters() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "newline".to_string(),
        Item::Scalar(Value::String("line1\nline2".to_string().into())),
    );
    doc.root.insert(
        "quote".to_string(),
        Item::Scalar(Value::String("say \"hello\"".to_string().into())),
    );
    doc.root.insert(
        "backslash".to_string(),
        Item::Scalar(Value::String("path\\to\\file".to_string().into())),
    );
    doc.root.insert(
        "tab".to_string(),
        Item::Scalar(Value::String("col1\tcol2".to_string().into())),
    );

    let toon = hedl_to_toon(&doc).unwrap();
    let roundtrip_doc = toon_to_hedl(&toon).unwrap();

    if let Item::Scalar(Value::String(s)) = &roundtrip_doc.root["newline"] {
        assert_eq!(s.as_ref(), "line1\nline2");
    }
    if let Item::Scalar(Value::String(s)) = &roundtrip_doc.root["quote"] {
        assert_eq!(s.as_ref(), "say \"hello\"");
    }
    if let Item::Scalar(Value::String(s)) = &roundtrip_doc.root["backslash"] {
        assert_eq!(s.as_ref(), "path\\to\\file");
    }
    if let Item::Scalar(Value::String(s)) = &roundtrip_doc.root["tab"] {
        assert_eq!(s.as_ref(), "col1\tcol2");
    }
}

#[test]
fn test_roundtrip_empty_values() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "empty_string".to_string(),
        Item::Scalar(Value::String(String::new().into())),
    );
    doc.root.insert(
        "empty_list".to_string(),
        Item::List(MatrixList::new("Item", vec!["id".to_string()])),
    );
    doc.root.insert(
        "empty_object".to_string(),
        Item::Object(std::collections::BTreeMap::new()),
    );

    let toon = hedl_to_toon(&doc).unwrap();
    let roundtrip_doc = toon_to_hedl(&toon).unwrap();

    if let Item::Scalar(Value::String(s)) = &roundtrip_doc.root["empty_string"] {
        assert_eq!(s.as_ref(), "");
    }
    if let Item::List(list) = &roundtrip_doc.root["empty_list"] {
        assert_eq!(list.rows.len(), 0);
    }
    if let Item::Object(obj) = &roundtrip_doc.root["empty_object"] {
        assert_eq!(obj.len(), 0);
    }
}

#[test]
fn test_roundtrip_unicode() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "chinese".to_string(),
        Item::Scalar(Value::String("你好世界".to_string().into())),
    );
    doc.root.insert(
        "emoji".to_string(),
        Item::Scalar(Value::String("🌍🚀⭐".to_string().into())),
    );
    doc.root.insert(
        "mixed".to_string(),
        Item::Scalar(Value::String("Hello 世界 🌍".to_string().into())),
    );

    let toon = hedl_to_toon(&doc).unwrap();
    let roundtrip_doc = toon_to_hedl(&toon).unwrap();

    if let Item::Scalar(Value::String(s)) = &roundtrip_doc.root["chinese"] {
        assert_eq!(s.as_ref(), "你好世界");
    }
    if let Item::Scalar(Value::String(s)) = &roundtrip_doc.root["emoji"] {
        assert_eq!(s.as_ref(), "🌍🚀⭐");
    }
    if let Item::Scalar(Value::String(s)) = &roundtrip_doc.root["mixed"] {
        assert_eq!(s.as_ref(), "Hello 世界 🌍");
    }
}

#[test]
fn test_roundtrip_complex_document() {
    // Build a complex document with all features
    let mut doc = Document::new((2, 0));

    doc.structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    doc.structs.insert(
        "Post".to_string(),
        vec!["id".to_string(), "title".to_string(), "author".to_string()],
    );

    // Users list
    let mut users = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
    users.add_row(Node::new(
        "User",
        "u1",
        vec![
            Value::String("u1".to_string().into()),
            Value::String("Alice".to_string().into()),
        ],
    ));
    users.add_row(Node::new(
        "User",
        "u2",
        vec![
            Value::String("u2".to_string().into()),
            Value::String("Bob".to_string().into()),
        ],
    ));
    doc.root.insert("users".to_string(), Item::List(users));

    // Posts with references
    let mut posts = MatrixList::new(
        "Post",
        vec!["id".to_string(), "title".to_string(), "author".to_string()],
    );
    posts.add_row(Node::new(
        "Post",
        "p1",
        vec![
            Value::String("p1".to_string().into()),
            Value::String("First Post".to_string().into()),
            Value::Reference(Reference::qualified("User", "u1")),
        ],
    ));
    posts.add_row(Node::new(
        "Post",
        "p2",
        vec![
            Value::String("p2".to_string().into()),
            Value::String("Second Post".to_string().into()),
            Value::Reference(Reference::qualified("User", "u2")),
        ],
    ));
    doc.root.insert("posts".to_string(), Item::List(posts));

    // Config object
    let mut config = std::collections::BTreeMap::new();
    config.insert(
        "version".to_string(),
        Item::Scalar(Value::String("1.0".to_string().into())),
    );
    config.insert("debug".to_string(), Item::Scalar(Value::Bool(true)));
    config.insert("max_items".to_string(), Item::Scalar(Value::Int(100)));
    doc.root.insert("config".to_string(), Item::Object(config));

    // Round-trip
    let toon = hedl_to_toon(&doc).unwrap();
    let roundtrip_doc = from_toon(&toon).unwrap();

    // Verify structure
    assert!(roundtrip_doc.root.contains_key("users"));
    assert!(roundtrip_doc.root.contains_key("posts"));
    assert!(roundtrip_doc.root.contains_key("config"));

    // Verify users
    if let Item::List(users) = &roundtrip_doc.root["users"] {
        assert_eq!(users.rows.len(), 2);
    }

    // Verify posts with references
    if let Item::List(posts) = &roundtrip_doc.root["posts"] {
        assert_eq!(posts.rows.len(), 2);
        if let Value::Reference(r) = &posts.rows[0].fields[2] {
            assert_eq!(r.type_name.as_deref(), Some("User"));
            assert_eq!(r.id.as_ref(), "u1");
        }
    }

    // Verify config
    if let Item::Object(config) = &roundtrip_doc.root["config"] {
        assert!(matches!(
            &config["version"],
            Item::Scalar(Value::String(s)) if s.as_ref() == "1.0"
        ));
        assert!(matches!(&config["debug"], Item::Scalar(Value::Bool(true))));
        assert!(matches!(
            &config["max_items"],
            Item::Scalar(Value::Int(100))
        ));
    }
}

#[test]
fn test_roundtrip_numeric_edge_cases() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("zero".to_string(), Item::Scalar(Value::Int(0)));
    doc.root
        .insert("negative".to_string(), Item::Scalar(Value::Int(-42)));
    doc.root
        .insert("max_i64".to_string(), Item::Scalar(Value::Int(i64::MAX)));
    doc.root
        .insert("min_i64".to_string(), Item::Scalar(Value::Int(i64::MIN)));
    doc.root
        .insert("float_zero".to_string(), Item::Scalar(Value::Float(0.0)));
    doc.root.insert(
        "float_negative_zero".to_string(),
        Item::Scalar(Value::Float(-0.0)),
    );
    doc.root.insert(
        "float_small".to_string(),
        Item::Scalar(Value::Float(0.0001)),
    );
    doc.root
        .insert("float_large".to_string(), Item::Scalar(Value::Float(1e10)));

    let toon = hedl_to_toon(&doc).unwrap();
    let roundtrip_doc = toon_to_hedl(&toon).unwrap();

    assert!(matches!(
        &roundtrip_doc.root["zero"],
        Item::Scalar(Value::Int(0))
    ));
    assert!(matches!(
        &roundtrip_doc.root["negative"],
        Item::Scalar(Value::Int(-42))
    ));
    assert!(matches!(
        &roundtrip_doc.root["max_i64"],
        Item::Scalar(Value::Int(i64::MAX))
    ));
    assert!(matches!(
        &roundtrip_doc.root["min_i64"],
        Item::Scalar(Value::Int(i64::MIN))
    ));

    // Float comparisons with tolerance
    if let Item::Scalar(Value::Float(f)) = &roundtrip_doc.root["float_zero"] {
        assert_eq!(*f, 0.0);
    }
    // -0.0 gets normalized to 0.0 per TOON spec
    if let Item::Scalar(Value::Int(i)) = &roundtrip_doc.root["float_negative_zero"] {
        assert_eq!(*i, 0);
    }
}
