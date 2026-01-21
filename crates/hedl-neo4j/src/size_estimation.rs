// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Note: Size estimation utilities for adaptive batching (future optimization).
#![allow(dead_code)]

//! Memory size estimation for Neo4j nodes and relationships.
//!
//! This module provides conservative estimates of in-memory size for nodes
//! and relationships to enable adaptive batch sizing.
//!
//! # Estimation Strategy
//!
//! Estimates are conservative (tend to underestimate) to avoid creating
//! batches that are too large. The estimation includes:
//!
//! - String data length (UTF-8 bytes)
//! - Collection overhead (Vec, `BTreeMap`)
//! - Pointer and metadata overhead
//! - Struct padding and alignment
//!
//! # Performance
//!
//! Size estimation is O(1) per primitive value and O(n) for collections,
//! where n is the number of elements. This is negligible compared to the
//! cost of Cypher generation and Neo4j execution.

use crate::cypher::CypherValue;
use crate::mapping::{Neo4jNode, Neo4jRelationship};
use std::mem;

/// Estimate the memory footprint of a `CypherValue` in bytes.
///
/// This is a conservative estimate that includes:
/// - String data length
/// - Collection overhead (Vec, `BTreeMap`)
/// - Pointer and metadata overhead
///
/// The estimate errs on the side of underestimation to avoid creating
/// batches that are too large.
///
/// # Examples
///
/// ```ignore
/// // Internal API - not exported from crate root
/// use hedl_neo4j::cypher::CypherValue;
/// use hedl_neo4j::size_estimation::estimate_cypher_value_size;
/// let value = CypherValue::String("hello".to_string());
/// let size = estimate_cypher_value_size(&value);
/// assert!(size >= 5); // At least the string length
/// ```
pub fn estimate_cypher_value_size(value: &CypherValue) -> usize {
    match value {
        CypherValue::Null => mem::size_of::<u8>(),
        CypherValue::Bool(_) => mem::size_of::<bool>(),
        CypherValue::Int(_) => mem::size_of::<i64>(),
        CypherValue::Float(_) => mem::size_of::<f64>(),
        CypherValue::String(s) => {
            // String data + heap allocation overhead
            s.len() + mem::size_of::<String>()
        }
        CypherValue::List(items) => {
            let items_size: usize = items.iter().map(estimate_cypher_value_size).sum();
            items_size + mem::size_of::<Vec<CypherValue>>()
        }
        CypherValue::Map(map) => {
            let entries_size: usize = map
                .iter()
                .map(|(k, v)| {
                    k.len() + estimate_cypher_value_size(v) + 32 // BTreeMap node overhead
                })
                .sum();
            entries_size + mem::size_of::<std::collections::BTreeMap<String, CypherValue>>()
        }
    }
}

/// Estimate the memory footprint of a Neo4j node in bytes.
///
/// Includes:
/// - Label and ID strings
/// - All property keys and values
/// - Struct and collection overhead
///
/// # Examples
///
/// ```ignore
/// // Internal API - not exported from crate root
/// use hedl_neo4j::mapping::Neo4jNode;
/// use hedl_neo4j::size_estimation::estimate_node_size;
/// let node = Neo4jNode::new("User", "alice");
/// let size = estimate_node_size(&node, "_hedl_id");
/// assert!(size > 0);
/// ```
pub fn estimate_node_size(node: &Neo4jNode, id_property: &str) -> usize {
    let mut size = 0;

    // Base strings
    size += node.label.len();
    size += node.id.len();
    size += id_property.len();

    // Properties
    for (key, value) in &node.properties {
        size += key.len();
        size += estimate_cypher_value_size(value);
    }

    // Struct and BTreeMap overhead
    size += mem::size_of::<Neo4jNode>();
    size += node.properties.len() * 50; // Conservative BTreeMap node overhead

    size
}

/// Estimate the memory footprint of a Neo4j relationship in bytes.
///
/// Includes:
/// - Label and ID strings for both endpoints
/// - Relationship type
/// - All property keys and values
/// - Struct and collection overhead
///
/// # Examples
///
/// ```ignore
/// // Internal API - not exported from crate root
/// use hedl_neo4j::mapping::Neo4jRelationship;
/// use hedl_neo4j::size_estimation::estimate_relationship_size;
/// let rel = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob");
/// let size = estimate_relationship_size(&rel);
/// assert!(size > 0);
/// ```
pub fn estimate_relationship_size(rel: &Neo4jRelationship) -> usize {
    let mut size = 0;

    // Base strings
    size += rel.from_label.len();
    size += rel.from_id.len();
    size += rel.rel_type.len();
    size += rel.to_label.len();
    size += rel.to_id.len();

    // Properties
    for (key, value) in &rel.properties {
        size += key.len();
        size += estimate_cypher_value_size(value);
    }

    // Struct and BTreeMap overhead
    size += mem::size_of::<Neo4jRelationship>();
    size += rel.properties.len() * 50;

    size
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::Neo4jNode;
    use std::collections::BTreeMap;

    #[test]
    fn test_estimate_cypher_value_size_primitives() {
        assert_eq!(
            estimate_cypher_value_size(&CypherValue::Null),
            mem::size_of::<u8>()
        );
        assert_eq!(
            estimate_cypher_value_size(&CypherValue::Bool(true)),
            mem::size_of::<bool>()
        );
        assert_eq!(
            estimate_cypher_value_size(&CypherValue::Int(42)),
            mem::size_of::<i64>()
        );
        assert_eq!(
            estimate_cypher_value_size(&CypherValue::Float(3.25)),
            mem::size_of::<f64>()
        );
    }

    #[test]
    fn test_estimate_cypher_value_size_string() {
        let s = CypherValue::String("hello".to_string());
        let size = estimate_cypher_value_size(&s);
        // Should be at least the string length
        assert!(size >= 5);
        // Should include String overhead
        assert!(size >= 5 + mem::size_of::<String>());
    }

    #[test]
    fn test_estimate_cypher_value_size_list() {
        let list = CypherValue::List(vec![
            CypherValue::Int(1),
            CypherValue::Int(2),
            CypherValue::Int(3),
        ]);
        let size = estimate_cypher_value_size(&list);
        // Should be at least 3 * i64 size + Vec overhead
        assert!(size >= 3 * mem::size_of::<i64>() + mem::size_of::<Vec<CypherValue>>());
    }

    #[test]
    fn test_estimate_cypher_value_size_map() {
        let mut map = BTreeMap::new();
        map.insert("key1".to_string(), CypherValue::Int(1));
        map.insert("key2".to_string(), CypherValue::Int(2));
        let value = CypherValue::Map(map);
        let size = estimate_cypher_value_size(&value);
        // Should include key lengths and values
        assert!(size >= 4 + 4 + 2 * mem::size_of::<i64>());
    }

    #[test]
    fn test_estimate_cypher_value_size_nested() {
        let nested = CypherValue::List(vec![
            CypherValue::Map({
                let mut map = BTreeMap::new();
                map.insert("a".to_string(), CypherValue::String("test".to_string()));
                map
            }),
            CypherValue::Int(42),
        ]);
        let size = estimate_cypher_value_size(&nested);
        // Should be reasonable for nested structure
        assert!(size > 0);
        assert!(size < 10000); // Sanity check
    }

    #[test]
    fn test_estimate_node_size_minimal() {
        let node = Neo4jNode::new("User", "alice");
        let size = estimate_node_size(&node, "_hedl_id");

        // Should be at least: "User" + "alice" + "_hedl_id" = 18 bytes
        assert!(size >= 18);
        // Should be reasonable (with overhead < 1000 bytes)
        assert!(size < 1000);
    }

    #[test]
    fn test_estimate_node_size_with_properties() {
        let mut node = Neo4jNode::new("User", "alice");
        node.properties
            .insert("name".to_string(), CypherValue::String("Alice".to_string()));
        node.properties
            .insert("age".to_string(), CypherValue::Int(30));

        let size = estimate_node_size(&node, "_hedl_id");

        // Should include property keys and values
        // label: "User" (4), id: "alice" (5), id_prop: "_hedl_id" (8)
        // prop1: "name" (4) + "Alice" (5), prop2: "age" (3) + i64
        assert!(size >= 18 + 4 + 5 + 3 + mem::size_of::<i64>());
        assert!(size < 2000);
    }

    #[test]
    fn test_estimate_node_size_large_string() {
        let mut node = Neo4jNode::new("Document", "doc1");
        let large_text = "x".repeat(10_000);
        node.properties
            .insert("content".to_string(), CypherValue::String(large_text));

        let size = estimate_node_size(&node, "_hedl_id");

        // Should be close to 10KB
        assert!(size >= 10_000);
        assert!(size < 12_000); // Allow for overhead
    }

    #[test]
    fn test_estimate_node_size_many_properties() {
        let mut node = Neo4jNode::new("ComplexNode", "node1");
        for i in 0..100 {
            node.properties.insert(
                format!("prop_{i}"),
                CypherValue::String(format!("value_{i}")),
            );
        }

        let size = estimate_node_size(&node, "_hedl_id");

        // Should account for many properties
        assert!(size > 1000); // Reasonable for 100 properties
        assert!(size < 50_000); // But not excessive
    }

    #[test]
    fn test_estimate_relationship_size_minimal() {
        let rel = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob");
        let size = estimate_relationship_size(&rel);

        // "User" (4) + "alice" (5) + "FOLLOWS" (7) + "User" (4) + "bob" (3) = 23
        assert!(size >= 23);
        assert!(size < 1000);
    }

    #[test]
    fn test_estimate_relationship_size_with_properties() {
        let mut rel = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob");
        rel.properties
            .insert("since".to_string(), CypherValue::String("2023".to_string()));
        rel.properties
            .insert("weight".to_string(), CypherValue::Float(0.8));

        let size = estimate_relationship_size(&rel);

        // Base + "since" (5) + "2023" (4) + "weight" (6) + f64
        assert!(size >= 23 + 5 + 4 + 6 + mem::size_of::<f64>());
        assert!(size < 2000);
    }

    #[test]
    fn test_size_estimation_consistency() {
        // Same node should always produce same size estimate
        let node1 = Neo4jNode::new("User", "alice");
        let size1 = estimate_node_size(&node1, "_hedl_id");
        let size2 = estimate_node_size(&node1, "_hedl_id");
        assert_eq!(size1, size2);

        // Clone should produce same size
        let node2 = node1.clone();
        let size3 = estimate_node_size(&node2, "_hedl_id");
        assert_eq!(size1, size3);
    }

    #[test]
    fn test_size_estimation_ordering() {
        // Larger nodes should have larger estimates
        let small_node = Neo4jNode::new("A", "1");
        let medium_node = {
            let mut node = Neo4jNode::new("ABC", "12345");
            node.properties
                .insert("x".to_string(), CypherValue::String("test".to_string()));
            node
        };
        let large_node = {
            let mut node = Neo4jNode::new("ABCDEFGH", "1234567890");
            for i in 0..10 {
                node.properties.insert(
                    format!("prop_{i}"),
                    CypherValue::String("large value".to_string()),
                );
            }
            node
        };

        let size_small = estimate_node_size(&small_node, "id");
        let size_medium = estimate_node_size(&medium_node, "id");
        let size_large = estimate_node_size(&large_node, "id");

        assert!(size_small < size_medium);
        assert!(size_medium < size_large);
    }
}
