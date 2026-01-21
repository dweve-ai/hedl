// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Error types for YAML conversion operations.
//!
//! ## Error Messages
//!
//! The YAML parser provides detailed error messages with:
//!
//! - **Precise Location**: Line and column numbers for all errors
//! - **Code Snippets**: Visual context showing the problematic YAML
//! - **Path Tracking**: Full path to the error (e.g., `root.users[2].name`)
//! - **Helpful Suggestions**: Actionable advice for fixing common mistakes
//!
//! ### Example Error Output
//!
//! ```text
//! Error: Non-string keys not supported, found number
//!   at line 3, column 3
//!   in path: users
//!
//!    2 | users:
//!    3 |   123: invalid
//!      |   ^^^ error here
//!    4 |   name: Alice
//!
//! Suggestions:
//!   1. YAML keys must be strings, but found number
//!   2. Convert the key to a string by wrapping it in quotes
//!   3. Example: "123": value
//! ```

use std::fmt;
use thiserror::Error;

/// Location in the YAML source (line and column).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Byte offset in the source
    pub byte_offset: usize,
}

impl Location {
    /// Creates a new location.
    #[must_use]
    pub fn new(line: usize, column: usize, byte_offset: usize) -> Self {
        Self {
            line,
            column,
            byte_offset,
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}

/// Span in the YAML source (start and end locations).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Start location
    pub start: Location,
    /// End location
    pub end: Location,
}

impl Span {
    /// Creates a new span.
    #[must_use]
    pub fn new(start: Location, end: Location) -> Self {
        Self { start, end }
    }
}

/// Errors that can occur during YAML to HEDL conversion.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum YamlError {
    /// YAML parsing failed
    #[error("YAML parse error: {message}")]
    ParseError {
        /// Error message describing the parse failure.
        message: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Root element must be a mapping/object
    #[error("Root must be a YAML mapping, found {found}")]
    InvalidRootType {
        /// The type that was found instead of mapping.
        found: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Non-string key encountered in mapping
    #[error("Non-string keys not supported, found {key_type} at path {path}")]
    NonStringKey {
        /// The type of the non-string key.
        key_type: String,
        /// Path to the problematic key.
        path: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Invalid number format
    #[error("Invalid number format: {value}")]
    InvalidNumber {
        /// The invalid number string.
        value: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Invalid expression syntax
    #[error("Invalid expression: {message}")]
    InvalidExpression {
        /// Error message describing the expression issue.
        message: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Invalid reference format
    #[error("Invalid reference format: {message}")]
    InvalidReference {
        /// Error message describing the reference issue.
        message: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Nested objects not allowed in scalar context
    #[error("Nested objects not allowed in scalar context at path {path}")]
    NestedObjectInScalar {
        /// Path where nesting was found.
        path: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Invalid tensor element type
    #[error("Invalid tensor element at path {path}: must be number or sequence")]
    InvalidTensorElement {
        /// Path to the invalid tensor element.
        path: String,
        /// Expected element type.
        expected: String,
        /// Type that was found.
        found: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {limit_type} (limit: {limit}, actual: {actual})")]
    ResourceLimitExceeded {
        /// Type of limit that was exceeded.
        limit_type: String,
        /// Maximum allowed value.
        limit: usize,
        /// Actual value that exceeded the limit.
        actual: usize,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Maximum nesting depth exceeded
    #[error(
        "Maximum nesting depth of {max_depth} exceeded at depth {actual_depth} at path {path}"
    )]
    MaxDepthExceeded {
        /// Maximum allowed nesting depth.
        max_depth: usize,
        /// Actual nesting depth encountered.
        actual_depth: usize,
        /// Path where excessive nesting was found.
        path: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Document too large
    #[error("Document size {size} bytes exceeds maximum of {max_size} bytes")]
    DocumentTooLarge {
        /// Actual document size in bytes.
        size: usize,
        /// Maximum allowed document size in bytes.
        max_size: usize,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Array too long
    #[error("Array length {length} exceeds maximum of {max_length} at path {path}")]
    ArrayTooLong {
        /// Actual array length.
        length: usize,
        /// Maximum allowed array length.
        max_length: usize,
        /// Path to the oversized array.
        path: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Generic conversion error
    #[error("Conversion error: {message}")]
    Conversion {
        /// Error message describing the conversion failure.
        message: String,
        /// Source location of the error.
        location: Option<Location>,
        /// Code snippet showing the error context.
        snippet: Option<String>,
    },

    /// Forward reference to undefined anchor
    #[error("Forward reference: alias '*{alias}' at line {line} references undefined anchor")]
    ForwardReference {
        /// Name of the undefined alias.
        alias: String,
        /// Line number where the forward reference occurred.
        line: usize,
    },

    /// Circular anchor reference detected
    #[error("Circular anchor reference: {cycle_path}")]
    CircularReference {
        /// Path describing the circular reference chain.
        cycle_path: String,
        /// Anchor names involved in the cycle.
        anchors: Vec<String>,
        /// Line numbers where each anchor is defined.
        locations: Vec<usize>,
    },

    /// Invalid anchor name
    #[error("Invalid anchor name '{name}': {reason}")]
    InvalidAnchorName {
        /// The invalid anchor name.
        name: String,
        /// Reason why the anchor name is invalid.
        reason: String,
    },

    /// Anchor redefinition
    #[error(
        "Anchor '{name}' redefined at line {new_line} (previously defined at line {old_line})"
    )]
    AnchorRedefinition {
        /// The redefined anchor name.
        name: String,
        /// Line number of the original definition.
        old_line: usize,
        /// Line number of the redefinition.
        new_line: usize,
    },
}

impl YamlError {
    /// Returns the location of the error, if available.
    #[must_use]
    pub fn location(&self) -> Option<&Location> {
        match self {
            Self::ParseError { location, .. }
            | Self::InvalidRootType { location, .. }
            | Self::NonStringKey { location, .. }
            | Self::InvalidNumber { location, .. }
            | Self::InvalidExpression { location, .. }
            | Self::InvalidReference { location, .. }
            | Self::NestedObjectInScalar { location, .. }
            | Self::InvalidTensorElement { location, .. }
            | Self::ResourceLimitExceeded { location, .. }
            | Self::MaxDepthExceeded { location, .. }
            | Self::DocumentTooLarge { location, .. }
            | Self::ArrayTooLong { location, .. }
            | Self::Conversion { location, .. } => location.as_ref(),
            // Anchor-related errors don't have Location fields
            Self::ForwardReference { .. }
            | Self::CircularReference { .. }
            | Self::InvalidAnchorName { .. }
            | Self::AnchorRedefinition { .. } => None,
        }
    }

    /// Returns the code snippet, if available.
    #[must_use]
    pub fn snippet(&self) -> Option<&str> {
        match self {
            Self::ParseError { snippet, .. }
            | Self::InvalidRootType { snippet, .. }
            | Self::NonStringKey { snippet, .. }
            | Self::InvalidNumber { snippet, .. }
            | Self::InvalidExpression { snippet, .. }
            | Self::InvalidReference { snippet, .. }
            | Self::NestedObjectInScalar { snippet, .. }
            | Self::InvalidTensorElement { snippet, .. }
            | Self::ResourceLimitExceeded { snippet, .. }
            | Self::MaxDepthExceeded { snippet, .. }
            | Self::DocumentTooLarge { snippet, .. }
            | Self::ArrayTooLong { snippet, .. }
            | Self::Conversion { snippet, .. } => snippet.as_deref(),
            // Anchor-related errors don't have snippet fields
            Self::ForwardReference { .. }
            | Self::CircularReference { .. }
            | Self::InvalidAnchorName { .. }
            | Self::AnchorRedefinition { .. } => None,
        }
    }

    /// Returns the path where the error occurred, if applicable.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        match self {
            Self::NonStringKey { path, .. }
            | Self::NestedObjectInScalar { path, .. }
            | Self::InvalidTensorElement { path, .. }
            | Self::MaxDepthExceeded { path, .. }
            | Self::ArrayTooLong { path, .. } => Some(path),
            _ => None,
        }
    }

    /// Returns helpful suggestions for fixing the error.
    #[must_use]
    pub fn suggestions(&self) -> Vec<String> {
        match self {
            Self::ParseError { .. } => vec![
                "Check YAML syntax for missing or extra colons, brackets, or quotes".to_string(),
                "Ensure proper indentation (YAML is whitespace-sensitive)".to_string(),
                "Verify that strings with special characters are quoted".to_string(),
            ],
            Self::InvalidRootType { found, .. } => vec![
                format!("Expected a YAML mapping at the root, but found {}", found),
                "HEDL documents must start with a mapping (key-value pairs)".to_string(),
                "Example:\nname: value\ncount: 42".to_string(),
            ],
            Self::NonStringKey { key_type, .. } => vec![
                format!("YAML keys must be strings, but found {}", key_type),
                "Convert the key to a string by wrapping it in quotes".to_string(),
                "Example: \"123\": value".to_string(),
            ],
            Self::InvalidNumber { value, .. } => vec![
                format!("The value '{}' is not a valid number", value),
                "Ensure numbers are in a valid format (e.g., 42, 3.14, -10)".to_string(),
            ],
            Self::InvalidExpression { .. } => vec![
                "Expression syntax must be $(...)".to_string(),
                "Example: $(add(x, 1))".to_string(),
                "Check for balanced parentheses and valid identifiers".to_string(),
            ],
            Self::InvalidReference { .. } => vec![
                "Reference format must be @id or @Type:id".to_string(),
                "Use mapping format: { \"@ref\": \"@user1\" }".to_string(),
                "Example: { \"@ref\": \"@User:user1\" }".to_string(),
            ],
            Self::NestedObjectInScalar { .. } => vec![
                "Nested objects are not allowed in this context".to_string(),
                "Consider moving the object to a separate field or list".to_string(),
            ],
            Self::InvalidTensorElement {
                expected, found, ..
            } => vec![
                format!("Tensor elements must be {}, but found {}", expected, found),
                "Ensure all array elements are numbers or nested arrays of numbers".to_string(),
                "Example: [1, 2, 3] or [[1, 2], [3, 4]]".to_string(),
            ],
            Self::ResourceLimitExceeded {
                limit_type,
                limit,
                actual,
                ..
            } => vec![
                format!("{} is {}, exceeding limit of {}", limit_type, actual, limit),
                "Consider reducing the size or increasing the limit".to_string(),
            ],
            Self::MaxDepthExceeded { max_depth, .. } => vec![
                format!("Maximum nesting depth is {}", max_depth),
                "Reduce nesting levels or increase max_nesting_depth in config".to_string(),
                format!(
                    "Use FromYamlConfig::builder().max_nesting_depth({}).build()",
                    max_depth * 2
                ),
            ],
            Self::DocumentTooLarge { max_size, .. } => vec![
                format!("Maximum document size is {} bytes", max_size),
                "Split the document into smaller files or increase max_document_size".to_string(),
                "Use FromYamlConfig::builder().max_document_size(N).build()".to_string(),
            ],
            Self::ArrayTooLong {
                max_length, length, ..
            } => vec![
                format!(
                    "Array has {} elements, exceeding limit of {}",
                    length, max_length
                ),
                format!(
                    "Consider splitting into smaller arrays or increasing max_array_length to {}",
                    length
                ),
                "Use FromYamlConfig::builder().max_array_length(N).build()".to_string(),
            ],
            Self::Conversion { .. } => {
                vec!["Check that the YAML structure matches the expected HEDL format".to_string()]
            }
            Self::ForwardReference { alias, .. } => vec![
                format!(
                    "The alias '*{}' references an anchor that hasn't been defined yet",
                    alias
                ),
                "Define the anchor before using it as an alias".to_string(),
                "Example: &anchor_name value ... *anchor_name".to_string(),
            ],
            Self::CircularReference { anchors, .. } => vec![
                format!(
                    "Circular reference detected involving anchors: {}",
                    anchors.join(", ")
                ),
                "Break the circular dependency by restructuring your YAML".to_string(),
            ],
            Self::InvalidAnchorName { name, reason, .. } => vec![
                format!("The anchor name '{}' is invalid: {}", name, reason),
                "Use alphanumeric characters and underscores for anchor names".to_string(),
            ],
            Self::AnchorRedefinition { name, old_line, .. } => vec![
                format!(
                    "The anchor '{}' was already defined at line {}",
                    name, old_line
                ),
                "Use unique names for each anchor in your document".to_string(),
            ],
        }
    }
}

impl From<serde_yaml::Error> for YamlError {
    fn from(err: serde_yaml::Error) -> Self {
        let location = err.location().map(|loc| Location {
            line: loc.line(),
            column: loc.column(),
            byte_offset: loc.index(),
        });

        YamlError::ParseError {
            message: err.to_string(),
            location,
            snippet: None,
        }
    }
}

impl From<String> for YamlError {
    fn from(err: String) -> Self {
        YamlError::Conversion {
            message: err,
            location: None,
            snippet: None,
        }
    }
}

impl From<&str> for YamlError {
    fn from(err: &str) -> Self {
        YamlError::Conversion {
            message: err.to_string(),
            location: None,
            snippet: None,
        }
    }
}

impl From<hedl_core::lex::LexError> for YamlError {
    fn from(err: hedl_core::lex::LexError) -> Self {
        YamlError::InvalidExpression {
            message: err.to_string(),
            location: None,
            snippet: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_new() {
        let loc = Location::new(10, 5, 123);
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.byte_offset, 123);
    }

    #[test]
    fn test_location_display() {
        let loc = Location::new(42, 10, 456);
        assert_eq!(loc.to_string(), "line 42, column 10");
    }

    #[test]
    fn test_span_new() {
        let start = Location::new(1, 1, 0);
        let end = Location::new(1, 10, 9);
        let span = Span::new(start.clone(), end.clone());
        assert_eq!(span.start, start);
        assert_eq!(span.end, end);
    }

    #[test]
    fn test_parse_error_display() {
        let err = YamlError::ParseError {
            message: "invalid syntax".to_string(),
            location: None,
            snippet: None,
        };
        let display = err.to_string();
        assert!(display.contains("YAML parse error: invalid syntax"));
        // Suggestions are available via the suggestions() method
        assert!(!err.suggestions().is_empty());
    }

    #[test]
    fn test_parse_error_with_location() {
        let err = YamlError::ParseError {
            message: "invalid syntax".to_string(),
            location: Some(Location::new(3, 5, 20)),
            snippet: None,
        };
        let display = err.to_string();
        assert!(display.contains("invalid syntax"));
        // Location is available via the location() method
        let loc = err.location().unwrap();
        assert_eq!(loc.line, 3);
        assert_eq!(loc.column, 5);
        assert_eq!(loc.to_string(), "line 3, column 5");
    }

    #[test]
    fn test_invalid_root_type_display() {
        let err = YamlError::InvalidRootType {
            found: "sequence".to_string(),
            location: None,
            snippet: None,
        };
        let display = err.to_string();
        assert!(display.contains("Root must be a YAML mapping, found sequence"));
        // Suggestions are available via the suggestions() method
        assert!(!err.suggestions().is_empty());
    }

    #[test]
    fn test_non_string_key_display() {
        let err = YamlError::NonStringKey {
            key_type: "number".to_string(),
            path: "root.config".to_string(),
            location: None,
            snippet: None,
        };
        let display = err.to_string();
        assert!(display.contains("Non-string keys not supported"));
        assert!(display.contains("number"));
        assert!(display.contains("root.config"));
    }

    #[test]
    fn test_resource_limit_exceeded_display() {
        let err = YamlError::ResourceLimitExceeded {
            limit_type: "array_length".to_string(),
            limit: 1000,
            actual: 2000,
            location: None,
            snippet: None,
        };
        let display = err.to_string();
        assert!(display.contains("Resource limit exceeded"));
        assert!(display.contains("1000"));
        assert!(display.contains("2000"));
    }

    #[test]
    fn test_max_depth_exceeded_display() {
        let err = YamlError::MaxDepthExceeded {
            max_depth: 100,
            actual_depth: 150,
            path: "root.deep.path".to_string(),
            location: None,
            snippet: None,
        };
        let display = err.to_string();
        assert!(display.contains("Maximum nesting depth"));
        assert!(display.contains("100"));
        assert!(display.contains("150"));
        assert!(display.contains("root.deep.path"));
    }

    #[test]
    fn test_document_too_large_display() {
        let err = YamlError::DocumentTooLarge {
            size: 20_000_000,
            max_size: 10_000_000,
            location: None,
            snippet: None,
        };
        let display = err.to_string();
        assert!(display.contains("Document size"));
        assert!(display.contains("20000000"));
        assert!(display.contains("10000000"));
    }

    #[test]
    fn test_array_too_long_display() {
        let err = YamlError::ArrayTooLong {
            length: 2000,
            max_length: 1000,
            path: "root.items".to_string(),
            location: None,
            snippet: None,
        };
        let display = err.to_string();
        assert!(display.contains("Array length"));
        assert!(display.contains("2000"));
        assert!(display.contains("1000"));
        assert!(display.contains("root.items"));
    }

    #[test]
    fn test_error_clone() {
        let err1 = YamlError::ParseError {
            message: "test".to_string(),
            location: None,
            snippet: None,
        };
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_error_equality() {
        let err1 = YamlError::ParseError {
            message: "test".to_string(),
            location: None,
            snippet: None,
        };
        let err2 = YamlError::ParseError {
            message: "test".to_string(),
            location: None,
            snippet: None,
        };
        let err3 = YamlError::ParseError {
            message: "different".to_string(),
            location: None,
            snippet: None,
        };

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }

    #[test]
    fn test_from_string() {
        let err: YamlError = "test error".to_string().into();
        match err {
            YamlError::Conversion { message, .. } => assert_eq!(message, "test error"),
            _ => panic!("Expected Conversion error"),
        }
    }

    #[test]
    fn test_from_str() {
        let err: YamlError = "test error".into();
        match err {
            YamlError::Conversion { message, .. } => assert_eq!(message, "test error"),
            _ => panic!("Expected Conversion error"),
        }
    }

    #[test]
    fn test_forward_reference_display() {
        let err = YamlError::ForwardReference {
            alias: "undefined".to_string(),
            line: 5,
        };
        assert_eq!(
            err.to_string(),
            "Forward reference: alias '*undefined' at line 5 references undefined anchor"
        );
    }

    #[test]
    fn test_circular_reference_display() {
        let err = YamlError::CircularReference {
            cycle_path: "a -> b -> c -> a".to_string(),
            anchors: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            locations: vec![1, 5, 10],
        };
        assert_eq!(
            err.to_string(),
            "Circular anchor reference: a -> b -> c -> a"
        );
    }

    #[test]
    fn test_invalid_anchor_name_display() {
        let err = YamlError::InvalidAnchorName {
            name: "__reserved".to_string(),
            reason: "Names starting with __ are reserved".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid anchor name '__reserved': Names starting with __ are reserved"
        );
    }

    #[test]
    fn test_anchor_redefinition_display() {
        let err = YamlError::AnchorRedefinition {
            name: "anchor1".to_string(),
            old_line: 5,
            new_line: 10,
        };
        assert_eq!(
            err.to_string(),
            "Anchor 'anchor1' redefined at line 10 (previously defined at line 5)"
        );
    }

    #[test]
    fn test_location_method() {
        let loc = Location::new(5, 10, 50);
        let err = YamlError::ParseError {
            message: "test".to_string(),
            location: Some(loc.clone()),
            snippet: None,
        };
        assert_eq!(err.location(), Some(&loc));
    }

    #[test]
    fn test_snippet_method() {
        let err = YamlError::ParseError {
            message: "test".to_string(),
            location: None,
            snippet: Some("test snippet".to_string()),
        };
        assert_eq!(err.snippet(), Some("test snippet"));
    }

    #[test]
    fn test_path_method() {
        let err = YamlError::NonStringKey {
            key_type: "number".to_string(),
            path: "root.items".to_string(),
            location: None,
            snippet: None,
        };
        assert_eq!(err.path(), Some("root.items"));
    }

    #[test]
    fn test_suggestions_parse_error() {
        let err = YamlError::ParseError {
            message: "test".to_string(),
            location: None,
            snippet: None,
        };
        let suggestions = err.suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("syntax"));
    }

    #[test]
    fn test_suggestions_non_string_key() {
        let err = YamlError::NonStringKey {
            key_type: "number".to_string(),
            path: "test".to_string(),
            location: None,
            snippet: None,
        };
        let suggestions = err.suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].contains("strings"));
    }

    #[test]
    fn test_error_with_all_fields() {
        let loc = Location::new(10, 5, 100);
        let err = YamlError::NonStringKey {
            key_type: "number".to_string(),
            path: "root.config".to_string(),
            location: Some(loc),
            snippet: Some("  123: value".to_string()),
        };

        // Check base message contains path
        let display = err.to_string();
        assert!(display.contains("root.config"));
        assert!(display.contains("number"));

        // Location is available via method
        let location = err.location().unwrap();
        assert_eq!(location.line, 10);
        assert_eq!(location.column, 5);
        assert_eq!(location.to_string(), "line 10, column 5");

        // Snippet is available via method
        assert_eq!(err.snippet().unwrap(), "  123: value");

        // Suggestions are available via method
        let suggestions = err.suggestions();
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_from_serde_yaml_error() {
        // Create a malformed YAML to generate a serde_yaml error
        let yaml = "{ invalid: [";
        let result: Result<serde_yaml::Value, serde_yaml::Error> = serde_yaml::from_str(yaml);
        assert!(result.is_err());

        let serde_err = result.unwrap_err();
        let yaml_err: YamlError = serde_err.into();

        match yaml_err {
            YamlError::ParseError {
                message, location, ..
            } => {
                assert!(!message.is_empty());
                // Location may or may not be present depending on the error
                if let Some(loc) = location {
                    assert!(loc.line > 0);
                    assert!(loc.column > 0);
                }
            }
            _ => panic!("Expected ParseError"),
        }
    }
}
