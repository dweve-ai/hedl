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

//! Integration tests for file discovery with glob patterns.

use hedl_cli::error::CliError;
use hedl_cli::file_discovery::{DiscoveryConfig, FileDiscovery};
use std::fs;
use tempfile::TempDir;

fn create_test_files(dir: &std::path::Path, files: &[&str]) -> Result<(), std::io::Error> {
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
fn test_glob_single_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    create_test_files(temp_dir.path(), &["test.hedl"])?;

    let pattern = format!("{}/test.hedl", temp_dir.path().display());
    let discovery = FileDiscovery::new(vec![pattern], DiscoveryConfig::default());

    let files = discovery.discover()?;
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("test.hedl"));

    Ok(())
}

#[test]
fn test_glob_wildcard_expansion() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    create_test_files(
        temp_dir.path(),
        &["file1.hedl", "file2.hedl", "file3.txt", "file4.hedl"],
    )?;

    let pattern = format!("{}/*.hedl", temp_dir.path().display());
    let discovery = FileDiscovery::new(vec![pattern], DiscoveryConfig::default());

    let files = discovery.discover()?;
    assert_eq!(files.len(), 3);
    assert!(files.iter().all(|p| p.extension().unwrap() == "hedl"));

    Ok(())
}

#[test]
fn test_glob_recursive_pattern() -> Result<(), Box<dyn std::error::Error>> {
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
    assert!(files.iter().all(|p| p.extension().unwrap() == "hedl"));

    Ok(())
}

#[test]
fn test_glob_depth_limiting() -> Result<(), Box<dyn std::error::Error>> {
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
    // With max_depth=2, should find files up to 2 levels deep
    assert!(files.len() >= 2);
    assert!(files.len() <= 3);

    Ok(())
}

#[test]
fn test_glob_no_matches() {
    let temp_dir = TempDir::new().unwrap();
    let pattern = format!("{}/*.hedl", temp_dir.path().display());
    let discovery = FileDiscovery::new(vec![pattern.clone()], DiscoveryConfig::default());

    let result = discovery.discover();
    assert!(result.is_err());
    match result.unwrap_err() {
        CliError::NoFilesMatched { patterns } => {
            assert_eq!(patterns.len(), 1);
        }
        _ => panic!("Expected NoFilesMatched error"),
    }
}

#[test]
fn test_glob_invalid_pattern() {
    let discovery = FileDiscovery::new(vec!["[invalid".to_string()], DiscoveryConfig::default());

    let result = discovery.validate_patterns();
    assert!(result.is_err());
    match result.unwrap_err() {
        CliError::GlobPattern { pattern, .. } => {
            assert_eq!(pattern, "[invalid");
        }
        _ => panic!("Expected GlobPattern error"),
    }
}

#[test]
fn test_glob_extension_filtering() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    create_test_files(
        temp_dir.path(),
        &["file1.hedl", "file2.txt", "file3.hedl", "file4.json"],
    )?;

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
fn test_glob_hidden_files_excluded_by_default() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    create_test_files(
        temp_dir.path(),
        &["file1.hedl", ".hidden.hedl", "dir/.hidden2.hedl"],
    )?;

    let pattern = format!("{}/**/*.hedl", temp_dir.path().display());
    let discovery = FileDiscovery::new(
        vec![pattern],
        DiscoveryConfig {
            recursive: true,
            include_hidden: false,
            ..Default::default()
        },
    );

    let files = discovery.discover()?;
    // Should only find file1.hedl, not hidden files
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("file1.hedl"));

    Ok(())
}

#[test]
fn test_glob_hidden_files_included() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    create_test_files(temp_dir.path(), &["file1.hedl", ".hidden.hedl"])?;

    let pattern = format!("{}/**/*.hedl", temp_dir.path().display());
    let discovery = FileDiscovery::new(
        vec![pattern],
        DiscoveryConfig {
            recursive: true,
            include_hidden: true,
            ..Default::default()
        },
    );

    let files = discovery.discover()?;
    // Should find both visible and hidden files
    assert!(files.len() >= 2);

    Ok(())
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
fn test_deduplication() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    create_test_files(temp_dir.path(), &["file1.hedl"])?;

    let file_path = format!("{}/file1.hedl", temp_dir.path().display());
    // Same file matched by multiple patterns
    let patterns = vec![
        file_path.clone(),
        format!("{}/*.hedl", temp_dir.path().display()),
    ];
    let discovery = FileDiscovery::new(patterns, DiscoveryConfig::default());

    let files = discovery.discover()?;
    // Should be deduplicated
    assert_eq!(files.len(), 1);

    Ok(())
}

#[test]
fn test_recursive_simple_pattern() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    create_test_files(
        temp_dir.path(),
        &["file1.hedl", "dir1/file2.hedl", "dir1/dir2/file3.hedl"],
    )?;

    // Simple pattern (no **) with recursive flag
    let pattern = format!("{}/*.hedl", temp_dir.path().display());
    let discovery = FileDiscovery::new(
        vec![pattern],
        DiscoveryConfig {
            recursive: true,
            ..Default::default()
        },
    );

    let files = discovery.discover()?;
    // Should recursively find all hedl files
    assert_eq!(files.len(), 3);

    Ok(())
}
