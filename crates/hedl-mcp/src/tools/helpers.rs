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

//! Helper functions shared across MCP tools.

use crate::error::{McpError, McpResult};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::path::{Path, PathBuf};

/// Parse JSON arguments into a typed structure.
pub fn parse_args<T: for<'de> Deserialize<'de>>(args: Option<JsonValue>) -> McpResult<T> {
    let args = args.unwrap_or(JsonValue::Object(serde_json::Map::new()));
    serde_json::from_value(args).map_err(|e| McpError::InvalidArguments(e.to_string()))
}

/// Resolve a path relative to root and ensure it doesn't escape the root directory.
///
/// # Security
///
/// This function implements comprehensive path traversal protection:
///
/// 1. **Canonicalization**: Resolves `..`, `.`, and symlinks to the actual filesystem path
/// 2. **Boundary Check**: Verifies the canonical path is within the root directory
/// 3. **Symlink Resolution**: Follows symlinks and validates their targets are within bounds
/// 4. **Attack Prevention**: Blocks attempts to escape via:
///    - Relative path traversal (`../../../etc/passwd`)
///    - Absolute paths outside root (`/etc/passwd`)
///    - Symlink escapes (symlink inside root pointing outside)
///
/// # Arguments
///
/// * `root` - Root directory that confines all file operations
/// * `path` - Requested path (relative or absolute)
///
/// # Returns
///
/// * `Ok(PathBuf)` - Canonical path within root
/// * `Err(McpError::FileNotFound)` - Path does not exist
/// * `Err(McpError::PathTraversal)` - Path escapes root directory
///
/// # Examples
///
/// ```ignore
/// // Safe: file inside root
/// resolve_safe_path(root, "data/file.hedl") // OK
///
/// // Blocked: traversal attack
/// resolve_safe_path(root, "../../../etc/passwd") // Err(PathTraversal)
///
/// // Blocked: symlink escape
/// // (symlink inside root -> file outside root)
/// resolve_safe_path(root, "escape_link") // Err(PathTraversal)
/// ```
pub fn resolve_safe_path(root: &Path, path: &str) -> McpResult<PathBuf> {
    // Security: Reject empty paths early
    if path.is_empty() {
        return Err(McpError::InvalidArguments(
            "Path cannot be empty".to_string(),
        ));
    }

    let path = Path::new(path);

    // Security: Check for obvious path traversal patterns BEFORE resolving
    // This catches traversal attempts even when target doesn't exist
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        // Check if path would escape root by normalizing components
        let mut depth: i32 = 0;
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    depth -= 1;
                    if depth < 0 {
                        return Err(McpError::PathTraversal(path.display().to_string()));
                    }
                }
                std::path::Component::Normal(_) => {
                    depth += 1;
                }
                _ => {}
            }
        }
    }

    // Security: Reject absolute paths outside root
    if path.is_absolute() {
        let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        if !path.starts_with(&canonical_root) {
            return Err(McpError::PathTraversal(path.display().to_string()));
        }
    }

    // Build the resolved path
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };

    // Security: Canonicalize to resolve .., ., and symlinks
    // This is the critical security operation that prevents traversal
    let canonical = resolved
        .canonicalize()
        .map_err(|_| McpError::FileNotFound(path.display().to_string()))?;

    // Security: Canonicalize root for comparison
    // Use unwrap_or to handle the case where root doesn't exist yet
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());

    // Security: Verify the canonical path is within root
    // This check happens AFTER symlink resolution, so symlink escapes are blocked
    if !canonical.starts_with(&canonical_root) {
        return Err(McpError::PathTraversal(path.display().to_string()));
    }

    Ok(canonical)
}

/// Simple token estimation (approximate GPT-4 cl100k tokenization).
/// Uses whitespace and punctuation splitting as a rough approximation.
#[must_use]
pub fn estimate_tokens(text: &str) -> usize {
    // Rough approximation: ~4 characters per token on average
    // More accurate for code/structured data
    let char_count = text.len();
    let whitespace_count = text.chars().filter(|c| c.is_whitespace()).count();
    let punct_count = text.chars().filter(char::is_ascii_punctuation).count();

    // Estimate based on character types
    (char_count + whitespace_count + punct_count) / 4
}

/// Validate input size to prevent memory exhaustion.
pub fn validate_input_size(input: &str, max_size: usize) -> McpResult<()> {
    if input.len() > max_size {
        return Err(McpError::ResourceLimit(format!(
            "Input size exceeds maximum: {} bytes (max: {} bytes)",
            input.len(),
            max_size
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_estimate_tokens_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_estimate_tokens_simple() {
        let tokens = estimate_tokens("hello world");
        assert!(tokens > 0);
        assert!(tokens < 10); // Should be roughly 2-3 tokens
    }

    #[test]
    fn test_estimate_tokens_code() {
        let code = r#"function foo() { return "bar"; }"#;
        let tokens = estimate_tokens(code);
        assert!(tokens > 5); // Code has punctuation
    }

    #[test]
    fn test_resolve_safe_path_relative() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "test").unwrap();

        let resolved = resolve_safe_path(temp_dir.path(), "test.txt").unwrap();
        assert!(resolved.ends_with("test.txt"));
    }

    #[test]
    fn test_resolve_safe_path_traversal() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file in the temp dir
        fs::write(temp_dir.path().join("test.txt"), "test").unwrap();

        // Try to access parent directory (path traversal)
        let result = resolve_safe_path(temp_dir.path(), "../../../etc/passwd");

        // Should fail because it's outside root
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_safe_path_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let result = resolve_safe_path(temp_dir.path(), "nonexistent.txt");

        assert!(result.is_err());
        assert!(matches!(result, Err(McpError::FileNotFound(_))));
    }

    #[test]
    fn test_parse_args_valid() {
        #[derive(Debug, serde::Deserialize, PartialEq)]
        struct TestArgs {
            name: String,
            count: i32,
        }

        let args = json!({ "name": "test", "count": 42 });
        let parsed: TestArgs = parse_args(Some(args)).unwrap();

        assert_eq!(parsed.name, "test");
        assert_eq!(parsed.count, 42);
    }

    #[test]
    fn test_parse_args_missing_required() {
        #[derive(Debug, serde::Deserialize)]
        #[allow(dead_code)]
        struct TestArgs {
            required_field: String,
        }

        let args = json!({});
        let result: McpResult<TestArgs> = parse_args(Some(args));

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_none() {
        #[derive(Debug, serde::Deserialize, Default)]
        struct TestArgs {
            #[serde(default)]
            optional: String,
        }

        let parsed: TestArgs = parse_args(None).unwrap();
        assert_eq!(parsed.optional, "");
    }

    #[test]
    fn test_validate_input_size_ok() {
        let input = "small input";
        assert!(validate_input_size(input, 1024).is_ok());
    }

    #[test]
    fn test_validate_input_size_too_large() {
        let input = "x".repeat(1000);
        let result = validate_input_size(&input, 100);
        assert!(result.is_err());
    }
}
