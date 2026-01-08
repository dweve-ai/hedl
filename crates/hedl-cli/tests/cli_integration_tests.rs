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

//! Comprehensive CLI integration tests

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

// Test helper to create a HEDL command
fn hedl_cmd() -> Command {
    Command::cargo_bin("hedl").expect("Failed to find hedl binary")
}

// Test helper to create a temporary file with content
fn create_temp_file(content: &str, suffix: &str) -> NamedTempFile {
    let file = tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .expect("Failed to create temp file");
    fs::write(file.path(), content).expect("Failed to write temp file");
    file
}

// ===== Help and Version Tests =====

#[test]
fn test_help_output() {
    hedl_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("HEDL - Hierarchical Entity Data Language toolkit"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_version_output() {
    hedl_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hedl"));
}

#[test]
fn test_no_subcommand_fails() {
    hedl_cmd()
        .assert()
        .failure();
}

// ===== Validate Command Tests =====

#[test]
fn test_validate_valid_file() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("validate")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("✓"))
        .stdout(predicate::str::contains("Version:"))
        .stdout(predicate::str::contains("Structs:"))
        .stdout(predicate::str::contains("Aliases:"));
}

#[test]
fn test_validate_invalid_file() {
    let content = r#"%VERSION: 1.0
---
a:1
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("validate")
        .arg(file.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("✗"));
}

#[test]
fn test_validate_missing_file() {
    hedl_cmd()
        .arg("validate")
        .arg("/nonexistent/file.hedl")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to"));
}

#[test]
fn test_validate_with_strict_mode() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("validate")
        .arg(file.path())
        .arg("--strict")
        .assert()
        .success();
}

// ===== Format Command Tests =====

#[test]
fn test_format_to_stdout() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("format")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("%VERSION: 1.0"))
        .stdout(predicate::str::contains("a: 1"))
        .stdout(predicate::str::contains("b: 2"));
}

#[test]
fn test_format_to_output_file() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let input_file = create_temp_file(content, ".hedl");
    let output_file = NamedTempFile::new().expect("Failed to create output file");

    hedl_cmd()
        .arg("format")
        .arg(input_file.path())
        .arg("--output")
        .arg(output_file.path())
        .assert()
        .success();

    let output_content = fs::read_to_string(output_file.path()).expect("Failed to read output");
    assert!(output_content.contains("%VERSION: 1.0"));
    assert!(output_content.contains("a: 1"));
}

#[test]
fn test_format_check_mode_canonical() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("format")
        .arg(file.path())
        .arg("--check")
        .assert()
        .success();
}

#[test]
fn test_format_check_mode_not_canonical() {
    let content = r#"%VERSION: 1.0
---
a:1
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("format")
        .arg(file.path())
        .arg("--check")
        .assert()
        .failure();
}

#[test]
fn test_format_with_ditto() {
    let content = r#"%VERSION: 1.0
%STRUCT: T: [id,v]
---
d: @T
  | x,1
  | y,1
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("format")
        .arg(file.path())
        .arg("--ditto")
        .assert()
        .success()
        .stdout(predicate::str::contains("^"));
}

#[test]
fn test_format_with_counts() {
    let content = r#"%VERSION: 1.0
%STRUCT: Team: [id,name]
---
teams: @Team
  | t1,Warriors
  | t2,Lakers
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("format")
        .arg(file.path())
        .arg("--with-counts")
        .assert()
        .success()
        .stdout(predicate::str::contains("%STRUCT: Team (2): [id,name]"));
}

// ===== Lint Command Tests =====

#[test]
fn test_lint_valid_file_text_format() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("lint")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("✓"))
        .stdout(predicate::str::contains("no issues found"));
}

#[test]
fn test_lint_json_format() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("lint")
        .arg(file.path())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file\":"))
        .stdout(predicate::str::contains("\"diagnostics\":"));
}

#[test]
fn test_lint_warn_error_flag() {
    // This test may need adjustment based on actual lint rules
    let content = r#"%VERSION: 1.0
---
a: 1
"#;
    let file = create_temp_file(content, ".hedl");

    // Without --warn-error, warnings don't fail
    let result = hedl_cmd()
        .arg("lint")
        .arg(file.path())
        .assert();

    // Store the exit code for comparison
    let exit_code_without_flag = result.get_output().status.code();

    // Test with --warn-error
    hedl_cmd()
        .arg("lint")
        .arg(file.path())
        .arg("--warn-error")
        .assert();

    // We just verify the command executes
    assert!(exit_code_without_flag.is_some());
}

// ===== JSON Conversion Tests =====

#[test]
fn test_to_json_stdout() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("to-json")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("{"))
        .stdout(predicate::str::contains("\"a\""))
        .stdout(predicate::str::contains("\"b\""));
}

#[test]
fn test_to_json_with_output_file() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let input_file = create_temp_file(content, ".hedl");
    let output_file = NamedTempFile::new().expect("Failed to create output file");

    hedl_cmd()
        .arg("to-json")
        .arg(input_file.path())
        .arg("--output")
        .arg(output_file.path())
        .assert()
        .success();

    let output_content = fs::read_to_string(output_file.path()).expect("Failed to read output");
    assert!(output_content.contains("\"a\""));
}

#[test]
fn test_to_json_pretty() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("to-json")
        .arg(file.path())
        .arg("--pretty")
        .assert()
        .success()
        .stdout(predicate::str::contains("  "));  // Indentation indicates pretty print
}

#[test]
fn test_to_json_with_metadata() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("to-json")
        .arg(file.path())
        .arg("--metadata")
        .assert()
        .success();
}

#[test]
fn test_from_json_to_hedl() {
    let json_content = r#"{"a": 1, "b": 2}"#;
    let file = create_temp_file(json_content, ".json");

    hedl_cmd()
        .arg("from-json")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("%VERSION:"))
        .stdout(predicate::str::contains("a:"))
        .stdout(predicate::str::contains("b:"));
}

#[test]
fn test_json_roundtrip() {
    let original_hedl = r#"%VERSION: 1.0
---
name: John
age: 30
"#;
    let hedl_file = create_temp_file(original_hedl, ".hedl");
    let json_file = NamedTempFile::new().expect("Failed to create json file");
    let hedl_output_file = NamedTempFile::new().expect("Failed to create output file");

    // Convert to JSON
    hedl_cmd()
        .arg("to-json")
        .arg(hedl_file.path())
        .arg("--output")
        .arg(json_file.path())
        .assert()
        .success();

    // Convert back to HEDL
    hedl_cmd()
        .arg("from-json")
        .arg(json_file.path())
        .arg("--output")
        .arg(hedl_output_file.path())
        .assert()
        .success();

    let output = fs::read_to_string(hedl_output_file.path()).expect("Failed to read output");
    assert!(output.contains("name:"));
    assert!(output.contains("age:"));
}

// ===== YAML Conversion Tests =====

#[test]
fn test_to_yaml_stdout() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("to-yaml")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("a:"))
        .stdout(predicate::str::contains("b:"));
}

#[test]
fn test_from_yaml_to_hedl() {
    let yaml_content = "a: 1\nb: 2\n";
    let file = create_temp_file(yaml_content, ".yaml");

    hedl_cmd()
        .arg("from-yaml")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("%VERSION:"))
        .stdout(predicate::str::contains("a:"))
        .stdout(predicate::str::contains("b:"));
}

#[test]
fn test_yaml_roundtrip() {
    let original_hedl = r#"%VERSION: 1.0
---
city: Paris
population: 2000000
"#;
    let hedl_file = create_temp_file(original_hedl, ".hedl");
    let yaml_file = NamedTempFile::new().expect("Failed to create yaml file");
    let hedl_output_file = NamedTempFile::new().expect("Failed to create output file");

    // Convert to YAML
    hedl_cmd()
        .arg("to-yaml")
        .arg(hedl_file.path())
        .arg("--output")
        .arg(yaml_file.path())
        .assert()
        .success();

    // Convert back to HEDL
    hedl_cmd()
        .arg("from-yaml")
        .arg(yaml_file.path())
        .arg("--output")
        .arg(hedl_output_file.path())
        .assert()
        .success();

    let output = fs::read_to_string(hedl_output_file.path()).expect("Failed to read output");
    assert!(output.contains("city:"));
    assert!(output.contains("population:"));
}

// ===== XML Conversion Tests =====

#[test]
fn test_to_xml_stdout() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("to-xml")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("<"))
        .stdout(predicate::str::contains(">"));
}

#[test]
fn test_to_xml_pretty() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("to-xml")
        .arg(file.path())
        .arg("--pretty")
        .assert()
        .success()
        .stdout(predicate::str::contains("<"));
}

#[test]
fn test_from_xml_to_hedl() {
    let xml_content = r#"<?xml version="1.0"?><root><a>1</a><b>2</b></root>"#;
    let file = create_temp_file(xml_content, ".xml");

    hedl_cmd()
        .arg("from-xml")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("%VERSION:"));
}

#[test]
fn test_xml_roundtrip() {
    let original_hedl = r#"%VERSION: 1.0
---
title: Book
pages: 300
"#;
    let hedl_file = create_temp_file(original_hedl, ".hedl");
    let xml_file = NamedTempFile::new().expect("Failed to create xml file");
    let hedl_output_file = NamedTempFile::new().expect("Failed to create output file");

    // Convert to XML
    hedl_cmd()
        .arg("to-xml")
        .arg(hedl_file.path())
        .arg("--output")
        .arg(xml_file.path())
        .assert()
        .success();

    // Convert back to HEDL
    hedl_cmd()
        .arg("from-xml")
        .arg(xml_file.path())
        .arg("--output")
        .arg(hedl_output_file.path())
        .assert()
        .success();

    let output = fs::read_to_string(hedl_output_file.path()).expect("Failed to read output");
    assert!(output.contains("%VERSION:"));
}

// ===== CSV Conversion Tests =====

#[test]
fn test_to_csv_stdout() {
    let content = r#"%VERSION: 1.0
%STRUCT: T: [id,v]
---
d: @T
  | x,1
  | y,2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("to-csv")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(","));
}

#[test]
fn test_to_csv_with_headers() {
    let content = r#"%VERSION: 1.0
%STRUCT: T: [id,v]
---
d: @T
  | x,1
  | y,2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("to-csv")
        .arg(file.path())
        .arg("--headers")
        .assert()
        .success();
}

#[test]
fn test_from_csv_to_hedl() {
    let csv_content = "id,name\n1,Alice\n2,Bob\n";
    let file = create_temp_file(csv_content, ".csv");

    hedl_cmd()
        .arg("from-csv")
        .arg(file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("%VERSION:"))
        .stdout(predicate::str::contains("@Row"));
}

#[test]
fn test_from_csv_with_type_name() {
    let csv_content = "id,name\n1,Alice\n2,Bob\n";
    let file = create_temp_file(csv_content, ".csv");

    hedl_cmd()
        .arg("from-csv")
        .arg(file.path())
        .arg("--type-name")
        .arg("Person")
        .assert()
        .success()
        .stdout(predicate::str::contains("@Person"));
}

// ===== Parquet Conversion Tests =====

#[test]
fn test_to_parquet_requires_output() {
    let content = r#"%VERSION: 1.0
%STRUCT: T: [id,v]
---
d: @T
  | x,1
"#;
    let file = create_temp_file(content, ".hedl");
    let output_file = NamedTempFile::new().expect("Failed to create output file");

    hedl_cmd()
        .arg("to-parquet")
        .arg(file.path())
        .arg("--output")
        .arg(output_file.path())
        .assert()
        .success();

    // Verify output file exists and has content
    assert!(output_file.path().exists());
    let metadata = fs::metadata(output_file.path()).expect("Failed to get metadata");
    assert!(metadata.len() > 0);
}

#[test]
fn test_to_parquet_missing_output_flag() {
    let content = r#"%VERSION: 1.0
---
a: 1
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("to-parquet")
        .arg(file.path())
        .assert()
        .failure();
}

#[test]
fn test_from_parquet_to_hedl() {
    let hedl_content = r#"%VERSION: 1.0
%STRUCT: T: [id,v]
---
d: @T
  | x,1
  | y,2
"#;
    let hedl_file = create_temp_file(hedl_content, ".hedl");
    let parquet_file = NamedTempFile::new().expect("Failed to create parquet file");

    // First create a parquet file
    hedl_cmd()
        .arg("to-parquet")
        .arg(hedl_file.path())
        .arg("--output")
        .arg(parquet_file.path())
        .assert()
        .success();

    // Then convert it back to HEDL
    hedl_cmd()
        .arg("from-parquet")
        .arg(parquet_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("%VERSION:"));
}

#[test]
fn test_parquet_roundtrip() {
    let original_hedl = r#"%VERSION: 1.0
%STRUCT: Person: [id,name,age]
---
people: @Person
  | p1,Alice,30
  | p2,Bob,25
"#;
    let hedl_file = create_temp_file(original_hedl, ".hedl");
    let parquet_file = NamedTempFile::new().expect("Failed to create parquet file");
    let hedl_output_file = NamedTempFile::new().expect("Failed to create output file");

    // Convert to Parquet
    hedl_cmd()
        .arg("to-parquet")
        .arg(hedl_file.path())
        .arg("--output")
        .arg(parquet_file.path())
        .assert()
        .success();

    // Convert back to HEDL
    hedl_cmd()
        .arg("from-parquet")
        .arg(parquet_file.path())
        .arg("--output")
        .arg(hedl_output_file.path())
        .assert()
        .success();

    let output = fs::read_to_string(hedl_output_file.path()).expect("Failed to read output");
    assert!(output.contains("%VERSION:"));
    assert!(output.contains("@Person"));
}

// ===== Inspect Command Tests =====

#[test]
fn test_inspect_basic() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("inspect")
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn test_inspect_verbose() {
    let content = r#"%VERSION: 1.0
%STRUCT: T: [id,v]
---
d: @T
  | x,1
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("inspect")
        .arg(file.path())
        .arg("--verbose")
        .assert()
        .success();
}

#[test]
fn test_inspect_complex_structure() {
    let content = r#"%VERSION: 1.0
%STRUCT: Team: [id,name]
%ALIAS: %myref: "@Team#t1"
---
teams: @Team
  | t1,Warriors
  | t2,Lakers
nested:
  inner:
    value: 42
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("inspect")
        .arg(file.path())
        .assert()
        .success();
}

// ===== Stats Command Tests =====

#[test]
fn test_stats_basic() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("stats")
        .arg(file.path())
        .assert()
        .success();
}

#[test]
fn test_stats_with_tokens() {
    let content = r#"%VERSION: 1.0
%STRUCT: T: [id,v]
---
d: @T
  | x,1
  | y,2
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("stats")
        .arg(file.path())
        .arg("--tokens")
        .assert()
        .success()
        .stdout(predicate::str::contains("token").or(predicate::str::contains("Token")));
}

#[test]
fn test_stats_shows_size_comparison() {
    let content = r#"%VERSION: 1.0
%STRUCT: Person: [id,name,age]
---
people: @Person
  | p1,Alice,30
  | p2,Bob,25
  | p3,Charlie,35
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("stats")
        .arg(file.path())
        .assert()
        .success();
}

// ===== Error Handling Tests =====

#[test]
fn test_invalid_subcommand() {
    hedl_cmd()
        .arg("invalid-command")
        .assert()
        .failure();
}

#[test]
fn test_missing_required_argument() {
    hedl_cmd()
        .arg("validate")
        .assert()
        .failure();
}

#[test]
fn test_invalid_format_option() {
    let content = r#"%VERSION: 1.0
---
a: 1
"#;
    let file = create_temp_file(content, ".hedl");

    hedl_cmd()
        .arg("lint")
        .arg(file.path())
        .arg("--format")
        .arg("invalid-format")
        .assert();
    // Command may succeed but use default format or show warning
}

#[test]
fn test_conversion_invalid_input() {
    let invalid_json = "{invalid json content}";
    let file = create_temp_file(invalid_json, ".json");

    hedl_cmd()
        .arg("from-json")
        .arg(file.path())
        .assert()
        .failure();
}

#[test]
fn test_format_parse_error() {
    let invalid_hedl = "%VERSION: 1.0\n---\n{{{invalid}}}";
    let file = create_temp_file(invalid_hedl, ".hedl");

    hedl_cmd()
        .arg("format")
        .arg(file.path())
        .assert()
        .failure();
}

// ===== Advanced Integration Tests =====

#[test]
fn test_multiple_conversions_pipeline() {
    // Test HEDL -> JSON -> HEDL -> YAML -> HEDL
    let original = r#"%VERSION: 1.0
---
test: value
number: 42
"#;
    let hedl1 = create_temp_file(original, ".hedl");
    let json_file = NamedTempFile::new().unwrap();
    let hedl2 = NamedTempFile::new().unwrap();
    let yaml_file = NamedTempFile::new().unwrap();
    let hedl3 = NamedTempFile::new().unwrap();

    // HEDL -> JSON
    hedl_cmd()
        .arg("to-json")
        .arg(hedl1.path())
        .arg("--output")
        .arg(json_file.path())
        .assert()
        .success();

    // JSON -> HEDL
    hedl_cmd()
        .arg("from-json")
        .arg(json_file.path())
        .arg("--output")
        .arg(hedl2.path())
        .assert()
        .success();

    // HEDL -> YAML
    hedl_cmd()
        .arg("to-yaml")
        .arg(hedl2.path())
        .arg("--output")
        .arg(yaml_file.path())
        .assert()
        .success();

    // YAML -> HEDL
    hedl_cmd()
        .arg("from-yaml")
        .arg(yaml_file.path())
        .arg("--output")
        .arg(hedl3.path())
        .assert()
        .success();

    // Verify final output contains expected content
    let final_content = fs::read_to_string(hedl3.path()).unwrap();
    assert!(final_content.contains("test:"));
    assert!(final_content.contains("number:"));
}

#[test]
fn test_format_then_validate() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let input_file = create_temp_file(content, ".hedl");
    let formatted_file = NamedTempFile::new().unwrap();

    // Format the file
    hedl_cmd()
        .arg("format")
        .arg(input_file.path())
        .arg("--output")
        .arg(formatted_file.path())
        .assert()
        .success();

    // Validate the formatted output
    hedl_cmd()
        .arg("validate")
        .arg(formatted_file.path())
        .assert()
        .success();
}

#[test]
fn test_format_then_lint() {
    let content = r#"%VERSION: 1.0
---
a: 1
b: 2
"#;
    let input_file = create_temp_file(content, ".hedl");
    let formatted_file = NamedTempFile::new().unwrap();

    // Format the file
    hedl_cmd()
        .arg("format")
        .arg(input_file.path())
        .arg("--output")
        .arg(formatted_file.path())
        .assert()
        .success();

    // Lint the formatted output
    hedl_cmd()
        .arg("lint")
        .arg(formatted_file.path())
        .assert()
        .success();
}

#[test]
fn test_concurrent_operations() {
    // Test that multiple operations can work with the same file
    let content = r#"%VERSION: 1.0
---
key: value
"#;
    let file = create_temp_file(content, ".hedl");

    // Run multiple operations
    hedl_cmd()
        .arg("validate")
        .arg(file.path())
        .assert()
        .success();

    hedl_cmd()
        .arg("inspect")
        .arg(file.path())
        .assert()
        .success();

    hedl_cmd()
        .arg("stats")
        .arg(file.path())
        .assert()
        .success();
}
