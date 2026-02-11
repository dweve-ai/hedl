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

//! Batch processing configuration.

use crate::error::CliError;
use colored::Colorize;

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
pub fn get_max_batch_files() -> usize {
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
