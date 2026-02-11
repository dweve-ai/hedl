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

//! Batch executor implementation.

use super::config::BatchConfig;
use super::results::{BatchResults, FileResult};
use super::traits::{BatchOperation, StreamingBatchOperation};
use crate::error::CliError;
use colored::Colorize;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Progress tracker for batch operations.
///
/// Uses atomic counters for lock-free concurrent progress tracking.
#[derive(Debug)]
struct ProgressTracker {
    total: usize,
    processed: AtomicUsize,
    succeeded: AtomicUsize,
    failed: AtomicUsize,
    interval: usize,
    verbose: bool,
    start_time: Instant,
}

impl ProgressTracker {
    /// Create a new progress tracker.
    fn new(total: usize, interval: usize, verbose: bool) -> Self {
        Self {
            total,
            processed: AtomicUsize::new(0),
            succeeded: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
            interval,
            verbose,
            start_time: Instant::now(),
        }
    }

    /// Record a successful file processing.
    fn record_success(&self, path: &Path) {
        let processed = self.processed.fetch_add(1, Ordering::Relaxed) + 1;
        self.succeeded.fetch_add(1, Ordering::Relaxed);

        if self.should_report(processed) {
            self.report_progress(path, true);
        }
    }

    /// Record a failed file processing.
    fn record_failure(&self, path: &Path, error: &CliError) {
        let processed = self.processed.fetch_add(1, Ordering::Relaxed) + 1;
        self.failed.fetch_add(1, Ordering::Relaxed);

        if self.verbose {
            eprintln!("{} {} - {}", "✗".red().bold(), path.display(), error);
        }

        if self.should_report(processed) {
            self.report_progress(path, false);
        }
    }

    /// Check if progress should be reported for this count.
    fn should_report(&self, processed: usize) -> bool {
        self.interval > 0 && (processed % self.interval == 0 || processed == self.total)
    }

    /// Report current progress to stderr.
    fn report_progress(&self, current_file: &Path, success: bool) {
        let processed = self.processed.load(Ordering::Relaxed);
        let succeeded = self.succeeded.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();
        let rate = processed as f64 / elapsed.as_secs_f64();

        if self.verbose {
            let status = if success {
                "✓".green().bold()
            } else {
                "✗".red().bold()
            };
            eprintln!(
                "{} [{}/{}] {} ({:.1} files/s)",
                status,
                processed,
                self.total,
                current_file.display(),
                rate
            );
        } else {
            eprintln!(
                "Progress: [{}/{}] {} succeeded, {} failed ({:.1} files/s)",
                processed, self.total, succeeded, failed, rate
            );
        }
    }

    /// Print final summary.
    fn print_summary(&self, operation_name: &str) {
        let processed = self.processed.load(Ordering::Relaxed);
        let succeeded = self.succeeded.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();

        println!();
        println!("{}", "═".repeat(60).bright_blue());
        println!(
            "{} {}",
            "Batch Operation:".bright_blue().bold(),
            operation_name.bright_white()
        );
        println!("{}", "═".repeat(60).bright_blue());
        println!(
            "  {} {}",
            "Total files:".bright_cyan(),
            processed.to_string().bright_white()
        );
        println!(
            "  {} {}",
            "Succeeded:".green().bold(),
            succeeded.to_string().bright_white()
        );
        println!(
            "  {} {}",
            "Failed:".red().bold(),
            failed.to_string().bright_white()
        );
        println!(
            "  {} {:.2}s",
            "Elapsed:".bright_cyan(),
            elapsed.as_secs_f64()
        );
        println!(
            "  {} {:.1} files/s",
            "Throughput:".bright_cyan(),
            processed as f64 / elapsed.as_secs_f64()
        );
        println!("{}", "═".repeat(60).bright_blue());
    }
}

/// High-performance batch processor for HEDL files.
///
/// Orchestrates parallel or serial processing based on configuration and workload.
/// Provides progress tracking and comprehensive error collection.
///
/// # Thread Safety
///
/// `BatchExecutor` is thread-safe and can be shared across threads via Arc.
///
/// # Examples
///
/// ```rust,no_run
/// use hedl_cli::batch::{BatchExecutor, BatchConfig, ValidationOperation};
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let processor = BatchExecutor::new(BatchConfig {
///     parallel_threshold: 5,
///     verbose: true,
///     ..Default::default()
/// });
///
/// let files: Vec<PathBuf> = vec![
///     "file1.hedl".into(),
///     "file2.hedl".into(),
/// ];
///
/// let results = processor.process(
///     &files,
///     ValidationOperation { strict: false },
///     true,
/// )?;
///
/// if results.has_failures() {
///     eprintln!("Some files failed validation");
///     for failure in results.failures() {
///         eprintln!("  - {}: {:?}", failure.path.display(), failure.result);
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct BatchExecutor {
    config: BatchConfig,
}

impl BatchExecutor {
    /// Create a new batch processor with the given configuration.
    #[must_use]
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }

    /// Create a batch processor with default configuration.
    #[must_use]
    pub fn default_config() -> Self {
        Self::new(BatchConfig::default())
    }

    /// Process multiple files with the given operation.
    ///
    /// Automatically selects parallel or serial processing based on configuration
    /// and file count. Provides progress reporting and collects all results.
    ///
    /// # Arguments
    ///
    /// * `files` - Slice of file paths to process
    /// * `operation` - The operation to perform on each file
    /// * `show_progress` - Whether to show progress updates
    ///
    /// # Returns
    ///
    /// * `Ok(BatchResults)` - Successfully processed all files (individual failures collected in results)
    /// * `Err(CliError::ThreadPoolError)` - Failed to create thread pool with requested configuration
    ///
    /// # Thread Pool Selection
    ///
    /// The method uses different thread pool strategies based on configuration:
    ///
    /// 1. **Serial Processing**: If `files.len() < parallel_threshold`, processes serially (no thread pool)
    /// 2. **Local Thread Pool**: If `max_threads` is `Some(n)`, creates isolated pool with `n` threads
    /// 3. **Global Thread Pool**: If `max_threads` is `None`, uses Rayon's global pool
    ///
    /// # Error Handling
    ///
    /// Thread pool creation can fail if:
    /// - `max_threads` is 0 (invalid configuration)
    /// - System cannot allocate thread resources
    /// - Thread stack allocation fails
    ///
    /// Individual file processing errors are collected in `BatchResults`, not returned as errors.
    ///
    /// # Performance
    ///
    /// - Serial processing for small batches to avoid thread pool overhead
    /// - Local thread pool: ~0.5-1ms creation overhead, ~2-8MB per thread
    /// - Global thread pool: zero overhead
    /// - Lock-free progress tracking using atomic counters
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::batch::{BatchExecutor, BatchConfig, FormatOperation};
    /// use hedl_cli::error::CliError;
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let processor = BatchExecutor::new(BatchConfig {
    ///     max_threads: Some(4),
    ///     ..Default::default()
    /// });
    ///
    /// let files = vec![PathBuf::from("a.hedl"), PathBuf::from("b.hedl")];
    ///
    /// match processor.process(
    ///     &files,
    ///     FormatOperation {
    ///         check: false,
    ///         ditto: true,
    ///         with_counts: false,
    ///     },
    ///     true,
    /// ) {
    ///     Ok(results) => {
    ///         println!("Formatted {} files", results.success_count());
    ///         if results.has_failures() {
    ///             // Handle individual file failures
    ///         }
    ///     }
    ///     Err(CliError::ThreadPoolError { message, requested_threads }) => {
    ///         eprintln!("Failed to create thread pool: {}", message);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Unexpected error: {}", e);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn process<O>(
        &self,
        files: &[PathBuf],
        operation: O,
        show_progress: bool,
    ) -> Result<BatchResults<O::Output>, CliError>
    where
        O: BatchOperation,
    {
        let start_time = Instant::now();

        if files.is_empty() {
            // Warn the user when no files match the provided patterns.
            // Only show warning when progress is enabled, as this indicates
            // the user expects feedback. Silent mode implies automated/scripted usage.
            if show_progress {
                eprintln!(
                    "{} No files matched the provided patterns",
                    "Warning:".yellow().bold()
                );
                eprintln!(
                    "{} Check that patterns are correct and files exist",
                    "Hint:".cyan()
                );
            }
            return Ok(BatchResults::new(vec![], 0));
        }

        let results = if files.len() < self.config.parallel_threshold {
            // Serial processing for small batches (no thread pool needed)
            self.process_serial(files, &operation, show_progress)
        } else if let Some(max_threads) = self.config.max_threads {
            // Use local thread pool with specified thread count
            self.process_with_local_pool(files, &operation, show_progress, max_threads)?
        } else {
            // Use default global thread pool (Rayon's default)
            self.process_parallel(files, &operation, show_progress)
        };

        let elapsed_ms = start_time.elapsed().as_millis();

        Ok(BatchResults::new(results, elapsed_ms))
    }

    /// Process files using a local thread pool with specified thread count.
    ///
    /// Creates an isolated Rayon thread pool that doesn't affect global state.
    /// The thread pool is created for this operation and destroyed when complete.
    ///
    /// # Arguments
    ///
    /// * `files` - Slice of file paths to process
    /// * `operation` - The operation to perform on each file
    /// * `show_progress` - Whether to show progress updates
    /// * `num_threads` - The number of threads to use
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<FileResult>)` - Successfully processed files with local pool
    /// * `Err(CliError::ThreadPoolError)` - Failed to create thread pool
    ///
    /// # Errors
    ///
    /// Returns `ThreadPoolError` if:
    /// - `num_threads` is 0 (invalid configuration)
    /// - System cannot allocate thread resources
    /// - Thread stack allocation fails
    ///
    /// # Performance
    ///
    /// - Thread pool creation: ~0.5-1ms overhead
    /// - Memory cost: ~2-8MB per thread (OS thread stacks)
    /// - Pool lifetime: Duration of this method call
    fn process_with_local_pool<O>(
        &self,
        files: &[PathBuf],
        operation: &O,
        show_progress: bool,
        num_threads: usize,
    ) -> Result<Vec<FileResult<O::Output>>, CliError>
    where
        O: BatchOperation,
    {
        // Validate thread count - 0 threads is invalid
        if num_threads == 0 {
            return Err(CliError::thread_pool_error(
                "Cannot create thread pool with 0 threads".to_string(),
                num_threads,
            ));
        }

        // Build a local thread pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .map_err(|e| {
                CliError::thread_pool_error(
                    format!("Failed to create thread pool with {num_threads} threads: {e}"),
                    num_threads,
                )
            })?;

        // Run parallel processing within the local pool
        let results = pool.install(|| self.process_parallel(files, operation, show_progress));

        Ok(results)
    }

    /// Process files serially (single-threaded).
    fn process_serial<O>(
        &self,
        files: &[PathBuf],
        operation: &O,
        show_progress: bool,
    ) -> Vec<FileResult<O::Output>>
    where
        O: BatchOperation,
    {
        let tracker = if show_progress {
            Some(ProgressTracker::new(
                files.len(),
                self.config.progress_interval,
                self.config.verbose,
            ))
        } else {
            None
        };

        let results: Vec<FileResult<O::Output>> = files
            .iter()
            .map(|path| {
                let result = operation.process_file(path);

                if let Some(ref t) = tracker {
                    match &result {
                        Ok(_) => t.record_success(path),
                        Err(e) => t.record_failure(path, e),
                    }
                }

                FileResult {
                    path: path.clone(),
                    result: result.map_err(|e| e.clone()),
                }
            })
            .collect();

        if show_progress {
            if let Some(tracker) = tracker {
                tracker.print_summary(operation.name());
            }
        }

        results
    }

    /// Process files in parallel using Rayon.
    fn process_parallel<O>(
        &self,
        files: &[PathBuf],
        operation: &O,
        show_progress: bool,
    ) -> Vec<FileResult<O::Output>>
    where
        O: BatchOperation,
    {
        let tracker = if show_progress {
            Some(Arc::new(ProgressTracker::new(
                files.len(),
                self.config.progress_interval,
                self.config.verbose,
            )))
        } else {
            None
        };

        let results: Vec<FileResult<O::Output>> = files
            .par_iter()
            .map(|path| {
                let result = operation.process_file(path);

                if let Some(ref t) = tracker {
                    match &result {
                        Ok(_) => t.record_success(path),
                        Err(e) => t.record_failure(path, e),
                    }
                }

                FileResult {
                    path: path.clone(),
                    result: result.map_err(|e| e.clone()),
                }
            })
            .collect();

        if show_progress {
            if let Some(tracker) = tracker {
                tracker.print_summary(operation.name());
            }
        }

        results
    }

    /// Process files using streaming operations for memory efficiency.
    ///
    /// This method uses the streaming parser from `hedl-stream` to process files
    /// with constant memory usage regardless of file size. Ideal for:
    /// - Files larger than 100MB
    /// - Memory-constrained environments
    /// - Processing thousands of files
    ///
    /// # Arguments
    ///
    /// * `files` - Slice of file paths to process
    /// * `operation` - The streaming operation to perform
    /// * `show_progress` - Whether to show progress updates
    ///
    /// # Returns
    ///
    /// * `Ok(BatchResults)` - Always succeeds and collects all individual results
    /// * `Err(CliError)` - Only on catastrophic failures
    ///
    /// # Memory Usage
    ///
    /// Peak memory = `buffer_size` (8KB) × `num_threads` + ID tracking set
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::batch::{BatchExecutor, StreamingValidationOperation, BatchConfig};
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let processor = BatchExecutor::default_config();
    /// let files = vec![PathBuf::from("large-file.hedl")];
    /// let operation = StreamingValidationOperation { strict: false };
    ///
    /// let results = processor.process_streaming(&files, operation, true)?;
    /// println!("Processed {} files with constant memory", results.success_count());
    /// # Ok(())
    /// # }
    /// ```
    pub fn process_streaming<O>(
        &self,
        files: &[PathBuf],
        operation: O,
        show_progress: bool,
    ) -> Result<BatchResults<O::Output>, CliError>
    where
        O: StreamingBatchOperation,
    {
        let start_time = Instant::now();

        if files.is_empty() {
            return Ok(BatchResults::new(vec![], 0));
        }

        // Configure thread pool if max_threads is specified
        if let Some(max_threads) = self.config.max_threads {
            rayon::ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build_global()
                .ok(); // Ignore error if already initialized
        }

        let results = if files.len() < self.config.parallel_threshold {
            self.process_streaming_serial(files, &operation, show_progress)
        } else {
            self.process_streaming_parallel(files, &operation, show_progress)
        };

        let elapsed_ms = start_time.elapsed().as_millis();
        Ok(BatchResults::new(results, elapsed_ms))
    }

    /// Process files serially using streaming.
    fn process_streaming_serial<O>(
        &self,
        files: &[PathBuf],
        operation: &O,
        show_progress: bool,
    ) -> Vec<FileResult<O::Output>>
    where
        O: StreamingBatchOperation,
    {
        let tracker = if show_progress {
            Some(ProgressTracker::new(
                files.len(),
                self.config.progress_interval,
                self.config.verbose,
            ))
        } else {
            None
        };

        let results: Vec<FileResult<O::Output>> = files
            .iter()
            .map(|path| {
                let result = operation.process_file_streaming(path);

                if let Some(ref t) = tracker {
                    match &result {
                        Ok(_) => t.record_success(path),
                        Err(e) => t.record_failure(path, e),
                    }
                }

                FileResult {
                    path: path.clone(),
                    result: result.map_err(|e| e.clone()),
                }
            })
            .collect();

        if show_progress {
            if let Some(tracker) = tracker {
                tracker.print_summary(operation.name());
            }
        }

        results
    }

    /// Process files in parallel using streaming.
    fn process_streaming_parallel<O>(
        &self,
        files: &[PathBuf],
        operation: &O,
        show_progress: bool,
    ) -> Vec<FileResult<O::Output>>
    where
        O: StreamingBatchOperation,
    {
        let tracker = if show_progress {
            Some(Arc::new(ProgressTracker::new(
                files.len(),
                self.config.progress_interval,
                self.config.verbose,
            )))
        } else {
            None
        };

        let results: Vec<FileResult<O::Output>> = files
            .par_iter()
            .map(|path| {
                let result = operation.process_file_streaming(path);

                if let Some(ref t) = tracker {
                    match &result {
                        Ok(_) => t.record_success(path),
                        Err(e) => t.record_failure(path, e),
                    }
                }

                FileResult {
                    path: path.clone(),
                    result: result.map_err(|e| e.clone()),
                }
            })
            .collect();

        if show_progress {
            if let Some(tracker) = tracker {
                tracker.print_summary(operation.name());
            }
        }

        results
    }

    /// Automatically choose between standard and streaming based on file size.
    ///
    /// Files larger than 100MB use streaming mode for memory efficiency,
    /// while smaller files use standard mode for better performance.
    ///
    /// # Arguments
    ///
    /// * `files` - Slice of file paths to process
    /// * `standard_op` - Standard operation for small files
    /// * `streaming_op` - Streaming operation for large files
    /// * `show_progress` - Whether to show progress updates
    ///
    /// # Returns
    ///
    /// * `Ok(BatchResults)` - Combined results from both modes
    /// * `Err(CliError)` - On catastrophic failures
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::batch::{BatchExecutor, ValidationOperation, StreamingValidationOperation};
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let processor = BatchExecutor::default_config();
    /// let files = vec![
    ///     PathBuf::from("small.hedl"),
    ///     PathBuf::from("large-200mb.hedl"),
    /// ];
    ///
    /// let results = processor.process_auto(
    ///     &files,
    ///     ValidationOperation { strict: false },
    ///     StreamingValidationOperation { strict: false },
    ///     true,
    /// )?;
    /// println!("Processed {} files", results.results.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn process_auto<O, SO>(
        &self,
        files: &[PathBuf],
        standard_op: O,
        streaming_op: SO,
        show_progress: bool,
    ) -> Result<BatchResults<O::Output>, CliError>
    where
        O: BatchOperation<Output = SO::Output>,
        SO: StreamingBatchOperation,
    {
        const STREAMING_THRESHOLD: u64 = 100 * 1024 * 1024; // 100MB

        if files.is_empty() {
            return Ok(BatchResults::new(vec![], 0));
        }

        let start_time = Instant::now();

        // Partition files by size
        let mut small_files = Vec::new();
        let mut large_files = Vec::new();

        for path in files {
            match std::fs::metadata(path) {
                Ok(meta) if meta.len() > STREAMING_THRESHOLD => {
                    large_files.push(path.clone());
                }
                Ok(_) => {
                    small_files.push(path.clone());
                }
                Err(_) => {
                    // If we can't get size, treat as small
                    small_files.push(path.clone());
                }
            }
        }

        // Process small files with standard ops
        let mut all_results = if small_files.is_empty() {
            Vec::new()
        } else {
            self.process(&small_files, standard_op, show_progress)?
                .results
        };

        // Process large files with streaming ops
        if !large_files.is_empty() {
            let streaming_results = self
                .process_streaming(&large_files, streaming_op, show_progress)?
                .results;
            all_results.extend(streaming_results);
        }

        // Restore original order
        let file_order: Vec<&PathBuf> = files.iter().collect();
        all_results.sort_by_key(|r| {
            file_order
                .iter()
                .position(|&p| p == &r.path)
                .unwrap_or(usize::MAX)
        });

        let elapsed_ms = start_time.elapsed().as_millis();
        Ok(BatchResults::new(all_results, elapsed_ms))
    }
}
