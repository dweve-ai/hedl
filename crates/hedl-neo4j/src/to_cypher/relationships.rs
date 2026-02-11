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

//! Relationship creation and conversion functions.

use std::collections::BTreeMap;

use crate::config::ToCypherConfig;
use crate::cypher::{
    escape_identifier, escape_label, escape_relationship_type, CypherScript, CypherStatement,
    CypherValue,
};
use crate::error::Result;
use crate::mapping::{group_relationships_by_type, Neo4jRelationship};

/// Convert a relationship to a Cypher map for UNWIND batching.
///
/// This helper transforms a Neo4j relationship into a map containing all
/// necessary fields for batched UNWIND operations.
fn relationship_to_cypher_map(rel: &Neo4jRelationship) -> CypherValue {
    let mut map = BTreeMap::new();
    map.insert(
        "from_label".to_string(),
        CypherValue::String(rel.from_label.clone()),
    );
    map.insert(
        "from_id".to_string(),
        CypherValue::String(rel.from_id.clone()),
    );
    map.insert(
        "to_label".to_string(),
        CypherValue::String(rel.to_label.clone()),
    );
    map.insert("to_id".to_string(), CypherValue::String(rel.to_id.clone()));

    // Include relationship properties
    for (k, v) in &rel.properties {
        map.insert(k.clone(), v.clone());
    }

    CypherValue::Map(map)
}

/// Build the SET clause for relationship properties in a Cypher query.
fn build_relationship_property_set(relationships: &[&Neo4jRelationship]) -> String {
    if !relationships.iter().any(|r| !r.properties.is_empty()) {
        return String::new();
    }

    // Collect all property keys
    let mut prop_keys: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for r in relationships {
        for k in r.properties.keys() {
            prop_keys.insert(k.clone());
        }
    }

    if prop_keys.is_empty() {
        String::new()
    } else {
        let props: Vec<String> = prop_keys
            .iter()
            .map(|k| {
                format!(
                    "rel.{} = row.{}",
                    escape_identifier(k),
                    escape_identifier(k)
                )
            })
            .collect();
        format!("\nSET {}", props.join(", "))
    }
}

/// Generate a Cypher UNWIND query for creating relationships between labeled nodes.
pub(crate) fn generate_relationship_query(
    from_label: &str,
    to_label: &str,
    rel_type: &str,
    config: &ToCypherConfig,
    prop_set: &str,
) -> String {
    let from_label_escaped = escape_label(from_label);
    let to_label_escaped = escape_label(to_label);
    let rel_type_escaped = escape_relationship_type(rel_type);
    let id_prop = escape_identifier(&config.id_property);
    let create_keyword = if config.use_merge { "MERGE" } else { "CREATE" };

    format!(
        "UNWIND $rows AS row\n\
         MATCH (from{from_label_escaped} {{{id_prop}: row.from_id}})\n\
         MATCH (to{to_label_escaped} {{{id_prop}: row.to_id}})\n\
         {create_keyword} (from)-[rel{rel_type_escaped}]->(to){prop_set}"
    )
}

/// Add a relationship creation statement to the script for a specific label combination.
///
/// # Performance Note
///
/// The `rows` parameter is borrowed to avoid redundant cloning when multiple label
/// combinations exist in a single batch. The clone only happens once when creating
/// the statement parameter, rather than N times (where N = number of label groups).
fn add_relationship_statement_to_script(
    chunk: &[&Neo4jRelationship],
    rel_type: &str,
    from_label: &str,
    to_label: &str,
    rows: &[CypherValue],
    config: &ToCypherConfig,
    script: &mut CypherScript,
) {
    // Build property SET clause
    let prop_set = build_relationship_property_set(chunk);

    // Generate the Cypher query
    let query = generate_relationship_query(from_label, to_label, rel_type, config, &prop_set);

    // Add the statement to the script
    // Note: Explicit clone here, but only once per statement instead of per label group
    script.add(
        CypherStatement::create_relationship(query)
            .with_param("rows", CypherValue::List(rows.to_vec()))
            .with_comment(format!(
                "Create {rel_type} relationships from {from_label} to {to_label}"
            )),
    );
}

/// Generate relationship creation statements.
///
/// This function groups relationships by type and label combinations, then
/// generates batched UNWIND statements for efficient bulk creation.
pub(crate) fn generate_relationship_statements(
    relationships: &[Neo4jRelationship],
    config: &ToCypherConfig,
    script: &mut CypherScript,
) -> Result<()> {
    if relationships.is_empty() {
        return Ok(());
    }

    // Group relationships by type for batch creation
    let grouped = group_relationships_by_type(relationships);

    for (rel_type, rels) in grouped {
        for chunk in rels.chunks(config.batch_size) {
            // Build data for UNWIND
            // Note: Shared across all label groups to avoid redundant cloning
            let rows: Vec<CypherValue> = chunk
                .iter()
                .map(|rel| relationship_to_cypher_map(rel))
                .collect();

            // Group by label combination for efficient matching
            let label_groups = group_by_labels(chunk);

            for ((from_label, to_label), _) in label_groups {
                add_relationship_statement_to_script(
                    chunk,
                    &rel_type,
                    &from_label,
                    &to_label,
                    &rows,
                    config,
                    script,
                );
            }
        }
    }

    Ok(())
}

/// Group relationships by (`from_label`, `to_label`) pairs.
fn group_by_labels<'a>(
    rels: &[&'a Neo4jRelationship],
) -> BTreeMap<(String, String), Vec<&'a Neo4jRelationship>> {
    let mut groups: BTreeMap<(String, String), Vec<&'a Neo4jRelationship>> = BTreeMap::new();

    for rel in rels {
        groups
            .entry((rel.from_label.clone(), rel.to_label.clone()))
            .or_default()
            .push(rel);
    }

    groups
}
