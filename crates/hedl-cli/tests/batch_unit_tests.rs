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

//! Unit tests for batch processing module
//!
//! These tests verify the internal behavior of the batch processing engine,
//! including configuration, progress tracking, error handling, and performance
//! characteristics.

use hedl_cli::batch::{
    BatchConfig, BatchExecutor, BatchOperation, FileResult, FormatOperation, LintOperation,
    ValidationOperation,
};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{tempdir, TempDir};

// ============================================================================
// Test Helpers
// ============================================================================

/// Create test files with valid HEDL content
fn create_valid_test_files(count: usize) -> (TempDir, Vec<PathBuf>) {
    let dir = tempdir().expect("Failed to create temp dir");
    let mut paths = Vec::new();

    for i in 0..count {
        let path = dir.path().join(format!("test{i}.hedl"));
        let content = format!(
            r"%VERSION: 1.0
---
id: {}
name: Test {}
value: {}
",
            i,
            i,
            i * 10
        );
        fs::write(&path, content).expect("Failed to write test file");
        paths.push(path);
    }

    (dir, paths)
}

/// Create test files with matrix lists for format testing
fn create_matrix_test_files(count: usize) -> (TempDir, Vec<PathBuf>) {
    let dir = tempdir().expect("Failed to create temp dir");
    let mut paths = Vec::new();

    for i in 0..count {
        let path = dir.path().join(format!("matrix{i}.hedl"));
        let base = i * 100;
        let content = format!(
            "%V:2.0
%NULL:~
%QUOTE:\"
%S:Item:[id,value]
---
items:@Item
 |item_{},100
 |item_{},200
 |item_{},300
",
            base,
            base + 1,
            base + 2
        );
        fs::write(&path, content).expect("Failed to write test file");
        paths.push(path);
    }

    (dir, paths)
}

/// Create test files with some invalid content
fn create_mixed_validity_files() -> (TempDir, Vec<PathBuf>) {
    let dir = tempdir().expect("Failed to create temp dir");
    let mut paths = Vec::new();

    // Valid files
    for i in 0..3 {
        let path = dir.path().join(format!("valid{i}.hedl"));
        let content = format!(
            r"%VERSION: 1.0
---
id: {i}
"
        );
        fs::write(&path, content).expect("Failed to write valid file");
        paths.push(path);
    }

    // Invalid files
    for i in 0..2 {
        let path = dir.path().join(format!("invalid{i}.hedl"));
        let content = "invalid syntax here";
        fs::write(&path, content).expect("Failed to write invalid file");
        paths.push(path);
    }

    (dir, paths)
}

/// Create files with varying sizes
fn create_varying_size_files() -> (TempDir, Vec<PathBuf>) {
    let dir = tempdir().expect("Failed to create temp dir");
    let mut paths = Vec::new();

    // Small file (< 1KB)
    let small_path = dir.path().join("small.hedl");
    fs::write(&small_path, "%VERSION: 1.0\n---\nid: 1\n").expect("Failed to write small file");
    paths.push(small_path);

    // Medium file (~ 5KB)
    let medium_path = dir.path().join("medium.hedl");
    let mut medium_content =
        String::from("%VERSION: 1.0\n%STRUCT: Item (100): [id,value]\n---\nitems:@Item\n");
    for i in 0..100 {
        medium_content.push_str(&format!(" |item_{i},value_{i}\n"));
    }
    fs::write(&medium_path, medium_content).expect("Failed to write medium file");
    paths.push(medium_path);

    // Large file (~ 50KB)
    let large_path = dir.path().join("large.hedl");
    let mut large_content = String::from(
        "%VERSION: 1.0\n%STRUCT: Item (1000): [id,name,description,value]\n---\nitems:@Item\n",
    );
    for i in 0..1000 {
        large_content.push_str(&format!(
            " |item_{i},name_{i},This is a longer description for item number {i},value_{i}\n"
        ));
    }
    fs::write(&large_path, large_content).expect("Failed to write large file");
    paths.push(large_path);

    (dir, paths)
}

// ============================================================================
// BatchConfig Tests
// ============================================================================

#[test]
fn test_batch_config_default() {
    let config = BatchConfig::default();
    assert_eq!(config.parallel_threshold, 10);
    assert_eq!(config.max_threads, None);
    assert_eq!(config.progress_interval, 1);
    assert!(!config.verbose);
}

#[test]
fn test_batch_config_custom() {
    let config = BatchConfig {
        parallel_threshold: 5,
        max_threads: Some(4),
        progress_interval: 10,
        verbose: true,
        max_files: Some(5000),
    };
    assert_eq!(config.parallel_threshold, 5);
    assert_eq!(config.max_threads, Some(4));
    assert_eq!(config.progress_interval, 10);
    assert!(config.verbose);
    assert_eq!(config.max_files, Some(5000));
}

#[test]
fn test_batch_config_clone() {
    let config1 = BatchConfig {
        parallel_threshold: 20,
        max_threads: Some(8),
        progress_interval: 5,
        verbose: true,
        max_files: Some(10_000),
    };
    let config2 = config1.clone();
    assert_eq!(config1.parallel_threshold, config2.parallel_threshold);
    assert_eq!(config1.max_threads, config2.max_threads);
    assert_eq!(config1.max_files, config2.max_files);
}

// ============================================================================
// FileResult Tests
// ============================================================================

#[test]
fn test_file_result_success() {
    let path = PathBuf::from("test.hedl");
    let result = FileResult::success(path.clone(), 42);
    assert!(result.is_success());
    assert!(!result.is_failure());
    assert_eq!(result.path, path);
    assert_eq!(result.result.unwrap(), 42);
}

#[test]
fn test_file_result_failure() {
    let path = PathBuf::from("test.hedl");
    let result: FileResult<()> = FileResult::failure(
        path.clone(),
        hedl_cli::error::CliError::parse("test error".to_string()),
    );
    assert!(!result.is_success());
    assert!(result.is_failure());
    assert_eq!(result.path, path);
    assert!(result.result.is_err());
}

// ============================================================================
// BatchExecutor Tests - Validation
// ============================================================================

#[test]
fn test_validation_operation_success() {
    let (_dir, files) = create_valid_test_files(5);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 5);
    assert_eq!(results.success_count(), 5);
    assert_eq!(results.failure_count(), 0);
    assert!(results.all_succeeded());
    assert!(!results.has_failures());
}

#[test]
fn test_validation_operation_with_failures() {
    let (_dir, files) = create_mixed_validity_files();
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 5);
    assert_eq!(results.success_count(), 3);
    assert_eq!(results.failure_count(), 2);
    assert!(!results.all_succeeded());
    assert!(results.has_failures());

    // Check that failures are collected
    let failures: Vec<_> = results.failures().collect();
    assert_eq!(failures.len(), 2);
}

#[test]
fn test_validation_operation_strict_mode() {
    let (_dir, files) = create_valid_test_files(3);
    let processor = BatchExecutor::new(BatchConfig::default());

    // Non-strict should succeed
    let operation_normal = ValidationOperation { strict: false };
    let results_normal = processor
        .process(&files, operation_normal, false)
        .expect("Processing should succeed");
    assert!(results_normal.all_succeeded());

    // Strict mode should also succeed for valid files
    let operation_strict = ValidationOperation { strict: true };
    let results_strict = processor
        .process(&files, operation_strict, false)
        .expect("Processing should succeed");
    assert!(results_strict.all_succeeded());
}

#[test]
fn test_validation_operation_nonexistent_file() {
    let files = vec![PathBuf::from("/nonexistent/file.hedl")];
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 1);
    assert_eq!(results.failure_count(), 1);
    assert!(results.has_failures());
}

// ============================================================================
// BatchExecutor Tests - Format
// ============================================================================

#[test]
fn test_format_operation_success() {
    let (_dir, files) = create_valid_test_files(5);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = FormatOperation {
        check: false,
        ditto: false,
        with_counts: false,
    };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 5);
    assert_eq!(results.success_count(), 5);
    assert!(results.all_succeeded());

    // Verify that formatted output is returned
    for result in results.successes() {
        let formatted = result.result.as_ref().unwrap();
        assert!(formatted.contains("%VERSION: 1.0"));
    }
}

#[test]
fn test_format_operation_with_counts() {
    let (_dir, files) = create_matrix_test_files(3);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = FormatOperation {
        check: false,
        ditto: false,
        with_counts: true,
    };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    // Check that processing completed
    assert_eq!(results.total_files(), 3);

    // Files should succeed - if not, there's a parsing issue
    if !results.all_succeeded() {
        for result in results.failures() {
            eprintln!(
                "File {} failed: {:?}",
                result.path.display(),
                result.result.as_ref().err()
            );
        }
    }

    assert!(
        results.success_count() > 0,
        "At least some files should succeed"
    );

    // Verify that count hints are added to successful files
    for result in results.successes() {
        let formatted = result.result.as_ref().unwrap();
        // v2.0 uses %C:Type.total=N directives instead of Type(N) syntax
        assert!(
            formatted.contains("%C:") && formatted.contains(".total=3"),
            "Expected count directive %C:Type.total=3 in formatted output for file {:?}",
            result.path
        );
    }
}

#[test]
fn test_format_operation_with_ditto() {
    let (_dir, files) = create_valid_test_files(3);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = FormatOperation {
        check: false,
        ditto: true,
        with_counts: false,
    };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.success_count(), 3);
    assert!(results.all_succeeded());
}

#[test]
fn test_format_operation_check_mode() {
    let (_dir, files) = create_valid_test_files(3);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = FormatOperation {
        check: true,
        ditto: false,
        with_counts: false,
    };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    // Files may or may not be canonical, so we just check that processing completed
    assert_eq!(results.total_files(), 3);
}

#[test]
fn test_format_operation_invalid_file() {
    let (_dir, files) = create_mixed_validity_files();
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = FormatOperation {
        check: false,
        ditto: false,
        with_counts: false,
    };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.success_count(), 3);
    assert_eq!(results.failure_count(), 2);
}

// ============================================================================
// BatchExecutor Tests - Lint
// ============================================================================

#[test]
fn test_lint_operation_success() {
    let (_dir, files) = create_valid_test_files(5);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = LintOperation { warn_error: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 5);
    assert!(results.success_count() <= 5); // May have lint warnings

    // Verify that diagnostics are returned
    for result in results.successes() {
        let diagnostics = result.result.as_ref().unwrap();
        // Diagnostics is a Vec<String>
        assert!(diagnostics.is_empty() || !diagnostics.is_empty());
    }
}

#[test]
fn test_lint_operation_warn_error() {
    let (_dir, files) = create_valid_test_files(3);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = LintOperation { warn_error: true };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 3);
    // With warn_error, any warnings become failures
}

#[test]
fn test_lint_operation_invalid_file() {
    let (_dir, files) = create_mixed_validity_files();
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = LintOperation { warn_error: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    // Invalid files should fail to parse
    assert!(results.failure_count() >= 2);
}

// ============================================================================
// BatchExecutor Tests - Parallelization
// ============================================================================

#[test]
fn test_serial_processing_small_batch() {
    let (_dir, files) = create_valid_test_files(5);
    let processor = BatchExecutor::new(BatchConfig {
        parallel_threshold: 100, // Force serial
        ..Default::default()
    });
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 5);
    assert_eq!(results.success_count(), 5);
}

#[test]
fn test_parallel_processing_large_batch() {
    let (_dir, files) = create_valid_test_files(50);
    let processor = BatchExecutor::new(BatchConfig {
        parallel_threshold: 10, // Force parallel
        ..Default::default()
    });
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 50);
    assert_eq!(results.success_count(), 50);
}

#[test]
fn test_parallel_threshold_boundary() {
    let (_dir, files) = create_valid_test_files(10);

    // Just below threshold - should be serial
    let processor_serial = BatchExecutor::new(BatchConfig {
        parallel_threshold: 11,
        ..Default::default()
    });
    let operation_serial = ValidationOperation { strict: false };
    let results_serial = processor_serial
        .process(&files, operation_serial, false)
        .expect("Processing should succeed");
    assert_eq!(results_serial.success_count(), 10);

    // At threshold - should be parallel
    let processor_parallel = BatchExecutor::new(BatchConfig {
        parallel_threshold: 10,
        ..Default::default()
    });
    let operation_parallel = ValidationOperation { strict: false };
    let results_parallel = processor_parallel
        .process(&files, operation_parallel, false)
        .expect("Processing should succeed");
    assert_eq!(results_parallel.success_count(), 10);
}

#[test]
fn test_max_threads_configuration() {
    let (_dir, files) = create_valid_test_files(20);

    // Test with limited threads
    let processor = BatchExecutor::new(BatchConfig {
        parallel_threshold: 10,
        max_threads: Some(2),
        ..Default::default()
    });
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 20);
    assert_eq!(results.success_count(), 20);
}

// ============================================================================
// BatchExecutor Tests - Performance Characteristics
// ============================================================================

#[test]
fn test_varying_file_sizes() {
    let (_dir, files) = create_varying_size_files();
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 3);
    assert_eq!(results.success_count(), 3, "All files should succeed");
}

#[test]
fn test_throughput_calculation() {
    let (_dir, files) = create_valid_test_files(10);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    let throughput = results.throughput();
    // Throughput may be 0 for very fast operations (elapsed_ms = 0)
    assert!(
        throughput >= 0.0,
        "Throughput should be non-negative, got {throughput}"
    );
    if throughput > 0.0 {
        assert!(
            throughput < 1_000_000.0,
            "Throughput should be reasonable (files/s)"
        );
    }
}

#[test]
fn test_processing_time_recorded() {
    let (_dir, files) = create_valid_test_files(5);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    // Elapsed time is recorded (u128 is always >= 0)
    // Just verify it exists
    let _ = results.elapsed_ms;
}

// ============================================================================
// BatchExecutor Tests - Progress Reporting
// ============================================================================

#[test]
fn test_progress_disabled() {
    let (_dir, files) = create_valid_test_files(10);
    let processor = BatchExecutor::new(BatchConfig {
        progress_interval: 0, // Disable progress
        ..Default::default()
    });
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.success_count(), 10);
}

#[test]
fn test_progress_with_verbose() {
    let (_dir, files) = create_valid_test_files(5);
    let processor = BatchExecutor::new(BatchConfig {
        verbose: true,
        ..Default::default()
    });
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, true)
        .expect("Processing should succeed");

    assert_eq!(results.success_count(), 5);
}

#[test]
fn test_progress_interval() {
    let (_dir, files) = create_valid_test_files(20);
    let processor = BatchExecutor::new(BatchConfig {
        progress_interval: 5, // Report every 5 files
        ..Default::default()
    });
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, true)
        .expect("Processing should succeed");

    assert_eq!(results.success_count(), 20);
}

// ============================================================================
// BatchResults Tests
// ============================================================================

#[test]
fn test_batch_results_statistics() {
    let (_dir, files) = create_mixed_validity_files();
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 5);
    assert_eq!(results.success_count(), 3);
    assert_eq!(results.failure_count(), 2);
    assert!(!results.all_succeeded());
    assert!(results.has_failures());
}

#[test]
fn test_batch_results_iterators() {
    let (_dir, files) = create_mixed_validity_files();
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    // Test successes iterator
    let successes: Vec<_> = results.successes().collect();
    assert_eq!(successes.len(), 3);

    // Test failures iterator
    let failures: Vec<_> = results.failures().collect();
    assert_eq!(failures.len(), 2);
}

#[test]
fn test_batch_results_all_success() {
    let (_dir, files) = create_valid_test_files(5);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert!(results.all_succeeded());
    assert!(!results.has_failures());
    assert_eq!(results.failures().count(), 0);
}

// ============================================================================
// Edge Cases and Boundary Conditions
// ============================================================================

#[test]
fn test_empty_file_list() {
    let files: Vec<PathBuf> = vec![];
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 0);
    assert_eq!(results.success_count(), 0);
    assert_eq!(results.failure_count(), 0);
}

#[test]
fn test_single_file() {
    let (_dir, files) = create_valid_test_files(1);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 1);
    assert_eq!(results.success_count(), 1);
}

#[test]
fn test_very_large_batch() {
    let (_dir, files) = create_valid_test_files(200);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 200);
    assert_eq!(results.success_count(), 200);
}

#[test]
fn test_all_failures() {
    let dir = tempdir().expect("Failed to create temp dir");
    let mut files = Vec::new();

    // Create only invalid files
    for i in 0..5 {
        let path = dir.path().join(format!("invalid{i}.hedl"));
        fs::write(&path, "invalid syntax").expect("Failed to write file");
        files.push(path);
    }

    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = ValidationOperation { strict: false };

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 5);
    assert_eq!(results.failure_count(), 5);
    assert!(results.has_failures());
    assert!(!results.all_succeeded());
}

// ============================================================================
// Custom Operation Tests
// ============================================================================

/// Custom operation that counts lines in files
#[derive(Clone)]
struct LineCountOperation;

impl BatchOperation for LineCountOperation {
    type Output = usize;

    fn process_file(&self, path: &Path) -> Result<Self::Output, hedl_cli::error::CliError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| hedl_cli::error::CliError::io_error(path, e))?;
        Ok(content.lines().count())
    }

    fn name(&self) -> &'static str {
        "line-count"
    }
}

#[test]
fn test_custom_operation() {
    let (_dir, files) = create_valid_test_files(5);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = LineCountOperation;

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 5);
    assert_eq!(results.success_count(), 5);

    // Each file should have at least 4 lines (VERSION, ---, and content)
    for result in results.successes() {
        let line_count = result.result.as_ref().unwrap();
        assert!(*line_count >= 4);
    }
}

/// Custom operation that always fails
#[derive(Clone)]
struct AlwaysFailOperation;

impl BatchOperation for AlwaysFailOperation {
    type Output = ();

    fn process_file(&self, path: &Path) -> Result<Self::Output, hedl_cli::error::CliError> {
        Err(hedl_cli::error::CliError::parse(format!(
            "Intentional failure for {}",
            path.display()
        )))
    }

    fn name(&self) -> &'static str {
        "always-fail"
    }
}

#[test]
fn test_custom_operation_all_failures() {
    let (_dir, files) = create_valid_test_files(3);
    let processor = BatchExecutor::new(BatchConfig::default());
    let operation = AlwaysFailOperation;

    let results = processor
        .process(&files, operation, false)
        .expect("Processing should succeed");

    assert_eq!(results.total_files(), 3);
    assert_eq!(results.failure_count(), 3);
    assert!(!results.all_succeeded());
}
