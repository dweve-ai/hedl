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

//! Performance tests for parallel file reading.
//!
//! These tests verify the correctness and basic performance characteristics
//! of parallel file reading. Note that debug builds have significant thread
//! pool overhead, so actual speedup measurements should be done with:
//!
//! ```bash
//! cargo bench -p hedl-mcp --bench parallel_file_reading
//! ```
//!
//! Expected speedups in **release mode**:
//! - 10 files: 2-3x
//! - 50 files: 3-5x
//! - 100 files: 4-6x
//!
//! Debug mode typically shows 0.8-1.5x due to overhead.

use hedl_mcp::execute_tool;
use serde_json::json;
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

/// Create test directory with N HEDL files
fn create_test_files(n: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    for i in 0..n {
        // Create simpler HEDL files that are definitely valid
        let content = format!(
            "%VERSION: 1.0\n%STRUCT: Item: [id, name, value]\n---\nitems{}: @Item\n  | item{}, Item {}, {}\n  | item{}, Item {}, {}\n",
            i,
            i, i, i * 10,
            i + 1000, i + 1000, (i + 1000) * 10
        );
        fs::write(
            temp_dir.path().join(format!("file{:03}.hedl", i)),
            content,
        )
        .unwrap();
    }

    temp_dir
}

#[test]
fn test_parallel_performance_100_files() {
    // This test measures performance but doesn't enforce strict requirements
    // due to debug mode overhead. See benchmarks for release mode measurements.
    let temp_dir = create_test_files(100);

    // Sequential read
    let args = json!({
        "path": ".",
        "num_threads": 1,
        "include_json": false
    });
    let start = Instant::now();
    execute_tool("hedl_read", Some(args), temp_dir.path()).unwrap();
    let sequential_time = start.elapsed();

    // Parallel read
    let args_par = json!({
        "path": ".",
        "include_json": false
    });
    let start = Instant::now();
    execute_tool("hedl_read", Some(args_par), temp_dir.path()).unwrap();
    let parallel_time = start.elapsed();

    let speedup = sequential_time.as_secs_f64() / parallel_time.as_secs_f64();

    println!("Sequential time (100 files): {:?}", sequential_time);
    println!("Parallel time (100 files): {:?}", parallel_time);
    println!("Speedup: {:.2}x (debug mode - see benchmarks for release mode)", speedup);

    // Just verify both modes complete successfully
    // Actual speedup verification is done in release mode benchmarks
}

#[test]
fn test_thread_configuration() {
    // Test that different thread counts work correctly
    let temp_dir = create_test_files(20);

    // Test 1 thread (sequential)
    let args_1 = json!({
        "path": ".",
        "num_threads": 1,
        "include_json": false
    });
    let result = execute_tool("hedl_read", Some(args_1), temp_dir.path()).unwrap();
    assert!(!result.content.is_empty());

    // Test 2 threads
    let args_2 = json!({
        "path": ".",
        "num_threads": 2,
        "include_json": false
    });
    let result = execute_tool("hedl_read", Some(args_2), temp_dir.path()).unwrap();
    assert!(!result.content.is_empty());

    // Test 4 threads
    let args_4 = json!({
        "path": ".",
        "num_threads": 4,
        "include_json": false
    });
    let result = execute_tool("hedl_read", Some(args_4), temp_dir.path()).unwrap();
    assert!(!result.content.is_empty());

    // Test default (None)
    let args_default = json!({
        "path": ".",
        "include_json": false
    });
    let result = execute_tool("hedl_read", Some(args_default), temp_dir.path()).unwrap();
    assert!(!result.content.is_empty());

    println!("All thread configurations work correctly");
}

#[test]
fn test_thread_count_measurement() {
    // Informational test showing performance with different thread counts
    // No strict assertions - just measurements
    let temp_dir = create_test_files(20);

    let thread_counts = [1, 2, 4];
    println!("\nThread count performance comparison (debug mode):");

    for &threads in &thread_counts {
        let args = json!({
            "path": ".",
            "num_threads": threads,
            "include_json": false
        });

        let start = Instant::now();
        for _ in 0..3 {
            execute_tool("hedl_read", Some(args.clone()), temp_dir.path()).unwrap();
        }
        let elapsed = start.elapsed();

        println!("  {} thread(s): {:?}", threads, elapsed);
    }

    println!("Note: Run benchmarks in release mode for accurate speedup measurements");
}

#[test]
fn test_parallel_correctness() {
    let temp_dir = create_test_files(30);

    // Read sequentially
    let args_seq = json!({
        "path": ".",
        "num_threads": 1,
        "include_json": false
    });
    let result_seq = execute_tool("hedl_read", Some(args_seq), temp_dir.path()).unwrap();

    // Read in parallel
    let args_par = json!({
        "path": ".",
        "include_json": false
    });
    let result_par = execute_tool("hedl_read", Some(args_par), temp_dir.path()).unwrap();

    // Both should read the same number of files
    assert_eq!(result_seq.content.len(), result_par.content.len());

    // Extract text content and parse
    let text_seq = match &result_seq.content[0] {
        hedl_mcp::Content::Text { text } => text,
        _ => panic!("Expected text content"),
    };
    let text_par = match &result_par.content[0] {
        hedl_mcp::Content::Text { text } => text,
        _ => panic!("Expected text content"),
    };

    let parsed_seq: serde_json::Value = serde_json::from_str(text_seq).unwrap();
    let parsed_par: serde_json::Value = serde_json::from_str(text_par).unwrap();

    // Both should report reading 30 files
    assert_eq!(parsed_seq["files_read"], 30);
    assert_eq!(parsed_par["files_read"], 30);

    // All files should be successfully parsed (no errors)
    let results_seq = parsed_seq["results"].as_array().unwrap();
    let results_par = parsed_par["results"].as_array().unwrap();

    for result in results_seq {
        assert!(
            result.get("error").is_none(),
            "Sequential read had error: {:?}",
            result
        );
    }

    for result in results_par {
        assert!(
            result.get("error").is_none(),
            "Parallel read had error: {:?}",
            result
        );
    }
}
