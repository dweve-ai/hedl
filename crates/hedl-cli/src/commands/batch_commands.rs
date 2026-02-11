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

//! Batch command implementations - Process multiple HEDL files efficiently
//!
//! This module provides batch processing capabilities for validating, formatting,
//! and linting multiple HEDL files in parallel or sequentially.

use crate::batch::{
    BatchConfig, BatchExecutor, FormatOperation, LintOperation, ValidationOperation,
};
use crate::error::CliError;
use crate::file_discovery::{DiscoveryConfig, FileDiscovery};
use colored::Colorize;
use std::path::PathBuf;

/// Parameters for batch format operations.
///
/// Groups all configuration for formatting multiple HEDL files,
/// avoiding excessive function arguments.
#[derive(Debug, Clone)]
pub struct BatchFormatParams {
    /// File patterns (glob patterns or explicit paths)
    pub patterns: Vec<String>,
    /// Optional output directory for formatted files
    pub output_dir: Option<String>,
    /// If `true`, checks if files are already canonical without modifying them
    pub check: bool,
    /// If `true`, enables ditto optimization in output
    pub ditto: bool,
    /// If `true`, includes line/value counts in output
    pub with_counts: bool,
    /// Enable recursive directory traversal
    pub recursive: bool,
    /// Maximum recursion depth for directory traversal
    pub max_depth: usize,
    /// If `true`, processes files in parallel
    pub parallel: bool,
    /// If `true`, shows detailed progress information
    pub verbose: bool,
    /// Optional override for the maximum number of files to process
    pub max_files_override: Option<Option<usize>>,
}

/// Batch validate multiple HEDL files.
///
/// Validates multiple HEDL files for syntax and structural correctness, with
/// optional parallel processing for improved performance on large file sets.
///
/// # Arguments
///
/// * `patterns` - List of file patterns (glob patterns or explicit paths)
/// * `strict` - If `true`, enables strict reference validation for all files
/// * `recursive` - Enable recursive directory traversal
/// * `max_depth` - Maximum recursion depth
/// * `parallel` - If `true`, processes files in parallel (automatically enabled for 4+ files)
/// * `verbose` - If `true`, shows detailed progress information
///
/// # Returns
///
/// Returns `Ok(())` if all files are valid, `Err` with a summary if any fail.
///
/// # Errors
///
/// Returns `Err` if:
/// - Any file cannot be read
/// - Any file contains syntax errors
/// - In strict mode, if any references cannot be resolved
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::batch_validate;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Validate multiple files in parallel
/// let patterns = vec!["*.hedl".to_string()];
/// batch_validate(patterns, false, false, 10, true, false)?;
///
/// // Strict validation with verbose output
/// let patterns = vec!["**/*.hedl".to_string()];
/// batch_validate(patterns, true, true, 10, true, true)?;
/// # Ok(())
/// # }
/// ```
///
/// # Output
///
/// Displays progress information and a summary:
/// - Success/failure for each file (✓ or ✗)
/// - Detailed error messages for failures
/// - Final count of failures
///
/// # Performance
///
/// Automatically uses parallel processing when beneficial (4+ files by default).
/// Can be forced with the `parallel` flag for smaller file sets.
pub fn batch_validate(
    patterns: Vec<String>,
    strict: bool,
    recursive: bool,
    max_depth: usize,
    parallel: bool,
    verbose: bool,
) -> Result<(), CliError> {
    batch_validate_with_config(
        patterns, strict, recursive, max_depth, parallel, verbose, None,
    )
}

/// Batch validate with custom configuration.
///
/// Like `batch_validate`, but allows overriding the max files limit.
pub fn batch_validate_with_config(
    patterns: Vec<String>,
    strict: bool,
    recursive: bool,
    max_depth: usize,
    parallel: bool,
    verbose: bool,
    max_files_override: Option<Option<usize>>,
) -> Result<(), CliError> {
    // Discover files from patterns
    let discovery_config = DiscoveryConfig {
        max_depth: Some(max_depth),
        extension: Some("hedl".to_string()),
        recursive,
        ..Default::default()
    };

    let discovery = FileDiscovery::new(patterns, discovery_config);
    let paths = discovery.discover()?;

    let mut config = BatchConfig {
        parallel_threshold: if parallel { 1 } else { usize::MAX },
        verbose,
        ..Default::default()
    };

    // Apply CLI override if provided
    if let Some(override_limit) = max_files_override {
        config.max_files = override_limit;
    }

    // Validate file count against limit
    crate::batch::validate_file_count(paths.len(), config.max_files)?;

    // Warn if processing many files
    crate::batch::warn_large_batch(paths.len(), verbose);

    let processor = BatchExecutor::new(config);
    let operation = ValidationOperation { strict };

    let results = processor.process(&paths, operation, true)?;

    if results.has_failures() {
        eprintln!();
        eprintln!("{}", "Validation failures:".red().bold());
        for failure in results.failures() {
            eprintln!("  {} {}", "✗".red(), failure.path.display());
            if let Err(e) = &failure.result {
                let e: &CliError = e;
                eprintln!("    {}", e.to_string().dimmed());
            }
        }
        return Err(CliError::invalid_input(format!(
            "{} of {} files failed validation",
            results.failure_count(),
            results.total_files()
        )));
    }

    Ok(())
}

/// Batch format multiple HEDL files to canonical form.
///
/// Formats multiple HEDL files to canonical form, with options for check-only mode,
/// ditto optimization, and count hints. Supports parallel processing for improved
/// performance on large file sets.
///
/// # Arguments
///
/// * `patterns` - List of file patterns (glob patterns or explicit paths)
/// * `output_dir` - Optional output directory for formatted files. If `None`, files are processed in-place
/// * `check` - If `true`, only checks if files are canonical without reformatting
/// * `ditto` - If `true`, uses ditto optimization (repeated values as `"`)
/// * `with_counts` - If `true`, automatically adds count hints to all matrix lists
/// * `recursive` - Enable recursive directory traversal
/// * `max_depth` - Maximum recursion depth
/// * `parallel` - If `true`, processes files in parallel (automatically enabled for 4+ files)
/// * `verbose` - If `true`, shows detailed progress information
///
/// # Returns
///
/// Returns `Ok(())` if all files are successfully formatted, `Err` with a summary if any fail.
///
/// # Errors
///
/// Returns `Err` if:
/// - Any file cannot be read
/// - Any file contains syntax errors
/// - Canonicalization fails for any file
/// - In check mode, if any file is not already canonical
/// - Output directory cannot be created
/// - Formatted files cannot be written
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::{batch_format, BatchFormatParams};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Format files to output directory
/// batch_format(BatchFormatParams {
///     patterns: vec!["*.hedl".to_string()],
///     output_dir: Some("formatted/".to_string()),
///     check: false,
///     ditto: true,
///     with_counts: false,
///     recursive: false,
///     max_depth: 10,
///     parallel: true,
///     verbose: false,
///     max_files_override: None,
/// })?;
/// # Ok(())
/// # }
/// ```
///
/// # Output
///
/// Displays progress information and a summary:
/// - Success/failure for each file (✓ or ✗)
/// - Detailed error messages for failures
/// - Final count of failures
///
/// # Performance
///
/// Automatically uses parallel processing when beneficial (4+ files by default).
/// Can be forced with the `parallel` flag for smaller file sets.
pub fn batch_format(params: BatchFormatParams) -> Result<(), CliError> {
    batch_format_with_config(params)
}

/// Batch format with custom configuration.
///
/// Like `batch_format`, but allows overriding the max files limit.
pub fn batch_format_with_config(params: BatchFormatParams) -> Result<(), CliError> {
    let BatchFormatParams {
        patterns,
        output_dir,
        check,
        ditto,
        with_counts,
        recursive,
        max_depth,
        parallel,
        verbose,
        max_files_override,
    } = params;
    // Discover files from patterns
    let discovery_config = DiscoveryConfig {
        max_depth: Some(max_depth),
        extension: Some("hedl".to_string()),
        recursive,
        ..Default::default()
    };

    let discovery = FileDiscovery::new(patterns, discovery_config);
    let paths = discovery.discover()?;

    let mut config = BatchConfig {
        parallel_threshold: if parallel { 1 } else { usize::MAX },
        verbose,
        ..Default::default()
    };

    // Apply CLI override if provided
    if let Some(override_limit) = max_files_override {
        config.max_files = override_limit;
    }

    // Validate file count against limit
    crate::batch::validate_file_count(paths.len(), config.max_files)?;

    // Warn if processing many files
    crate::batch::warn_large_batch(paths.len(), verbose);

    let processor = BatchExecutor::new(config);
    let operation = FormatOperation {
        check,
        ditto,
        with_counts,
    };

    let results = processor.process(&paths, operation, true)?;

    // If not in check mode and output_dir is specified, write formatted files
    if !check {
        if let Some(out_dir) = output_dir {
            std::fs::create_dir_all(&out_dir).map_err(|e| CliError::io_error(&out_dir, e))?;

            for result in results.successes() {
                if let Ok(formatted) = &result.result {
                    let output_path = PathBuf::from(&out_dir).join(
                        result
                            .path
                            .file_name()
                            .ok_or_else(|| CliError::invalid_input("Invalid file name"))?,
                    );
                    std::fs::write(&output_path, formatted)
                        .map_err(|e| CliError::io_error(&output_path, e))?;
                }
            }
        }
    }

    if results.has_failures() {
        eprintln!();
        eprintln!("{}", "Format failures:".red().bold());
        for failure in results.failures() {
            eprintln!("  {} {}", "✗".red(), failure.path.display());
            if let Err(e) = &failure.result {
                let e: &CliError = e;
                eprintln!("    {}", e.to_string().dimmed());
            }
        }
        return Err(CliError::invalid_input(format!(
            "{} of {} files failed formatting",
            results.failure_count(),
            results.total_files()
        )));
    }

    Ok(())
}

/// Batch lint multiple HEDL files for best practices and style issues.
///
/// Lints multiple HEDL files for potential issues, style violations, and best
/// practice deviations. Supports parallel processing for improved performance
/// on large file sets.
///
/// # Arguments
///
/// * `patterns` - List of file patterns (glob patterns or explicit paths)
/// * `warn_error` - If `true`, treat warnings as errors (fail on any warning)
/// * `recursive` - Enable recursive directory traversal
/// * `max_depth` - Maximum recursion depth
/// * `parallel` - If `true`, processes files in parallel (automatically enabled for 4+ files)
/// * `verbose` - If `true`, shows detailed progress information
///
/// # Returns
///
/// Returns `Ok(())` if no issues are found (or only hints), `Err` if errors or warnings
/// (with `warn_error` enabled) are detected.
///
/// # Errors
///
/// Returns `Err` if:
/// - Any file cannot be read
/// - Any file contains syntax errors
/// - Lint errors are found in any file
/// - Warnings are found and `warn_error` is `true`
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::batch_lint;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Lint multiple files
/// let patterns = vec!["*.hedl".to_string()];
/// batch_lint(patterns, false, false, 10, true, false)?;
///
/// // Strict linting (warnings as errors)
/// let patterns = vec!["**/*.hedl".to_string()];
/// batch_lint(patterns, true, true, 10, true, true)?;
/// # Ok(())
/// # }
/// ```
///
/// # Output
///
/// Displays:
/// - Progress information for each file
/// - All lint diagnostics with severity, rule ID, message, and line number
/// - Suggestions for fixing issues
/// - Summary of total issues found
///
/// # Performance
///
/// Automatically uses parallel processing when beneficial (4+ files by default).
/// Can be forced with the `parallel` flag for smaller file sets.
pub fn batch_lint(
    patterns: Vec<String>,
    warn_error: bool,
    recursive: bool,
    max_depth: usize,
    parallel: bool,
    verbose: bool,
) -> Result<(), CliError> {
    batch_lint_with_config(
        patterns, warn_error, recursive, max_depth, parallel, verbose, None,
    )
}

/// Batch lint with custom configuration.
///
/// Like `batch_lint`, but allows overriding the max files limit.
pub fn batch_lint_with_config(
    patterns: Vec<String>,
    warn_error: bool,
    recursive: bool,
    max_depth: usize,
    parallel: bool,
    verbose: bool,
    max_files_override: Option<Option<usize>>,
) -> Result<(), CliError> {
    // Discover files from patterns
    let discovery_config = DiscoveryConfig {
        max_depth: Some(max_depth),
        extension: Some("hedl".to_string()),
        recursive,
        ..Default::default()
    };

    let discovery = FileDiscovery::new(patterns, discovery_config);
    let paths = discovery.discover()?;

    let mut config = BatchConfig {
        parallel_threshold: if parallel { 1 } else { usize::MAX },
        verbose,
        ..Default::default()
    };

    // Apply CLI override if provided
    if let Some(override_limit) = max_files_override {
        config.max_files = override_limit;
    }

    // Validate file count against limit
    crate::batch::validate_file_count(paths.len(), config.max_files)?;

    // Warn if processing many files
    crate::batch::warn_large_batch(paths.len(), verbose);

    let processor = BatchExecutor::new(config);
    let operation = LintOperation { warn_error };

    let results = processor.process(&paths, operation, true)?;

    // Show lint diagnostics for files that have issues
    let mut total_issues = 0;
    for result in results.successes() {
        if let Ok(diagnostics) = &result.result {
            let diagnostics: &Vec<String> = diagnostics;
            if !diagnostics.is_empty() {
                total_issues += diagnostics.len();
                println!();
                println!("{} {}:", "Linting".yellow().bold(), result.path.display());
                for diagnostic in diagnostics {
                    println!("  {diagnostic}");
                }
            }
        }
    }

    if results.has_failures() {
        eprintln!();
        eprintln!("{}", "Lint failures:".red().bold());
        for failure in results.failures() {
            eprintln!("  {} {}", "✗".red(), failure.path.display());
            if let Err(e) = &failure.result {
                let e: &CliError = e;
                eprintln!("    {}", e.to_string().dimmed());
            }
        }
        return Err(CliError::invalid_input(format!(
            "{} of {} files failed linting",
            results.failure_count(),
            results.total_files()
        )));
    }

    if total_issues > 0 {
        println!();
        println!(
            "{} {} issues found across {} files",
            "Summary:".bright_blue().bold(),
            total_issues,
            results.total_files()
        );
    }

    Ok(())
}
