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

//! Integration tests for batch operations.

use hedl_mcp::batch::{BatchMode, BatchOperation, BatchRequest};
use hedl_mcp::BatchExecutor;
use serde_json::json;
use std::path::Path;

#[test]
fn test_batch_execute_single_operation() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![BatchOperation {
            id: "val1".to_string(),
            tool: "hedl_validate".to_string(),
            arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
            depends_on: vec![],
        }],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    assert_eq!(response.summary.total, 1);
    assert_eq!(response.summary.succeeded, 1);
    assert_eq!(response.summary.failed, 0);
    assert!(response.success);
}

#[test]
fn test_batch_execute_multiple_operations() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "val1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "val2".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    assert_eq!(response.summary.total, 2);
    assert_eq!(response.summary.succeeded, 2);
    assert_eq!(response.summary.failed, 0);
}

#[test]
fn test_batch_continue_on_error() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "val1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "invalid".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "invalid"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "val3".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    assert_eq!(response.summary.total, 3);
    assert_eq!(response.summary.succeeded, 2);
    assert_eq!(response.summary.failed, 1);
    // All operations executed despite failure
    assert_eq!(response.results.len(), 3);
}

#[test]
fn test_batch_stop_on_error() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "val1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "invalid".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "invalid"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "val3".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::StopOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    assert_eq!(response.summary.total, 3);
    assert_eq!(response.summary.succeeded, 1);
    assert_eq!(response.summary.failed, 1);
    assert_eq!(response.summary.skipped, 1);
    // First two operations completed, third skipped
    assert_eq!(response.results.len(), 3);
    assert!(response.results[0].success);
    assert!(!response.results[1].success);
    assert!(!response.results[2].success);
}

#[test]
fn test_batch_dependency_resolution() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "op2".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec!["op1".to_string()],
            },
            BatchOperation {
                id: "op1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    assert_eq!(response.summary.succeeded, 2);
    // Results should be in dependency order: op1 then op2
    assert_eq!(response.results[0].id, "op1");
    assert_eq!(response.results[1].id, "op2");
}

#[test]
fn test_batch_circular_dependency_detection() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: None,
                depends_on: vec!["op2".to_string()],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: None,
                depends_on: vec!["op1".to_string()],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let result = executor.execute(batch);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Circular dependency"));
}

#[test]
fn test_batch_missing_dependency() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![BatchOperation {
            id: "op1".to_string(),
            tool: "hedl_validate".to_string(),
            arguments: None,
            depends_on: vec!["missing".to_string()],
        }],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let result = executor.execute(batch);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("does not exist"));
}

#[test]
fn test_batch_empty_operations() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let result = executor.execute(batch);
    assert!(result.is_err());
}

#[test]
fn test_batch_duplicate_operation_ids() {
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
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let result = batch.validate();
    assert!(result.is_err());
}

#[test]
fn test_batch_invalid_tool_name() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![BatchOperation {
            id: "op1".to_string(),
            tool: "nonexistent_tool".to_string(),
            arguments: None,
            depends_on: vec![],
        }],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    assert_eq!(response.summary.failed, 1);
    assert!(!response.results[0].success);
}

#[test]
fn test_batch_parallel_execution() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "val1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "val2".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "val3".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "val4".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: true,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    assert_eq!(response.summary.total, 4);
    assert_eq!(response.summary.succeeded, 4);
    assert!(response.summary.parallel);
}

#[test]
fn test_batch_transaction_rollback() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "val1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "invalid".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "invalid"})),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: true,
        timeout: None,
    };

    let result = executor.execute(batch);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Transaction failed"));
}

#[test]
fn test_batch_all_operations_fail() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "invalid1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "bad"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "invalid2".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "bad"})),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    assert_eq!(response.summary.failed, 2);
    assert_eq!(response.summary.succeeded, 0);
    assert!(!response.success);
}

#[test]
fn test_batch_mixed_tools() {
    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "val1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "format1".to_string(),
                tool: "hedl_format".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "stats1".to_string(),
                tool: "hedl_stats".to_string(),
                arguments: Some(json!({"hedl": "%VERSION: 1.0\n---"})),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: true,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    assert_eq!(response.summary.succeeded, 3);
}
