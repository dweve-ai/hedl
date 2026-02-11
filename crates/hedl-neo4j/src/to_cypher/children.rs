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

//! Child node collection from NEST hierarchies.

use std::collections::BTreeMap;

use crate::config::ToCypherConfig;
use crate::error::{Neo4jError, Result};
use crate::mapping::value::value_to_cypher;
use crate::mapping::Neo4jNode;

use super::validation::DEFAULT_MAX_NEST_DEPTH;

/// Collect child nodes from NEST hierarchies, grouped by type.
///
/// # Deprecated
///
/// This function uses eager materialization which can cause memory spikes for large
/// hierarchical documents. Prefer using `stream_child_nodes()` or enable
/// `config.streaming_children = true` (default) for better memory efficiency.
pub(crate) fn collect_child_nodes(
    nodes: &[hedl_core::Node],
    structs: &BTreeMap<String, Vec<String>>,
    config: &ToCypherConfig,
) -> Result<BTreeMap<String, Vec<Neo4jNode>>> {
    let mut children_by_type: BTreeMap<String, Vec<Neo4jNode>> = BTreeMap::new();

    for node in nodes {
        collect_children_recursive(
            node,
            structs,
            config,
            &mut children_by_type,
            0,
            DEFAULT_MAX_NEST_DEPTH,
        )?;
    }

    Ok(children_by_type)
}

/// Recursively collect children from a node.
///
/// This function traverses child nodes in NEST hierarchies and converts them to `Neo4jNode` format.
/// It uses the schema definitions from the document's structs to map field indices to proper
/// column names (e.g., "title" instead of "`field_1`").
///
/// # Arguments
///
/// * `node` - The parent node to extract children from
/// * `structs` - Schema definitions mapping type names to column names
/// * `config` - Conversion configuration
/// * `children_by_type` - Accumulator for collecting child nodes grouped by type
/// * `depth` - Current recursion depth
/// * `max_depth` - Maximum allowed recursion depth
///
/// # Schema Resolution
///
/// For each child node, the function:
/// 1. Looks up the schema for the child's type in the `structs` map
/// 2. Maps field indices to schema column names (e.g., fields[1] -> "title")
/// 3. Falls back to generic names ("`field_N`") only if schema is not found
///
/// This ensures child nodes have the same property naming convention as parent nodes,
/// as required by SPEC.md Section 10.5.
///
/// # Errors
///
/// Returns `Neo4jError::RecursionLimitExceeded` if the depth exceeds `max_depth`.
fn collect_children_recursive(
    node: &hedl_core::Node,
    structs: &BTreeMap<String, Vec<String>>,
    config: &ToCypherConfig,
    children_by_type: &mut BTreeMap<String, Vec<Neo4jNode>>,
    depth: usize,
    max_depth: usize,
) -> Result<()> {
    if depth > max_depth {
        return Err(Neo4jError::RecursionLimitExceeded { depth, max_depth });
    }

    if let Some(children_map) = node.children() {
        for children in children_map.values() {
            for child in children {
                // Convert child to Neo4jNode
                let mut neo4j_node = Neo4jNode::new(&child.type_name, &child.id);

                // Look up schema for this child type
                if let Some(schema) = structs.get(&child.type_name) {
                    // Use schema column names for properties
                    for (i, field) in child.fields.iter().enumerate() {
                        // Skip ID field (first column)
                        if i == 0 {
                            continue;
                        }

                        // Get the column name from schema
                        if let Some(column_name) = schema.get(i) {
                            // Skip references as they become relationships
                            if !matches!(field, hedl_core::Value::Reference(_)) {
                                let cypher_value = value_to_cypher(field, column_name, config)?;
                                neo4j_node
                                    .properties
                                    .insert(column_name.clone(), cypher_value);
                            }
                        }
                    }
                } else {
                    // Fallback: use generic field names if schema not found
                    // This maintains backward compatibility for edge cases
                    for (i, field) in child.fields.iter().enumerate() {
                        if i == 0 {
                            continue; // Skip ID field
                        }
                        if !matches!(field, hedl_core::Value::Reference(_)) {
                            let prop_name = format!("field_{i}");
                            let cypher_value = value_to_cypher(field, &prop_name, config)?;
                            neo4j_node.properties.insert(prop_name, cypher_value);
                        }
                    }
                }

                children_by_type
                    .entry(child.type_name.clone())
                    .or_default()
                    .push(neo4j_node);

                // Recurse into nested children with incremented depth
                let next_depth = depth
                    .checked_add(1)
                    .ok_or(Neo4jError::RecursionLimitExceeded { depth, max_depth })?;
                collect_children_recursive(
                    child,
                    structs,
                    config,
                    children_by_type,
                    next_depth,
                    max_depth,
                )?;
            }
        }
    }

    Ok(())
}
