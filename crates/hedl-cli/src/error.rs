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

//! Structured error types for the HEDL CLI.
//!
//! This module provides type-safe, composable error handling using `thiserror`.
//! All CLI operations return `Result<T, CliError>` for consistent error reporting.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// The main error type for HEDL CLI operations.
///
/// This enum represents all possible error conditions that can occur during
/// CLI command execution. Each variant provides rich context for debugging
/// and user-friendly error messages.
///
/// # Cloning
///
/// Implements `Clone` to support parallel error handling in multi-threaded
/// operations.
///
/// # Examples
///
/// ```rust,no_run
/// use hedl_cli::error::CliError;
///
/// fn read_and_parse(path: &str) -> Result<(), CliError> {
///     // Error is automatically converted and contextualized
///     let content = std::fs::read_to_string(path)
///         .map_err(|e| CliError::io_error(path, e))?;
///     Ok(())
/// }
/// ```
#[derive(Error, Debug, Clone)]
pub enum CliError {
    /// I/O operation failed (file read, write, or metadata access).
    ///
    /// This error includes the file path and the error kind/message.
    #[error("I/O error for '{path}': {message}")]
    Io {
        /// The file path that caused the error
        path: PathBuf,
        /// The error message
        message: String,
    },

    /// File size exceeds the maximum allowed limit (100 MB).
    ///
    /// This prevents denial-of-service attacks via memory exhaustion.
    /// The error includes the actual file size and the configured limit.
    #[error(
        "File '{path}' is too large ({actual} bytes). Maximum allowed: {max} bytes ({max_mb} MB)"
    )]
    FileTooLarge {
        /// The file path that exceeded the limit
        path: PathBuf,
        /// The actual file size in bytes
        actual: u64,
        /// The maximum allowed file size in bytes
        max: u64,
        /// The maximum allowed file size in MB (for display)
        max_mb: u64,
    },

    /// I/O operation timed out.
    ///
    /// This prevents indefinite hangs on slow or unresponsive filesystems.
    #[error("I/O operation timed out for '{path}' after {timeout_secs} seconds")]
    IoTimeout {
        /// The file path that timed out
        path: PathBuf,
        /// The timeout duration in seconds
        timeout_secs: u64,
    },

    /// HEDL parsing error.
    ///
    /// This wraps errors from the hedl-core parser with additional context.
    #[error("Parse error: {0}")]
    Parse(String),

    /// HEDL canonicalization error.
    ///
    /// This wraps errors from the hedl-c14n canonicalizer.
    #[error("Canonicalization error: {0}")]
    Canonicalization(String),

    /// JSON conversion error.
    ///
    /// This includes both HEDL→JSON and JSON→HEDL conversion errors.
    #[error("JSON conversion error: {0}")]
    JsonConversion(String),

    /// JSON serialization/deserialization error.
    ///
    /// This wraps `serde_json` errors during formatting.
    #[error("JSON format error: {message}")]
    JsonFormat {
        /// The error message
        message: String,
    },

    /// YAML conversion error.
    ///
    /// This includes both HEDL→YAML and YAML→HEDL conversion errors.
    #[error("YAML conversion error: {0}")]
    YamlConversion(String),

    /// XML conversion error.
    ///
    /// This includes both HEDL→XML and XML→HEDL conversion errors.
    #[error("XML conversion error: {0}")]
    XmlConversion(String),

    /// CSV conversion error.
    ///
    /// This includes both HEDL→CSV and CSV→HEDL conversion errors.
    #[error("CSV conversion error: {0}")]
    CsvConversion(String),

    /// Parquet conversion error.
    ///
    /// This includes both HEDL→Parquet and Parquet→HEDL conversion errors.
    #[error("Parquet conversion error: {0}")]
    ParquetConversion(String),

    /// Linting error.
    ///
    /// This indicates that linting found issues that should cause failure.
    #[error("Lint errors found")]
    LintErrors,

    /// File is not in canonical form.
    ///
    /// This is returned by the `format --check` command.
    #[error("File is not in canonical form")]
    NotCanonical,

    /// Invalid input provided by the user.
    ///
    /// This covers validation failures like invalid type names, empty files, etc.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Thread pool creation error.
    ///
    /// This occurs when creating a local Rayon thread pool fails, typically due to
    /// invalid configuration (e.g., zero threads) or resource exhaustion.
    ///
    /// # Context
    ///
    /// * `message` - Detailed error message from Rayon
    /// * `requested_threads` - The number of threads requested
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::error::CliError;
    ///
    /// // Requesting zero threads is invalid
    /// let err = CliError::thread_pool_error("thread count must be positive", 0);
    /// ```
    #[error("Failed to create thread pool: {message}")]
    ThreadPoolError {
        /// The error message from Rayon
        message: String,
        /// The number of threads requested
        requested_threads: usize,
    },

    /// Invalid glob pattern.
    ///
    /// This error occurs when a glob pattern is malformed or contains invalid syntax.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::error::CliError;
    ///
    /// let err = CliError::GlobPattern {
    ///     pattern: "[invalid".to_string(),
    ///     message: "unclosed character class".to_string(),
    /// };
    /// ```
    #[error("Invalid glob pattern '{pattern}': {message}")]
    GlobPattern {
        /// The invalid pattern
        pattern: String,
        /// The error message
        message: String,
    },

    /// No files matched the provided patterns.
    ///
    /// This error occurs when glob patterns don't match any files.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::error::CliError;
    ///
    /// let err = CliError::NoFilesMatched {
    ///     patterns: vec!["*.hedl".to_string(), "test/*.hedl".to_string()],
    /// };
    /// ```
    #[error("File discovery failed: no files matched patterns: {}", patterns.join(", "))]
    NoFilesMatched {
        /// The patterns that didn't match any files
        patterns: Vec<String>,
    },

    /// Directory traversal error.
    ///
    /// This error occurs when directory traversal fails due to permissions,
    /// I/O errors, or other filesystem issues.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::error::CliError;
    /// use std::path::PathBuf;
    ///
    /// let err = CliError::DirectoryTraversal {
    ///     path: PathBuf::from("/restricted"),
    ///     message: "permission denied".to_string(),
    /// };
    /// ```
    #[error("Failed to traverse directory '{path}': {message}")]
    DirectoryTraversal {
        /// The directory path that caused the error
        path: PathBuf,
        /// The error message
        message: String,
    },

    /// Resource exhaustion error.
    ///
    /// This error occurs when system resources are exhausted (e.g., file handles, memory).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::error::CliError;
    ///
    /// let err = CliError::ResourceExhaustion {
    ///     resource_type: "file_handles".to_string(),
    ///     message: "too many open files".to_string(),
    ///     current_usage: 1024,
    ///     limit: 1024,
    /// };
    /// ```
    #[error("Resource exhaustion: {resource_type} - {message} (usage: {current_usage}/{limit})")]
    ResourceExhaustion {
        /// The type of resource exhausted
        resource_type: String,
        /// The error message
        message: String,
        /// Current resource usage
        current_usage: u64,
        /// Resource limit
        limit: u64,
    },
}

impl CliError {
    /// Create an I/O error with file path context.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path that caused the error
    /// * `source` - The underlying I/O error
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::error::CliError;
    /// use std::fs;
    ///
    /// let result = fs::read_to_string("file.hedl")
    ///     .map_err(|e| CliError::io_error("file.hedl", e));
    /// ```
    pub fn io_error(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            message: source.to_string(),
        }
    }

    /// Create a file-too-large error.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path that exceeded the limit
    /// * `actual` - The actual file size in bytes
    /// * `max` - The maximum allowed file size in bytes
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::error::CliError;
    ///
    /// const MAX_SIZE: u64 = 100 * 1024 * 1024; // 100 MB
    /// let err = CliError::file_too_large("huge.hedl", 200_000_000, MAX_SIZE);
    /// ```
    pub fn file_too_large(path: impl Into<PathBuf>, actual: u64, max: u64) -> Self {
        Self::FileTooLarge {
            path: path.into(),
            actual,
            max,
            max_mb: max / (1024 * 1024),
        }
    }

    /// Create an I/O timeout error.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path that timed out
    /// * `timeout_secs` - The timeout duration in seconds
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::error::CliError;
    ///
    /// let err = CliError::io_timeout("/slow/filesystem/file.hedl", 30);
    /// ```
    pub fn io_timeout(path: impl Into<PathBuf>, timeout_secs: u64) -> Self {
        Self::IoTimeout {
            path: path.into(),
            timeout_secs,
        }
    }

    /// Create a parse error.
    ///
    /// # Arguments
    ///
    /// * `msg` - The parse error message
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }

    /// Create a canonicalization error.
    ///
    /// # Arguments
    ///
    /// * `msg` - The canonicalization error message
    pub fn canonicalization(msg: impl Into<String>) -> Self {
        Self::Canonicalization(msg.into())
    }

    /// Create an invalid input error.
    ///
    /// # Arguments
    ///
    /// * `msg` - Description of the invalid input
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::error::CliError;
    ///
    /// let err = CliError::invalid_input("Type name must be alphanumeric");
    /// ```
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Create a JSON conversion error.
    ///
    /// # Arguments
    ///
    /// * `msg` - The JSON conversion error message
    pub fn json_conversion(msg: impl Into<String>) -> Self {
        Self::JsonConversion(msg.into())
    }

    /// Create a YAML conversion error.
    ///
    /// # Arguments
    ///
    /// * `msg` - The YAML conversion error message
    pub fn yaml_conversion(msg: impl Into<String>) -> Self {
        Self::YamlConversion(msg.into())
    }

    /// Create an XML conversion error.
    ///
    /// # Arguments
    ///
    /// * `msg` - The XML conversion error message
    pub fn xml_conversion(msg: impl Into<String>) -> Self {
        Self::XmlConversion(msg.into())
    }

    /// Create a CSV conversion error.
    ///
    /// # Arguments
    ///
    /// * `msg` - The CSV conversion error message
    pub fn csv_conversion(msg: impl Into<String>) -> Self {
        Self::CsvConversion(msg.into())
    }

    /// Create a Parquet conversion error.
    ///
    /// # Arguments
    ///
    /// * `msg` - The Parquet conversion error message
    pub fn parquet_conversion(msg: impl Into<String>) -> Self {
        Self::ParquetConversion(msg.into())
    }

    /// Create a thread pool error.
    ///
    /// # Arguments
    ///
    /// * `msg` - The error message from Rayon
    /// * `requested_threads` - The number of threads requested
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::error::CliError;
    ///
    /// let err = CliError::thread_pool_error("thread count must be positive", 0);
    /// ```
    pub fn thread_pool_error(msg: impl Into<String>, requested_threads: usize) -> Self {
        Self::ThreadPoolError {
            message: msg.into(),
            requested_threads,
        }
    }

    /// Check if two errors are similar for grouping purposes.
    ///
    /// Errors are considered similar if they have the same variant type,
    /// allowing aggregation of similar errors in batch processing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_cli::error::CliError;
    ///
    /// let err1 = CliError::parse("syntax error");
    /// let err2 = CliError::parse("unexpected token");
    /// assert!(err1.similar_to(&err2));
    ///
    /// let err3 = CliError::NotCanonical;
    /// assert!(!err1.similar_to(&err3));
    /// ```
    #[must_use]
    pub fn similar_to(&self, other: &CliError) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    /// Get the error category for reporting.
    ///
    /// Categorizes errors into broad types for summary reporting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_cli::error::{CliError, ErrorCategory};
    ///
    /// let err = CliError::parse("syntax error");
    /// assert!(matches!(err.category(), ErrorCategory::ParseError));
    /// ```
    #[must_use]
    pub fn category(&self) -> ErrorCategory {
        match self {
            CliError::Io { .. } | CliError::FileTooLarge { .. } | CliError::IoTimeout { .. } => {
                ErrorCategory::IoError
            }
            CliError::Parse(_) => ErrorCategory::ParseError,
            CliError::Canonicalization(_) | CliError::NotCanonical => ErrorCategory::FormatError,
            CliError::LintErrors => ErrorCategory::LintError,
            CliError::GlobPattern { .. }
            | CliError::NoFilesMatched { .. }
            | CliError::DirectoryTraversal { .. } => ErrorCategory::FileDiscoveryError,
            CliError::ResourceExhaustion { .. } | CliError::ThreadPoolError { .. } => {
                ErrorCategory::ResourceError
            }
            CliError::JsonConversion(_)
            | CliError::JsonFormat { .. }
            | CliError::YamlConversion(_)
            | CliError::XmlConversion(_)
            | CliError::CsvConversion(_)
            | CliError::ParquetConversion(_) => ErrorCategory::ConversionError,
            CliError::InvalidInput(_) => ErrorCategory::ValidationError,
        }
    }
}

/// Error category for classification and reporting.
///
/// Used to group errors by type in batch processing reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// I/O errors (file not found, permission denied, etc.)
    IoError,
    /// Parsing errors (syntax errors, malformed input)
    ParseError,
    /// Formatting/canonicalization errors
    FormatError,
    /// Lint errors and warnings
    LintError,
    /// File discovery errors (glob patterns, directory traversal)
    FileDiscoveryError,
    /// Resource exhaustion (memory, file handles, threads)
    ResourceError,
    /// Format conversion errors (JSON, YAML, XML, CSV, Parquet)
    ConversionError,
    /// Input validation errors
    ValidationError,
}

// Automatic conversion from serde_json::Error
impl From<serde_json::Error> for CliError {
    fn from(source: serde_json::Error) -> Self {
        Self::JsonFormat {
            message: source.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let err = CliError::io_error(
            "test.hedl",
            io::Error::new(io::ErrorKind::NotFound, "file not found"),
        );
        let msg = err.to_string();
        assert!(msg.contains("test.hedl"));
        assert!(msg.contains("file not found"));
    }

    #[test]
    fn test_file_too_large_display() {
        let err = CliError::file_too_large("big.hedl", 200_000_000, 100 * 1024 * 1024);
        let msg = err.to_string();
        assert!(msg.contains("big.hedl"));
        assert!(msg.contains("200000000 bytes"));
        assert!(msg.contains("100 MB"));
    }

    #[test]
    fn test_io_timeout_display() {
        let err = CliError::io_timeout("/slow/file.hedl", 30);
        let msg = err.to_string();
        assert!(msg.contains("/slow/file.hedl"));
        assert!(msg.contains("30 seconds"));
    }

    #[test]
    fn test_parse_error_display() {
        let err = CliError::parse("unexpected token");
        assert_eq!(err.to_string(), "Parse error: unexpected token");
    }

    #[test]
    fn test_invalid_input_display() {
        let err = CliError::invalid_input("CSV file is empty");
        assert_eq!(err.to_string(), "Invalid input: CSV file is empty");
    }

    #[test]
    fn test_json_format_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let cli_err: CliError = json_err.into();
        assert!(matches!(cli_err, CliError::JsonFormat { .. }));
    }

    #[test]
    fn test_error_cloning() {
        let err = CliError::io_error(
            "test.hedl",
            io::Error::new(io::ErrorKind::NotFound, "not found"),
        );
        let cloned = err.clone();
        assert_eq!(err.to_string(), cloned.to_string());
    }
}
