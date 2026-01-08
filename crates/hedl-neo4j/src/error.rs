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

//! Error types for hedl-neo4j conversions.

use thiserror::Error;

/// Error type for Neo4j conversion operations.
#[derive(Debug, Error)]
pub enum Neo4jError {
    /// Missing required schema information.
    #[error("missing schema for type '{0}'")]
    MissingSchema(String),

    /// Invalid reference format.
    #[error("invalid reference: {0}")]
    InvalidReference(String),

    /// Unresolved reference to a non-existent node.
    #[error("unresolved reference: @{type_name}:{id}", type_name = .type_name.as_deref().unwrap_or(""), id = .id)]
    UnresolvedReference {
        /// The type name of the reference (if specified).
        type_name: Option<String>,
        /// The ID being referenced.
        id: String,
    },

    /// Invalid node ID (must be a string).
    #[error("invalid node ID: expected string, got {0}")]
    InvalidNodeId(String),

    /// Empty matrix list (no rows to convert).
    #[error("empty matrix list for type '{0}'")]
    EmptyMatrixList(String),

    /// Inconsistent data structure.
    #[error("inconsistent data: {0}")]
    InconsistentData(String),

    /// Invalid Cypher identifier.
    #[error("invalid Cypher identifier: '{0}'")]
    InvalidIdentifier(String),

    /// Neo4j record parsing error.
    #[error("failed to parse Neo4j record: {0}")]
    RecordParseError(String),

    /// Missing required property in Neo4j node.
    #[error("missing property '{property}' in node with label '{label}'")]
    MissingProperty {
        /// The label of the node.
        label: String,
        /// The missing property name.
        property: String,
    },

    /// Type conversion error.
    #[error("type conversion error: {0}")]
    TypeConversion(String),

    /// Circular reference detected.
    #[error("circular reference detected: {0}")]
    CircularReference(String),

    /// Recursion limit exceeded during NEST hierarchy traversal.
    #[error("NEST hierarchy depth {depth} exceeds maximum allowed depth {max_depth}")]
    RecursionLimitExceeded {
        /// Current depth reached.
        depth: usize,
        /// Maximum allowed depth.
        max_depth: usize,
    },

    /// String length limit exceeded.
    #[error("String length {length} exceeds maximum allowed length {max_length} for property '{property}'")]
    StringLengthExceeded {
        /// Actual length of the string.
        length: usize,
        /// Maximum allowed length.
        max_length: usize,
        /// Property name where the violation occurred.
        property: String,
    },

    /// Node count limit exceeded.
    #[error("Node count {count} exceeds maximum allowed count {max_count}")]
    NodeCountExceeded {
        /// Number of nodes processed.
        count: usize,
        /// Maximum allowed nodes.
        max_count: usize,
    },

    /// Serialization error from serde_json.
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// HEDL core error.
    #[error("HEDL error: {0}")]
    HedlError(String),
}

/// Result type alias for Neo4j operations.
pub type Result<T> = std::result::Result<T, Neo4jError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_missing_schema() {
        let err = Neo4jError::MissingSchema("User".to_string());
        assert!(err.to_string().contains("User"));
        assert!(err.to_string().contains("schema"));
    }

    #[test]
    fn test_error_display_invalid_reference() {
        let err = Neo4jError::InvalidReference("invalid ref".to_string());
        assert!(err.to_string().contains("invalid ref"));
    }

    #[test]
    fn test_error_display_unresolved_reference() {
        let err = Neo4jError::UnresolvedReference {
            type_name: Some("User".to_string()),
            id: "alice".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("User"));
        assert!(msg.contains("alice"));
    }

    #[test]
    fn test_error_display_unresolved_reference_no_type() {
        let err = Neo4jError::UnresolvedReference {
            type_name: None,
            id: "unknown".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("unknown"));
    }

    #[test]
    fn test_error_display_missing_property() {
        let err = Neo4jError::MissingProperty {
            label: "User".to_string(),
            property: "name".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("User"));
        assert!(msg.contains("name"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }
        assert_eq!(returns_result().unwrap(), 42);
    }

    #[test]
    fn test_error_from_json_error() {
        let json_err: serde_json::Error = serde_json::from_str::<i32>("invalid").unwrap_err();
        let neo4j_err: Neo4jError = json_err.into();
        assert!(matches!(neo4j_err, Neo4jError::JsonError(_)));
    }
}
