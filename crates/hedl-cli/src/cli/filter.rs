// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Filter commands for command output compression and intelligent filtering.
//!
//! Provides commands that intercept, filter, and compress command outputs
//! before they reach LLM context windows.

use clap::Subcommand;
use std::path::PathBuf;

/// Filter commands for output compression.
#[derive(Subcommand)]
pub enum FilterCommands {
    /// Execute a command and apply intelligent filtering
    ///
    /// Runs any command and applies the best available filter (native or TOML)
    /// to compress the output for LLM consumption.
    Run {
        /// Command to execute
        #[arg(required = true)]
        command: String,
        /// Arguments for the command
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
        /// Bypass TOML filters
        #[arg(long)]
        no_toml: bool,
        /// Show debug information
        #[arg(short, long)]
        debug: bool,
    },

    /// Read a file with intelligent filtering
    ///
    /// Reads a file and applies language-aware comment stripping
    /// and code compression based on the filter level.
    Read {
        /// File(s) to read
        #[arg(required = true, num_args = 1..)]
        files: Vec<PathBuf>,
        /// Filter level: none, minimal, aggressive
        #[arg(short, long, default_value = "none")]
        level: String,
        /// Maximum lines to show
        #[arg(short, long)]
        max_lines: Option<usize>,
        /// Show only last N lines
        #[arg(long, conflicts_with = "max_lines")]
        tail_lines: Option<usize>,
        /// Show line numbers
        #[arg(short = 'n', long)]
        line_numbers: bool,
    },

    /// Git commands with compact output
    ///
    /// Provides compact, token-efficient output for common git commands.
    Git {
        /// Git subcommand (status, log, diff, show, branch)
        #[arg(required = true)]
        subcommand: String,
        /// Additional arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Cargo commands with compact output
    ///
    /// Filters cargo test, build, check, and clippy output to show
    /// only failures and errors.
    Cargo {
        /// Cargo subcommand (test, build, check, clippy)
        #[arg(required = true)]
        subcommand: String,
        /// Additional arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Docker commands with compact output
    ///
    /// Filters docker ps, images, and logs output.
    Docker {
        /// Docker subcommand (ps, images, logs)
        #[arg(required = true)]
        subcommand: String,
        /// Additional arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Kubernetes commands with compact output
    ///
    /// Filters kubectl get pods, services, and logs output.
    Kubectl {
        /// Kubectl subcommand and arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// System commands with compact output
    ///
    /// Filters ls, find, grep, env, ps, df, du, and ping output.
    Sys {
        /// System command (ls, find, grep, env, ps, df, du, ping)
        #[arg(required = true)]
        command: String,
        /// Additional arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Show token savings statistics
    ///
    /// Displays statistics about token savings achieved through filtering.
    #[command(name = "filter-stats")]
    FilterStats {
        /// Show detailed breakdown
        #[arg(short, long)]
        detailed: bool,
    },

    /// Verify TOML filters
    ///
    /// Runs inline tests for all built-in TOML filters.
    Verify {
        /// Run tests only for this filter
        #[arg(long)]
        filter: Option<String>,
        /// Fail if any filter has no tests
        #[arg(long)]
        require_all: bool,
    },
}

impl FilterCommands {
    pub fn execute(self) -> Result<(), crate::error::CliError> {
        match self {
            FilterCommands::Run {
                command,
                args,
                no_toml,
                debug,
            } => {
                let config = hedl_filter::FilterConfig {
                    no_toml,
                    debug,
                    ..Default::default()
                };
                match hedl_filter::run_command(&command, &args, &config) {
                    Ok(result) => {
                        println!("{}", result.output);
                        if result.exit_code != 0 {
                            std::process::exit(result.exit_code);
                        }
                        Ok(())
                    }
                    Err(e) => Err(crate::error::CliError::invalid_input(e)),
                }
            }

            FilterCommands::Read {
                files,
                level,
                max_lines,
                tail_lines,
                line_numbers,
            } => {
                let filter_level = level.parse::<hedl_filter::FilterLevel>()
                    .map_err(|e| crate::error::CliError::invalid_input(e))?;

                for file in &files {
                    let content = crate::commands::read_file(
                        file.to_str().unwrap_or("")
                    )?;

                    let ext = file.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    let lang = hedl_filter::Language::from_extension(ext);

                    let mut filtered = hedl_filter::filter_source(&content,
                        filter_level,
                        lang,
                    );

                    if let Some(max) = max_lines {
                        filtered = hedl_filter::smart_truncate(&filtered,
                            max,
                            &lang,
                        );
                    }

                    if line_numbers {
                        filtered = filtered
                            .lines()
                            .enumerate()
                            .map(|(i, line)| format!("{:4} {}", i + 1, line))
                            .collect::<Vec<_>>()
                            .join("\n");
                    }

                    println!("{}", filtered);
                }

                Ok(())
            }

            FilterCommands::Git { subcommand, args } => {
                let mut all_args = vec![subcommand.clone()];
                all_args.extend(args);
                let config = hedl_filter::FilterConfig::default();
                match hedl_filter::run_command("git", &all_args, &config) {
                    Ok(result) => {
                        println!("{}", result.output);
                        if result.exit_code != 0 {
                            std::process::exit(result.exit_code);
                        }
                        Ok(())
                    }
                    Err(e) => Err(crate::error::CliError::invalid_input(e)),
                }
            }

            FilterCommands::Cargo { subcommand, args } => {
                let mut all_args = vec![subcommand.clone()];
                all_args.extend(args);
                let config = hedl_filter::FilterConfig::default();
                match hedl_filter::run_command("cargo", &all_args, &config) {
                    Ok(result) => {
                        println!("{}", result.output);
                        if result.exit_code != 0 {
                            std::process::exit(result.exit_code);
                        }
                        Ok(())
                    }
                    Err(e) => Err(crate::error::CliError::invalid_input(e)),
                }
            }

            FilterCommands::Docker { subcommand, args } => {
                let mut all_args = vec![subcommand.clone()];
                all_args.extend(args);
                let config = hedl_filter::FilterConfig::default();
                match hedl_filter::run_command("docker", &all_args, &config) {
                    Ok(result) => {
                        println!("{}", result.output);
                        if result.exit_code != 0 {
                            std::process::exit(result.exit_code);
                        }
                        Ok(())
                    }
                    Err(e) => Err(crate::error::CliError::invalid_input(e)),
                }
            }

            FilterCommands::Kubectl { args } => {
                let config = hedl_filter::FilterConfig::default();
                match hedl_filter::run_command("kubectl", &args, &config) {
                    Ok(result) => {
                        println!("{}", result.output);
                        if result.exit_code != 0 {
                            std::process::exit(result.exit_code);
                        }
                        Ok(())
                    }
                    Err(e) => Err(crate::error::CliError::invalid_input(e)),
                }
            }

            FilterCommands::Sys { command, args } => {
                let config = hedl_filter::FilterConfig::default();
                match hedl_filter::run_command(&command, &args, &config) {
                    Ok(result) => {
                        println!("{}", result.output);
                        if result.exit_code != 0 {
                            std::process::exit(result.exit_code);
                        }
                        Ok(())
                    }
                    Err(e) => Err(crate::error::CliError::invalid_input(e)),
                }
            }

            FilterCommands::FilterStats { detailed } => {
                let report = hedl_analytics::get_analytics_report();
                if detailed {
                    println!("{}", report.format());
                } else {
                    let fmt = |n: u64| -> String {
                        if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
                        else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) }
                        else { format!("{}", n) }
                    };
                    println!(
                        "Savings: {:.1}% ({} commands, {} -> {} tokens)",
                        report.savings_percentage,
                        report.total_commands,
                        fmt(report.total_input_tokens),
                        fmt(report.total_output_tokens),
                    );
                }
                Ok(())
            }

            FilterCommands::Verify { filter, require_all } => {
                let results = hedl_filter::hedl::run_filter_tests(
                    filter.as_deref()
                );
                let mut failed = false;

                for outcome in &results.outcomes {
                    let status = if outcome.passed { "PASS" } else { "FAIL" };
                    println!(
                        "[{}] {}::{} ",
                        status, outcome.filter_name, outcome.test_name
                    );
                    if !outcome.passed {
                        failed = true;
                        println!("  expected:\n{}", outcome.expected);
                        println!("  actual:\n{}", outcome.actual);
                    }
                }

                if require_all && !results.filters_without_tests.is_empty() {
                    println!(
                        "\nFilters without tests: {}",
                        results.filters_without_tests.join(", ")
                    );
                    failed = true;
                }

                if failed {
                    Err(crate::error::CliError::invalid_input(
                        "Some filter tests failed"
                    ))
                } else {
                    println!("\nAll tests passed!");
                    Ok(())
                }
            }
        }
    }
}
