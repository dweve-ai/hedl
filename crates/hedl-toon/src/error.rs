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

//! Error types for TOON conversion
//!
//! This module defines all errors that can occur during HEDL to TOON conversion.

use thiserror::Error;

/// Maximum allowed nesting depth to prevent stack overflow attacks
///
/// This limit protects against malicious documents with extremely deep nesting
/// that could cause stack overflow. A depth of 100 is sufficient for any
/// reasonable document while preventing DoS attacks.
pub const MAX_NESTING_DEPTH: usize = 100;

/// Errors that can occur during TOON conversion
///
/// # Examples
///
/// ```
/// use hedl_toon::{to_toon, ToToonConfig, ToonError};
/// use hedl_core::Document;
///
/// let doc = Document::new((1, 0));
/// let result = to_toon(&doc, &ToToonConfig::default());
/// match result {
///     Ok(toon_string) => println!("Success: {}", toon_string),
///     Err(ToonError::MaxDepthExceeded { depth, max }) => {
///         eprintln!("Document too deeply nested: {} > {}", depth, max);
///     }
///     Err(ToonError::InvalidIndent(indent)) => {
///         eprintln!("Invalid indentation: {}", indent);
///     }
///     Err(ToonError::SchemaMismatch { type_name, expected, actual }) => {
///         eprintln!("Schema mismatch for {}: expected {} fields, got {}", type_name, expected, actual);
///     }
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ToonError {
    /// Nesting depth exceeded maximum allowed
    ///
    /// This error occurs when the document structure is nested too deeply,
    /// which could indicate a malicious document designed to cause stack
    /// overflow. The maximum depth is [`MAX_NESTING_DEPTH`].
    ///
    /// # Security
    ///
    /// This protection prevents stack overflow DoS attacks by limiting
    /// recursion depth during document traversal.
    #[error("Maximum nesting depth exceeded: {depth} > {max}")]
    MaxDepthExceeded {
        /// The depth that was reached
        depth: usize,
        /// The maximum allowed depth
        max: usize,
    },

    /// Invalid indentation configuration
    ///
    /// Indentation must be at least 1 space per level.
    #[error("Invalid indentation: {0} (must be at least 1)")]
    InvalidIndent(usize),

    /// Schema mismatch between expected and actual field counts
    ///
    /// This indicates a structural inconsistency in the document where
    /// a node has a different number of fields than its schema declares.
    #[error("Schema mismatch for type {type_name}: expected {expected} fields, got {actual}")]
    SchemaMismatch {
        /// The type name with the mismatch
        type_name: String,
        /// Expected number of fields
        expected: usize,
        /// Actual number of fields
        actual: usize,
    },

    // ========================================================================
    // Parsing errors (from_toon)
    // ========================================================================

    /// Invalid TOON syntax at a specific line
    #[error("Parse error at line {line}: {message}")]
    ParseError {
        /// Line number (1-indexed)
        line: usize,
        /// Description of the error
        message: String,
    },

    /// Unexpected end of input
    #[error("Unexpected end of input: {0}")]
    UnexpectedEof(String),

    /// Invalid array header format
    #[error("Invalid array header at line {line}: {message}")]
    InvalidArrayHeader {
        /// Line number (1-indexed)
        line: usize,
        /// Description of the error
        message: String,
    },

    /// Invalid value format
    #[error("Invalid value at line {line}: {message}")]
    InvalidValue {
        /// Line number (1-indexed)
        line: usize,
        /// Description of the error
        message: String,
    },

    /// Indentation error
    #[error("Indentation error at line {line}: {message}")]
    IndentationError {
        /// Line number (1-indexed)
        line: usize,
        /// Description of the error
        message: String,
    },
}

/// Result type for TOON conversion operations
///
/// This is a convenience alias for `Result<T, ToonError>`.
pub type Result<T> = std::result::Result<T, ToonError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ToonError::MaxDepthExceeded {
            depth: 150,
            max: MAX_NESTING_DEPTH,
        };
        let msg = err.to_string();
        assert!(msg.contains("150"));
        assert!(msg.contains("100"));
    }

    #[test]
    fn test_error_equality() {
        let err1 = ToonError::MaxDepthExceeded {
            depth: 150,
            max: 100,
        };
        let err2 = ToonError::MaxDepthExceeded {
            depth: 150,
            max: 100,
        };
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_invalid_indent() {
        let err = ToonError::InvalidIndent(0);
        assert!(err.to_string().contains("Invalid indentation"));
    }

    #[test]
    fn test_schema_mismatch() {
        let err = ToonError::SchemaMismatch {
            type_name: "User".to_string(),
            expected: 3,
            actual: 2,
        };
        let msg = err.to_string();
        assert!(msg.contains("User"));
        assert!(msg.contains("3"));
        assert!(msg.contains("2"));
    }

    #[test]
    fn test_max_depth_constant() {
        assert_eq!(MAX_NESTING_DEPTH, 100);
    }
}
