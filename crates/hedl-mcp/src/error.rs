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

//! Error types for the MCP server.

use thiserror::Error;

/// Error category for client/server/transient classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Client error (4xx-equivalent): Invalid input, bad request
    Client,
    /// Server error (5xx-equivalent): Internal failure, external dependency
    Server,
    /// Transient error: Temporary condition, retry may succeed
    Transient,
}

/// Error severity for logging and monitoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational message
    Info,
    /// Warning condition
    Warning,
    /// Error condition
    Error,
    /// Critical failure
    Critical,
}

/// MCP server error type.
#[derive(Error, Debug)]
pub enum McpError {
    /// HEDL parsing error.
    #[error("HEDL parse error: {0}")]
    Parse(#[from] hedl_core::HedlError),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid request.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Tool not found.
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Resource not found.
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    /// Invalid arguments.
    #[error("Invalid arguments: {0}")]
    InvalidArguments(String),

    /// Path traversal attempt.
    #[error("Path traversal not allowed: {0}")]
    PathTraversal(String),

    /// File not found.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Resource limit exceeded.
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
}

/// Result type for MCP operations.
pub type McpResult<T> = Result<T, McpError>;

impl McpError {
    /// Get the MCP error code.
    #[must_use]
    pub fn code(&self) -> i32 {
        match self {
            Self::Parse(_) => -32001,
            Self::Json(_) => -32700,
            Self::Io(_) => -32002,
            Self::InvalidRequest(_) => -32600,
            Self::ToolNotFound(_) => -32601,
            Self::ResourceNotFound(_) => -32603,
            Self::InvalidArguments(_) => -32602,
            Self::PathTraversal(_) => -32003,
            Self::FileNotFound(_) => -32004,
            Self::ResourceLimit(_) => -32005,
        }
    }

    /// Get the error category for client/server/transient classification.
    #[must_use]
    pub fn category(&self) -> ErrorCategory {
        match self {
            // Client errors: invalid input, bad request, not found
            Self::Parse(_) => ErrorCategory::Client,
            Self::Json(_) => ErrorCategory::Client,
            Self::InvalidRequest(_) => ErrorCategory::Client,
            Self::ToolNotFound(_) => ErrorCategory::Client,
            Self::ResourceNotFound(_) => ErrorCategory::Client,
            Self::InvalidArguments(_) => ErrorCategory::Client,
            Self::PathTraversal(_) => ErrorCategory::Client,
            Self::FileNotFound(_) => ErrorCategory::Client,

            // Server errors: internal failures
            Self::ResourceLimit(_) => ErrorCategory::Server,

            // Transient errors: I/O operations that might succeed on retry
            Self::Io(_) => ErrorCategory::Transient,
        }
    }

    /// Get the error severity for logging and monitoring.
    #[must_use]
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Info: expected client errors, not found
            Self::ToolNotFound(_) => ErrorSeverity::Info,
            Self::ResourceNotFound(_) => ErrorSeverity::Info,
            Self::FileNotFound(_) => ErrorSeverity::Info,

            // Warning: invalid input, potential issues
            Self::Parse(_) => ErrorSeverity::Warning,
            Self::Json(_) => ErrorSeverity::Warning,
            Self::InvalidRequest(_) => ErrorSeverity::Warning,
            Self::InvalidArguments(_) => ErrorSeverity::Warning,

            // Error: security violations, I/O failures
            Self::PathTraversal(_) => ErrorSeverity::Error,
            Self::Io(_) => ErrorSeverity::Error,

            // Critical: resource exhaustion, system limits
            Self::ResourceLimit(_) => ErrorSeverity::Critical,
        }
    }

    /// Check if the error is retryable.
    ///
    /// Returns `true` for transient errors where a retry might succeed,
    /// `false` for client errors and permanent failures.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            // Transient I/O errors are retryable
            Self::Io(_) => true,

            // Resource limits might clear up after a delay
            Self::ResourceLimit(_) => true,

            // Client errors are not retryable
            Self::Parse(_) => false,
            Self::Json(_) => false,
            Self::InvalidRequest(_) => false,
            Self::ToolNotFound(_) => false,
            Self::ResourceNotFound(_) => false,
            Self::InvalidArguments(_) => false,
            Self::PathTraversal(_) => false,
            Self::FileNotFound(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes_unique() {
        // Collect all error codes
        let errors = vec![
            McpError::Parse(hedl_core::HedlError::syntax("test", 1)),
            McpError::Json(serde_json::from_str::<i32>("invalid").unwrap_err()),
            McpError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")),
            McpError::InvalidRequest("test".to_string()),
            McpError::ToolNotFound("test".to_string()),
            McpError::ResourceNotFound("test".to_string()),
            McpError::InvalidArguments("test".to_string()),
            McpError::PathTraversal("test".to_string()),
            McpError::FileNotFound("test".to_string()),
            McpError::ResourceLimit("test".to_string()),
        ];

        let codes: Vec<i32> = errors.iter().map(super::McpError::code).collect();

        // Check that ResourceNotFound and InvalidArguments have different codes
        let resource_not_found = McpError::ResourceNotFound("test".to_string());
        let invalid_arguments = McpError::InvalidArguments("test".to_string());
        assert_ne!(
            resource_not_found.code(),
            invalid_arguments.code(),
            "ResourceNotFound and InvalidArguments must have different error codes"
        );

        // Verify specific codes
        assert_eq!(resource_not_found.code(), -32603);
        assert_eq!(invalid_arguments.code(), -32602);

        // Check for duplicates (allowing the ones we expect)
        let mut sorted_codes = codes.clone();
        sorted_codes.sort_unstable();
        for window in sorted_codes.windows(2) {
            assert!(
                window[0] != window[1],
                "Duplicate error code found: {}",
                window[0]
            );
        }
    }

    #[test]
    fn test_error_category_client() {
        let errors = vec![
            McpError::Parse(hedl_core::HedlError::syntax("test", 1)),
            McpError::Json(serde_json::from_str::<i32>("invalid").unwrap_err()),
            McpError::InvalidRequest("test".to_string()),
            McpError::ToolNotFound("test".to_string()),
            McpError::ResourceNotFound("test".to_string()),
            McpError::InvalidArguments("test".to_string()),
            McpError::PathTraversal("test".to_string()),
            McpError::FileNotFound("test".to_string()),
        ];

        for error in errors {
            assert_eq!(
                error.category(),
                ErrorCategory::Client,
                "Error {error:?} should be categorized as Client"
            );
        }
    }

    #[test]
    fn test_error_category_server() {
        let error = McpError::ResourceLimit("test".to_string());
        assert_eq!(error.category(), ErrorCategory::Server);
    }

    #[test]
    fn test_error_category_transient() {
        let error = McpError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
        assert_eq!(error.category(), ErrorCategory::Transient);
    }

    #[test]
    fn test_error_severity_info() {
        let errors = vec![
            McpError::ToolNotFound("test".to_string()),
            McpError::ResourceNotFound("test".to_string()),
            McpError::FileNotFound("test".to_string()),
        ];

        for error in errors {
            assert_eq!(
                error.severity(),
                ErrorSeverity::Info,
                "Error {error:?} should have Info severity"
            );
        }
    }

    #[test]
    fn test_error_severity_warning() {
        let errors = vec![
            McpError::Parse(hedl_core::HedlError::syntax("test", 1)),
            McpError::Json(serde_json::from_str::<i32>("invalid").unwrap_err()),
            McpError::InvalidRequest("test".to_string()),
            McpError::InvalidArguments("test".to_string()),
        ];

        for error in errors {
            assert_eq!(
                error.severity(),
                ErrorSeverity::Warning,
                "Error {error:?} should have Warning severity"
            );
        }
    }

    #[test]
    fn test_error_severity_error() {
        let errors = vec![
            McpError::PathTraversal("test".to_string()),
            McpError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")),
        ];

        for error in errors {
            assert_eq!(
                error.severity(),
                ErrorSeverity::Error,
                "Error {error:?} should have Error severity"
            );
        }
    }

    #[test]
    fn test_error_severity_critical() {
        let error = McpError::ResourceLimit("test".to_string());
        assert_eq!(error.severity(), ErrorSeverity::Critical);
    }

    #[test]
    fn test_is_retryable_true() {
        let errors = vec![
            McpError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")),
            McpError::ResourceLimit("test".to_string()),
        ];

        for error in errors {
            assert!(error.is_retryable(), "Error {error:?} should be retryable");
        }
    }

    #[test]
    fn test_is_retryable_false() {
        let errors = vec![
            McpError::Parse(hedl_core::HedlError::syntax("test", 1)),
            McpError::Json(serde_json::from_str::<i32>("invalid").unwrap_err()),
            McpError::InvalidRequest("test".to_string()),
            McpError::ToolNotFound("test".to_string()),
            McpError::ResourceNotFound("test".to_string()),
            McpError::InvalidArguments("test".to_string()),
            McpError::PathTraversal("test".to_string()),
            McpError::FileNotFound("test".to_string()),
        ];

        for error in errors {
            assert!(
                !error.is_retryable(),
                "Error {error:?} should not be retryable"
            );
        }
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Info < ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_category_equality() {
        assert_eq!(ErrorCategory::Client, ErrorCategory::Client);
        assert_ne!(ErrorCategory::Client, ErrorCategory::Server);
        assert_ne!(ErrorCategory::Client, ErrorCategory::Transient);
    }

    #[test]
    fn test_error_category_clone() {
        let category = ErrorCategory::Client;
        let cloned = category;
        assert_eq!(category, cloned);
    }

    #[test]
    fn test_error_severity_clone() {
        let severity = ErrorSeverity::Warning;
        let cloned = severity;
        assert_eq!(severity, cloned);
    }

    #[test]
    fn test_all_errors_have_consistent_categorization() {
        // This test ensures that the categorization is consistent with retry logic
        let errors = vec![
            McpError::Parse(hedl_core::HedlError::syntax("test", 1)),
            McpError::Json(serde_json::from_str::<i32>("invalid").unwrap_err()),
            McpError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test")),
            McpError::InvalidRequest("test".to_string()),
            McpError::ToolNotFound("test".to_string()),
            McpError::ResourceNotFound("test".to_string()),
            McpError::InvalidArguments("test".to_string()),
            McpError::PathTraversal("test".to_string()),
            McpError::FileNotFound("test".to_string()),
            McpError::ResourceLimit("test".to_string()),
        ];

        for error in errors {
            // Transient errors should generally be retryable
            if error.category() == ErrorCategory::Transient {
                assert!(
                    error.is_retryable(),
                    "Transient error {error:?} should be retryable"
                );
            }

            // Client errors should generally not be retryable (with some exceptions)
            if error.category() == ErrorCategory::Client {
                assert!(
                    !error.is_retryable(),
                    "Client error {error:?} should not be retryable"
                );
            }
        }
    }

    #[test]
    fn test_specific_error_codes() {
        assert_eq!(
            McpError::Parse(hedl_core::HedlError::syntax("", 1)).code(),
            -32001
        );
        assert_eq!(
            McpError::Json(serde_json::from_str::<i32>("x").unwrap_err()).code(),
            -32700
        );
        assert_eq!(
            McpError::Io(std::io::Error::new(std::io::ErrorKind::Other, "")).code(),
            -32002
        );
        assert_eq!(McpError::InvalidRequest(String::new()).code(), -32600);
        assert_eq!(McpError::ToolNotFound(String::new()).code(), -32601);
        assert_eq!(McpError::InvalidArguments(String::new()).code(), -32602);
        assert_eq!(McpError::ResourceNotFound(String::new()).code(), -32603);
        assert_eq!(McpError::PathTraversal(String::new()).code(), -32003);
        assert_eq!(McpError::FileNotFound(String::new()).code(), -32004);
        assert_eq!(McpError::ResourceLimit(String::new()).code(), -32005);
    }

    #[test]
    fn test_error_display() {
        let error = McpError::InvalidRequest("bad param".to_string());
        assert_eq!(format!("{error}"), "Invalid request: bad param");

        let error = McpError::ToolNotFound("unknown_tool".to_string());
        assert_eq!(format!("{error}"), "Tool not found: unknown_tool");

        let error = McpError::ResourceNotFound("missing.hedl".to_string());
        assert_eq!(format!("{error}"), "Resource not found: missing.hedl");
    }
}
