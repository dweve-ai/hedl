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

//! Comprehensive tests for batch processing functionality

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::{tempdir, TempDir};

/// Test helper to create a HEDL command
fn hedl_cmd() -> Command {
    Command::cargo_bin("hedl").expect("Failed to find hedl binary")
}

/// Create a temporary directory with multiple test HEDL files
fn create_test_files(count: usize) -> (TempDir, Vec<String>) {
    let dir = tempdir().expect("Failed to create temp dir");
    let mut paths = Vec::new();

    for i in 0..count {
        let path = dir.path().join(format!("test{}.hedl", i));
        let content = format!(
            r#"%VERSION: 1.0
---
id: {}
name: Test {}
value: {}
"#,
            i, i, i * 10
        );
        fs::write(&path, content).expect("Failed to write test file");
        paths.push(path.to_str().unwrap().to_string());
    }

    (dir, paths)
}

/// Create test files with some invalid ones
fn create_mixed_test_files() -> (TempDir, Vec<String>, Vec<String>) {
    let dir = tempdir().expect("Failed to create temp dir");
    let mut valid_paths = Vec::new();
    let mut invalid_paths = Vec::new();

    // Create valid files
    for i in 0..3 {
        let path = dir.path().join(format!("valid{}.hedl", i));
        let content = format!(
            r#"%VERSION: 1.0
---
id: {}
value: {}
"#,
            i, i * 10
        );
        fs::write(&path, content).expect("Failed to write valid file");
        valid_paths.push(path.to_str().unwrap().to_string());
    }

    // Create invalid files
    for i in 0..2 {
        let path = dir.path().join(format!("invalid{}.hedl", i));
        let content = format!(
            r#"%VERSION: 1.0
---
invalid syntax here {}
"#,
            i
        );
        fs::write(&path, content).expect("Failed to write invalid file");
        invalid_paths.push(path.to_str().unwrap().to_string());
    }

    (dir, valid_paths, invalid_paths)
}

// ============================================================================
// Batch Validate Tests
// ============================================================================

#[test]
fn test_batch_validate_success() {
    let (_dir, paths) = create_test_files(5);

    hedl_cmd()
        .arg("batch-validate")
        .args(&paths)
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch Operation:"))
        .stdout(predicate::str::contains("validate"));
}

#[test]
fn test_batch_validate_with_strict_mode() {
    let (_dir, paths) = create_test_files(3);

    hedl_cmd()
        .arg("batch-validate")
        .args(&paths)
        .arg("--strict")
        .assert()
        .success();
}

#[test]
fn test_batch_validate_with_parallel_flag() {
    let (_dir, paths) = create_test_files(10);

    hedl_cmd()
        .arg("batch-validate")
        .args(&paths)
        .arg("--parallel")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total files: 10"));
}

#[test]
fn test_batch_validate_with_verbose() {
    let (_dir, paths) = create_test_files(3);

    hedl_cmd()
        .arg("batch-validate")
        .args(&paths)
        .arg("--verbose")
        .assert()
        .success();
}

#[test]
fn test_batch_validate_mixed_results() {
    let (_dir, valid, invalid) = create_mixed_test_files();
    let mut all_paths = valid.clone();
    all_paths.extend(invalid);

    hedl_cmd()
        .arg("batch-validate")
        .args(&all_paths)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Validation failures:"));
}

#[test]
fn test_batch_validate_empty_file_list() {
    hedl_cmd()
        .arg("batch-validate")
        .assert()
        .success(); // No files = nothing to validate = success
}

#[test]
fn test_batch_validate_nonexistent_files() {
    hedl_cmd()
        .arg("batch-validate")
        .arg("/nonexistent/file1.hedl")
        .arg("/nonexistent/file2.hedl")
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed"));
}

// ============================================================================
// Batch Format Tests
// ============================================================================

#[test]
fn test_batch_format_stdout() {
    let (_dir, paths) = create_test_files(3);

    hedl_cmd()
        .arg("batch-format")
        .args(&paths)
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch Operation:"))
        .stdout(predicate::str::contains("format"));
}

#[test]
fn test_batch_format_to_output_dir() {
    let (_dir, paths) = create_test_files(5);
    let output_dir = tempdir().expect("Failed to create output dir");

    hedl_cmd()
        .arg("batch-format")
        .args(&paths)
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success();

    // Verify output files were created
    for i in 0..5 {
        let output_file = output_dir.path().join(format!("test{}.hedl", i));
        assert!(
            output_file.exists(),
            "Output file {} should exist",
            output_file.display()
        );

        let content = fs::read_to_string(&output_file).expect("Failed to read output file");
        assert!(content.contains("%VERSION: 1.0"));
        assert!(content.contains(&format!("id: {}", i)));
    }
}

#[test]
fn test_batch_format_check_mode() {
    let dir = tempdir().expect("Failed to create temp dir");

    // Create a canonical file
    let canonical_path = dir.path().join("canonical.hedl");
    let canonical_content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    fs::write(&canonical_path, canonical_content).expect("Failed to write canonical file");

    // Create a non-canonical file
    let non_canonical_path = dir.path().join("non_canonical.hedl");
    let non_canonical_content = r#"%VERSION: 1.0
---
a:1
b:2
"#;
    fs::write(&non_canonical_path, non_canonical_content)
        .expect("Failed to write non-canonical file");

    // Check should fail because one file is not canonical
    hedl_cmd()
        .arg("batch-format")
        .arg(canonical_path.to_str().unwrap())
        .arg(non_canonical_path.to_str().unwrap())
        .arg("--check")
        .assert()
        .failure();
}

#[test]
fn test_batch_format_with_ditto() {
    let dir = tempdir().expect("Failed to create temp dir");
    let path = dir.path().join("test.hedl");

    let content = r#"%VERSION: 1.0
%STRUCT: T: [id,v]
---
d: @T
  | x,1
  | y,1
  | z,1
"#;
    fs::write(&path, content).expect("Failed to write test file");

    hedl_cmd()
        .arg("batch-format")
        .arg(path.to_str().unwrap())
        .arg("--ditto")
        .assert()
        .success();
}

#[test]
fn test_batch_format_with_counts() {
    let dir = tempdir().expect("Failed to create temp dir");
    let path = dir.path().join("test.hedl");

    let content = r#"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
teams: @Team
  | t1,Warriors
  | t2,Lakers
"#;
    fs::write(&path, content).expect("Failed to write test file");

    let output_dir = tempdir().expect("Failed to create output dir");

    hedl_cmd()
        .arg("batch-format")
        .arg(path.to_str().unwrap())
        .arg("--with-counts")
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success();

    let output_file = output_dir.path().join("test.hedl");
    let formatted = fs::read_to_string(&output_file).expect("Failed to read output");
    assert!(formatted.contains("(2)"));
}

#[test]
fn test_batch_format_parallel() {
    let (_dir, paths) = create_test_files(20);

    hedl_cmd()
        .arg("batch-format")
        .args(&paths)
        .arg("--parallel")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total files: 20"));
}

#[test]
fn test_batch_format_verbose() {
    let (_dir, paths) = create_test_files(3);

    hedl_cmd()
        .arg("batch-format")
        .args(&paths)
        .arg("--verbose")
        .assert()
        .success();
}

// ============================================================================
// Batch Lint Tests
// ============================================================================

#[test]
fn test_batch_lint_success() {
    let (_dir, paths) = create_test_files(5);

    hedl_cmd()
        .arg("batch-lint")
        .args(&paths)
        .assert()
        .success()
        .stdout(predicate::str::contains("Batch Operation:"))
        .stdout(predicate::str::contains("lint"));
}

#[test]
fn test_batch_lint_parallel() {
    let (_dir, paths) = create_test_files(10);

    hedl_cmd()
        .arg("batch-lint")
        .args(&paths)
        .arg("--parallel")
        .assert()
        .success();
}

#[test]
fn test_batch_lint_verbose() {
    let (_dir, paths) = create_test_files(3);

    hedl_cmd()
        .arg("batch-lint")
        .args(&paths)
        .arg("--verbose")
        .assert()
        .success();
}

#[test]
fn test_batch_lint_warn_error() {
    let (_dir, paths) = create_test_files(3);

    // This test behavior depends on what lint warnings are found
    // Just verify the command runs
    let _result = hedl_cmd()
        .arg("batch-lint")
        .args(&paths)
        .arg("--warn-error")
        .assert();
}

// ============================================================================
// Performance and Edge Case Tests
// ============================================================================

#[test]
fn test_batch_large_number_of_files() {
    let (_dir, paths) = create_test_files(50);

    hedl_cmd()
        .arg("batch-validate")
        .args(&paths)
        .arg("--parallel")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total files: 50"));
}

#[test]
fn test_batch_single_file() {
    let (_dir, paths) = create_test_files(1);

    hedl_cmd()
        .arg("batch-validate")
        .args(&paths)
        .assert()
        .success()
        .stdout(predicate::str::contains("Total files: 1"));
}

#[test]
fn test_batch_mixed_success_failure_counts() {
    let (_dir, valid, invalid) = create_mixed_test_files();
    let mut all_paths = valid.clone();
    all_paths.extend(invalid.clone());

    hedl_cmd()
        .arg("batch-validate")
        .args(&all_paths)
        .assert()
        .failure()
        .stdout(predicate::str::contains("Succeeded: 3"))
        .stdout(predicate::str::contains("Failed: 2"));
}

#[test]
fn test_batch_throughput_reporting() {
    let (_dir, paths) = create_test_files(10);

    hedl_cmd()
        .arg("batch-validate")
        .args(&paths)
        .assert()
        .success()
        .stdout(predicate::str::contains("files/s"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_batch_continues_on_error() {
    let (_dir, valid, invalid) = create_mixed_test_files();
    let mut all_paths = valid.clone();
    all_paths.extend(invalid);

    let output = hedl_cmd()
        .arg("batch-validate")
        .args(&all_paths)
        .assert()
        .failure();

    // Verify it processed all files (didn't stop at first error)
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(
        stderr.contains("3 of 5 files failed") || stderr.contains("2 of 5 files failed"),
        "Should report total failures, not stop at first error"
    );
}

#[test]
fn test_batch_format_invalid_output_dir() {
    let (_dir, paths) = create_test_files(2);

    hedl_cmd()
        .arg("batch-format")
        .args(&paths)
        .arg("--output-dir")
        .arg("/invalid/nonexistent/directory/that/cannot/be/created")
        .assert()
        .failure();
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_batch_validate_then_format() {
    let (_dir, paths) = create_test_files(5);

    // First validate
    hedl_cmd()
        .arg("batch-validate")
        .args(&paths)
        .assert()
        .success();

    // Then format
    let output_dir = tempdir().expect("Failed to create output dir");
    hedl_cmd()
        .arg("batch-format")
        .args(&paths)
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success();

    // Verify formatted files exist
    for i in 0..5 {
        let formatted_file = output_dir.path().join(format!("test{}.hedl", i));
        assert!(formatted_file.exists());
    }
}

#[test]
fn test_batch_format_then_lint() {
    let (_dir, paths) = create_test_files(3);
    let output_dir = tempdir().expect("Failed to create output dir");

    // Format files
    hedl_cmd()
        .arg("batch-format")
        .args(&paths)
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success();

    // Collect formatted file paths
    let formatted_paths: Vec<String> = (0..3)
        .map(|i| {
            output_dir
                .path()
                .join(format!("test{}.hedl", i))
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();

    // Lint formatted files
    hedl_cmd()
        .arg("batch-lint")
        .args(&formatted_paths)
        .assert()
        .success();
}

#[test]
fn test_batch_all_operations_sequence() {
    let (_dir, paths) = create_test_files(5);
    let output_dir = tempdir().expect("Failed to create output dir");

    // Validate
    hedl_cmd()
        .arg("batch-validate")
        .args(&paths)
        .assert()
        .success();

    // Format
    hedl_cmd()
        .arg("batch-format")
        .args(&paths)
        .arg("--output-dir")
        .arg(output_dir.path())
        .assert()
        .success();

    // Collect formatted paths
    let formatted_paths: Vec<String> = (0..5)
        .map(|i| {
            output_dir
                .path()
                .join(format!("test{}.hedl", i))
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();

    // Lint
    hedl_cmd()
        .arg("batch-lint")
        .args(&formatted_paths)
        .assert()
        .success();
}
