// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! AI agent hook system for the entire HEDL ecosystem.
//!
//! Intercepts commands from Claude Code, Cursor, Copilot, and Gemini,
//! rewriting them to use HEDL for optimal token efficiency across ALL
//! operations - not just filtering, but conversion, validation, and more.

pub mod agents;
pub mod init;
pub mod rewrite;

use std::io::{self, Read, Write};

const STDIN_CAP: usize = 1_048_576; // 1 MiB

/// Read stdin with a size limit to prevent memory exhaustion.
pub fn read_stdin_limited() -> Result<String, String> {
    let mut input = String::new();
    io::stdin()
        .take((STDIN_CAP + 1) as u64)
        .read_to_string(&mut input)
        .map_err(|e| format!("Failed to read stdin: {}", e))?;
    if input.len() > STDIN_CAP {
        return Err(format!("hook stdin exceeds {} byte limit", STDIN_CAP));
    }
    Ok(input)
}

/// Write JSON to stdout (using writeln to avoid stdout/stderr corruption).
pub fn write_json(value: &serde_json::Value) -> Result<(), String> {
    writeln!(io::stdout(), "{}", value)
        .map_err(|e| format!("Failed to write stdout: {}", e))
}
