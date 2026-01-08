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

//! JSONPath query support for HEDL documents
//!
//! This module provides JSONPath query functionality for HEDL documents,
//! allowing efficient extraction of specific data using standard JSONPath syntax.
//!
//! # Features
//!
//! - **Standard JSONPath Syntax**: Full support for JSONPath expressions
//! - **Efficient Queries**: Optimized query execution with minimal allocations
//! - **Type-Safe Results**: Returns strongly-typed query results
//! - **Error Handling**: Comprehensive error reporting for invalid queries
//!
//! # Examples
//!
//! ```text
//! use hedl_json::jsonpath::{query, QueryConfig};
//! use hedl_core::Document;
//!
//! fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let doc = hedl_core::parse("name: \"Alice\"\nage: 30".as_bytes())?;
//!     let config = QueryConfig::default();
//!
//!     // Simple field access
//!     let results = query(&doc, "$.name", &config)?;
//!     assert_eq!(results.len(), 1);
//!
//!     // Array filtering
//!     let results = query(&doc, "$.users[?(@.age > 25)]", &config)?;
//!     Ok(())
//! }
//! ```

use hedl_core::Document;
use serde_json::Value as JsonValue;
use serde_json_path::JsonPath;
use std::str::FromStr;
use thiserror::Error;

use crate::{to_json_value, ToJsonConfig};

/// Errors that can occur during JSONPath queries
#[derive(Debug, Error, Clone, PartialEq)]
pub enum QueryError {
    /// Invalid JSONPath expression
    #[error("Invalid JSONPath expression: {0}")]
    InvalidExpression(String),

    /// Document conversion error
    #[error("Failed to convert HEDL document to JSON: {0}")]
    ConversionError(String),

    /// Query execution error
    #[error("Query execution failed: {0}")]
    ExecutionError(String),
}

/// Result type for JSONPath queries
pub type QueryResult<T> = Result<T, QueryError>;

/// Configuration for JSONPath queries
#[derive(Debug, Clone)]
pub struct QueryConfig {
    /// Include HEDL metadata in JSON conversion
    pub include_metadata: bool,

    /// Flatten matrix lists to plain arrays
    pub flatten_lists: bool,

    /// Include children as nested arrays
    pub include_children: bool,

    /// Maximum number of results to return (0 = unlimited)
    pub max_results: usize,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            include_metadata: false,
            flatten_lists: false,
            include_children: true,
            max_results: 0, // Unlimited
        }
    }
}

impl From<&QueryConfig> for ToJsonConfig {
    fn from(config: &QueryConfig) -> Self {
        ToJsonConfig {
            include_metadata: config.include_metadata,
            flatten_lists: config.flatten_lists,
            include_children: config.include_children,
        }
    }
}

/// Query a HEDL document using JSONPath expression
///
/// # Arguments
///
/// * `doc` - The HEDL document to query
/// * `path` - JSONPath expression (e.g., "$.users[*].name")
/// * `config` - Query configuration
///
/// # Returns
///
/// Vector of matching JSON values
///
/// # Examples
///
/// ```text
/// use hedl_json::jsonpath::{query, QueryConfig};
/// use hedl_core::Document;
///
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let doc = hedl_core::parse("users: [@User]\n  u1 Alice 30".as_bytes())?;
///     let config = QueryConfig::default();
///
///     let results = query(&doc, "$.users", &config)?;
///     assert!(!results.is_empty());
///     Ok(())
/// }
/// ```
pub fn query(doc: &Document, path: &str, config: &QueryConfig) -> QueryResult<Vec<JsonValue>> {
    // Convert HEDL document to JSON
    let json_config: ToJsonConfig = config.into();
    let json_value = to_json_value(doc, &json_config).map_err(QueryError::ConversionError)?;

    // Parse JSONPath expression
    let json_path = JsonPath::from_str(path)
        .map_err(|e| QueryError::InvalidExpression(format!("{}", e)))?;

    // Execute query
    let node_list = json_path.query(&json_value);

    // Collect results with optional limit
    let results: Vec<JsonValue> = if config.max_results > 0 {
        node_list.into_iter().take(config.max_results).cloned().collect()
    } else {
        node_list.all().into_iter().cloned().collect()
    };

    Ok(results)
}

/// Query a HEDL document and return the first match
///
/// Convenience function for queries expected to return a single result.
///
/// # Arguments
///
/// * `doc` - The HEDL document to query
/// * `path` - JSONPath expression
/// * `config` - Query configuration
///
/// # Returns
///
/// The first matching JSON value, or None if no matches found
///
/// # Examples
///
/// ```text
/// use hedl_json::jsonpath::{query_first, QueryConfig};
/// use hedl_core::Document;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let doc = hedl_core::parse_hedl("name: \"Alice\""?;
/// let config = QueryConfig::default();
///
/// let result = query_first(&doc, "$.name", &config)?;
/// assert!(result.is_some());
/// # Ok(())
/// # }
/// ```
pub fn query_first(
    doc: &Document,
    path: &str,
    config: &QueryConfig,
) -> QueryResult<Option<JsonValue>> {
    let results = query(doc, path, config)?;
    Ok(results.into_iter().next())
}

/// Query a HEDL document and return a single expected match
///
/// Returns an error if the query returns zero or multiple results.
///
/// # Arguments
///
/// * `doc` - The HEDL document to query
/// * `path` - JSONPath expression
/// * `config` - Query configuration
///
/// # Returns
///
/// The single matching JSON value
///
/// # Errors
///
/// Returns error if zero or multiple matches found
///
/// # Examples
///
/// ```text
/// use hedl_json::jsonpath::{query_single, QueryConfig};
/// use hedl_core::Document;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let doc = hedl_core::parse_hedl("name: \"Alice\""?;
/// let config = QueryConfig::default();
///
/// let result = query_single(&doc, "$.name", &config)?;
/// assert_eq!(result.as_str(), Some("Alice"));
/// # Ok(())
/// # }
/// ```
pub fn query_single(doc: &Document, path: &str, config: &QueryConfig) -> QueryResult<JsonValue> {
    let results = query(doc, path, config)?;

    match results.len() {
        0 => Err(QueryError::ExecutionError(
            "Query returned no results".to_string(),
        )),
        1 => Ok(results.into_iter().next().unwrap()),
        n => Err(QueryError::ExecutionError(format!(
            "Query returned {} results, expected exactly 1",
            n
        ))),
    }
}

/// Check if a JSONPath query matches any elements in a HEDL document
///
/// # Arguments
///
/// * `doc` - The HEDL document to query
/// * `path` - JSONPath expression
/// * `config` - Query configuration
///
/// # Returns
///
/// true if at least one match found, false otherwise
///
/// # Examples
///
/// ```text
/// use hedl_json::jsonpath::{query_exists, QueryConfig};
/// use hedl_core::Document;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let doc = hedl_core::parse_hedl("name: \"Alice\""?;
/// let config = QueryConfig::default();
///
/// assert!(query_exists(&doc, "$.name", &config)?);
/// assert!(!query_exists(&doc, "$.missing", &config)?);
/// # Ok(())
/// # }
/// ```
pub fn query_exists(doc: &Document, path: &str, config: &QueryConfig) -> QueryResult<bool> {
    let results = query(doc, path, config)?;
    Ok(!results.is_empty())
}

/// Count the number of matches for a JSONPath query
///
/// # Arguments
///
/// * `doc` - The HEDL document to query
/// * `path` - JSONPath expression
/// * `config` - Query configuration
///
/// # Returns
///
/// Number of matching elements
///
/// # Examples
///
/// ```text
/// use hedl_json::jsonpath::{query_count, QueryConfig};
/// use hedl_core::Document;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let doc = hedl_core::parse_hedl("items: [1, 2, 3]")?;
/// let config = QueryConfig::default();
///
/// let count = query_count(&doc, "$.items[*]", &config)?;
/// assert_eq!(count, 3);
/// # Ok(())
/// # }
/// ```
pub fn query_count(doc: &Document, path: &str, config: &QueryConfig) -> QueryResult<usize> {
    let results = query(doc, path, config)?;
    Ok(results.len())
}

/// Builder for constructing QueryConfig instances
#[derive(Debug, Default)]
pub struct QueryConfigBuilder {
    include_metadata: bool,
    flatten_lists: bool,
    include_children: bool,
    max_results: usize,
}

impl QueryConfigBuilder {
    /// Create a new QueryConfigBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Include HEDL metadata in JSON conversion
    pub fn include_metadata(mut self, value: bool) -> Self {
        self.include_metadata = value;
        self
    }

    /// Flatten matrix lists to plain arrays
    pub fn flatten_lists(mut self, value: bool) -> Self {
        self.flatten_lists = value;
        self
    }

    /// Include children as nested arrays
    pub fn include_children(mut self, value: bool) -> Self {
        self.include_children = value;
        self
    }

    /// Set maximum number of results (0 = unlimited)
    pub fn max_results(mut self, value: usize) -> Self {
        self.max_results = value;
        self
    }

    /// Build the QueryConfig
    pub fn build(self) -> QueryConfig {
        QueryConfig {
            include_metadata: self.include_metadata,
            flatten_lists: self.flatten_lists,
            include_children: self.include_children,
            max_results: self.max_results,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::parse;

    /// Helper to parse HEDL from string for tests
    fn parse_hedl(input: &str) -> Document {
        // Prepend HEDL header if not present, or separate header from body if needed
        let hedl = if input.contains("%VERSION") || input.starts_with("%HEDL") {
            input.to_string()
        } else if input.contains("%STRUCT") || input.contains("%NEST") {
            // Has directives but no VERSION - add VERSION and ensure separator
            let (header, body) = if input.contains("---") {
                let parts: Vec<&str> = input.splitn(2, "---").collect();
                (parts[0].trim().to_string(), parts.get(1).map(|s| s.trim().to_string()).unwrap_or_default())
            } else {
                // Extract directives to header
                let mut header_lines = Vec::new();
                let mut body_lines = Vec::new();
                for line in input.lines() {
                    if line.trim().starts_with('%') {
                        header_lines.push(line.to_string());
                    } else {
                        body_lines.push(line.to_string());
                    }
                }
                (header_lines.join("\n"), body_lines.join("\n"))
            };
            format!("%VERSION: 1.0\n{}\n---\n{}", header, body)
        } else {
            format!("%VERSION: 1.0\n---\n{}", input)
        };
        parse(hedl.as_bytes()).unwrap()
    }

    // ==================== Basic Query Tests ====================

    #[test]
    fn test_query_simple_field() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        let results = query(&doc, "$.name", &config).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].as_str(), Some("Alice"));
    }

    #[test]
    fn test_query_nested_field() {
        let doc = parse_hedl("user:\n  name: \"Bob\"\n  age: 25");
        let config = QueryConfig::default();

        let results = query(&doc, "$.user.name", &config).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].as_str(), Some("Bob"));
    }

    #[test]
    fn test_query_missing_field() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        let results = query(&doc, "$.missing", &config).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_query_root() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        let results = query(&doc, "$", &config).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_object());
    }

    #[test]
    fn test_query_wildcard() {
        let doc = parse_hedl("a: 1\nb: 2\nc: 3");
        let config = QueryConfig::default();

        let results = query(&doc, "$.*", &config).unwrap();
        assert_eq!(results.len(), 3);
    }

    // ==================== Query Helper Tests ====================

    #[test]
    fn test_query_first_success() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        let result = query_first(&doc, "$.name", &config).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), Some("Alice"));
    }

    #[test]
    fn test_query_first_no_match() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        let result = query_first(&doc, "$.missing", &config).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_query_single_success() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        let result = query_single(&doc, "$.name", &config).unwrap();
        assert_eq!(result.as_str(), Some("Alice"));
    }

    #[test]
    fn test_query_single_no_results() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        let result = query_single(&doc, "$.missing", &config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::ExecutionError(_)));
    }

    #[test]
    fn test_query_single_multiple_results() {
        let doc = parse_hedl("a: 1\nb: 2");
        let config = QueryConfig::default();

        let result = query_single(&doc, "$.*", &config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::ExecutionError(_)));
    }

    #[test]
    fn test_query_exists_true() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        assert!(query_exists(&doc, "$.name", &config).unwrap());
    }

    #[test]
    fn test_query_exists_false() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        assert!(!query_exists(&doc, "$.missing", &config).unwrap());
    }

    #[test]
    fn test_query_count() {
        let doc = parse_hedl("a: 1\nb: 2\nc: 3");
        let config = QueryConfig::default();

        let count = query_count(&doc, "$.*", &config).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_query_count_zero() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        let count = query_count(&doc, "$.missing", &config).unwrap();
        assert_eq!(count, 0);
    }

    // ==================== Configuration Tests ====================

    #[test]
    fn test_config_builder() {
        let config = QueryConfigBuilder::new()
            .include_metadata(true)
            .flatten_lists(true)
            .include_children(false)
            .max_results(10)
            .build();

        assert!(config.include_metadata);
        assert!(config.flatten_lists);
        assert!(!config.include_children);
        assert_eq!(config.max_results, 10);
    }

    #[test]
    fn test_config_default() {
        let config = QueryConfig::default();
        assert!(!config.include_metadata);
        assert!(!config.flatten_lists);
        assert!(config.include_children);
        assert_eq!(config.max_results, 0);
    }

    #[test]
    fn test_config_max_results() {
        let doc = parse_hedl("a: 1\nb: 2\nc: 3\nd: 4");
        let config = QueryConfigBuilder::new().max_results(2).build();

        let results = query(&doc, "$.*", &config).unwrap();
        assert_eq!(results.len(), 2);
    }

    // ==================== Error Handling Tests ====================

    #[test]
    fn test_invalid_jsonpath_expression() {
        let doc = parse_hedl("name: \"Alice\"");
        let config = QueryConfig::default();

        let result = query(&doc, "$$invalid", &config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::InvalidExpression(_)));
    }

    #[test]
    fn test_error_display() {
        let err = QueryError::InvalidExpression("test error".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("Invalid JSONPath expression"));
        assert!(msg.contains("test error"));
    }

    #[test]
    fn test_error_equality() {
        let err1 = QueryError::InvalidExpression("test".to_string());
        let err2 = QueryError::InvalidExpression("test".to_string());
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_error_clone() {
        let err1 = QueryError::ConversionError("test".to_string());
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    // ==================== Complex Query Tests ====================

    #[test]
    fn test_query_nested_objects() {
        let doc = parse_hedl("user:\n  profile:\n    name: \"Alice\"\n    age: 30");
        let config = QueryConfig::default();

        let results = query(&doc, "$.user.profile.name", &config).unwrap();
        assert_eq!(results[0].as_str(), Some("Alice"));
    }

    #[test]
    fn test_query_multiple_values() {
        let doc = parse_hedl("a: 1\nb: 2\nc: 3");
        let config = QueryConfig::default();

        let results = query(&doc, "$.*", &config).unwrap();
        assert_eq!(results.len(), 3);

        let sum: i64 = results
            .iter()
            .filter_map(|v| v.as_i64())
            .sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_query_with_numbers() {
        let doc = parse_hedl("count: 42\nprice: 19.99");
        let config = QueryConfig::default();

        let count = query_single(&doc, "$.count", &config).unwrap();
        assert_eq!(count.as_i64(), Some(42));

        let price = query_single(&doc, "$.price", &config).unwrap();
        assert_eq!(price.as_f64(), Some(19.99));
    }

    #[test]
    fn test_query_with_booleans() {
        let doc = parse_hedl("active: true\ndeleted: false");
        let config = QueryConfig::default();

        let active = query_single(&doc, "$.active", &config).unwrap();
        assert_eq!(active.as_bool(), Some(true));

        let deleted = query_single(&doc, "$.deleted", &config).unwrap();
        assert_eq!(deleted.as_bool(), Some(false));
    }

    #[test]
    fn test_query_with_null() {
        let doc = parse_hedl("value: ~");
        let config = QueryConfig::default();

        let result = query_single(&doc, "$.value", &config).unwrap();
        assert!(result.is_null());
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_query_empty_document() {
        let doc = parse_hedl("");
        let config = QueryConfig::default();

        let results = query(&doc, "$", &config).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_object());
    }

    #[test]
    fn test_query_unicode_fields() {
        // HEDL field names must be ASCII identifiers, but values can be unicode
        let doc = parse_hedl("name: \"太郎\"");
        let config = QueryConfig::default();

        let results = query(&doc, "$.name", &config).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].as_str(), Some("太郎"));
    }

    #[test]
    fn test_query_with_special_characters() {
        // HEDL field names must be valid identifiers (no hyphens)
        // Use underscore instead, and bracket notation still works
        let doc = parse_hedl("field_name: \"value\"");
        let config = QueryConfig::default();

        let results = query(&doc, "$['field_name']", &config).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].as_str(), Some("value"));
    }

    #[test]
    fn test_query_max_results_zero() {
        let doc = parse_hedl("a: 1\nb: 2\nc: 3");
        let config = QueryConfigBuilder::new().max_results(0).build();

        let results = query(&doc, "$.*", &config).unwrap();
        assert_eq!(results.len(), 3); // 0 means unlimited
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_query_builder_chain() {
        let config = QueryConfigBuilder::new()
            .include_metadata(true)
            .flatten_lists(false)
            .max_results(5)
            .include_children(true)
            .build();

        assert!(config.include_metadata);
        assert!(!config.flatten_lists);
        assert!(config.include_children);
        assert_eq!(config.max_results, 5);
    }

    #[test]
    fn test_config_to_json_config_conversion() {
        let query_config = QueryConfigBuilder::new()
            .include_metadata(true)
            .flatten_lists(true)
            .include_children(false)
            .build();

        let json_config: ToJsonConfig = (&query_config).into();

        assert_eq!(json_config.include_metadata, true);
        assert_eq!(json_config.flatten_lists, true);
        assert_eq!(json_config.include_children, false);
    }
}
