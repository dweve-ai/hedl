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

//! Mapping between HEDL nodes and Neo4j nodes.

use hedl_core::{MatrixList, Node, Value};
use std::collections::BTreeMap;

use crate::config::{ObjectHandling, ToCypherConfig};
use crate::cypher::CypherValue;
use crate::error::{Neo4jError, Result};
use crate::mapping::value::value_to_cypher;

/// A Neo4j node representation for import/export.
#[derive(Debug, Clone)]
pub struct Neo4jNode {
    /// The node's label (HEDL type name).
    pub label: String,
    /// The node's unique ID within its label.
    pub id: String,
    /// The node's properties.
    pub properties: BTreeMap<String, CypherValue>,
}

impl Neo4jNode {
    /// Create a new Neo4j node.
    pub fn new(label: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: id.into(),
            properties: BTreeMap::new(),
        }
    }

    /// Add a property to the node.
    pub fn with_property(mut self, name: impl Into<String>, value: impl Into<CypherValue>) -> Self {
        self.properties.insert(name.into(), value.into());
        self
    }

    /// Add multiple properties to the node.
    pub fn with_properties(
        mut self,
        props: impl IntoIterator<Item = (String, CypherValue)>,
    ) -> Self {
        self.properties.extend(props);
        self
    }

    /// Get a property value.
    pub fn get_property(&self, name: &str) -> Option<&CypherValue> {
        self.properties.get(name)
    }

    /// Convert to a Cypher map representation.
    pub fn to_cypher_map(&self, id_property: &str) -> CypherValue {
        let mut map = BTreeMap::new();
        map.insert(
            id_property.to_string(),
            CypherValue::String(self.id.clone()),
        );
        map.extend(self.properties.clone());
        CypherValue::Map(map)
    }
}

/// Convert a HEDL Node to a Neo4jNode.
pub fn node_to_neo4j(node: &Node, schema: &[String], config: &ToCypherConfig) -> Result<Neo4jNode> {
    let mut neo4j_node = Neo4jNode::new(&node.type_name, &node.id);

    // Map fields according to schema columns
    for (i, field) in node.fields.iter().enumerate() {
        // Skip the ID field (first column) as it's handled separately
        if i == 0 {
            continue;
        }

        // Get the column name from schema (offset by 1 since we skip ID)
        if let Some(column_name) = schema.get(i) {
            // Skip references as they become relationships
            if !matches!(field, Value::Reference(_)) {
                let cypher_value = value_to_cypher(field, column_name, config)?;
                neo4j_node
                    .properties
                    .insert(column_name.clone(), cypher_value);
            }
        }
    }

    // Add type metadata if configured
    if config.include_type_metadata {
        neo4j_node.properties.insert(
            config.type_property.clone(),
            CypherValue::String(node.type_name.clone()),
        );
    }

    Ok(neo4j_node)
}

/// Convert a MatrixList to a collection of Neo4jNodes.
pub fn matrix_list_to_nodes(list: &MatrixList, config: &ToCypherConfig) -> Result<Vec<Neo4jNode>> {
    if list.rows.is_empty() {
        return Err(Neo4jError::EmptyMatrixList(list.type_name.clone()));
    }

    let mut nodes = Vec::with_capacity(list.rows.len());

    for node in &list.rows {
        let neo4j_node = node_to_neo4j(node, &list.schema, config)?;
        nodes.push(neo4j_node);
    }

    Ok(nodes)
}

/// Extract reference fields from a node for relationship creation.
pub fn extract_references(node: &Node, schema: &[String]) -> Vec<(String, hedl_core::Reference)> {
    let mut references = Vec::new();

    for (i, field) in node.fields.iter().enumerate() {
        if let Value::Reference(ref_value) = field {
            // Get the property name from schema
            if let Some(column_name) = schema.get(i) {
                references.push((column_name.clone(), ref_value.clone()));
            }
        }
    }

    references
}

/// Neo4j relationship representation.
#[derive(Debug, Clone)]
pub struct Neo4jRelationship {
    /// Source node label.
    pub from_label: String,
    /// Source node ID.
    pub from_id: String,
    /// Relationship type.
    pub rel_type: String,
    /// Target node label.
    pub to_label: String,
    /// Target node ID.
    pub to_id: String,
    /// Relationship properties.
    pub properties: BTreeMap<String, CypherValue>,
}

impl Neo4jRelationship {
    /// Create a new relationship.
    pub fn new(
        from_label: impl Into<String>,
        from_id: impl Into<String>,
        rel_type: impl Into<String>,
        to_label: impl Into<String>,
        to_id: impl Into<String>,
    ) -> Self {
        Self {
            from_label: from_label.into(),
            from_id: from_id.into(),
            rel_type: rel_type.into(),
            to_label: to_label.into(),
            to_id: to_id.into(),
            properties: BTreeMap::new(),
        }
    }

    /// Add a property to the relationship.
    pub fn with_property(mut self, name: impl Into<String>, value: impl Into<CypherValue>) -> Self {
        self.properties.insert(name.into(), value.into());
        self
    }
}

/// Build a node from Neo4j properties.
pub fn neo4j_to_node(
    label: &str,
    id: &str,
    properties: &BTreeMap<String, CypherValue>,
    schema: &[String],
    _object_handling: ObjectHandling,
) -> Result<Node> {
    use crate::mapping::value::cypher_to_value;

    let mut fields = Vec::with_capacity(schema.len());

    for (i, column) in schema.iter().enumerate() {
        if i == 0 {
            // First column is the ID
            fields.push(Value::String(id.to_string()));
        } else if let Some(value) = properties.get(column) {
            fields.push(cypher_to_value(value)?);
        } else {
            fields.push(Value::Null);
        }
    }

    Ok(Node {
        type_name: label.to_string(),
        id: id.to_string(),
        fields,
        children: BTreeMap::new(),
        child_count: None,
    })
}

/// Group Neo4j nodes by label.
pub fn group_nodes_by_label(nodes: &[Neo4jNode]) -> BTreeMap<String, Vec<&Neo4jNode>> {
    let mut groups: BTreeMap<String, Vec<&Neo4jNode>> = BTreeMap::new();

    for node in nodes {
        groups.entry(node.label.clone()).or_default().push(node);
    }

    groups
}

/// Infer schema from Neo4j nodes with the same label.
pub fn infer_schema_from_nodes(nodes: &[&Neo4jNode], id_property: &str) -> Vec<String> {
    let mut columns: Vec<String> = vec!["id".to_string()]; // ID is always first

    // Collect all unique property names
    let mut property_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for node in nodes {
        for key in node.properties.keys() {
            if key != id_property {
                property_names.insert(key.clone());
            }
        }
    }

    columns.extend(property_names);
    columns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neo4j_node_new() {
        let node = Neo4jNode::new("User", "alice");
        assert_eq!(node.label, "User");
        assert_eq!(node.id, "alice");
        assert!(node.properties.is_empty());
    }

    #[test]
    fn test_neo4j_node_with_properties() {
        let node = Neo4jNode::new("User", "alice")
            .with_property("name", "Alice Smith")
            .with_property("age", 30i64);

        assert_eq!(
            node.get_property("name"),
            Some(&CypherValue::String("Alice Smith".to_string()))
        );
        assert_eq!(node.get_property("age"), Some(&CypherValue::Int(30)));
    }

    #[test]
    fn test_neo4j_node_to_cypher_map() {
        let node = Neo4jNode::new("User", "alice").with_property("name", "Alice Smith");

        let map = node.to_cypher_map("_hedl_id");
        if let CypherValue::Map(m) = map {
            assert_eq!(
                m.get("_hedl_id"),
                Some(&CypherValue::String("alice".to_string()))
            );
            assert_eq!(
                m.get("name"),
                Some(&CypherValue::String("Alice Smith".to_string()))
            );
        } else {
            panic!("Expected map");
        }
    }

    #[test]
    fn test_node_to_neo4j() {
        let hedl_node = Node {
            type_name: "User".to_string(),
            id: "alice".to_string(),
            fields: vec![
                Value::String("alice".to_string()),
                Value::String("Alice Smith".to_string()),
                Value::Int(30),
            ],
            children: BTreeMap::new(),
            child_count: None,
        };
        let schema = vec!["id".to_string(), "name".to_string(), "age".to_string()];
        let config = ToCypherConfig::default();

        let neo4j_node = node_to_neo4j(&hedl_node, &schema, &config).unwrap();

        assert_eq!(neo4j_node.label, "User");
        assert_eq!(neo4j_node.id, "alice");
        assert_eq!(
            neo4j_node.get_property("name"),
            Some(&CypherValue::String("Alice Smith".to_string()))
        );
        assert_eq!(neo4j_node.get_property("age"), Some(&CypherValue::Int(30)));
    }

    #[test]
    fn test_extract_references() {
        let hedl_node = Node {
            type_name: "Post".to_string(),
            id: "p1".to_string(),
            fields: vec![
                Value::String("p1".to_string()),
                Value::String("Hello World".to_string()),
                Value::Reference(hedl_core::Reference {
                    type_name: Some("User".to_string()),
                    id: "alice".to_string(),
                }),
            ],
            children: BTreeMap::new(),
            child_count: None,
        };
        let schema = vec![
            "id".to_string(),
            "content".to_string(),
            "author".to_string(),
        ];

        let refs = extract_references(&hedl_node, &schema);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].0, "author");
        assert_eq!(refs[0].1.id, "alice");
    }

    #[test]
    fn test_neo4j_relationship() {
        let rel = Neo4jRelationship::new("Post", "p1", "AUTHOR", "User", "alice")
            .with_property("since", "2024");

        assert_eq!(rel.from_label, "Post");
        assert_eq!(rel.from_id, "p1");
        assert_eq!(rel.rel_type, "AUTHOR");
        assert_eq!(rel.to_label, "User");
        assert_eq!(rel.to_id, "alice");
        assert_eq!(
            rel.properties.get("since"),
            Some(&CypherValue::String("2024".to_string()))
        );
    }

    #[test]
    fn test_group_nodes_by_label() {
        let nodes = vec![
            Neo4jNode::new("User", "alice"),
            Neo4jNode::new("User", "bob"),
            Neo4jNode::new("Post", "p1"),
        ];

        let groups = group_nodes_by_label(&nodes);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("User").unwrap().len(), 2);
        assert_eq!(groups.get("Post").unwrap().len(), 1);
    }

    #[test]
    fn test_infer_schema_from_nodes() {
        let node1 = Neo4jNode::new("User", "alice")
            .with_property("name", "Alice")
            .with_property("age", 30i64);
        let node2 = Neo4jNode::new("User", "bob")
            .with_property("name", "Bob")
            .with_property("email", "bob@example.com");

        let nodes: Vec<&Neo4jNode> = vec![&node1, &node2];
        let schema = infer_schema_from_nodes(&nodes, "_hedl_id");

        assert_eq!(schema[0], "id"); // ID always first
        assert!(schema.contains(&"name".to_string()));
        assert!(schema.contains(&"age".to_string()));
        assert!(schema.contains(&"email".to_string()));
    }

    #[test]
    fn test_matrix_list_to_nodes_empty() {
        let list = MatrixList {
            type_name: "Empty".to_string(),
            schema: vec!["id".to_string()],
            rows: vec![],
            count_hint: None,
        };
        let config = ToCypherConfig::default();

        let result = matrix_list_to_nodes(&list, &config);
        assert!(matches!(result, Err(Neo4jError::EmptyMatrixList(_))));
    }
}
