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

//! Property-based tests for HEDL CLI operations using proptest.
//!
//! This module tests key properties and invariants of HEDL CLI commands:
//! - Format idempotence: formatting twice produces identical results
//! - Round-trip conversion: HEDL -> JSON -> HEDL preserves data
//! - Validation consistency: valid documents remain valid after formatting
//! - Error handling: invalid inputs produce appropriate errors
//! - Size limits: files exceeding limits are rejected

use assert_cmd::Command;
use hedl_cli::commands::{format, from_json, read_file, to_json, validate};
use hedl_core::parse;
use proptest::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

// ===== Test Helpers =====

/// Create a temporary file with content
fn create_temp_file(content: &str, suffix: &str) -> NamedTempFile {
    let file = tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .expect("Failed to create temp file");
    fs::write(file.path(), content).expect("Failed to write temp file");
    file
}

/// Create a HEDL command builder
fn hedl_cmd() -> Command {
    Command::cargo_bin("hedl").expect("Failed to find hedl binary")
}

// ===== Property-Based Test Generators =====

/// Generate valid HEDL identifiers (lowercase + underscores + digits, starting with lowercase or underscore)
/// HEDL identifiers must match: [a-z_][a-z0-9_]*
/// Note: Excludes keys starting with double underscore (__) as they may be reserved
fn identifier() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z_][a-z0-9_]{0,19}")
        .expect("Failed to create identifier regex")
        .prop_filter("Exclude double underscore prefix", |s| !s.starts_with("__"))
}

/// Generate valid HEDL type names (PascalCase identifiers)
fn type_name() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-zA-Z0-9]{0,19}")
        .expect("Failed to create type name regex")
}

/// Generate valid HEDL string values (escaped quotes and backslashes)
fn hedl_string() -> impl Strategy<Value = String> {
    prop::string::string_regex(r#"[a-zA-Z0-9 .,!?()-]{0,100}"#)
        .expect("Failed to create string regex")
}

/// Generate valid HEDL integers
fn hedl_int() -> impl Strategy<Value = i64> {
    any::<i64>()
}

/// Generate valid HEDL floats (avoiding NaN and infinity)
#[allow(dead_code)]
fn hedl_float() -> impl Strategy<Value = f64> {
    any::<f64>().prop_filter("Must be finite", |f| f.is_finite())
}

/// Generate simple valid HEDL documents with scalar values
fn simple_hedl_document() -> impl Strategy<Value = String> {
    (identifier(), hedl_string()).prop_map(|(key, value)| {
        format!(
            "%VERSION: 1.0\n---\n{}: \"{}\"",
            key,
            value.replace('\\', "\\\\").replace('"', "\\\"")
        )
    })
}

/// Generate HEDL documents with multiple scalar fields
fn multi_field_document() -> impl Strategy<Value = String> {
    prop::collection::vec((identifier(), hedl_string()), 1..10).prop_map(|fields| {
        let mut doc = String::from("%VERSION: 1.0\n---\n");
        let mut used_keys = std::collections::HashSet::new();
        for (key, value) in fields {
            // Ensure unique keys by appending a counter if needed
            let mut unique_key = key.clone();
            let mut counter = 1;
            while !used_keys.insert(unique_key.clone()) {
                unique_key = format!("{}_{}", key, counter);
                counter += 1;
            }
            doc.push_str(&format!(
                "{}: \"{}\"\n",
                unique_key,
                value.replace('\\', "\\\\").replace('"', "\\\"")
            ));
        }
        doc
    })
}

/// Generate HEDL documents with integer values
fn integer_document() -> impl Strategy<Value = String> {
    prop::collection::vec((identifier(), hedl_int()), 1..10).prop_map(|fields| {
        let mut doc = String::from("%VERSION: 1.0\n---\n");
        let mut used_keys = std::collections::HashSet::new();
        for (key, value) in fields {
            let mut unique_key = key.clone();
            let mut counter = 1;
            while !used_keys.insert(unique_key.clone()) {
                unique_key = format!("{}_{}", key, counter);
                counter += 1;
            }
            doc.push_str(&format!("{}: {}\n", unique_key, value));
        }
        doc
    })
}

/// Generate HEDL documents with boolean values
fn boolean_document() -> impl Strategy<Value = String> {
    prop::collection::vec((identifier(), any::<bool>()), 1..10).prop_map(|fields| {
        let mut doc = String::from("%VERSION: 1.0\n---\n");
        let mut used_keys = std::collections::HashSet::new();
        for (key, value) in fields {
            let mut unique_key = key.clone();
            let mut counter = 1;
            while !used_keys.insert(unique_key.clone()) {
                unique_key = format!("{}_{}", key, counter);
                counter += 1;
            }
            doc.push_str(&format!("{}: {}\n", unique_key, value));
        }
        doc
    })
}

/// Generate HEDL documents with matrix lists
fn matrix_list_document() -> impl Strategy<Value = String> {
    (
        type_name(),
        identifier(),
        prop::collection::vec((identifier(), hedl_string()), 1..5),
    )
        .prop_map(|(type_name, list_name, rows)| {
            let mut doc = format!("%VERSION: 1.0\n---\n{}: @{}[id, name]\n", list_name, type_name);
            let mut used_ids = std::collections::HashSet::new();
            for (id, name) in rows.iter() {
                // Ensure unique row IDs
                let mut unique_id = id.clone();
                let mut counter = 1;
                while !used_ids.insert(unique_id.clone()) {
                    unique_id = format!("{}_{}", id, counter);
                    counter += 1;
                }
                // Quote the name value to handle commas and special characters properly
                doc.push_str(&format!(
                    "  | {}, \"{}\"\n",
                    unique_id,
                    name.replace('\\', "\\\\").replace('"', "\\\"")
                ));
            }
            doc
        })
}

/// Generate HEDL documents with null values
fn null_document() -> impl Strategy<Value = String> {
    prop::collection::vec(identifier(), 1..10).prop_map(|fields| {
        let mut doc = String::from("%VERSION: 1.0\n---\n");
        let mut used_keys = std::collections::HashSet::new();
        for key in fields {
            let mut unique_key = key.clone();
            let mut counter = 1;
            while !used_keys.insert(unique_key.clone()) {
                unique_key = format!("{}_{}", key, counter);
                counter += 1;
            }
            doc.push_str(&format!("{}: ~\n", unique_key));
        }
        doc
    })
}

/// Generate mixed-type HEDL documents
fn mixed_document() -> impl Strategy<Value = String> {
    (
        prop::collection::vec(identifier(), 4..5),
        hedl_string(),
        hedl_int(),
        any::<bool>(),
    )
        .prop_map(|(keys, v1, v2, v3)| {
            // Ensure unique keys
            let mut used_keys = std::collections::HashSet::new();
            let mut unique_keys = Vec::new();
            for key in keys {
                let mut unique_key = key.clone();
                let mut counter = 1;
                while !used_keys.insert(unique_key.clone()) {
                    unique_key = format!("{}_{}", key, counter);
                    counter += 1;
                }
                unique_keys.push(unique_key);
            }
            // Pad with additional unique keys if needed
            while unique_keys.len() < 4 {
                let mut key = format!("key{}", unique_keys.len());
                while used_keys.contains(&key) {
                    key = format!("key{}_{}", unique_keys.len(), used_keys.len());
                }
                used_keys.insert(key.clone());
                unique_keys.push(key);
            }
            format!(
                "%VERSION: 1.0\n---\n{}: \"{}\"\n{}: {}\n{}: {}\n{}: ~\n",
                unique_keys[0],
                v1.replace('\\', "\\\\").replace('"', "\\\""),
                unique_keys[1],
                v2,
                unique_keys[2],
                v3,
                unique_keys[3]
            )
        })
}

// ===== Property Tests: Format Command =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Formatting is idempotent - format(format(x)) == format(x)
    #[test]
    fn prop_format_idempotent(doc in simple_hedl_document()) {
        let file1 = create_temp_file(&doc, ".hedl");
        let _file2 = create_temp_file(&doc, ".hedl");
        let out1 = create_temp_file("", ".hedl");
        let out2 = create_temp_file("", ".hedl");

        // Format once
        let result1 = format(
            file1.path().to_str().unwrap(),
            Some(out1.path().to_str().unwrap()),
            false,
            true,
            false,
        );
        prop_assert!(result1.is_ok(), "First format failed: {:?}", result1.err());

        // Format the formatted output
        let result2 = format(
            out1.path().to_str().unwrap(),
            Some(out2.path().to_str().unwrap()),
            false,
            true,
            false,
        );
        prop_assert!(result2.is_ok(), "Second format failed: {:?}", result2.err());

        // Read both outputs
        let formatted1 = fs::read_to_string(out1.path()).unwrap();
        let formatted2 = fs::read_to_string(out2.path()).unwrap();

        // Should be identical
        prop_assert_eq!(formatted1, formatted2, "Formatting is not idempotent");
    }

    /// Property: All valid documents can be formatted without errors
    #[test]
    fn prop_format_accepts_valid_docs(doc in multi_field_document()) {
        let file = create_temp_file(&doc, ".hedl");
        let output = create_temp_file("", ".hedl");

        let result = format(
            file.path().to_str().unwrap(),
            Some(output.path().to_str().unwrap()),
            false,
            true,
            false,
        );

        prop_assert!(result.is_ok(), "Format rejected valid document: {:?}", result.err());
    }

    /// Property: Formatted output can be parsed successfully
    #[test]
    fn prop_formatted_output_is_parseable(doc in integer_document()) {
        let file = create_temp_file(&doc, ".hedl");
        let output = create_temp_file("", ".hedl");

        format(
            file.path().to_str().unwrap(),
            Some(output.path().to_str().unwrap()),
            false,
            true,
            false,
        ).unwrap();

        let formatted_content = fs::read_to_string(output.path()).unwrap();
        let parse_result = parse(formatted_content.as_bytes());

        prop_assert!(parse_result.is_ok(), "Formatted output is not parseable: {:?}", parse_result.err());
    }

    /// Property: Format with ditto flag preserves data
    #[test]
    fn prop_format_ditto_preserves_data(doc in boolean_document()) {
        let file = create_temp_file(&doc, ".hedl");
        let out_ditto = create_temp_file("", ".hedl");
        let out_no_ditto = create_temp_file("", ".hedl");

        // Format with ditto
        format(
            file.path().to_str().unwrap(),
            Some(out_ditto.path().to_str().unwrap()),
            false,
            true,
            false,
        ).unwrap();

        // Format without ditto
        format(
            file.path().to_str().unwrap(),
            Some(out_no_ditto.path().to_str().unwrap()),
            false,
            false,
            false,
        ).unwrap();

        // Parse both outputs
        let ditto_content = fs::read_to_string(out_ditto.path()).unwrap();
        let no_ditto_content = fs::read_to_string(out_no_ditto.path()).unwrap();

        let ditto_doc = parse(ditto_content.as_bytes()).unwrap();
        let no_ditto_doc = parse(no_ditto_content.as_bytes()).unwrap();

        // Data should be semantically equivalent
        prop_assert_eq!(ditto_doc.root, no_ditto_doc.root, "Ditto and no-ditto produce different data");
    }

    /// Property: Format with counts adds count hints to all lists
    #[test]
    fn prop_format_with_counts_adds_hints(doc in matrix_list_document()) {
        let file = create_temp_file(&doc, ".hedl");
        let output = create_temp_file("", ".hedl");

        format(
            file.path().to_str().unwrap(),
            Some(output.path().to_str().unwrap()),
            false,
            true,
            true, // with_counts = true
        ).unwrap();

        let formatted_content = fs::read_to_string(output.path()).unwrap();

        // Count hints should be present in the formatted output
        // Format is: type_name(N): @Type[...]
        prop_assert!(
            formatted_content.contains("(") && formatted_content.contains("):"),
            "Formatted output missing count hints"
        );
    }
}

// ===== Property Tests: Validate Command =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: All parseable documents pass validation
    #[test]
    fn prop_validate_accepts_parseable(doc in simple_hedl_document()) {
        let file = create_temp_file(&doc, ".hedl");

        let result = validate(file.path().to_str().unwrap(), false);

        prop_assert!(result.is_ok(), "Validation rejected parseable document: {:?}", result.err());
    }

    /// Property: Formatted documents always pass validation
    #[test]
    fn prop_formatted_docs_validate(doc in multi_field_document()) {
        let file = create_temp_file(&doc, ".hedl");
        let output = create_temp_file("", ".hedl");

        format(
            file.path().to_str().unwrap(),
            Some(output.path().to_str().unwrap()),
            false,
            true,
            false,
        ).unwrap();

        let result = validate(output.path().to_str().unwrap(), false);

        prop_assert!(result.is_ok(), "Formatted document failed validation: {:?}", result.err());
    }

    /// Property: Validation is consistent across multiple calls
    #[test]
    fn prop_validation_is_consistent(doc in integer_document()) {
        let file = create_temp_file(&doc, ".hedl");

        let result1 = validate(file.path().to_str().unwrap(), false);
        let result2 = validate(file.path().to_str().unwrap(), false);

        prop_assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "Validation is not consistent across calls"
        );
    }
}

// ===== Property Tests: Conversion Round-Trips =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: HEDL -> JSON -> HEDL round-trip preserves data
    #[test]
    fn prop_json_roundtrip_preserves_data(doc in simple_hedl_document()) {
        let hedl_file = create_temp_file(&doc, ".hedl");
        let json_file = create_temp_file("", ".json");
        let hedl_output = create_temp_file("", ".hedl");

        // HEDL -> JSON
        let to_json_result = to_json(
            hedl_file.path().to_str().unwrap(),
            Some(json_file.path().to_str().unwrap()),
            false,
            true,
        );
        prop_assert!(to_json_result.is_ok(), "HEDL to JSON conversion failed: {:?}", to_json_result.err());

        // JSON -> HEDL
        let from_json_result = from_json(
            json_file.path().to_str().unwrap(),
            Some(hedl_output.path().to_str().unwrap()),
        );
        prop_assert!(from_json_result.is_ok(), "JSON to HEDL conversion failed: {:?}", from_json_result.err());

        // Parse original and round-tripped documents
        let original_content = fs::read_to_string(hedl_file.path()).unwrap();
        let roundtrip_content = fs::read_to_string(hedl_output.path()).unwrap();

        let original_doc = parse(original_content.as_bytes()).unwrap();
        let roundtrip_doc = parse(roundtrip_content.as_bytes()).unwrap();

        // Data should be preserved (root items should be equal)
        prop_assert_eq!(
            original_doc.root,
            roundtrip_doc.root,
            "JSON round-trip did not preserve data"
        );
    }

    /// Property: JSON conversion produces valid JSON
    #[test]
    fn prop_json_conversion_produces_valid_json(doc in multi_field_document()) {
        let hedl_file = create_temp_file(&doc, ".hedl");
        let json_file = create_temp_file("", ".json");

        to_json(
            hedl_file.path().to_str().unwrap(),
            Some(json_file.path().to_str().unwrap()),
            false,
            true,
        ).unwrap();

        let json_content = fs::read_to_string(json_file.path()).unwrap();
        let parse_result: Result<serde_json::Value, _> = serde_json::from_str(&json_content);

        prop_assert!(parse_result.is_ok(), "Generated JSON is not valid: {:?}", parse_result.err());
    }

    /// Property: Pretty and compact JSON conversions preserve same data
    #[test]
    fn prop_json_pretty_vs_compact(doc in integer_document()) {
        let hedl_file = create_temp_file(&doc, ".hedl");
        let json_pretty = create_temp_file("", ".json");
        let json_compact = create_temp_file("", ".json");

        // Convert to pretty JSON
        to_json(
            hedl_file.path().to_str().unwrap(),
            Some(json_pretty.path().to_str().unwrap()),
            false,
            true, // pretty
        ).unwrap();

        // Convert to compact JSON
        to_json(
            hedl_file.path().to_str().unwrap(),
            Some(json_compact.path().to_str().unwrap()),
            false,
            false, // compact
        ).unwrap();

        // Parse both JSON outputs
        let pretty_content = fs::read_to_string(json_pretty.path()).unwrap();
        let compact_content = fs::read_to_string(json_compact.path()).unwrap();

        let pretty_value: serde_json::Value = serde_json::from_str(&pretty_content).unwrap();
        let compact_value: serde_json::Value = serde_json::from_str(&compact_content).unwrap();

        // Data should be identical
        prop_assert_eq!(
            pretty_value,
            compact_value,
            "Pretty and compact JSON contain different data"
        );
    }
}

// ===== Property Tests: Error Handling =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Invalid syntax produces descriptive errors
    #[test]
    fn prop_invalid_syntax_gives_error(key in identifier()) {
        // Create intentionally invalid HEDL (missing space after colon)
        let invalid_doc = format!("%VERSION: 1.0\n---\n{}:invalid", key);
        let file = create_temp_file(&invalid_doc, ".hedl");

        let result = validate(file.path().to_str().unwrap(), false);

        prop_assert!(result.is_err(), "Validation should fail for invalid syntax");
    }

    /// Property: Missing version header produces error
    #[test]
    fn prop_missing_version_gives_error(doc in simple_hedl_document()) {
        // Remove version header
        let no_version = doc.replace("%VERSION: 1.0\n", "");
        let file = create_temp_file(&no_version, ".hedl");

        let result = validate(file.path().to_str().unwrap(), false);

        prop_assert!(result.is_err(), "Validation should fail for missing version");
    }
}

// ===== Property Tests: File Size Limits =====

#[test]
fn test_file_size_limit_respected() {
    // Create a document that's just under 1 KB
    let small_doc = format!("%VERSION: 1.0\n---\n{}\n", "a: 1\n".repeat(50));
    let file = create_temp_file(&small_doc, ".hedl");

    // Set very small file size limit (1 KB)
    std::env::set_var("HEDL_MAX_FILE_SIZE", "1024");

    let result = read_file(file.path().to_str().unwrap());

    // Should succeed since file is under limit
    assert!(result.is_ok(), "Small file should be readable");

    // Clean up
    std::env::remove_var("HEDL_MAX_FILE_SIZE");
}

#[test]
fn test_oversized_file_rejected() {
    // Create a document larger than our test limit
    let large_doc = format!("%VERSION: 1.0\n---\n{}\n", "a: 1\n".repeat(1000));
    let file = create_temp_file(&large_doc, ".hedl");

    // Set very small file size limit (100 bytes)
    std::env::set_var("HEDL_MAX_FILE_SIZE", "100");

    let result = read_file(file.path().to_str().unwrap());

    // Should fail since file exceeds limit
    assert!(result.is_err(), "Oversized file should be rejected");
    assert!(
        result.unwrap_err().contains("too large"),
        "Error message should mention file size"
    );

    // Clean up
    std::env::remove_var("HEDL_MAX_FILE_SIZE");
}

// ===== Property Tests: Special Cases =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// Property: Empty documents (just header) are valid
    #[test]
    fn prop_empty_document_is_valid(_seed in any::<u64>()) {
        let doc = "%VERSION: 1.0\n---\n";
        let file = create_temp_file(doc, ".hedl");

        let result = validate(file.path().to_str().unwrap(), false);

        prop_assert!(result.is_ok(), "Empty document should be valid");
    }

    /// Property: Documents with only null values are valid
    #[test]
    fn prop_null_only_document_is_valid(doc in null_document()) {
        let file = create_temp_file(&doc, ".hedl");

        let result = validate(file.path().to_str().unwrap(), false);

        prop_assert!(result.is_ok(), "Null-only document should be valid: {:?}", result.err());
    }

    /// Property: Mixed-type documents are valid
    #[test]
    fn prop_mixed_type_document_is_valid(doc in mixed_document()) {
        let file = create_temp_file(&doc, ".hedl");

        let result = validate(file.path().to_str().unwrap(), false);

        prop_assert!(result.is_ok(), "Mixed-type document should be valid: {:?}", result.err());
    }
}

// ===== Property Tests: Batch Operations =====

#[test]
fn test_format_preserves_data_for_all_types() {
    use hedl_cli::commands::format;

    let test_docs = vec![
        // String document
        "%VERSION: 1.0\n---\nname: \"test\"\n",
        // Integer document
        "%VERSION: 1.0\n---\ncount: 42\n",
        // Boolean document
        "%VERSION: 1.0\n---\nflag: true\n",
        // Null document
        "%VERSION: 1.0\n---\nvalue: ~\n",
        // Float document
        "%VERSION: 1.0\n---\npi: 3.14159\n",
    ];

    for (i, doc) in test_docs.iter().enumerate() {
        let file = create_temp_file(doc, ".hedl");
        let output = create_temp_file("", ".hedl");

        let result = format(
            file.path().to_str().unwrap(),
            Some(output.path().to_str().unwrap()),
            false,
            true,
            false,
        );

        assert!(
            result.is_ok(),
            "Format failed for test case {}: {:?}",
            i,
            result.err()
        );

        // Verify formatted output is parseable
        let formatted_content = fs::read_to_string(output.path()).unwrap();
        let parse_result = parse(formatted_content.as_bytes());

        assert!(
            parse_result.is_ok(),
            "Formatted output for test case {} is not parseable: {:?}",
            i,
            parse_result.err()
        );
    }
}

// ===== Integration Tests: CLI Commands =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Property: CLI format command produces same results as library function
    #[test]
    fn prop_cli_format_matches_library(doc in simple_hedl_document()) {
        let file = create_temp_file(&doc, ".hedl");
        let cli_output = create_temp_file("", ".hedl");
        let lib_output = create_temp_file("", ".hedl");

        // Format via CLI
        hedl_cmd()
            .arg("format")
            .arg(file.path())
            .arg("-o")
            .arg(cli_output.path())
            .assert()
            .success();

        // Format via library
        format(
            file.path().to_str().unwrap(),
            Some(lib_output.path().to_str().unwrap()),
            false,
            true,
            false,
        ).unwrap();

        // Results should be identical
        let cli_content = fs::read_to_string(cli_output.path()).unwrap();
        let lib_content = fs::read_to_string(lib_output.path()).unwrap();

        prop_assert_eq!(cli_content, lib_content, "CLI and library format produce different results");
    }

    /// Property: CLI validate command agrees with library validation
    #[test]
    fn prop_cli_validate_matches_library(doc in multi_field_document()) {
        let file = create_temp_file(&doc, ".hedl");

        // Validate via CLI
        let cli_result = hedl_cmd()
            .arg("validate")
            .arg(file.path())
            .assert();

        // Validate via library
        let lib_result = validate(file.path().to_str().unwrap(), false);

        // Results should agree
        prop_assert_eq!(
            cli_result.get_output().status.success(),
            lib_result.is_ok(),
            "CLI and library validation disagree"
        );
    }
}

// ===== Property Tests: Invariants =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Invariant: parse(canonicalize(parse(x))) == parse(x) for all valid x
    #[test]
    fn invariant_parse_canonicalize_parse(doc in simple_hedl_document()) {
        let file = create_temp_file(&doc, ".hedl");
        let formatted = create_temp_file("", ".hedl");

        // Parse original
        let content1 = fs::read_to_string(file.path()).unwrap();
        let doc1 = parse(content1.as_bytes()).unwrap();

        // Format and parse
        format(
            file.path().to_str().unwrap(),
            Some(formatted.path().to_str().unwrap()),
            false,
            true,
            false,
        ).unwrap();

        let content2 = fs::read_to_string(formatted.path()).unwrap();
        let doc2 = parse(content2.as_bytes()).unwrap();

        // Documents should be semantically equal
        prop_assert_eq!(doc1.root, doc2.root, "Parse-canonicalize-parse changed data");
    }

    /// Invariant: Formatting preserves document version
    #[test]
    fn invariant_format_preserves_version(doc in simple_hedl_document()) {
        let file = create_temp_file(&doc, ".hedl");
        let output = create_temp_file("", ".hedl");

        let original = parse(doc.as_bytes()).unwrap();

        format(
            file.path().to_str().unwrap(),
            Some(output.path().to_str().unwrap()),
            false,
            true,
            false,
        ).unwrap();

        let formatted_content = fs::read_to_string(output.path()).unwrap();
        let formatted = parse(formatted_content.as_bytes()).unwrap();

        prop_assert_eq!(
            original.version,
            formatted.version,
            "Formatting changed document version"
        );
    }

    /// Invariant: Number of root items is preserved by formatting
    #[test]
    fn invariant_format_preserves_item_count(doc in multi_field_document()) {
        let file = create_temp_file(&doc, ".hedl");
        let output = create_temp_file("", ".hedl");

        let original = parse(doc.as_bytes()).unwrap();
        let original_count = original.root.len();

        format(
            file.path().to_str().unwrap(),
            Some(output.path().to_str().unwrap()),
            false,
            true,
            false,
        ).unwrap();

        let formatted_content = fs::read_to_string(output.path()).unwrap();
        let formatted = parse(formatted_content.as_bytes()).unwrap();
        let formatted_count = formatted.root.len();

        prop_assert_eq!(
            original_count,
            formatted_count,
            "Formatting changed number of root items"
        );
    }
}

// ===== Property Tests: Performance Characteristics =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Property: Format performance scales reasonably with document size
    #[test]
    fn prop_format_performance_scales(field_count in 1usize..100) {
        let mut doc = String::from("%VERSION: 1.0\n---\n");
        for i in 0..field_count {
            doc.push_str(&format!("field{}: {}\n", i, i));
        }

        let file = create_temp_file(&doc, ".hedl");
        let output = create_temp_file("", ".hedl");

        let start = std::time::Instant::now();
        let result = format(
            file.path().to_str().unwrap(),
            Some(output.path().to_str().unwrap()),
            false,
            true,
            false,
        );
        let duration = start.elapsed();

        prop_assert!(result.is_ok(), "Format failed for {} fields", field_count);

        // Performance should be reasonable (< 1 second for even large documents)
        prop_assert!(
            duration.as_secs() < 1,
            "Format took too long: {:?} for {} fields",
            duration,
            field_count
        );
    }
}
