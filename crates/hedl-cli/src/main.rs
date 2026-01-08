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

//! HEDL Command Line Interface

use clap::Parser;
use hedl_cli::cli::Commands;
use std::process::ExitCode;

/// HEDL - Hierarchical Entity Data Language toolkit
///
/// A comprehensive command-line interface for working with HEDL files,
/// providing validation, formatting, linting, format conversion, and
/// batch processing capabilities.
///
/// # Examples
///
/// ```bash
/// # Validate a HEDL file
/// hedl validate example.hedl
///
/// # Format and optimize a HEDL file
/// hedl format example.hedl --output formatted.hedl
///
/// # Convert HEDL to JSON
/// hedl to-json data.hedl --pretty
///
/// # Batch process multiple files
/// hedl batch-format "*.hedl" --parallel
/// ```
#[derive(Parser)]
#[command(name = "hedl")]
#[command(author, version, about = "HEDL - Hierarchical Entity Data Language toolkit", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command.execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}
