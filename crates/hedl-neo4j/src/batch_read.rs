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

//! Batch read operations for efficient Neo4j queries.
//!
//! This module provides optimized batch query functions that use Cypher's UNWIND
//! operator to fetch multiple nodes in a single query, reducing network roundtrips
//! and achieving 5-10x or better throughput improvements.

use std::collections::BTreeMap;

use crate::cypher::escape::escape_identifier;
use crate::cypher::CypherValue;
use crate::error::{Neo4jError, Result};
use crate::mapping::Neo4jNode;

/// Configuration for batch read operations.
#[derive(Debug, Clone)]
pub struct BatchReadConfig {
    /// Maximum batch size for a single query (default: 1000)
    pub max_batch_size: usize,

    /// Number of parallel query streams (default: 4)
    pub parallel_streams: usize,

    /// Whether to prefetch relationships (default: true)
    pub prefetch_relationships: bool,

    /// ID property name (default: "_`hedl_id`")
    pub id_property: String,

    /// Whether to use index hints (default: true)
    pub use_index_hints: bool,
}

impl Default for BatchReadConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            parallel_streams: 4,
            prefetch_relationships: true,
            id_property: "_hedl_id".to_string(),
            use_index_hints: true,
        }
    }
}

impl BatchReadConfig {
    /// Create a new batch read configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum batch size.
    #[must_use]
    pub fn with_max_batch_size(mut self, size: usize) -> Self {
        self.max_batch_size = size;
        self
    }

    /// Set the number of parallel streams.
    #[must_use]
    pub fn with_parallel_streams(mut self, streams: usize) -> Self {
        self.parallel_streams = streams;
        self
    }

    /// Set whether to prefetch relationships.
    #[must_use]
    pub fn with_prefetch_relationships(mut self, prefetch: bool) -> Self {
        self.prefetch_relationships = prefetch;
        self
    }

    /// Set the ID property name.
    pub fn with_id_property(mut self, property: impl Into<String>) -> Self {
        self.id_property = property.into();
        self
    }

    /// Set whether to use index hints.
    #[must_use]
    pub fn with_index_hints(mut self, use_hints: bool) -> Self {
        self.use_index_hints = use_hints;
        self
    }
}

/// A batch query specification for multi-label queries.
#[derive(Debug, Clone)]
pub struct BatchQuery {
    /// The node label to query.
    pub label: String,
    /// The IDs to fetch.
    pub ids: Vec<String>,
    /// Optional relationship pattern for filtering (e.g., "-\[r:AUTHOR\]->").
    pub relationship_pattern: Option<String>,
}

impl BatchQuery {
    /// Create a new batch query.
    pub fn new(label: impl Into<String>, ids: Vec<String>) -> Self {
        Self {
            label: label.into(),
            ids,
            relationship_pattern: None,
        }
    }

    /// Set the relationship pattern.
    pub fn with_relationship_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.relationship_pattern = Some(pattern.into());
        self
    }
}

/// Build a batch query with optional index hints.
#[must_use]
pub fn build_batch_query(label: &str, id_property: &str, use_index_hint: bool) -> String {
    let index_hint = if use_index_hint {
        format!(" USING INDEX n:{label}({id_property})")
    } else {
        String::new()
    };

    format!(
        "UNWIND $ids AS id \
         MATCH (n:{} {{{}: id}}){} \
         RETURN n, id AS matched_id",
        escape_identifier(label),
        escape_identifier(id_property),
        index_hint
    )
}

/// Build a batch relationship query.
#[must_use]
pub fn build_batch_relationship_query(
    label: &str,
    id_property: &str,
    relationship_pattern: Option<&str>,
) -> String {
    let rel_pattern = relationship_pattern.unwrap_or("-[r]->");

    format!(
        "UNWIND $ids AS id \
         MATCH (from:{} {{{}: id}}){rel_pattern}(to) \
         RETURN from.{} AS from_id, \
                type(r) AS rel_type, \
                properties(r) AS rel_props, \
                labels(to)[0] AS to_label, \
                to.{} AS to_id",
        escape_identifier(label),
        escape_identifier(id_property),
        escape_identifier(id_property),
        escape_identifier(id_property)
    )
}

/// Parse Neo4j properties from a map.
#[cfg(feature = "async")]
pub fn parse_neo4j_properties(
    props_map: &BTreeMap<String, neo4rs::BoltType>,
) -> Result<BTreeMap<String, CypherValue>> {
    let mut result = BTreeMap::new();

    for (key, value) in props_map {
        result.insert(key.clone(), bolt_type_to_cypher_value(value)?);
    }

    Ok(result)
}

/// Convert `neo4rs::BoltType` to `CypherValue`.
#[cfg(feature = "async")]
fn bolt_type_to_cypher_value(bolt: &neo4rs::BoltType) -> Result<CypherValue> {
    use neo4rs::BoltType;

    match bolt {
        BoltType::Null(_) => Ok(CypherValue::Null),
        BoltType::Boolean(b) => Ok(CypherValue::Bool(b.value)),
        BoltType::Integer(i) => Ok(CypherValue::Int(i.value)),
        BoltType::Float(f) => Ok(CypherValue::Float(f.value)),
        BoltType::String(s) => Ok(CypherValue::String(s.value.clone())),
        BoltType::List(list) => {
            let values: Result<Vec<CypherValue>> =
                list.value.iter().map(bolt_type_to_cypher_value).collect();
            Ok(CypherValue::List(values?))
        }
        BoltType::Map(map) => {
            let mut result = BTreeMap::new();
            for (k, v) in &map.value {
                result.insert(k.value.clone(), bolt_type_to_cypher_value(v)?);
            }
            Ok(CypherValue::Map(result))
        }
        _ => Err(Neo4jError::TypeConversion(format!(
            "Unsupported BoltType: {bolt:?}"
        ))),
    }
}

/// Convert `neo4rs::Node` to `Neo4jNode`.
#[cfg(feature = "async")]
pub fn node_from_neo4rs(node: &neo4rs::Node, id_property: &str) -> Result<Neo4jNode> {
    let labels = node.labels();
    let label = (*labels
        .first()
        .ok_or_else(|| Neo4jError::RecordParseError("Node has no labels".to_string()))?)
    .to_string();

    // Collect all properties from the node
    let mut props = BTreeMap::new();
    for key in node.keys() {
        if let Ok(value) = node.get::<neo4rs::BoltType>(key) {
            props.insert(key.to_string(), value);
        }
    }

    let id_value = props
        .get(id_property)
        .ok_or_else(|| Neo4jError::MissingProperty {
            label: label.clone(),
            property: id_property.to_string(),
        })?;

    let id = match id_value {
        neo4rs::BoltType::String(s) => s.value.clone(),
        neo4rs::BoltType::Integer(i) => i.value.to_string(),
        _ => {
            return Err(Neo4jError::TypeConversion(format!(
                "ID property '{id_property}' must be string or integer"
            )))
        }
    };

    let properties = parse_neo4j_properties(&props)?;

    Ok(Neo4jNode {
        label,
        id,
        properties,
    })
}

/// Chunk a list of IDs into smaller batches.
#[must_use]
pub fn chunk_ids(ids: &[String], chunk_size: usize) -> Vec<Vec<String>> {
    if chunk_size == 0 {
        return vec![];
    }

    ids.chunks(chunk_size)
        .map(<[std::string::String]>::to_vec)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_read_config_default() {
        let config = BatchReadConfig::default();
        assert_eq!(config.max_batch_size, 1000);
        assert_eq!(config.parallel_streams, 4);
        assert!(config.prefetch_relationships);
        assert_eq!(config.id_property, "_hedl_id");
        assert!(config.use_index_hints);
    }

    #[test]
    fn test_batch_read_config_builder() {
        let config = BatchReadConfig::new()
            .with_max_batch_size(500)
            .with_parallel_streams(8)
            .with_prefetch_relationships(false)
            .with_id_property("custom_id")
            .with_index_hints(false);

        assert_eq!(config.max_batch_size, 500);
        assert_eq!(config.parallel_streams, 8);
        assert!(!config.prefetch_relationships);
        assert_eq!(config.id_property, "custom_id");
        assert!(!config.use_index_hints);
    }

    #[test]
    fn test_batch_query_new() {
        let query = BatchQuery::new("User", vec!["alice".to_string(), "bob".to_string()]);
        assert_eq!(query.label, "User");
        assert_eq!(query.ids.len(), 2);
        assert!(query.relationship_pattern.is_none());
    }

    #[test]
    fn test_batch_query_with_relationship_pattern() {
        let query = BatchQuery::new("User", vec!["alice".to_string()])
            .with_relationship_pattern("-[r:FOLLOWS]->");

        assert_eq!(
            query.relationship_pattern,
            Some("-[r:FOLLOWS]->".to_string())
        );
    }

    #[test]
    fn test_build_batch_query_with_hints() {
        let query = build_batch_query("User", "_hedl_id", true);
        assert!(query.contains("UNWIND $ids AS id"));
        assert!(query.contains("MATCH (n:User {_hedl_id: id})"));
        assert!(query.contains("USING INDEX n:User(_hedl_id)"));
        assert!(query.contains("RETURN n"));
    }

    #[test]
    fn test_build_batch_query_without_hints() {
        let query = build_batch_query("User", "_hedl_id", false);
        assert!(query.contains("UNWIND $ids AS id"));
        assert!(query.contains("MATCH (n:User {_hedl_id: id})"));
        assert!(!query.contains("USING INDEX"));
        assert!(query.contains("RETURN n"));
    }

    #[test]
    fn test_build_batch_relationship_query_default() {
        let query = build_batch_relationship_query("User", "_hedl_id", None);
        assert!(query.contains("UNWIND $ids AS id"));
        assert!(query.contains("MATCH (from:User {_hedl_id: id})-[r]->(to)"));
        assert!(query.contains("RETURN from._hedl_id AS from_id"));
        assert!(query.contains("type(r) AS rel_type"));
        assert!(query.contains("properties(r) AS rel_props"));
    }

    #[test]
    fn test_build_batch_relationship_query_custom_pattern() {
        let query = build_batch_relationship_query("User", "_hedl_id", Some("-[r:FOLLOWS]->"));
        assert!(query.contains("MATCH (from:User {_hedl_id: id})-[r:FOLLOWS]->(to)"));
    }

    #[test]
    fn test_chunk_ids_empty() {
        let ids: Vec<String> = vec![];
        let chunks = chunk_ids(&ids, 100);
        assert_eq!(chunks.len(), 0);
    }

    #[test]
    fn test_chunk_ids_single_chunk() {
        let ids: Vec<String> = (0..50).map(|i| format!("id_{}", i)).collect();
        let chunks = chunk_ids(&ids, 100);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].len(), 50);
    }

    #[test]
    fn test_chunk_ids_multiple_chunks() {
        let ids: Vec<String> = (0..250).map(|i| format!("id_{}", i)).collect();
        let chunks = chunk_ids(&ids, 100);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 100);
        assert_eq!(chunks[1].len(), 100);
        assert_eq!(chunks[2].len(), 50);
    }

    #[test]
    fn test_chunk_ids_exact_multiple() {
        let ids: Vec<String> = (0..200).map(|i| format!("id_{}", i)).collect();
        let chunks = chunk_ids(&ids, 100);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 100);
        assert_eq!(chunks[1].len(), 100);
    }

    #[test]
    fn test_chunk_ids_zero_size() {
        let ids: Vec<String> = vec!["a".to_string(), "b".to_string()];
        let chunks = chunk_ids(&ids, 0);
        assert_eq!(chunks.len(), 0);
    }
}
