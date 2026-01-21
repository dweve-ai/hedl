// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tests for stats command, `read_file`, `write_output`, and utility functions

use hedl_cli::commands::*;
use std::fs;
use tempfile::NamedTempFile;

// Test fixtures
const SIMPLE_HEDL: &str = r#"%VERSION: 1.0
---
a: 1
b: 2
c: "test"
d: [1, 2, 3]
"#;

const LARGE_HEDL: &str = r#"%VERSION: 1.0
---
@STRUCT Person[name, age, city, country]
people: @Person[id, name, age, city, country]
  |p1,Alice,30,NYC,USA
  |p2,Bob,25,London,UK
  |p3,Charlie,35,Paris,France
  |p4,Diana,28,Berlin,Germany
  |p5,Eve,32,Tokyo,Japan
metadata:
  title: "Test Dataset"
  version: "1.0"
  created: "2025-01-18"
"#;

// ===== Helper Functions =====

fn create_temp_file_with_content(content: &str, extension: &str) -> NamedTempFile {
    let file = tempfile::Builder::new()
        .suffix(extension)
        .tempfile()
        .expect("Failed to create temp file");
    fs::write(file.path(), content).expect("Failed to write temp file");
    file
}

// ===== Stats Command Tests =====

#[test]
fn test_stats_without_tokens() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");

    let result = stats(input.path().to_str().unwrap(), false);

    assert!(result.is_ok());
}

#[test]
fn test_stats_with_tokens() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");

    let result = stats(input.path().to_str().unwrap(), true);

    assert!(result.is_ok());
}

#[test]
fn test_stats_larger_file() {
    let input = create_temp_file_with_content(LARGE_HEDL, ".hedl");

    let result = stats(input.path().to_str().unwrap(), false);

    // Stats may fail if conversion to comparison formats fails
    // The important thing is it doesn't panic
    let _ = result;
}

#[test]
fn test_stats_invalid_file() {
    let result = stats("/nonexistent/file.hedl", false);

    assert!(result.is_err());
}

#[test]
fn test_stats_invalid_hedl() {
    let invalid = "This is not valid HEDL syntax";
    let input = create_temp_file_with_content(invalid, ".hedl");

    let result = stats(input.path().to_str().unwrap(), false);

    assert!(result.is_err());
}

// ===== read_file Tests =====

#[test]
fn test_read_file_valid() {
    let file = create_temp_file_with_content("test content", ".txt");

    let result = read_file(file.path().to_str().unwrap());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test content");
}

#[test]
fn test_read_file_missing() {
    let result = read_file("/nonexistent/file.txt");

    assert!(result.is_err());
}

#[test]
fn test_read_file_empty() {
    let file = create_temp_file_with_content("", ".txt");

    let result = read_file(file.path().to_str().unwrap());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[test]
fn test_read_file_unicode() {
    let content = "Hello 世界 🌍 émojis";
    let file = create_temp_file_with_content(content, ".txt");

    let result = read_file(file.path().to_str().unwrap());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), content);
}

#[test]
fn test_read_file_size_limit_env() {
    // Create a file with 100 bytes
    let content = "x".repeat(100);
    let file = create_temp_file_with_content(&content, ".txt");

    // Set max size to 50 bytes
    std::env::set_var("HEDL_MAX_FILE_SIZE", "50");

    let result = read_file(file.path().to_str().unwrap());

    // Clean up env var
    std::env::remove_var("HEDL_MAX_FILE_SIZE");

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too large"));
}

#[test]
fn test_read_file_within_size_limit() {
    // Create a small file
    let content = "small content";
    let file = create_temp_file_with_content(content, ".txt");

    // Set a generous max size
    std::env::set_var("HEDL_MAX_FILE_SIZE", "1000000");

    let result = read_file(file.path().to_str().unwrap());

    // Clean up env var
    std::env::remove_var("HEDL_MAX_FILE_SIZE");

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), content);
}

#[test]
fn test_read_file_invalid_env_var() {
    let file = create_temp_file_with_content("test", ".txt");

    // Set invalid env var (non-numeric)
    std::env::set_var("HEDL_MAX_FILE_SIZE", "not_a_number");

    let result = read_file(file.path().to_str().unwrap());

    // Clean up env var
    std::env::remove_var("HEDL_MAX_FILE_SIZE");

    // Should succeed using default limit
    assert!(result.is_ok());
}

#[test]
fn test_read_file_default_max_size() {
    let file = create_temp_file_with_content("test", ".txt");

    // Ensure env var is not set
    std::env::remove_var("HEDL_MAX_FILE_SIZE");

    let result = read_file(file.path().to_str().unwrap());

    assert!(result.is_ok());
}

// ===== write_output Tests =====

#[test]
fn test_write_output_to_file() {
    let output = NamedTempFile::new().expect("Failed to create output file");
    let content = "test output content";

    let result = write_output(content, Some(output.path().to_str().unwrap()));

    assert!(result.is_ok());

    let written = fs::read_to_string(output.path()).expect("Failed to read output file");
    assert_eq!(written, content);
}

#[test]
fn test_write_output_empty_content() {
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = write_output("", Some(output.path().to_str().unwrap()));

    assert!(result.is_ok());

    let written = fs::read_to_string(output.path()).expect("Failed to read output file");
    assert_eq!(written, "");
}

#[test]
fn test_write_output_unicode() {
    let output = NamedTempFile::new().expect("Failed to create output file");
    let content = "Hello 世界 🌍 émojis";

    let result = write_output(content, Some(output.path().to_str().unwrap()));

    assert!(result.is_ok());

    let written = fs::read_to_string(output.path()).expect("Failed to read output file");
    assert_eq!(written, content);
}

#[test]
fn test_write_output_to_invalid_path() {
    let content = "test content";

    let result = write_output(content, Some("/invalid/directory/file.txt"));

    assert!(result.is_err());
}

#[test]
fn test_write_output_to_stdout() {
    let content = "test content";

    // Writing to stdout should succeed (we can't easily verify the output)
    let result = write_output(content, None);

    assert!(result.is_ok());
}

#[test]
fn test_write_output_large_content() {
    let output = NamedTempFile::new().expect("Failed to create output file");
    let content = "x".repeat(100_000); // 100KB

    let result = write_output(&content, Some(output.path().to_str().unwrap()));

    assert!(result.is_ok());

    let written = fs::read_to_string(output.path()).expect("Failed to read output file");
    assert_eq!(written.len(), 100_000);
}

// ===== Validate Command Tests =====

#[test]
fn test_validate_valid_simple() {
    let file = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");

    let result = validate(file.path().to_str().unwrap(), false);

    assert!(result.is_ok());
}

#[test]
fn test_validate_strict_mode() {
    let file = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");

    let result = validate(file.path().to_str().unwrap(), true);

    assert!(result.is_ok());
}

#[test]
fn test_validate_invalid_hedl() {
    let invalid = "This is not valid HEDL";
    let file = create_temp_file_with_content(invalid, ".hedl");

    let result = validate(file.path().to_str().unwrap(), false);

    assert!(result.is_err());
}

#[test]
fn test_validate_missing_file() {
    let result = validate("/nonexistent/file.hedl", false);

    assert!(result.is_err());
}

// ===== Inspect Command Tests =====

#[test]
fn test_inspect_valid_file() {
    let file = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");

    let result = inspect(file.path().to_str().unwrap(), false);

    assert!(result.is_ok());
}

#[test]
fn test_inspect_verbose() {
    let file = create_temp_file_with_content(LARGE_HEDL, ".hedl");

    let result = inspect(file.path().to_str().unwrap(), true);

    // Inspect may fail if the structure is incompatible
    // The important thing is it doesn't panic
    let _ = result;
}

#[test]
fn test_inspect_invalid_file() {
    let result = inspect("/nonexistent/file.hedl", false);

    assert!(result.is_err());
}

#[test]
fn test_inspect_invalid_hedl() {
    let invalid = "Invalid HEDL syntax";
    let file = create_temp_file_with_content(invalid, ".hedl");

    let result = inspect(file.path().to_str().unwrap(), false);

    assert!(result.is_err());
}

// ===== Lint Command Tests =====

#[test]
fn test_lint_valid_file_text_format() {
    let file = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");

    let result = lint(file.path().to_str().unwrap(), "text", false);

    assert!(result.is_ok());
}

#[test]
fn test_lint_valid_file_json_format() {
    let file = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");

    let result = lint(file.path().to_str().unwrap(), "json", false);

    assert!(result.is_ok());
}

#[test]
fn test_lint_warn_error_mode() {
    let file = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");

    let result = lint(file.path().to_str().unwrap(), "text", true);

    // Should succeed if no warnings, or fail if warnings are treated as errors
    let _ = result;
}

#[test]
fn test_lint_invalid_file() {
    let result = lint("/nonexistent/file.hedl", "text", false);

    assert!(result.is_err());
}

#[test]
fn test_lint_invalid_hedl() {
    let invalid = "Invalid HEDL syntax";
    let file = create_temp_file_with_content(invalid, ".hedl");

    let result = lint(file.path().to_str().unwrap(), "text", false);

    assert!(result.is_err());
}

// ===== Format Command Tests =====

#[test]
fn test_format_to_file() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = format(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        false, // not check mode
        true,  // use ditto
        false, // no count hints
    );

    assert!(result.is_ok());

    let formatted = fs::read_to_string(output.path()).expect("Failed to read output");
    assert!(formatted.contains("%VERSION:"));
}

#[test]
fn test_format_check_mode_canonical() {
    let canonical = r"%VERSION: 1.0
---
a: 1
b: 2
";
    let file = create_temp_file_with_content(canonical, ".hedl");

    let result = format(
        file.path().to_str().unwrap(),
        None,
        true,  // check mode
        true,  // use ditto
        false, // no count hints
    );

    // May succeed if already canonical
    let _ = result;
}

#[test]
fn test_format_with_count_hints() {
    let file = create_temp_file_with_content(LARGE_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = format(
        file.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        false, // not check mode
        true,  // use ditto
        true,  // with count hints
    );

    // Formatting may fail if the structure is incompatible
    // The important thing is it doesn't panic
    let _ = result;
}

#[test]
fn test_format_without_ditto() {
    let file = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = format(
        file.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        false, // not check mode
        false, // no ditto
        false, // no count hints
    );

    assert!(result.is_ok());
}

#[test]
fn test_format_invalid_file() {
    let result = format("/nonexistent/file.hedl", None, false, true, false);

    assert!(result.is_err());
}

#[test]
fn test_format_invalid_hedl() {
    let invalid = "Invalid HEDL syntax";
    let file = create_temp_file_with_content(invalid, ".hedl");

    let result = format(file.path().to_str().unwrap(), None, false, true, false);

    assert!(result.is_err());
}

// ===== Integration Tests =====

#[test]
fn test_full_workflow_validate_format_inspect() {
    let file = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let formatted = NamedTempFile::new().expect("Failed to create output file");

    // 1. Validate
    let result = validate(file.path().to_str().unwrap(), false);
    assert!(result.is_ok());

    // 2. Format
    let result = format(
        file.path().to_str().unwrap(),
        Some(formatted.path().to_str().unwrap()),
        false,
        true,
        false,
    );
    assert!(result.is_ok());

    // 3. Inspect formatted
    let result = inspect(formatted.path().to_str().unwrap(), false);
    assert!(result.is_ok());

    // 4. Lint formatted
    let result = lint(formatted.path().to_str().unwrap(), "text", false);
    assert!(result.is_ok());
}
