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

//! Convert Neo4j records to HEDL documents.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use std::collections::{BTreeMap, BTreeSet};

use crate::config::FromNeo4jConfig;
use crate::cypher::CypherValue;
use crate::error::{Neo4jError, Result};
use crate::mapping::{
    group_nodes_by_label, infer_nests_from_relationships, infer_schema_from_nodes,
    unflatten_properties, Neo4jNode, Neo4jRelationship,
};

/// Type alias for parent-child relationship mapping (label, id) -> Vec<(child_label, child_id, order)>
type ParentChildrenMap = BTreeMap<(String, String), Vec<(String, String, i64)>>;

/// Type alias for node reference mapping (label, id) -> Vec<(rel_type, target_label, target_id)>
type NodeRefsMap = BTreeMap<(String, String), Vec<(String, String, String)>>;
use crate::mapping::reference::Nest;

/// A Neo4j record containing a node and its relationships.
#[derive(Debug, Clone)]
pub struct Neo4jRecord {
    /// The node data.
    pub node: Neo4jNode,
    /// Outgoing relationships from this node.
    pub relationships: Vec<Neo4jRelationship>,
}

impl Neo4jRecord {
    /// Create a new record with a node.
    pub fn new(node: Neo4jNode) -> Self {
        Self {
            node,
            relationships: Vec::new(),
        }
    }

    /// Add a relationship to this record.
    pub fn with_relationship(mut self, rel: Neo4jRelationship) -> Self {
        self.relationships.push(rel);
        self
    }

    /// Add multiple relationships.
    pub fn with_relationships(mut self, rels: impl IntoIterator<Item = Neo4jRelationship>) -> Self {
        self.relationships.extend(rels);
        self
    }
}

/// Convert Neo4j records to a HEDL document.
pub fn from_neo4j_records(records: &[Neo4jRecord], config: &FromNeo4jConfig) -> Result<Document> {
    if records.is_empty() {
        return Ok(Document {
            version: config.version,
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root: BTreeMap::new(),
        });
    }

    // Extract all nodes
    let nodes: Vec<Neo4jNode> = records
        .iter()
        .filter(|r| !config.exclude_labels.contains(&r.node.label))
        .map(|r| r.node.clone())
        .collect();

    // Extract all relationships
    let relationships: Vec<Neo4jRelationship> = records
        .iter()
        .flat_map(|r| r.relationships.clone())
        .collect();

    // Infer NEST relationships
    let nests: Vec<Nest> = if config.infer_nests {
        infer_nests_from_relationships(&relationships)
    } else {
        vec![]
    };

    // Group nodes by label
    let grouped = group_nodes_by_label(&nodes);

    // Build struct definitions and matrix lists
    let mut structs = BTreeMap::new();
    let mut root = BTreeMap::new();

    for (label, label_nodes) in &grouped {
        // Infer schema from nodes
        let schema = infer_schema_from_nodes(label_nodes, &config.id_property);

        // Store struct definition
        structs.insert(label.clone(), schema.clone());

        // Build matrix list
        let hedl_nodes: Result<Vec<Node>> = label_nodes
            .iter()
            .map(|n| neo4j_node_to_hedl_node(n, &schema, config))
            .collect();

        let matrix_list = MatrixList {
            type_name: label.clone(),
            schema,
            rows: hedl_nodes?,
            count_hint: None,
        };

        // Use lowercase label as the key
        let key = label.to_lowercase();
        root.insert(key, Item::List(matrix_list));
    }

    // Attach children based on NEST relationships and HAS_* patterns
    attach_children(&mut root, &relationships, &nests, config)?;

    // Convert non-NEST relationships to references
    convert_relationships_to_references(&mut root, &relationships, &nests, config)?;

    // Convert Vec<Nest> to BTreeMap<String, String> for Document
    let nests_map: BTreeMap<String, String> = nests
        .iter()
        .map(|n| (n.parent.clone(), n.child.clone()))
        .collect();

    Ok(Document {
        version: config.version,
        aliases: BTreeMap::new(),
        structs,
        nests: nests_map,
        root,
    })
}

/// Convert Neo4j records to a HEDL document using default configuration.
pub fn neo4j_to_hedl(records: &[Neo4jRecord]) -> Result<Document> {
    from_neo4j_records(records, &FromNeo4jConfig::default())
}

/// Convert a Neo4jNode to a HEDL Node.
fn neo4j_node_to_hedl_node(
    neo4j_node: &Neo4jNode,
    schema: &[String],
    config: &FromNeo4jConfig,
) -> Result<Node> {
    // Filter excluded properties
    let mut properties = neo4j_node.properties.clone();
    for prop in &config.exclude_properties {
        properties.remove(prop);
    }
    properties.remove(&config.id_property);
    properties.remove(&config.type_property);

    // Unflatten properties if needed (handles dot-notation)
    let unflattened = unflatten_properties(&properties)?;

    // Build fields according to schema
    let mut fields = Vec::with_capacity(schema.len());

    for (i, column) in schema.iter().enumerate() {
        if i == 0 {
            // First column is the ID
            fields.push(Value::String(neo4j_node.id.clone()));
        } else if let Some(value) = unflattened.get(column) {
            fields.push(value.clone());
        } else {
            fields.push(Value::Null);
        }
    }

    Ok(Node {
        type_name: neo4j_node.label.clone(),
        id: neo4j_node.id.clone(),
        fields,
        children: BTreeMap::new(),
        child_count: None,
    })
}

/// Attach children to parent nodes based on NEST relationships.
fn attach_children(
    root: &mut BTreeMap<String, Item>,
    relationships: &[Neo4jRelationship],
    nests: &[Nest],
    _config: &FromNeo4jConfig,
) -> Result<()> {
    // Build a set of NEST relationship types for quick lookup
    let nest_rel_types: BTreeSet<String> = nests
        .iter()
        .map(|n| format!("HAS_{}", n.child.to_uppercase()))
        .collect();

    // Group relationships by parent
    let mut parent_children: ParentChildrenMap = BTreeMap::new();

    for rel in relationships {
        if nest_rel_types.contains(&rel.rel_type) || rel.rel_type.starts_with("HAS_") {
            let order = rel
                .properties
                .get("_nest_order")
                .and_then(|v| v.as_int())
                .unwrap_or(0);

            parent_children
                .entry((rel.from_label.clone(), rel.from_id.clone()))
                .or_default()
                .push((rel.to_label.clone(), rel.to_id.clone(), order));
        }
    }

    // First, collect all child nodes we need (to avoid borrow conflicts)
    let mut children_to_attach: Vec<(String, String, String, Node)> = Vec::new();

    for ((parent_label, parent_id), mut children) in parent_children {
        children.sort_by_key(|(_, _, order)| *order);

        for (child_label, child_id, _) in children {
            let child_key = child_label.to_lowercase();

            // Find and clone child node
            if let Some(Item::List(child_list)) = root.get(&child_key) {
                if let Some(child_node) = child_list.rows.iter().find(|n| n.id == child_id) {
                    children_to_attach.push((
                        parent_label.clone(),
                        parent_id.clone(),
                        child_key,
                        child_node.clone(),
                    ));
                }
            }
        }
    }

    // Now attach children to parents
    for (parent_label, parent_id, child_key, child_node) in children_to_attach {
        let parent_key = parent_label.to_lowercase();
        if let Some(Item::List(list)) = root.get_mut(&parent_key) {
            if let Some(parent_node) = list.rows.iter_mut().find(|n| n.id == parent_id) {
                parent_node
                    .children
                    .entry(child_key)
                    .or_default()
                    .push(child_node);
            }
        }
    }

    Ok(())
}

/// Convert non-NEST relationships to reference fields.
fn convert_relationships_to_references(
    root: &mut BTreeMap<String, Item>,
    relationships: &[Neo4jRelationship],
    nests: &[Nest],
    config: &FromNeo4jConfig,
) -> Result<()> {
    // Build set of NEST-related relationship types
    let mut nest_rel_types: BTreeSet<String> = BTreeSet::new();
    for nest in nests {
        nest_rel_types.insert(format!("HAS_{}", nest.child.to_uppercase()));
    }

    // Also treat configured reference relationships as non-NEST
    let ref_rel_types: BTreeSet<&String> = config.reference_relationships.iter().collect();

    // Group relationships by source node
    let mut node_refs: NodeRefsMap = BTreeMap::new();

    for rel in relationships {
        // Skip NEST relationships unless explicitly marked as reference
        let is_nest = rel.rel_type.starts_with("HAS_") && !ref_rel_types.contains(&rel.rel_type);

        if !is_nest || ref_rel_types.contains(&rel.rel_type) {
            node_refs
                .entry((rel.from_label.clone(), rel.from_id.clone()))
                .or_default()
                .push((
                    rel.rel_type.clone(),
                    rel.to_label.clone(),
                    rel.to_id.clone(),
                ));
        }
    }

    // For each node with references, update its fields
    for ((from_label, from_id), refs) in node_refs {
        let from_key = from_label.to_lowercase();

        if let Some(Item::List(list)) = root.get_mut(&from_key) {
            if let Some(node) = list.rows.iter_mut().find(|n| n.id == from_id) {
                // For each reference, try to find a matching column or add a new one
                for (rel_type, to_label, to_id) in refs {
                    // Convert relationship type to column name
                    let column_name = rel_type.to_lowercase();

                    // Check if column exists in schema
                    if let Some(col_idx) = list.schema.iter().position(|c| c == &column_name) {
                        // Update the field with a reference
                        if col_idx < node.fields.len() {
                            node.fields[col_idx] = Value::Reference(hedl_core::Reference {
                                type_name: Some(to_label),
                                id: to_id,
                            });
                        }
                    }
                    // Note: We don't add new columns dynamically to maintain schema consistency
                }
            }
        }
    }

    Ok(())
}

/// Build a Neo4jRecord from raw property maps.
///
/// This is a helper for creating records from database query results.
pub fn build_record(
    label: String,
    properties: BTreeMap<String, CypherValue>,
    id_property: &str,
) -> Result<Neo4jRecord> {
    let id = properties
        .get(id_property)
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| {
            // Try to find any property that could be an ID
            properties
                .iter()
                .find(|(k, _)| k.contains("id") || *k == "name")
                .and_then(|(_, v)| v.as_str())
                .map(String::from)
        })
        .ok_or_else(|| Neo4jError::MissingProperty {
            label: label.clone(),
            property: id_property.to_string(),
        })?;

    let mut node = Neo4jNode::new(label, id);
    for (k, v) in properties {
        if k != id_property {
            node.properties.insert(k, v);
        }
    }

    Ok(Neo4jRecord::new(node))
}

/// Parse a relationship from raw data.
pub fn build_relationship(
    from_label: String,
    from_id: String,
    rel_type: String,
    to_label: String,
    to_id: String,
    properties: BTreeMap<String, CypherValue>,
) -> Neo4jRelationship {
    Neo4jRelationship {
        from_label,
        from_id,
        rel_type,
        to_label,
        to_id,
        properties,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user_record(id: &str, name: &str) -> Neo4jRecord {
        Neo4jRecord::new(Neo4jNode::new("User", id).with_property("name", name))
    }

    fn make_post_record(id: &str, content: &str) -> Neo4jRecord {
        Neo4jRecord::new(Neo4jNode::new("Post", id).with_property("content", content))
    }

    #[test]
    fn test_neo4j_to_hedl_empty() {
        let records: Vec<Neo4jRecord> = vec![];
        let doc = neo4j_to_hedl(&records).unwrap();

        assert!(doc.root.is_empty());
        assert!(doc.nests.is_empty());
    }

    #[test]
    fn test_neo4j_to_hedl_simple() {
        let records = vec![
            make_user_record("alice", "Alice Smith"),
            make_user_record("bob", "Bob Jones"),
        ];

        let doc = neo4j_to_hedl(&records).unwrap();

        assert!(doc.root.contains_key("user"));
        if let Item::List(list) = doc.root.get("user").unwrap() {
            assert_eq!(list.rows.len(), 2);
            assert_eq!(list.type_name, "User");
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_neo4j_to_hedl_multiple_labels() {
        let records = vec![
            make_user_record("alice", "Alice"),
            make_post_record("p1", "Hello World"),
        ];

        let doc = neo4j_to_hedl(&records).unwrap();

        assert!(doc.root.contains_key("user"));
        assert!(doc.root.contains_key("post"));
    }

    #[test]
    fn test_neo4j_to_hedl_with_relationships() {
        let records = vec![
            make_user_record("alice", "Alice"),
            make_post_record("p1", "Hello").with_relationship(Neo4jRelationship::new(
                "Post", "p1", "AUTHOR", "User", "alice",
            )),
        ];

        let doc = neo4j_to_hedl(&records).unwrap();

        // Both should exist
        assert!(doc.root.contains_key("user"));
        assert!(doc.root.contains_key("post"));
    }

    #[test]
    fn test_neo4j_to_hedl_with_nest() {
        let user_record = make_user_record("alice", "Alice").with_relationship(
            Neo4jRelationship::new("User", "alice", "HAS_POST", "Post", "p1")
                .with_property("_nest_order", 0i64),
        );
        let post_record = make_post_record("p1", "Hello World");

        let records = vec![user_record, post_record];
        let doc = neo4j_to_hedl(&records).unwrap();

        // Should have inferred NEST (nests is BTreeMap<parent, child>)
        assert!(!doc.nests.is_empty());
        assert_eq!(doc.nests.get("User"), Some(&"Post".to_string()));

        // User should have Post as child
        if let Item::List(list) = doc.root.get("user").unwrap() {
            let alice = list.rows.iter().find(|n| n.id == "alice").unwrap();
            assert!(!alice.children.is_empty());
        }
    }

    #[test]
    fn test_neo4j_to_hedl_custom_config() {
        let records = vec![make_user_record("alice", "Alice")];

        let config = FromNeo4jConfig::new()
            .with_version(2, 0)
            .with_id_property("id");

        let doc = from_neo4j_records(&records, &config).unwrap();

        assert_eq!(doc.version, (2, 0));
    }

    #[test]
    fn test_neo4j_to_hedl_exclude_labels() {
        let records = vec![
            make_user_record("alice", "Alice"),
            Neo4jRecord::new(Neo4jNode::new("Internal", "sys1")),
        ];

        let config = FromNeo4jConfig::new().exclude_label("Internal");
        let doc = from_neo4j_records(&records, &config).unwrap();

        assert!(doc.root.contains_key("user"));
        assert!(!doc.root.contains_key("internal"));
    }

    #[test]
    fn test_build_record() {
        let mut props = BTreeMap::new();
        props.insert(
            "_hedl_id".to_string(),
            CypherValue::String("alice".to_string()),
        );
        props.insert("name".to_string(), CypherValue::String("Alice".to_string()));

        let record = build_record("User".to_string(), props, "_hedl_id").unwrap();

        assert_eq!(record.node.label, "User");
        assert_eq!(record.node.id, "alice");
        assert!(record.node.properties.contains_key("name"));
    }

    #[test]
    fn test_build_record_missing_id() {
        let props = BTreeMap::new();
        let result = build_record("User".to_string(), props, "_hedl_id");

        assert!(matches!(result, Err(Neo4jError::MissingProperty { .. })));
    }

    #[test]
    fn test_build_relationship() {
        let mut props = BTreeMap::new();
        props.insert("since".to_string(), CypherValue::String("2024".to_string()));

        let rel = build_relationship(
            "Post".to_string(),
            "p1".to_string(),
            "AUTHOR".to_string(),
            "User".to_string(),
            "alice".to_string(),
            props,
        );

        assert_eq!(rel.from_label, "Post");
        assert_eq!(rel.from_id, "p1");
        assert_eq!(rel.rel_type, "AUTHOR");
        assert_eq!(rel.to_label, "User");
        assert_eq!(rel.to_id, "alice");
        assert!(rel.properties.contains_key("since"));
    }

    #[test]
    fn test_neo4j_record_builder() {
        let record = Neo4jRecord::new(Neo4jNode::new("User", "alice"))
            .with_relationship(Neo4jRelationship::new(
                "User", "alice", "KNOWS", "User", "bob",
            ))
            .with_relationships(vec![Neo4jRelationship::new(
                "User", "alice", "FOLLOWS", "User", "carol",
            )]);

        assert_eq!(record.relationships.len(), 2);
    }
}
