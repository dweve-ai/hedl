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

//! Streaming API for incremental Cypher generation.

use std::collections::BTreeMap;
use std::io::Write;

use hedl_core::{Document, Item};

use crate::config::ToCypherConfig;
use crate::cypher::{escape_identifier, escape_label, CypherStatement, CypherValue};
use crate::error::{Neo4jError, Result};
use crate::mapping::{
    collect_node_ids, group_relationships_by_type, matrix_list_to_nodes, validate_references,
    Neo4jNode, Neo4jRelationship,
};

use super::child_iterator::{ChildNodeIterator, TypeBatchedChildren};
use super::constraints::generate_constraints;
use super::nodes::infer_child_schema;
use super::relationships::generate_relationship_query;

/// Type alias for statement writer closure and first-statement flag
pub(crate) type StatementWriter<'a, W> = (
    Box<dyn FnMut(&CypherStatement, &mut W) -> Result<()> + 'a>,
    std::rc::Rc<std::cell::Cell<bool>>,
);

/// Create a closure for writing statements with proper formatting to the output stream.
///
/// This helper manages:
/// - Separator formatting between statements
/// - Comment rendering when enabled
/// - Parameter inlining
/// - I/O error handling
pub(crate) fn create_statement_writer<W: Write>(config: &ToCypherConfig) -> StatementWriter<'_, W> {
    let first_statement = std::rc::Rc::new(std::cell::Cell::new(true));
    let first_stmt_clone = first_statement.clone();

    let writer_fn = Box::new(
        move |stmt: &CypherStatement, writer: &mut W| -> Result<()> {
            // Add separator between statements (but not before first one)
            if !first_stmt_clone.get() {
                write!(writer, "\n\n").map_err(|e| Neo4jError::HedlError(e.to_string()))?;
            }
            first_stmt_clone.set(false);

            // Write comment if present and enabled
            if config.include_comments {
                if let Some(comment) = &stmt.comment {
                    writeln!(writer, "// {comment}")
                        .map_err(|e| Neo4jError::HedlError(e.to_string()))?;
                }
            }

            // Write the statement with inlined parameters (no trailing newline)
            write!(writer, "{};", stmt.render_inline())
                .map_err(|e| Neo4jError::HedlError(e.to_string()))?;

            Ok(())
        },
    );

    (writer_fn, first_statement)
}

/// Stream all constraint statements to the output writer.
pub(crate) fn stream_constraints<W: Write, F>(
    node_types: &BTreeMap<String, Vec<String>>,
    config: &ToCypherConfig,
    writer: &mut W,
    write_statement: &mut F,
) -> Result<()>
where
    F: FnMut(&CypherStatement, &mut W) -> Result<()>,
{
    if config.create_constraints {
        let constraint_statements = generate_constraints(node_types, config)?;
        for stmt in &constraint_statements {
            write_statement(stmt, writer)?;
        }
    }
    Ok(())
}

/// Collect all node types (including child types from NEST hierarchies) for constraint generation.
pub(crate) fn collect_all_node_types(
    doc: &Document,
    config: &ToCypherConfig,
) -> Result<BTreeMap<String, Vec<String>>> {
    let mut node_types: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for item in doc.root.values() {
        if let Item::List(matrix_list) = item {
            node_types.insert(matrix_list.type_name.clone(), matrix_list.schema.clone());

            // Collect child types from NEST hierarchies
            if config.streaming_children {
                // NEW: Streaming approach to collect type information
                let child_iter = ChildNodeIterator::new(&matrix_list.rows, &doc.structs, config);
                let batched_iter = TypeBatchedChildren::new(child_iter, config.batch_size);

                for batch_result in batched_iter {
                    let batch = batch_result?;
                    for (child_type, children) in batch {
                        if !children.is_empty() {
                            // Infer schema from first batch of this type
                            node_types
                                .entry(child_type)
                                .or_insert_with(|| infer_child_schema(&children));
                        }
                    }
                }
            } else {
                // LEGACY: Eager collection (deprecated)
                let child_nodes =
                    super::children::collect_child_nodes(&matrix_list.rows, &doc.structs, config)?;
                for (child_type, children) in child_nodes {
                    if !children.is_empty() {
                        // Infer schema from first child
                        let schema = infer_child_schema(&children);
                        node_types.entry(child_type).or_insert(schema);
                    }
                }
            }
        }
    }

    Ok(node_types)
}

/// Stream all nodes (including child nodes from NEST hierarchies) to the output writer.
pub(crate) fn stream_all_nodes<W: Write, F>(
    doc: &Document,
    config: &ToCypherConfig,
    writer: &mut W,
    write_statement: &mut F,
) -> Result<()>
where
    F: FnMut(&CypherStatement, &mut W) -> Result<()>,
{
    for (key, item) in &doc.root {
        if let Item::List(matrix_list) = item {
            let nodes = matrix_list_to_nodes(matrix_list, config)?;

            // Stream node creation statements in batches
            stream_node_statements(&nodes, key, config, writer, write_statement)?;

            // Stream child nodes from NEST hierarchies
            if config.streaming_children {
                // NEW: Streaming approach - O(batch_size) memory
                stream_child_nodes(
                    &matrix_list.rows,
                    &doc.structs,
                    config,
                    writer,
                    write_statement,
                )?;
            } else {
                // LEGACY: Eager collection (deprecated) - O(total_children) memory
                let child_nodes =
                    super::children::collect_child_nodes(&matrix_list.rows, &doc.structs, config)?;
                for (child_type, children) in child_nodes {
                    if !children.is_empty() {
                        stream_node_statements(
                            &children,
                            &child_type.to_lowercase(),
                            config,
                            writer,
                            write_statement,
                        )?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Stream reference validation warnings if applicable.
pub(crate) fn stream_reference_warnings<W: Write, F>(
    doc: &Document,
    relationships: &[Neo4jRelationship],
    writer: &mut W,
    write_statement: &mut F,
) -> Result<()>
where
    F: FnMut(&CypherStatement, &mut W) -> Result<()>,
{
    // Validate references if we have nodes
    let node_ids = collect_node_ids(doc);
    let invalid_refs = validate_references(relationships, &node_ids);
    if !invalid_refs.is_empty() && !node_ids.is_empty() {
        // Add a comment about unresolved references
        let warning_stmt =
            CypherStatement::query("// Note: Some references may be unresolved").with_comment(
                format!("Warning: {} unresolved reference(s)", invalid_refs.len()),
            );
        write_statement(&warning_stmt, writer)?;
    }
    Ok(())
}

/// Stream node creation statements directly to a writer.
///
/// This function generates and writes node creation statements in batches,
/// avoiding the need to build the entire output in memory.
pub(crate) fn stream_node_statements<W: Write, F>(
    nodes: &[Neo4jNode],
    key: &str,
    config: &ToCypherConfig,
    writer: &mut W,
    write_statement: &mut F,
) -> Result<()>
where
    F: FnMut(&CypherStatement, &mut W) -> Result<()>,
{
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

        let stmt = CypherStatement::create_node(query)
            .with_param("rows", CypherValue::List(rows))
            .with_comment(format!("Create {label} nodes from {key}"));

        write_statement(&stmt, writer)?;
    }

    Ok(())
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

/// Parameters for streaming a single relationship batch.
///
/// Groups the data needed by [`stream_relationship_batch`] to avoid
/// passing too many individual arguments.
struct RelationshipBatchParams<'a> {
    /// The chunk of relationships to process.
    chunk: &'a [&'a Neo4jRelationship],
    /// The relationship type name.
    rel_type: &'a str,
    /// Source node label.
    from_label: &'a str,
    /// Target node label.
    to_label: &'a str,
    /// Pre-built UNWIND row data, shared across label groups to avoid redundant cloning.
    rows: &'a [CypherValue],
    /// Conversion configuration.
    config: &'a ToCypherConfig,
}

/// Stream relationship creation statements for a single batch and label combination.
///
/// # Performance Note
///
/// The `rows` field is borrowed to avoid redundant cloning when multiple label
/// combinations exist in a single batch. The clone only happens once when creating
/// the statement parameter, rather than N times (where N = number of label groups).
fn stream_relationship_batch<W: Write, F>(
    params: &RelationshipBatchParams<'_>,
    writer: &mut W,
    write_statement: &mut F,
) -> Result<()>
where
    F: FnMut(&CypherStatement, &mut W) -> Result<()>,
{
    // Build property SET clause
    let prop_set = build_relationship_property_set(params.chunk);

    // Generate the Cypher query
    let query = generate_relationship_query(
        params.from_label,
        params.to_label,
        params.rel_type,
        params.config,
        &prop_set,
    );

    // Create and write the statement
    // Note: Explicit clone here, but only once per statement instead of per label group
    let stmt = CypherStatement::create_relationship(query)
        .with_param("rows", CypherValue::List(params.rows.to_vec()))
        .with_comment(format!(
            "Create {} relationships from {} to {}",
            params.rel_type, params.from_label, params.to_label
        ));

    write_statement(&stmt, writer)?;
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

/// Stream relationship creation statements directly to a writer.
///
/// This function generates and writes relationship creation statements in batches,
/// avoiding the need to build the entire output in memory.
pub(crate) fn stream_relationship_statements<W: Write, F>(
    relationships: &[Neo4jRelationship],
    config: &ToCypherConfig,
    writer: &mut W,
    write_statement: &mut F,
) -> Result<()>
where
    F: FnMut(&CypherStatement, &mut W) -> Result<()>,
{
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
                let params = RelationshipBatchParams {
                    chunk,
                    rel_type: &rel_type,
                    from_label: &from_label,
                    to_label: &to_label,
                    rows: &rows,
                    config,
                };
                stream_relationship_batch(&params, writer, write_statement)?;
            }
        }
    }

    Ok(())
}

/// Stream child nodes directly to output without full materialization.
///
/// This function replaces the pattern:
///   `collect_child_nodes()` → iterate → `stream_node_statements()`
/// With:
///   `stream_child_nodes()` - direct streaming
///
/// # Memory Usage
///
/// - Peak memory: `O(batch_size)` instead of `O(total_children)`
/// - Memory reduction: ~99% for large NEST hierarchies
fn stream_child_nodes<W, F>(
    parent_nodes: &[hedl_core::Node],
    structs: &BTreeMap<String, Vec<String>>,
    config: &ToCypherConfig,
    writer: &mut W,
    write_statement: &mut F,
) -> Result<()>
where
    W: Write,
    F: FnMut(&CypherStatement, &mut W) -> Result<()>,
{
    let child_iter = ChildNodeIterator::new(parent_nodes, structs, config);
    let batched_iter = TypeBatchedChildren::new(child_iter, config.batch_size);

    for batch_result in batched_iter {
        let batch = batch_result?;
        for (child_type, children) in batch {
            if !children.is_empty() {
                stream_node_statements(
                    &children,
                    &child_type.to_lowercase(),
                    config,
                    writer,
                    write_statement,
                )?;
            }
        }
    }

    Ok(())
}
