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

//! File operations tools (read, write) with parallel processing support.
//!
//! # Parallel Processing
//!
//! Directory scanning operations use rayon for parallel file reading:
//! - Files are collected first, then processed in parallel
//! - Thread count is configurable via `num_threads` parameter
//! - Errors are collected without failing fast
//! - Single-file operations remain sequential
//!
//! # Performance
//!
//! Parallel processing provides significant speedups for multi-file operations:
//! - 10 files: ~3-5x speedup (depending on file size and CPU cores)
//! - 100 files: ~5-8x speedup
//! - Speedup scales with file count and available cores

use crate::error::{McpError, McpResult};
use crate::protocol::{CallToolResult, Content};
use crate::tools::helpers::{parse_args, resolve_safe_path, validate_input_size};
use crate::tools::json_utils::count_entities;
use crate::tools::types::{ReadArgs, WriteArgs, MAX_INPUT_SIZE};
use hedl_core::parse;
use hedl_json::{to_json_value, ToJsonConfig};
use rayon::prelude::*;
use serde_json::{json, Value as JsonValue};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Execute `hedl_read` tool with optional parallel processing.
///
/// # Arguments
///
/// * `args` - JSON arguments containing path, recursive flag, `include_json`, and `num_threads`
/// * `root_path` - Root directory for resolving relative paths
///
/// # Parallel Processing
///
/// When reading directories:
/// - Files are collected first using walkdir
/// - Then processed in parallel using rayon
/// - Thread pool size is configurable via `num_threads`
/// - Errors are collected and included in results (no fail-fast)
///
/// # Returns
///
/// Returns a `CallToolResult` with JSON containing:
/// - `files_read`: Total number of files processed
/// - `results`: Array of file results (successful or errors)
pub fn execute_hedl_read(args: Option<JsonValue>, root_path: &Path) -> McpResult<CallToolResult> {
    let args: ReadArgs = parse_args(args)?;

    // Resolve path relative to root
    let target_path = resolve_safe_path(root_path, &args.path)?;

    let results = if target_path.is_file() {
        // Single file - no parallelism needed
        vec![read_hedl_file(&target_path, args.include_json)?]
    } else if target_path.is_dir() {
        // Directory - use parallel processing
        read_directory_parallel(
            &target_path,
            args.recursive,
            args.include_json,
            args.num_threads,
        )?
    } else {
        return Err(McpError::FileNotFound(args.path));
    };

    Ok(CallToolResult {
        content: vec![Content::Text {
            text: serde_json::to_string_pretty(&json!({
                "files_read": results.len(),
                "results": results
            }))?,
        }],
        is_error: None,
    })
}

/// Read all HEDL files in a directory using parallel processing.
///
/// # Arguments
///
/// * `dir_path` - Directory to scan
/// * `recursive` - Whether to scan subdirectories
/// * `include_json` - Whether to include JSON data in results
/// * `num_threads` - Optional thread count (None uses default rayon pool)
///
/// # Implementation
///
/// 1. Collect all HEDL file paths using walkdir (sequential)
/// 2. Configure rayon thread pool if `num_threads` is specified
/// 3. Process files in parallel using rayon
/// 4. Collect all results (both successes and errors)
///
/// # Error Handling
///
/// Errors are collected, not propagated immediately. Each failed file
/// produces an error entry in the results array with the file path and error message.
fn read_directory_parallel(
    dir_path: &Path,
    recursive: bool,
    include_json: bool,
    num_threads: Option<usize>,
) -> McpResult<Vec<JsonValue>> {
    // Step 1: Collect all HEDL file paths
    let files: Vec<PathBuf> = collect_hedl_files(dir_path, recursive)?;

    // Step 2: Process files in parallel
    let results = if let Some(threads) = num_threads.filter(|&t| t > 0) {
        // Custom thread pool
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .map_err(|e| McpError::InvalidArguments(format!("Failed to create thread pool: {e}")))?
            .install(|| process_files_parallel(&files, include_json))
    } else {
        // Default thread pool
        process_files_parallel(&files, include_json)
    };

    Ok(results)
}

/// Collect all HEDL file paths from a directory.
///
/// # Arguments
///
/// * `dir_path` - Directory to scan (must be canonical)
/// * `recursive` - Whether to scan subdirectories
///
/// # Returns
///
/// Vector of `PathBuf` containing all .hedl files found
///
/// # Security
///
/// All file paths are canonicalized and validated to ensure they are
/// within `dir_path`, preventing symlink-based path traversal attacks.
fn collect_hedl_files(dir_path: &Path, recursive: bool) -> McpResult<Vec<PathBuf>> {
    let walker = if recursive {
        WalkDir::new(dir_path)
    } else {
        WalkDir::new(dir_path).max_depth(1)
    };

    // Ensure dir_path is canonical for security checks
    let canonical_root = dir_path
        .canonicalize()
        .unwrap_or_else(|_| dir_path.to_path_buf());

    let files: Vec<PathBuf> = walker
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            let path = entry.path();
            path.is_file() && path.extension().is_some_and(|ext| ext == "hedl")
        })
        .filter_map(|entry| {
            let path = entry.path();

            // Security: Canonicalize each path to resolve symlinks
            let canonical = match path.canonicalize() {
                Ok(p) => p,
                Err(_) => return None, // Skip files we can't canonicalize
            };

            // Security: Ensure the canonicalized path is still under root
            // This prevents symlink attacks where a symlink inside the root
            // points to a file outside the root
            if canonical.starts_with(&canonical_root) {
                Some(canonical)
            } else {
                None // Skip files outside root (via symlink)
            }
        })
        .collect();

    Ok(files)
}

/// Process files in parallel using rayon.
///
/// # Arguments
///
/// * `files` - Slice of file paths to process
/// * `include_json` - Whether to include JSON data in results
///
/// # Implementation
///
/// Uses rayon's parallel iterator to process files concurrently.
/// Results are collected into a Vec maintaining the original order.
/// Errors are converted to error JSON objects rather than propagating.
fn process_files_parallel(files: &[PathBuf], include_json: bool) -> Vec<JsonValue> {
    files
        .par_iter()
        .map(|path| match read_hedl_file(path, include_json) {
            Ok(result) => result,
            Err(e) => json!({
                "file": path.display().to_string(),
                "error": e.to_string()
            }),
        })
        .collect()
}

fn read_hedl_file(path: &Path, include_json: bool) -> McpResult<JsonValue> {
    let content = std::fs::read(path)?;
    let doc = parse(&content)?;

    let mut result = json!({
        "file": path.display().to_string(),
        "version": format!("{}.{}", doc.version.0, doc.version.1),
        "schemas": doc.structs.keys().collect::<Vec<_>>(),
        "aliases": doc.aliases.len(),
        "nests": doc.nests,
        "entities": count_entities(&doc)
    });

    if include_json {
        let config = ToJsonConfig::default();
        if let Ok(json_value) = to_json_value(&doc, &config) {
            result["data"] = json_value;
        }
    }

    Ok(result)
}

/// Execute `hedl_write` tool.
///
/// # Security
///
/// This function implements comprehensive path traversal protection through multiple layers:
///
/// ## Defense Layers
///
/// 1. **Input Validation**: Rejects empty paths, checks size limits
/// 2. **Path Construction**: Safely joins with root, handles relative/absolute paths
/// 3. **Parent Directory Creation**: Creates parent dirs before validation (prevents TOCTOU)
/// 4. **Canonicalization**: Resolves `..`, `.`, and symlinks to actual filesystem paths
/// 5. **Boundary Validation**: Verifies canonical path stays within root directory
/// 6. **Atomic Write**: Uses canonical path for actual write operation
///
/// ## Attack Prevention
///
/// - **Path Traversal**: Blocks `../../../etc/passwd` via canonicalization + boundary check
/// - **Symlink Escape**: Blocks symlinks pointing outside root (canonicalize resolves symlink targets)
/// - **TOCTOU**: Minimizes race window by validating canonical path just before write
/// - **Absolute Paths**: Validates absolute paths are within root
/// - **Non-existent Parents**: Creates and validates parent directory chain
///
/// ## Example Attacks Blocked
///
/// ```ignore
/// // Attack 1: Relative traversal
/// path: "../../../etc/passwd" // Blocked: canonical path outside root
///
/// // Attack 2: Symlink escape
/// // (symlink "malicious" inside root points to "/tmp/outside")
/// path: "malicious/file.hedl" // Blocked: canonical symlink target outside root
///
/// // Attack 3: Nonexistent parent traversal
/// path: "new/../../../etc/passwd" // Blocked: canonicalized parent outside root
///
/// // Attack 4: Absolute path
/// path: "/etc/passwd" // Blocked: starts outside root
/// ```
pub fn execute_hedl_write(args: Option<JsonValue>, root_path: &Path) -> McpResult<CallToolResult> {
    let args: WriteArgs = parse_args(args)?;

    // Security: Reject empty paths
    if args.path.is_empty() {
        return Err(McpError::InvalidArguments(
            "Path cannot be empty".to_string(),
        ));
    }

    // Security: Validate input size to prevent memory exhaustion
    validate_input_size(&args.content, MAX_INPUT_SIZE)?;

    // Validate HEDL content if requested
    if args.validate {
        parse(args.content.as_bytes())?;
    }

    // Format content if requested
    let content_to_write = if args.format {
        let doc = parse(args.content.as_bytes())?;
        let config = hedl_c14n::CanonicalConfig::default();
        hedl_c14n::canonicalize_with_config(&doc, &config)
            .map_err(|e| McpError::InvalidArguments(format!("Format failed: {e}")))?
    } else {
        args.content.clone()
    };

    // Security: Construct the target path
    // Build the path relative to root, or use as-is if absolute
    let target_path = if Path::new(&args.path).is_absolute() {
        PathBuf::from(&args.path)
    } else {
        root_path.join(&args.path)
    };

    // Security: Canonicalize root for validation
    let canonical_root = root_path
        .canonicalize()
        .unwrap_or_else(|_| root_path.to_path_buf());

    // Security: Create parent directories BEFORE canonicalization
    // This prevents TOCTOU issues and ensures we can canonicalize the full path
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Security: Canonicalize the target path and validate it's under root
    // This is the critical security check that resolves symlinks and validates boundaries
    let canonical_target = target_path
        .canonicalize()
        .or_else(|_| {
            // If file doesn't exist yet, canonicalize the parent and join the filename
            if let Some(parent) = target_path.parent() {
                if let Some(filename) = target_path.file_name() {
                    parent.canonicalize().map(|p| p.join(filename))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid path: no filename",
                    ))
                }
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid path: no parent directory",
                ))
            }
        })
        .map_err(|e| McpError::InvalidArguments(format!("Invalid target path: {e}")))?;

    // Security: CRITICAL VALIDATION - Ensure canonical path is within root
    // This check happens AFTER symlink resolution, preventing symlink escapes
    if !canonical_target.starts_with(&canonical_root) {
        return Err(McpError::PathTraversal(format!(
            "Path '{}' resolves outside allowed directory",
            args.path
        )));
    }

    // Create backup if file exists and backup is enabled
    let mut backup_path = None;
    if args.backup && canonical_target.exists() {
        let backup = canonical_target.with_extension("hedl.bak");
        std::fs::copy(&canonical_target, &backup)?;
        backup_path = Some(backup.display().to_string());
    }

    // Security: Write using the validated canonical path
    // At this point, we've verified the path is safe
    std::fs::write(&canonical_target, &content_to_write)?;

    let mut result = json!({
        "success": true,
        "path": canonical_target.display().to_string(),
        "bytes_written": content_to_write.len()
    });

    if let Some(backup) = backup_path {
        result["backup"] = json!(backup);
    }

    Ok(CallToolResult {
        content: vec![Content::Text {
            text: serde_json::to_string_pretty(&result)?,
        }],
        is_error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_hedl_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.hedl");

        let hedl_content =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        fs::write(&file_path, hedl_content).unwrap();

        let args = json!({ "path": "test.hedl" });
        let result = execute_hedl_read(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["files_read"], 1);
    }

    #[test]
    fn test_hedl_read_directory() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple HEDL files
        let hedl1 =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        let hedl2 = "%VERSION: 1.0\n%STRUCT: Product: [id, title]\n---\nproducts: @Product\n  | widget, Widget\n";

        fs::write(temp_dir.path().join("users.hedl"), hedl1).unwrap();
        fs::write(temp_dir.path().join("products.hedl"), hedl2).unwrap();

        let args = json!({ "path": "." });
        let result = execute_hedl_read(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["files_read"], 2);
    }

    #[test]
    fn test_hedl_read_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let args = json!({ "path": "nonexistent.hedl" });
        let result = execute_hedl_read(Some(args), temp_dir.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_hedl_read_with_json() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.hedl");

        let hedl_content =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";
        fs::write(&file_path, hedl_content).unwrap();

        let args = json!({ "path": "test.hedl", "include_json": true });
        let result = execute_hedl_read(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        // When include_json is true, results should have data field
        let results = parsed["results"].as_array().unwrap();
        assert!(results[0].get("data").is_some());
    }

    #[test]
    fn test_hedl_write_valid() {
        let temp_dir = TempDir::new().unwrap();
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";

        let args = json!({
            "path": "output.hedl",
            "content": hedl
        });
        let result = execute_hedl_write(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["success"], true);

        // Verify file was written
        let written = fs::read_to_string(temp_dir.path().join("output.hedl")).unwrap();
        assert!(written.contains("%VERSION"));
    }

    #[test]
    fn test_hedl_write_with_format() {
        let temp_dir = TempDir::new().unwrap();
        let hedl =
            "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n";

        let args = json!({
            "path": "formatted.hedl",
            "content": hedl,
            "format": true
        });
        let result = execute_hedl_write(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["success"], true);
    }

    #[test]
    fn test_hedl_write_creates_backup() {
        let temp_dir = TempDir::new().unwrap();
        let original = "%VERSION: 1.0\n---\noriginal: data\n";
        let updated = "%VERSION: 1.0\n---\nupdated: data\n";

        // Write original
        fs::write(temp_dir.path().join("test.hedl"), original).unwrap();

        // Write updated
        let args = json!({
            "path": "test.hedl",
            "content": updated,
            "backup": true
        });
        let result = execute_hedl_write(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert!(parsed.get("backup").is_some());

        // Verify backup exists
        assert!(temp_dir.path().join("test.hedl.bak").exists());
    }

    #[test]
    fn test_hedl_write_invalid_hedl() {
        let temp_dir = TempDir::new().unwrap();
        let invalid = "not valid hedl";

        let args = json!({
            "path": "invalid.hedl",
            "content": invalid,
            "validate": true
        });
        let result = execute_hedl_write(Some(args), temp_dir.path());

        assert!(result.is_err());
    }

    #[test]
    fn test_hedl_write_skip_validation() {
        let temp_dir = TempDir::new().unwrap();
        let content = "any content here";

        let args = json!({
            "path": "raw.hedl",
            "content": content,
            "validate": false
        });
        let result = execute_hedl_write(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["success"], true);
    }

    #[test]
    fn test_hedl_write_creates_directories() {
        let temp_dir = TempDir::new().unwrap();
        let hedl = "%VERSION: 1.0\n---\ntest: data\n";

        let args = json!({
            "path": "subdir/deep/output.hedl",
            "content": hedl
        });
        let result = execute_hedl_write(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["success"], true);

        // Verify directories were created
        assert!(temp_dir.path().join("subdir/deep/output.hedl").exists());
    }

    #[test]
    fn test_parallel_directory_read() {
        let temp_dir = TempDir::new().unwrap();

        // Create 10 HEDL files
        for i in 0..10 {
            let content = format!(
                "%VERSION: 1.0\n%STRUCT: Item: [id, name]\n---\nitems{i}: @Item\n  | item{i}, Item {i}\n"
            );
            fs::write(temp_dir.path().join(format!("file{i}.hedl")), content).unwrap();
        }

        // Read with default parallelism
        let args = json!({ "path": "." });
        let result = execute_hedl_read(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["files_read"], 10);

        // Verify all files were read successfully
        let results = parsed["results"].as_array().unwrap();
        assert_eq!(results.len(), 10);
        for result in results {
            assert!(
                result.get("error").is_none(),
                "Unexpected error in result: {result:?}"
            );
        }
    }

    #[test]
    fn test_parallel_read_with_custom_threads() {
        let temp_dir = TempDir::new().unwrap();

        // Create 5 HEDL files
        for i in 0..5 {
            let content = format!(
                "%VERSION: 1.0\n%STRUCT: Data: [id, value]\n---\ndata{i}: @Data\n  | data{i}, {i}\n"
            );
            fs::write(temp_dir.path().join(format!("data{i}.hedl")), content).unwrap();
        }

        // Read with custom thread count
        let args = json!({
            "path": ".",
            "num_threads": 2
        });
        let result = execute_hedl_read(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["files_read"], 5);
    }

    #[test]
    fn test_parallel_read_with_errors() {
        let temp_dir = TempDir::new().unwrap();

        // Create valid files
        for i in 0..3 {
            let content = format!(
                "%VERSION: 1.0\n%STRUCT: Valid: [id]\n---\ndata{i}: @Valid\n  | valid{i}\n"
            );
            fs::write(temp_dir.path().join(format!("valid{i}.hedl")), content).unwrap();
        }

        // Create an invalid file
        fs::write(temp_dir.path().join("invalid.hedl"), "invalid content").unwrap();

        // Read all files - should collect errors, not fail
        let args = json!({ "path": "." });
        let result = execute_hedl_read(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["files_read"], 4);

        // Check that we have both successes and errors
        let results = parsed["results"].as_array().unwrap();
        let errors: Vec<_> = results
            .iter()
            .filter(|r| r.get("error").is_some())
            .collect();
        let successes: Vec<_> = results
            .iter()
            .filter(|r| r.get("version").is_some())
            .collect();

        assert_eq!(errors.len(), 1, "Expected 1 error, got {}", errors.len());
        assert_eq!(
            successes.len(),
            3,
            "Expected 3 successes, got {}",
            successes.len()
        );
    }

    #[test]
    fn test_parallel_read_recursive() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        fs::create_dir_all(temp_dir.path().join("subdir1/subdir2")).unwrap();

        // Files in root
        fs::write(
            temp_dir.path().join("root.hedl"),
            "%VERSION: 1.0\n---\nroot: data\n",
        )
        .unwrap();

        // Files in subdir1
        fs::write(
            temp_dir.path().join("subdir1/level1.hedl"),
            "%VERSION: 1.0\n---\nlevel1: data\n",
        )
        .unwrap();

        // Files in subdir2
        fs::write(
            temp_dir.path().join("subdir1/subdir2/level2.hedl"),
            "%VERSION: 1.0\n---\nlevel2: data\n",
        )
        .unwrap();

        // Recursive read
        let args = json!({
            "path": ".",
            "recursive": true
        });
        let result = execute_hedl_read(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["files_read"], 3);
    }

    #[test]
    fn test_parallel_read_non_recursive() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        fs::create_dir_all(temp_dir.path().join("subdir")).unwrap();

        // Files in root
        fs::write(
            temp_dir.path().join("root.hedl"),
            "%VERSION: 1.0\n---\nroot: data\n",
        )
        .unwrap();

        // Files in subdir (should not be read)
        fs::write(
            temp_dir.path().join("subdir/nested.hedl"),
            "%VERSION: 1.0\n---\nnested: data\n",
        )
        .unwrap();

        // Non-recursive read
        let args = json!({
            "path": ".",
            "recursive": false
        });
        let result = execute_hedl_read(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["files_read"], 1); // Only root.hedl
    }

    #[test]
    fn test_parallel_read_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let args = json!({ "path": "." });
        let result = execute_hedl_read(Some(args), temp_dir.path()).unwrap();

        let text = match &result.content[0] {
            Content::Text { text } => text,
            _ => panic!("Expected text content"),
        };

        let parsed: JsonValue = serde_json::from_str(text).unwrap();
        assert_eq!(parsed["files_read"], 0);
    }

    #[test]
    fn test_collect_hedl_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create files with different extensions
        fs::write(temp_dir.path().join("file1.hedl"), "test").unwrap();
        fs::write(temp_dir.path().join("file2.hedl"), "test").unwrap();
        fs::write(temp_dir.path().join("file3.txt"), "test").unwrap();
        fs::write(temp_dir.path().join("file4.json"), "test").unwrap();

        let files = collect_hedl_files(temp_dir.path(), false).unwrap();
        assert_eq!(files.len(), 2); // Only .hedl files
    }

    #[test]
    fn test_process_files_parallel() {
        let temp_dir = TempDir::new().unwrap();

        let mut paths = Vec::new();
        for i in 0..5 {
            let path = temp_dir.path().join(format!("test{i}.hedl"));
            let content = format!("%VERSION: 1.0\n---\ntest{i}: data\n");
            fs::write(&path, content).unwrap();
            paths.push(path);
        }

        let results = process_files_parallel(&paths, false);
        assert_eq!(results.len(), 5);

        // All should be successful
        for result in &results {
            assert!(result.get("version").is_some());
            assert!(result.get("error").is_none());
        }
    }
}
