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

//! Core HEDL commands for validation, formatting, linting, and inspection.
//!
//! This module contains the fundamental HEDL CLI commands that operate on
//! individual HEDL files for validation, formatting, and analysis.

use crate::commands;
use clap::Subcommand;

/// Core HEDL commands.
///
/// These commands provide the essential functionality for working with HEDL files:
/// validation, formatting, linting, inspection, and statistics.
///
/// # Commands
///
/// - **Validate**: Check HEDL syntax and semantic correctness
/// - **Format**: Convert to canonical form with optional optimizations
/// - **Lint**: Check for best practices and style issues
/// - **Inspect**: Visualize internal structure
/// - **Stats**: Analyze size and token efficiency
#[derive(Subcommand)]
pub enum CoreCommands {
    /// Validate a HEDL file
    ///
    /// Checks the syntax and semantic correctness of a HEDL file. In strict mode,
    /// all references must resolve to existing entities.
    Validate {
        /// Input file path
        #[arg(value_name = "FILE")]
        file: String,

        /// Strict mode (fail on any error)
        #[arg(short, long)]
        strict: bool,
    },

    /// Format a HEDL file to canonical form
    ///
    /// Reformats a HEDL file to its canonical representation. Supports various
    /// formatting options including ditto optimization and automatic count hints.
    Format {
        /// Input file path
        #[arg(value_name = "FILE")]
        file: String,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Check only (exit 1 if not canonical)
        #[arg(short, long)]
        check: bool,

        /// Use ditto optimization
        #[arg(long, default_value = "true")]
        ditto: bool,

        /// Automatically add count hints to all matrix lists
        #[arg(long)]
        with_counts: bool,
    },

    /// Lint a HEDL file for best practices
    ///
    /// Analyzes a HEDL file for style issues, best practices violations, and
    /// potential problems. Can output results in text or JSON format.
    Lint {
        /// Input file path
        #[arg(value_name = "FILE")]
        file: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Treat warnings as errors
        #[arg(short = 'W', long)]
        warn_error: bool,
    },

    /// Print parsed structure (debug)
    ///
    /// Displays the internal structure of a HEDL file as a tree, useful for
    /// debugging and understanding how HEDL parses the file.
    Inspect {
        /// Input file path
        #[arg(value_name = "FILE")]
        file: String,

        /// Show detailed internal structure
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show size/token savings vs other formats
    ///
    /// Analyzes a HEDL file and compares its size and token count against
    /// equivalent representations in JSON, YAML, XML, CSV, and Parquet.
    Stats {
        /// Input HEDL file
        #[arg(value_name = "FILE")]
        file: String,

        /// Show estimated token counts for LLM context
        #[arg(short, long)]
        tokens: bool,
    },
}

impl CoreCommands {
    /// Execute the core command.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error message on failure.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the command execution fails.
    pub fn execute(self) -> Result<(), String> {
        match self {
            CoreCommands::Validate { file, strict } => commands::validate(&file, strict),
            CoreCommands::Format {
                file,
                output,
                check,
                ditto,
                with_counts,
            } => commands::format(&file, output.as_deref(), check, ditto, with_counts),
            CoreCommands::Lint {
                file,
                format,
                warn_error,
            } => commands::lint(&file, &format, warn_error),
            CoreCommands::Inspect { file, verbose } => commands::inspect(&file, verbose),
            CoreCommands::Stats { file, tokens } => commands::stats(&file, tokens),
        }
    }
}
