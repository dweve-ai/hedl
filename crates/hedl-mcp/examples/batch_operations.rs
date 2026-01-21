// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Example demonstrating batch operations in the HEDL MCP server.

use hedl_mcp::batch::{BatchMode, BatchOperation, BatchRequest};
use hedl_mcp::BatchExecutor;
use serde_json::json;
use std::path::Path;

fn main() {
    println!("HEDL MCP Batch Operations Examples\n");
    println!("===================================\n");

    example_basic_batch();
    example_dependency_resolution();
    example_parallel_execution();
    example_error_handling();
    example_mixed_tools();
}

fn example_basic_batch() {
    println!("Example 1: Basic Batch Execution");
    println!("---------------------------------");

    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "val1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "val2".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    match executor.execute(batch) {
        Ok(response) => {
            println!("Success: {}", response.success);
            println!("Total operations: {}", response.summary.total);
            println!("Succeeded: {}", response.summary.succeeded);
            println!("Failed: {}", response.summary.failed);
            println!("Duration: {}ms\n", response.summary.duration_ms);
        }
        Err(e) => {
            eprintln!("Error: {e}\n");
        }
    }
}

fn example_dependency_resolution() {
    println!("Example 2: Dependency Resolution");
    println!("---------------------------------");

    let executor = BatchExecutor::new(Path::new("."), None);

    // Operations submitted out of order, but executed in dependency order
    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "step3".to_string(),
                tool: "hedl_stats".to_string(),
                arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                depends_on: vec!["step2".to_string()],
            },
            BatchOperation {
                id: "step1".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "step2".to_string(),
                tool: "hedl_format".to_string(),
                arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                depends_on: vec!["step1".to_string()],
            },
        ],
        mode: BatchMode::StopOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    match executor.execute(batch) {
        Ok(response) => {
            println!("Execution order (by result):");
            for (i, result) in response.results.iter().enumerate() {
                println!("  {}. {}", i + 1, result.id);
            }
            println!("Total duration: {}ms\n", response.summary.duration_ms);
        }
        Err(e) => {
            eprintln!("Error: {e}\n");
        }
    }
}

fn example_parallel_execution() {
    println!("Example 3: Parallel Execution");
    println!("------------------------------");

    let executor = BatchExecutor::new(Path::new("."), None);

    // Create 10 independent validation operations
    let operations: Vec<_> = (0..10)
        .map(|i| BatchOperation {
            id: format!("val_{i}"),
            tool: "hedl_validate".to_string(),
            arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
            depends_on: vec![],
        })
        .collect();

    let batch_parallel = BatchRequest {
        operations: operations.clone(),
        mode: BatchMode::ContinueOnError,
        parallel: true,
        transaction: false,
        timeout: None,
    };

    let batch_serial = BatchRequest {
        operations,
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let parallel_result = executor.execute(batch_parallel).unwrap();
    let serial_result = executor.execute(batch_serial).unwrap();

    println!(
        "Parallel execution: {}ms",
        parallel_result.summary.duration_ms
    );
    println!("Serial execution: {}ms", serial_result.summary.duration_ms);
    println!(
        "Speedup: {:.2}x\n",
        serial_result.summary.duration_ms as f64 / parallel_result.summary.duration_ms as f64
    );
}

fn example_error_handling() {
    println!("Example 4: Error Handling");
    println!("-------------------------");

    let executor = BatchExecutor::new(Path::new("."), None);

    let operations = vec![
        BatchOperation {
            id: "valid1".to_string(),
            tool: "hedl_validate".to_string(),
            arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
            depends_on: vec![],
        },
        BatchOperation {
            id: "invalid".to_string(),
            tool: "hedl_validate".to_string(),
            arguments: Some(json!({"hedl": "invalid hedl"})),
            depends_on: vec![],
        },
        BatchOperation {
            id: "valid2".to_string(),
            tool: "hedl_validate".to_string(),
            arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
            depends_on: vec![],
        },
    ];

    // Continue-on-error mode
    println!("Continue-on-error mode:");
    let batch = BatchRequest {
        operations: operations.clone(),
        mode: BatchMode::ContinueOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    println!("  Succeeded: {}", response.summary.succeeded);
    println!("  Failed: {}", response.summary.failed);
    println!("  Total executed: {}", response.results.len());

    // Stop-on-error mode
    println!("\nStop-on-error mode:");
    let batch = BatchRequest {
        operations,
        mode: BatchMode::StopOnError,
        parallel: false,
        transaction: false,
        timeout: None,
    };

    let response = executor.execute(batch).unwrap();
    println!("  Succeeded: {}", response.summary.succeeded);
    println!("  Failed: {}", response.summary.failed);
    println!("  Skipped: {}", response.summary.skipped);
    println!("  Total executed: {}\n", response.results.len());
}

fn example_mixed_tools() {
    println!("Example 5: Mixed Tool Types");
    println!("---------------------------");

    let executor = BatchExecutor::new(Path::new("."), None);

    let batch = BatchRequest {
        operations: vec![
            BatchOperation {
                id: "validate".to_string(),
                tool: "hedl_validate".to_string(),
                arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "format".to_string(),
                tool: "hedl_format".to_string(),
                arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                depends_on: vec![],
            },
            BatchOperation {
                id: "stats".to_string(),
                tool: "hedl_stats".to_string(),
                arguments: Some(json!({"hedl": "%VERSION 1.0\n---"})),
                depends_on: vec![],
            },
        ],
        mode: BatchMode::ContinueOnError,
        parallel: true,
        transaction: false,
        timeout: None,
    };

    match executor.execute(batch) {
        Ok(response) => {
            println!(
                "Executed {} different tools in parallel",
                response.summary.total
            );
            println!("Total duration: {}ms", response.summary.duration_ms);
            println!("All succeeded: {}\n", response.success);
        }
        Err(e) => {
            eprintln!("Error: {e}\n");
        }
    }
}
