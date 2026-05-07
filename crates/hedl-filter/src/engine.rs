// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Core filter engine coordinating native and TOML filters with HEDL output.

use crate::{
    commands, hedl, utils,
    utils::{exit_code_from_output, resolved_command},
};
use hedl_analytics::tracker::{track_command, start_timer, elapsed_ms};
use std::process::Stdio;

/// Execution mode for commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Normal filtered execution
    Filtered,
    /// Passthrough without filtering (but track)
    Passthrough,
    /// Stream output in real-time
    Streaming,
}

/// Configuration for command filtering.
#[derive(Debug, Clone)]
pub struct FilterConfig {
    /// Whether to bypass TOML filters
    pub no_toml: bool,
    /// Whether to show debug info
    pub debug: bool,
    /// Maximum output size in bytes
    pub max_output_size: usize,
    /// Execution mode
    pub mode: ExecutionMode,
    /// Generate HEDL output for structured data
    pub hedl_output: bool,
    /// Tee raw output on failure
    pub tee: bool,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            no_toml: false,
            debug: false,
            max_output_size: 10 * 1024 * 1024,
            mode: ExecutionMode::Filtered,
            hedl_output: true,
            tee: true,
        }
    }
}

/// Result of running a filtered command.
pub struct FilterResult {
    /// Filtered output
    pub output: String,
    /// Exit code from the underlying command
    pub exit_code: i32,
    /// Whether a native filter was applied
    pub native_filtered: bool,
    /// Whether a TOML filter was applied
    pub toml_filtered: bool,
    /// Whether HEDL output was generated
    pub hedl_output: bool,
    /// Input tokens
    pub input_tokens: usize,
    /// Output tokens
    pub output_tokens: usize,
}

/// Run a command and apply the best available filter.
pub fn run_command(cmd: &str, args: &[String], config: &FilterConfig) -> Result<FilterResult, String> {
    match config.mode {
        ExecutionMode::Streaming => run_streaming(cmd, args, config),
        ExecutionMode::Passthrough => run_passthrough(cmd, args, config),
        ExecutionMode::Filtered => run_filtered(cmd, args, config),
    }
}

fn run_filtered(cmd: &str, args: &[String], config: &FilterConfig) -> Result<FilterResult, String> {
    let cmd_str = std::iter::once(cmd)
        .chain(args.iter().map(|s| s.as_str()))
        .collect::<Vec<_>>()
        .join(" ");

    // Check for native filter first
    if let Some(filter_fn) = commands::get_native_filter(cmd, args) {
        return run_native_filtered(cmd, args, filter_fn, config);
    }

    // Check for HEDL declarative filter
    if !config.no_toml {
        if let Some(filter) = hedl::find_matching_filter(&cmd_str) {
            return run_toml_filtered(cmd, args, filter, config);
        }
    }

    // Passthrough with basic safety
    run_passthrough(cmd, args, config)
}

fn run_native_filtered(
    cmd: &str,
    args: &[String],
    filter_fn: commands::NativeFilterFn,
    config: &FilterConfig,
) -> Result<FilterResult, String> {
    let output = resolved_command(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?;

    let exit_code = exit_code_from_output(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    let input_tokens = utils::count_tokens(&combined);
    let timer = start_timer();

    let filtered = if config.hedl_output {
        filter_fn(&combined, exit_code != 0, true)
    } else {
        filter_fn(&combined, exit_code != 0, false)
    };

    let output_tokens = utils::count_tokens(&filtered);
    let duration_ms = elapsed_ms(timer);

    track_command(
        &format!("{} {}", cmd, args.join(" ")),
        input_tokens,
        output_tokens,
        "native",
        exit_code,
        duration_ms,
    );

    if config.debug {
        eprintln!("[hedl-filter] native: {} {} -> {} tokens saved", cmd, args.join(" "), input_tokens - output_tokens);
    }

    Ok(FilterResult {
        output: filtered,
        exit_code,
        native_filtered: true,
        toml_filtered: false,
        hedl_output: config.hedl_output,
        input_tokens,
        output_tokens,
    })
}

fn run_toml_filtered(
    cmd: &str,
    args: &[String],
    filter: &hedl::CompiledFilter,
    config: &FilterConfig,
) -> Result<FilterResult, String> {
    let mut command = resolved_command(cmd);
    command.args(args);
    command.stdin(Stdio::inherit());

    if filter.filter_stderr {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
    } else {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::inherit());
    }

    let output = command
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?;

    let exit_code = exit_code_from_output(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let combined = if filter.filter_stderr {
        format!("{}{}", stdout, stderr)
    } else {
        stdout.to_string()
    };

    let input_tokens = utils::count_tokens(&combined);
    let timer = start_timer();
    let filtered = hedl::apply_toml_filter(filter, &combined);
    let output_tokens = utils::count_tokens(&filtered);
    let duration_ms = elapsed_ms(timer);

    track_command(
        &format!("{} {}", cmd, args.join(" ")),
        input_tokens,
        output_tokens,
        "toml",
        exit_code,
        duration_ms,
    );

    if config.debug {
        eprintln!("[hedl-filter] toml '{}': {} -> {} tokens", filter.name, input_tokens, output_tokens);
    }

    Ok(FilterResult {
        output: filtered,
        exit_code,
        native_filtered: false,
        toml_filtered: true,
        hedl_output: false,
        input_tokens,
        output_tokens,
    })
}

fn run_passthrough(
    cmd: &str,
    args: &[String],
    config: &FilterConfig,
) -> Result<FilterResult, String> {
    let output = resolved_command(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", cmd, e))?;

    let exit_code = exit_code_from_output(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let input_tokens = utils::count_tokens(&stdout);

    let output_str = if stdout.len() > config.max_output_size {
        format!(
            "{}\n... ({} bytes truncated)",
            &stdout[..config.max_output_size],
            stdout.len() - config.max_output_size
        )
    } else {
        stdout.to_string()
    };

    let output_tokens = utils::count_tokens(&output_str);
    track_command(
        &format!("{} {}", cmd, args.join(" ")),
        input_tokens,
        output_tokens,
        "passthrough",
        exit_code,
        0,
    );

    Ok(FilterResult {
        output: output_str,
        exit_code,
        native_filtered: false,
        toml_filtered: false,
        hedl_output: false,
        input_tokens,
        output_tokens,
    })
}

fn run_streaming(
    cmd: &str,
    args: &[String],
    _config: &FilterConfig,
) -> Result<FilterResult, String> {
    use std::io::{BufRead, BufReader};

    let mut child = resolved_command(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", cmd, e))?;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
            }
        }
    }

    let status = child.wait().map_err(|e| format!("Wait failed: {}", e))?;
    let exit_code = status.code().unwrap_or(1);

    Ok(FilterResult {
        output: String::new(),
        exit_code,
        native_filtered: false,
        toml_filtered: false,
        hedl_output: false,
        input_tokens: 0,
        output_tokens: 0,
    })
}
