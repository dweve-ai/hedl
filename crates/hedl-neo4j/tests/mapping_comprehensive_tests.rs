// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for mapping module.

use hedl_neo4j::cypher::CypherValue;
use hedl_neo4j::mapping::{Neo4jNode, Neo4jRelationship};
use std::collections::BTreeMap;

#[test]
fn test_neo4j_node_new() {
    let node = Neo4jNode::new("User", "alice");
    assert_eq!(node.label, "User");
    assert_eq!(node.id, "alice");
    assert!(node.properties.is_empty());
}

#[test]
fn test_neo4j_node_with_property() {
    let node = Neo4jNode::new("User", "alice")
        .with_property("name", "Alice Smith")
        .with_property("age", 30);

    assert_eq!(node.properties.len(), 2);
    assert_eq!(
        node.properties.get("name").unwrap().as_str(),
        Some("Alice Smith")
    );
    assert_eq!(node.properties.get("age").unwrap().as_int(), Some(30));
}

#[test]
fn test_neo4j_node_clone() {
    let original = Neo4jNode::new("User", "alice").with_property("name", "Alice");

    let cloned = original.clone();
    assert_eq!(original.label, cloned.label);
    assert_eq!(original.id, cloned.id);
    assert_eq!(original.properties.len(), cloned.properties.len());
}

#[test]
fn test_neo4j_node_debug() {
    let node = Neo4jNode::new("User", "alice");
    let debug = format!("{node:?}");
    assert!(debug.contains("User"));
    assert!(debug.contains("alice"));
}

#[test]
fn test_neo4j_node_with_multiple_property_types() {
    let node = Neo4jNode::new("Product", "p1")
        .with_property("name", "Widget")
        .with_property("price", 29.99)
        .with_property("in_stock", true)
        .with_property("quantity", 100);

    assert_eq!(node.properties.len(), 4);
    assert_eq!(
        node.properties.get("name").unwrap().as_str(),
        Some("Widget")
    );
    assert_eq!(
        node.properties.get("price").unwrap().as_float(),
        Some(29.99)
    );
    assert!(matches!(
        node.properties.get("in_stock").unwrap(),
        CypherValue::Bool(true)
    ));
    assert_eq!(node.properties.get("quantity").unwrap().as_int(), Some(100));
}

#[test]
fn test_neo4j_node_with_null_property() {
    let mut node = Neo4jNode::new("User", "alice");
    node.properties
        .insert("middle_name".to_string(), CypherValue::Null);

    assert_eq!(node.properties.len(), 1);
    assert!(node.properties.get("middle_name").unwrap().is_null());
}

#[test]
fn test_neo4j_node_with_complex_properties() {
    let mut node = Neo4jNode::new("Document", "doc1");

    // List property
    let tags: Vec<CypherValue> = vec!["rust".into(), "neo4j".into(), "database".into()];
    node.properties
        .insert("tags".to_string(), CypherValue::List(tags));

    // Map property
    let mut metadata = BTreeMap::new();
    metadata.insert(
        "author".to_string(),
        CypherValue::String("Alice".to_string()),
    );
    metadata.insert("version".to_string(), CypherValue::Int(2));
    node.properties
        .insert("metadata".to_string(), CypherValue::Map(metadata));

    assert_eq!(node.properties.len(), 2);
}

#[test]
fn test_neo4j_node_with_unicode_label_and_id() {
    let node = Neo4jNode::new("用户", "用户001").with_property("名字", "张三");

    assert_eq!(node.label, "用户");
    assert_eq!(node.id, "用户001");
    assert_eq!(node.properties.len(), 1);
}

#[test]
fn test_neo4j_node_with_empty_strings() {
    let node = Neo4jNode::new("", "");
    assert_eq!(node.label, "");
    assert_eq!(node.id, "");
}

#[test]
fn test_neo4j_node_with_special_characters() {
    let node = Neo4jNode::new("User-Type", "user@123").with_property("email", "alice@example.com");

    assert_eq!(node.label, "User-Type");
    assert_eq!(node.id, "user@123");
}

#[test]
fn test_neo4j_relationship_new() {
    let rel = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob");

    assert_eq!(rel.from_label, "User");
    assert_eq!(rel.from_id, "alice");
    assert_eq!(rel.rel_type, "FOLLOWS");
    assert_eq!(rel.to_label, "User");
    assert_eq!(rel.to_id, "bob");
    assert!(rel.properties.is_empty());
}

#[test]
fn test_neo4j_relationship_with_property() {
    let rel = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob")
        .with_property("since", "2024-01-01")
        .with_property("weight", 0.85);

    assert_eq!(rel.properties.len(), 2);
    assert_eq!(
        rel.properties.get("since").unwrap().as_str(),
        Some("2024-01-01")
    );
    assert_eq!(rel.properties.get("weight").unwrap().as_float(), Some(0.85));
}

#[test]
fn test_neo4j_relationship_clone() {
    let original = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob")
        .with_property("since", "2024-01-01");

    let cloned = original.clone();
    assert_eq!(original.from_label, cloned.from_label);
    assert_eq!(original.from_id, cloned.from_id);
    assert_eq!(original.rel_type, cloned.rel_type);
    assert_eq!(original.to_label, cloned.to_label);
    assert_eq!(original.to_id, cloned.to_id);
    assert_eq!(original.properties.len(), cloned.properties.len());
}

#[test]
fn test_neo4j_relationship_debug() {
    let rel = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob");
    let debug = format!("{rel:?}");
    assert!(debug.contains("User"));
    assert!(debug.contains("alice"));
    assert!(debug.contains("FOLLOWS"));
    assert!(debug.contains("bob"));
}

#[test]
fn test_neo4j_relationship_self_reference() {
    let rel = Neo4jRelationship::new("User", "alice", "LIKES", "User", "alice");

    assert_eq!(rel.from_id, "alice");
    assert_eq!(rel.to_id, "alice");
}

#[test]
fn test_neo4j_relationship_with_unicode() {
    let rel = Neo4jRelationship::new("用户", "用户001", "关注", "用户", "用户002")
        .with_property("时间", "2024-01-01");

    assert_eq!(rel.from_label, "用户");
    assert_eq!(rel.rel_type, "关注");
    assert_eq!(rel.properties.len(), 1);
}

#[test]
fn test_neo4j_relationship_different_labels() {
    let rel = Neo4jRelationship::new("User", "alice", "AUTHORED", "Post", "post1");

    assert_eq!(rel.from_label, "User");
    assert_eq!(rel.to_label, "Post");
    assert_ne!(rel.from_label, rel.to_label);
}

#[test]
fn test_neo4j_relationship_with_complex_properties() {
    let mut rel = Neo4jRelationship::new("User", "alice", "REVIEWED", "Product", "p1");

    // Add various property types
    rel.properties
        .insert("rating".to_string(), CypherValue::Int(5));
    rel.properties
        .insert("verified".to_string(), CypherValue::Bool(true));
    rel.properties.insert(
        "comment".to_string(),
        CypherValue::String("Great product!".to_string()),
    );

    let tags = vec!["helpful".into(), "detailed".into()];
    rel.properties
        .insert("tags".to_string(), CypherValue::List(tags));

    assert_eq!(rel.properties.len(), 4);
}

#[test]
fn test_neo4j_relationship_empty_strings() {
    let rel = Neo4jRelationship::new("", "", "", "", "");
    assert_eq!(rel.from_label, "");
    assert_eq!(rel.from_id, "");
    assert_eq!(rel.rel_type, "");
    assert_eq!(rel.to_label, "");
    assert_eq!(rel.to_id, "");
}

#[test]
fn test_neo4j_relationship_special_characters() {
    let rel = Neo4jRelationship::new(
        "User-Type",
        "user@123",
        "HAS_PERMISSION",
        "Resource-Type",
        "resource#456",
    );

    assert_eq!(rel.from_label, "User-Type");
    assert_eq!(rel.from_id, "user@123");
    assert_eq!(rel.rel_type, "HAS_PERMISSION");
    assert_eq!(rel.to_label, "Resource-Type");
    assert_eq!(rel.to_id, "resource#456");
}

#[test]
fn test_neo4j_node_property_chaining() {
    let node = Neo4jNode::new("User", "alice")
        .with_property("name", "Alice")
        .with_property("age", 30)
        .with_property("active", true);

    assert_eq!(node.properties.len(), 3);
}

#[test]
fn test_neo4j_relationship_property_chaining() {
    let rel = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob")
        .with_property("since", "2024")
        .with_property("weight", 0.9)
        .with_property("active", true);

    assert_eq!(rel.properties.len(), 3);
}

#[test]
fn test_neo4j_node_btreemap_ordering() {
    let node = Neo4jNode::new("User", "alice")
        .with_property("zebra", 1)
        .with_property("alpha", 2)
        .with_property("beta", 3);

    // BTreeMap maintains sorted order
    let keys: Vec<&String> = node.properties.keys().collect();
    assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
}

#[test]
fn test_neo4j_relationship_btreemap_ordering() {
    let rel = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob")
        .with_property("zebra", 1)
        .with_property("alpha", 2)
        .with_property("beta", 3);

    let keys: Vec<&String> = rel.properties.keys().collect();
    assert_eq!(keys, vec!["alpha", "beta", "zebra"]);
}

#[test]
fn test_neo4j_node_large_property_set() {
    let mut node = Neo4jNode::new("ComplexNode", "cn1");

    // Add many properties
    for i in 0..100 {
        node.properties
            .insert(format!("prop_{i}"), CypherValue::Int(i64::from(i)));
    }

    assert_eq!(node.properties.len(), 100);

    // Verify first and last
    assert_eq!(node.properties.get("prop_0").unwrap().as_int(), Some(0));
    assert_eq!(node.properties.get("prop_99").unwrap().as_int(), Some(99));
}

#[test]
fn test_neo4j_relationship_with_null_properties() {
    let mut rel = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob");
    rel.properties
        .insert("optional_field".to_string(), CypherValue::Null);

    assert_eq!(rel.properties.len(), 1);
    assert!(rel.properties.get("optional_field").unwrap().is_null());
}
