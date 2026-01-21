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

//! Property-based tests for batch read operations.

#![cfg(all(feature = "async", feature = "integration-tests"))]

use hedl_neo4j::query_nodes_batch;
use neo4rs::{Graph, Query};
use proptest::prelude::*;
use serial_test::serial;

async fn connect() -> Graph {
    Graph::new("neo4j://localhost:7687", "neo4j", "password").expect("Failed to connect to Neo4j")
}

async fn cleanup(graph: &Graph) {
    let _ = graph
        .run(Query::new("MATCH (n) DETACH DELETE n".to_string()))
        .await;
}

async fn setup_test_data(graph: &Graph, count: usize) {
    cleanup(graph).await;

    // Use UNWIND with inline map syntax instead of parameterized rows
    // This avoids the BoltType conversion issue with complex parameters
    for i in 0..count {
        let query_str = format!(
            "CREATE (n:TestNode {{_hedl_id: 'node_{}', value: {}}})",
            i, i
        );
        graph
            .run(Query::new(query_str))
            .await
            .expect("Failed to create test node");
    }
}

async fn query_single_node(
    graph: &Graph,
    label: &str,
    id: &str,
    id_property: &str,
) -> Option<String> {
    let query_str = format!(
        "MATCH (n:{} {{{}: $id}}) RETURN n.{}",
        label, id_property, id_property
    );

    let mut query = Query::new(query_str);
    query = query.param("id", id);

    let mut result = graph.execute(query).await.ok()?;
    if let Ok(Some(row)) = result.next().await {
        row.get(&format!("n.{}", id_property)).ok()
    } else {
        None
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    #[serial]
    fn test_batch_query_correctness(
        num_nodes in 10usize..100,
        sample_size in 5usize..50
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let graph = runtime.block_on(async { connect().await });

        runtime.block_on(async {
            setup_test_data(&graph, num_nodes).await;

            // Take a sample of nodes to query
            let sample_size = sample_size.min(num_nodes);
            let ids: Vec<String> = (0..sample_size)
                .map(|i| format!("node_{}", i))
                .collect();

            // Sequential read
            let mut sequential_count = 0;
            for id in &ids {
                if query_single_node(&graph, "TestNode", id, "_hedl_id").await.is_some() {
                    sequential_count += 1;
                }
            }

            // Batch read
            let batch_results = query_nodes_batch(&graph, "TestNode", ids.clone(), "_hedl_id")
                .await
                .unwrap();

            // Results should match
            assert_eq!(sequential_count, batch_results.len());
            assert_eq!(sequential_count, sample_size);

            cleanup(&graph).await;
        });
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    #[serial]
    fn test_batch_query_with_missing_ids(
        num_nodes in 10usize..100,
        num_missing in 1usize..10
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let graph = runtime.block_on(async { connect().await });

        runtime.block_on(async {
            setup_test_data(&graph, num_nodes).await;

            // Mix valid and invalid IDs
            let mut ids: Vec<String> = (0..num_nodes.min(20))
                .map(|i| format!("node_{}", i))
                .collect();

            // Add missing IDs
            for i in 0..num_missing {
                ids.push(format!("missing_{}", i));
            }

            let batch_results = query_nodes_batch(&graph, "TestNode", ids.clone(), "_hedl_id")
                .await
                .unwrap();

            // Should return only valid nodes
            assert_eq!(batch_results.len(), num_nodes.min(20));

            cleanup(&graph).await;
        });
    }
}
