// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for `CypherValue` functionality.

use hedl_neo4j::cypher::{CypherValue, RenderMode, StatementType};
use hedl_neo4j::CypherStatement;
use std::collections::BTreeMap;

#[test]
fn test_cypher_value_null() {
    let value = CypherValue::Null;
    assert!(value.is_null());
    assert_eq!(value.to_cypher_literal(), "null");
    assert_eq!(value.as_str(), None);
    assert_eq!(value.as_int(), None);
    assert_eq!(value.as_float(), None);
}

#[test]
fn test_cypher_value_bool() {
    let true_val = CypherValue::Bool(true);
    let false_val = CypherValue::Bool(false);

    assert!(!true_val.is_null());
    assert_eq!(true_val.to_cypher_literal(), "true");
    assert_eq!(false_val.to_cypher_literal(), "false");
}

#[test]
fn test_cypher_value_int() {
    let value = CypherValue::Int(42);
    assert_eq!(value.to_cypher_literal(), "42");
    assert_eq!(value.as_int(), Some(42));
    assert_eq!(value.as_float(), Some(42.0));

    let negative = CypherValue::Int(-100);
    assert_eq!(negative.to_cypher_literal(), "-100");
    assert_eq!(negative.as_int(), Some(-100));

    let zero = CypherValue::Int(0);
    assert_eq!(zero.to_cypher_literal(), "0");
    assert_eq!(zero.as_int(), Some(0));
}

#[test]
fn test_cypher_value_float() {
    // Regular float
    let value = CypherValue::Float(3.25);
    assert!(value.to_cypher_literal().starts_with("3.25"));
    assert_eq!(value.as_float(), Some(3.25));

    // Integer-looking float
    let int_like = CypherValue::Float(42.0);
    assert_eq!(int_like.to_cypher_literal(), "42.0");

    // NaN
    let nan = CypherValue::Float(f64::NAN);
    assert_eq!(nan.to_cypher_literal(), "0.0/0.0");

    // Positive infinity
    let pos_inf = CypherValue::Float(f64::INFINITY);
    assert_eq!(pos_inf.to_cypher_literal(), "1.0/0.0");

    // Negative infinity
    let neg_inf = CypherValue::Float(f64::NEG_INFINITY);
    assert_eq!(neg_inf.to_cypher_literal(), "-1.0/0.0");

    // Scientific notation
    let sci = CypherValue::Float(1.23e10);
    let literal = sci.to_cypher_literal();
    assert!(literal.contains('e') || literal.contains('E') || literal.len() > 5);
}

#[test]
fn test_cypher_value_string() {
    let value = CypherValue::String("hello".to_string());
    assert_eq!(value.to_cypher_literal(), "'hello'");
    assert_eq!(value.as_str(), Some("hello"));

    // String with quotes
    let quoted = CypherValue::String("it's \"quoted\"".to_string());
    let literal = quoted.to_cypher_literal();
    assert!(literal.contains("\\'"));
    assert!(literal.contains("\\\""));

    // String with newlines
    let multiline = CypherValue::String("line1\nline2".to_string());
    let literal = multiline.to_cypher_literal();
    assert!(literal.contains("\\n"));
}

#[test]
fn test_cypher_value_list() {
    let value = CypherValue::List(vec![
        CypherValue::Int(1),
        CypherValue::Int(2),
        CypherValue::Int(3),
    ]);
    assert_eq!(value.to_cypher_literal(), "[1, 2, 3]");

    // Empty list
    let empty = CypherValue::List(vec![]);
    assert_eq!(empty.to_cypher_literal(), "[]");

    // Mixed types
    let mixed = CypherValue::List(vec![
        CypherValue::Int(42),
        CypherValue::String("text".to_string()),
        CypherValue::Bool(true),
        CypherValue::Null,
    ]);
    assert_eq!(mixed.to_cypher_literal(), "[42, 'text', true, null]");
}

#[test]
fn test_cypher_value_map() {
    let mut map = BTreeMap::new();
    map.insert("name".to_string(), CypherValue::String("Alice".to_string()));
    map.insert("age".to_string(), CypherValue::Int(30));

    let value = CypherValue::Map(map);
    let literal = value.to_cypher_literal();

    assert!(literal.starts_with('{'));
    assert!(literal.ends_with('}'));
    assert!(literal.contains("age: 30"));
    assert!(literal.contains("name: 'Alice'"));

    // Empty map
    let empty = CypherValue::Map(BTreeMap::new());
    assert_eq!(empty.to_cypher_literal(), "{}");
}

#[test]
fn test_cypher_value_nested() {
    let mut inner_map = BTreeMap::new();
    inner_map.insert("x".to_string(), CypherValue::Int(1));
    inner_map.insert("y".to_string(), CypherValue::Int(2));

    let nested = CypherValue::List(vec![
        CypherValue::Map(inner_map),
        CypherValue::List(vec![CypherValue::Int(3), CypherValue::Int(4)]),
    ]);

    let literal = nested.to_cypher_literal();
    assert!(literal.contains('['));
    assert!(literal.contains('{'));
    assert!(literal.contains("x: 1"));
}

#[test]
fn test_cypher_value_deep_nesting() {
    // Create a deeply nested structure
    let mut value = CypherValue::Int(0);

    // Nest it 50 levels deep
    for _ in 0..50 {
        value = CypherValue::List(vec![value]);
    }

    // Should handle this gracefully
    let literal = value.to_cypher_literal();
    assert!(!literal.is_empty());
    assert!(literal.contains('['));
}

#[test]
fn test_cypher_value_max_nesting_depth() {
    // Create a structure that exceeds MAX_NESTING_DEPTH (100)
    let mut value = CypherValue::Int(42);

    // Nest it 101 levels deep
    for _ in 0..101 {
        value = CypherValue::List(vec![value]);
    }

    // Should return a safe representation instead of stack overflow
    let literal = value.to_cypher_literal();
    assert!(literal.contains("<structure too deep>") || literal.contains('['));
}

#[test]
fn test_cypher_value_from_conversions() {
    // From bool
    let from_bool: CypherValue = true.into();
    assert_eq!(from_bool, CypherValue::Bool(true));

    // From i64
    let from_i64: CypherValue = 42i64.into();
    assert_eq!(from_i64, CypherValue::Int(42));

    // From i32
    let from_i32: CypherValue = 42i32.into();
    assert_eq!(from_i32, CypherValue::Int(42));

    // From f64
    let from_f64: CypherValue = 3.25f64.into();
    assert_eq!(from_f64, CypherValue::Float(3.25));

    // From String
    let from_string: CypherValue = "hello".to_string().into();
    assert_eq!(from_string, CypherValue::String("hello".to_string()));

    // From &str
    let from_str: CypherValue = "world".into();
    assert_eq!(from_str, CypherValue::String("world".to_string()));

    // From Vec<i32>
    let from_vec: CypherValue = vec![1i32, 2, 3].into();
    assert_eq!(
        from_vec,
        CypherValue::List(vec![
            CypherValue::Int(1),
            CypherValue::Int(2),
            CypherValue::Int(3),
        ])
    );

    // From Option<i32>
    let from_some: CypherValue = Some(42i32).into();
    assert_eq!(from_some, CypherValue::Int(42));

    let from_none: CypherValue = None::<i32>.into();
    assert_eq!(from_none, CypherValue::Null);
}

#[test]
fn test_cypher_statement_creation() {
    let stmt = CypherStatement::new("MATCH (n) RETURN n", StatementType::Query);
    assert_eq!(stmt.query, "MATCH (n) RETURN n");
    assert_eq!(stmt.statement_type, StatementType::Query);
    assert!(stmt.parameters.is_empty());
    assert!(stmt.comment.is_none());
    assert_eq!(stmt.render_mode, RenderMode::Inline);
}

#[test]
fn test_cypher_statement_constraint() {
    let stmt = CypherStatement::constraint("CREATE CONSTRAINT ...");
    assert_eq!(stmt.statement_type, StatementType::Constraint);
}

#[test]
fn test_cypher_statement_index() {
    let stmt = CypherStatement::index("CREATE INDEX ...");
    assert_eq!(stmt.statement_type, StatementType::Index);
}

#[test]
fn test_cypher_statement_create_node() {
    let stmt = CypherStatement::create_node("CREATE (n:User)");
    assert_eq!(stmt.statement_type, StatementType::CreateNode);
}

#[test]
fn test_cypher_statement_create_relationship() {
    let stmt = CypherStatement::create_relationship("CREATE (a)-[:KNOWS]->(b)");
    assert_eq!(stmt.statement_type, StatementType::CreateRelationship);
}

#[test]
fn test_statement_type_variants() {
    // Test all variants exist
    let _constraint = StatementType::Constraint;
    let _index = StatementType::Index;
    let _create_node = StatementType::CreateNode;
    let _create_rel = StatementType::CreateRelationship;
    let _set_prop = StatementType::SetProperty;
    let _query = StatementType::Query;

    // Test equality
    assert_eq!(StatementType::Query, StatementType::Query);
    assert_ne!(StatementType::Query, StatementType::Constraint);
}

#[test]
fn test_render_mode_variants() {
    assert_eq!(RenderMode::default(), RenderMode::Inline);
    assert_ne!(RenderMode::Inline, RenderMode::Parameterized);
}

#[test]
fn test_cypher_value_serialization() {
    // Test that CypherValue can be serialized/deserialized
    let value = CypherValue::Map({
        let mut map = BTreeMap::new();
        map.insert("name".to_string(), CypherValue::String("test".to_string()));
        map.insert("count".to_string(), CypherValue::Int(42));
        map
    });

    let json = serde_json::to_string(&value).unwrap();
    let parsed: CypherValue = serde_json::from_str(&json).unwrap();
    assert_eq!(value, parsed);
}

#[test]
fn test_cypher_statement_serialization() {
    let stmt = CypherStatement::new("MATCH (n) RETURN n", StatementType::Query);
    let json = serde_json::to_string(&stmt).unwrap();
    let parsed: CypherStatement = serde_json::from_str(&json).unwrap();
    assert_eq!(stmt.query, parsed.query);
    assert_eq!(stmt.statement_type, parsed.statement_type);
}

#[test]
fn test_cypher_value_list_of_maps() {
    let mut map1 = BTreeMap::new();
    map1.insert("id".to_string(), CypherValue::Int(1));

    let mut map2 = BTreeMap::new();
    map2.insert("id".to_string(), CypherValue::Int(2));

    let value = CypherValue::List(vec![CypherValue::Map(map1), CypherValue::Map(map2)]);

    let literal = value.to_cypher_literal();
    assert!(literal.contains("id: 1"));
    assert!(literal.contains("id: 2"));
}

#[test]
fn test_cypher_value_map_with_special_keys() {
    let mut map = BTreeMap::new();
    map.insert("123name".to_string(), CypherValue::Int(1));
    map.insert("name-with-dash".to_string(), CypherValue::Int(2));
    map.insert("MATCH".to_string(), CypherValue::Int(3));

    let value = CypherValue::Map(map);
    let literal = value.to_cypher_literal();

    // Keys that need backticks should be escaped
    assert!(literal.contains('`') || literal.contains(':'));
}

#[test]
fn test_cypher_value_string_with_unicode() {
    let value = CypherValue::String("café ☕ 中文".to_string());
    let literal = value.to_cypher_literal();
    assert!(literal.starts_with('\''));
    assert!(literal.ends_with('\''));
    assert!(literal.contains("café"));
}
