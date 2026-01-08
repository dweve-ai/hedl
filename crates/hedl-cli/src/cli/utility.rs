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

//! Utility commands for HEDL CLI.
//!
//! This module provides utility commands that enhance the CLI experience,
//! such as shell completion generation.

use crate::commands;
use clap::Subcommand;
use clap::CommandFactory;
use clap_complete::shells::*;

/// Utility commands.
///
/// These commands provide helpful utilities for working with the HEDL CLI,
/// including shell completion generation and help system enhancements.
#[derive(Subcommand)]
pub enum UtilityCommands {
    /// Generate shell completion scripts
    ///
    /// Generates shell completion scripts for various shells, enabling
    /// tab completion for HEDL commands, options, and file names.
    ///
    /// Supported shells: bash, zsh, fish, powershell, elvish
    Completion {
        /// Shell to generate completions for (bash, zsh, fish, powershell, elvish)
        #[arg(value_name = "SHELL")]
        shell: String,

        /// Print installation instructions instead of generating script
        #[arg(short, long)]
        install: bool,
    },
}

impl UtilityCommands {
    /// Execute the utility command.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error message on failure.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// - Unsupported shell is specified
    /// - Completion generation fails
    pub fn execute(self) -> Result<(), String> {
        match self {
            UtilityCommands::Completion { shell, install } => {
                if install {
                    println!("{}", commands::print_installation_instructions(&shell));
                    Ok(())
                } else {
                    generate_completion(&shell)
                }
            }
        }
    }
}

/// Generate shell completion for the specified shell.
///
/// This is a helper function that creates a temporary command instance
/// for completion generation. It needs access to the full CLI structure.
///
/// # Arguments
///
/// * `shell` - Shell name (bash, zsh, fish, powershell, elvish)
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error message on failure.
///
/// # Errors
///
/// Returns `Err` if the shell is not supported.
fn generate_completion(shell: &str) -> Result<(), String> {
    // We need to create a temporary CLI structure for completion generation.
    // This is a bit awkward since we're in a submodule, but we can use
    // a trait-based approach to avoid circular dependencies.

    // For now, we'll use a temporary command that matches the main CLI.
    // This should be replaced with a better solution that uses the actual
    // CLI command from main.rs.

    use clap::Parser;

    #[derive(Parser)]
    #[command(name = "hedl")]
    #[command(author, version, about = "HEDL - Hierarchical Entity Data Language toolkit")]
    struct TempCli {
        #[command(subcommand)]
        command: super::Commands,
    }

    let mut cmd = TempCli::command();

    match shell.to_lowercase().as_str() {
        "bash" => commands::generate_completion_for_command(Bash, &mut cmd),
        "zsh" => commands::generate_completion_for_command(Zsh, &mut cmd),
        "fish" => commands::generate_completion_for_command(Fish, &mut cmd),
        "powershell" | "pwsh" => commands::generate_completion_for_command(PowerShell, &mut cmd),
        "elvish" => commands::generate_completion_for_command(Elvish, &mut cmd),
        _ => Err(format!(
            "Unsupported shell: '{}'. Supported shells: bash, zsh, fish, powershell, elvish",
            shell
        )),
    }
}
