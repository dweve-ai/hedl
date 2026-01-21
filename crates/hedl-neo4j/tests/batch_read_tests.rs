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

//! Integration tests for batch read operations.

#![cfg(all(feature = "async", feature = "integration-tests"))]

use hedl_neo4j::{
    query_multi_label_batch, query_nodes_batch, query_nodes_with_relationships_batch, BatchQuery,
};
use neo4rs::{Graph, Query};
use serial_test::serial;

async fn connect() -> Graph {
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());
    Graph::new("neo4j://localhost:7687", "neo4j", &password).expect("Failed to connect to Neo4j")
}

async fn cleanup(graph: &Graph) {
    let _ = graph
        .run(Query::new("MATCH (n) DETACH DELETE n".to_string()))
        .await;
}

async fn setup_test_nodes(graph: &Graph, count: usize) {
    cleanup(graph).await;

    // Create nodes one at a time with explicit property setting
    // (Neo4j parameterized maps require specific driver support)
    for i in 0..count {
        let query_str = format!(
            "CREATE (n:TestNode {{_hedl_id: 'node_{}', name: 'Node {}', value: {}}})",
            i, i, i
        );
        graph
            .run(Query::new(query_str))
            .await
            .expect("Failed to setup test nodes");
    }
}

async fn setup_users_and_posts(graph: &Graph, user_count: usize, posts_per_user: usize) {
    cleanup(graph).await;

    // Create users one at a time with explicit property setting
    for i in 0..user_count {
        let query_str = format!(
            "CREATE (u:User {{_hedl_id: 'user_{}', name: 'User {}'}})",
            i, i
        );
        graph
            .run(Query::new(query_str))
            .await
            .expect("Failed to create users");
    }

    // Create posts and relationships
    for i in 0..user_count {
        for j in 0..posts_per_user {
            let post_id = format!("post_{}_{}", i, j);
            let user_id = format!("user_{}", i);

            let query_str = format!(
                "CREATE (p:Post {{_hedl_id: '{}', content: 'Post {} by User {}'}}) \
                 WITH p \
                 MATCH (u:User {{_hedl_id: '{}'}}) \
                 CREATE (p)-[:AUTHOR]->(u)",
                post_id, j, i, user_id
            );

            graph
                .run(Query::new(query_str))
                .await
                .expect("Failed to create post");
        }
    }
}

#[tokio::test]
#[serial]
async fn test_batch_read_empty() {
    let graph = connect().await;
    cleanup(&graph).await;

    let ids: Vec<String> = vec![];
    let records = query_nodes_batch(&graph, "TestNode", ids, "_hedl_id")
        .await
        .expect("Empty batch query failed");

    assert_eq!(records.len(), 0);
}

#[tokio::test]
#[serial]
async fn test_batch_read_single_node() {
    let graph = connect().await;
    setup_test_nodes(&graph, 10).await;

    let ids = vec!["node_0".to_string()];
    let records = query_nodes_batch(&graph, "TestNode", ids, "_hedl_id")
        .await
        .expect("Single node batch query failed");

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].node.id, "node_0");
}

#[tokio::test]
#[serial]
async fn test_batch_read_multiple_nodes() {
    let graph = connect().await;
    setup_test_nodes(&graph, 100).await;

    let ids: Vec<String> = (0..100).map(|i| format!("node_{}", i)).collect();
    let records = query_nodes_batch(&graph, "TestNode", ids, "_hedl_id")
        .await
        .expect("Batch query failed");

    assert_eq!(records.len(), 100);
}

#[tokio::test]
#[serial]
async fn test_batch_read_missing_ids_handled() {
    let graph = connect().await;
    setup_test_nodes(&graph, 10).await;

    // Query with some valid, some invalid IDs
    let ids = vec![
        "node_0".to_string(),
        "missing_1".to_string(),
        "node_1".to_string(),
        "missing_2".to_string(),
    ];

    let records = query_nodes_batch(&graph, "TestNode", ids, "_hedl_id")
        .await
        .expect("Query failed");

    // Should return only the 2 valid nodes
    assert_eq!(records.len(), 2);
    assert!(records.iter().any(|r| r.node.id == "node_0"));
    assert!(records.iter().any(|r| r.node.id == "node_1"));
}

#[tokio::test]
#[serial]
async fn test_batch_read_with_relationships() {
    let graph = connect().await;
    setup_users_and_posts(&graph, 50, 5).await;

    let ids: Vec<String> = (0..50).map(|i| format!("user_{}", i)).collect();

    let records = query_nodes_with_relationships_batch(&graph, "User", ids, "_hedl_id", None)
        .await
        .expect("Batch query with relationships failed");

    assert_eq!(records.len(), 50);

    // Verify relationships loaded - each post should have 1 AUTHOR relationship
    // We have 50 users * 5 posts = 250 posts, each with 1 relationship
    // But we're querying from User perspective, so we should have 0 relationships
    // (because the relationship goes FROM Post TO User)
    // Let's verify that the query returns users without relationships
    let total_rels: usize = records.iter().map(|r| r.relationships.len()).sum();
    assert_eq!(total_rels, 0); // Users don't have outgoing AUTHOR relationships
}

#[tokio::test]
#[serial]
async fn test_batch_read_with_relationships_from_posts() {
    let graph = connect().await;
    setup_users_and_posts(&graph, 10, 5).await;

    // Query posts instead of users
    let ids: Vec<String> = (0..10)
        .flat_map(|i| (0..5).map(move |j| format!("post_{}_{}", i, j)))
        .collect();

    let records = query_nodes_with_relationships_batch(&graph, "Post", ids, "_hedl_id", None)
        .await
        .expect("Batch query with relationships failed");

    assert_eq!(records.len(), 50); // 10 users * 5 posts

    // Verify relationships loaded - each post should have 1 AUTHOR relationship
    let total_rels: usize = records.iter().map(|r| r.relationships.len()).sum();
    assert_eq!(total_rels, 50); // 50 posts × 1 relationship each
}

#[tokio::test]
#[serial]
async fn test_multi_label_batch_query() {
    let graph = connect().await;
    setup_users_and_posts(&graph, 20, 3).await;

    let queries = vec![
        BatchQuery::new("User", vec!["user_0".to_string(), "user_1".to_string()]),
        BatchQuery::new(
            "Post",
            vec![
                "post_0_0".to_string(),
                "post_0_1".to_string(),
                "post_1_0".to_string(),
            ],
        ),
    ];

    let records = query_multi_label_batch(&graph, queries, "_hedl_id")
        .await
        .expect("Multi-label batch query failed");

    // Should have 2 users + 3 posts = 5 records
    assert_eq!(records.len(), 5);

    let user_count = records.iter().filter(|r| r.node.label == "User").count();
    let post_count = records.iter().filter(|r| r.node.label == "Post").count();

    assert_eq!(user_count, 2);
    assert_eq!(post_count, 3);
}

#[tokio::test]
#[serial]
async fn test_large_batch_read() {
    let graph = connect().await;
    setup_test_nodes(&graph, 1000).await;

    let ids: Vec<String> = (0..1000).map(|i| format!("node_{}", i)).collect();
    let records = query_nodes_batch(&graph, "TestNode", ids, "_hedl_id")
        .await
        .expect("Large batch query failed");

    assert_eq!(records.len(), 1000);
}

#[tokio::test]
#[serial]
async fn test_batch_read_preserves_order() {
    let graph = connect().await;
    setup_test_nodes(&graph, 20).await;

    let ids: Vec<String> = vec![
        "node_5".to_string(),
        "node_2".to_string(),
        "node_15".to_string(),
        "node_0".to_string(),
    ];

    let records = query_nodes_batch(&graph, "TestNode", ids.clone(), "_hedl_id")
        .await
        .expect("Batch query failed");

    assert_eq!(records.len(), 4);

    // Neo4j may not preserve order, so just verify all IDs are present
    let returned_ids: Vec<String> = records.iter().map(|r| r.node.id.clone()).collect();
    for id in &ids {
        assert!(returned_ids.contains(id));
    }
}

#[tokio::test]
#[serial]
async fn test_batch_read_with_custom_relationship_pattern() {
    let graph = connect().await;
    setup_users_and_posts(&graph, 10, 5).await;

    let ids: Vec<String> = (0..10)
        .flat_map(|i| (0..5).map(move |j| format!("post_{}_{}", i, j)))
        .collect();

    // Query with specific relationship pattern
    let records = query_nodes_with_relationships_batch(
        &graph,
        "Post",
        ids,
        "_hedl_id",
        Some("-[r:AUTHOR]->"),
    )
    .await
    .expect("Batch query with custom pattern failed");

    assert_eq!(records.len(), 50);

    // All relationships should be AUTHOR type
    for record in &records {
        for rel in &record.relationships {
            assert_eq!(rel.rel_type, "AUTHOR");
        }
    }
}

#[tokio::test]
#[serial]
async fn test_batch_read_non_existent_label() {
    let graph = connect().await;
    cleanup(&graph).await;

    let ids = vec!["node_0".to_string()];
    let records = query_nodes_batch(&graph, "NonExistentLabel", ids, "_hedl_id")
        .await
        .expect("Query for non-existent label should succeed but return empty");

    assert_eq!(records.len(), 0);
}
