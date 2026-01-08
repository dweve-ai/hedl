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

//! CLI command definitions and argument parsing.
//!
//! This module contains all command-line interface structures for the HEDL CLI,
//! organized into logical categories for better maintainability.
//!
//! # Organization
//!
//! Commands are organized into the following modules:
//!
//! - [`core`]: Core commands (validate, format, lint, inspect, stats)
//! - [`conversion`]: Format conversion commands (JSON, YAML, XML, CSV, Parquet)
//! - [`batch`]: Batch processing commands (batch-validate, batch-format, batch-lint)
//! - [`utility`]: Utility commands (completion)
//!
//! # Design Principles
//!
//! - **Single Responsibility**: Each submodule handles one category of commands
//! - **Consistent API**: All commands follow the same argument patterns
//! - **Type Safety**: Strongly typed arguments with validation
//! - **Extensibility**: Easy to add new commands within existing categories

mod batch;
mod conversion;
mod core;
mod utility;

use clap::Subcommand;

pub use batch::BatchCommands;
pub use conversion::ConversionCommands;
pub use core::CoreCommands;
pub use utility::UtilityCommands;

/// Top-level CLI commands enum.
///
/// This is the main command dispatcher that delegates to specialized command
/// categories. Each variant represents a category of related commands.
///
/// # Architecture
///
/// The commands are organized hierarchically:
///
/// ```text
/// Commands
/// ├── Core (validate, format, lint, inspect, stats)
/// ├── Conversion (JSON, YAML, XML, CSV, Parquet)
/// ├── Batch (batch-validate, batch-format, batch-lint)
/// └── Utility (completion)
/// ```
///
/// # Examples
///
/// ```no_run
/// use clap::Parser;
/// use hedl_cli::cli::Commands;
///
/// #[derive(Parser)]
/// struct Cli {
///     #[command(subcommand)]
///     command: Commands,
/// }
/// ```
#[derive(Subcommand)]
pub enum Commands {
    // Core commands - flattened to appear at top level
    #[command(flatten)]
    Core(CoreCommands),

    // Conversion commands - flattened to appear at top level
    #[command(flatten)]
    Conversion(ConversionCommands),

    // Batch commands - flattened to appear at top level
    #[command(flatten)]
    Batch(BatchCommands),

    // Utility commands - flattened to appear at top level
    #[command(flatten)]
    Utility(UtilityCommands),
}

impl Commands {
    /// Execute the command with the provided arguments.
    ///
    /// This method dispatches to the appropriate command handler based on the
    /// command variant.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful execution, or an error message on failure.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// - File I/O fails
    /// - Parsing or validation fails
    /// - Conversion fails
    /// - Any other command-specific error occurs
    pub fn execute(self) -> Result<(), String> {
        match self {
            Commands::Core(cmd) => cmd.execute(),
            Commands::Conversion(cmd) => cmd.execute(),
            Commands::Batch(cmd) => cmd.execute(),
            Commands::Utility(cmd) => cmd.execute(),
        }
    }
}
