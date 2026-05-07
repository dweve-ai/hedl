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
//! - `core`: Core commands (validate, format, lint, inspect, stats)
//! - `conversion`: Format conversion commands (JSON, YAML, XML, CSV, Parquet)
//! - `batch`: Batch processing commands (batch-validate, batch-format, batch-lint)
//! - `completion`: Shell completion generation
//!
//! # Design Principles
//!
//! - **Single Responsibility**: Each submodule handles one category of commands
//! - **Consistent API**: All commands follow the same argument patterns
//! - **Type Safety**: Strongly typed arguments with validation
//! - **Extensibility**: Easy to add new commands within existing categories

mod batch;
mod completion;
mod conversion;
mod core;
mod filter;
mod hook;

use clap::Subcommand;

pub use batch::BatchCommands;
pub use completion::CompletionCommands;
pub use conversion::ConversionCommands;
pub use core::CoreCommands;
pub use filter::FilterCommands;
pub use hook::{Agent, HookCommands};

/// Top-level CLI commands enum.
#[derive(Subcommand)]
pub enum Commands {
    /// Core commands (validate, format, lint, inspect, stats).
    #[command(flatten)]
    Core(CoreCommands),

    /// Conversion commands (JSON, YAML, XML, CSV, Parquet).
    #[command(flatten)]
    Conversion(ConversionCommands),

    /// Batch processing commands (batch-validate, batch-format, batch-lint).
    #[command(flatten)]
    Batch(BatchCommands),

    /// Filter commands (run, read, git, cargo, docker, stats, verify).
    #[command(flatten)]
    Filter(FilterCommands),

    /// Shell completion generation.
    #[command(flatten)]
    Completion(CompletionCommands),

    /// Hook commands for AI agent integration.
    #[command(subcommand)]
    Hook(HookCommands),
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
    pub fn execute(self) -> Result<(), crate::error::CliError> {
        match self {
            Commands::Core(cmd) => cmd.execute(),
            Commands::Conversion(cmd) => cmd.execute(),
            Commands::Batch(cmd) => cmd.execute(),
            Commands::Filter(cmd) => cmd.execute(),
            Commands::Completion(cmd) => cmd.execute(),
            Commands::Hook(cmd) => cmd.execute(),
        }
    }
}
