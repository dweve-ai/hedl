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

//! Bidirectional conversion between HEDL documents and Neo4j graph databases.
//!
//! This crate provides functionality to:
//! - Export HEDL documents to Cypher queries for Neo4j import
//! - Import Neo4j graph data back to HEDL documents
//!
//! # Mapping Strategy
//!
//! ## HEDL → Neo4j
//!
//! | HEDL Concept | Neo4j Representation |
//! |--------------|---------------------|
//! | `MatrixList` type | Node label |
//! | Node ID | `_hedl_id` property (configurable) |
//! | Node fields | Node properties |
//! | Reference (`@Type:id`) | Relationship to target node |
//! | NEST hierarchy | `HAS_<CHILDTYPE>` relationships |
//! | Tensor | JSON string property |
//! | Expression | String property with `$()` preserved |
//!
//! ## Neo4j → HEDL
//!
//! | Neo4j Concept | HEDL Representation |
//! |---------------|---------------------|
//! | Node label | Struct type / `MatrixList` type |
//! | Node properties | Field values |
//! | Relationship | Reference field or NEST hierarchy |
//! | `HAS_*` relationship | Inferred NEST |
//!
//! # Example: Export to Cypher
//!
//! ```rust
//! use hedl_core::Document;
//! use hedl_neo4j::{to_cypher, ToCypherConfig};
//!
//! fn example(doc: &Document) -> Result<(), hedl_neo4j::Neo4jError> {
//!     // Using default configuration
//!     let cypher = hedl_neo4j::hedl_to_cypher(doc)?;
//!     println!("{}", cypher);
//!
//!     // With fluent configuration API
//!     let config = ToCypherConfig::new()
//!         .with_create()  // Use CREATE instead of MERGE
//!         .with_id_property("nodeId")
//!         .without_constraints();
//!
//!     let cypher = to_cypher(doc, &config)?;
//!     println!("{}", cypher);
//!
//!     // With builder pattern
//!     let config = ToCypherConfig::builder()
//!         .use_merge(false)
//!         .create_constraints(false)
//!         .id_property("nodeId")
//!         .batch_size(500)
//!         .build();
//!
//!     let cypher = to_cypher(doc, &config)?;
//!     println!("{}", cypher);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Example: Import from Neo4j
//!
//! ```rust
//! use hedl_neo4j::{Neo4jRecord, Neo4jNode, Neo4jRelationship, neo4j_to_hedl};
//!
//! fn example() -> Result<(), hedl_neo4j::Neo4jError> {
//!     // Build records from Neo4j query results
//!     let records = vec![
//!         Neo4jRecord::new(
//!             Neo4jNode::new("User", "alice")
//!                 .with_property("name", "Alice Smith")
//!         ).with_relationship(
//!             Neo4jRelationship::new("User", "alice", "HAS_POST", "Post", "p1")
//!         ),
//!         Neo4jRecord::new(
//!             Neo4jNode::new("Post", "p1")
//!                 .with_property("content", "Hello World")
//!         ),
//!     ];
//!
//!     let doc = neo4j_to_hedl(&records)?;
//!     println!("Imported {} structs", doc.structs.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! # Generated Cypher Format
//!
//! The generated Cypher uses best practices for Neo4j imports:
//!
//! - **Constraints**: Creates uniqueness constraints for ID properties
//! - **UNWIND batching**: Uses `UNWIND` for efficient bulk operations
//! - **MERGE by default**: Uses `MERGE` for idempotent imports
//! - **Parameterized queries**: Returns statements with parameters for security
//!
//! Example output:
//!
//! ```cypher
//! // Ensure unique User IDs
//! CREATE CONSTRAINT user__hedl_id IF NOT EXISTS FOR (n:User) REQUIRE n._hedl_id IS UNIQUE;
//!
//! // Create User nodes from users
//! UNWIND $rows AS row
//! MERGE (n:User {_hedl_id: row._hedl_id})
//! SET n.name = row.name;
//!
//! // Create AUTHOR relationships from Post to User
//! UNWIND $rows AS row
//! MATCH (from:Post {_hedl_id: row.from_id})
//! MATCH (to:User {_hedl_id: row.to_id})
//! MERGE (from)-[rel:AUTHOR]->(to);
//! ```

#![cfg_attr(not(test), warn(missing_docs))]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// Batch execution strategies for Neo4j write operations.
pub mod batch_executor;
/// Configuration types for Neo4j conversion operations.
pub mod config;
/// Constants used throughout the hedl-neo4j library.
pub mod constants;
/// Cypher query building utilities.
pub mod cypher;
/// Error types for hedl-neo4j conversions.
pub mod error;
/// Convert Neo4j records to HEDL documents.
pub mod from_neo4j;
/// Mapping between HEDL types and Neo4j graph structures.
pub mod mapping;
/// Safe arithmetic operations for hedl-neo4j.
pub mod safe_arithmetic;
/// Convert HEDL documents to Cypher queries.
pub mod to_cypher;

/// Adaptive batching utilities for optimal memory and performance.
pub mod batching;

/// Batch read operations for efficient Neo4j queries.
#[cfg(feature = "async")]
pub mod batch_read;

/// Async Neo4j client for executing HEDL operations.
#[cfg(feature = "async")]
pub mod async_client;

// Re-export main types at crate root for convenience
pub use config::{
    BatchSizeStrategy, FromNeo4jConfig, FromNeo4jConfigBuilder, IsolationLevel, ObjectHandling,
    RelationshipNaming, ToCypherConfig, ToCypherConfigBuilder, TransactionStrategy,
    DEFAULT_FROM_NEO4J_BATCH_SIZE, DEFAULT_MAX_STRING_LENGTH, DEFAULT_TRANSACTION_BATCH_SIZE,
    DEFAULT_TRANSACTION_ROW_LIMIT,
};
pub use cypher::{CypherScript, CypherStatement, CypherValue, RenderMode, StatementType};
pub use error::{Neo4jError, Result};
pub use from_neo4j::{
    build_record, build_relationship, from_neo4j_records, from_records_iter,
    from_records_streaming, neo4j_to_hedl, Neo4jRecord,
};

#[cfg(feature = "async")]
pub use from_neo4j::{
    query_multi_label_batch, query_nodes_batch, query_nodes_with_relationships_batch,
};

#[cfg(feature = "async")]
pub use batch_read::{BatchQuery, BatchReadConfig};

#[cfg(feature = "async")]
pub use async_client::AsyncNeo4jClient;

pub use mapping::{Neo4jNode, Neo4jRelationship};
pub use to_cypher::{
    hedl_to_cypher, node_to_cypher_inline, to_cypher, to_cypher_statements, to_cypher_stream,
};

// Re-export batching utilities for transaction optimization
pub use batching::{batch_statements, TransactionBatch};
