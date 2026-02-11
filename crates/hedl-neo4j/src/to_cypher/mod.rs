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
//! - **Node creation**: Converting HEDL `MatrixLists` to Neo4j nodes
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

use hedl_core::Document;
use std::collections::BTreeMap;
use std::io::Write;

use crate::config::ToCypherConfig;
use crate::cypher::{CypherScript, CypherStatement};
use crate::error::Result;
use crate::mapping::{
    collect_node_ids, extract_relationships, matrix_list_to_nodes, validate_references,
};

/// Streaming iterator for child nodes in NEST hierarchies.
pub mod child_iterator;
mod children;
mod constraints;
mod nodes;
mod relationships;
mod streaming;
mod validation;

// Re-export public API
pub use nodes::node_to_cypher_inline;

// Internal imports
use children::collect_child_nodes;
use constraints::generate_constraints;
use nodes::{generate_node_statements, infer_child_schema};
use relationships::generate_relationship_statements;
use streaming::{
    collect_all_node_types, create_statement_writer, stream_all_nodes, stream_constraints,
    stream_reference_warnings, stream_relationship_statements,
};
use validation::validate_node_count;

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
/// Returns `Neo4jError::EmptyMatrixList` if a `MatrixList` has no rows.
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
    // SECURITY: Validate node count before any allocation
    // This prevents DoS attacks through memory exhaustion
    validate_node_count(doc, config)?;

    let mut script = CypherScript::new();

    // Collect all node types for constraint generation
    let mut node_types: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // Process all matrix lists
    for (key, item) in &doc.root {
        if let hedl_core::Item::List(matrix_list) = item {
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
/// Returns `Neo4jError::EmptyMatrixList` if a `MatrixList` has no rows.
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
/// Returns `Neo4jError::EmptyMatrixList` if a `MatrixList` has no rows.
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
/// Returns `Neo4jError::EmptyMatrixList` if a `MatrixList` has no rows.
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
/// - **Memory complexity**: `O(batch_size)` instead of O(n)
/// - **I/O pattern**: Sequential writes, optimal for buffered I/O
/// - **Throughput**: ~same as `to_cypher()`, limited by conversion not I/O
pub fn to_cypher_stream<W: Write>(
    doc: &Document,
    config: &ToCypherConfig,
    writer: &mut W,
) -> Result<()> {
    // SECURITY: Validate node count before any processing
    // This prevents DoS attacks through memory exhaustion
    validate_node_count(doc, config)?;

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
        .map_err(|e| crate::error::Neo4jError::HedlError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests;
