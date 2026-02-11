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

//! Tests for the async Neo4j client.
//!
//! These tests require both the `async` and `integration-tests` features
//! and a running Neo4j instance at localhost:7687.

#![cfg(all(feature = "async", feature = "integration-tests"))]

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_neo4j::{AsyncNeo4jClient, ToCypherConfig};
use hedl_test::fixtures;
use neo4rs::{ConfigBuilder, Graph, Query};
use serial_test::serial;
use smallvec::SmallVec;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

/// Helper to create a Neo4j connection.
/// Returns None if Neo4j is not available (tests should skip gracefully).
async fn connect() -> Option<Arc<Graph>> {
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let config = ConfigBuilder::default()
        .uri(&uri)
        .user(&user)
        .password(&password)
        .build()
        .expect("Failed to build config");

    match Graph::connect(config) {
        Ok(graph) => Some(Arc::new(graph)),
        Err(_) => {
            eprintln!("Neo4j not available, skipping test");
            None
        }
    }
}

/// Macro to skip test if Neo4j is not available.
macro_rules! require_neo4j {
    ($graph:expr) => {
        match $graph {
            Some(g) => g,
            None => return, // Skip test gracefully
        }
    };
}

/// Helper to clean up test data.
async fn cleanup(graph: &Graph) {
    let _ = graph
        .run(Query::new("MATCH (n) DETACH DELETE n".to_string()))
        .await;
}

/// Helper to get node count by label.
async fn count_nodes(graph: &Graph, label: &str) -> i64 {
    let mut result = graph
        .execute(Query::new(format!(
            "MATCH (n:{label}) RETURN count(n) as count"
        )))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    row.get("count").expect("No count")
}

/// Helper to get relationship count by type.
async fn count_relationships(graph: &Graph, rel_type: &str) -> i64 {
    let mut result = graph
        .execute(Query::new(format!(
            "MATCH ()-[r:{rel_type}]->() RETURN count(r) as count"
        )))
        .await
        .expect("Failed to query");

    let row = result
        .next()
        .await
        .expect("Failed to get row")
        .expect("No row");
    row.get("count").expect("No count")
}

// ============================================================================
// Basic Connection Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_async_client_connection() {
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password).await;
    assert!(client.is_ok(), "Failed to connect: {:?}", client.err());
}

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_async_client_with_config() {
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let config = ToCypherConfig::new().with_create().without_constraints();
    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect")
        .with_config(config);

    // Just verify it doesn't panic
    drop(client);
}

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_async_client_with_retry_config() {
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect")
        .with_max_retries(5)
        .with_initial_retry_delay(Duration::from_millis(50));

    // Just verify it doesn't panic
    drop(client);
}

// ============================================================================
// Document Import Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_import_simple_document() {
    let graph = require_neo4j!(connect().await);
    cleanup(&graph).await;

    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect");

    // Create a simple document
    let doc = fixtures::user_list();

    // Import the document
    let result = client.import_document(&doc).await;
    assert!(result.is_ok(), "Failed to import: {:?}", result.err());

    // Verify data was imported
    let count = count_nodes(&graph, "User").await;
    assert!(count > 0, "Expected some User nodes");

    cleanup(&graph).await;
}

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_import_document_with_references() {
    let graph = require_neo4j!(connect().await);
    cleanup(&graph).await;

    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect");

    let doc = fixtures::with_references();

    // Import the document
    client
        .import_document(&doc)
        .await
        .expect("Failed to import");

    // Verify nodes were created
    let user_count = count_nodes(&graph, "User").await;
    assert_eq!(user_count, 2, "Expected 2 User nodes");

    let post_count = count_nodes(&graph, "Post").await;
    assert!(post_count > 0, "Expected some Post nodes");

    // Verify relationships were created
    let rel_count = count_relationships(&graph, "AUTHOR").await;
    assert!(rel_count > 0, "Expected some AUTHOR relationships");

    cleanup(&graph).await;
}

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_import_document_with_nest() {
    let graph = require_neo4j!(connect().await);
    cleanup(&graph).await;

    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect");

    // The with_nest() fixture creates Users with nested Posts:
    // - 2 User nodes (Alice and Bob)
    // - 3 Post nodes (Alice has 2 posts, Bob has 1 post)
    // - 3 HAS_POST relationships (User -> Post nesting)
    let doc = fixtures::with_nest();

    // Import the document
    client
        .import_document(&doc)
        .await
        .expect("Failed to import");

    // Verify User nodes were created
    let user_count = count_nodes(&graph, "User").await;
    assert_eq!(user_count, 2, "Expected 2 User nodes");

    // Verify Post nodes were created
    let post_count = count_nodes(&graph, "Post").await;
    assert_eq!(post_count, 3, "Expected 3 Post nodes");

    // Verify NEST relationships were created (HAS_POST for User -> Post nesting)
    // Relationship type is derived from child type name (Post), not the property key (posts)
    let nest_count = count_relationships(&graph, "HAS_POST").await;
    assert_eq!(nest_count, 3, "Expected 3 HAS_POST relationships");

    cleanup(&graph).await;
}

// ============================================================================
// Transactional Import Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_import_document_transactional() {
    let graph = require_neo4j!(connect().await);
    cleanup(&graph).await;

    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect");

    let doc = fixtures::user_list();

    // Import within transaction
    client
        .import_document_transactional(&doc)
        .await
        .expect("Failed to import transactionally");

    // Verify data was imported
    let count = count_nodes(&graph, "User").await;
    assert!(count > 0, "Expected some User nodes");

    cleanup(&graph).await;
}

// ============================================================================
// Concurrent Execution Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_concurrent_document_imports() {
    let graph = require_neo4j!(connect().await);
    cleanup(&graph).await;

    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect");

    let doc1 = fixtures::user_list();
    let doc2 = fixtures::with_references();

    // Import both documents concurrently
    let (r1, r2) = tokio::join!(client.import_document(&doc1), client.import_document(&doc2),);

    assert!(r1.is_ok(), "First import failed: {:?}", r1.err());
    assert!(r2.is_ok(), "Second import failed: {:?}", r2.err());

    // Verify all data was imported
    let user_count = count_nodes(&graph, "User").await;
    assert!(user_count > 0, "Expected User nodes from both imports");

    cleanup(&graph).await;
}

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_concurrent_imports_stress() {
    let graph = require_neo4j!(connect().await);
    cleanup(&graph).await;

    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = Arc::new(
        AsyncNeo4jClient::connect(&uri, &user, &password)
            .await
            .expect("Failed to connect"),
    );

    // Create multiple tasks that import different document types
    let mut tasks = vec![];
    for _ in 0..5 {
        let client = client.clone();
        let doc = fixtures::user_list();
        tasks.push(tokio::spawn(
            async move { client.import_document(&doc).await },
        ));
    }

    // Wait for all tasks
    for task in tasks {
        let result = task.await.expect("Task panicked");
        assert!(result.is_ok(), "Import failed: {:?}", result.err());
    }

    // Verify data was imported (should have nodes from all imports)
    let user_count = count_nodes(&graph, "User").await;
    assert!(user_count > 0, "Expected User nodes from all imports");

    cleanup(&graph).await;
}

// ============================================================================
// Raw Query Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_execute_raw_query() {
    let graph = require_neo4j!(connect().await);
    cleanup(&graph).await;

    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect");

    // Execute a raw query
    let result = client
        .execute_query("CREATE (n:TestNode {name: 'test'}) RETURN n")
        .await;
    assert!(
        result.is_ok(),
        "Failed to execute query: {:?}",
        result.err()
    );

    // Verify the node was created
    let count = count_nodes(&graph, "TestNode").await;
    assert_eq!(count, 1, "Expected 1 TestNode");

    cleanup(&graph).await;
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_import_empty_document() {
    let graph = require_neo4j!(connect().await);
    cleanup(&graph).await;

    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect");

    // Create an empty document
    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root: BTreeMap::new(),
    };

    // Import should succeed but do nothing
    let result = client.import_document(&doc).await;
    assert!(
        result.is_ok(),
        "Failed to import empty doc: {:?}",
        result.err()
    );

    cleanup(&graph).await;
}

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_import_large_batch() {
    let graph = require_neo4j!(connect().await);
    cleanup(&graph).await;

    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect");

    // Create a document with many nodes (to test batching)
    let mut nodes = vec![];
    for i in 0..1500 {
        nodes.push(Node {
            type_name: "BatchUser".to_string(),
            id: format!("user{i}"),
            fields: SmallVec::from_vec(vec![
                Value::String(format!("user{i}").into()),
                Value::String(format!("User {i}").into()),
            ]),
            children: None,
            child_count: 0,
        });
    }

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "BatchUser".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: nodes,
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Import should handle large batch
    let result = client.import_document(&doc).await;
    assert!(
        result.is_ok(),
        "Failed to import large batch: {:?}",
        result.err()
    );

    // Verify all nodes were created
    let count = count_nodes(&graph, "BatchUser").await;
    assert_eq!(count, 1500, "Expected all 1500 nodes");

    cleanup(&graph).await;
}

// ============================================================================
// Performance Comparison Tests
// ============================================================================

#[tokio::test]
#[ignore = "requires running Neo4j instance"]
#[serial]
async fn test_concurrent_vs_sequential_performance() {
    let graph = require_neo4j!(connect().await);
    cleanup(&graph).await;

    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| String::new());

    let client = AsyncNeo4jClient::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect");

    let doc = fixtures::comprehensive();

    // Test concurrent import
    let start = std::time::Instant::now();
    client
        .import_document(&doc)
        .await
        .expect("Concurrent import failed");
    let concurrent_duration = start.elapsed();

    cleanup(&graph).await;

    // For comparison, we'd need a sequential version, but we can at least
    // verify the concurrent version completes in reasonable time
    println!("Concurrent import took: {concurrent_duration:?}");
    assert!(
        concurrent_duration < Duration::from_secs(30),
        "Import took too long: {concurrent_duration:?}"
    );

    cleanup(&graph).await;
}
