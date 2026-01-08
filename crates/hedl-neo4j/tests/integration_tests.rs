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

//! Integration tests for hedl-neo4j against a real Neo4j instance.
//!
//! These tests require a running Neo4j instance at localhost:7687.
//! Run with: cargo test -p hedl-neo4j --features integration-tests --test integration_tests

#![cfg(feature = "integration-tests")]

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_neo4j::{
    from_neo4j_records, hedl_to_cypher, to_cypher, to_cypher_statements, CypherValue,
    FromNeo4jConfig, Neo4jNode, Neo4jRecord, Neo4jRelationship, RelationshipNaming, ToCypherConfig,
};
use hedl_test::fixtures;
use neo4rs::{ConfigBuilder, Graph, Node as Neo4rsNode, Query};
use serial_test::serial;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Helper to create a Neo4j connection.
async fn connect() -> Arc<Graph> {
    // Use environment variables with fallback defaults
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "".to_string());

    let config = ConfigBuilder::default()
        .uri(&uri)
        .user(&user)
        .password(&password)
        .build()
        .expect("Failed to build config");

    Arc::new(
        Graph::connect(config)
            .expect("Failed to connect to Neo4j"),
    )
}

/// Helper to clean up test data.
async fn cleanup(graph: &Graph) {
    graph
        .run(Query::new("MATCH (n) DETACH DELETE n".to_string()))
        .await
        .expect("Failed to cleanup");
}

/// Execute CypherStatements against Neo4j.
async fn execute_statements(graph: &Graph, statements: &[hedl_neo4j::CypherStatement]) {
    for stmt in statements {
        let query = stmt.render_inline();
        if !query.trim().is_empty() {
            graph
                .run(Query::new(query.clone()))
                .await
                .unwrap_or_else(|_| panic!("Failed to execute: {}", query));
        }
    }
}

/// Helper to get a test HEDL document with users and posts.
/// Uses shared fixtures from hedl-test for consistency.
fn create_test_document() -> Document {
    fixtures::with_references()
}

// ============================================================================
// Basic Integration Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_execute_generated_cypher() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = create_test_document();
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    // Execute the generated Cypher statements
    execute_statements(&graph, &statements).await;

    // Verify nodes were created
    let mut result = graph
        .execute(Query::new(
            "MATCH (u:User) RETURN count(u) as count".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 2, "Expected 2 User nodes");

    let mut result = graph
        .execute(Query::new(
            "MATCH (p:Post) RETURN count(p) as count".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(
        count, 3,
        "Expected 3 Post nodes (from fixtures::with_references)"
    );

    cleanup(&graph).await;
}

#[tokio::test]
#[serial]
async fn test_relationships_created() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = create_test_document();
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    // Execute the generated Cypher statements
    execute_statements(&graph, &statements).await;

    // Verify relationships were created
    let mut result = graph
        .execute(Query::new(
            "MATCH (:Post)-[r:AUTHOR]->(:User) RETURN count(r) as count".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(
        count, 3,
        "Expected 3 AUTHOR relationships (from fixtures::with_references)"
    );

    cleanup(&graph).await;
}

#[tokio::test]
#[serial]
async fn test_node_properties() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = create_test_document();
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    // Execute the generated Cypher statements
    execute_statements(&graph, &statements).await;

    // Verify Alice's properties (fixtures::with_references has [id, name])
    let mut result = graph
        .execute(Query::new(
            "MATCH (u:User {_hedl_id: 'alice'}) RETURN u".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let node: Neo4rsNode = row.get("u").expect("No node");

    let name: String = node.get("name").expect("No name");
    assert_eq!(name, "Alice Smith");

    cleanup(&graph).await;
}

// ============================================================================
// Round-trip Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_roundtrip_simple() {
    let graph = connect().await;
    cleanup(&graph).await;

    // Create a simple document
    let mut root = BTreeMap::new();
    root.insert(
        "items".to_string(),
        Item::List(MatrixList::with_rows(
            "Item",
            vec!["id".to_string(), "name".to_string(), "count".to_string()],
            vec![Node {
                type_name: "Item".to_string(),
                id: "item1".to_string(),
                fields: vec![
                    Value::String("item1".to_string()),
                    Value::String("First Item".to_string()),
                    Value::Int(42),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
        )),
    );

    let mut structs = BTreeMap::new();
    structs.insert(
        "Item".to_string(),
        vec!["id".to_string(), "name".to_string(), "count".to_string()],
    );

    let original = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs,
        nests: BTreeMap::new(),
        root,
    };

    // Export to Cypher and execute
    let statements = to_cypher_statements(&original, &ToCypherConfig::default())
        .expect("Failed to generate Cypher");
    execute_statements(&graph, &statements).await;

    // Query back and convert to HEDL
    let mut result = graph
        .execute(Query::new("MATCH (n:Item) RETURN n".to_string()))
        .await
        .expect("Failed to query");

    let mut records = Vec::new();
    while let Some(row) = result.next().await.expect("Failed to get row") {
        let node: Neo4rsNode = row.get("n").expect("No node");
        let id: String = node.get("_hedl_id").expect("No id");
        let name: String = node.get("name").expect("No name");
        let count: i64 = node.get("count").expect("No count");

        records.push(Neo4jRecord::new(
            Neo4jNode::new("Item", &id)
                .with_property("name", CypherValue::String(name))
                .with_property("count", CypherValue::Int(count)),
        ));
    }

    let imported =
        from_neo4j_records(&records, &FromNeo4jConfig::default()).expect("Failed to import");

    // Verify the imported document matches key aspects
    assert_eq!(imported.version, (1, 0));

    // Check that we have an Item type (key is lowercase label)
    if let Some(Item::List(list)) = imported.root.get("item") {
        assert_eq!(list.type_name, "Item");
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.rows[0].id, "item1");
    } else {
        panic!(
            "Expected item list in imported document, got keys: {:?}",
            imported.root.keys().collect::<Vec<_>>()
        );
    }

    cleanup(&graph).await;
}

// ============================================================================
// Value Type Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_all_value_types() {
    let graph = connect().await;
    cleanup(&graph).await;

    // Create document with various value types
    let mut root = BTreeMap::new();
    root.insert(
        "data".to_string(),
        Item::List(MatrixList::with_rows(
            "Data",
            vec![
                "id".to_string(),
                "str_val".to_string(),
                "int_val".to_string(),
                "float_val".to_string(),
                "bool_val".to_string(),
                "null_val".to_string(),
            ],
            vec![Node {
                type_name: "Data".to_string(),
                id: "d1".to_string(),
                fields: vec![
                    Value::String("d1".to_string()),
                    Value::String("hello".to_string()),
                    Value::Int(42),
                    Value::Float(3.25),
                    Value::Bool(true),
                    Value::Null,
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
        )),
    );

    let mut structs = BTreeMap::new();
    structs.insert(
        "Data".to_string(),
        vec![
            "id".to_string(),
            "str_val".to_string(),
            "int_val".to_string(),
            "float_val".to_string(),
            "bool_val".to_string(),
            "null_val".to_string(),
        ],
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs,
        nests: BTreeMap::new(),
        root,
    };

    // Export and execute
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");
    execute_statements(&graph, &statements).await;

    // Query and verify all types
    let mut result = graph
        .execute(Query::new(
            "MATCH (d:Data {_hedl_id: 'd1'}) RETURN d".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let node: Neo4rsNode = row.get("d").expect("No node");

    let str_val: String = node.get("str_val").expect("No str_val");
    assert_eq!(str_val, "hello");

    let int_val: i64 = node.get("int_val").expect("No int_val");
    assert_eq!(int_val, 42);

    let float_val: f64 = node.get("float_val").expect("No float_val");
    assert!((float_val - 3.25).abs() < 0.001);

    let bool_val: bool = node.get("bool_val").expect("No bool_val");
    assert!(bool_val);

    // null_val should be absent or null
    let null_val: Option<String> = node.get("null_val").ok();
    assert!(null_val.is_none());

    cleanup(&graph).await;
}

#[tokio::test]
#[serial]
async fn test_special_characters() {
    let graph = connect().await;
    cleanup(&graph).await;

    // Create document with special characters
    let mut root = BTreeMap::new();
    root.insert(
        "items".to_string(),
        Item::List(MatrixList::with_rows(
            "Item",
            vec!["id".to_string(), "text".to_string()],
            vec![Node {
                type_name: "Item".to_string(),
                id: "special".to_string(),
                fields: vec![
                    Value::String("special".to_string()),
                    Value::String(
                        "Hello 'World' with \"quotes\" and \\ backslash\nnewline\ttab".to_string(),
                    ),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
        )),
    );

    let mut structs = BTreeMap::new();
    structs.insert(
        "Item".to_string(),
        vec!["id".to_string(), "text".to_string()],
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs,
        nests: BTreeMap::new(),
        root,
    };

    // Export and execute
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");
    execute_statements(&graph, &statements).await;

    // Query and verify special characters preserved
    let mut result = graph
        .execute(Query::new(
            "MATCH (i:Item {_hedl_id: 'special'}) RETURN i.text as text".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let text: String = row.get("text").expect("No text");
    assert!(text.contains("'World'"));
    assert!(text.contains("\"quotes\""));
    assert!(text.contains("\\"));
    assert!(text.contains("\n"));
    assert!(text.contains("\t"));

    cleanup(&graph).await;
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_config_use_create() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = create_test_document();

    let config = ToCypherConfig {
        use_merge: false, // Use CREATE instead of MERGE
        ..Default::default()
    };

    let statements = to_cypher_statements(&doc, &config).expect("Failed to generate Cypher");

    // Verify CREATE is used (render one statement to check)
    let cypher = to_cypher(&doc, &config).expect("Failed to generate Cypher");
    assert!(cypher.contains("CREATE"), "Expected CREATE statements");
    assert!(!cypher.contains("MERGE"), "Did not expect MERGE statements");

    // Execute and verify it works
    execute_statements(&graph, &statements).await;

    cleanup(&graph).await;
}

#[tokio::test]
#[serial]
async fn test_config_no_constraints() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = create_test_document();

    let config = ToCypherConfig {
        create_constraints: false,
        ..Default::default()
    };

    let cypher = to_cypher(&doc, &config).expect("Failed to generate Cypher");

    // Verify no CONSTRAINT statements
    assert!(
        !cypher.contains("CONSTRAINT"),
        "Did not expect CONSTRAINT statements"
    );

    cleanup(&graph).await;
}

#[tokio::test]
#[serial]
async fn test_config_generic_relationship_naming() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = create_test_document();

    let config = ToCypherConfig {
        reference_naming: RelationshipNaming::Generic,
        ..Default::default()
    };

    let statements = to_cypher_statements(&doc, &config).expect("Failed to generate Cypher");

    // Execute
    execute_statements(&graph, &statements).await;

    // Verify REFERENCES relationships
    let mut result = graph
        .execute(Query::new(
            "MATCH ()-[r:REFERENCES]->() RETURN count(r) as count".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(
        count, 3,
        "Expected 3 REFERENCES relationships (from fixtures::with_references)"
    );

    cleanup(&graph).await;
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
#[serial]
async fn test_empty_document() {
    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root: BTreeMap::new(),
    };

    let cypher = hedl_to_cypher(&doc).expect("Failed to generate Cypher");

    // Should produce minimal or empty output
    let non_comment_lines: Vec<&str> = cypher
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect();

    assert!(
        non_comment_lines.is_empty() || non_comment_lines.iter().all(|l| l.is_empty()),
        "Empty document should produce no statements"
    );
}

#[tokio::test]
#[serial]
async fn test_unicode_identifiers() {
    let graph = connect().await;
    cleanup(&graph).await;

    let mut root = BTreeMap::new();
    root.insert(
        "items".to_string(),
        Item::List(MatrixList::with_rows(
            "Item",
            vec!["id".to_string(), "name".to_string()],
            vec![Node {
                type_name: "Item".to_string(),
                id: "unicode_test".to_string(),
                fields: vec![
                    Value::String("unicode_test".to_string()),
                    Value::String("Test with unicode".to_string()),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
        )),
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    // Execute
    execute_statements(&graph, &statements).await;

    // Verify node exists
    let mut result = graph
        .execute(Query::new(
            "MATCH (i:Item {_hedl_id: 'unicode_test'}) RETURN i.name as name".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let name: String = row.get("name").expect("No name");
    assert_eq!(name, "Test with unicode");

    cleanup(&graph).await;
}

#[tokio::test]
#[serial]
async fn test_large_batch() {
    let graph = connect().await;
    cleanup(&graph).await;

    // Create document with many nodes
    let mut rows = Vec::new();
    for i in 0..100 {
        rows.push(Node {
            type_name: "Item".to_string(),
            id: format!("item_{}", i),
            fields: vec![
                Value::String(format!("item_{}", i)),
                Value::String(format!("Item number {}", i)),
                Value::Int(i),
            ],
            children: BTreeMap::new(),
            child_count: None,
        });
    }

    let mut root = BTreeMap::new();
    root.insert(
        "items".to_string(),
        Item::List(MatrixList::with_rows(
            "Item",
            vec!["id".to_string(), "name".to_string(), "index".to_string()],
            rows,
        )),
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    // Execute
    execute_statements(&graph, &statements).await;

    // Verify all nodes created
    let mut result = graph
        .execute(Query::new(
            "MATCH (i:Item) RETURN count(i) as count".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 100, "Expected 100 Item nodes");

    cleanup(&graph).await;
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_missing_schema_error() {
    // Create document with matrix list but no schema in struct
    let mut root = BTreeMap::new();
    root.insert(
        "items".to_string(),
        Item::List(MatrixList::with_rows(
            "UnknownType",
            vec![], // Empty schema
            vec![Node {
                type_name: "UnknownType".to_string(),
                id: "item1".to_string(),
                fields: vec![Value::String("test".to_string())],
                children: BTreeMap::new(),
                child_count: None,
            }],
        )),
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // This should still work - schema comes from MatrixList
    let result = hedl_to_cypher(&doc);
    // May succeed or fail depending on implementation
    // Just verify it doesn't panic
    let _ = result;
}

// ============================================================================
// NEST Hierarchy Tests
// ============================================================================

#[tokio::test]
#[serial]
async fn test_nest_relationships() {
    let graph = connect().await;
    cleanup(&graph).await;

    // Create document with NEST hierarchy (User > Post)
    let mut alice_children = BTreeMap::new();
    alice_children.insert(
        "posts".to_string(),
        vec![
            Node {
                type_name: "Post".to_string(),
                id: "p1".to_string(),
                fields: vec![
                    Value::String("p1".to_string()),
                    Value::String("Alice's first post".to_string()),
                ],
                children: BTreeMap::new(),
                child_count: None,
            },
            Node {
                type_name: "Post".to_string(),
                id: "p2".to_string(),
                fields: vec![
                    Value::String("p2".to_string()),
                    Value::String("Alice's second post".to_string()),
                ],
                children: BTreeMap::new(),
                child_count: None,
            },
        ],
    );

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList::with_rows(
            "User",
            vec!["id".to_string(), "name".to_string()],
            vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: vec![
                    Value::String("alice".to_string()),
                    Value::String("Alice".to_string()),
                ],
                children: alice_children,
                child_count: None,
            }],
        )),
    );

    let mut nests = BTreeMap::new();
    nests.insert("User".to_string(), "Post".to_string());

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests,
        root,
    };

    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    // Execute
    execute_statements(&graph, &statements).await;

    // Verify NEST relationships (HAS_POST)
    let mut result = graph
        .execute(Query::new(
            "MATCH (u:User)-[r:HAS_POST]->(p:Post) RETURN count(r) as count".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 2, "Expected 2 HAS_POST relationships");

    // Verify order is preserved
    let mut result = graph
        .execute(Query::new(
            "MATCH (u:User {_hedl_id: 'alice'})-[r:HAS_POST]->(p:Post) \
         RETURN p._hedl_id as id, r._nest_order as order \
         ORDER BY r._nest_order"
                .to_string(),
        ))
        .await
        .expect("Failed to query");

    let row1 = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let id1: String = row1.get("id").expect("No id");
    let order1: i64 = row1.get("order").expect("No order");
    assert_eq!(id1, "p1");
    assert_eq!(order1, 0);

    let row2 = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    let id2: String = row2.get("id").expect("No id");
    let order2: i64 = row2.get("order").expect("No order");
    assert_eq!(id2, "p2");
    assert_eq!(order2, 1);

    cleanup(&graph).await;
}

// ============================================================================
// Import Tests
// ============================================================================

#[test]
fn test_import_from_records() {
    // Test building records and importing
    let records = vec![
        Neo4jRecord::new(
            Neo4jNode::new("User", "alice")
                .with_property("name", CypherValue::String("Alice".to_string()))
                .with_property(
                    "email",
                    CypherValue::String("alice@example.com".to_string()),
                ),
        ),
        Neo4jRecord::new(
            Neo4jNode::new("User", "bob")
                .with_property("name", CypherValue::String("Bob".to_string())),
        ),
    ];

    let doc = from_neo4j_records(&records, &FromNeo4jConfig::default()).expect("Failed to import");

    assert_eq!(doc.version, (1, 0));

    // Check that we have users (key is lowercase label)
    if let Some(Item::List(list)) = doc.root.get("user") {
        assert_eq!(list.type_name, "User");
        assert_eq!(list.rows.len(), 2);
    } else {
        panic!(
            "Expected user list, got keys: {:?}",
            doc.root.keys().collect::<Vec<_>>()
        );
    }
}

#[test]
fn test_import_with_relationships() {
    let records = vec![
        Neo4jRecord::new(
            Neo4jNode::new("User", "alice")
                .with_property("name", CypherValue::String("Alice".to_string())),
        )
        .with_relationship(Neo4jRelationship::new(
            "User", "alice", "HAS_POST", "Post", "p1",
        )),
        Neo4jRecord::new(
            Neo4jNode::new("Post", "p1")
                .with_property("title", CypherValue::String("Hello".to_string())),
        ),
    ];

    let config = FromNeo4jConfig {
        infer_nests: true,
        ..Default::default()
    };

    let doc = from_neo4j_records(&records, &config).expect("Failed to import");

    // With infer_nests=true, should have NEST relationship
    assert!(doc.nests.contains_key("User"));
    assert_eq!(doc.nests.get("User"), Some(&"Post".to_string()));
}

// ============================================================================
// Comprehensive Fixture-Based Tests
// ============================================================================

/// Test user_list fixture with full Neo4j round-trip.
#[tokio::test]
#[serial]
async fn test_fixture_user_list() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = fixtures::user_list();
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    execute_statements(&graph, &statements).await;

    // Verify 3 users created
    let mut result = graph
        .execute(Query::new(
            "MATCH (u:User) RETURN count(u) as count".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 3, "Expected 3 User nodes from fixtures::user_list()");

    // Verify alice's properties
    let mut result = graph
        .execute(Query::new(
            "MATCH (u:User {_hedl_id: 'alice'}) RETURN u.name as name, u.email as email"
                .to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result.next().await.expect("No row").expect("No result");
    let name: String = row.get("name").expect("No name");
    let email: String = row.get("email").expect("No email");
    assert_eq!(name, "Alice Smith");
    assert_eq!(email, "alice@example.com");

    cleanup(&graph).await;
}

/// Test mixed_type_list fixture - verifies all value types.
#[tokio::test]
#[serial]
async fn test_fixture_mixed_types() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = fixtures::mixed_type_list();
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    execute_statements(&graph, &statements).await;

    // Verify items created
    let mut result = graph
        .execute(Query::new(
            "MATCH (i:Item {_hedl_id: 'item1'}) RETURN i".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result.next().await.expect("No row").expect("No result");
    let node: Neo4rsNode = row.get("i").expect("No node");

    // Verify different value types
    let name: String = node.get("name").expect("No name");
    assert_eq!(name, "Widget");

    let count: i64 = node.get("count").expect("No count");
    assert_eq!(count, 100);

    let price: f64 = node.get("price").expect("No price");
    assert!((price - 9.99).abs() < 0.001);

    let active: bool = node.get("active").expect("No active");
    assert!(active);

    // Verify null handling (item2.notes is null)
    let mut result = graph
        .execute(Query::new(
            "MATCH (i:Item {_hedl_id: 'item2'}) RETURN i.notes as notes".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result.next().await.expect("No row").expect("No result");
    let notes: Option<String> = row.get("notes").ok();
    assert!(notes.is_none(), "Expected null notes");

    cleanup(&graph).await;
}

/// Test with_nest fixture - verifies NEST hierarchy relationships.
#[tokio::test]
#[serial]
async fn test_fixture_with_nest() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = fixtures::with_nest();
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    execute_statements(&graph, &statements).await;

    // Verify users and posts created
    let mut result = graph
        .execute(Query::new(
            "MATCH (u:User) RETURN count(u) as count".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 2, "Expected 2 User nodes");

    // Verify HAS_POST relationships from NEST
    let mut result = graph
        .execute(Query::new(
            "MATCH (u:User)-[r:HAS_POST]->(p:Post) RETURN count(r) as count".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 3, "Expected 3 HAS_POST relationships");

    // Verify alice has 2 posts with correct order
    let mut result = graph
        .execute(Query::new(
            "MATCH (u:User {_hedl_id: 'alice'})-[r:HAS_POST]->(p:Post) \
         RETURN p._hedl_id as id ORDER BY r._nest_order"
                .to_string(),
        ))
        .await
        .expect("Failed to query");

    let row1 = result.next().await.expect("No row").expect("No result");
    let id1: String = row1.get("id").expect("No id");
    assert_eq!(id1, "post1");

    let row2 = result.next().await.expect("No row").expect("No result");
    let id2: String = row2.get("id").expect("No id");
    assert_eq!(id2, "post2");

    cleanup(&graph).await;
}

/// Test that child nodes use schema column names, not generic field_N names.
#[tokio::test]
#[serial]
async fn test_child_node_property_names() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = fixtures::with_nest();
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    execute_statements(&graph, &statements).await;

    // Verify that Post nodes have 'title' property, not 'field_1'
    let mut result = graph
        .execute(Query::new(
            "MATCH (p:Post {_hedl_id: 'post1'}) RETURN p.title as title".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result.next().await.expect("No row").expect("No result");
    let title: String = row
        .get("title")
        .expect("Expected 'title' property on Post node");
    assert_eq!(title, "Alice's first post");

    // Verify that 'field_1' property does NOT exist
    let mut result = graph
        .execute(Query::new(
            "MATCH (p:Post {_hedl_id: 'post1'}) RETURN p.field_1 as field_1".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result.next().await.expect("No row").expect("No result");
    let field_1: Option<String> = row.get("field_1").ok();
    assert!(
        field_1.is_none(),
        "Post node should not have generic 'field_1' property"
    );

    cleanup(&graph).await;
}

/// Test deep_nest fixture - verifies 3-level NEST hierarchy.
#[tokio::test]
#[serial]
async fn test_fixture_deep_nest() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = fixtures::deep_nest();
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    execute_statements(&graph, &statements).await;

    // Verify Organization nodes created
    let mut result = graph
        .execute(Query::new(
            "MATCH (o:Organization) RETURN count(o) as count".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 1, "Expected 1 Organization node");

    // Verify Department nodes created
    let mut result = graph
        .execute(Query::new(
            "MATCH (d:Department) RETURN count(d) as count".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 1, "Expected 1 Department node");

    // Verify Employee nodes created
    let mut result = graph
        .execute(Query::new(
            "MATCH (e:Employee) RETURN count(e) as count".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 2, "Expected 2 Employee nodes");

    // Verify Organization -> Department relationship (HAS_DEPARTMENT)
    let mut result = graph
        .execute(Query::new(
            "MATCH (o:Organization)-[r:HAS_DEPARTMENT]->(d:Department) RETURN count(r) as count"
                .to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 1, "Expected 1 HAS_DEPARTMENT relationship");

    // Verify Department -> Employee relationship (HAS_EMPLOYEE)
    let mut result = graph
        .execute(Query::new(
            "MATCH (d:Department)-[r:HAS_EMPLOYEE]->(e:Employee) RETURN count(r) as count"
                .to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 2, "Expected 2 HAS_EMPLOYEE relationships");

    // Verify full path traversal works
    let mut result = graph.execute(Query::new(
        "MATCH (o:Organization {_hedl_id: 'acme'})-[:HAS_DEPARTMENT]->(d:Department)-[:HAS_EMPLOYEE]->(e:Employee) \
         RETURN e._hedl_id as id ORDER BY e._hedl_id".to_string()
    )).await.expect("Failed to query");

    let row1 = result.next().await.expect("No row").expect("No result");
    let id1: String = row1.get("id").expect("No id");
    assert_eq!(id1, "emp1");

    let row2 = result.next().await.expect("No row").expect("No result");
    let id2: String = row2.get("id").expect("No id");
    assert_eq!(id2, "emp2");

    cleanup(&graph).await;
}

/// Test comprehensive fixture - covers all HEDL features.
#[tokio::test]
#[serial]
async fn test_fixture_comprehensive() {
    let graph = connect().await;
    cleanup(&graph).await;

    let doc = fixtures::comprehensive();
    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    execute_statements(&graph, &statements).await;

    // Verify users (with NEST to posts)
    let mut result = graph
        .execute(Query::new(
            "MATCH (u:User) RETURN count(u) as count".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 2, "Expected 2 User nodes");

    // Verify posts (nested under users via NEST)
    let mut result = graph
        .execute(Query::new(
            "MATCH (p:Post) RETURN count(p) as count".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert!(count >= 1, "Expected at least 1 Post node");

    // Verify comments with references
    let mut result = graph
        .execute(Query::new(
            "MATCH (c:Comment)-[r:AUTHOR]->(u:User) RETURN count(r) as count".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 1, "Expected 1 Comment->User AUTHOR relationship");

    // Verify tags
    let mut result = graph
        .execute(Query::new(
            "MATCH (t:Tag) RETURN count(t) as count".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let count: i64 = row.get("count").expect("No count");
    assert_eq!(count, 2, "Expected 2 Tag nodes");

    cleanup(&graph).await;
}

/// Test edge_cases fixture - extreme values.
#[tokio::test]
#[serial]
async fn test_fixture_edge_cases() {
    let graph = connect().await;
    cleanup(&graph).await;

    // Create a list from edge case scalars for Neo4j testing
    let edge_doc = fixtures::edge_cases();

    // Extract edge case values and create a testable matrix list
    let mut root = BTreeMap::new();
    let mut list = MatrixList::new(
        "Edge",
        vec![
            "id".to_string(),
            "large_int".to_string(),
            "long_string".to_string(),
        ],
    );

    if let (Some(Item::Scalar(Value::Int(large))), Some(Item::Scalar(Value::String(long_str)))) = (
        edge_doc.root.get("large_int"),
        edge_doc.root.get("long_string"),
    ) {
        list.rows.push(Node {
            type_name: "Edge".to_string(),
            id: "edge1".to_string(),
            fields: vec![
                Value::String("edge1".to_string()),
                Value::Int(*large),
                Value::String(long_str.clone()),
            ],
            children: BTreeMap::new(),
            child_count: None,
        });
    }

    root.insert("edges".to_string(), Item::List(list));

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    execute_statements(&graph, &statements).await;

    // Verify large integer preserved
    let mut result = graph
        .execute(Query::new(
            "MATCH (e:Edge {_hedl_id: 'edge1'}) RETURN e.large_int as large_int".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result.next().await.expect("No row").expect("No result");
    let large_int: i64 = row.get("large_int").expect("No large_int");
    assert_eq!(large_int, i64::MAX);

    // Verify long string preserved
    let mut result = graph
        .execute(Query::new(
            "MATCH (e:Edge {_hedl_id: 'edge1'}) RETURN e.long_string as long_string".to_string(),
        ))
        .await
        .expect("Failed to query");

    let row = result.next().await.expect("No row").expect("No result");
    let long_string: String = row.get("long_string").expect("No long_string");
    assert_eq!(long_string.len(), 10000);
    assert!(long_string.chars().all(|c| c == 'x'));

    cleanup(&graph).await;
}

/// Test special_strings fixture - escaping and unicode.
#[tokio::test]
#[serial]
async fn test_fixture_special_strings() {
    let graph = connect().await;
    cleanup(&graph).await;

    let strings_doc = fixtures::special_strings();

    // Create a matrix list from special strings
    let mut root = BTreeMap::new();
    let mut list = MatrixList::new("StringTest", vec!["id".to_string(), "value".to_string()]);

    let test_cases = [
        ("quotes", "with_quotes"),
        ("backslash", "with_backslash"),
        ("newline", "with_newline"),
        ("tab", "with_tab"),
        ("unicode", "with_unicode"),
    ];

    for (id, key) in test_cases {
        if let Some(Item::Scalar(Value::String(s))) = strings_doc.root.get(key) {
            list.rows.push(Node {
                type_name: "StringTest".to_string(),
                id: id.to_string(),
                fields: vec![Value::String(id.to_string()), Value::String(s.clone())],
                children: BTreeMap::new(),
                child_count: None,
            });
        }
    }

    root.insert("strings".to_string(), Item::List(list));

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    execute_statements(&graph, &statements).await;

    // Verify quotes escaped correctly
    let mut result = graph
        .execute(Query::new(
            "MATCH (s:StringTest {_hedl_id: 'quotes'}) RETURN s.value as value".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let value: String = row.get("value").expect("No value");
    assert!(
        value.contains("\"hello\""),
        "Expected escaped quotes in: {}",
        value
    );

    // Verify unicode preserved
    let mut result = graph
        .execute(Query::new(
            "MATCH (s:StringTest {_hedl_id: 'unicode'}) RETURN s.value as value".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let value: String = row.get("value").expect("No value");
    assert!(value.contains("日本語"), "Expected Japanese characters");
    assert!(value.contains("🎉"), "Expected emoji");

    // Verify newline preserved
    let mut result = graph
        .execute(Query::new(
            "MATCH (s:StringTest {_hedl_id: 'newline'}) RETURN s.value as value".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let value: String = row.get("value").expect("No value");
    assert!(value.contains('\n'), "Expected newline in: {:?}", value);

    cleanup(&graph).await;
}

/// Test tensors fixture - stored as JSON strings in Neo4j.
#[tokio::test]
#[serial]
async fn test_fixture_tensors() {
    let graph = connect().await;
    cleanup(&graph).await;

    let tensors_doc = fixtures::tensors();

    // Create a matrix list with tensor values
    let mut root = BTreeMap::new();
    let mut list = MatrixList::new("TensorTest", vec!["id".to_string(), "tensor".to_string()]);

    if let Some(Item::Scalar(Value::Tensor(t))) = tensors_doc.root.get("tensor_1d") {
        list.rows.push(Node {
            type_name: "TensorTest".to_string(),
            id: "t1d".to_string(),
            fields: vec![Value::String("t1d".to_string()), Value::Tensor(t.clone())],
            children: BTreeMap::new(),
            child_count: None,
        });
    }

    if let Some(Item::Scalar(Value::Tensor(t))) = tensors_doc.root.get("tensor_2d") {
        list.rows.push(Node {
            type_name: "TensorTest".to_string(),
            id: "t2d".to_string(),
            fields: vec![Value::String("t2d".to_string()), Value::Tensor(t.clone())],
            children: BTreeMap::new(),
            child_count: None,
        });
    }

    root.insert("tensors".to_string(), Item::List(list));

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    execute_statements(&graph, &statements).await;

    // Verify 1D tensor stored as JSON string
    let mut result = graph
        .execute(Query::new(
            "MATCH (t:TensorTest {_hedl_id: 't1d'}) RETURN t.tensor as tensor".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let tensor: String = row.get("tensor").expect("No tensor");
    assert!(
        tensor.contains("1.0") || tensor.contains("1"),
        "Expected 1D tensor JSON"
    );
    assert!(
        tensor.contains("2.0") || tensor.contains("2"),
        "Expected 1D tensor JSON"
    );
    assert!(
        tensor.contains("3.0") || tensor.contains("3"),
        "Expected 1D tensor JSON"
    );

    // Verify 2D tensor stored as JSON string
    let mut result = graph
        .execute(Query::new(
            "MATCH (t:TensorTest {_hedl_id: 't2d'}) RETURN t.tensor as tensor".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let tensor: String = row.get("tensor").expect("No tensor");
    // Should be nested array structure
    assert!(tensor.contains("["), "Expected nested array in 2D tensor");

    cleanup(&graph).await;
}

/// Test expressions fixture - stored as strings in Neo4j.
#[tokio::test]
#[serial]
async fn test_fixture_expressions() {
    let graph = connect().await;
    cleanup(&graph).await;

    let expr_doc = fixtures::expressions();

    // Create a matrix list with expression values
    let mut root = BTreeMap::new();
    let mut list = MatrixList::new("ExprTest", vec!["id".to_string(), "expr".to_string()]);

    if let Some(Item::Scalar(Value::Expression(e))) = expr_doc.root.get("simple_expr") {
        list.rows.push(Node {
            type_name: "ExprTest".to_string(),
            id: "simple".to_string(),
            fields: vec![
                Value::String("simple".to_string()),
                Value::Expression(e.clone()),
            ],
            children: BTreeMap::new(),
            child_count: None,
        });
    }

    if let Some(Item::Scalar(Value::Expression(e))) = expr_doc.root.get("complex_expr") {
        list.rows.push(Node {
            type_name: "ExprTest".to_string(),
            id: "complex".to_string(),
            fields: vec![
                Value::String("complex".to_string()),
                Value::Expression(e.clone()),
            ],
            children: BTreeMap::new(),
            child_count: None,
        });
    }

    root.insert("expressions".to_string(), Item::List(list));

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let statements =
        to_cypher_statements(&doc, &ToCypherConfig::default()).expect("Failed to generate Cypher");

    execute_statements(&graph, &statements).await;

    // Verify simple expression preserved
    let mut result = graph
        .execute(Query::new(
            "MATCH (e:ExprTest {_hedl_id: 'simple'}) RETURN e.expr as expr".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let expr: String = row.get("expr").expect("No expr");
    assert_eq!(expr, "$(now())");

    // Verify complex expression preserved
    let mut result = graph
        .execute(Query::new(
            "MATCH (e:ExprTest {_hedl_id: 'complex'}) RETURN e.expr as expr".to_string(),
        ))
        .await
        .expect("Failed to query");
    let row = result.next().await.expect("No row").expect("No result");
    let expr: String = row.get("expr").expect("No expr");
    assert!(expr.contains("concat"));

    cleanup(&graph).await;
}
