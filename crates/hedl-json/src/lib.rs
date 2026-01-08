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

//! HEDL JSON Conversion
//!
//! Provides bidirectional conversion between HEDL documents and JSON.
//!
//! # Features
//!
//! - **Bidirectional Conversion**: HEDL ↔ JSON with full fidelity
//! - **JSONPath Queries**: Extract data using standard JSONPath expressions
//! - **JSON Schema Generation**: Generate JSON Schema Draft 7 from HEDL documents
//! - **Partial Parsing**: Continue parsing despite errors and collect all errors
//! - **Streaming Support**: Memory-efficient processing of large files
//! - **JSONL Support**: Newline-delimited JSON for logs and streaming
//! - **Zero-Copy Optimization**: Reduced allocations for better performance
//! - **Security Limits**: Configurable limits to prevent DoS attacks
//!
//! # Modules
//!
//! - [`jsonpath`]: JSONPath query engine for extracting specific data
//! - [`schema_gen`]: JSON Schema generation from HEDL documents
//! - [`streaming`]: Streaming parsers for large files and JSONL format
//!
//! # Examples
//!
//! ## Basic Conversion
//!
//! ```rust
//! use hedl_json::{json_to_hedl, hedl_to_json};
//!
//! let json = r#"{"name": "Alice", "age": 30}"#;
//! let doc = json_to_hedl(json).unwrap();
//! let json_out = hedl_to_json(&doc).unwrap();
//! ```
//!
//! ## JSONPath Queries
//!
//! ```rust
//! use hedl_json::jsonpath::{query, QueryConfig};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let doc = hedl_core::parse(b"name: \"Alice\"\nage: 30")?;
//! let config = QueryConfig::default();
//!
//! // Extract specific fields
//! let results = query(&doc, "$.name", &config)?;
//! assert_eq!(results[0].as_str(), Some("Alice"));
//! # Ok(())
//! # }
//! ```
//!
//! ## JSON Schema Generation
//!
//! ```rust
//! use hedl_json::schema_gen::{generate_schema, SchemaConfig};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let doc = hedl_core::parse(b"name: \"Alice\"\nage: 30")?;
//! let config = SchemaConfig::builder()
//!     .title("User Schema")
//!     .strict(true)
//!     .build();
//!
//! let schema = generate_schema(&doc, &config)?;
//! // schema is a valid JSON Schema Draft 7 document
//! # Ok(())
//! # }
//! ```
//!
//! ## Streaming Large Files
//!
//! ```rust
//! use hedl_json::streaming::{JsonLinesStreamer, StreamConfig};
//! use std::io::Cursor;
//!
//! let jsonl = "{\"id\": \"1\"}\n{\"id\": \"2\"}";
//! let reader = Cursor::new(jsonl.as_bytes());
//! let config = StreamConfig::default();
//!
//! for result in JsonLinesStreamer::new(reader, config) {
//!     let doc = result.unwrap();
//!     // Process each document incrementally
//! }
//! ```
//!
//! ## Partial Parsing with Error Recovery
//!
//! ```rust
//! use hedl_json::{partial_parse_json, PartialConfig, ErrorTolerance};
//!
//! let json = r#"{
//!     "valid": "data",
//!     "users": [
//!         {"id": "1", "name": "Alice"},
//!         {"id": "2", "name": "Bob"}
//!     ]
//! }"#;
//!
//! let config = PartialConfig::builder()
//!     .tolerance(ErrorTolerance::CollectAll)
//!     .build();
//!
//! let result = partial_parse_json(json, &config);
//!
//! // Check if parsing completed successfully
//! if result.is_complete() {
//!     let doc = result.document.unwrap();
//!     // Use the document
//! } else {
//!     // Handle errors
//!     for error in &result.errors {
//!         eprintln!("Error at {}: {}", error.location.path, error.error);
//!     }
//!     // Use partial results if available
//!     if let Some(doc) = result.document {
//!         // Process what was successfully parsed
//!     }
//! }
//! ```
//!
mod from_json;
mod to_json;
pub mod jsonpath;
pub mod streaming;
pub mod schema_gen;
// pub mod partial;

// Re-export the shared DEFAULT_SCHEMA from hedl-core for internal use
pub(crate) use hedl_core::convert::DEFAULT_SCHEMA;

pub use from_json::{
    from_json, from_json_value, from_json_value_owned, FromJsonConfig, FromJsonConfigBuilder,
    DEFAULT_MAX_ARRAY_SIZE, DEFAULT_MAX_DEPTH, DEFAULT_MAX_OBJECT_SIZE,
    DEFAULT_MAX_STRING_LENGTH,
    // Partial parsing exports
    partial_parse_json, partial_parse_json_value, ErrorTolerance, ErrorLocation,
    ParseError, PartialConfig, PartialConfigBuilder, PartialResult,
};
pub use to_json::{to_json, to_json_value, ToJsonConfig};

use hedl_core::Document;

/// Convert HEDL document to JSON string
pub fn hedl_to_json(doc: &Document) -> Result<String, String> {
    to_json(doc, &ToJsonConfig::default())
}

/// Convert JSON string to HEDL document
pub fn json_to_hedl(json: &str) -> Result<Document, String> {
    from_json(json, &FromJsonConfig::default()).map_err(|e| e.to_string())
}
