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

//! Node creation and conversion functions.

use crate::config::ToCypherConfig;
use crate::cypher::{escape_identifier, escape_label, CypherScript, CypherStatement, CypherValue};
use crate::error::Result;
use crate::mapping::Neo4jNode;

/// Generate node creation statements with UNWIND batching.
pub(crate) fn generate_node_statements(
    nodes: &[Neo4jNode],
    key: &str,
    config: &ToCypherConfig,
    script: &mut CypherScript,
) -> Result<()> {
    if nodes.is_empty() {
        return Ok(());
    }

    let label = &nodes[0].label;

    // Calculate optimal batch size based on strategy
    let batch_size = crate::batch_executor::calculate_optimal_batch_size(nodes, config);

    // Batch nodes for UNWIND
    for chunk in nodes.chunks(batch_size) {
        let rows: Vec<CypherValue> = chunk
            .iter()
            .map(|n| n.to_cypher_map(&config.id_property))
            .collect();

        let create_keyword = if config.use_merge { "MERGE" } else { "CREATE" };
        let label_escaped = escape_label(label);
        let id_prop = escape_identifier(&config.id_property);

        // Build SET clauses for all properties except ID
        let mut set_clauses = Vec::new();
        if let Some(first_node) = chunk.first() {
            for prop_name in first_node.properties.keys() {
                let prop_escaped = escape_identifier(prop_name);
                set_clauses.push(format!("n.{prop_escaped} = row.{prop_escaped}"));
            }
        }

        let query = if set_clauses.is_empty() {
            format!(
                "UNWIND $rows AS row\n{create_keyword} (n{label_escaped} {{{id_prop}: row.{id_prop}}})"
            )
        } else {
            format!(
                "UNWIND $rows AS row\n{} (n{} {{{}: row.{}}})\nSET {}",
                create_keyword,
                label_escaped,
                id_prop,
                id_prop,
                set_clauses.join(", ")
            )
        };

        script.add(
            CypherStatement::create_node(query)
                .with_param("rows", CypherValue::List(rows))
                .with_comment(format!("Create {label} nodes from {key}")),
        );
    }

    Ok(())
}

/// Generate Cypher for a single node (inline, no parameters).
#[must_use]
pub fn node_to_cypher_inline(node: &Neo4jNode, config: &ToCypherConfig) -> String {
    let label = escape_label(&node.label);
    let id_prop = escape_identifier(&config.id_property);

    let mut props = vec![format!(
        "{}: {}",
        id_prop,
        CypherValue::String(node.id.clone()).to_cypher_literal()
    )];

    for (k, v) in &node.properties {
        props.push(format!(
            "{}: {}",
            escape_identifier(k),
            v.to_cypher_literal()
        ));
    }

    let create_keyword = if config.use_merge { "MERGE" } else { "CREATE" };
    format!("{} (n{} {{{}}})", create_keyword, label, props.join(", "))
}

/// Infer schema from Neo4j nodes.
pub(crate) fn infer_child_schema(nodes: &[Neo4jNode]) -> Vec<String> {
    let mut schema = vec!["id".to_string()];
    let mut property_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for node in nodes {
        for key in node.properties.keys() {
            property_names.insert(key.clone());
        }
    }

    schema.extend(property_names);
    schema
}
