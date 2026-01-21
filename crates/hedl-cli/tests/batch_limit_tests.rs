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

//! Integration tests for batch file count limits.
//!
//! These tests verify that batch operations properly enforce file count limits
//! to prevent resource exhaustion.

use assert_cmd::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Valid minimal HEDL content for tests (in canonical form).
const VALID_HEDL: &str = "%VERSION: 1.0\n---\na: 1\n";

#[test]
fn test_batch_validate_exceeds_custom_limit() {
    let temp = TempDir::new().unwrap();

    // Create test files with valid HEDL content
    fs::write(temp.path().join("test1.hedl"), VALID_HEDL).unwrap();
    fs::write(temp.path().join("test2.hedl"), VALID_HEDL).unwrap();

    let mut cmd = Command::new(cargo_bin!("hedl"));
    cmd.arg("batch-validate")
        .arg("--max-files")
        .arg("1") // Set very low limit
        .arg(temp.path().join("test1.hedl").to_str().unwrap())
        .arg(temp.path().join("test2.hedl").to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("exceeds maximum limit"));
}

#[test]
fn test_batch_validate_unlimited() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("test1.hedl"), VALID_HEDL).unwrap();
    fs::write(temp.path().join("test2.hedl"), VALID_HEDL).unwrap();

    let mut cmd = Command::new(cargo_bin!("hedl"));
    cmd.arg("batch-validate")
        .arg("--max-files")
        .arg("0") // 0 = unlimited
        .arg(temp.path().join("test1.hedl").to_str().unwrap())
        .arg(temp.path().join("test2.hedl").to_str().unwrap());

    cmd.assert().success();
}

#[test]
fn test_batch_validate_within_custom_limit() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("test1.hedl"), VALID_HEDL).unwrap();
    fs::write(temp.path().join("test2.hedl"), VALID_HEDL).unwrap();

    let mut cmd = Command::new(cargo_bin!("hedl"));
    cmd.arg("batch-validate")
        .arg("--max-files")
        .arg("10")
        .arg(temp.path().join("test1.hedl").to_str().unwrap())
        .arg(temp.path().join("test2.hedl").to_str().unwrap());

    cmd.assert().success();
}

#[test]
fn test_batch_format_exceeds_custom_limit() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("test1.hedl"), VALID_HEDL).unwrap();
    fs::write(temp.path().join("test2.hedl"), VALID_HEDL).unwrap();

    let mut cmd = Command::new(cargo_bin!("hedl"));
    cmd.arg("batch-format")
        .arg("--max-files")
        .arg("1")
        .arg("--check")
        .arg(temp.path().join("test1.hedl").to_str().unwrap())
        .arg(temp.path().join("test2.hedl").to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("exceeds maximum limit"));
}

#[test]
fn test_batch_format_within_limit() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("test1.hedl"), VALID_HEDL).unwrap();
    fs::write(temp.path().join("test2.hedl"), VALID_HEDL).unwrap();

    let mut cmd = Command::new(cargo_bin!("hedl"));
    cmd.arg("batch-format")
        .arg("--max-files")
        .arg("5")
        .arg("--check")
        .arg(temp.path().join("test1.hedl").to_str().unwrap())
        .arg(temp.path().join("test2.hedl").to_str().unwrap());

    cmd.assert().success();
}

#[test]
fn test_batch_lint_exceeds_custom_limit() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("test1.hedl"), VALID_HEDL).unwrap();
    fs::write(temp.path().join("test2.hedl"), VALID_HEDL).unwrap();

    let mut cmd = Command::new(cargo_bin!("hedl"));
    cmd.arg("batch-lint")
        .arg("--max-files")
        .arg("1")
        .arg(temp.path().join("test1.hedl").to_str().unwrap())
        .arg(temp.path().join("test2.hedl").to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("exceeds maximum limit"));
}

#[test]
fn test_batch_lint_within_limit() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("test1.hedl"), VALID_HEDL).unwrap();
    fs::write(temp.path().join("test2.hedl"), VALID_HEDL).unwrap();

    let mut cmd = Command::new(cargo_bin!("hedl"));
    cmd.arg("batch-lint")
        .arg("--max-files")
        .arg("5")
        .arg(temp.path().join("test1.hedl").to_str().unwrap())
        .arg(temp.path().join("test2.hedl").to_str().unwrap());

    cmd.assert().success();
}

#[test]
fn test_batch_validate_at_limit_boundary() {
    let temp = TempDir::new().unwrap();

    // Create exactly 3 files with valid HEDL content
    fs::write(temp.path().join("test1.hedl"), VALID_HEDL).unwrap();
    fs::write(temp.path().join("test2.hedl"), VALID_HEDL).unwrap();
    fs::write(temp.path().join("test3.hedl"), VALID_HEDL).unwrap();

    // Set limit to exactly 3
    let mut cmd = Command::new(cargo_bin!("hedl"));
    cmd.arg("batch-validate")
        .arg("--max-files")
        .arg("3")
        .arg(temp.path().join("test1.hedl").to_str().unwrap())
        .arg(temp.path().join("test2.hedl").to_str().unwrap())
        .arg(temp.path().join("test3.hedl").to_str().unwrap());

    cmd.assert().success();
}

#[test]
fn test_error_message_contains_helpful_suggestions() {
    let temp = TempDir::new().unwrap();

    fs::write(temp.path().join("test1.hedl"), VALID_HEDL).unwrap();
    fs::write(temp.path().join("test2.hedl"), VALID_HEDL).unwrap();

    let mut cmd = Command::new(cargo_bin!("hedl"));
    cmd.arg("batch-validate")
        .arg("--max-files")
        .arg("1")
        .arg(temp.path().join("test1.hedl").to_str().unwrap())
        .arg(temp.path().join("test2.hedl").to_str().unwrap());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Consider:"))
        .stderr(predicate::str::contains("--max-files"))
        .stderr(predicate::str::contains("HEDL_MAX_BATCH_FILES"));
}
