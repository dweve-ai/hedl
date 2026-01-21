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

//! Batch tool implementation for executing multiple operations.

use crate::batch::BatchRequest;
use crate::batch_executor::BatchExecutor;
use crate::cache::OperationCache;
use crate::error::{McpError, McpResult};
use crate::protocol::{CallToolResult, Content};
use serde_json::Value as JsonValue;
use std::path::Path;
use std::sync::Arc;

/// Execute batch operation.
///
/// # Arguments
///
/// * `arguments` - Batch request containing operations to execute
/// * `root_path` - Root directory for file operations
/// * `cache` - Optional operation cache for result caching
///
/// # Returns
///
/// Batch response with results for all operations and summary statistics.
pub fn execute_batch(
    arguments: Option<JsonValue>,
    root_path: &Path,
    cache: Option<Arc<OperationCache>>,
) -> McpResult<CallToolResult> {
    let args =
        arguments.ok_or_else(|| McpError::InvalidArguments("Missing arguments".to_string()))?;

    let batch_request: BatchRequest = serde_json::from_value(args)
        .map_err(|e| McpError::InvalidArguments(format!("Invalid batch request: {e}")))?;

    let executor = BatchExecutor::new(root_path, cache);
    let response = executor.execute(batch_request)?;

    let response_json = serde_json::to_string_pretty(&response).map_err(McpError::Json)?;

    Ok(CallToolResult {
        content: vec![Content::Text {
            text: response_json,
        }],
        is_error: Some(!response.success),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::{BatchMode, BatchOperation};
    use serde_json::json;

    #[test]
    fn test_execute_batch_missing_arguments() {
        let result = execute_batch(None, Path::new("."), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_batch_invalid_arguments() {
        let args = json!({"invalid": "data"});
        let result = execute_batch(Some(args), Path::new("."), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_batch_valid() {
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

        let args = serde_json::to_value(&batch).unwrap();
        let result = execute_batch(Some(args), Path::new("."), None).unwrap();

        assert!(!result.content.is_empty());
        if let Content::Text { text } = &result.content[0] {
            assert!(text.contains("results"));
            assert!(text.contains("summary"));
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_execute_batch_multiple_operations() {
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
            parallel: true,
            transaction: false,
            timeout: None,
        };

        let args = serde_json::to_value(&batch).unwrap();
        let result = execute_batch(Some(args), Path::new("."), None).unwrap();

        assert!(!result.content.is_empty());
        assert_eq!(result.is_error, Some(false));
    }
}
