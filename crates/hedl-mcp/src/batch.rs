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

//! Batch operation types and execution for MCP server.
//!
//! Provides batch processing capabilities for executing multiple tool operations
//! in a single request with support for:
//! - Dependency resolution via topological sort
//! - Parallel execution for independent operations
//! - Transaction semantics with rollback support
//! - Flexible error handling (continue-on-error vs stop-on-error)

use crate::error::{McpError, McpResult};
use crate::protocol::CallToolResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Batch operation request containing multiple tool calls.
///
/// # Examples
///
/// ```
/// use hedl_mcp::batch::{BatchRequest, BatchOperation, BatchMode};
/// use serde_json::json;
///
/// let batch = BatchRequest {
///     operations: vec![
///         BatchOperation {
///             id: "val1".to_string(),
///             tool: "hedl_validate".to_string(),
///             arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
///             depends_on: vec![],
///         },
///         BatchOperation {
///             id: "val2".to_string(),
///             tool: "hedl_validate".to_string(),
///             arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
///             depends_on: vec![],
///         },
///     ],
///     mode: BatchMode::ContinueOnError,
///     parallel: true,
///     transaction: false,
///     timeout: None,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// List of operations to execute.
    ///
    /// Operations with dependencies will be executed in topological order.
    /// Independent operations may be executed in parallel if `parallel` is true.
    pub operations: Vec<BatchOperation>,

    /// Execution mode: continue on error or stop on first error.
    ///
    /// - `ContinueOnError`: Execute all operations regardless of failures
    /// - `StopOnError`: Stop execution at first failure
    #[serde(default = "default_batch_mode")]
    pub mode: BatchMode,

    /// Enable parallel execution for independent operations.
    ///
    /// When true, operations without dependencies may be executed concurrently
    /// using Rayon thread pool. Operations with dependencies are always executed
    /// serially in topological order.
    #[serde(default = "default_parallel")]
    pub parallel: bool,

    /// Transaction semantics (all-or-nothing).
    ///
    /// When true, all operations must succeed or all changes are rolled back.
    /// Currently only supports read-only operations in transaction mode.
    #[serde(default)]
    pub transaction: bool,

    /// Maximum time for batch execution in seconds.
    ///
    /// If specified and exceeded, the batch execution is aborted. None means
    /// no timeout.
    #[serde(default)]
    pub timeout: Option<u64>,
}

/// Individual operation within a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    /// Unique identifier for this operation.
    ///
    /// Used for result correlation and dependency resolution. Must be unique
    /// within the batch.
    pub id: String,

    /// Tool name to execute (e.g., "`hedl_validate`", "`hedl_format`").
    pub tool: String,

    /// Tool arguments as JSON value.
    ///
    /// Should match the input schema for the specified tool.
    #[serde(default)]
    pub arguments: Option<Value>,

    /// Dependencies on other operation IDs.
    ///
    /// This operation will only execute after all dependencies have completed
    /// successfully. Circular dependencies are detected and rejected.
    #[serde(default)]
    pub depends_on: Vec<String>,
}

/// Batch execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchMode {
    /// Continue processing even if operations fail.
    ///
    /// All operations are executed and results include success/failure status
    /// for each operation.
    ContinueOnError,

    /// Stop on first error (return partial results).
    ///
    /// Execution stops at the first failure. Completed operations are included
    /// in results, remaining operations are marked as skipped.
    StopOnError,
}

fn default_batch_mode() -> BatchMode {
    BatchMode::ContinueOnError
}

fn default_parallel() -> bool {
    true
}

/// Batch operation response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// Results for each operation (by ID).
    ///
    /// Results are returned in execution order (dependency order if dependencies
    /// exist, submission order otherwise).
    pub results: Vec<BatchOperationResult>,

    /// Summary statistics for the batch execution.
    pub summary: BatchSummary,

    /// Whether batch completed successfully.
    ///
    /// True if all operations succeeded (or if mode is `ContinueOnError` and at
    /// least one operation succeeded). False if batch-level error occurred.
    pub success: bool,

    /// Error message if batch-level failure occurred.
    ///
    /// Batch-level errors include dependency resolution failures, circular
    /// dependencies, duplicate operation IDs, etc. Does not include individual
    /// operation failures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result of a single operation in the batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResult {
    /// Operation ID from request.
    pub id: String,

    /// Whether this operation succeeded.
    pub success: bool,

    /// Operation result (if successful).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<CallToolResult>,

    /// Error details (if failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BatchError>,

    /// Execution time in milliseconds.
    pub duration_ms: u64,
}

/// Error details for failed operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Error code (JSON-RPC style).
    pub code: i32,

    /// Human-readable error message.
    pub message: String,

    /// Additional error context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl From<&McpError> for BatchError {
    fn from(error: &McpError) -> Self {
        Self {
            code: error.code(),
            message: error.to_string(),
            data: None,
        }
    }
}

/// Summary statistics for batch execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSummary {
    /// Total number of operations.
    pub total: usize,

    /// Number of successful operations.
    pub succeeded: usize,

    /// Number of failed operations.
    pub failed: usize,

    /// Number of skipped operations (due to dependencies or stop-on-error).
    pub skipped: usize,

    /// Total execution time in milliseconds.
    pub duration_ms: u64,

    /// Whether parallel execution was used.
    pub parallel: bool,
}

/// Categories of batch errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchErrorCategory {
    /// Error in batch request structure.
    BatchLevel,

    /// Error in individual operation.
    OperationLevel,

    /// Dependency resolution error.
    DependencyError,

    /// Transaction rollback error.
    RollbackError,
}

impl BatchError {
    /// Categorize error for proper handling.
    #[must_use]
    pub fn category(&self) -> BatchErrorCategory {
        match self.code {
            -32600 => BatchErrorCategory::BatchLevel, // Invalid request
            -32601 => BatchErrorCategory::OperationLevel, // Tool not found
            -32602 => BatchErrorCategory::OperationLevel, // Invalid arguments
            -32700 => BatchErrorCategory::BatchLevel, // Parse error
            -33000 => BatchErrorCategory::DependencyError, // Circular dependency
            -33001 => BatchErrorCategory::RollbackError, // Rollback failed
            _ => BatchErrorCategory::OperationLevel,
        }
    }

    /// Create a circular dependency error.
    #[must_use]
    pub fn circular_dependency(op_id: &str) -> Self {
        Self {
            code: -33000,
            message: format!("Circular dependency detected involving operation: {op_id}"),
            data: None,
        }
    }

    /// Create a missing dependency error.
    #[must_use]
    pub fn missing_dependency(op_id: &str, dep_id: &str) -> Self {
        Self {
            code: -33000,
            message: format!("Operation '{op_id}' depends on '{dep_id}' which does not exist"),
            data: None,
        }
    }

    /// Create a duplicate operation ID error.
    #[must_use]
    pub fn duplicate_id(op_id: &str) -> Self {
        Self {
            code: -32600,
            message: format!("Duplicate operation ID: {op_id}"),
            data: None,
        }
    }
}

impl BatchRequest {
    /// Validate batch request structure.
    ///
    /// Checks for:
    /// - Empty operations list
    /// - Duplicate operation IDs
    /// - Valid timeout values
    pub fn validate(&self) -> McpResult<()> {
        if self.operations.is_empty() {
            return Err(McpError::InvalidRequest(
                "Batch operations list cannot be empty".to_string(),
            ));
        }

        // Check for duplicate operation IDs
        let mut seen_ids = std::collections::HashSet::new();
        for op in &self.operations {
            if !seen_ids.insert(&op.id) {
                return Err(McpError::InvalidRequest(format!(
                    "Duplicate operation ID: {}",
                    op.id
                )));
            }
        }

        // Validate timeout is reasonable
        if let Some(timeout) = self.timeout {
            if timeout == 0 {
                return Err(McpError::InvalidRequest(
                    "Timeout must be greater than 0".to_string(),
                ));
            }
            if timeout > 3600 {
                // Max 1 hour
                return Err(McpError::InvalidRequest(
                    "Timeout cannot exceed 3600 seconds".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_batch_request_serialization() {
        let batch = BatchRequest {
            operations: vec![BatchOperation {
                id: "op1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "test"})),
                depends_on: vec![],
            }],
            mode: BatchMode::ContinueOnError,
            parallel: true,
            transaction: false,
            timeout: Some(30),
        };

        let json = serde_json::to_string(&batch).unwrap();
        let deserialized: BatchRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.operations.len(), 1);
        assert_eq!(deserialized.mode, BatchMode::ContinueOnError);
        assert!(deserialized.parallel);
        assert!(!deserialized.transaction);
        assert_eq!(deserialized.timeout, Some(30));
    }

    #[test]
    fn test_batch_mode_serialization() {
        let continue_mode = BatchMode::ContinueOnError;
        let json = serde_json::to_string(&continue_mode).unwrap();
        assert_eq!(json, "\"continue_on_error\"");

        let stop_mode = BatchMode::StopOnError;
        let json = serde_json::to_string(&stop_mode).unwrap();
        assert_eq!(json, "\"stop_on_error\"");
    }

    #[test]
    fn test_batch_error_categorization() {
        let batch_error = BatchError {
            code: -32600,
            message: "Invalid request".to_string(),
            data: None,
        };
        assert_eq!(batch_error.category(), BatchErrorCategory::BatchLevel);

        let op_error = BatchError {
            code: -32601,
            message: "Tool not found".to_string(),
            data: None,
        };
        assert_eq!(op_error.category(), BatchErrorCategory::OperationLevel);

        let dep_error = BatchError::circular_dependency("op1");
        assert_eq!(dep_error.category(), BatchErrorCategory::DependencyError);
    }

    #[test]
    fn test_batch_request_validate_empty() {
        let batch = BatchRequest {
            operations: vec![],
            mode: BatchMode::ContinueOnError,
            parallel: true,
            transaction: false,
            timeout: None,
        };

        assert!(batch.validate().is_err());
    }

    #[test]
    fn test_batch_request_validate_duplicate_ids() {
        let batch = BatchRequest {
            operations: vec![
                BatchOperation {
                    id: "op1".to_string(),
                    tool: "hedl_validate".to_string(),
                    arguments: None,
                    depends_on: vec![],
                },
                BatchOperation {
                    id: "op1".to_string(),
                    tool: "hedl_format".to_string(),
                    arguments: None,
                    depends_on: vec![],
                },
            ],
            mode: BatchMode::ContinueOnError,
            parallel: true,
            transaction: false,
            timeout: None,
        };

        assert!(batch.validate().is_err());
    }

    #[test]
    fn test_batch_request_validate_invalid_timeout() {
        let batch = BatchRequest {
            operations: vec![BatchOperation {
                id: "op1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: None,
                depends_on: vec![],
            }],
            mode: BatchMode::ContinueOnError,
            parallel: true,
            transaction: false,
            timeout: Some(0),
        };

        assert!(batch.validate().is_err());

        let batch = BatchRequest {
            operations: vec![BatchOperation {
                id: "op1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: None,
                depends_on: vec![],
            }],
            mode: BatchMode::ContinueOnError,
            parallel: true,
            transaction: false,
            timeout: Some(5000),
        };

        assert!(batch.validate().is_err());
    }

    #[test]
    fn test_batch_request_validate_valid() {
        let batch = BatchRequest {
            operations: vec![
                BatchOperation {
                    id: "op1".to_string(),
                    tool: "hedl_validate".to_string(),
                    arguments: None,
                    depends_on: vec![],
                },
                BatchOperation {
                    id: "op2".to_string(),
                    tool: "hedl_format".to_string(),
                    arguments: None,
                    depends_on: vec!["op1".to_string()],
                },
            ],
            mode: BatchMode::ContinueOnError,
            parallel: true,
            transaction: false,
            timeout: Some(60),
        };

        assert!(batch.validate().is_ok());
    }
}
