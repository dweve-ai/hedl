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

//! Error types for CSV conversion operations.

use thiserror::Error;

/// CSV conversion error types.
///
/// This enum provides structured error handling for CSV parsing and generation,
/// with contextual information to help diagnose issues.
///
/// # Examples
///
/// ```
/// use hedl_csv::CsvError;
///
/// let err = CsvError::TypeMismatch {
///     column: "age".to_string(),
///     expected: "integer".to_string(),
///     value: "abc".to_string(),
/// };
///
/// assert_eq!(
///     err.to_string(),
///     "Type mismatch in column 'age': expected integer, got 'abc'"
/// );
/// ```
#[derive(Debug, Error)]
pub enum CsvError {
    /// CSV parsing error at a specific line.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::ParseError {
    ///     line: 42,
    ///     message: "Invalid escape sequence".to_string(),
    /// };
    /// assert!(err.to_string().contains("line 42"));
    /// ```
    #[error("CSV parse error at line {line}: {message}")]
    ParseError {
        /// Line number where the error occurred (1-based).
        line: usize,
        /// Detailed error message.
        message: String,
    },

    /// Type mismatch when converting values.
    ///
    /// This error occurs when a CSV field value cannot be converted to the expected type.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::TypeMismatch {
    ///     column: "price".to_string(),
    ///     expected: "float".to_string(),
    ///     value: "not-a-number".to_string(),
    /// };
    /// ```
    #[error("Type mismatch in column '{column}': expected {expected}, got '{value}'")]
    TypeMismatch {
        /// Column name where the mismatch occurred.
        column: String,
        /// Expected type description.
        expected: String,
        /// Actual value that failed to convert.
        value: String,
    },

    /// Missing required column in CSV data.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::MissingColumn("id".to_string());
    /// assert_eq!(err.to_string(), "Missing required column: id");
    /// ```
    #[error("Missing required column: {0}")]
    MissingColumn(String),

    /// Invalid header format or content.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::InvalidHeader {
    ///     position: 0,
    ///     reason: "Empty column name".to_string(),
    /// };
    /// ```
    #[error("Invalid header at position {position}: {reason}")]
    InvalidHeader {
        /// Position of the invalid header (0-based).
        position: usize,
        /// Reason the header is invalid.
        reason: String,
    },

    /// Row has wrong number of columns.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::WidthMismatch {
    ///     expected: 5,
    ///     actual: 3,
    ///     row: 10,
    /// };
    /// assert!(err.to_string().contains("expected 5 columns"));
    /// assert!(err.to_string().contains("got 3"));
    /// ```
    #[error("Row width mismatch: expected {expected} columns, got {actual} in row {row}")]
    WidthMismatch {
        /// Expected number of columns.
        expected: usize,
        /// Actual number of columns in the row.
        actual: usize,
        /// Row number where the mismatch occurred (1-based).
        row: usize,
    },

    /// I/O error during CSV reading or writing.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    /// use std::io;
    ///
    /// let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    /// let csv_err = CsvError::from(io_err);
    /// ```
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error from underlying CSV library.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// // This error type wraps csv::Error transparently
    /// ```
    #[error("CSV library error: {0}")]
    CsvLib(#[from] csv::Error),

    /// HEDL core error during conversion.
    ///
    /// This wraps errors from the `hedl_core` crate when they occur during
    /// CSV conversion operations.
    #[error("HEDL core error: {0}")]
    HedlCore(String),

    /// Row count exceeded security limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::SecurityLimit {
    ///     limit: 1_000_000,
    ///     actual: 1_000_001,
    /// };
    /// assert!(err.to_string().contains("Security limit"));
    /// ```
    #[error("Security limit exceeded: row count {actual} exceeds maximum {limit}")]
    SecurityLimit {
        /// Maximum allowed rows.
        limit: usize,
        /// Actual row count encountered.
        actual: usize,
    },

    /// Empty ID field in CSV data.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::EmptyId { row: 5 };
    /// assert_eq!(err.to_string(), "Empty 'id' field at row 5");
    /// ```
    #[error("Empty 'id' field at row {row}")]
    EmptyId {
        /// Row number with empty ID (1-based).
        row: usize,
    },

    /// Matrix list not found in document.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::ListNotFound {
    ///     name: "people".to_string(),
    ///     available: "users, items".to_string(),
    /// };
    /// assert!(err.to_string().contains("not found"));
    /// ```
    #[error("Matrix list '{name}' not found in document (available: {available})")]
    ListNotFound {
        /// Name of the list that was not found.
        name: String,
        /// Available list names in the document.
        available: String,
    },

    /// Item is not a matrix list.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::NotAList {
    ///     name: "value".to_string(),
    ///     actual_type: "scalar".to_string(),
    /// };
    /// ```
    #[error("Item '{name}' is not a matrix list (found: {actual_type})")]
    NotAList {
        /// Name of the item.
        name: String,
        /// Actual type of the item.
        actual_type: String,
    },

    /// No matrix lists found in document.
    #[error("No matrix lists found in document")]
    NoLists,

    /// Invalid UTF-8 in CSV output.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::InvalidUtf8 {
    ///     context: "CSV serialization".to_string(),
    /// };
    /// ```
    #[error("Invalid UTF-8 in {context}")]
    InvalidUtf8 {
        /// Context where the invalid UTF-8 was encountered.
        context: String,
    },

    /// Generic error with custom message.
    ///
    /// This is a catch-all for errors that don't fit other categories.
    #[error("{0}")]
    Other(String),
}

/// Convenience type alias for `Result` with `CsvError`.
pub type Result<T> = std::result::Result<T, CsvError>;

impl CsvError {
    /// Add context to an error message.
    ///
    /// This is useful for providing additional information about where an error occurred.
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_csv::CsvError;
    ///
    /// let err = CsvError::ParseError {
    ///     line: 5,
    ///     message: "Invalid value".to_string(),
    /// };
    /// let with_context = err.with_context("in column 'age' at line 10".to_string());
    /// ```
    pub fn with_context(self, context: String) -> Self {
        match self {
            CsvError::ParseError { line, message } => CsvError::ParseError {
                line,
                message: format!("{} ({})", message, context),
            },
            CsvError::HedlCore(msg) => CsvError::HedlCore(format!("{} ({})", msg, context)),
            CsvError::Other(msg) => CsvError::Other(format!("{} ({})", msg, context)),
            // For other variants, wrap in Other with context
            other => CsvError::Other(format!("{} ({})", other, context)),
        }
    }
}

impl From<hedl_core::HedlError> for CsvError {
    fn from(err: hedl_core::HedlError) -> Self {
        CsvError::HedlCore(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = CsvError::ParseError {
            line: 42,
            message: "Invalid escape sequence".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "CSV parse error at line 42: Invalid escape sequence"
        );
    }

    #[test]
    fn test_type_mismatch_display() {
        let err = CsvError::TypeMismatch {
            column: "age".to_string(),
            expected: "integer".to_string(),
            value: "abc".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Type mismatch in column 'age': expected integer, got 'abc'"
        );
    }

    #[test]
    fn test_missing_column_display() {
        let err = CsvError::MissingColumn("id".to_string());
        assert_eq!(err.to_string(), "Missing required column: id");
    }

    #[test]
    fn test_invalid_header_display() {
        let err = CsvError::InvalidHeader {
            position: 3,
            reason: "Empty column name".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid header at position 3: Empty column name"
        );
    }

    #[test]
    fn test_width_mismatch_display() {
        let err = CsvError::WidthMismatch {
            expected: 5,
            actual: 3,
            row: 10,
        };
        assert_eq!(
            err.to_string(),
            "Row width mismatch: expected 5 columns, got 3 in row 10"
        );
    }

    #[test]
    fn test_security_limit_display() {
        let err = CsvError::SecurityLimit {
            limit: 1_000_000,
            actual: 1_500_000,
        };
        assert_eq!(
            err.to_string(),
            "Security limit exceeded: row count 1500000 exceeds maximum 1000000"
        );
    }

    #[test]
    fn test_empty_id_display() {
        let err = CsvError::EmptyId { row: 5 };
        assert_eq!(err.to_string(), "Empty 'id' field at row 5");
    }

    #[test]
    fn test_list_not_found_display() {
        let err = CsvError::ListNotFound {
            name: "people".to_string(),
            available: "users, items".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Matrix list 'people' not found in document (available: users, items)"
        );
    }

    #[test]
    fn test_not_a_list_display() {
        let err = CsvError::NotAList {
            name: "value".to_string(),
            actual_type: "scalar".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Item 'value' is not a matrix list (found: scalar)"
        );
    }

    #[test]
    fn test_no_lists_display() {
        let err = CsvError::NoLists;
        assert_eq!(err.to_string(), "No matrix lists found in document");
    }

    #[test]
    fn test_invalid_utf8_display() {
        let err = CsvError::InvalidUtf8 {
            context: "CSV output".to_string(),
        };
        assert_eq!(err.to_string(), "Invalid UTF-8 in CSV output");
    }

    #[test]
    fn test_other_display() {
        let err = CsvError::Other("Custom error message".to_string());
        assert_eq!(err.to_string(), "Custom error message");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let csv_err = CsvError::from(io_err);
        assert!(csv_err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_hedl_error_conversion() {
        let hedl_err = hedl_core::HedlError::new(
            hedl_core::HedlErrorKind::Syntax,
            "Syntax error".to_string(),
            1,
        );
        let csv_err = CsvError::from(hedl_err);
        assert!(csv_err.to_string().contains("HEDL core error"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CsvError>();
    }

    #[test]
    fn test_error_debug() {
        let err = CsvError::MissingColumn("id".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("MissingColumn"));
        assert!(debug.contains("id"));
    }

    #[test]
    fn test_error_messages() {
        let err = CsvError::TypeMismatch {
            column: "age".to_string(),
            expected: "integer".to_string(),
            value: "abc".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Type mismatch in column 'age': expected integer, got 'abc'"
        );
    }

    #[test]
    fn test_with_context() {
        let err = CsvError::ParseError {
            line: 10,
            message: "Invalid value".to_string(),
        };
        let with_ctx = err.with_context("in field 'name'".to_string());
        assert_eq!(
            with_ctx.to_string(),
            "CSV parse error at line 10: Invalid value (in field 'name')"
        );
    }
}
