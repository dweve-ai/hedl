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

//! Batch processing for multiple HEDL files with parallel execution and progress reporting.
//!
//! This module provides efficient batch processing capabilities for operations on multiple
//! HEDL files. It uses Rayon for parallel processing when beneficial and provides real-time
//! progress reporting with detailed error tracking.
//!
//! # Features
//!
//! - **Parallel Processing**: Automatic parallelization using Rayon's work-stealing scheduler
//! - **Progress Reporting**: Real-time progress with file counts and success/failure tracking
//! - **Error Resilience**: Continues processing on errors, collecting all failures for reporting
//! - **Performance Optimization**: Intelligent parallel/serial mode selection based on workload
//! - **Type Safety**: Strongly typed operation definitions with compile-time guarantees
//!
//! # Architecture
//!
//! The batch processing system uses a functional architecture with:
//! - Operation trait for extensible batch operations
//! - Result aggregation with detailed error context
//! - Atomic counters for thread-safe progress tracking
//! - Zero-copy file path handling
//!
//! # Examples
//!
//! ```rust,no_run
//! use hedl_cli::batch::{BatchExecutor, BatchConfig, ValidationOperation};
//! use std::path::PathBuf;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a batch processor with default configuration
//! let processor = BatchExecutor::new(BatchConfig::default());
//!
//! // Validate multiple files in parallel
//! let files = vec![
//!     PathBuf::from("file1.hedl"),
//!     PathBuf::from("file2.hedl"),
//!     PathBuf::from("file3.hedl"),
//! ];
//!
//! let operation = ValidationOperation { strict: true };
//! let results = processor.process(&files, operation, true)?;
//!
//! println!("Processed {} files, {} succeeded, {} failed",
//!     results.total_files(),
//!     results.success_count(),
//!     results.failure_count()
//! );
//! # Ok(())
//! # }
//! ```
//!
//! # Performance Characteristics
//!
//! - **Small batches (< 10 files)**: Serial processing to avoid overhead
//! - **Medium batches (10-100 files)**: Parallel with Rayon thread pool
//! - **Large batches (> 100 files)**: Chunked parallel processing with progress updates
//!
//! # Thread Safety
//!
//! All progress tracking uses atomic operations for lock-free concurrent access.
//! Operations are required to be Send + Sync for parallel execution.
//!
//! # Thread Pool Management
//!
//! The batch processor supports two thread pool strategies:
//!
//! ## Global Thread Pool (Default)
//!
//! When `max_threads` is `None`, operations use Rayon's global thread pool:
//! - Zero overhead (no pool creation)
//! - Shared across all Rayon operations in the process
//! - Thread count typically matches CPU core count
//!
//! ## Local Thread Pool (Isolated)
//!
//! When `max_threads` is `Some(n)`, each operation creates an isolated local pool:
//! - Guaranteed thread count of exactly `n` threads
//! - No global state pollution
//! - Supports concurrent operations with different configurations
//! - Small creation overhead (~0.5-1ms) and memory cost (~2-8MB per thread)
//!
//! # Examples
//!
//! ```rust,no_run
//! use hedl_cli::batch::{BatchExecutor, BatchConfig};
//! use std::path::PathBuf;
//!
//! // Concurrent operations with different thread counts
//! use std::thread;
//!
//! let files: Vec<PathBuf> = vec!["a.hedl".into(), "b.hedl".into()];
//!
//! let handle1 = thread::spawn(|| {
//!     let processor = BatchExecutor::new(BatchConfig {
//!         max_threads: Some(2),
//!         ..Default::default()
//!     });
//!     // Uses 2 threads
//! });
//!
//! let handle2 = thread::spawn(|| {
//!     let processor = BatchExecutor::new(BatchConfig {
//!         max_threads: Some(4),
//!         ..Default::default()
//!     });
//!     // Uses 4 threads, isolated from handle1
//! });
//! ```

mod config;
mod executor;
mod operations;
mod results;
mod traits;

// Re-export public API
pub use config::{get_max_batch_files, validate_file_count, warn_large_batch, BatchConfig};
pub use executor::BatchExecutor;
pub use operations::{
    FormatOperation, LintOperation, StreamingValidationOperation, ValidationOperation,
    ValidationStats,
};
pub use results::{BatchResults, FileResult};
pub use traits::{BatchOperation, StreamingBatchOperation};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CliError;
    use serial_test::serial;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.parallel_threshold, 10);
        assert!(config.max_threads.is_none());
        assert_eq!(config.progress_interval, 1);
        assert!(!config.verbose);
    }

    #[test]
    fn test_file_result_success() {
        let result = FileResult::success(PathBuf::from("test.hedl"), 42);
        assert!(result.is_success());
        assert!(!result.is_failure());
        assert_eq!(result.result.unwrap(), 42);
    }

    #[test]
    fn test_file_result_failure() {
        let result: FileResult<()> =
            FileResult::failure(PathBuf::from("test.hedl"), CliError::NotCanonical);
        assert!(!result.is_success());
        assert!(result.is_failure());
        assert!(result.result.is_err());
    }

    #[test]
    fn test_batch_results_statistics() {
        let results = vec![
            FileResult::success(PathBuf::from("a.hedl"), ()),
            FileResult::success(PathBuf::from("b.hedl"), ()),
            FileResult::failure(PathBuf::from("c.hedl"), CliError::NotCanonical),
        ];

        let batch = BatchResults::new(results, 1000);

        assert_eq!(batch.total_files(), 3);
        assert_eq!(batch.success_count(), 2);
        assert_eq!(batch.failure_count(), 1);
        assert!(!batch.all_succeeded());
        assert!(batch.has_failures());
        assert_eq!(batch.successes().count(), 2);
        assert_eq!(batch.failures().count(), 1);
    }

    #[test]
    fn test_batch_results_throughput() {
        let results = vec![
            FileResult::success(PathBuf::from("a.hedl"), ()),
            FileResult::success(PathBuf::from("b.hedl"), ()),
        ];

        let batch = BatchResults::new(results, 1000); // 1 second
        assert!((batch.throughput() - 2.0).abs() < 0.01);

        let batch_zero: BatchResults<()> = BatchResults::new(vec![], 0);
        assert_eq!(batch_zero.throughput(), 0.0);
    }

    // Mock operation for testing
    struct MockOperation {
        should_fail: bool,
    }

    impl BatchOperation for MockOperation {
        type Output = String;

        fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
            if self.should_fail {
                Err(CliError::NotCanonical)
            } else {
                Ok(path.to_string_lossy().to_string())
            }
        }

        fn name(&self) -> &'static str {
            "mock"
        }
    }

    #[test]
    fn test_batch_processor_empty() {
        let processor = BatchExecutor::default_config();
        let results = processor
            .process(&[], MockOperation { should_fail: false }, false)
            .unwrap();

        assert_eq!(results.total_files(), 0);
        assert!(results.all_succeeded());
    }

    #[test]
    fn test_batch_processor_empty_with_progress_shows_warning() {
        // This test verifies that empty file list with show_progress=true
        // completes successfully (does not panic or return an error).
        // The actual warning output goes to stderr and is difficult to capture
        // in unit tests, but integration tests verify the output.
        let processor = BatchExecutor::default_config();

        let results = processor
            .process(&[], MockOperation { should_fail: false }, true)
            .unwrap();

        // Empty batch should succeed (not error)
        assert_eq!(results.total_files(), 0);
        assert_eq!(results.success_count(), 0);
        assert_eq!(results.failure_count(), 0);
        assert!(results.all_succeeded());
    }

    #[test]
    fn test_batch_processor_empty_without_progress_silent() {
        // Verify that empty file list with show_progress=false succeeds silently
        let processor = BatchExecutor::default_config();

        let results = processor
            .process(&[], MockOperation { should_fail: false }, false)
            .unwrap();

        assert_eq!(results.total_files(), 0);
        assert!(results.all_succeeded());
        // No warning should be printed (verified via integration test)
    }

    #[test]
    fn test_empty_batch_returns_ok_not_error() {
        // Ensure backward compatibility: empty batch is NOT an error condition
        let processor = BatchExecutor::default_config();

        let result = processor.process(&[], MockOperation { should_fail: false }, true);

        // Empty batch should return Ok, not Err
        assert!(result.is_ok());

        let results = result.unwrap();
        assert_eq!(results.total_files(), 0);
        assert_eq!(results.success_count(), 0);
        assert_eq!(results.failure_count(), 0);
    }

    #[test]
    fn test_batch_processor_serial_success() {
        let processor = BatchExecutor::new(BatchConfig {
            parallel_threshold: 100, // Force serial for small batch
            ..Default::default()
        });

        let files = vec![
            PathBuf::from("a.hedl"),
            PathBuf::from("b.hedl"),
            PathBuf::from("c.hedl"),
        ];

        let results = processor
            .process(&files, MockOperation { should_fail: false }, false)
            .unwrap();

        assert_eq!(results.total_files(), 3);
        assert_eq!(results.success_count(), 3);
        assert_eq!(results.failure_count(), 0);
        assert!(results.all_succeeded());
    }

    #[test]
    fn test_batch_processor_serial_with_failures() {
        let processor = BatchExecutor::new(BatchConfig {
            parallel_threshold: 100,
            ..Default::default()
        });

        let files = vec![PathBuf::from("a.hedl"), PathBuf::from("b.hedl")];

        let results = processor
            .process(&files, MockOperation { should_fail: true }, false)
            .unwrap();

        assert_eq!(results.total_files(), 2);
        assert_eq!(results.success_count(), 0);
        assert_eq!(results.failure_count(), 2);
        assert!(!results.all_succeeded());
        assert!(results.has_failures());
    }

    #[test]
    fn test_batch_processor_parallel() {
        let processor = BatchExecutor::new(BatchConfig {
            parallel_threshold: 2, // Force parallel
            ..Default::default()
        });

        let files: Vec<PathBuf> = (0..20)
            .map(|i| PathBuf::from(format!("file{i}.hedl")))
            .collect();

        let results = processor
            .process(&files, MockOperation { should_fail: false }, false)
            .unwrap();

        assert_eq!(results.total_files(), 20);
        assert_eq!(results.success_count(), 20);
    }

    #[test]
    fn test_validate_file_count_within_limit() {
        assert!(validate_file_count(100, Some(1000)).is_ok());
    }

    #[test]
    fn test_validate_file_count_at_limit() {
        assert!(validate_file_count(1000, Some(1000)).is_ok());
    }

    #[test]
    fn test_validate_file_count_exceeds_limit() {
        let result = validate_file_count(2000, Some(1000));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("exceeds maximum limit"));
    }

    #[test]
    fn test_validate_file_count_unlimited() {
        // None = unlimited
        assert!(validate_file_count(1_000_000, None).is_ok());
    }

    #[test]
    fn test_validate_file_count_zero_files() {
        // Zero files always OK regardless of limit
        assert!(validate_file_count(0, Some(100)).is_ok());
    }

    #[test]
    #[serial]
    fn test_get_max_batch_files_default() {
        std::env::remove_var("HEDL_MAX_BATCH_FILES");
        let max = get_max_batch_files();
        assert_eq!(max, 10_000);
    }

    #[test]
    #[serial]
    fn test_get_max_batch_files_env_override() {
        std::env::set_var("HEDL_MAX_BATCH_FILES", "50000");
        let max = get_max_batch_files();
        assert_eq!(max, 50_000);
        std::env::remove_var("HEDL_MAX_BATCH_FILES");
    }

    #[test]
    #[serial]
    fn test_get_max_batch_files_invalid_env() {
        std::env::set_var("HEDL_MAX_BATCH_FILES", "invalid");
        let max = get_max_batch_files();
        assert_eq!(max, 10_000); // Falls back to default
        std::env::remove_var("HEDL_MAX_BATCH_FILES");
    }

    #[test]
    #[serial]
    fn test_batch_config_default_has_limit() {
        std::env::remove_var("HEDL_MAX_BATCH_FILES");
        let config = BatchConfig::default();
        assert!(config.max_files.is_some());
        assert_eq!(config.max_files.unwrap(), 10_000);
    }

    #[test]
    fn test_warn_large_batch_above_threshold() {
        // Note: This test just verifies no panic, can't easily test stderr output
        warn_large_batch(5000, false);
    }

    #[test]
    fn test_warn_large_batch_below_threshold() {
        warn_large_batch(500, false);
    }

    #[test]
    fn test_warn_large_batch_verbose_suppresses() {
        warn_large_batch(5000, true);
    }

    // ============================================================================
    // Thread Pool Tests
    // ============================================================================

    #[test]
    fn test_local_thread_pool_creation() {
        let processor = BatchExecutor::new(BatchConfig {
            max_threads: Some(2),
            parallel_threshold: 1, // Force parallel even with 2 files
            ..Default::default()
        });

        let files = vec![PathBuf::from("test1.hedl"), PathBuf::from("test2.hedl")];

        let results = processor.process(&files, MockOperation { should_fail: false }, false);
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.total_files(), 2);
        assert_eq!(results.success_count(), 2);
        assert_eq!(results.failure_count(), 0);
    }

    #[test]
    fn test_invalid_thread_count() {
        let processor = BatchExecutor::new(BatchConfig {
            max_threads: Some(0), // Invalid: zero threads
            parallel_threshold: 1,
            ..Default::default()
        });

        let files = vec![PathBuf::from("test.hedl")];
        let results = processor.process(&files, MockOperation { should_fail: false }, false);

        assert!(results.is_err());
        match results {
            Err(CliError::ThreadPoolError {
                requested_threads, ..
            }) => {
                assert_eq!(requested_threads, 0);
            }
            _ => panic!("Expected ThreadPoolError, got: {results:?}"),
        }
    }

    #[test]
    fn test_concurrent_batch_operations_different_pools() {
        use std::sync::Arc;
        use std::thread;

        let files = vec![PathBuf::from("test1.hedl"), PathBuf::from("test2.hedl")];

        // Run two batch operations concurrently with different thread counts
        let processor1 = Arc::new(BatchExecutor::new(BatchConfig {
            max_threads: Some(2),
            parallel_threshold: 1,
            ..Default::default()
        }));

        let processor2 = Arc::new(BatchExecutor::new(BatchConfig {
            max_threads: Some(4),
            parallel_threshold: 1,
            ..Default::default()
        }));

        let files1 = files.clone();
        let p1 = processor1.clone();
        let handle1 =
            thread::spawn(move || p1.process(&files1, MockOperation { should_fail: false }, false));

        let files2 = files.clone();
        let p2 = processor2.clone();
        let handle2 =
            thread::spawn(move || p2.process(&files2, MockOperation { should_fail: false }, false));

        // Both should succeed with their respective configurations
        let result1 = handle1.join().unwrap();
        let result2 = handle2.join().unwrap();

        assert!(result1.is_ok(), "First processor should succeed");
        assert!(result2.is_ok(), "Second processor should succeed");

        let results1 = result1.unwrap();
        let results2 = result2.unwrap();

        assert_eq!(results1.total_files(), 2);
        assert_eq!(results1.success_count(), 2);
        assert_eq!(results2.total_files(), 2);
        assert_eq!(results2.success_count(), 2);
    }

    #[test]
    fn test_default_config_uses_global_pool() {
        // Verify that default config (no max_threads) doesn't create local pool
        let processor = BatchExecutor::default_config();

        let files = vec![
            PathBuf::from("test1.hedl"),
            PathBuf::from("test2.hedl"),
            PathBuf::from("test3.hedl"),
            PathBuf::from("test4.hedl"),
            PathBuf::from("test5.hedl"),
            PathBuf::from("test6.hedl"),
            PathBuf::from("test7.hedl"),
            PathBuf::from("test8.hedl"),
            PathBuf::from("test9.hedl"),
            PathBuf::from("test10.hedl"),
        ];

        let results = processor.process(&files, MockOperation { should_fail: false }, false);
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.total_files(), 10);
        assert_eq!(results.success_count(), 10);
        // This should use global pool, not create a local one
    }

    #[test]
    fn test_local_pool_with_failures() {
        // Verify that local thread pool works correctly even when operations fail
        let processor = BatchExecutor::new(BatchConfig {
            max_threads: Some(3),
            parallel_threshold: 1,
            ..Default::default()
        });

        let files = vec![
            PathBuf::from("test1.hedl"),
            PathBuf::from("test2.hedl"),
            PathBuf::from("test3.hedl"),
        ];

        let results = processor.process(&files, MockOperation { should_fail: true }, false);
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.total_files(), 3);
        assert_eq!(results.success_count(), 0);
        assert_eq!(results.failure_count(), 3);
    }

    #[test]
    fn test_serial_processing_ignores_max_threads() {
        // When file count is below parallel_threshold, max_threads should be ignored
        let processor = BatchExecutor::new(BatchConfig {
            max_threads: Some(8),
            parallel_threshold: 100, // High threshold forces serial
            ..Default::default()
        });

        let files = vec![PathBuf::from("test1.hedl"), PathBuf::from("test2.hedl")];

        let results = processor.process(&files, MockOperation { should_fail: false }, false);
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.total_files(), 2);
        assert_eq!(results.success_count(), 2);
    }

    #[test]
    fn test_local_pool_single_thread() {
        // Test that a local pool with just 1 thread works correctly
        let processor = BatchExecutor::new(BatchConfig {
            max_threads: Some(1),
            parallel_threshold: 1,
            ..Default::default()
        });

        let files = vec![
            PathBuf::from("test1.hedl"),
            PathBuf::from("test2.hedl"),
            PathBuf::from("test3.hedl"),
        ];

        let results = processor.process(&files, MockOperation { should_fail: false }, false);
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.total_files(), 3);
        assert_eq!(results.success_count(), 3);
    }

    #[test]
    fn test_local_pool_many_threads() {
        // Test that a local pool with many threads works correctly
        let processor = BatchExecutor::new(BatchConfig {
            max_threads: Some(16),
            parallel_threshold: 1,
            ..Default::default()
        });

        let files: Vec<PathBuf> = (0..32)
            .map(|i| PathBuf::from(format!("file{i}.hedl")))
            .collect();

        let results = processor.process(&files, MockOperation { should_fail: false }, false);
        assert!(results.is_ok());

        let results = results.unwrap();
        assert_eq!(results.total_files(), 32);
        assert_eq!(results.success_count(), 32);
    }

    #[test]
    fn test_thread_pool_error_message() {
        let processor = BatchExecutor::new(BatchConfig {
            max_threads: Some(0),
            parallel_threshold: 1,
            ..Default::default()
        });

        let files = vec![PathBuf::from("test.hedl")];
        let result = processor.process(&files, MockOperation { should_fail: false }, false);

        match result {
            Err(CliError::ThreadPoolError {
                message,
                requested_threads,
            }) => {
                assert_eq!(requested_threads, 0);
                assert!(message.contains("0 threads"), "Message: {message}");
            }
            _ => panic!("Expected ThreadPoolError"),
        }
    }
}
