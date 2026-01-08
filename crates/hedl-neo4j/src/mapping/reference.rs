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

//! Reference to relationship mapping.

use hedl_core::{Document, MatrixList, Node, Reference, Value};
use std::collections::{BTreeMap, HashSet};

use crate::config::{RelationshipNaming, ToCypherConfig};
use crate::cypher::{to_relationship_type, CypherValue};
use crate::error::Result;
use crate::mapping::node::Neo4jRelationship;

/// A NEST relationship definition (parent -> child).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nest {
    /// Parent type name.
    pub parent: String,
    /// Child type name.
    pub child: String,
}

/// Extract all relationships from a HEDL document.
pub fn extract_relationships(
    doc: &Document,
    config: &ToCypherConfig,
) -> Result<Vec<Neo4jRelationship>> {
    let mut relationships = Vec::new();

    // Extract relationships from references in matrix lists
    for item in doc.root.values() {
        if let hedl_core::Item::List(matrix_list) = item {
            extract_reference_relationships(matrix_list, config, &mut relationships)?;
        }
    }

    // Extract relationships from NEST hierarchies (doc.nests is BTreeMap<String, String>)
    for (parent, child) in &doc.nests {
        let nest = Nest {
            parent: parent.clone(),
            child: child.clone(),
        };
        extract_nest_relationships(doc, &nest, config, &mut relationships)?;
    }

    Ok(relationships)
}

/// Extract relationships from reference fields in a matrix list.
fn extract_reference_relationships(
    list: &MatrixList,
    config: &ToCypherConfig,
    relationships: &mut Vec<Neo4jRelationship>,
) -> Result<()> {
    for node in &list.rows {
        for (i, field) in node.fields.iter().enumerate() {
            if let Value::Reference(ref_value) = field {
                // Get the column name for relationship naming
                let column_name = list.schema.get(i).cloned().unwrap_or_default();

                // Determine relationship type
                let rel_type =
                    determine_relationship_type(&column_name, ref_value, config.reference_naming);

                // Determine target label
                let target_label = ref_value
                    .type_name
                    .clone()
                    .unwrap_or_else(|| list.type_name.clone());

                relationships.push(Neo4jRelationship::new(
                    &list.type_name,
                    &node.id,
                    rel_type,
                    target_label,
                    &ref_value.id,
                ));
            }
        }

        // Recursively handle nested children
        extract_child_references(node, &list.type_name, config, relationships)?;
    }

    Ok(())
}

/// Extract reference relationships from nested children.
fn extract_child_references(
    node: &Node,
    _parent_type: &str,
    config: &ToCypherConfig,
    relationships: &mut Vec<Neo4jRelationship>,
) -> Result<()> {
    for (child_key, children) in &node.children {
        for child in children {
            // Check for references in child fields
            // Note: Children don't have a schema in the same way, so we use field index
            for (i, field) in child.fields.iter().enumerate() {
                if let Value::Reference(ref_value) = field {
                    let rel_type = determine_relationship_type(
                        &format!("{}_{}", child_key, i),
                        ref_value,
                        config.reference_naming,
                    );

                    let target_label = ref_value
                        .type_name
                        .clone()
                        .unwrap_or_else(|| child.type_name.clone());

                    relationships.push(Neo4jRelationship::new(
                        &child.type_name,
                        &child.id,
                        rel_type,
                        target_label,
                        &ref_value.id,
                    ));
                }
            }

            // Recurse into nested children
            extract_child_references(child, &child.type_name, config, relationships)?;
        }
    }

    Ok(())
}

/// Extract relationships from NEST hierarchies.
fn extract_nest_relationships(
    doc: &Document,
    nest: &Nest,
    config: &ToCypherConfig,
    relationships: &mut Vec<Neo4jRelationship>,
) -> Result<()> {
    // Collect all nodes of the parent type (including nested children)
    let parent_nodes = collect_nodes_of_type(doc, &nest.parent);

    for parent_node in parent_nodes {
        // Look for children of the NEST child type
        for (child_key, children) in &parent_node.children {
            for (order, child) in children.iter().enumerate() {
                if child.type_name == nest.child {
                    // Determine relationship type for NEST
                    let rel_type =
                        determine_nest_relationship_type(&nest.child, config.nest_naming);

                    let mut rel = Neo4jRelationship::new(
                        &nest.parent,
                        &parent_node.id,
                        rel_type,
                        &nest.child,
                        &child.id,
                    );

                    // Add order property for NEST relationships
                    rel.properties
                        .insert("_nest_order".to_string(), CypherValue::Int(order as i64));
                    rel.properties.insert(
                        "_nest_key".to_string(),
                        CypherValue::String(child_key.clone()),
                    );

                    relationships.push(rel);
                }
            }
        }
    }

    Ok(())
}

/// Collect all nodes of a specific type, including nested children.
fn collect_nodes_of_type<'a>(doc: &'a Document, type_name: &str) -> Vec<&'a Node> {
    let mut nodes = Vec::new();

    for item in doc.root.values() {
        if let hedl_core::Item::List(list) = item {
            for node in &list.rows {
                if node.type_name == type_name {
                    nodes.push(node);
                }
                // Recursively search children
                collect_nodes_of_type_recursive(node, type_name, &mut nodes);
            }
        }
    }

    nodes
}

/// Recursively collect nodes of a specific type from children.
fn collect_nodes_of_type_recursive<'a>(
    parent: &'a Node,
    type_name: &str,
    nodes: &mut Vec<&'a Node>,
) {
    for children in parent.children.values() {
        for child in children {
            if child.type_name == type_name {
                nodes.push(child);
            }
            // Recurse into nested children
            collect_nodes_of_type_recursive(child, type_name, nodes);
        }
    }
}

/// Determine the relationship type name based on configuration.
fn determine_relationship_type(
    property_name: &str,
    reference: &Reference,
    naming: RelationshipNaming,
) -> String {
    match naming {
        RelationshipNaming::PropertyName => to_relationship_type(property_name),
        RelationshipNaming::Generic => "REFERENCES".to_string(),
        RelationshipNaming::TargetType => {
            if let Some(type_name) = &reference.type_name {
                type_name.to_uppercase()
            } else {
                to_relationship_type(property_name)
            }
        }
    }
}

/// Determine the relationship type name for NEST hierarchies.
fn determine_nest_relationship_type(child_type: &str, naming: RelationshipNaming) -> String {
    match naming {
        RelationshipNaming::PropertyName => format!("HAS_{}", child_type.to_uppercase()),
        RelationshipNaming::Generic => "HAS_CHILD".to_string(),
        RelationshipNaming::TargetType => child_type.to_uppercase(),
    }
}

/// Build a set of all valid node IDs in the document for reference validation.
pub fn collect_node_ids(doc: &Document) -> HashSet<(Option<String>, String)> {
    let mut ids = HashSet::new();

    for item in doc.root.values() {
        if let hedl_core::Item::List(list) = item {
            for node in &list.rows {
                // Add with type name
                ids.insert((Some(list.type_name.clone()), node.id.clone()));
                // Also add without type name for untyped references
                ids.insert((None, node.id.clone()));

                // Collect from children
                collect_child_ids(node, &mut ids);
            }
        }
    }

    ids
}

/// Collect node IDs from children recursively.
fn collect_child_ids(node: &Node, ids: &mut HashSet<(Option<String>, String)>) {
    for children in node.children.values() {
        for child in children {
            ids.insert((Some(child.type_name.clone()), child.id.clone()));
            ids.insert((None, child.id.clone()));
            collect_child_ids(child, ids);
        }
    }
}

/// Validate that all references point to existing nodes.
pub fn validate_references(
    relationships: &[Neo4jRelationship],
    node_ids: &HashSet<(Option<String>, String)>,
) -> Vec<(String, String)> {
    let mut invalid = Vec::new();

    for rel in relationships {
        let with_type = (Some(rel.to_label.clone()), rel.to_id.clone());
        let without_type = (None, rel.to_id.clone());

        if !node_ids.contains(&with_type) && !node_ids.contains(&without_type) {
            invalid.push((rel.to_label.clone(), rel.to_id.clone()));
        }
    }

    invalid
}

/// Infer NEST relationships from relationship patterns.
///
/// This is used when importing from Neo4j to detect which relationships
/// should become NEST hierarchies.
pub fn infer_nests_from_relationships(relationships: &[Neo4jRelationship]) -> Vec<Nest> {
    let mut nests = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();

    for rel in relationships {
        // Look for HAS_* patterns
        if rel.rel_type.starts_with("HAS_") {
            let pair = (rel.from_label.clone(), rel.to_label.clone());
            if !seen.contains(&pair) {
                seen.insert(pair);
                nests.push(Nest {
                    parent: rel.from_label.clone(),
                    child: rel.to_label.clone(),
                });
            }
        }
    }

    nests
}

/// Group relationships by source node for efficient Cypher generation.
pub fn group_relationships_by_source(
    relationships: &[Neo4jRelationship],
) -> BTreeMap<(String, String), Vec<&Neo4jRelationship>> {
    let mut groups: BTreeMap<(String, String), Vec<&Neo4jRelationship>> = BTreeMap::new();

    for rel in relationships {
        groups
            .entry((rel.from_label.clone(), rel.from_id.clone()))
            .or_default()
            .push(rel);
    }

    groups
}

/// Group relationships by type for batch creation.
pub fn group_relationships_by_type(
    relationships: &[Neo4jRelationship],
) -> BTreeMap<String, Vec<&Neo4jRelationship>> {
    let mut groups: BTreeMap<String, Vec<&Neo4jRelationship>> = BTreeMap::new();

    for rel in relationships {
        groups.entry(rel.rel_type.clone()).or_default().push(rel);
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ref(type_name: Option<&str>, id: &str) -> Reference {
        Reference {
            type_name: type_name.map(String::from),
            id: id.to_string(),
        }
    }

    #[test]
    fn test_determine_relationship_type_property_name() {
        let rel_type = determine_relationship_type(
            "author",
            &make_ref(Some("User"), "alice"),
            RelationshipNaming::PropertyName,
        );
        assert_eq!(rel_type, "AUTHOR");
    }

    #[test]
    fn test_determine_relationship_type_generic() {
        let rel_type = determine_relationship_type(
            "author",
            &make_ref(Some("User"), "alice"),
            RelationshipNaming::Generic,
        );
        assert_eq!(rel_type, "REFERENCES");
    }

    #[test]
    fn test_determine_relationship_type_target() {
        let rel_type = determine_relationship_type(
            "author",
            &make_ref(Some("User"), "alice"),
            RelationshipNaming::TargetType,
        );
        assert_eq!(rel_type, "USER");
    }

    #[test]
    fn test_determine_nest_relationship_type() {
        assert_eq!(
            determine_nest_relationship_type("Post", RelationshipNaming::PropertyName),
            "HAS_POST"
        );
        assert_eq!(
            determine_nest_relationship_type("Post", RelationshipNaming::Generic),
            "HAS_CHILD"
        );
        assert_eq!(
            determine_nest_relationship_type("Post", RelationshipNaming::TargetType),
            "POST"
        );
    }

    #[test]
    fn test_collect_node_ids() {
        let mut root = BTreeMap::new();
        root.insert(
            "users".to_string(),
            hedl_core::Item::List(MatrixList {
                type_name: "User".to_string(),
                schema: vec!["id".to_string(), "name".to_string()],
                rows: vec![Node {
                    type_name: "User".to_string(),
                    id: "alice".to_string(),
                    fields: vec![
                        Value::String("alice".to_string()),
                        Value::String("Alice".to_string()),
                    ],
                    children: BTreeMap::new(),
                    child_count: None,
                }],
                count_hint: None,
            }),
        );

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };

        let ids = collect_node_ids(&doc);
        assert!(ids.contains(&(Some("User".to_string()), "alice".to_string())));
        assert!(ids.contains(&(None, "alice".to_string())));
    }

    #[test]
    fn test_validate_references() {
        let mut node_ids = HashSet::new();
        node_ids.insert((Some("User".to_string()), "alice".to_string()));
        node_ids.insert((None, "alice".to_string()));

        let rels = vec![
            Neo4jRelationship::new("Post", "p1", "AUTHOR", "User", "alice"),
            Neo4jRelationship::new("Post", "p2", "AUTHOR", "User", "bob"),
        ];

        let invalid = validate_references(&rels, &node_ids);
        assert_eq!(invalid.len(), 1);
        assert_eq!(invalid[0], ("User".to_string(), "bob".to_string()));
    }

    #[test]
    fn test_infer_nests_from_relationships() {
        let rels = vec![
            Neo4jRelationship::new("User", "alice", "HAS_POST", "Post", "p1"),
            Neo4jRelationship::new("User", "alice", "HAS_POST", "Post", "p2"),
            Neo4jRelationship::new("Post", "p1", "AUTHOR", "User", "alice"),
        ];

        let nests = infer_nests_from_relationships(&rels);
        assert_eq!(nests.len(), 1);
        assert_eq!(nests[0].parent, "User");
        assert_eq!(nests[0].child, "Post");
    }

    #[test]
    fn test_group_relationships_by_source() {
        let rels = vec![
            Neo4jRelationship::new("Post", "p1", "AUTHOR", "User", "alice"),
            Neo4jRelationship::new("Post", "p1", "TAG", "Tag", "rust"),
            Neo4jRelationship::new("Post", "p2", "AUTHOR", "User", "bob"),
        ];

        let groups = group_relationships_by_source(&rels);
        assert_eq!(groups.len(), 2);
        assert_eq!(
            groups
                .get(&("Post".to_string(), "p1".to_string()))
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn test_group_relationships_by_type() {
        let rels = vec![
            Neo4jRelationship::new("Post", "p1", "AUTHOR", "User", "alice"),
            Neo4jRelationship::new("Post", "p2", "AUTHOR", "User", "bob"),
            Neo4jRelationship::new("Post", "p1", "TAG", "Tag", "rust"),
        ];

        let groups = group_relationships_by_type(&rels);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("AUTHOR").unwrap().len(), 2);
        assert_eq!(groups.get("TAG").unwrap().len(), 1);
    }
}
