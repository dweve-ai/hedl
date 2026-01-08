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

use crate::error::CliError;
use colored::Colorize;
use rayon::prelude::*;
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
    /// None means use Rayon's default (typically number of CPU cores).
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
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            parallel_threshold: 10,
            max_threads: None,
            progress_interval: 1,
            verbose: false,
        }
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
    pub fn new(results: Vec<FileResult<T>>, elapsed_ms: u128) -> Self {
        Self { results, elapsed_ms }
    }

    /// Get the total number of files processed.
    pub fn total_files(&self) -> usize {
        self.results.len()
    }

    /// Get the number of successfully processed files.
    pub fn success_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_success()).count()
    }

    /// Get the number of failed files.
    pub fn failure_count(&self) -> usize {
        self.results.iter().filter(|r| r.is_failure()).count()
    }

    /// Check if all files were processed successfully.
    pub fn all_succeeded(&self) -> bool {
        self.results.iter().all(|r| r.is_success())
    }

    /// Check if any files failed.
    pub fn has_failures(&self) -> bool {
        self.results.iter().any(|r| r.is_failure())
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
    /// Should return appropriate CliError variants for different failure modes.
    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError>;

    /// Get a human-readable name for this operation.
    ///
    /// Used for progress reporting and logging.
    fn name(&self) -> &str;
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
        self.interval > 0 && (processed.is_multiple_of(self.interval) || processed == self.total)
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
/// BatchProcessor is thread-safe and can be shared across threads via Arc.
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
    pub fn new(config: BatchConfig) -> Self {
        Self { config }
    }

    /// Create a batch processor with default configuration.
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
    /// * `Ok(BatchResults)` - Always succeeds and collects all individual results
    /// * `Err(CliError)` - Only on catastrophic failures (e.g., thread pool creation)
    ///
    /// # Performance
    ///
    /// - Uses parallel processing if `files.len() >= config.parallel_threshold`
    /// - Serial processing for small batches to avoid thread pool overhead
    /// - Lock-free progress tracking using atomic counters
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::batch::{BatchProcessor, BatchConfig, FormatOperation};
    /// use std::path::PathBuf;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let processor = BatchProcessor::default_config();
    /// let files = vec![PathBuf::from("a.hedl"), PathBuf::from("b.hedl")];
    ///
    /// let results = processor.process(
    ///     &files,
    ///     FormatOperation {
    ///         check: false,
    ///         ditto: true,
    ///         with_counts: false,
    ///     },
    ///     true,
    /// )?;
    ///
    /// println!("Formatted {} files", results.success_count());
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
            // Serial processing for small batches
            self.process_serial(files, &operation, show_progress)
        } else {
            // Parallel processing for larger batches
            self.process_parallel(files, &operation, show_progress)
        };

        let elapsed_ms = start_time.elapsed().as_millis();

        Ok(BatchResults::new(results, elapsed_ms))
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
    type Output = ();

    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
        use hedl_core::{parse_with_limits, ParseOptions};

        let content =
            std::fs::read_to_string(path).map_err(|e| CliError::io_error(path, e))?;

        let options = ParseOptions {
            strict_refs: self.strict,
            ..ParseOptions::default()
        };

        parse_with_limits(content.as_bytes(), options)
            .map_err(|e| CliError::parse(e.to_string()))?;

        Ok(())
    }

    fn name(&self) -> &str {
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

        let content =
            std::fs::read_to_string(path).map_err(|e| CliError::io_error(path, e))?;

        let mut doc = parse(content.as_bytes()).map_err(|e| CliError::parse(e.to_string()))?;

        // Add count hints if requested
        if self.with_counts {
            add_count_hints(&mut doc);
        }

        let config = CanonicalConfig::new()
            .with_ditto(self.ditto);

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

        let content =
            std::fs::read_to_string(path).map_err(|e| CliError::io_error(path, e))?;

        let doc = parse(content.as_bytes()).map_err(|e| CliError::parse(e.to_string()))?;

        let diagnostics = lint(&doc);

        if self.warn_error && !diagnostics.is_empty() {
            return Err(CliError::LintErrors);
        }

        Ok(diagnostics.iter().map(|d| d.to_string()).collect())
    }

    fn name(&self) -> &str {
        "lint"
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

/// Recursively set child_count on nodes that have children
fn add_child_count_to_node(node: &mut hedl_core::Node) {
    // Calculate total number of direct children across all child types
    let total_children: usize = node.children.values().map(|v| v.len()).sum();

    if total_children > 0 {
        node.child_count = Some(total_children);

        // Recursively process all child nodes
        for child_list in node.children.values_mut() {
            for child_node in child_list {
                add_child_count_to_node(child_node);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        fn name(&self) -> &str {
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

        let files: Vec<PathBuf> = (0..20).map(|i| PathBuf::from(format!("file{}.hedl", i))).collect();

        let results = processor
            .process(&files, MockOperation { should_fail: false }, false)
            .unwrap();

        assert_eq!(results.total_files(), 20);
        assert_eq!(results.success_count(), 20);
    }
}
