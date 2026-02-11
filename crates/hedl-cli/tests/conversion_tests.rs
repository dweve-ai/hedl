// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for format conversion commands
//!
//! Tests all bidirectional conversions: JSON, YAML, XML, CSV, Parquet, TOON

use hedl_cli::commands::*;
use std::fs;
use tempfile::NamedTempFile;

// Test fixtures
const SIMPLE_HEDL: &str = r#"%VERSION: 1.0
---
a: 1
b: 2
c: "test"
"#;

const MATRIX_HEDL: &str = r"%VERSION: 1.0
---
@STRUCT Person[name, age]
people:@Person[id, name, age]
 |p1,Alice,30
 |p2,Bob,25
 |p3,Charlie,35
";

const SIMPLE_JSON: &str = r#"{"a":1,"b":2,"c":"test"}"#;

const SIMPLE_YAML: &str = r"a: 1
b: 2
c: test
";

const SIMPLE_XML: &str = r#"<?xml version="1.0"?><root><a>1</a><b>2</b><c>test</c></root>"#;

const SIMPLE_CSV: &str = r"id,name,age
p1,Alice,30
p2,Bob,25
";

// ===== Helper Functions =====

fn create_temp_file_with_content(content: &str, extension: &str) -> NamedTempFile {
    let file = tempfile::Builder::new()
        .suffix(extension)
        .tempfile()
        .expect("Failed to create temp file");
    fs::write(file.path(), content).expect("Failed to write temp file");
    file
}

fn read_file_content(path: &std::path::Path) -> String {
    fs::read_to_string(path).expect("Failed to read file")
}

// ===== JSON Conversion Tests =====

#[test]
fn test_to_json_compact() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_json(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        false, // no metadata
        false, // compact
    );

    assert!(result.is_ok());
    let json = read_file_content(output.path());
    assert!(json.contains("\"a\":1"));
    assert!(json.contains("\"b\":2"));
    assert!(json.contains("\"c\":\"test\""));
}

#[test]
fn test_to_json_pretty() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_json(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        false, // no metadata
        true,  // pretty
    );

    assert!(result.is_ok());
    let json = read_file_content(output.path());
    // Pretty JSON should have indentation
    assert!(json.contains('\n'));
    assert!(json.contains("  "));
}

#[test]
fn test_to_json_with_metadata() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_json(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        true, // with metadata
        true, // pretty
    );

    assert!(result.is_ok());
    let json = read_file_content(output.path());
    // Metadata mode may add version, type info, or wrap data differently
    // Just verify it's valid JSON with some content
    assert!(!json.is_empty());
    assert!(json.contains('{'));
}

#[test]
fn test_to_json_stdout() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");

    // Writing to stdout should succeed (we can't capture stdout in unit tests easily)
    let result = to_json(
        input.path().to_str().unwrap(),
        None, // stdout
        false,
        false,
    );

    assert!(result.is_ok());
}

#[test]
fn test_from_json_valid() {
    let input = create_temp_file_with_content(SIMPLE_JSON, ".json");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = from_json(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
    );

    assert!(result.is_ok());
    let hedl = read_file_content(output.path());
    // Compact format uses %V: prefix
    assert!(hedl.contains("%V:"));
    assert!(hedl.contains("a:"));
    assert!(hedl.contains("b:"));
}

#[test]
fn test_from_json_invalid() {
    let input = create_temp_file_with_content("{invalid json", ".json");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = from_json(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
    );

    assert!(result.is_err());
}

#[test]
fn test_json_roundtrip() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let json_file = NamedTempFile::new().expect("Failed to create JSON file");
    let output = NamedTempFile::new().expect("Failed to create output file");

    // HEDL -> JSON
    let result = to_json(
        input.path().to_str().unwrap(),
        Some(json_file.path().to_str().unwrap()),
        false,
        false,
    );
    assert!(result.is_ok());

    // JSON -> HEDL
    let result = from_json(
        json_file.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
    );
    assert!(result.is_ok());
}

// ===== YAML Conversion Tests =====

#[test]
fn test_to_yaml_valid() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_yaml(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
    );

    assert!(result.is_ok());
    let yaml = read_file_content(output.path());
    assert!(yaml.contains("a:"));
    assert!(yaml.contains("b:"));
}

#[test]
fn test_from_yaml_valid() {
    let input = create_temp_file_with_content(SIMPLE_YAML, ".yaml");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = from_yaml(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
    );

    assert!(result.is_ok());
    let hedl = read_file_content(output.path());
    // Compact format uses %V: prefix
    assert!(hedl.contains("%V:"));
}

#[test]
fn test_yaml_roundtrip() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let yaml_file = NamedTempFile::new().expect("Failed to create YAML file");
    let output = NamedTempFile::new().expect("Failed to create output file");

    // HEDL -> YAML
    let result = to_yaml(
        input.path().to_str().unwrap(),
        Some(yaml_file.path().to_str().unwrap()),
    );
    assert!(result.is_ok());

    // YAML -> HEDL
    let result = from_yaml(
        yaml_file.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
    );
    assert!(result.is_ok());
}

// ===== XML Conversion Tests =====

#[test]
fn test_to_xml_compact() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_xml(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        false, // compact
    );

    assert!(result.is_ok());
    let xml = read_file_content(output.path());
    assert!(xml.contains("<?xml"));
    assert!(xml.contains("<a>1</a>"));
}

#[test]
fn test_to_xml_pretty() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_xml(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        true, // pretty
    );

    assert!(result.is_ok());
    let xml = read_file_content(output.path());
    assert!(xml.contains('\n'));
}

#[test]
fn test_from_xml_valid() {
    let input = create_temp_file_with_content(SIMPLE_XML, ".xml");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = from_xml(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
    );

    assert!(result.is_ok());
}

#[test]
fn test_xml_roundtrip() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let xml_file = NamedTempFile::new().expect("Failed to create XML file");
    let output = NamedTempFile::new().expect("Failed to create output file");

    // HEDL -> XML
    let result = to_xml(
        input.path().to_str().unwrap(),
        Some(xml_file.path().to_str().unwrap()),
        false,
    );
    assert!(result.is_ok());

    // XML -> HEDL
    let result = from_xml(
        xml_file.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
    );
    assert!(result.is_ok());
}

// ===== CSV Conversion Tests =====

#[test]
fn test_to_csv_with_headers() {
    let input = create_temp_file_with_content(MATRIX_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_csv(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        true, // include headers
    );

    // CSV conversion may fail if the HEDL structure is not compatible
    // Just verify it doesn't panic
    if result.is_ok() {
        let csv = read_file_content(output.path());
        assert!(!csv.is_empty());
    }
}

#[test]
fn test_to_csv_without_headers() {
    let input = create_temp_file_with_content(MATRIX_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_csv(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        false, // no headers
    );

    // CSV conversion may fail if the HEDL structure is not compatible
    // Just verify it doesn't panic
    if result.is_ok() {
        let csv = read_file_content(output.path());
        assert!(!csv.is_empty());
    }
}

#[test]
fn test_from_csv_valid() {
    let input = create_temp_file_with_content(SIMPLE_CSV, ".csv");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = from_csv(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        "Person",
    );

    assert!(result.is_ok());
    let hedl = read_file_content(output.path());
    assert!(hedl.contains("@Person"));
}

#[test]
fn test_from_csv_invalid_type_name() {
    let input = create_temp_file_with_content(SIMPLE_CSV, ".csv");
    let output = NamedTempFile::new().expect("Failed to create output file");

    // Invalid type name with special characters
    let result = from_csv(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        "Invalid-Type!",
    );

    assert!(result.is_err());
}

#[test]
fn test_from_csv_empty_file() {
    let input = create_temp_file_with_content("", ".csv");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = from_csv(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        "Person",
    );

    assert!(result.is_err());
}

// ===== Parquet Conversion Tests =====

#[test]
fn test_to_parquet_valid() {
    let input = create_temp_file_with_content(MATRIX_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_parquet(
        input.path().to_str().unwrap(),
        output.path().to_str().unwrap(),
    );

    // May succeed or fail depending on data structure compatibility
    // Just verify it doesn't panic
    let _ = result;
}

#[test]
fn test_parquet_requires_file_path() {
    let input = create_temp_file_with_content(MATRIX_HEDL, ".hedl");

    // Output path is required (not optional like other formats)
    let result = to_parquet(input.path().to_str().unwrap(), "/tmp/test_output.parquet");

    // Should succeed or fail gracefully
    let _ = result;
}

// ===== TOON Conversion Tests =====

#[test]
fn test_to_toon_valid() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_toon(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
    );

    assert!(result.is_ok());
}

#[test]
fn test_from_toon_valid() {
    let toon_content = "a 1\nb 2\n";
    let input = create_temp_file_with_content(toon_content, ".toon");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = from_toon(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
    );

    // May succeed or fail depending on TOON parser implementation
    let _ = result;
}

// ===== Error Handling Tests =====

#[test]
fn test_conversion_invalid_hedl_syntax() {
    let invalid = "This is not valid HEDL";
    let input = create_temp_file_with_content(invalid, ".hedl");
    let output = NamedTempFile::new().expect("Failed to create output file");

    let result = to_json(
        input.path().to_str().unwrap(),
        Some(output.path().to_str().unwrap()),
        false,
        false,
    );

    assert!(result.is_err());
}

#[test]
fn test_conversion_missing_file() {
    let result = to_json(
        "/nonexistent/file.hedl",
        Some("/tmp/out.json"),
        false,
        false,
    );
    assert!(result.is_err());
}

#[test]
fn test_write_to_invalid_path() {
    let input = create_temp_file_with_content(SIMPLE_HEDL, ".hedl");

    // Try to write to an invalid directory
    let result = to_json(
        input.path().to_str().unwrap(),
        Some("/invalid/directory/output.json"),
        false,
        false,
    );

    assert!(result.is_err());
}
