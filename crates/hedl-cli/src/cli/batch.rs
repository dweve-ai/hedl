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

//! Batch processing commands for HEDL.
//!
//! This module provides commands for processing multiple HEDL files in parallel,
//! enabling efficient bulk operations on large collections of files.

use crate::commands;
use clap::Subcommand;

/// Batch processing commands.
///
/// These commands operate on multiple HEDL files simultaneously, with automatic
/// parallelization for improved performance. All batch commands support glob
/// patterns for file selection.
///
/// # Performance
///
/// Batch commands automatically use parallel processing when beneficial:
/// - CPU-bound operations scale with available cores
/// - I/O-bound operations use async parallelization
/// - Progress reporting shows real-time status
///
/// # Design
///
/// All batch commands follow consistent patterns:
/// - Multiple file inputs (with glob support)
/// - Optional parallel processing flag
/// - Verbose mode for detailed progress
#[derive(Subcommand)]
pub enum BatchCommands {
    /// Batch validate multiple HEDL files
    ///
    /// Validates multiple HEDL files in parallel. Supports glob patterns for
    /// file selection and provides aggregated results.
    BatchValidate {
        /// Input file paths (supports glob patterns)
        #[arg(value_name = "FILES", num_args = 1..)]
        files: Vec<String>,

        /// Strict mode (fail on any error)
        #[arg(short, long)]
        strict: bool,

        /// Force parallel processing
        #[arg(short, long)]
        parallel: bool,

        /// Show verbose progress
        #[arg(short, long)]
        verbose: bool,
    },

    /// Batch format multiple HEDL files
    ///
    /// Formats multiple HEDL files to canonical form in parallel. Can either
    /// modify files in-place or write to an output directory.
    BatchFormat {
        /// Input file paths (supports glob patterns)
        #[arg(value_name = "FILES", num_args = 1..)]
        files: Vec<String>,

        /// Output directory for formatted files
        #[arg(short, long)]
        output_dir: Option<String>,

        /// Check only (exit 1 if not canonical)
        #[arg(short, long)]
        check: bool,

        /// Use ditto optimization
        #[arg(long, default_value = "true")]
        ditto: bool,

        /// Automatically add count hints to all matrix lists
        #[arg(long)]
        with_counts: bool,

        /// Force parallel processing
        #[arg(short, long)]
        parallel: bool,

        /// Show verbose progress
        #[arg(short, long)]
        verbose: bool,
    },

    /// Batch lint multiple HEDL files
    ///
    /// Lints multiple HEDL files in parallel, checking for best practices
    /// and style issues. Provides aggregated results across all files.
    BatchLint {
        /// Input file paths (supports glob patterns)
        #[arg(value_name = "FILES", num_args = 1..)]
        files: Vec<String>,

        /// Treat warnings as errors
        #[arg(short = 'W', long)]
        warn_error: bool,

        /// Force parallel processing
        #[arg(short, long)]
        parallel: bool,

        /// Show verbose progress
        #[arg(short, long)]
        verbose: bool,
    },
}

impl BatchCommands {
    /// Execute the batch command.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error message on failure.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// - No files match the provided patterns
    /// - Any file operation fails
    /// - Processing fails for any file
    ///
    /// # Performance
    ///
    /// Batch commands automatically parallelize when beneficial. The `parallel`
    /// flag forces parallelization even for small file sets.
    pub fn execute(self) -> Result<(), String> {
        match self {
            BatchCommands::BatchValidate {
                files,
                strict,
                parallel,
                verbose,
            } => commands::batch_validate(files, strict, parallel, verbose),
            BatchCommands::BatchFormat {
                files,
                output_dir,
                check,
                ditto,
                with_counts,
                parallel,
                verbose,
            } => commands::batch_format(
                files,
                output_dir,
                check,
                ditto,
                with_counts,
                parallel,
                verbose,
            ),
            BatchCommands::BatchLint {
                files,
                warn_error,
                parallel,
                verbose,
            } => commands::batch_lint(files, warn_error, parallel, verbose),
        }
    }
}
