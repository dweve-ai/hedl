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

//! Batch operation executor with dependency resolution and parallel execution.

use crate::batch::{
    BatchError, BatchMode, BatchOperation, BatchOperationResult, BatchRequest, BatchResponse,
    BatchSummary,
};
use crate::cache::OperationCache;
use crate::error::{McpError, McpResult};
use crate::tools::execute_tool;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

/// Batch executor with parallel processing support.
pub struct BatchExecutor<'a> {
    /// Root path for file operations.
    root_path: &'a Path,

    /// Cache reference for operation results.
    cache: Option<Arc<OperationCache>>,

    /// Minimum number of operations to enable parallel execution.
    parallel_threshold: usize,
}

impl<'a> BatchExecutor<'a> {
    /// Create a new batch executor.
    ///
    /// # Arguments
    ///
    /// * `root_path` - Root directory for file operations
    /// * `cache` - Optional operation cache for result caching
    #[must_use]
    pub fn new(root_path: &'a Path, cache: Option<Arc<OperationCache>>) -> Self {
        Self {
            root_path,
            cache,
            parallel_threshold: 4, // Parallel execution for 4+ operations
        }
    }

    /// Execute batch request with optimal strategy.
    ///
    /// Automatically chooses between serial and parallel execution based on
    /// dependencies and batch size. Handles all error modes and provides
    /// detailed result tracking.
    pub fn execute(&self, request: BatchRequest) -> McpResult<BatchResponse> {
        let start = Instant::now();

        // Validate request structure
        request.validate()?;

        // Resolve operation dependencies (topological sort)
        let execution_order = self.resolve_dependencies(&request.operations)?;

        // Choose execution strategy based on dependencies and settings
        let results = if request.transaction {
            // Transaction mode: execute all, rollback on any failure
            self.execute_transactional(&execution_order, &request)?
        } else if request.parallel
            && execution_order.len() >= self.parallel_threshold
            && self.has_independent_operations(&execution_order)
        {
            // Parallel mode: execute independent operations concurrently
            self.execute_parallel(&execution_order, &request)?
        } else {
            // Serial mode: execute one by one
            self.execute_serial(&execution_order, &request)?
        };

        // Build response with summary
        let duration = start.elapsed();
        Ok(self.build_response(results, duration, request.parallel))
    }

    /// Check if batch has operations that can run in parallel.
    fn has_independent_operations(&self, operations: &[BatchOperation]) -> bool {
        operations.iter().any(|op| op.depends_on.is_empty())
    }

    /// Execute operations in parallel (independent operations only).
    ///
    /// Operations with dependencies are executed serially in topological order.
    /// Independent operations are executed concurrently using Rayon.
    fn execute_parallel(
        &self,
        operations: &[BatchOperation],
        request: &BatchRequest,
    ) -> McpResult<Vec<BatchOperationResult>> {
        // Separate operations by dependency levels
        let levels = self.compute_dependency_levels(operations);
        let mut all_results = Vec::with_capacity(operations.len());

        // Execute each level
        for (level_idx, level_ops) in levels.iter().enumerate() {
            tracing::debug!(
                "Executing dependency level {} with {} operations",
                level_idx,
                level_ops.len()
            );

            // Execute operations at this level in parallel
            let level_results: Vec<_> = level_ops
                .par_iter()
                .map(|op| self.execute_single_operation(op))
                .collect();

            // Check for errors in stop-on-error mode
            if request.mode == BatchMode::StopOnError {
                // Find the index of the first error (if any)
                let first_error_idx = level_results.iter().position(|r| !r.success);
                if let Some(err_idx) = first_error_idx {
                    // Collect successful results before the error, then the error itself
                    all_results.extend(level_results.into_iter().take(err_idx + 1));
                    // Mark remaining operations as skipped
                    for remaining_level in levels.iter().skip(level_idx + 1) {
                        for op in remaining_level {
                            all_results.push(self.create_skipped_result(op));
                        }
                    }
                    return Ok(all_results);
                }
            }

            all_results.extend(level_results);
        }

        Ok(all_results)
    }

    /// Compute dependency levels for parallel execution.
    ///
    /// Operations at the same level have no dependencies on each other and can
    /// be executed in parallel.
    fn compute_dependency_levels(&self, operations: &[BatchOperation]) -> Vec<Vec<BatchOperation>> {
        let mut levels: Vec<Vec<BatchOperation>> = Vec::new();
        let mut completed: HashSet<String> = HashSet::new();
        let mut remaining: Vec<_> = operations.to_vec();

        while !remaining.is_empty() {
            let mut current_level = Vec::new();
            let mut next_remaining = Vec::new();

            for op in remaining {
                // Can execute if all dependencies are completed (from PREVIOUS levels only)
                if op.depends_on.iter().all(|dep| completed.contains(dep)) {
                    current_level.push(op);
                } else {
                    next_remaining.push(op);
                }
            }

            if current_level.is_empty() && !next_remaining.is_empty() {
                // No progress made - should not happen if dependencies are valid
                tracing::warn!(
                    "Dependency resolution stalled with {} operations remaining",
                    next_remaining.len()
                );
                break;
            }

            // Mark operations in current level as completed AFTER building the level
            for op in &current_level {
                completed.insert(op.id.clone());
            }

            if !current_level.is_empty() {
                levels.push(current_level);
            }
            remaining = next_remaining;
        }

        levels
    }

    /// Execute operations serially (respects dependencies).
    fn execute_serial(
        &self,
        operations: &[BatchOperation],
        request: &BatchRequest,
    ) -> McpResult<Vec<BatchOperationResult>> {
        let mut results = Vec::with_capacity(operations.len());

        for op in operations {
            let result = self.execute_single_operation(op);

            if request.mode == BatchMode::StopOnError && !result.success {
                results.push(result);
                // Mark remaining operations as skipped
                for remaining_op in operations.iter().skip(results.len()) {
                    results.push(self.create_skipped_result(remaining_op));
                }
                break;
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Execute operations in transaction mode.
    ///
    /// All operations must succeed or none of their effects are applied.
    /// Currently only supports read-only operations.
    fn execute_transactional(
        &self,
        operations: &[BatchOperation],
        request: &BatchRequest,
    ) -> McpResult<Vec<BatchOperationResult>> {
        // Execute all operations
        let results = self.execute_serial(operations, request)?;

        // Check if all succeeded
        let all_succeeded = results.iter().all(|r| r.success);

        if !all_succeeded {
            // In transaction mode, return error for the batch
            return Err(McpError::InvalidRequest(
                "Transaction failed: one or more operations failed. All changes rolled back."
                    .to_string(),
            ));
        }

        Ok(results)
    }

    /// Execute single operation with timing and caching.
    fn execute_single_operation(&self, op: &BatchOperation) -> BatchOperationResult {
        let start = Instant::now();

        // Try cache first
        let result = if let Some(cached) = self.try_cache_get(&op.tool, &op.arguments) {
            Ok(cached)
        } else {
            execute_tool(&op.tool, op.arguments.clone(), self.root_path)
        };

        let duration = start.elapsed();

        match result {
            Ok(tool_result) => {
                // Cache if applicable
                self.try_cache_put(&op.tool, &op.arguments, &tool_result);

                // Check if tool reported an error (e.g., validation failed)
                let is_tool_error = tool_result.is_error.unwrap_or(false);

                // Create error info if tool reported failure
                let error = if is_tool_error {
                    Some(BatchError {
                        code: -32001,
                        message: "Tool execution reported failure".to_string(),
                        data: None,
                    })
                } else {
                    None
                };

                BatchOperationResult {
                    id: op.id.clone(),
                    success: !is_tool_error,
                    result: Some(tool_result),
                    error,
                    duration_ms: duration.as_millis() as u64,
                }
            }
            Err(e) => BatchOperationResult {
                id: op.id.clone(),
                success: false,
                result: None,
                error: Some(BatchError::from(&e)),
                duration_ms: duration.as_millis() as u64,
            },
        }
    }

    /// Create a skipped result for an operation.
    fn create_skipped_result(&self, op: &BatchOperation) -> BatchOperationResult {
        BatchOperationResult {
            id: op.id.clone(),
            success: false,
            result: None,
            error: Some(BatchError {
                code: -32000,
                message: "Operation skipped due to previous failure".to_string(),
                data: None,
            }),
            duration_ms: 0,
        }
    }

    /// Try to get a cached result for an operation.
    fn try_cache_get(
        &self,
        tool_name: &str,
        arguments: &Option<serde_json::Value>,
    ) -> Option<crate::protocol::CallToolResult> {
        let cache = self.cache.as_ref()?;
        let args = arguments.as_ref()?;

        // Extract the primary input field for each cacheable operation
        let cache_key = match tool_name {
            "hedl_validate" => {
                let hedl = args.get("hedl")?.as_str()?;
                let strict = args
                    .get("strict")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                let lint = args
                    .get("lint")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                format!("{hedl}:{strict}:{lint}")
            }
            "hedl_query" => {
                let hedl = args.get("hedl")?.as_str()?;
                let type_name = args.get("type_name").and_then(|v| v.as_str()).unwrap_or("");
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let include_children = args
                    .get("include_children")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                format!("{hedl}:{type_name}:{id}:{include_children}")
            }
            "hedl_stats" => {
                let hedl = args.get("hedl")?.as_str()?;
                let tokenizer = args
                    .get("tokenizer")
                    .and_then(|v| v.as_str())
                    .unwrap_or("simple");
                format!("{hedl}:{tokenizer}")
            }
            _ => return None, // Non-cacheable operation
        };

        let cached_value = cache.get(tool_name, &cache_key)?;
        serde_json::from_value(cached_value).ok()
    }

    /// Try to cache the result of an operation.
    fn try_cache_put(
        &self,
        tool_name: &str,
        arguments: &Option<serde_json::Value>,
        result: &crate::protocol::CallToolResult,
    ) {
        let cache = match &self.cache {
            Some(c) => c,
            None => return,
        };

        let args = match arguments {
            Some(a) => a,
            None => return,
        };

        // Extract the primary input field for each cacheable operation
        let cache_key = match tool_name {
            "hedl_validate" => {
                let hedl = match args.get("hedl").and_then(|v| v.as_str()) {
                    Some(h) => h,
                    None => return,
                };
                let strict = args
                    .get("strict")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                let lint = args
                    .get("lint")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                format!("{hedl}:{strict}:{lint}")
            }
            "hedl_query" => {
                let hedl = match args.get("hedl").and_then(|v| v.as_str()) {
                    Some(h) => h,
                    None => return,
                };
                let type_name = args.get("type_name").and_then(|v| v.as_str()).unwrap_or("");
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let include_children = args
                    .get("include_children")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(true);
                format!("{hedl}:{type_name}:{id}:{include_children}")
            }
            "hedl_stats" => {
                let hedl = match args.get("hedl").and_then(|v| v.as_str()) {
                    Some(h) => h,
                    None => return,
                };
                let tokenizer = args
                    .get("tokenizer")
                    .and_then(|v| v.as_str())
                    .unwrap_or("simple");
                format!("{hedl}:{tokenizer}")
            }
            _ => return, // Non-cacheable operation
        };

        let result_value = match serde_json::to_value(result) {
            Ok(v) => v,
            Err(_) => return,
        };

        cache.insert(tool_name, &cache_key, result_value);
    }

    /// Resolve operation dependencies to determine execution order.
    ///
    /// Uses topological sort to order operations based on their dependencies.
    /// Detects circular dependencies and missing dependencies.
    fn resolve_dependencies(
        &self,
        operations: &[BatchOperation],
    ) -> McpResult<Vec<BatchOperation>> {
        let mut sorted = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        // Build operation map for fast lookup
        let op_map: HashMap<_, _> = operations.iter().map(|op| (op.id.as_str(), op)).collect();

        // Verify all dependencies exist
        for op in operations {
            for dep_id in &op.depends_on {
                if !op_map.contains_key(dep_id.as_str()) {
                    return Err(McpError::InvalidRequest(format!(
                        "Operation '{}' depends on '{}' which does not exist",
                        op.id, dep_id
                    )));
                }
            }
        }

        // Topological sort with cycle detection
        fn visit(
            op_id: &str,
            op_map: &HashMap<&str, &BatchOperation>,
            visited: &mut HashSet<String>,
            visiting: &mut HashSet<String>,
            sorted: &mut Vec<BatchOperation>,
        ) -> McpResult<()> {
            if visited.contains(op_id) {
                return Ok(());
            }

            if visiting.contains(op_id) {
                return Err(McpError::InvalidRequest(format!(
                    "Circular dependency detected involving operation: {op_id}"
                )));
            }

            visiting.insert(op_id.to_string());

            let op = op_map
                .get(op_id)
                .ok_or_else(|| McpError::InvalidRequest(format!("Operation not found: {op_id}")))?;

            // Visit dependencies first
            for dep_id in &op.depends_on {
                visit(dep_id, op_map, visited, visiting, sorted)?;
            }

            visiting.remove(op_id);
            visited.insert(op_id.to_string());
            sorted.push((*op).clone());

            Ok(())
        }

        for op in operations {
            visit(&op.id, &op_map, &mut visited, &mut visiting, &mut sorted)?;
        }

        Ok(sorted)
    }

    /// Build batch response with summary statistics.
    fn build_response(
        &self,
        results: Vec<BatchOperationResult>,
        duration: std::time::Duration,
        parallel: bool,
    ) -> BatchResponse {
        // Error codes:
        // -32000: Skipped operation (due to previous failure)
        // -32001: Tool execution reported failure
        // Other: Various MCP errors
        const SKIPPED_ERROR_CODE: i32 = -32000;

        let total = results.len();
        let succeeded = results.iter().filter(|r| r.success).count();
        // Skipped operations have error code -32000
        let skipped = results
            .iter()
            .filter(|r| {
                !r.success
                    && r.error
                        .as_ref()
                        .is_some_and(|e| e.code == SKIPPED_ERROR_CODE)
            })
            .count();
        // Failed operations have errors but are not skipped
        let failed = results
            .iter()
            .filter(|r| {
                !r.success
                    && r.error
                        .as_ref()
                        .is_some_and(|e| e.code != SKIPPED_ERROR_CODE)
            })
            .count();

        let success = failed == 0;

        BatchResponse {
            results,
            summary: BatchSummary {
                total,
                succeeded,
                failed,
                skipped,
                duration_ms: duration.as_millis() as u64,
                parallel,
            },
            success,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::BatchMode;
    use serde_json::json;

    #[test]
    fn test_resolve_dependencies_simple() {
        let executor = BatchExecutor::new(Path::new("."), None);

        let operations = vec![
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
        ];

        let sorted = executor.resolve_dependencies(&operations).unwrap();
        assert_eq!(sorted.len(), 2);
        assert_eq!(sorted[0].id, "op1");
        assert_eq!(sorted[1].id, "op2");
    }

    #[test]
    fn test_resolve_dependencies_circular() {
        let executor = BatchExecutor::new(Path::new("."), None);

        let operations = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: None,
                depends_on: vec!["op2".to_string()],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "hedl_format".to_string(),
                arguments: None,
                depends_on: vec!["op1".to_string()],
            },
        ];

        let result = executor.resolve_dependencies(&operations);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_dependencies_missing() {
        let executor = BatchExecutor::new(Path::new("."), None);

        let operations = vec![BatchOperation {
            id: "op1".to_string(),
            tool: "hedl_validate".to_string(),
            arguments: None,
            depends_on: vec!["missing".to_string()],
        }];

        let result = executor.resolve_dependencies(&operations);
        assert!(result.is_err());
    }

    #[test]
    fn test_compute_dependency_levels() {
        let executor = BatchExecutor::new(Path::new("."), None);

        let operations = vec![
            BatchOperation {
                id: "op1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: None,
                depends_on: vec![],
            },
            BatchOperation {
                id: "op2".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: None,
                depends_on: vec![],
            },
            BatchOperation {
                id: "op3".to_string(),
                tool: "hedl_format".to_string(),
                arguments: None,
                depends_on: vec!["op1".to_string(), "op2".to_string()],
            },
        ];

        let levels = executor.compute_dependency_levels(&operations);
        assert_eq!(levels.len(), 2);
        assert_eq!(levels[0].len(), 2); // op1 and op2 in parallel
        assert_eq!(levels[1].len(), 1); // op3 depends on both
    }

    #[test]
    fn test_execute_serial() {
        let executor = BatchExecutor::new(Path::new("."), None);

        let operations = vec![
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
        ];

        let request = BatchRequest {
            operations: operations.clone(),
            mode: BatchMode::ContinueOnError,
            parallel: false,
            transaction: false,
            timeout: None,
        };

        let results = executor.execute_serial(&operations, &request).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }

    #[test]
    fn test_execute_stop_on_error() {
        let executor = BatchExecutor::new(Path::new("."), None);

        let operations = vec![
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
        ];

        let request = BatchRequest {
            operations: operations.clone(),
            mode: BatchMode::StopOnError,
            parallel: false,
            transaction: false,
            timeout: None,
        };

        let results = executor.execute_serial(&operations, &request).unwrap();
        assert_eq!(results.len(), 3);
        assert!(results[0].success);
        assert!(!results[1].success);
        assert!(!results[2].success); // Skipped
    }
}
