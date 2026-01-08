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

//! Integration tests for --with-counts flag

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

fn get_hedl_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // Go up to workspace root
    path.pop();
    path.push("target");
    path.push("debug");
    path.push("hedl");
    path
}

#[test]
fn test_with_counts_adds_counts_to_simple_lists() {
    let input = r#"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
teams: @Team
  | t1,Warriors
  | t2,Lakers
  | t3,Celtics
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(get_hedl_binary())
        .args(["format", temp_file.path().to_str().unwrap(), "--with-counts"])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    // New format: count goes in %STRUCT header, not list declaration
    assert!(output_str.contains("%STRUCT: Team (3): [id,name]"));
    assert!(output_str.contains("teams: @Team"));
    assert!(!output_str.contains("teams(3)"));

    // Tempfile automatically cleaned up when it goes out of scope
}

#[test]
fn test_with_counts_overwrites_existing_counts() {
    let input = r#"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
teams(5): @Team
  | t1,Warriors
  | t2,Lakers
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(get_hedl_binary())
        .args(["format", temp_file.path().to_str().unwrap(), "--with-counts"])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    // New format: count goes in %STRUCT header
    assert!(output_str.contains("%STRUCT: Team (2): [id,name]"));
    assert!(output_str.contains("teams: @Team"));
    assert!(!output_str.contains("teams(2)"));
    assert!(!output_str.contains("teams(5)"));

    // Tempfile automatically cleaned up when it goes out of scope
}

#[test]
fn test_with_counts_handles_empty_lists() {
    let input = r#"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
teams: @Team
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(get_hedl_binary())
        .args(["format", temp_file.path().to_str().unwrap(), "--with-counts"])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    // New format: count goes in %STRUCT header (0 for empty list)
    assert!(output_str.contains("%STRUCT: Team (0): [id,name]"));
    assert!(output_str.contains("teams: @Team"));

    // Tempfile automatically cleaned up when it goes out of scope
}

#[test]
fn test_with_counts_handles_nested_objects() {
    let input = r#"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
sports:
  basketball:
    teams: @Team
      | t1,Lakers
      | t2,Celtics
  football:
    teams: @Team
      | t3,Chiefs
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(get_hedl_binary())
        .args(["format", temp_file.path().to_str().unwrap(), "--with-counts"])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);

    // New format: counts are in %STRUCT header (total count for the type)
    // Since there are 3 Team rows total (2 in basketball, 1 in football)
    assert!(output_str.contains("%STRUCT: Team (3): [id,name]"));
    assert!(output_str.contains("teams: @Team"));
    assert!(!output_str.contains("teams("));

    // Tempfile automatically cleaned up when it goes out of scope
}

#[test]
fn test_without_counts_flag_preserves_no_counts() {
    let input = r#"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
teams: @Team
  | t1,Warriors
  | t2,Lakers
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(get_hedl_binary())
        .args(["format", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("teams: @Team"));
    assert!(!output_str.contains("teams(2): @Team"));

    // Tempfile automatically cleaned up when it goes out of scope
}

#[test]
fn test_with_counts_multiple_lists() {
    let input = r#"%VERSION: 1.0
%STRUCT: Team: [id,name]
%STRUCT: Player: [id,name]
---
teams: @Team
  | t1,Warriors
  | t2,Lakers
  | t3,Celtics

players: @Player
  | p1,Curry
  | p2,James
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(get_hedl_binary())
        .args(["format", temp_file.path().to_str().unwrap(), "--with-counts"])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);

    // New format: counts are in %STRUCT headers
    assert!(output_str.contains("%STRUCT: Player (2): [id,name]"));
    assert!(output_str.contains("%STRUCT: Team (3): [id,name]"));
    assert!(output_str.contains("teams: @Team"));
    assert!(output_str.contains("players: @Player"));
    assert!(!output_str.contains("teams("));
    assert!(!output_str.contains("players("));

    // Tempfile automatically cleaned up when it goes out of scope
}

#[test]
fn test_with_counts_inline_schema() {
    let input = r#"%VERSION: 1.0
---
teams: @Team[id,name]
  | t1,Warriors
  | t2,Lakers
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(get_hedl_binary())
        .args(["format", temp_file.path().to_str().unwrap(), "--with-counts"])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    // Canonical format converts inline schemas to STRUCT declarations with counts
    assert!(output_str.contains("teams: @Team"));
    assert!(output_str.contains("%STRUCT: Team (2): [id,name]"));

    // Tempfile automatically cleaned up when it goes out of scope
}
