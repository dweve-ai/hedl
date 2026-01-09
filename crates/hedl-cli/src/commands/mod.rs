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

//! CLI command implementations

mod batch_commands;
mod completion;
mod convert;
mod format;
mod inspect;
mod lint;
mod stats;
mod validate;

pub use batch_commands::{batch_format, batch_lint, batch_validate};
pub use completion::{generate_completion_for_command, print_installation_instructions};
pub use convert::{
    from_csv, from_json, from_parquet, from_toon, from_xml, from_yaml, to_csv, to_json,
    to_parquet, to_toon, to_xml, to_yaml,
};
pub use format::format;
pub use inspect::inspect;
pub use lint::lint;
pub use stats::stats;
pub use validate::validate;

use std::fs;
use std::io::{self, Write};

/// Default maximum file size to prevent OOM attacks (1 GB)
/// Can be overridden via HEDL_MAX_FILE_SIZE environment variable
pub const DEFAULT_MAX_FILE_SIZE: u64 = 1024 * 1024 * 1024;

/// Get the maximum file size from environment or use default.
///
/// Reads the `HEDL_MAX_FILE_SIZE` environment variable to allow customization
/// of the maximum allowed file size. Falls back to [`DEFAULT_MAX_FILE_SIZE`] if
/// the variable is not set or contains an invalid value.
///
/// # Examples
///
/// ```
/// use hedl_cli::commands::DEFAULT_MAX_FILE_SIZE;
///
/// // Default behavior
/// std::env::remove_var("HEDL_MAX_FILE_SIZE");
/// // Note: get_max_file_size is private, so this example shows the concept
/// // let size = get_max_file_size();
/// // assert_eq!(size, DEFAULT_MAX_FILE_SIZE);
///
/// // Custom size via environment variable
/// std::env::set_var("HEDL_MAX_FILE_SIZE", "500000000"); // 500 MB
/// // let size = get_max_file_size();
/// // assert_eq!(size, 500_000_000);
/// ```
fn get_max_file_size() -> u64 {
    std::env::var("HEDL_MAX_FILE_SIZE")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_MAX_FILE_SIZE)
}

/// Read a file from disk with size validation.
///
/// Reads the entire contents of a file into a string, with built-in protection
/// against out-of-memory (OOM) attacks. Files larger than the configured maximum
/// size will be rejected before reading.
///
/// # Arguments
///
/// * `path` - Path to the file to read
///
/// # Returns
///
/// Returns the file contents as a `String` on success.
///
/// # Errors
///
/// Returns `Err` if:
/// - The file metadata cannot be accessed
/// - The file size exceeds the maximum allowed size (configurable via `HEDL_MAX_FILE_SIZE`)
/// - The file cannot be read
/// - The file contains invalid UTF-8
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::read_file;
///
/// # fn main() -> Result<(), String> {
/// // Read a small HEDL file
/// let content = read_file("example.hedl")?;
/// assert!(!content.is_empty());
///
/// // Files larger than the limit will fail
/// std::env::set_var("HEDL_MAX_FILE_SIZE", "1000"); // 1 KB limit
/// let result = read_file("large_file.hedl");
/// assert!(result.is_err());
/// # Ok(())
/// # }
/// ```
///
/// # Security
///
/// This function includes protection against OOM attacks by checking the file
/// size before reading. The maximum file size defaults to 1 GB but can be
/// customized via the `HEDL_MAX_FILE_SIZE` environment variable.
///
/// # Performance
///
/// Uses `fs::metadata()` to check file size before allocating memory, preventing
/// unnecessary memory allocation for oversized files.
pub fn read_file(path: &str) -> Result<String, String> {
    // Check file size first to prevent reading extremely large files
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to get metadata for '{}': {}", path, e))?;

    let max_file_size = get_max_file_size();

    if metadata.len() > max_file_size {
        return Err(format!(
            "File '{}' is too large ({} bytes). Maximum allowed size is {} bytes ({} MB).\n\
             To process larger files, set HEDL_MAX_FILE_SIZE environment variable (in bytes).",
            path,
            metadata.len(),
            max_file_size,
            max_file_size / (1024 * 1024)
        ));
    }

    fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", path, e))
}

/// Write content to a file or stdout.
///
/// Writes the provided content to either a specified file path or to stdout
/// if no path is provided. This is the standard output mechanism used by
/// all HEDL CLI commands.
///
/// # Arguments
///
/// * `content` - The string content to write
/// * `path` - Optional output file path. If `None`, writes to stdout
///
/// # Returns
///
/// Returns `Ok(())` on success.
///
/// # Errors
///
/// Returns `Err` if:
/// - File creation or writing fails (when `path` is `Some`)
/// - Writing to stdout fails (when `path` is `None`)
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::write_output;
///
/// # fn main() -> Result<(), String> {
/// // Write to stdout
/// let hedl_content = "%VERSION: 1.0\n---\nteams: @Team[name]\n  |t1,Team A\n  |t2,Team B";
/// write_output(hedl_content, None)?;
///
/// // Write to file
/// write_output(hedl_content, Some("output.hedl"))?;
/// # Ok(())
/// # }
/// ```
///
/// # Panics
///
/// Does not panic under normal circumstances. All I/O errors are returned
/// as `Err` values.
pub fn write_output(content: &str, path: Option<&str>) -> Result<(), String> {
    match path {
        Some(p) => fs::write(p, content).map_err(|e| format!("Failed to write '{}': {}", p, e)),
        None => io::stdout()
            .write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write to stdout: {}", e)),
    }
}
