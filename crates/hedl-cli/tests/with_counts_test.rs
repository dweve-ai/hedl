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

use assert_cmd::cargo_bin;
use assert_cmd::Command;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_with_counts_adds_counts_to_simple_lists() {
    let input = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Team:[id,name]
---
teams:@Team
 |t1,Warriors
 |t2,Lakers
 |t3,Celtics
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(cargo_bin!("hedl"))
        .args([
            "format",
            temp_file.path().to_str().unwrap(),
            "--with-counts",
        ])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    // v2.0 format: count goes in %C directive
    assert!(
        output_str.contains("%C:Team.total=3"),
        "Expected %C:Team.total=3, got: {output_str}"
    );
    assert!(output_str.contains("teams:@Team"));
}

#[test]
fn test_with_counts_overwrites_existing_counts() {
    let input = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Team:[id,name]
%C:Team.total=5
---
teams:@Team
 |t1,Warriors
 |t2,Lakers
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(cargo_bin!("hedl"))
        .args([
            "format",
            temp_file.path().to_str().unwrap(),
            "--with-counts",
        ])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    // v2.0 format: count directive should be updated to actual count
    assert!(
        output_str.contains("%C:Team.total=2"),
        "Expected %C:Team.total=2, got: {output_str}"
    );
    assert!(output_str.contains("teams:@Team"));
    // Old count should not be present
    assert!(!output_str.contains(".total=5"));
}

#[test]
fn test_with_counts_handles_empty_lists() {
    let input = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Team:[id,name]
---
teams:@Team
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(cargo_bin!("hedl"))
        .args([
            "format",
            temp_file.path().to_str().unwrap(),
            "--with-counts",
        ])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    // v2.0 format: count directive for empty list (0)
    assert!(
        output_str.contains("%C:Team.total=0"),
        "Expected %C:Team.total=0, got: {output_str}"
    );
    assert!(output_str.contains("teams:@Team"));
}

#[test]
fn test_with_counts_handles_nested_objects() {
    let input = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Team:[id,name]
---
sports:
 basketball:
  teams:@Team
   |t1,Lakers
   |t2,Celtics
 football:
  teams:@Team
   |t3,Chiefs
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(cargo_bin!("hedl"))
        .args([
            "format",
            temp_file.path().to_str().unwrap(),
            "--with-counts",
        ])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);

    // v2.0 format: count directive (total count for the type)
    // Since there are 3 Team rows total (2 in basketball, 1 in football)
    assert!(
        output_str.contains("%C:Team.total=3"),
        "Expected %C:Team.total=3, got: {output_str}"
    );
    assert!(output_str.contains("teams:@Team"));
}

#[test]
fn test_without_counts_flag_still_emits_actual_counts() {
    // v2.0 canonical output always includes %C: directives that reflect actual data
    let input = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Team:[id,name]
---
teams:@Team
 |t1,Warriors
 |t2,Lakers
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(cargo_bin!("hedl"))
        .args(["format", temp_file.path().to_str().unwrap()])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(output_str.contains("teams:@Team"));
    // v2.0 canonical output always includes actual counts
    assert!(
        output_str.contains("%C:Team.total=2"),
        "Expected %C:Team.total=2 in canonical output, got: {output_str}"
    );
}

#[test]
fn test_with_counts_multiple_lists() {
    let input = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Team:[id,name]
%S:Player:[id,name]
---
teams:@Team
 |t1,Warriors
 |t2,Lakers
 |t3,Celtics

players:@Player
 |p1,Curry
 |p2,James
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(cargo_bin!("hedl"))
        .args([
            "format",
            temp_file.path().to_str().unwrap(),
            "--with-counts",
        ])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);

    // v2.0 format: count directives for each type
    assert!(
        output_str.contains("%C:Player.total=2"),
        "Expected %C:Player.total=2, got: {output_str}"
    );
    assert!(
        output_str.contains("%C:Team.total=3"),
        "Expected %C:Team.total=3, got: {output_str}"
    );
    assert!(output_str.contains("teams:@Team"));
    assert!(output_str.contains("players:@Player"));
}

#[test]
fn test_with_counts_inline_schema() {
    let input = r#"%V:2.0
%NULL:~
%QUOTE:"
---
teams:@Team[id,name]
 |t1,Warriors
 |t2,Lakers
"#;

    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), input).unwrap();

    let output = Command::new(cargo_bin!("hedl"))
        .args([
            "format",
            temp_file.path().to_str().unwrap(),
            "--with-counts",
        ])
        .output()
        .expect("Failed to execute hedl");

    let output_str = String::from_utf8_lossy(&output.stdout);
    // Canonical format converts inline schemas to %S declarations with %C counts
    assert!(output_str.contains("teams:@Team"));
    assert!(
        output_str.contains("%C:Team.total=2"),
        "Expected %C:Team.total=2, got: {output_str}"
    );
}
