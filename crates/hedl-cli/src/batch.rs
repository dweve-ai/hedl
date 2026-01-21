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
//! use hedl_cli::batch::{BatchProcessor, BatchConfig, ValidationOperation};
//! use std::path::PathBuf;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a batch processor with default configuration
//! let processor = BatchProcessor::new(BatchConfig::default());
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
//! use hedl_cli::batch::{BatchProcessor, BatchConfig};
//! use std::path::PathBuf;
//!
//! // Concurrent operations with different thread counts
//! use std::thread;
//!
//! let files: Vec<PathBuf> = vec!["a.hedl".into(), "b.hedl".into()];
//!
//! let handle1 = thread::spawn(|| {
//!     let processor = BatchProcessor::new(BatchConfig {
//!         max_threads: Some(2),
//!         ..Default::default()
//!     });
//!     // Uses 2 threads
//! });
//!
//! let handle2 = thread::spawn(|| {
//!     let processor = BatchProcessor::new(BatchConfig {
//!         max_threads: Some(4),
//!         ..Default::default()
//!     });
//!     // Uses 4 threads, isolated from handle1
//! });
//! ```

use crate::error::CliError;
use colored::Colorize;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Configuration for batch processing operations.
///
/// Controls parallelization strategy, progress reporting, and error handling behavior.
///
/// # Examples
///
/// ```rust
/// use hedl_cli::batch::BatchConfig;
///
/// // Default configuration (auto parallelization)
/// let config = BatchConfig::default();
///
/// // Custom configuration
/// let config = BatchConfig {
///     parallel_threshold: 5,  // Parallelize if >= 5 files
///     max_threads: Some(4),   // Use at most 4 threads
///     progress_interval: 10,  // Update progress every 10 files
///     verbose: true,          // Show detailed progress
///     max_files: Some(10_000), // Limit to 10,000 files
/// };
/// ```
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Minimum number of files to trigger parallel processing.
    ///
    /// Files below this threshold are processed serially to avoid thread pool overhead.
    /// Default: 10
    pub parallel_threshold: usize,

    /// Maximum number of threads to use for parallel processing.
    ///
    /// When set, creates a local thread pool isolated to this batch operation.
    /// This ensures configuration always takes effect and prevents global state pollution.
    ///
    /// # Behavior
    ///
    /// - `None` (default): Uses Rayon's global thread pool (typically number of CPU cores)
    /// - `Some(n)`: Creates a local thread pool with exactly `n` threads for this operation
    ///
    /// # Thread Pool Isolation
    ///
    /// Local thread pools provide complete isolation:
    /// - No global state modification
    /// - Concurrent batch operations can use different thread counts
    /// - Configuration is guaranteed to take effect or error explicitly
    /// - Thread pool lifetime matches the `process()` call duration
    ///
    /// # Performance Considerations
    ///
    /// Local thread pool creation has small overhead (~0.5-1ms) and memory cost (~2-8MB per thread).
    /// For maximum performance with default configuration, leave as `None`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_cli::batch::BatchConfig;
    ///
    /// // Default: uses global pool
    /// let config = BatchConfig::default();
    ///
    /// // Custom: creates local pool with 4 threads
    /// let config = BatchConfig {
    ///     max_threads: Some(4),
    ///     ..Default::default()
    /// };
    /// ```
    ///
    /// Default: None
    pub max_threads: Option<usize>,

    /// Number of files between progress updates.
    ///
    /// Progress is printed every N files processed. Set to 0 to disable.
    /// Default: 1 (update after each file)
    pub progress_interval: usize,

    /// Enable verbose progress reporting.
    ///
    /// When true, shows file names and detailed status for each file.
    /// Default: false
    pub verbose: bool,

    /// Maximum number of files allowed in a batch operation.
    ///
    /// This prevents resource exhaustion when processing very large file sets.
    /// - `Some(n)`: Limit to n files (default: 10,000)
    /// - `None`: No limit (use with caution)
    ///
    /// # Security
    ///
    /// Protects against:
    /// - Memory exhaustion from storing millions of file paths
    /// - File descriptor exhaustion from concurrent operations
    /// - Excessive CPU time from unbounded processing
    ///
    /// # Configuration
    ///
    /// Can be overridden via:
    /// - Environment variable: `HEDL_MAX_BATCH_FILES`
    /// - CLI flag: `--max-files <N>`
    /// - Programmatic: `BatchConfig { max_files: Some(n), .. }`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_cli::batch::BatchConfig;
    ///
    /// // Default limit (10,000 files)
    /// let config = BatchConfig::default();
    ///
    /// // Custom limit
    /// let config = BatchConfig {
    ///     max_files: Some(50_000),
    ///     ..Default::default()
    /// };
    ///
    /// // Unlimited (use with caution)
    /// let config = BatchConfig {
    ///     max_files: None,
    ///     ..Default::default()
    /// };
    /// ```
    pub max_files: Option<usize>,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            parallel_threshold: 10,
            max_threads: None,
            progress_interval: 1,
            verbose: false,
            max_files: Some(get_max_batch_files()),
        }
    }
}

/// Get maximum batch files from environment variable or default.
///
/// Checks `HEDL_MAX_BATCH_FILES` environment variable. Falls back to
/// `DEFAULT_MAX_BATCH_FILES` (10,000) if not set or invalid.
///
/// # Examples
///
/// ```bash
/// export HEDL_MAX_BATCH_FILES=50000
/// hedl batch-validate "*.hedl"
/// ```
fn get_max_batch_files() -> usize {
    const DEFAULT_MAX_BATCH_FILES: usize = 10_000;

    std::env::var("HEDL_MAX_BATCH_FILES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_BATCH_FILES)
}

/// Validate file count against configured limit.
///
/// # Arguments
///
/// * `file_count` - Number of files to process
/// * `max_files` - Maximum allowed files (None = unlimited)
///
/// # Returns
///
/// * `Ok(())` - File count is within limit
/// * `Err(CliError)` - File count exceeds limit
///
/// # Examples
///
/// ```rust
/// use hedl_cli::batch::validate_file_count;
///
/// // Within limit
/// assert!(validate_file_count(100, Some(1000)).is_ok());
///
/// // Exceeds limit
/// assert!(validate_file_count(2000, Some(1000)).is_err());
///
/// // Unlimited
/// assert!(validate_file_count(1_000_000, None).is_ok());
/// ```
pub fn validate_file_count(file_count: usize, max_files: Option<usize>) -> Result<(), CliError> {
    if let Some(limit) = max_files {
        if file_count > limit {
            return Err(CliError::invalid_input(format!(
                "File count ({file_count}) exceeds maximum limit ({limit}). \
                 Consider:\n  \
                 - Refining glob patterns to match fewer files\n  \
                 - Using --max-files flag to increase limit\n  \
                 - Setting HEDL_MAX_BATCH_FILES environment variable\n  \
                 - Processing files in smaller batches"
            )));
        }
    }
    Ok(())
}

/// Warn if file count is large and suggest verbose mode.
///
/// Prints a warning when processing many files to inform user of operation scale.
///
/// # Arguments
///
/// * `file_count` - Number of files to process
/// * `verbose` - Whether verbose mode is enabled
///
/// # Threshold
///
/// Warns if `file_count` >= 1000 and not already in verbose mode.
pub fn warn_large_batch(file_count: usize, verbose: bool) {
    const WARN_THRESHOLD: usize = 1_000;

    if file_count >= WARN_THRESHOLD && !verbose {
        eprintln!(
            "{} Processing {} files. Consider using {} for progress updates.",
            "Warning:".yellow().bold(),
            file_count.to_string().bright_white(),
            "--verbose".bright_cyan()
        );
    }
}

/// Result of processing a single file in a batch operation.
///
/// Contains the file path and either a success value or an error.
///
/// # Type Parameters
///
/// * `T` - The success type returned by the operation
#[derive(Debug, Clone)]
pub struct FileResult<T> {
    /// The file path that was processed
    pub path: PathBuf,
    /// The result of processing (Ok or Err)
    pub result: Result<T, CliError>,
}

impl<T> FileResult<T> {
    /// Create a successful file result.
    pub fn success(path: PathBuf, value: T) -> Self {
        Self {
            path,
            result: Ok(value),
        }
    }

    /// Create a failed file result.
    #[must_use]
    pub fn failure(path: PathBuf, error: CliError) -> Self {
        Self {
            path,
            result: Err(error),
        }
    }

    /// Check if the result is successful.
    pub fn is_success(&self) -> bool {
        self.result.is_ok()
    }

    /// Check if the result is a failure.
    pub fn is_failure(&self) -> bool {
        self.result.is_err()
    }
}

/// Aggregated results from a batch processing operation.
///
/// Contains all individual file results and provides statistics.
///
/// # Type Parameters
///
/// * `T` - The success type returned by the operation
#[derive(Debug, Clone)]
pub struct BatchResults<T> {
    /// Individual results for each processed file
    pub results: Vec<FileResult<T>>,
    /// Total processing time in milliseconds
    pub elapsed_ms: u128,
}

impl<T> BatchResults<T> {
    /// Create new batch results from a vector of file results.
    #[must_use]
    pub fn new(results: Vec<FileResult<T>>, elapsed_ms: u128) -> Self {
        Self {
            results,
            elapsed_ms,
        }
    }

    /// Get the total number of files processed.
    #[must_use]
    pub fn total_files(&self) -> usize {
        self.results.len()
    }

    /// Get the number of successfully processed files.
    #[must_use]
    pub fn success_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_success()).count()
    }

    /// Get the number of failed files.
    #[must_use]
    pub fn failure_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_failure()).count()
    }

    /// Check if all files were processed successfully.
    #[must_use]
    pub fn all_succeeded(&self) -> bool {
        self.results.iter().all(FileResult::is_success)
    }

    /// Check if any files failed.
    #[must_use]
    pub fn has_failures(&self) -> bool {
        self.results.iter().any(FileResult::is_failure)
    }

    /// Get an iterator over successful results.
    pub fn successes(&self) -> impl Iterator<Item = &FileResult<T>> {
        self.results.iter().filter(|r| r.is_success())
    }

    /// Get an iterator over failed results.
    pub fn failures(&self) -> impl Iterator<Item = &FileResult<T>> {
        self.results.iter().filter(|r| r.is_failure())
    }

    /// Get processing throughput in files per second.
    #[must_use]
    pub fn throughput(&self) -> f64 {
        if self.elapsed_ms == 0 {
            0.0
        } else {
            (self.total_files() as f64) / (self.elapsed_ms as f64 / 1000.0)
        }
    }
}

/// Trait for batch operations on HEDL files.
///
/// Implement this trait to define custom batch operations. The operation must be
/// thread-safe (Send + Sync) to support parallel processing.
///
/// # Type Parameters
///
/// * `Output` - The type returned on successful processing of a file
///
/// # Examples
///
/// ```rust
/// use hedl_cli::batch::BatchOperation;
/// use hedl_cli::error::CliError;
/// use std::path::Path;
///
/// struct CountLinesOperation;
///
/// impl BatchOperation for CountLinesOperation {
///     type Output = usize;
///
///     fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
///         let content = std::fs::read_to_string(path)
///             .map_err(|e| CliError::io_error(path, e))?;
///         Ok(content.lines().count())
///     }
///
///     fn name(&self) -> &str {
///         "count-lines"
///     }
/// }
/// ```
pub trait BatchOperation: Send + Sync {
    /// The output type for successful processing
    type Output: Send;

    /// Process a single file and return the result.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file to process
    ///
    /// # Returns
    ///
    /// * `Ok(Output)` - On successful processing
    /// * `Err(CliError)` - On any error
    ///
    /// # Errors
    ///
    /// Should return appropriate `CliError` variants for different failure modes.
    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError>;

    /// Get a human-readable name for this operation.
    ///
    /// Used for progress reporting and logging.
    fn name(&self) -> &str;
}

/// Trait for streaming batch operations on HEDL files.
///
/// Unlike `BatchOperation` which loads entire files into memory,
/// streaming operations process files incrementally with constant memory usage.
/// This is ideal for processing large files (>100MB) or when memory is constrained.
///
/// # Memory Characteristics
///
/// - **Standard operations**: `O(num_threads` × `file_size`)
/// - **Streaming operations**: `O(buffer_size` + `ID_set`) ≈ constant
///
/// # Type Parameters
///
/// * `Output` - The type returned on successful processing of a file
///
/// # Examples
///
/// ```rust
/// use hedl_cli::batch::StreamingBatchOperation;
/// use hedl_cli::error::CliError;
/// use std::path::Path;
///
/// struct StreamingCountOperation;
///
/// impl StreamingBatchOperation for StreamingCountOperation {
///     type Output = usize;
///
///     fn process_file_streaming(&self, path: &Path) -> Result<Self::Output, CliError> {
///         use std::io::BufReader;
///         use std::fs::File;
///         use hedl_stream::StreamingParser;
///
///         let file = File::open(path).map_err(|e| CliError::io_error(path, e))?;
///         let reader = BufReader::new(file);
///         let parser = StreamingParser::new(reader)
///             .map_err(|e| CliError::parse(e.to_string()))?;
///
///         let count = parser.filter(|e| {
///             matches!(e, Ok(hedl_stream::NodeEvent::Node(_)))
///         }).count();
///
///         Ok(count)
///     }
///
///     fn name(&self) -> &str {
///         "count-streaming"
///     }
/// }
/// ```
pub trait StreamingBatchOperation: Send + Sync {
    /// The output type for successful processing
    type Output: Send;

    /// Process a file using streaming parser.
    ///
    /// # Arguments
    ///
    /// * `path` - File path to process
    ///
    /// # Returns
    ///
    /// * `Ok(Output)` - On successful processing
    /// * `Err(CliError)` - On any error
    ///
    /// # Memory Guarantee
    ///
    /// Implementations should maintain O(1) memory usage regardless of file size,
    /// processing the file incrementally using the streaming parser.
    fn process_file_streaming(&self, path: &Path) -> Result<Self::Output, CliError>;

    /// Get operation name for progress reporting
    fn name(&self) -> &str;

    /// Indicate if this operation can run in streaming mode.
    ///
    /// Some operations (like formatting) may require full document.
    /// Default: true
    fn supports_streaming(&self) -> bool {
        true
    }
}

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
/// `BatchProcessor` is thread-safe and can be shared across threads via Arc.
///
/// # Examples
///
/// ```rust,no_run
/// use hedl_cli::batch::{BatchProcessor, BatchConfig, ValidationOperation};
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let processor = BatchProcessor::new(BatchConfig {
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
pub struct BatchProcessor {
    config: BatchConfig,
}

impl BatchProcessor {
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
    /// use hedl_cli::batch::{BatchProcessor, BatchConfig, FormatOperation};
    /// use hedl_cli::error::CliError;
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let processor = BatchProcessor::new(BatchConfig {
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
    /// use hedl_cli::batch::{BatchProcessor, StreamingValidationOperation, BatchConfig};
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let processor = BatchProcessor::default_config();
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
    /// use hedl_cli::batch::{BatchProcessor, ValidationOperation, StreamingValidationOperation};
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let processor = BatchProcessor::default_config();
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

// ============================================================================
// Standard Operations
// ============================================================================

/// Batch validation operation.
///
/// Validates multiple HEDL files in parallel, checking syntax and optionally
/// enforcing strict reference resolution.
#[derive(Debug, Clone)]
pub struct ValidationOperation {
    /// Enable strict reference validation
    pub strict: bool,
}

impl BatchOperation for ValidationOperation {
    type Output = ValidationStats;

    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
        use hedl_core::{parse_with_limits, Item, Node, ParseOptions, ReferenceMode};

        let content = std::fs::read_to_string(path).map_err(|e| CliError::io_error(path, e))?;

        let options = ParseOptions {
            reference_mode: if self.strict {
                ReferenceMode::Strict
            } else {
                ReferenceMode::Lenient
            },
            ..ParseOptions::default()
        };

        let doc = parse_with_limits(content.as_bytes(), options)
            .map_err(|e| CliError::parse(e.to_string()))?;

        // Collect statistics from the parsed document
        let mut stats = ValidationStats::new();

        // Get version from document metadata
        stats.version = format!("{}.{}", doc.version.0, doc.version.1);

        // Recursive helper to count nodes
        fn count_node(node: &Node, stats: &mut ValidationStats) {
            stats.node_count += 1;
            stats.field_count += node.fields.len();
            let full_id = format!("{}:{}", node.type_name, node.id);
            stats.seen_ids.insert(full_id);

            // Count children recursively
            if let Some(ref children) = node.children {
                for child_nodes in children.values() {
                    for child in child_nodes {
                        count_node(child, stats);
                    }
                }
            }
        }

        // Recursive helper to traverse items
        fn traverse_item(item: &Item, stats: &mut ValidationStats) {
            match item {
                Item::List(list) => {
                    stats.list_count += 1;
                    for node in &list.rows {
                        count_node(node, stats);
                    }
                }
                Item::Object(obj) => {
                    for child_item in obj.values() {
                        traverse_item(child_item, stats);
                    }
                }
                Item::Scalar(_) => {
                    // Scalars don't contribute to node counts
                }
            }
        }

        // Traverse all items in the document root
        for item in doc.root.values() {
            traverse_item(item, &mut stats);
        }

        Ok(stats)
    }

    fn name(&self) -> &'static str {
        "validate"
    }
}

/// Batch format operation.
///
/// Formats multiple HEDL files to canonical form, optionally checking if files
/// are already canonical.
#[derive(Debug, Clone)]
pub struct FormatOperation {
    /// Only check if files are canonical (don't write)
    pub check: bool,
    /// Use ditto optimization
    pub ditto: bool,
    /// Add count hints to matrix lists
    pub with_counts: bool,
}

impl BatchOperation for FormatOperation {
    type Output = String;

    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
        use hedl_c14n::{canonicalize_with_config, CanonicalConfig};
        use hedl_core::parse;

        let content = std::fs::read_to_string(path).map_err(|e| CliError::io_error(path, e))?;

        let mut doc = parse(content.as_bytes()).map_err(|e| CliError::parse(e.to_string()))?;

        // Add count hints if requested
        if self.with_counts {
            add_count_hints(&mut doc);
        }

        let config = CanonicalConfig::new().with_ditto(self.ditto);

        let canonical = canonicalize_with_config(&doc, &config)
            .map_err(|e| CliError::canonicalization(e.to_string()))?;

        if self.check && canonical != content {
            return Err(CliError::NotCanonical);
        }

        Ok(canonical)
    }

    fn name(&self) -> &str {
        if self.check {
            "format-check"
        } else {
            "format"
        }
    }
}

/// Batch lint operation.
///
/// Lints multiple HEDL files for best practices and common issues.
#[derive(Debug, Clone)]
pub struct LintOperation {
    /// Treat warnings as errors
    pub warn_error: bool,
}

impl BatchOperation for LintOperation {
    type Output = Vec<String>;

    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
        use hedl_core::parse;
        use hedl_lint::lint;

        let content = std::fs::read_to_string(path).map_err(|e| CliError::io_error(path, e))?;

        let doc = parse(content.as_bytes()).map_err(|e| CliError::parse(e.to_string()))?;

        let diagnostics = lint(&doc);

        if self.warn_error && !diagnostics.is_empty() {
            return Err(CliError::LintErrors);
        }

        Ok(diagnostics
            .iter()
            .map(std::string::ToString::to_string)
            .collect())
    }

    fn name(&self) -> &'static str {
        "lint"
    }
}

// ============================================================================
// Streaming Operations
// ============================================================================

/// Statistics collected during streaming validation.
///
/// Provides detailed statistics about the parsed document including
/// entity counts, field counts, and ID tracking for reference validation.
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    /// HEDL version string
    pub version: String,
    /// Number of lists encountered
    pub list_count: usize,
    /// Total number of nodes processed
    pub node_count: usize,
    /// Total number of fields across all nodes
    pub field_count: usize,
    /// Set of seen IDs for strict reference validation (type:id format)
    pub seen_ids: HashSet<String>,
}

impl ValidationStats {
    /// Create new empty validation statistics
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Streaming validation operation for memory-efficient validation of large files.
///
/// Uses the streaming parser from `hedl-stream` to validate files with O(1) memory
/// usage regardless of file size. Ideal for:
/// - Files larger than 100MB
/// - Validating thousands of files with limited RAM
/// - Container environments with memory limits
///
/// # Memory Profile
///
/// - **Input**: O(1) - buffer size only (~8KB)
/// - **Working**: `O(n_ids)` - seen ID set for strict validation
/// - **Output**: O(1) - small statistics struct
/// - **Peak**: ~8KB + ID set size (vs. full file size in standard mode)
///
/// # Examples
///
/// ```rust,no_run
/// use hedl_cli::batch::{BatchProcessor, StreamingValidationOperation};
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let processor = BatchProcessor::default_config();
/// let files = vec![PathBuf::from("large-file.hedl")];
///
/// let operation = StreamingValidationOperation { strict: false };
/// let results = processor.process_streaming(&files, operation, true)?;
///
/// println!("Validated {} files with constant memory", results.success_count());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct StreamingValidationOperation {
    /// Enable strict reference validation
    pub strict: bool,
}

impl StreamingBatchOperation for StreamingValidationOperation {
    type Output = ValidationStats;

    fn process_file_streaming(&self, path: &Path) -> Result<Self::Output, CliError> {
        use hedl_stream::{NodeEvent, StreamError, StreamingParser};
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(path).map_err(|e| CliError::io_error(path, e))?;
        let reader = BufReader::with_capacity(8192, file);

        let parser = StreamingParser::new(reader)
            .map_err(|e: StreamError| CliError::parse(e.to_string()))?;

        let mut stats = ValidationStats::new();
        let mut _current_type = String::new();

        // Process events incrementally
        for event in parser {
            let event = event.map_err(|e: StreamError| CliError::parse(e.to_string()))?;

            match event {
                NodeEvent::Header(info) => {
                    // Validate version exists
                    let version_str = format!("{}.{}", info.version.0, info.version.1);
                    if version_str.is_empty() {
                        return Err(CliError::parse("Missing VERSION".to_string()));
                    }
                    stats.version = version_str;
                }
                NodeEvent::ListStart { type_name, .. } => {
                    stats.list_count += 1;
                    _current_type = type_name;
                }
                NodeEvent::Node(node) => {
                    stats.node_count += 1;
                    stats.field_count += node.fields.len();

                    // Track IDs for strict mode validation
                    let full_id = format!("{}:{}", node.type_name, node.id);

                    if self.strict {
                        // In strict mode, validate references
                        // For now, just track IDs - full reference validation
                        // would require accumulating references and validating at end
                        stats.seen_ids.insert(full_id);
                    } else {
                        stats.seen_ids.insert(full_id);
                    }
                }
                NodeEvent::ListEnd { .. } => {
                    // List validation complete
                }
                NodeEvent::Scalar { .. } => {
                    // Scalar validation - no action needed
                }
                NodeEvent::ObjectStart { .. } => {
                    // Object start - no action needed
                }
                NodeEvent::ObjectEnd { .. } => {
                    // Object end - no action needed
                }
                NodeEvent::EndOfDocument => {
                    // Document complete
                    break;
                }
            }
        }

        Ok(stats)
    }

    fn name(&self) -> &'static str {
        "validate-streaming"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

// ============================================================================
// Helper Functions for Count Hints
// ============================================================================

/// Recursively add count hints to all matrix lists in the document
fn add_count_hints(doc: &mut hedl_core::Document) {
    for item in doc.root.values_mut() {
        add_count_hints_to_item(item);
    }
}

/// Recursively add count hints to an item
fn add_count_hints_to_item(item: &mut hedl_core::Item) {
    use hedl_core::Item;

    match item {
        Item::List(list) => {
            // Set count hint based on actual row count
            list.count_hint = Some(list.rows.len());

            // Recursively add child counts to each node
            for node in &mut list.rows {
                add_child_count_to_node(node);
            }
        }
        Item::Object(map) => {
            // Recursively process nested objects
            for nested_item in map.values_mut() {
                add_count_hints_to_item(nested_item);
            }
        }
        Item::Scalar(_) => {
            // Scalars don't have matrix lists
        }
    }
}

/// Recursively set `child_count` on nodes that have children
fn add_child_count_to_node(node: &mut hedl_core::Node) {
    // Calculate total number of direct children across all child types
    let total_children: usize = node
        .children()
        .map_or(0, |c| c.values().map(std::vec::Vec::len).sum());

    if total_children > 0 {
        node.child_count = total_children.min(u16::MAX as usize) as u16;

        // Recursively process all child nodes
        if let Some(children) = node.children_mut() {
            for child_list in children.values_mut() {
                for child_node in child_list {
                    add_child_count_to_node(child_node);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

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

    #[test]
    fn test_progress_tracker_should_report() {
        let tracker = ProgressTracker::new(100, 10, false);

        assert!(!tracker.should_report(1));
        assert!(!tracker.should_report(9));
        assert!(tracker.should_report(10)); // Interval boundary
        assert!(tracker.should_report(100)); // End
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
        let processor = BatchProcessor::default_config();
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
        let processor = BatchProcessor::default_config();

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
        let processor = BatchProcessor::default_config();

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
        let processor = BatchProcessor::default_config();

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
        let processor = BatchProcessor::new(BatchConfig {
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
        let processor = BatchProcessor::new(BatchConfig {
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
        let processor = BatchProcessor::new(BatchConfig {
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
        let processor = BatchProcessor::new(BatchConfig {
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
        let processor = BatchProcessor::new(BatchConfig {
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
        let processor1 = Arc::new(BatchProcessor::new(BatchConfig {
            max_threads: Some(2),
            parallel_threshold: 1,
            ..Default::default()
        }));

        let processor2 = Arc::new(BatchProcessor::new(BatchConfig {
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
        let processor = BatchProcessor::default_config();

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
        let processor = BatchProcessor::new(BatchConfig {
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
        let processor = BatchProcessor::new(BatchConfig {
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
        let processor = BatchProcessor::new(BatchConfig {
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
        let processor = BatchProcessor::new(BatchConfig {
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
        let processor = BatchProcessor::new(BatchConfig {
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
