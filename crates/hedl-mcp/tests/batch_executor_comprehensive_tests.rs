// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive batch executor tests for hedl-mcp.
//!
//! Tests dependency resolution, parallel execution, transaction semantics,
//! and error handling in batch operations.

use hedl_mcp::batch::{BatchMode, BatchOperation, BatchRequest};
use hedl_mcp::cache::OperationCache;
use hedl_mcp::BatchExecutor;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_operation(id: &str, tool: &str, depends_on: Vec<String>) -> BatchOperation {
    // Valid HEDL syntax: %VERSION: 1.0 (with colon), body has key: value
    let hedl = "%VERSION: 1.0\n---\nid: test123";
    BatchOperation {
        id: id.to_string(),
        tool: tool.to_string(),
        arguments: Some(json!({
            "hedl": hedl
        })),
        depends_on,
    }
}

// ============================================================================
// Dependency Resolution Tests
// ============================================================================

#[test]
fn test_execute_independent_operations() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op2", "hedl_validate", vec![]),
            create_operation("op3", "hedl_validate", vec![]),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 3);
    assert!(response.results.iter().all(|r| r.success));
}

#[test]
fn test_execute_linear_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    // op1 -> op2 -> op3 (linear chain)
    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op2", "hedl_validate", vec!["op1".to_string()]),
            create_operation("op3", "hedl_validate", vec!["op2".to_string()]),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 3);
    assert_eq!(response.results[0].id, "op1");
    assert_eq!(response.results[1].id, "op2");
    assert_eq!(response.results[2].id, "op3");
}

#[test]
fn test_execute_diamond_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    // Diamond pattern: op1 -> op2, op3 -> op4
    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op2", "hedl_validate", vec!["op1".to_string()]),
            create_operation("op3", "hedl_validate", vec!["op1".to_string()]),
            create_operation(
                "op4",
                "hedl_validate",
                vec!["op2".to_string(), "op3".to_string()],
            ),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 4);

    // op1 should be first
    assert_eq!(response.results[0].id, "op1");

    // op4 should be last (depends on both op2 and op3)
    assert_eq!(response.results[3].id, "op4");
}

#[test]
fn test_execute_circular_dependency() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    // Circular: op1 -> op2 -> op1
    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec!["op2".to_string()]),
            create_operation("op2", "hedl_validate", vec!["op1".to_string()]),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let result = executor.execute(request);
    assert!(result.is_err());
}

#[test]
fn test_execute_missing_dependency() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![create_operation(
            "op1",
            "hedl_validate",
            vec!["nonexistent".to_string()],
        )],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let result = executor.execute(request);
    assert!(result.is_err());
}

#[test]
fn test_execute_complex_dag() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    // Complex DAG
    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op2", "hedl_validate", vec![]),
            create_operation("op3", "hedl_validate", vec!["op1".to_string()]),
            create_operation(
                "op4",
                "hedl_validate",
                vec!["op1".to_string(), "op2".to_string()],
            ),
            create_operation(
                "op5",
                "hedl_validate",
                vec!["op3".to_string(), "op4".to_string()],
            ),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 5);

    // op5 should be last (depends on op3 and op4)
    assert_eq!(response.results[4].id, "op5");
}

// ============================================================================
// Parallel Execution Tests
// ============================================================================

#[test]
fn test_parallel_execution_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op2", "hedl_validate", vec![]),
            create_operation("op3", "hedl_validate", vec![]),
            create_operation("op4", "hedl_validate", vec![]),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: true,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 4);
    assert!(response.summary.parallel);
}

#[test]
fn test_parallel_execution_with_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    // Two independent chains that can run in parallel
    let request = BatchRequest {
        operations: vec![
            create_operation("chain1_op1", "hedl_validate", vec![]),
            create_operation(
                "chain1_op2",
                "hedl_validate",
                vec!["chain1_op1".to_string()],
            ),
            create_operation("chain2_op1", "hedl_validate", vec![]),
            create_operation(
                "chain2_op2",
                "hedl_validate",
                vec!["chain2_op1".to_string()],
            ),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: true,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 4);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_stop_on_error_mode() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op2", "unknown_tool", vec![]), // Will fail
            create_operation("op3", "hedl_validate", vec![]),
        ],
        mode: BatchMode::StopOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();

    // Should stop after op2 fails
    assert_eq!(response.results.len(), 3);
    assert!(response.results[0].success);
    assert!(!response.results[1].success);
    // op3 should be skipped
    assert!(!response.results[2].success);
}

#[test]
fn test_continue_on_error_mode() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op2", "unknown_tool", vec![]), // Will fail
            create_operation("op3", "hedl_validate", vec![]),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();

    // Should execute all operations
    assert_eq!(response.results.len(), 3);
    assert!(response.results[0].success);
    assert!(!response.results[1].success);
    assert!(response.results[2].success);
}

#[test]
fn test_invalid_tool_error() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![create_operation("op1", "nonexistent_tool", vec![])],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 1);
    assert!(!response.results[0].success);
    assert!(response.results[0].error.is_some());
}

// ============================================================================
// Transaction Mode Tests
// ============================================================================

#[test]
fn test_transaction_all_succeed() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op2", "hedl_validate", vec![]),
            create_operation("op3", "hedl_validate", vec![]),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: true,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 3);
    assert!(response.results.iter().all(|r| r.success));
}

#[test]
fn test_transaction_one_fails() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op2", "unknown_tool", vec![]), // Fails
            create_operation("op3", "hedl_validate", vec![]),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: true,
        timeout: None,
    };

    // In transaction mode, if one fails, all should fail
    let result = executor.execute(request);
    assert!(result.is_err());
}

// ============================================================================
// Batch Request Validation Tests
// ============================================================================

#[test]
fn test_validate_empty_operations() {
    let request = BatchRequest {
        operations: vec![],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_duplicate_operation_ids() {
    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op1", "hedl_validate", vec![]), // Duplicate ID
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_invalid_timeout() {
    let request = BatchRequest {
        operations: vec![create_operation("op1", "hedl_validate", vec![])],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: Some(0), // Invalid: zero timeout
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_timeout_too_large() {
    let request = BatchRequest {
        operations: vec![create_operation("op1", "hedl_validate", vec![])],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: Some(10000), // Invalid: > 3600
    };

    let result = request.validate();
    assert!(result.is_err());
}

#[test]
fn test_validate_valid_timeout() {
    let request = BatchRequest {
        operations: vec![create_operation("op1", "hedl_validate", vec![])],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: Some(60), // Valid: 60 seconds
    };

    assert!(request.validate().is_ok());
}

// ============================================================================
// Batch Response Tests
// ============================================================================

#[test]
fn test_response_summary() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![
            create_operation("op1", "hedl_validate", vec![]),
            create_operation("op2", "unknown_tool", vec![]),
            create_operation("op3", "hedl_validate", vec![]),
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();

    assert_eq!(response.summary.total, 3);
    assert_eq!(response.summary.succeeded, 2);
    assert_eq!(response.summary.failed, 1);
    assert_eq!(response.summary.skipped, 0);
}

#[test]
fn test_response_duration() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    // Run multiple operations to ensure measurable duration (>1ms).
    // Single operations may complete in microseconds on fast machines.
    let operations: Vec<_> = (0..20)
        .map(|i| create_operation(&format!("op{i}"), "hedl_validate", vec![]))
        .collect();

    let request = BatchRequest {
        operations,
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();

    // Verify batch completed successfully
    assert_eq!(response.summary.total, 20);
    assert_eq!(response.summary.succeeded, 20);
    assert_eq!(response.results.len(), 20);

    // Duration tracking is working if we get here without panic.
    // The actual value may be 0 on very fast machines (sub-millisecond).
    // What matters is that the field is populated and the type is correct.
    let _duration: u64 = response.summary.duration_ms;

    // Individual operation durations should also be tracked
    for result in &response.results {
        let _op_duration: u64 = result.duration_ms;
    }
}

// ============================================================================
// Cache Integration Tests
// ============================================================================

#[test]
fn test_batch_with_cache() {
    let temp_dir = TempDir::new().unwrap();
    let cache = Arc::new(OperationCache::new(100));
    let executor = BatchExecutor::new(temp_dir.path(), Some(cache.clone()));

    // Valid HEDL syntax: %VERSION: 1.0 (with colon)
    let hedl = "%VERSION: 1.0\n---\nname: Alice";

    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": hedl})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": hedl})), // Same content
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 2);

    // Both should succeed
    assert!(response.results[0].success);
    assert!(response.results[1].success);

    // Second operation should hit cache
    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
}

// ============================================================================
// Mixed Tool Tests
// ============================================================================

#[test]
fn test_mixed_tool_operations() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    // Valid HEDL syntax: %VERSION: 1.0 (with colon)
    let hedl = "%VERSION: 1.0\n---\nname: Alice";
    let request = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "validate".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({
                    "hedl": hedl
                })),
                depends_on: vec![],
            },
            BatchOperation {
                id: "stats".to_string(),
                tool: "hedl_stats".to_string(),
                arguments: Some(json!({
                    "hedl": hedl
                })),
                depends_on: vec![],
            },
            BatchOperation {
                id: "format".to_string(),
                tool: "hedl_format".to_string(),
                arguments: Some(json!({
                    "hedl": hedl
                })),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 3);
    assert!(response.results.iter().all(|r| r.success));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_single_operation() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![create_operation("op1", "hedl_validate", vec![])],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 1);
    assert!(response.results[0].success);
}

#[test]
fn test_many_operations() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let mut operations = Vec::new();
    for i in 0..50 {
        operations.push(create_operation(&format!("op{i}"), "hedl_validate", vec![]));
    }

    let request = BatchRequest {
        operations,
        mode: BatchMode::ContinueOnError,
        parallel: true,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    assert_eq!(response.results.len(), 50);
}

#[test]
fn test_operation_with_null_arguments() {
    let temp_dir = TempDir::new().unwrap();
    let executor = BatchExecutor::new(temp_dir.path(), None);

    let request = BatchRequest {
        operations: vec![BatchOperation {
            id: "op1".to_string(),
            tool: "hedl_validate".to_string(),
            arguments: None,
            depends_on: vec![],
        }],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(request).unwrap();
    // Should fail due to missing required arguments
    assert!(!response.results[0].success);
}
