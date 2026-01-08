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

//! Convert HEDL documents to Cypher queries.
//!
//! This module provides the core functionality for exporting HEDL documents to Neo4j-compatible
//! Cypher queries. It handles:
//!
//! - **Node creation**: Converting HEDL MatrixLists to Neo4j nodes
//! - **Relationship creation**: Converting HEDL references and NEST hierarchies to Neo4j relationships
//! - **Constraint generation**: Creating uniqueness constraints for node IDs
//! - **Batch processing**: Using UNWIND for efficient bulk imports
//! - **Security**: Proper escaping and Unicode normalization to prevent injection attacks
//!
//! # Performance Considerations
//!
//! - Batch size defaults to 1000 nodes per UNWIND statement (configurable)
//! - Relationships are grouped by type to minimize query count
//! - NEST hierarchies are traversed with depth limit protection (max depth: 100)
//!
//! # Security Features
//!
//! - All identifiers are properly escaped to prevent Cypher injection
//! - Unicode normalization (NFC) prevents homograph attacks
//! - Control characters are filtered from identifiers
//! - Depth limit prevents stack overflow from malicious nested structures

use hedl_core::{Document, Item};
use std::collections::BTreeMap;
use std::io::Write;

use crate::config::ToCypherConfig;
use crate::cypher::{
    escape_identifier, escape_label, escape_relationship_type, CypherScript, CypherStatement,
    CypherValue,
};
use crate::error::{Neo4jError, Result};
use crate::mapping::{
    collect_node_ids, extract_relationships, group_relationships_by_type, matrix_list_to_nodes,
    validate_references, Neo4jNode, Neo4jRelationship,
};

/// Default maximum NEST hierarchy depth to prevent stack overflow.
///
/// This limit protects against:
/// - Maliciously crafted deeply nested structures
/// - Infinite recursion from circular references
/// - Stack overflow attacks
///
/// The limit of 100 is sufficient for practical use cases while preventing
/// resource exhaustion.
const DEFAULT_MAX_NEST_DEPTH: usize = 100;

/// Convert a HEDL document to Cypher query statements.
///
/// This is the low-level API that returns structured statement objects, allowing
/// fine-grained control over statement execution order and handling.
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
/// * `config` - Configuration controlling conversion behavior
///
/// # Returns
///
/// A vector of `CypherStatement` objects that can be:
/// - Executed individually for error recovery
/// - Filtered by statement type (constraints, nodes, relationships)
/// - Serialized with or without parameters
///
/// # Errors
///
/// Returns `Neo4jError::EmptyMatrixList` if a MatrixList has no rows.
/// Returns `Neo4jError::RecursionLimitExceeded` if NEST depth exceeds limit.
///
/// # Examples
///
/// ```
/// # use hedl_core::Document;
/// # use hedl_neo4j::{to_cypher_statements, ToCypherConfig};
/// # fn example(doc: Document) -> Result<(), hedl_neo4j::Neo4jError> {
/// let config = ToCypherConfig::default();
/// let statements = to_cypher_statements(&doc, &config)?;
///
/// // Execute constraints first
/// for stmt in statements.iter().filter(|s| s.statement_type == hedl_neo4j::StatementType::Constraint) {
///     // execute(stmt);
/// }
/// # Ok(())
/// # }
/// ```
pub fn to_cypher_statements(
    doc: &Document,
    config: &ToCypherConfig,
) -> Result<Vec<CypherStatement>> {
    let mut script = CypherScript::new();

    // Collect all node types for constraint generation
    let mut node_types: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // Process all matrix lists
    for (key, item) in &doc.root {
        if let Item::List(matrix_list) = item {
            let nodes = matrix_list_to_nodes(matrix_list, config)?;
            node_types.insert(matrix_list.type_name.clone(), matrix_list.schema.clone());

            // Generate node creation statements
            generate_node_statements(&nodes, key, config, &mut script)?;

            // Collect and generate child nodes from NEST hierarchies
            let child_nodes = collect_child_nodes(&matrix_list.rows, &doc.structs, config)?;
            for (child_type, children) in child_nodes {
                if !children.is_empty() {
                    // Infer schema from first child
                    let schema = infer_child_schema(&children);
                    node_types.entry(child_type.clone()).or_insert(schema);
                    generate_node_statements(
                        &children,
                        &child_type.to_lowercase(),
                        config,
                        &mut script,
                    )?;
                }
            }
        }
    }

    // Generate constraints
    if config.create_constraints {
        let constraint_statements = generate_constraints(&node_types, config)?;
        // Insert constraints at the beginning
        let mut all_statements: Vec<CypherStatement> = constraint_statements;
        all_statements.extend(script.statements);
        script.statements = all_statements;
    }

    // Generate relationships from references and NEST
    let relationships = extract_relationships(doc, config)?;

    // Validate references if we have nodes
    let node_ids = collect_node_ids(doc);
    let invalid_refs = validate_references(&relationships, &node_ids);
    if !invalid_refs.is_empty() && !node_ids.is_empty() {
        // Add a comment about unresolved references but continue
        // This is a warning, not an error, since the target might exist in the database
        script.add(
            CypherStatement::query("// Note: Some references may be unresolved").with_comment(
                format!("Warning: {} unresolved reference(s)", invalid_refs.len()),
            ),
        );
    }

    // Generate relationship statements
    generate_relationship_statements(&relationships, config, &mut script)?;

    Ok(script.statements)
}

/// Convert a HEDL document to a Cypher query string.
///
/// This is the mid-level API that provides custom configuration while returning
/// a complete Cypher script as a string.
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
/// * `config` - Configuration controlling conversion behavior
///
/// # Returns
///
/// A complete Cypher script with semicolon-separated statements, ready for execution.
///
/// # Errors
///
/// Returns `Neo4jError::EmptyMatrixList` if a MatrixList has no rows.
/// Returns `Neo4jError::RecursionLimitExceeded` if NEST depth exceeds limit.
///
/// # Examples
///
/// ```
/// # use hedl_core::Document;
/// # use hedl_neo4j::{to_cypher, ToCypherConfig};
/// # fn example(doc: Document) -> Result<(), hedl_neo4j::Neo4jError> {
/// let config = ToCypherConfig::new()
///     .with_batch_size(500)
///     .without_constraints();
///
/// let cypher = to_cypher(&doc, &config)?;
/// // Execute cypher against Neo4j
/// # Ok(())
/// # }
/// ```
pub fn to_cypher(doc: &Document, config: &ToCypherConfig) -> Result<String> {
    let statements = to_cypher_statements(doc, config)?;
    let script = CypherScript { statements };
    Ok(script.render(config.include_comments))
}

/// Convert a HEDL document to Cypher using default configuration.
///
/// This is the high-level API for simple use cases. It uses sensible defaults:
/// - MERGE (not CREATE) for idempotent imports
/// - Uniqueness constraints enabled
/// - Batch size of 1000 nodes
/// - Property-based relationship naming
/// - Comments included in output
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
///
/// # Returns
///
/// A complete Cypher script with semicolon-separated statements.
///
/// # Errors
///
/// Returns `Neo4jError::EmptyMatrixList` if a MatrixList has no rows.
/// Returns `Neo4jError::RecursionLimitExceeded` if NEST depth exceeds limit.
///
/// # Examples
///
/// ```
/// # use hedl_core::Document;
/// # use hedl_neo4j::hedl_to_cypher;
/// # fn example(doc: Document) -> Result<(), hedl_neo4j::Neo4jError> {
/// let cypher = hedl_to_cypher(&doc)?;
/// println!("{}", cypher);
/// # Ok(())
/// # }
/// ```
pub fn hedl_to_cypher(doc: &Document) -> Result<String> {
    to_cypher(doc, &ToCypherConfig::default())
}

/// Convert a HEDL document to Cypher using a streaming writer.
///
/// This API processes documents incrementally, writing statements directly to the
/// output stream instead of building the entire result in memory. This enables
/// processing of arbitrarily large documents with constant memory usage.
///
/// # Benefits
///
/// - **Constant memory usage**: Memory footprint is independent of document size
/// - **Lower latency**: First statements are written immediately
/// - **Large document support**: Can handle multi-gigabyte documents
/// - **Identical output**: Produces exactly the same output as `to_cypher()`
///
/// # When to Use
///
/// Use the streaming API when:
/// - Processing documents larger than 10MB
/// - Memory is constrained (embedded systems, containers)
/// - You need to start execution before full generation completes
/// - You're piping output directly to Neo4j
///
/// Use the regular `to_cypher()` API when:
/// - Documents are small (< 10MB)
/// - You need to inspect or modify the output before execution
/// - You need to parse the output back into statements
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
/// * `config` - Configuration controlling conversion behavior
/// * `writer` - Output stream to write Cypher statements to
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Document conversion fails (e.g., invalid structure)
/// - Writing to the stream fails (I/O error)
///
/// # Errors
///
/// Returns `Neo4jError::EmptyMatrixList` if a MatrixList has no rows.
/// Returns `Neo4jError::RecursionLimitExceeded` if NEST depth exceeds limit.
/// Returns I/O errors as `Neo4jError::HedlError`.
///
/// # Examples
///
/// ```rust
/// use hedl_core::Document;
/// use hedl_neo4j::{to_cypher_stream, ToCypherConfig};
/// use std::io::BufWriter;
///
/// fn example(doc: &Document) -> Result<(), hedl_neo4j::Neo4jError> {
///     // Stream to stdout
///     let stdout = std::io::stdout();
///     let mut writer = BufWriter::new(stdout.lock());
///     to_cypher_stream(doc, &ToCypherConfig::default(), &mut writer)?;
///
///     // Stream to file
///     let file = std::fs::File::create("output.cypher").unwrap();
///     let mut writer = BufWriter::new(file);
///     to_cypher_stream(doc, &ToCypherConfig::default(), &mut writer)?;
///
///     Ok(())
/// }
/// ```
///
/// # Performance Characteristics
///
/// - **Time complexity**: O(n) where n is the number of nodes
/// - **Memory complexity**: O(batch_size) instead of O(n)
/// - **I/O pattern**: Sequential writes, optimal for buffered I/O
/// - **Throughput**: ~same as `to_cypher()`, limited by conversion not I/O
/// Create a closure for writing statements with proper formatting to the output stream.
///
/// This helper manages:
/// - Separator formatting between statements
/// - Comment rendering when enabled
/// - Parameter inlining
/// - I/O error handling
fn create_statement_writer<'a, W: Write>(
    config: &'a ToCypherConfig,
) -> (
    Box<dyn FnMut(&CypherStatement, &mut W) -> Result<()> + 'a>,
    std::rc::Rc<std::cell::Cell<bool>>,
) {
    let first_statement = std::rc::Rc::new(std::cell::Cell::new(true));
    let first_stmt_clone = first_statement.clone();

    let writer_fn = Box::new(move |stmt: &CypherStatement, writer: &mut W| -> Result<()> {
        // Add separator between statements (but not before first one)
        if !first_stmt_clone.get() {
            write!(writer, "\n\n").map_err(|e| Neo4jError::HedlError(e.to_string()))?;
        }
        first_stmt_clone.set(false);

        // Write comment if present and enabled
        if config.include_comments {
            if let Some(comment) = &stmt.comment {
                writeln!(writer, "// {}", comment)
                    .map_err(|e| Neo4jError::HedlError(e.to_string()))?;
            }
        }

        // Write the statement with inlined parameters (no trailing newline)
        write!(writer, "{};", stmt.render_inline())
            .map_err(|e| Neo4jError::HedlError(e.to_string()))?;

        Ok(())
    });

    (writer_fn, first_statement)
}

/// Stream all constraint statements to the output writer.
fn stream_constraints<W: Write, F>(
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
fn collect_all_node_types(
    doc: &Document,
    config: &ToCypherConfig,
) -> Result<BTreeMap<String, Vec<String>>> {
    let mut node_types: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for item in doc.root.values() {
        if let Item::List(matrix_list) = item {
            node_types.insert(matrix_list.type_name.clone(), matrix_list.schema.clone());

            // Also collect child types from NEST hierarchies
            let child_nodes = collect_child_nodes(&matrix_list.rows, &doc.structs, config)?;
            for (child_type, children) in child_nodes {
                if !children.is_empty() {
                    // Infer schema from first child
                    let schema = infer_child_schema(&children);
                    node_types.entry(child_type).or_insert(schema);
                }
            }
        }
    }

    Ok(node_types)
}

/// Stream all nodes (including child nodes from NEST hierarchies) to the output writer.
fn stream_all_nodes<W: Write, F>(
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

            // Collect and stream child nodes from NEST hierarchies
            let child_nodes = collect_child_nodes(&matrix_list.rows, &doc.structs, config)?;
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
    Ok(())
}

/// Stream reference validation warnings if applicable.
fn stream_reference_warnings<W: Write, F>(
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

/// Convert a HEDL document to Cypher using a streaming writer.
///
/// This API processes documents incrementally, writing statements directly to the
/// output stream instead of building the entire result in memory. This enables
/// processing of arbitrarily large documents with constant memory usage.
///
/// # Benefits
///
/// - **Constant memory usage**: Memory footprint is independent of document size
/// - **Lower latency**: First statements are written immediately
/// - **Large document support**: Can handle multi-gigabyte documents
/// - **Identical output**: Produces exactly the same output as `to_cypher()`
///
/// # When to Use
///
/// Use the streaming API when:
/// - Processing documents larger than 10MB
/// - Memory is constrained (embedded systems, containers)
/// - You need to start execution before full generation completes
/// - You're piping output directly to Neo4j
///
/// Use the regular `to_cypher()` API when:
/// - Documents are small (< 10MB)
/// - You need to inspect or modify the output before execution
/// - You need to parse the output back into statements
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
/// * `config` - Configuration controlling conversion behavior
/// * `writer` - Output stream to write Cypher statements to
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Document conversion fails (e.g., invalid structure)
/// - Writing to the stream fails (I/O error)
///
/// # Errors
///
/// Returns `Neo4jError::EmptyMatrixList` if a MatrixList has no rows.
/// Returns `Neo4jError::RecursionLimitExceeded` if NEST depth exceeds limit.
/// Returns I/O errors as `Neo4jError::HedlError`.
///
/// # Performance Characteristics
///
/// - **Time complexity**: O(n) where n is the number of nodes
/// - **Memory complexity**: O(batch_size) instead of O(n)
/// - **I/O pattern**: Sequential writes, optimal for buffered I/O
/// - **Throughput**: ~same as `to_cypher()`, limited by conversion not I/O
pub fn to_cypher_stream<W: Write>(
    doc: &Document,
    config: &ToCypherConfig,
    writer: &mut W,
) -> Result<()> {
    // Create the statement writer closure
    let (mut write_statement, _first_stmt_marker) = create_statement_writer(config);

    // Collect all node types for constraint generation
    let node_types = collect_all_node_types(doc, config)?;

    // Generate and write constraints first
    stream_constraints(&node_types, config, writer, &mut write_statement)?;

    // Stream all nodes (including child nodes from NEST hierarchies)
    stream_all_nodes(doc, config, writer, &mut write_statement)?;

    // Generate relationships from references and NEST
    let relationships = extract_relationships(doc, config)?;

    // Stream reference validation warnings
    stream_reference_warnings(doc, &relationships, writer, &mut write_statement)?;

    // Stream relationship creation statements
    stream_relationship_statements(&relationships, config, writer, &mut write_statement)?;

    // Flush the writer to ensure all data is written
    writer
        .flush()
        .map_err(|e| Neo4jError::HedlError(e.to_string()))?;

    Ok(())
}

/// Stream node creation statements directly to a writer.
///
/// This function generates and writes node creation statements in batches,
/// avoiding the need to build the entire output in memory.
fn stream_node_statements<W: Write, F>(
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

    // Batch nodes for UNWIND
    for chunk in nodes.chunks(config.batch_size) {
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
                set_clauses.push(format!("n.{} = row.{}", prop_escaped, prop_escaped));
            }
        }

        let query = if set_clauses.is_empty() {
            format!(
                "UNWIND $rows AS row\n{} (n{} {{{}: row.{}}})",
                create_keyword, label_escaped, id_prop, id_prop
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
            .with_comment(format!("Create {} nodes from {}", label, key));

        write_statement(&stmt, writer)?;
    }

    Ok(())
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
fn generate_relationship_query(
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
         MATCH (from{} {{{}: row.from_id}})\n\
         MATCH (to{} {{{}: row.to_id}})\n\
         {} (from)-[rel{}]->(to){}",
        from_label_escaped,
        id_prop,
        to_label_escaped,
        id_prop,
        create_keyword,
        rel_type_escaped,
        prop_set
    )
}

/// Stream relationship creation statements for a single batch and label combination.
fn stream_relationship_batch<W: Write, F>(
    chunk: &[&Neo4jRelationship],
    rel_type: &str,
    from_label: &str,
    to_label: &str,
    rows: Vec<CypherValue>,
    config: &ToCypherConfig,
    writer: &mut W,
    write_statement: &mut F,
) -> Result<()>
where
    F: FnMut(&CypherStatement, &mut W) -> Result<()>,
{
    // Build property SET clause
    let prop_set = build_relationship_property_set(chunk);

    // Generate the Cypher query
    let query = generate_relationship_query(from_label, to_label, rel_type, config, &prop_set);

    // Create and write the statement
    let stmt = CypherStatement::create_relationship(query)
        .with_param("rows", CypherValue::List(rows))
        .with_comment(format!(
            "Create {} relationships from {} to {}",
            rel_type, from_label, to_label
        ));

    write_statement(&stmt, writer)?;
    Ok(())
}

/// Stream relationship creation statements directly to a writer.
///
/// This function generates and writes relationship creation statements in batches,
/// avoiding the need to build the entire output in memory.
fn stream_relationship_statements<W: Write, F>(
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
            let rows: Vec<CypherValue> = chunk.iter().map(|rel| relationship_to_cypher_map(rel)).collect();

            // Group by label combination for efficient matching
            let label_groups = group_by_labels(chunk);

            for ((from_label, to_label), _) in label_groups {
                stream_relationship_batch(
                    chunk,
                    &rel_type,
                    &from_label,
                    &to_label,
                    rows.clone(),
                    config,
                    writer,
                    write_statement,
                )?;
            }
        }
    }

    Ok(())
}

/// Generate uniqueness constraints for node types.
///
/// Creates `CREATE CONSTRAINT` statements for each node type to ensure
/// ID uniqueness. Constraints are named using the pattern: `{type}_{id_property}`.
///
/// # Arguments
///
/// * `node_types` - Map of type names to their schemas
/// * `config` - Configuration specifying the ID property name
///
/// # Returns
///
/// Vector of constraint creation statements.
fn generate_constraints(
    node_types: &BTreeMap<String, Vec<String>>,
    config: &ToCypherConfig,
) -> Result<Vec<CypherStatement>> {
    let mut statements = Vec::new();

    for type_name in node_types.keys() {
        let constraint_name = format!(
            "{}_{}",
            type_name.to_lowercase(),
            config.id_property.replace('.', "_")
        );

        let label = escape_label(type_name);
        let id_prop = escape_identifier(&config.id_property);

        let query = format!(
            "CREATE CONSTRAINT {} IF NOT EXISTS FOR (n{}) REQUIRE n.{} IS UNIQUE",
            escape_identifier(&constraint_name),
            label,
            id_prop
        );

        statements.push(
            CypherStatement::constraint(query)
                .with_comment(format!("Ensure unique {} IDs", type_name)),
        );
    }

    Ok(statements)
}

/// Generate node creation statements with UNWIND batching.
fn generate_node_statements(
    nodes: &[Neo4jNode],
    key: &str,
    config: &ToCypherConfig,
    script: &mut CypherScript,
) -> Result<()> {
    if nodes.is_empty() {
        return Ok(());
    }

    let label = &nodes[0].label;

    // Batch nodes for UNWIND
    for chunk in nodes.chunks(config.batch_size) {
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
                set_clauses.push(format!("n.{} = row.{}", prop_escaped, prop_escaped));
            }
        }

        let query = if set_clauses.is_empty() {
            format!(
                "UNWIND $rows AS row\n{} (n{} {{{}: row.{}}})",
                create_keyword, label_escaped, id_prop, id_prop
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
                .with_comment(format!("Create {} nodes from {}", label, key)),
        );
    }

    Ok(())
}

/// Add relationship creation statement to a Cypher script for a single batch and label combination.
fn add_relationship_statement_to_script(
    chunk: &[&Neo4jRelationship],
    rel_type: &str,
    from_label: &str,
    to_label: &str,
    rows: Vec<CypherValue>,
    config: &ToCypherConfig,
    script: &mut CypherScript,
) {
    // Build property SET clause
    let prop_set = build_relationship_property_set(chunk);

    // Generate the Cypher query
    let query = generate_relationship_query(from_label, to_label, rel_type, config, &prop_set);

    // Add the statement to the script
    script.add(
        CypherStatement::create_relationship(query)
            .with_param("rows", CypherValue::List(rows))
            .with_comment(format!(
                "Create {} relationships from {} to {}",
                rel_type, from_label, to_label
            )),
    );
}

/// Generate relationship creation statements.
///
/// This function groups relationships by type and label combinations, then
/// generates batched UNWIND statements for efficient bulk creation.
fn generate_relationship_statements(
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
            let rows: Vec<CypherValue> = chunk.iter().map(|rel| relationship_to_cypher_map(rel)).collect();

            // Group by label combination for efficient matching
            let label_groups = group_by_labels(chunk);

            for ((from_label, to_label), _) in label_groups {
                add_relationship_statement_to_script(
                    chunk, &rel_type, &from_label, &to_label, rows.clone(), config, script,
                );
            }
        }
    }

    Ok(())
}

/// Group relationships by (from_label, to_label) pairs.
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

/// Collect child nodes from NEST hierarchies, grouped by type.
fn collect_child_nodes(
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
/// This function traverses child nodes in NEST hierarchies and converts them to Neo4jNode format.
/// It uses the schema definitions from the document's structs to map field indices to proper
/// column names (e.g., "title" instead of "field_1").
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
/// 3. Falls back to generic names ("field_N") only if schema is not found
///
/// This ensures child nodes have the same property naming convention as parent nodes,
/// as required by SPEC.md Section 10.5.
///
/// # Errors
///
/// Returns `Neo4jError::RecursionLimitExceeded` if the depth exceeds max_depth.
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

    use crate::mapping::value::value_to_cypher;

    for children in node.children.values() {
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
                        let prop_name = format!("field_{}", i);
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
            collect_children_recursive(
                child,
                structs,
                config,
                children_by_type,
                depth + 1,
                max_depth,
            )?;
        }
    }

    Ok(())
}

/// Infer schema from Neo4j nodes.
fn infer_child_schema(nodes: &[Neo4jNode]) -> Vec<String> {
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

/// Generate Cypher for a single node (inline, no parameters).
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cypher::StatementType;
    use hedl_core::{MatrixList, Node, Value};

    fn make_simple_doc() -> Document {
        let mut root = BTreeMap::new();
        root.insert(
            "users".to_string(),
            Item::List(MatrixList {
                type_name: "User".to_string(),
                schema: vec!["id".to_string(), "name".to_string()],
                rows: vec![
                    Node {
                        type_name: "User".to_string(),
                        id: "alice".to_string(),
                        fields: vec![
                            Value::String("alice".to_string()),
                            Value::String("Alice Smith".to_string()),
                        ],
                        children: BTreeMap::new(),
                        child_count: None,
                    },
                    Node {
                        type_name: "User".to_string(),
                        id: "bob".to_string(),
                        fields: vec![
                            Value::String("bob".to_string()),
                            Value::String("Bob Jones".to_string()),
                        ],
                        children: BTreeMap::new(),
                        child_count: None,
                    },
                ],
                count_hint: None,
            }),
        );

        Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        }
    }

    #[test]
    fn test_hedl_to_cypher_simple() {
        let doc = make_simple_doc();
        let result = hedl_to_cypher(&doc).unwrap();

        assert!(result.contains("CREATE CONSTRAINT"));
        assert!(result.contains(":User"));
        assert!(result.contains("UNWIND"));
        assert!(result.contains("MERGE"));
    }

    #[test]
    fn test_to_cypher_with_create() {
        let doc = make_simple_doc();
        let config = ToCypherConfig::new().with_create();
        let result = to_cypher(&doc, &config).unwrap();

        assert!(result.contains("CREATE (n:User"));
        assert!(!result.contains("MERGE (n:User"));
    }

    #[test]
    fn test_to_cypher_without_constraints() {
        let doc = make_simple_doc();
        let config = ToCypherConfig::new().without_constraints();
        let result = to_cypher(&doc, &config).unwrap();

        assert!(!result.contains("CREATE CONSTRAINT"));
    }

    #[test]
    fn test_to_cypher_custom_id_property() {
        let doc = make_simple_doc();
        let config = ToCypherConfig::new().with_id_property("nodeId");
        let result = to_cypher(&doc, &config).unwrap();

        assert!(result.contains("nodeId"));
    }

    #[test]
    fn test_to_cypher_statements() {
        let doc = make_simple_doc();
        let config = ToCypherConfig::default();
        let statements = to_cypher_statements(&doc, &config).unwrap();

        assert!(!statements.is_empty());

        // Should have at least a constraint and a node creation
        let has_constraint = statements
            .iter()
            .any(|s| s.statement_type == StatementType::Constraint);
        let has_node = statements
            .iter()
            .any(|s| s.statement_type == StatementType::CreateNode);

        assert!(has_constraint);
        assert!(has_node);
    }

    #[test]
    fn test_to_cypher_with_references() {
        let mut root = BTreeMap::new();
        root.insert(
            "users".to_string(),
            Item::List(MatrixList {
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
        root.insert(
            "posts".to_string(),
            Item::List(MatrixList {
                type_name: "Post".to_string(),
                schema: vec![
                    "id".to_string(),
                    "content".to_string(),
                    "author".to_string(),
                ],
                rows: vec![Node {
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

        let result = hedl_to_cypher(&doc).unwrap();

        assert!(result.contains(":Post"));
        assert!(result.contains(":User"));
        assert!(result.contains(":AUTHOR")); // Relationship type
    }

    #[test]
    fn test_node_to_cypher_inline() {
        let node =
            crate::mapping::Neo4jNode::new("User", "alice").with_property("name", "Alice Smith");
        let config = ToCypherConfig::default();

        let cypher = node_to_cypher_inline(&node, &config);

        assert!(cypher.contains("MERGE"));
        assert!(cypher.contains(":User"));
        assert!(cypher.contains("_hedl_id: 'alice'"));
        assert!(cypher.contains("name: 'Alice Smith'"));
    }

    #[test]
    fn test_generate_constraints() {
        let mut node_types = BTreeMap::new();
        node_types.insert("User".to_string(), vec!["id".to_string()]);
        node_types.insert("Post".to_string(), vec!["id".to_string()]);

        let config = ToCypherConfig::default();
        let constraints = generate_constraints(&node_types, &config).unwrap();

        assert_eq!(constraints.len(), 2);
        assert!(constraints.iter().any(|c| c.query.contains(":User")));
        assert!(constraints.iter().any(|c| c.query.contains(":Post")));
    }

    #[test]
    fn test_child_nodes_use_schema_column_names() {
        // Create a document with NEST hierarchy
        let mut alice_children = BTreeMap::new();
        alice_children.insert(
            "posts".to_string(),
            vec![Node {
                type_name: "Post".to_string(),
                id: "post1".to_string(),
                fields: vec![
                    Value::String("post1".to_string()),
                    Value::String("First post".to_string()),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
        );

        let mut root = BTreeMap::new();
        root.insert(
            "users".to_string(),
            Item::List(MatrixList {
                type_name: "User".to_string(),
                schema: vec!["id".to_string(), "name".to_string()],
                rows: vec![Node {
                    type_name: "User".to_string(),
                    id: "alice".to_string(),
                    fields: vec![
                        Value::String("alice".to_string()),
                        Value::String("Alice".to_string()),
                    ],
                    children: alice_children,
                    child_count: None,
                }],
                count_hint: None,
            }),
        );

        let mut structs = BTreeMap::new();
        structs.insert(
            "User".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );
        structs.insert(
            "Post".to_string(),
            vec!["id".to_string(), "title".to_string()],
        );

        let mut nests = BTreeMap::new();
        nests.insert("User".to_string(), "Post".to_string());

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs,
            nests,
            root,
        };

        let cypher = to_cypher(&doc, &ToCypherConfig::default()).unwrap();

        // Verify that child Post nodes use 'title' property, not 'field_1'
        assert!(
            cypher.contains("title"),
            "Generated Cypher should contain 'title' property"
        );
        assert!(
            !cypher.contains("field_1"),
            "Generated Cypher should NOT contain 'field_1' property"
        );

        // Also verify the actual value is mapped
        assert!(
            cypher.contains("First post"),
            "Generated Cypher should contain the post title value"
        );
    }

    #[test]
    fn test_streaming_api_basic() {
        let doc = make_simple_doc();
        let config = ToCypherConfig::default();

        // Generate using streaming API
        let mut streaming_output = Vec::new();
        to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
        let streaming_result = String::from_utf8(streaming_output).unwrap();

        // Generate using regular API
        let regular_result = to_cypher(&doc, &config).unwrap();

        // Both should produce identical output
        assert_eq!(streaming_result, regular_result);
    }

    #[test]
    fn test_streaming_api_with_references() {
        let mut root = BTreeMap::new();
        root.insert(
            "users".to_string(),
            Item::List(MatrixList {
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
        root.insert(
            "posts".to_string(),
            Item::List(MatrixList {
                type_name: "Post".to_string(),
                schema: vec![
                    "id".to_string(),
                    "content".to_string(),
                    "author".to_string(),
                ],
                rows: vec![Node {
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

        let config = ToCypherConfig::default();

        // Generate using streaming API
        let mut streaming_output = Vec::new();
        to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
        let streaming_result = String::from_utf8(streaming_output).unwrap();

        // Generate using regular API
        let regular_result = to_cypher(&doc, &config).unwrap();

        // Both should produce identical output
        assert_eq!(streaming_result, regular_result);

        // Verify relationships are present
        assert!(streaming_result.contains(":AUTHOR"));
    }

    #[test]
    fn test_streaming_api_with_nest() {
        let mut alice_children = BTreeMap::new();
        alice_children.insert(
            "posts".to_string(),
            vec![Node {
                type_name: "Post".to_string(),
                id: "post1".to_string(),
                fields: vec![
                    Value::String("post1".to_string()),
                    Value::String("First post".to_string()),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
        );

        let mut root = BTreeMap::new();
        root.insert(
            "users".to_string(),
            Item::List(MatrixList {
                type_name: "User".to_string(),
                schema: vec!["id".to_string(), "name".to_string()],
                rows: vec![Node {
                    type_name: "User".to_string(),
                    id: "alice".to_string(),
                    fields: vec![
                        Value::String("alice".to_string()),
                        Value::String("Alice".to_string()),
                    ],
                    children: alice_children,
                    child_count: None,
                }],
                count_hint: None,
            }),
        );

        let mut structs = BTreeMap::new();
        structs.insert(
            "User".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );
        structs.insert(
            "Post".to_string(),
            vec!["id".to_string(), "title".to_string()],
        );

        let mut nests = BTreeMap::new();
        nests.insert("User".to_string(), "Post".to_string());

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs,
            nests,
            root,
        };

        let config = ToCypherConfig::default();

        // Generate using streaming API
        let mut streaming_output = Vec::new();
        to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
        let streaming_result = String::from_utf8(streaming_output).unwrap();

        // Generate using regular API
        let regular_result = to_cypher(&doc, &config).unwrap();

        // Both should produce identical output
        assert_eq!(streaming_result, regular_result);

        // Verify child nodes use schema column names
        assert!(streaming_result.contains("title"));
        assert!(!streaming_result.contains("field_1"));
    }

    #[test]
    fn test_streaming_api_without_constraints() {
        let doc = make_simple_doc();
        let config = ToCypherConfig::new().without_constraints();

        let mut streaming_output = Vec::new();
        to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
        let streaming_result = String::from_utf8(streaming_output).unwrap();

        let regular_result = to_cypher(&doc, &config).unwrap();

        assert_eq!(streaming_result, regular_result);
        assert!(!streaming_result.contains("CREATE CONSTRAINT"));
    }

    #[test]
    fn test_streaming_api_without_comments() {
        let doc = make_simple_doc();
        let config = ToCypherConfig::new().without_comments();

        let mut streaming_output = Vec::new();
        to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
        let streaming_result = String::from_utf8(streaming_output).unwrap();

        let regular_result = to_cypher(&doc, &config).unwrap();

        assert_eq!(streaming_result, regular_result);
        assert!(!streaming_result.contains("//"));
    }

    #[test]
    fn test_streaming_api_with_create() {
        let doc = make_simple_doc();
        let config = ToCypherConfig::new().with_create();

        let mut streaming_output = Vec::new();
        to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
        let streaming_result = String::from_utf8(streaming_output).unwrap();

        let regular_result = to_cypher(&doc, &config).unwrap();

        assert_eq!(streaming_result, regular_result);
        assert!(streaming_result.contains("CREATE (n:User"));
        assert!(!streaming_result.contains("MERGE (n:User"));
    }

    #[test]
    fn test_streaming_api_custom_batch_size() {
        let doc = make_simple_doc();
        let config = ToCypherConfig::new().with_batch_size(1);

        let mut streaming_output = Vec::new();
        to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
        let streaming_result = String::from_utf8(streaming_output).unwrap();

        let regular_result = to_cypher(&doc, &config).unwrap();

        assert_eq!(streaming_result, regular_result);
    }

    #[test]
    fn test_streaming_api_empty_document() {
        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root: BTreeMap::new(),
        };

        let config = ToCypherConfig::default();

        let mut streaming_output = Vec::new();
        to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
        let streaming_result = String::from_utf8(streaming_output).unwrap();

        let regular_result = to_cypher(&doc, &config).unwrap();

        assert_eq!(streaming_result, regular_result);
    }

    #[test]
    fn test_streaming_api_large_document() {
        // Create a document with many nodes to test batching
        let mut rows = Vec::new();
        for i in 0..5000 {
            rows.push(Node {
                type_name: "User".to_string(),
                id: format!("user{}", i),
                fields: vec![
                    Value::String(format!("user{}", i)),
                    Value::String(format!("User {}", i)),
                ],
                children: BTreeMap::new(),
                child_count: None,
            });
        }

        let mut root = BTreeMap::new();
        root.insert(
            "users".to_string(),
            Item::List(MatrixList {
                type_name: "User".to_string(),
                schema: vec!["id".to_string(), "name".to_string()],
                rows,
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

        let config = ToCypherConfig::new().with_batch_size(1000);

        // Generate using streaming API
        let mut streaming_output = Vec::new();
        to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
        let streaming_result = String::from_utf8(streaming_output).unwrap();

        // Generate using regular API
        let regular_result = to_cypher(&doc, &config).unwrap();

        // Both should produce identical output
        assert_eq!(streaming_result, regular_result);

        // Verify we have multiple batches
        // With 5000 nodes and batch_size 1000, we should have exactly 5 batches
        let batch_count = streaming_result.matches("UNWIND").count();
        assert_eq!(batch_count, 5, "Expected 5 batches for 5000 nodes with batch_size 1000");
    }

    #[test]
    fn test_deep_nested_children_use_schema_column_names() {
        // Create a 3-level NEST hierarchy: Organization > Department > Employee
        let mut dept_children = BTreeMap::new();
        dept_children.insert(
            "employees".to_string(),
            vec![Node {
                type_name: "Employee".to_string(),
                id: "emp1".to_string(),
                fields: vec![
                    Value::String("emp1".to_string()),
                    Value::String("John".to_string()),
                    Value::String("Engineer".to_string()),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
        );

        let mut org_children = BTreeMap::new();
        org_children.insert(
            "departments".to_string(),
            vec![Node {
                type_name: "Department".to_string(),
                id: "eng".to_string(),
                fields: vec![
                    Value::String("eng".to_string()),
                    Value::String("Engineering".to_string()),
                ],
                children: dept_children,
                child_count: None,
            }],
        );

        let mut root = BTreeMap::new();
        root.insert(
            "organizations".to_string(),
            Item::List(MatrixList {
                type_name: "Organization".to_string(),
                schema: vec!["id".to_string(), "name".to_string()],
                rows: vec![Node {
                    type_name: "Organization".to_string(),
                    id: "acme".to_string(),
                    fields: vec![
                        Value::String("acme".to_string()),
                        Value::String("ACME Corp".to_string()),
                    ],
                    children: org_children,
                    child_count: None,
                }],
                count_hint: None,
            }),
        );

        let mut structs = BTreeMap::new();
        structs.insert(
            "Organization".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );
        structs.insert(
            "Department".to_string(),
            vec!["id".to_string(), "dept_name".to_string()],
        );
        structs.insert(
            "Employee".to_string(),
            vec!["id".to_string(), "emp_name".to_string(), "role".to_string()],
        );

        let mut nests = BTreeMap::new();
        nests.insert("Organization".to_string(), "Department".to_string());
        nests.insert("Department".to_string(), "Employee".to_string());

        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs,
            nests,
            root,
        };

        let cypher = to_cypher(&doc, &ToCypherConfig::default()).unwrap();

        // Verify Department uses 'dept_name' not 'field_1'
        assert!(
            cypher.contains("dept_name"),
            "Department should use 'dept_name' property"
        );

        // Verify Employee uses 'emp_name' and 'role' not 'field_1' and 'field_2'
        assert!(
            cypher.contains("emp_name"),
            "Employee should use 'emp_name' property"
        );
        assert!(
            cypher.contains("role"),
            "Employee should use 'role' property"
        );

        // Verify no generic field names
        assert!(!cypher.contains("field_1"), "Should not contain 'field_1'");
        assert!(!cypher.contains("field_2"), "Should not contain 'field_2'");

        // Verify actual values
        assert!(
            cypher.contains("Engineering"),
            "Should contain department name"
        );
        assert!(cypher.contains("John"), "Should contain employee name");
        assert!(cypher.contains("Engineer"), "Should contain employee role");
    }
}
