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

//! Integration tests for CLI structure refactoring.
//!
//! These tests verify that the refactored CLI structure maintains the same
//! public API and functionality as the original monolithic implementation.

use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "hedl")]
#[command(author, version, about = "HEDL CLI")]
struct TestCli {
    #[command(subcommand)]
    command: hedl_cli::cli::Commands,
}

/// Test that all core commands are accessible at the top level.
#[test]
fn test_core_commands_available() {
    let cmd = TestCli::command();
    let subcommands: Vec<_> = cmd.get_subcommands().map(|c| c.get_name()).collect();

    // Core commands
    assert!(subcommands.contains(&"validate"));
    assert!(subcommands.contains(&"format"));
    assert!(subcommands.contains(&"lint"));
    assert!(subcommands.contains(&"inspect"));
    assert!(subcommands.contains(&"stats"));
}

/// Test that all conversion commands are accessible at the top level.
#[test]
fn test_conversion_commands_available() {
    let cmd = TestCli::command();
    let subcommands: Vec<_> = cmd.get_subcommands().map(|c| c.get_name()).collect();

    // Conversion commands
    assert!(subcommands.contains(&"to-json"));
    assert!(subcommands.contains(&"from-json"));
    assert!(subcommands.contains(&"to-yaml"));
    assert!(subcommands.contains(&"from-yaml"));
    assert!(subcommands.contains(&"to-xml"));
    assert!(subcommands.contains(&"from-xml"));
    assert!(subcommands.contains(&"to-csv"));
    assert!(subcommands.contains(&"from-csv"));
    assert!(subcommands.contains(&"to-parquet"));
    assert!(subcommands.contains(&"from-parquet"));
}

/// Test that all batch commands are accessible at the top level.
#[test]
fn test_batch_commands_available() {
    let cmd = TestCli::command();
    let subcommands: Vec<_> = cmd.get_subcommands().map(|c| c.get_name()).collect();

    // Batch commands
    assert!(subcommands.contains(&"batch-validate"));
    assert!(subcommands.contains(&"batch-format"));
    assert!(subcommands.contains(&"batch-lint"));
}

/// Test that all utility commands are accessible at the top level.
#[test]
fn test_utility_commands_available() {
    let cmd = TestCli::command();
    let subcommands: Vec<_> = cmd.get_subcommands().map(|c| c.get_name()).collect();

    // Utility commands
    assert!(subcommands.contains(&"completion"));
}

/// Test that validate command has correct arguments.
#[test]
fn test_validate_command_args() {
    let result = TestCli::try_parse_from(["hedl", "validate", "test.hedl"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "validate", "test.hedl", "--strict"]);
    assert!(result.is_ok());
}

/// Test that format command has correct arguments.
#[test]
fn test_format_command_args() {
    let result = TestCli::try_parse_from(["hedl", "format", "test.hedl"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "format", "test.hedl", "--output", "out.hedl"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "format", "test.hedl", "--check"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "format", "test.hedl", "--ditto"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "format", "test.hedl", "--with-counts"]);
    assert!(result.is_ok());
}

/// Test that conversion commands have correct arguments.
#[test]
fn test_conversion_command_args() {
    // JSON
    let result = TestCli::try_parse_from(["hedl", "to-json", "test.hedl"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "to-json", "test.hedl", "--pretty"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "from-json", "test.json"]);
    assert!(result.is_ok());

    // YAML
    let result = TestCli::try_parse_from(["hedl", "to-yaml", "test.hedl"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "from-yaml", "test.yaml"]);
    assert!(result.is_ok());

    // XML
    let result = TestCli::try_parse_from(["hedl", "to-xml", "test.hedl", "--pretty"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "from-xml", "test.xml"]);
    assert!(result.is_ok());

    // CSV
    let result = TestCli::try_parse_from(["hedl", "to-csv", "test.hedl", "--headers"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "from-csv", "test.csv", "--type-name", "Row"]);
    assert!(result.is_ok());

    // Parquet
    let result = TestCli::try_parse_from(["hedl", "to-parquet", "test.hedl", "--output", "test.parquet"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "from-parquet", "test.parquet"]);
    assert!(result.is_ok());
}

/// Test that batch commands have correct arguments.
#[test]
fn test_batch_command_args() {
    // Batch validate
    let result = TestCli::try_parse_from(["hedl", "batch-validate", "file1.hedl", "file2.hedl"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "batch-validate", "*.hedl", "--strict", "--parallel"]);
    assert!(result.is_ok());

    // Batch format
    let result = TestCli::try_parse_from(["hedl", "batch-format", "file1.hedl", "file2.hedl"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from([
        "hedl",
        "batch-format",
        "*.hedl",
        "--output-dir",
        "formatted",
        "--parallel",
    ]);
    assert!(result.is_ok());

    // Batch lint
    let result = TestCli::try_parse_from(["hedl", "batch-lint", "file1.hedl", "file2.hedl"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "batch-lint", "*.hedl", "--warn-error", "--parallel"]);
    assert!(result.is_ok());
}

/// Test that completion command has correct arguments.
#[test]
fn test_completion_command_args() {
    let result = TestCli::try_parse_from(["hedl", "completion", "bash"]);
    assert!(result.is_ok());

    let result = TestCli::try_parse_from(["hedl", "completion", "zsh", "--install"]);
    assert!(result.is_ok());
}

/// Test that the CLI help message is generated correctly.
#[test]
fn test_cli_help_generation() {
    let mut cmd = TestCli::command();
    let help = cmd.render_help();
    let help_str = help.to_string();

    // Verify command name and description
    assert!(help_str.contains("HEDL"));

    // Verify some key commands are mentioned
    assert!(help_str.contains("validate"));
    assert!(help_str.contains("format"));
    assert!(help_str.contains("to-json"));
}

/// Test command count to ensure no commands were lost in refactoring.
#[test]
fn test_command_count() {
    let cmd = TestCli::command();
    let count = cmd.get_subcommands().count();

    // We expect 21 commands total:
    // - 5 core (validate, format, lint, inspect, stats)
    // - 12 conversion (to/from for json, yaml, xml, csv, parquet, toon)
    // - 3 batch (batch-validate, batch-format, batch-lint)
    // - 1 utility (completion)
    assert_eq!(count, 21, "Expected 21 commands, found {}", count);
}
