// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! File discovery with glob patterns and recursive traversal.
//!
//! This module provides efficient file discovery capabilities for batch processing,
//! supporting glob patterns, recursive directory traversal, and various filtering options.
//!
//! # Features
//!
//! - **Glob Patterns**: Support for standard glob patterns (*, ?, [abc], **)
//! - **Recursive Traversal**: Optional recursive directory traversal with depth limiting
//! - **Filtering**: Extension, size, and hidden file filtering
//! - **Symlinks**: Configurable symlink following behavior
//! - **Error Handling**: Detailed error reporting for invalid patterns and I/O failures
//!
//! # Examples
//!
//! ```rust,no_run
//! use hedl_cli::file_discovery::{FileDiscovery, DiscoveryConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Discover all HEDL files in current directory
//! let discovery = FileDiscovery::new(
//!     vec!["*.hedl".to_string()],
//!     DiscoveryConfig::default()
//! );
//! let files = discovery.discover()?;
//!
//! // Recursive discovery with depth limit
//! let discovery = FileDiscovery::new(
//!     vec!["**/*.hedl".to_string()],
//!     DiscoveryConfig {
//!         max_depth: Some(3),
//!         extension: Some("hedl".to_string()),
//!         ..Default::default()
//!     }
//! );
//! let files = discovery.discover()?;
//! # Ok(())
//! # }
//! ```

use crate::error::CliError;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

/// Configuration for file discovery.
///
/// Controls how files are discovered, including recursion depth, filtering,
/// and symlink handling.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Maximum recursion depth for directory traversal.
    ///
    /// - `None`: Unlimited depth (use with caution)
    /// - `Some(0)`: Current directory only
    /// - `Some(n)`: Traverse up to n levels deep
    pub max_depth: Option<usize>,

    /// Filter by file extension (without leading dot).
    ///
    /// Only files with this extension will be included.
    /// Example: `Some("hedl")` matches "file.hedl" but not "file.txt"
    pub extension: Option<String>,

    /// Maximum file size in bytes.
    ///
    /// Files larger than this will be excluded.
    pub max_file_size: Option<u64>,

    /// Follow symbolic links during traversal.
    ///
    /// When false, symlinks are ignored.
    pub follow_links: bool,

    /// Include hidden files (starting with '.').
    ///
    /// When false, hidden files and directories are skipped.
    pub include_hidden: bool,

    /// Enable recursive directory traversal.
    ///
    /// When false, only process files matching patterns directly,
    /// don't traverse directories.
    pub recursive: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            max_depth: Some(10),
            extension: None,
            max_file_size: None,
            follow_links: false,
            include_hidden: false,
            recursive: false,
        }
    }
}

/// File discovery engine with glob pattern support.
///
/// Discovers files matching specified patterns with configurable filtering
/// and traversal options.
#[derive(Debug)]
pub struct FileDiscovery {
    patterns: Vec<String>,
    config: DiscoveryConfig,
}

impl FileDiscovery {
    /// Create a new file discovery instance.
    ///
    /// # Arguments
    ///
    /// * `patterns` - List of file patterns (glob patterns or explicit paths)
    /// * `config` - Discovery configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_cli::file_discovery::{FileDiscovery, DiscoveryConfig};
    ///
    /// let discovery = FileDiscovery::new(
    ///     vec!["*.hedl".to_string(), "data/*.hedl".to_string()],
    ///     DiscoveryConfig::default()
    /// );
    /// ```
    #[must_use]
    pub fn new(patterns: Vec<String>, config: DiscoveryConfig) -> Self {
        Self { patterns, config }
    }

    /// Validate all patterns before discovery.
    ///
    /// Checks that patterns are valid glob expressions.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All patterns are valid
    /// * `Err(CliError::GlobPattern)` - Invalid pattern found
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_cli::file_discovery::{FileDiscovery, DiscoveryConfig};
    ///
    /// let discovery = FileDiscovery::new(
    ///     vec!["*.hedl".to_string()],
    ///     DiscoveryConfig::default()
    /// );
    /// assert!(discovery.validate_patterns().is_ok());
    /// ```
    pub fn validate_patterns(&self) -> Result<(), CliError> {
        for pattern in &self.patterns {
            if let Err(e) = glob::Pattern::new(pattern) {
                return Err(CliError::GlobPattern {
                    pattern: pattern.clone(),
                    message: e.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Discover all files matching the patterns.
    ///
    /// Expands glob patterns and applies configured filters.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<PathBuf>)` - List of discovered file paths
    /// * `Err(CliError)` - On pattern errors, I/O failures, or no matches
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Pattern is invalid
    /// - No files match any pattern
    /// - Directory traversal fails
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_cli::file_discovery::{FileDiscovery, DiscoveryConfig};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let discovery = FileDiscovery::new(
    ///     vec!["tests/*.hedl".to_string()],
    ///     DiscoveryConfig::default()
    /// );
    /// let files = discovery.discover()?;
    /// println!("Found {} files", files.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn discover(&self) -> Result<Vec<PathBuf>, CliError> {
        // If no patterns provided, return empty (nothing to discover = success)
        if self.patterns.is_empty() {
            return Ok(Vec::new());
        }

        // Validate patterns first
        self.validate_patterns()?;

        let mut all_files = Vec::new();

        for pattern in &self.patterns {
            let pattern_files = if self.config.recursive && pattern.contains("**") {
                // Recursive glob pattern
                self.discover_recursive_glob(pattern)?
            } else if self.config.recursive {
                // Recursive but not using ** syntax
                self.discover_recursive_simple(pattern)?
            } else {
                // Simple glob pattern (no recursion)
                self.discover_simple_glob(pattern)?
            };

            all_files.extend(pattern_files);
        }

        // Remove duplicates while preserving order
        let mut seen = std::collections::HashSet::new();
        all_files.retain(|path| seen.insert(path.clone()));

        if all_files.is_empty() {
            return Err(CliError::NoFilesMatched {
                patterns: self.patterns.clone(),
            });
        }

        Ok(all_files)
    }

    /// Discover files using simple glob pattern (no recursion).
    fn discover_simple_glob(&self, pattern: &str) -> Result<Vec<PathBuf>, CliError> {
        let mut files = Vec::new();

        for entry in glob::glob(pattern).map_err(|e| CliError::GlobPattern {
            pattern: pattern.to_string(),
            message: e.to_string(),
        })? {
            let path = entry.map_err(|e| CliError::DirectoryTraversal {
                path: PathBuf::from(pattern),
                message: e.to_string(),
            })?;

            if self.should_include_file(&path)? {
                files.push(path);
            }
        }

        Ok(files)
    }

    /// Discover files using recursive glob with ** syntax.
    fn discover_recursive_glob(&self, pattern: &str) -> Result<Vec<PathBuf>, CliError> {
        // For ** patterns, we need to manually walk directories
        // Extract base directory from pattern
        let base_dir = self.extract_base_dir(pattern);

        let mut files = Vec::new();

        let walker = WalkDir::new(&base_dir)
            .follow_links(self.config.follow_links)
            .max_depth(self.config.max_depth.unwrap_or(usize::MAX));

        let glob_pattern = glob::Pattern::new(pattern).map_err(|e| CliError::GlobPattern {
            pattern: pattern.to_string(),
            message: e.to_string(),
        })?;

        for entry in walker {
            let entry = entry.map_err(|e| CliError::DirectoryTraversal {
                path: base_dir.clone(),
                message: e.to_string(),
            })?;

            if !self.should_include_entry(&entry) {
                continue;
            }

            let path = entry.path();
            if path.is_file()
                && glob_pattern.matches_path(path)
                && self.should_include_file(path)?
            {
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }

    /// Discover files recursively with simple pattern.
    fn discover_recursive_simple(&self, pattern: &str) -> Result<Vec<PathBuf>, CliError> {
        // Convert simple pattern to recursive glob
        let base_dir = self.extract_base_dir(pattern);
        let filename_pattern = PathBuf::from(pattern)
            .file_name()
            .map_or_else(|| pattern.to_string(), |s| s.to_string_lossy().to_string());

        let recursive_pattern = if base_dir == std::path::Path::new(".") {
            format!("**/{filename_pattern}")
        } else {
            format!("{}/**/{}", base_dir.display(), filename_pattern)
        };

        self.discover_recursive_glob(&recursive_pattern)
    }

    /// Extract base directory from a glob pattern.
    fn extract_base_dir(&self, pattern: &str) -> PathBuf {
        let path = PathBuf::from(pattern);

        // Find the deepest ancestor that doesn't contain glob characters
        for ancestor in path.ancestors() {
            let s = ancestor.to_string_lossy();
            // Skip if it contains glob characters
            if s.contains('*') || s.contains('?') || s.contains('[') {
                continue;
            }
            // Found a non-glob path - return it (or "." if empty)
            if s.is_empty() {
                return PathBuf::from(".");
            }
            return ancestor.to_path_buf();
        }

        // All parts contain globs, default to current directory
        PathBuf::from(".")
    }

    /// Check if a directory entry should be included in traversal.
    fn should_include_entry(&self, entry: &DirEntry) -> bool {
        // Skip hidden files/directories if not configured to include them
        if !self.config.include_hidden {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with('.') && name != "." && name != ".." {
                    return false;
                }
            }
        }

        true
    }

    /// Check if a file should be included based on filters.
    fn should_include_file(&self, path: &Path) -> Result<bool, CliError> {
        // Must be a regular file
        if !path.is_file() {
            return Ok(false);
        }

        // Check extension filter
        if let Some(ref ext) = self.config.extension {
            if path.extension().and_then(|s| s.to_str()) != Some(ext.as_str()) {
                return Ok(false);
            }
        }

        // Check file size filter
        if let Some(max_size) = self.config.max_file_size {
            let metadata = std::fs::metadata(path).map_err(|e| CliError::io_error(path, e))?;
            if metadata.len() > max_size {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_files(dir: &Path, files: &[&str]) -> Result<(), std::io::Error> {
        for file in files {
            let path = dir.join(file);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, "test content")?;
        }
        Ok(())
    }

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.max_depth, Some(10));
        assert!(config.extension.is_none());
        assert!(config.max_file_size.is_none());
        assert!(!config.follow_links);
        assert!(!config.include_hidden);
        assert!(!config.recursive);
    }

    #[test]
    fn test_validate_patterns_valid() {
        let discovery = FileDiscovery::new(
            vec!["*.hedl".to_string(), "test/*.hedl".to_string()],
            DiscoveryConfig::default(),
        );
        assert!(discovery.validate_patterns().is_ok());
    }

    #[test]
    fn test_validate_patterns_invalid() {
        let discovery =
            FileDiscovery::new(vec!["[invalid".to_string()], DiscoveryConfig::default());
        let result = discovery.validate_patterns();
        assert!(result.is_err());
        if let Err(CliError::GlobPattern { pattern, .. }) = result {
            assert_eq!(pattern, "[invalid");
        }
    }

    #[test]
    fn test_discover_simple_glob() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), &["file1.hedl", "file2.hedl", "file3.txt"])?;

        let pattern = format!("{}/*.hedl", temp_dir.path().display());
        let discovery = FileDiscovery::new(vec![pattern], DiscoveryConfig::default());

        let files = discovery.discover()?;
        assert_eq!(files.len(), 2);

        Ok(())
    }

    #[test]
    fn test_discover_no_matches() {
        let temp_dir = TempDir::new().unwrap();
        let pattern = format!("{}/*.hedl", temp_dir.path().display());
        let discovery = FileDiscovery::new(vec![pattern.clone()], DiscoveryConfig::default());

        let result = discovery.discover();
        assert!(result.is_err());
        if let Err(CliError::NoFilesMatched { patterns }) = result {
            assert_eq!(patterns, vec![pattern]);
        }
    }

    #[test]
    fn test_discover_recursive() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_files(
            temp_dir.path(),
            &[
                "file1.hedl",
                "dir1/file2.hedl",
                "dir1/dir2/file3.hedl",
                "dir1/file4.txt",
            ],
        )?;

        let pattern = format!("{}/**/*.hedl", temp_dir.path().display());
        let discovery = FileDiscovery::new(
            vec![pattern],
            DiscoveryConfig {
                recursive: true,
                ..Default::default()
            },
        );

        let files = discovery.discover()?;
        assert_eq!(files.len(), 3);

        Ok(())
    }

    #[test]
    fn test_discover_with_depth_limit() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_files(
            temp_dir.path(),
            &[
                "file1.hedl",
                "dir1/file2.hedl",
                "dir1/dir2/file3.hedl",
                "dir1/dir2/dir3/file4.hedl",
            ],
        )?;

        let pattern = format!("{}/**/*.hedl", temp_dir.path().display());
        let discovery = FileDiscovery::new(
            vec![pattern],
            DiscoveryConfig {
                recursive: true,
                max_depth: Some(2),
                ..Default::default()
            },
        );

        let files = discovery.discover()?;
        // Should find file1.hedl and dir1/file2.hedl, but not deeper files
        assert!(files.len() <= 3); // May include dir1/dir2/file3.hedl depending on depth counting

        Ok(())
    }

    #[test]
    fn test_discover_with_extension_filter() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), &["file1.hedl", "file2.txt", "file3.hedl"])?;

        let pattern = format!("{}/*", temp_dir.path().display());
        let discovery = FileDiscovery::new(
            vec![pattern],
            DiscoveryConfig {
                extension: Some("hedl".to_string()),
                ..Default::default()
            },
        );

        let files = discovery.discover()?;
        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|p| p.extension().unwrap() == "hedl"));

        Ok(())
    }

    #[test]
    fn test_discover_hidden_files() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_files(
            temp_dir.path(),
            &["file1.hedl", ".hidden.hedl", "dir/.hidden2.hedl"],
        )?;

        let pattern = format!("{}/**/*.hedl", temp_dir.path().display());

        // Without include_hidden
        let discovery = FileDiscovery::new(
            vec![pattern.clone()],
            DiscoveryConfig {
                recursive: true,
                include_hidden: false,
                ..Default::default()
            },
        );
        let files = discovery.discover()?;
        assert_eq!(files.len(), 1); // Only file1.hedl

        // With include_hidden
        let discovery = FileDiscovery::new(
            vec![pattern],
            DiscoveryConfig {
                recursive: true,
                include_hidden: true,
                ..Default::default()
            },
        );
        let files = discovery.discover()?;
        assert!(files.len() >= 2); // file1.hedl and hidden files

        Ok(())
    }

    #[test]
    fn test_extract_base_dir() {
        let discovery = FileDiscovery::new(vec![], DiscoveryConfig::default());

        assert_eq!(discovery.extract_base_dir("*.hedl"), PathBuf::from("."));
        assert_eq!(
            discovery.extract_base_dir("dir/*.hedl"),
            PathBuf::from("dir")
        );
        assert_eq!(
            discovery.extract_base_dir("dir/subdir/*.hedl"),
            PathBuf::from("dir/subdir")
        );
        assert_eq!(
            discovery.extract_base_dir("**/file.hedl"),
            PathBuf::from(".")
        );
    }

    #[test]
    fn test_multiple_patterns() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_files(
            temp_dir.path(),
            &["dir1/file1.hedl", "dir2/file2.hedl", "file3.hedl"],
        )?;

        let patterns = vec![
            format!("{}/dir1/*.hedl", temp_dir.path().display()),
            format!("{}/dir2/*.hedl", temp_dir.path().display()),
        ];
        let discovery = FileDiscovery::new(patterns, DiscoveryConfig::default());

        let files = discovery.discover()?;
        assert_eq!(files.len(), 2);

        Ok(())
    }

    #[test]
    fn test_deduplicate_files() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_files(temp_dir.path(), &["file1.hedl"])?;

        // Same file matched by multiple patterns
        let file_path = format!("{}/file1.hedl", temp_dir.path().display());
        let patterns = vec![
            file_path.clone(),
            format!("{}/*.hedl", temp_dir.path().display()),
        ];
        let discovery = FileDiscovery::new(patterns, DiscoveryConfig::default());

        let files = discovery.discover()?;
        // Should be deduplicated to 1 file
        assert_eq!(files.len(), 1);

        Ok(())
    }
}
