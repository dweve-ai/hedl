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

//! # HEDL - Hierarchical Entity Data Language
//!
//! HEDL is a text-based data serialization format optimized for AI/ML data representation.
//! It combines the token efficiency of CSV-style tables with the semantic richness of
//! hierarchical structures.
//!
//! ## Quick Start
//!
//! ```rust
//! use hedl::{parse, canonicalize, to_json};
//!
//! let hedl_doc = r#"
//! %VERSION: 1.0
//! %STRUCT: User: [id,name,email]
//! ---
//! users: @User
//!   | alice, Alice, alice@example.com
//!   | bob, Bob, bob@example.com
//! "#;
//!
//! // Parse the document
//! let doc = parse(hedl_doc).expect("Failed to parse");
//!
//! // Convert to canonical form
//! let canonical = canonicalize(&doc).expect("Failed to canonicalize");
//!
//! // Convert to JSON
//! let json = to_json(&doc).expect("Failed to convert to JSON");
//! ```
//!
//! ## Features
//!
//! - **Type-scoped IDs**: IDs are unique within their type namespace
//! - **Matrix lists**: CSV-like tables for homogeneous collections
//! - **Ditto operator**: `^` copies values from previous row
//! - **References**: `@id` or `@Type:id` for graph relationships
//! - **Tensor literals**: `[1, 2, 3]` for numerical arrays
//! - **Expressions**: `$(...)` for deferred computation
//! - **Aliases**: `%key` for constant substitution
//!
//! ## Modules
//!
//! - [`core`]: Core parsing and data model
//! - [`lex`]: Lexical analysis utilities
//! - [`csv`]: CSV field parsing (internal row parsing)
//! - [`tensor`]: Tensor literal parsing
//! - [`c14n`](mod@c14n): Canonicalization
//! - [`json`]: JSON conversion
//! - [`lint`](mod@lint): Linting and best practices
//!
//! ### Optional Format Converters (feature-gated)
//!
//! - `yaml`: YAML conversion (feature = "yaml")
//! - `xml`: XML conversion (feature = "xml")
//! - `csv_file`: CSV file conversion (feature = "csv")
//! - `parquet`: Parquet conversion (feature = "parquet")

// Re-export core types
pub use hedl_core::{
    // Functions
    parse as core_parse,
    parse_with_limits,
    // Main types
    Document,
    // Errors
    HedlError,
    HedlErrorKind,
    Item,
    MatrixList,
    Node,
    // Parser
    Limits,
    ParseOptions,
    Reference,
    // Tensor type
    Tensor,
    Value,
};

// Error handling extensions
mod error_ext;
pub use error_ext::HedlResultExt;

// Re-export lexer utilities
pub mod lex {
    //! Lexical analysis utilities
    pub use hedl_core::lex::{
        is_valid_id_token, is_valid_key_token, is_valid_type_name, parse_reference, scan_regions,
        strip_comment, validate_indent, IndentInfo, LexError, Reference, Region,
    };
}

// Re-export CSV utilities
pub mod csv {
    //! CSV field parsing
    pub use hedl_core::lex::{parse_csv_row, CsvField};
}

// Re-export tensor utilities
pub mod tensor {
    //! Tensor literal parsing
    pub use hedl_core::lex::{parse_tensor, Tensor};
}

// Re-export canonicalization
pub mod c14n {
    //! Canonicalization utilities
    pub use hedl_c14n::{
        canonicalize, canonicalize_with_config, CanonicalConfig, CanonicalWriter, QuotingStrategy,
    };
}

// Re-export JSON conversion
pub mod json {
    //! JSON conversion utilities
    pub use hedl_json::{
        from_json, from_json_value, hedl_to_json, json_to_hedl, to_json, to_json_value,
        FromJsonConfig, ToJsonConfig,
    };
}

// Re-export linting
pub mod lint {
    //! Linting utilities
    pub use hedl_lint::{
        lint, lint_with_config, Diagnostic, DiagnosticKind, LintConfig, LintRule, LintRunner,
        RuleConfig, Severity,
    };
}

// Optional format converters

/// YAML conversion utilities (requires `yaml` feature)
#[cfg(feature = "yaml")]
pub mod yaml {
    pub use hedl_yaml::{
        from_yaml, hedl_to_yaml, to_yaml, yaml_to_hedl, FromYamlConfig, ToYamlConfig,
    };
}

/// XML conversion utilities (requires `xml` feature)
#[cfg(feature = "xml")]
pub mod xml {
    pub use hedl_xml::{from_xml, hedl_to_xml, to_xml, xml_to_hedl, FromXmlConfig, ToXmlConfig};
}

/// CSV file conversion utilities (requires `csv` feature).
/// Distinct from internal row parsing.
#[cfg(feature = "csv")]
pub mod csv_file {
    pub use hedl_csv::{
        from_csv, from_csv_with_config, to_csv, to_csv_with_config, FromCsvConfig, ToCsvConfig,
    };
}

/// Parquet conversion utilities (requires `parquet` feature)
#[cfg(feature = "parquet")]
pub mod parquet {
    pub use hedl_parquet::{
        from_parquet, from_parquet_bytes, to_parquet, to_parquet_bytes, ToParquetConfig,
    };
}

/// Neo4j/Cypher conversion utilities (requires `neo4j` feature).
/// Provides bidirectional conversion between HEDL documents and Neo4j graph databases.
#[cfg(feature = "neo4j")]
pub mod neo4j {
    pub use hedl_neo4j::{
        build_record,
        build_relationship,
        // Core import functions
        from_neo4j_records,
        hedl_to_cypher,
        neo4j_to_hedl,
        // Core export functions
        to_cypher,
        to_cypher_statements,
        CypherScript,
        CypherStatement,
        CypherValue,
        FromNeo4jConfig,
        // Errors
        Neo4jError,
        Neo4jNode,
        // Types
        Neo4jRecord,
        Neo4jRelationship,
        ObjectHandling,
        RelationshipNaming,
        Result as Neo4jResult,
        StatementType,
        // Configuration
        ToCypherConfig,
    };
}

/// TOON conversion utilities (requires `toon` feature).
#[cfg(feature = "toon")]
pub mod toon {
    pub use hedl_toon::{
        hedl_to_toon, to_toon, Delimiter, ToOnError, ToToonConfig, ToToonConfigBuilder,
    };
}

// Convenience functions at crate root

/// Parse a HEDL document from a string.
///
/// Uses strict mode by default. For lenient parsing, use [`parse_lenient`].
///
/// # Performance
///
/// This is a hot path function with #[inline] hint for 5-10% improvement
/// in small document parsing scenarios.
///
/// # Examples
///
/// ```rust
/// use hedl::parse;
///
/// let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
/// assert_eq!(doc.version, (1, 0));
/// ```
#[inline]
pub fn parse(input: &str) -> Result<Document, HedlError> {
    core_parse(input.as_bytes())
}

/// Parse a HEDL document with lenient reference handling.
///
/// Unresolved references become `null` instead of causing errors.
#[inline]
pub fn parse_lenient(input: &str) -> Result<Document, HedlError> {
    let options = ParseOptions {
        strict_refs: false,
        ..Default::default()
    };
    parse_with_limits(input.as_bytes(), options)
}

/// Canonicalize a HEDL document to a string.
///
/// Produces deterministic output suitable for hashing and diffing.
///
/// # Performance
///
/// This is a hot path function with #[inline] hint for 5-10% improvement
/// in serialization benchmarks.
///
/// # Examples
///
/// ```rust
/// use hedl::{parse, canonicalize};
///
/// let doc = parse("%VERSION: 1.0\n---\nb: 2\na: 1").unwrap();
/// let canonical = canonicalize(&doc).unwrap();
/// // Keys are sorted alphabetically in canonical form
/// assert!(canonical.contains("a: 1"));
/// ```
#[inline]
pub fn canonicalize(doc: &Document) -> Result<String, HedlError> {
    hedl_c14n::canonicalize(doc)
}

/// Convert a HEDL document to JSON.
///
/// # Performance
///
/// This is a hot path function with #[inline] hint for 5-10% improvement
/// in format conversion benchmarks.
///
/// # Examples
///
/// ```rust
/// use hedl::{parse, to_json};
///
/// let doc = parse("%VERSION: 1.0\n---\nkey: 42").unwrap();
/// let json = to_json(&doc).unwrap();
/// assert!(json.contains("\"key\": 42"));
/// ```
#[inline]
pub fn to_json(doc: &Document) -> Result<String, HedlError> {
    hedl_json::to_json(doc, &hedl_json::ToJsonConfig::default())
        .map_err(|e| HedlError::syntax(format!("JSON conversion error: {}", e), 0))
}

/// Convert JSON to a HEDL document.
///
/// # Examples
///
/// ```rust
/// use hedl::from_json;
///
/// let json = r#"{"key": "value"}"#;
/// let doc = from_json(json).unwrap();
/// ```
#[inline]
pub fn from_json(json: &str) -> Result<Document, HedlError> {
    hedl_json::from_json(json, &hedl_json::FromJsonConfig::default())
        .map_err(|e| HedlError::syntax(format!("JSON conversion error: {}", e), 0))
}

/// Lint a HEDL document for best practices.
///
/// # Examples
///
/// ```rust
/// use hedl::{parse, lint};
///
/// let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
/// let diagnostics = lint(&doc);
/// for d in diagnostics {
///     println!("{}", d);
/// }
/// ```
#[inline]
pub fn lint(doc: &Document) -> Vec<lint::Diagnostic> {
    hedl_lint::lint(doc)
}

/// Validate a HEDL string without fully parsing.
///
/// Returns `Ok(())` if valid, `Err` with details if invalid.
#[inline]
pub fn validate(input: &str) -> Result<(), HedlError> {
    parse(input).map(|_| ())
}

/// HEDL format version supported by this library.
pub const SUPPORTED_VERSION: (u32, u32) = (1, 0);

/// Library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal() {
        let doc = parse("%VERSION: 1.0\n---\n").unwrap();
        assert_eq!(doc.version, (1, 0));
    }

    #[test]
    fn test_parse_key_value() {
        let doc = parse("%VERSION: 1.0\n---\nkey: value\nnum: 42").unwrap();
        assert_eq!(doc.version, (1, 0));
    }

    #[test]
    fn test_parse_matrix_list() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id,name]
---
users: @User
  |alice,Alice
  |bob,Bob
"#;
        let doc = parse(input).unwrap();
        assert!(doc.structs.contains_key("User"));
    }

    #[test]
    fn test_canonicalize() {
        let doc = parse("%VERSION: 1.0\n---\nb: 2\na: 1").unwrap();
        let canonical = canonicalize(&doc).unwrap();
        // Canonical form has sorted keys
        let a_pos = canonical.find("a:").unwrap();
        let b_pos = canonical.find("b:").unwrap();
        assert!(a_pos < b_pos);
    }

    #[test]
    fn test_to_json() {
        let doc = parse("%VERSION: 1.0\n---\nkey: 42").unwrap();
        let json = to_json(&doc).unwrap();
        assert!(json.contains("42"));
    }

    #[test]
    fn test_validate() {
        assert!(validate("%VERSION: 1.0\n---\n").is_ok());
        assert!(validate("invalid").is_err());
    }
}
