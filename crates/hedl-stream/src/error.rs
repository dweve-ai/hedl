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

//! Error types for streaming parser.
//!
//! This module defines all error types that can occur during HEDL streaming
//! parsing. All errors include contextual information (like line numbers)
//! to aid debugging.
//!
//! # Error Categories
//!
//! - **I/O Errors**: Problems reading the input stream
//! - **Syntax Errors**: Malformed HEDL syntax
//! - **Schema Errors**: Type/schema definition issues
//! - **Validation Errors**: Data doesn't match schema
//! - **Timeout Errors**: Parsing exceeded time limit
//!
//! # Error Handling Examples
//!
//! ## Basic Error Handling
//!
//! ```rust
//! use hedl_stream::{StreamingParser, StreamError};
//! use std::io::Cursor;
//!
//! let bad_input = r#"
//! %VERSION: 1.0
//! ---
//! invalid line without colon
//! "#;
//!
//! let parser = StreamingParser::new(Cursor::new(bad_input)).unwrap();
//!
//! for event in parser {
//!     if let Err(e) = event {
//!         eprintln!("Error: {}", e);
//!         if let Some(line) = e.line() {
//!             eprintln!("  at line {}", line);
//!         }
//!     }
//! }
//! ```
//!
//! ## Match on Error Type
//!
//! ```rust
//! use hedl_stream::{StreamingParser, StreamError};
//! use std::io::Cursor;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let parser = StreamingParser::new(Cursor::new("..."))?;
//!
//! for event in parser {
//!     match event {
//!         Ok(event) => { /* process */ }
//!         Err(StreamError::Timeout { elapsed, limit }) => {
//!             eprintln!("Timeout: took {:?}, limit {:?}", elapsed, limit);
//!             break;
//!         }
//!         Err(StreamError::ShapeMismatch { line, expected, got }) => {
//!             eprintln!("Line {}: column mismatch (expected {}, got {})",
//!                 line, expected, got);
//!         }
//!         Err(e) => {
//!             eprintln!("Other error: {}", e);
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use thiserror::Error;

/// Errors that can occur during streaming parsing.
///
/// All variants include contextual information to help diagnose and fix issues.
/// Most errors include line numbers; use the [`line()`](Self::line) method to
/// extract them uniformly.
///
/// # Examples
///
/// ## Creating Errors
///
/// ```rust
/// use hedl_stream::StreamError;
///
/// let err = StreamError::syntax(42, "unexpected token");
/// assert_eq!(err.line(), Some(42));
///
/// let schema_err = StreamError::schema(10, "type not found");
/// assert_eq!(schema_err.line(), Some(10));
/// ```
///
/// ## Error Display
///
/// ```rust
/// use hedl_stream::StreamError;
///
/// let err = StreamError::syntax(5, "missing colon");
/// let msg = format!("{}", err);
/// assert!(msg.contains("line 5"));
/// assert!(msg.contains("missing colon"));
/// ```
#[derive(Error, Debug)]
pub enum StreamError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid UTF-8 encoding.
    #[error("Invalid UTF-8 at line {line}: {message}")]
    Utf8 { line: usize, message: String },

    /// Syntax error.
    #[error("Syntax error at line {line}: {message}")]
    Syntax { line: usize, message: String },

    /// Schema error.
    #[error("Schema error at line {line}: {message}")]
    Schema { line: usize, message: String },

    /// Invalid header.
    #[error("Invalid header: {0}")]
    Header(String),

    /// Missing version directive.
    #[error("Missing %VERSION directive")]
    MissingVersion,

    /// Invalid version.
    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    /// Orphan row (child without parent).
    #[error("Orphan row at line {line}: {message}")]
    OrphanRow { line: usize, message: String },

    /// Shape mismatch.
    #[error("Shape mismatch at line {line}: expected {expected} columns, got {got}")]
    ShapeMismatch {
        line: usize,
        expected: usize,
        got: usize,
    },

    /// Timeout exceeded during parsing.
    #[error("Parsing timeout: elapsed {elapsed:?} exceeded limit {limit:?}")]
    Timeout {
        elapsed: std::time::Duration,
        limit: std::time::Duration,
    },
}

impl StreamError {
    /// Create a syntax error.
    #[inline]
    pub fn syntax(line: usize, message: impl Into<String>) -> Self {
        Self::Syntax {
            line,
            message: message.into(),
        }
    }

    /// Create a schema error.
    #[inline]
    pub fn schema(line: usize, message: impl Into<String>) -> Self {
        Self::Schema {
            line,
            message: message.into(),
        }
    }

    /// Create an orphan row error.
    #[inline]
    pub fn orphan_row(line: usize, message: impl Into<String>) -> Self {
        Self::OrphanRow {
            line,
            message: message.into(),
        }
    }

    /// Get the line number if available.
    #[inline]
    pub fn line(&self) -> Option<usize> {
        match self {
            Self::Utf8 { line, .. }
            | Self::Syntax { line, .. }
            | Self::Schema { line, .. }
            | Self::OrphanRow { line, .. }
            | Self::ShapeMismatch { line, .. } => Some(*line),
            _ => None,
        }
    }
}

/// Result type for streaming operations.
pub type StreamResult<T> = Result<T, StreamError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // ==================== StreamError variant tests ====================

    #[test]
    fn test_stream_error_io() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = StreamError::Io(io_err);
        let display = format!("{}", err);
        assert!(display.contains("IO error"));
        assert!(display.contains("file not found"));
    }

    #[test]
    fn test_stream_error_utf8() {
        let err = StreamError::Utf8 {
            line: 42,
            message: "invalid byte sequence".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Invalid UTF-8"));
        assert!(display.contains("42"));
        assert!(display.contains("invalid byte sequence"));
    }

    #[test]
    fn test_stream_error_syntax() {
        let err = StreamError::Syntax {
            line: 10,
            message: "unexpected token".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Syntax error"));
        assert!(display.contains("10"));
        assert!(display.contains("unexpected token"));
    }

    #[test]
    fn test_stream_error_schema() {
        let err = StreamError::Schema {
            line: 5,
            message: "undefined type".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Schema error"));
        assert!(display.contains("5"));
        assert!(display.contains("undefined type"));
    }

    #[test]
    fn test_stream_error_header() {
        let err = StreamError::Header("invalid header format".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Invalid header"));
        assert!(display.contains("invalid header format"));
    }

    #[test]
    fn test_stream_error_missing_version() {
        let err = StreamError::MissingVersion;
        let display = format!("{}", err);
        assert!(display.contains("Missing %VERSION"));
    }

    #[test]
    fn test_stream_error_invalid_version() {
        let err = StreamError::InvalidVersion("abc".to_string());
        let display = format!("{}", err);
        assert!(display.contains("Invalid version"));
        assert!(display.contains("abc"));
    }

    #[test]
    fn test_stream_error_orphan_row() {
        let err = StreamError::OrphanRow {
            line: 25,
            message: "child without parent".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Orphan row"));
        assert!(display.contains("25"));
        assert!(display.contains("child without parent"));
    }

    #[test]
    fn test_stream_error_shape_mismatch() {
        let err = StreamError::ShapeMismatch {
            line: 100,
            expected: 5,
            got: 3,
        };
        let display = format!("{}", err);
        assert!(display.contains("Shape mismatch"));
        assert!(display.contains("100"));
        assert!(display.contains("5"));
        assert!(display.contains("3"));
    }

    // ==================== Constructor tests ====================

    #[test]
    fn test_syntax_constructor() {
        let err = StreamError::syntax(15, "invalid syntax");
        if let StreamError::Syntax { line, message } = err {
            assert_eq!(line, 15);
            assert_eq!(message, "invalid syntax");
        } else {
            panic!("Expected Syntax variant");
        }
    }

    #[test]
    fn test_syntax_constructor_string() {
        let err = StreamError::syntax(20, String::from("detailed error"));
        if let StreamError::Syntax { line, message } = err {
            assert_eq!(line, 20);
            assert_eq!(message, "detailed error");
        } else {
            panic!("Expected Syntax variant");
        }
    }

    #[test]
    fn test_schema_constructor() {
        let err = StreamError::schema(30, "type not found");
        if let StreamError::Schema { line, message } = err {
            assert_eq!(line, 30);
            assert_eq!(message, "type not found");
        } else {
            panic!("Expected Schema variant");
        }
    }

    #[test]
    fn test_schema_constructor_string() {
        let err = StreamError::schema(35, String::from("schema validation failed"));
        if let StreamError::Schema { line, message } = err {
            assert_eq!(line, 35);
            assert_eq!(message, "schema validation failed");
        } else {
            panic!("Expected Schema variant");
        }
    }

    #[test]
    fn test_orphan_row_constructor() {
        let err = StreamError::orphan_row(50, "no parent context");
        if let StreamError::OrphanRow { line, message } = err {
            assert_eq!(line, 50);
            assert_eq!(message, "no parent context");
        } else {
            panic!("Expected OrphanRow variant");
        }
    }

    #[test]
    fn test_orphan_row_constructor_string() {
        let err = StreamError::orphan_row(55, String::from("orphan details"));
        if let StreamError::OrphanRow { line, message } = err {
            assert_eq!(line, 55);
            assert_eq!(message, "orphan details");
        } else {
            panic!("Expected OrphanRow variant");
        }
    }

    // ==================== line() method tests ====================

    #[test]
    fn test_line_utf8() {
        let err = StreamError::Utf8 {
            line: 10,
            message: "test".to_string(),
        };
        assert_eq!(err.line(), Some(10));
    }

    #[test]
    fn test_line_syntax() {
        let err = StreamError::syntax(20, "test");
        assert_eq!(err.line(), Some(20));
    }

    #[test]
    fn test_line_schema() {
        let err = StreamError::schema(30, "test");
        assert_eq!(err.line(), Some(30));
    }

    #[test]
    fn test_line_orphan_row() {
        let err = StreamError::orphan_row(40, "test");
        assert_eq!(err.line(), Some(40));
    }

    #[test]
    fn test_line_shape_mismatch() {
        let err = StreamError::ShapeMismatch {
            line: 50,
            expected: 3,
            got: 2,
        };
        assert_eq!(err.line(), Some(50));
    }

    #[test]
    fn test_line_io_none() {
        let io_err = io::Error::other("test");
        let err = StreamError::Io(io_err);
        assert_eq!(err.line(), None);
    }

    #[test]
    fn test_line_header_none() {
        let err = StreamError::Header("test".to_string());
        assert_eq!(err.line(), None);
    }

    #[test]
    fn test_line_missing_version_none() {
        let err = StreamError::MissingVersion;
        assert_eq!(err.line(), None);
    }

    #[test]
    fn test_line_invalid_version_none() {
        let err = StreamError::InvalidVersion("1.x".to_string());
        assert_eq!(err.line(), None);
    }

    // ==================== From<io::Error> tests ====================

    #[test]
    fn test_from_io_error() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let stream_err: StreamError = io_err.into();
        assert!(matches!(stream_err, StreamError::Io(_)));
        let display = format!("{}", stream_err);
        assert!(display.contains("access denied"));
    }

    #[test]
    fn test_from_io_error_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
        let stream_err: StreamError = io_err.into();
        assert!(matches!(stream_err, StreamError::Io(_)));
    }

    // ==================== Debug tests ====================

    #[test]
    fn test_debug_syntax() {
        let err = StreamError::syntax(10, "test error");
        let debug = format!("{:?}", err);
        assert!(debug.contains("Syntax"));
        assert!(debug.contains("10"));
    }

    #[test]
    fn test_debug_schema() {
        let err = StreamError::schema(20, "schema issue");
        let debug = format!("{:?}", err);
        assert!(debug.contains("Schema"));
    }

    #[test]
    fn test_debug_missing_version() {
        let err = StreamError::MissingVersion;
        let debug = format!("{:?}", err);
        assert!(debug.contains("MissingVersion"));
    }

    // ==================== Edge case tests ====================

    #[test]
    fn test_line_zero() {
        let err = StreamError::syntax(0, "at start");
        assert_eq!(err.line(), Some(0));
    }

    #[test]
    fn test_line_max() {
        let err = StreamError::syntax(usize::MAX, "at end");
        assert_eq!(err.line(), Some(usize::MAX));
    }

    #[test]
    fn test_empty_message() {
        let err = StreamError::syntax(1, "");
        if let StreamError::Syntax { message, .. } = err {
            assert!(message.is_empty());
        }
    }

    #[test]
    fn test_unicode_message() {
        let err = StreamError::syntax(1, "错误信息 🚫");
        let display = format!("{}", err);
        assert!(display.contains("错误信息"));
        assert!(display.contains("🚫"));
    }

    #[test]
    fn test_multiline_message() {
        let err = StreamError::syntax(1, "line1\nline2\nline3");
        let display = format!("{}", err);
        assert!(display.contains("line1"));
    }

    #[test]
    fn test_shape_mismatch_zero_columns() {
        let err = StreamError::ShapeMismatch {
            line: 1,
            expected: 0,
            got: 0,
        };
        assert_eq!(err.line(), Some(1));
    }

    #[test]
    fn test_shape_mismatch_large_numbers() {
        let err = StreamError::ShapeMismatch {
            line: 1000000,
            expected: 1000,
            got: 999,
        };
        let display = format!("{}", err);
        assert!(display.contains("1000000"));
        assert!(display.contains("1000"));
        assert!(display.contains("999"));
    }

    // ==================== Timeout error tests ====================

    #[test]
    fn test_stream_error_timeout() {
        use std::time::Duration;

        let elapsed = Duration::from_millis(150);
        let limit = Duration::from_millis(100);
        let err = StreamError::Timeout { elapsed, limit };

        let display = format!("{}", err);
        assert!(display.contains("timeout"));
        assert!(display.contains("150ms"));
        assert!(display.contains("100ms"));
    }

    #[test]
    fn test_timeout_error_debug() {
        use std::time::Duration;

        let err = StreamError::Timeout {
            elapsed: Duration::from_secs(1),
            limit: Duration::from_millis(500),
        };

        let debug = format!("{:?}", err);
        assert!(debug.contains("Timeout"));
    }

    #[test]
    fn test_timeout_error_no_line() {
        use std::time::Duration;

        let err = StreamError::Timeout {
            elapsed: Duration::from_millis(200),
            limit: Duration::from_millis(100),
        };

        // Timeout errors don't have line numbers
        assert_eq!(err.line(), None);
    }

    #[test]
    fn test_timeout_elapsed_greater_than_limit() {
        use std::time::Duration;

        let elapsed = Duration::from_secs(5);
        let limit = Duration::from_secs(1);
        let err = StreamError::Timeout { elapsed, limit };

        let display = format!("{}", err);
        assert!(display.contains("5s"));
        assert!(display.contains("1s"));
    }

    #[test]
    fn test_timeout_with_nanoseconds() {
        use std::time::Duration;

        let elapsed = Duration::from_nanos(1500);
        let limit = Duration::from_nanos(1000);
        let err = StreamError::Timeout { elapsed, limit };

        let display = format!("{}", err);
        assert!(display.contains("timeout"));
    }
}
