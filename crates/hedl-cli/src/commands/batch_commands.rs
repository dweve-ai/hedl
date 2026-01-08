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

use crate::batch::{BatchConfig, BatchProcessor, FormatOperation, LintOperation, ValidationOperation};
use crate::error::CliError;
use colored::Colorize;
use std::path::PathBuf;

/// Batch validate multiple HEDL files.
///
/// Validates multiple HEDL files for syntax and structural correctness, with
/// optional parallel processing for improved performance on large file sets.
///
/// # Arguments
///
/// * `files` - List of file paths to validate
/// * `strict` - If `true`, enables strict reference validation for all files
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
/// # fn main() -> Result<(), String> {
/// // Validate multiple files in parallel
/// let files = vec!["file1.hedl".to_string(), "file2.hedl".to_string()];
/// batch_validate(files, false, true, false)?;
///
/// // Strict validation with verbose output
/// let files = vec!["test1.hedl".to_string(), "test2.hedl".to_string()];
/// batch_validate(files, true, true, true)?;
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
    files: Vec<String>,
    strict: bool,
    parallel: bool,
    verbose: bool,
) -> Result<(), String> {
    let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();

    let config = BatchConfig {
        parallel_threshold: if parallel { 1 } else { usize::MAX },
        verbose,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config);
    let operation = ValidationOperation { strict };

    let results = processor
        .process(&paths, operation, true)
        .map_err(|e: CliError| e.to_string())?;

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
        return Err(format!(
            "{} of {} files failed validation",
            results.failure_count(),
            results.total_files()
        ));
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
/// * `files` - List of file paths to format
/// * `output_dir` - Optional output directory for formatted files. If `None`, files are processed in-place
/// * `check` - If `true`, only checks if files are canonical without reformatting
/// * `ditto` - If `true`, uses ditto optimization (repeated values as `"`)
/// * `with_counts` - If `true`, automatically adds count hints to all matrix lists
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
/// use hedl_cli::commands::batch_format;
///
/// # fn main() -> Result<(), String> {
/// // Format files to output directory
/// let files = vec!["file1.hedl".to_string(), "file2.hedl".to_string()];
/// batch_format(files, Some("formatted/".to_string()), false, true, false, true, false)?;
///
/// // Check if files are canonical
/// let files = vec!["test1.hedl".to_string(), "test2.hedl".to_string()];
/// batch_format(files, None, true, true, false, true, false)?;
///
/// // Format with count hints
/// let files = vec!["data.hedl".to_string()];
/// batch_format(files, Some("output/".to_string()), false, true, true, false, true)?;
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
pub fn batch_format(
    files: Vec<String>,
    output_dir: Option<String>,
    check: bool,
    ditto: bool,
    with_counts: bool,
    parallel: bool,
    verbose: bool,
) -> Result<(), String> {
    let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();

    let config = BatchConfig {
        parallel_threshold: if parallel { 1 } else { usize::MAX },
        verbose,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config);
    let operation = FormatOperation {
        check,
        ditto,
        with_counts,
    };

    let results = processor
        .process(&paths, operation, true)
        .map_err(|e: CliError| e.to_string())?;

    // If not in check mode and output_dir is specified, write formatted files
    if !check && output_dir.is_some() {
        let out_dir = output_dir.unwrap();
        std::fs::create_dir_all(&out_dir)
            .map_err(|e| format!("Failed to create output directory '{}': {}", out_dir, e))?;

        for result in results.successes() {
            if let Ok(formatted) = &result.result {
                let output_path = PathBuf::from(&out_dir).join(
                    result
                        .path
                        .file_name()
                        .ok_or("Invalid file name")?,
                );
                std::fs::write(&output_path, formatted).map_err(|e| {
                    format!("Failed to write '{}': {}", output_path.display(), e)
                })?;
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
        return Err(format!(
            "{} of {} files failed formatting",
            results.failure_count(),
            results.total_files()
        ));
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
/// * `files` - List of file paths to lint
/// * `warn_error` - If `true`, treat warnings as errors (fail on any warning)
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
/// # fn main() -> Result<(), String> {
/// // Lint multiple files
/// let files = vec!["file1.hedl".to_string(), "file2.hedl".to_string()];
/// batch_lint(files, false, true, false)?;
///
/// // Strict linting (warnings as errors)
/// let files = vec!["test1.hedl".to_string(), "test2.hedl".to_string()];
/// batch_lint(files, true, true, true)?;
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
    files: Vec<String>,
    warn_error: bool,
    parallel: bool,
    verbose: bool,
) -> Result<(), String> {
    let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();

    let config = BatchConfig {
        parallel_threshold: if parallel { 1 } else { usize::MAX },
        verbose,
        ..Default::default()
    };

    let processor = BatchProcessor::new(config);
    let operation = LintOperation { warn_error };

    let results = processor
        .process(&paths, operation, true)
        .map_err(|e: CliError| e.to_string())?;

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
                    println!("  {}", diagnostic);
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
        return Err(format!(
            "{} of {} files failed linting",
            results.failure_count(),
            results.total_files()
        ));
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
