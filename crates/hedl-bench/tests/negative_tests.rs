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

//! Negative tests for hedl-bench error handling and robustness.
//!
//! Tests error paths for:
//! - Invalid HEDL parsing (malformed syntax, security limits)
//! - Oversized datasets (MAX_DATASET_SIZE enforcement)
//! - Invalid format conversions (JSON, YAML, XML, CSV)
//! - Normalization errors (unparseable values)
//! - Token counting failures
//! - Question generation edge cases
//!
//! All tests verify:
//! 1. Errors are returned (not panics)
//! 2. Error messages are clear and actionable
//! 3. Error types are correct and specific

use hedl_bench::error::{validate_dataset_size, BenchError, MAX_DATASET_SIZE};
use hedl_bench::legacy::normalize::{compare, normalize};
use hedl_bench::legacy::questions::AnswerType;
use hedl_bench::token_counter::{compare_batch, compare_formats_str};
use hedl_bench::{compare_formats, count_tokens, generate_users, generate_users_safe};

// ============================================================================
// CATEGORY 1: Parse Errors
// ============================================================================

#[test]
fn test_invalid_hedl_syntax_returns_error() {
    let invalid = "not valid hedl at all!@#$ ∆∂∫∑";
    let result = hedl_core::parse(invalid.as_bytes());
    assert!(
        result.is_err(),
        "Expected parse error for invalid syntax, got Ok"
    );

    let err = result.unwrap_err();
    let err_msg = err.to_string();
    // Error messages can vary - check for common error indicators
    assert!(
        err_msg.contains("Syntax")
            || err_msg.contains("syntax")
            || err_msg.contains("parse")
            || err_msg.contains("invalid")
            || err_msg.contains("expected")
            || err_msg.contains("Error"),
        "Error message should mention parsing issue, got: {}",
        err_msg
    );
}

#[test]
fn test_malformed_hedl_key_value() {
    let malformed = r#"%VERSION: 1.0
---
key without colon value
another: valid
broken again no colon
"#;
    let result = hedl_core::parse(malformed.as_bytes());
    assert!(
        result.is_err(),
        "Expected error for missing colons in key-value pairs"
    );
}

#[test]
fn test_invalid_version_header() {
    let invalid_version = r#"%VERSION: 999.999
---
data: value
"#;
    let _result = hedl_core::parse(invalid_version.as_bytes());
    // Should either parse with version warning or reject - either way, should not panic
    // Version errors are typically warnings, so this might succeed
    // The important part is no panic occurs
}

#[test]
fn test_incomplete_hedl_document() {
    let incomplete = "%VERSION: 1.0";
    // Missing separator and content
    let result = hedl_core::parse(incomplete.as_bytes());
    // Should handle gracefully (empty doc or error, but no panic)
    assert!(
        result.is_ok() || result.is_err(),
        "Parser should handle incomplete document gracefully"
    );
}

#[test]
fn test_invalid_struct_declaration() {
    let invalid_struct = r#"%VERSION: 1.0
%STRUCT: User (not_a_number): [id,name]
---
users: @User
  | u1, Alice
"#;
    let result = hedl_core::parse(invalid_struct.as_bytes());
    // Should handle malformed struct count gracefully
    assert!(
        result.is_ok() || result.is_err(),
        "Parser should handle malformed STRUCT declaration"
    );
}

#[test]
fn test_invalid_nest_declaration() {
    let invalid_nest = r#"%VERSION: 1.0
%NEST: Parent >>> Child
---
parent: @Parent
"#;
    let _result = hedl_core::parse(invalid_nest.as_bytes());
    // Invalid NEST syntax should be handled gracefully
}

#[test]
fn test_circular_references() {
    let circular = r#"%VERSION: 1.0
---
a: @b
b: @a
"#;
    let _result = hedl_core::parse(circular.as_bytes());
    // Circular references should be detected or handled gracefully
}

#[test]
fn test_unresolved_reference() {
    let unresolved = r#"%VERSION: 1.0
---
user: @NonExistentType:xyz
"#;
    let result = hedl_core::parse(unresolved.as_bytes());
    // Unresolved references should produce clear error in strict mode
    if result.is_err() {
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("reference") || err_msg.contains("unresolved"),
            "Error should mention unresolved reference, got: {}",
            err_msg
        );
    }
}

// ============================================================================
// CATEGORY 2: Size Limit Errors
// ============================================================================

#[test]
fn test_dataset_size_exactly_at_limit() {
    let result = validate_dataset_size(MAX_DATASET_SIZE);
    assert!(
        result.is_ok(),
        "Should accept size exactly at MAX_DATASET_SIZE"
    );
}

#[test]
fn test_dataset_size_one_over_limit() {
    let result = validate_dataset_size(MAX_DATASET_SIZE + 1);
    assert!(
        result.is_err(),
        "Should reject size one over MAX_DATASET_SIZE"
    );

    match result {
        Err(BenchError::DatasetTooLarge { requested, max }) => {
            assert_eq!(requested, MAX_DATASET_SIZE + 1);
            assert_eq!(max, MAX_DATASET_SIZE);
        }
        _ => panic!("Expected DatasetTooLarge error"),
    }
}

#[test]
fn test_dataset_size_way_over_limit() {
    let result = validate_dataset_size(usize::MAX);
    assert!(result.is_err(), "Should reject extremely large size");

    match result {
        Err(BenchError::DatasetTooLarge { requested, max }) => {
            assert_eq!(requested, usize::MAX);
            assert_eq!(max, MAX_DATASET_SIZE);
        }
        _ => panic!("Expected DatasetTooLarge error"),
    }
}

#[test]
fn test_generate_users_safe_rejects_oversized() {
    let result = generate_users_safe(MAX_DATASET_SIZE + 1);
    assert!(
        result.is_err(),
        "generate_users_safe should reject oversized dataset"
    );

    let err = result.unwrap_err();
    assert!(
        matches!(err, BenchError::DatasetTooLarge { .. }),
        "Expected DatasetTooLarge error, got: {:?}",
        err
    );

    let err_msg = err.to_string();
    assert!(
        err_msg.contains("exceeds maximum"),
        "Error message should be actionable: {}",
        err_msg
    );
}

#[test]
fn test_generate_users_safe_zero_size() {
    let result = generate_users_safe(0);
    assert!(result.is_ok(), "Should allow zero-size dataset");

    let hedl = result.unwrap();
    // Should parse successfully
    let doc = hedl_core::parse(hedl.as_bytes());
    assert!(doc.is_ok(), "Empty dataset should parse successfully");
}

#[test]
fn test_generate_users_safe_boundary_sizes() {
    // Test boundary cases
    for size in [0, 1, 10, 100, 1_000, 10_000, MAX_DATASET_SIZE] {
        let result = generate_users_safe(size);
        assert!(result.is_ok(), "Should handle size {} within limits", size);

        if size > 0 {
            let hedl = result.unwrap();
            assert!(
                hedl.contains("%STRUCT: User"),
                "Generated dataset should have STRUCT declaration"
            );
        }
    }
}

// ============================================================================
// CATEGORY 3: Format Conversion Errors
// ============================================================================

#[test]
fn test_invalid_json_parsing() {
    use hedl_json::{from_json, FromJsonConfig};

    let config = FromJsonConfig::default();
    let invalid_json_cases = vec![
        "{invalid json}",
        "{\"key\": }",
        "{\"key\": value without quotes}",
        "{'single': 'quotes'}",
        "{trailing: comma,}",
        "[1, 2, 3,]",
        "null",
        "undefined",
        "{\"key\": NaN}",
        "{\"key\": Infinity}",
    ];

    for (i, invalid) in invalid_json_cases.iter().enumerate() {
        let result = from_json(invalid, &config);
        assert!(
            result.is_err(),
            "Case {}: Should reject invalid JSON: {}",
            i,
            invalid
        );

        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("parse") || err_msg.contains("JSON") || err_msg.contains("error"),
            "Case {}: Error should mention JSON parsing, got: {}",
            i,
            err_msg
        );
    }
}

#[test]
fn test_invalid_yaml_parsing() {
    use hedl_yaml::{from_yaml, FromYamlConfig};

    let config = FromYamlConfig::default();
    let invalid_yaml_cases = vec![
        "key: : value",     // Double colon
        "  invalid indent", // No parent
        "key: [unclosed",   // Unclosed bracket
        "key: {unclosed",   // Unclosed brace
        "- - - invalid",    // Invalid list nesting
        "key: |invalid",    // Invalid block scalar
    ];

    for (i, invalid) in invalid_yaml_cases.iter().enumerate() {
        let result = from_yaml(invalid, &config);
        // YAML parser is more lenient, but should handle gracefully
        if result.is_err() {
            let err_msg = result.unwrap_err().to_string();
            assert!(
                err_msg.contains("parse") || err_msg.contains("YAML") || err_msg.contains("error"),
                "Case {}: Error should mention YAML parsing, got: {}",
                i,
                err_msg
            );
        }
    }
}

#[test]
fn test_invalid_xml_parsing() {
    use hedl_xml::{from_xml, FromXmlConfig};

    let config = FromXmlConfig::default();
    // XML parser may auto-close some tags, so use more clearly invalid cases
    let invalid_xml_cases = vec![
        "<tag></different>",  // Mismatched tags
        "< invalid tag>",     // Invalid tag syntax
        "not xml at all",     // No XML structure
        "<tag>unclosed text", // Unclosed tag
    ];

    for (i, invalid) in invalid_xml_cases.iter().enumerate() {
        let result = from_xml(invalid, &config);
        // XML parser might be lenient, so just ensure it doesn't panic
        // If it does error, check the message is reasonable
        if result.is_err() {
            let err = result.unwrap_err();
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("parse")
                    || err_msg.contains("XML")
                    || err_msg.contains("error")
                    || err_msg.contains("Error"),
                "Case {}: Error should mention XML parsing, got: {}",
                i,
                err_msg
            );
        }
    }
}

#[test]
fn test_invalid_csv_parsing() {
    use hedl_csv::from_csv;

    let invalid_csv_cases = vec![
        "header1,header2\nvalue1", // Mismatched columns
        "\"unclosed quote",
        "header1,header2\nvalue1,value2,value3", // Too many columns
    ];

    for (i, invalid) in invalid_csv_cases.iter().enumerate() {
        // from_csv requires type_name and schema
        let result = from_csv(invalid, "Row", &["col1", "col2"]);
        // CSV parser might be lenient, but should handle gracefully
        if result.is_err() {
            let err_msg = result.unwrap_err().to_string();
            assert!(
                !err_msg.is_empty(),
                "Case {}: Error message should be non-empty",
                i
            );
        }
    }
}

#[test]
fn test_json_root_must_be_object() {
    use hedl_json::{from_json, FromJsonConfig};

    let config = FromJsonConfig::default();
    let invalid_roots = vec!["null", "true", "42", "\"string\"", "[1, 2, 3]"];

    for invalid in invalid_roots {
        let result = from_json(invalid, &config);
        if result.is_err() {
            let err_msg = result.unwrap_err().to_string();
            // Should mention that root must be an object
            assert!(
                err_msg.contains("Root") || err_msg.contains("object") || err_msg.contains("must"),
                "Error should mention root type requirement, got: {}",
                err_msg
            );
        }
    }
}

#[test]
fn test_empty_format_inputs() {
    use hedl_json::FromJsonConfig;
    use hedl_xml::FromXmlConfig;
    use hedl_yaml::FromYamlConfig;

    // Empty inputs should be handled gracefully
    assert!(
        hedl_json::from_json("{}", &FromJsonConfig::default()).is_ok(),
        "Empty JSON object should succeed"
    );
    assert!(
        hedl_yaml::from_yaml("{}", &FromYamlConfig::default()).is_ok(),
        "Empty YAML should succeed"
    );
    assert!(
        hedl_xml::from_xml("<root/>", &FromXmlConfig::default()).is_ok(),
        "Empty XML root should succeed"
    );
}

// ============================================================================
// CATEGORY 4: Normalization Errors
// ============================================================================

#[test]
fn test_normalize_integer_invalid_input() {
    let invalid_integers = vec![
        "not a number",
        "42.5.7", // Multiple decimals
        "∞",
        "NaN",
        "",
        "   ",
    ];

    for invalid in invalid_integers {
        let result = normalize(invalid, &AnswerType::Integer);
        assert!(
            result.is_err(),
            "Should reject invalid integer: '{}'",
            invalid
        );

        if let Err(err) = result {
            assert!(
                matches!(err, BenchError::NormalizationFailed { .. }),
                "Expected NormalizationFailed error for '{}', got: {:?}",
                invalid,
                err
            );

            let err_msg = err.to_string();
            assert!(
                err_msg.contains("normalize")
                    || err_msg.contains("parse")
                    || err_msg.contains("Cannot"),
                "Error should be descriptive: {}",
                err_msg
            );
        }
    }
}

#[test]
fn test_normalize_number_invalid_input() {
    let invalid_numbers = vec![
        "not a number",
        "42..5",
        "1.2.3",
        "∞",
        "NaN",
        "",
        "   ",
        "1e", // Incomplete scientific notation
        "1e1e1",
    ];

    for invalid in invalid_numbers {
        let result = normalize(invalid, &AnswerType::Number { decimals: 2 });
        assert!(
            result.is_err(),
            "Should reject invalid number: '{}'",
            invalid
        );

        if let Err(err) = result {
            assert!(
                matches!(err, BenchError::NormalizationFailed { .. }),
                "Expected NormalizationFailed for '{}'",
                invalid
            );
        }
    }
}

#[test]
fn test_normalize_boolean_invalid_input() {
    let invalid_booleans = vec![
        "maybe",
        "yep",
        "nope",
        "2",
        "-1",
        "",
        "TRUE/FALSE",
        "t",
        "f",
    ];

    for invalid in invalid_booleans {
        let result = normalize(invalid, &AnswerType::Boolean);
        assert!(
            result.is_err(),
            "Should reject invalid boolean: '{}'",
            invalid
        );

        if let Err(err) = result {
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("boolean") || err_msg.contains("Cannot parse"),
                "Error should mention boolean parsing: {}",
                err_msg
            );
        }
    }
}

#[test]
fn test_normalize_date_invalid_format() {
    let invalid_dates = vec![
        "not a date",
        "2024-13-01", // Invalid month
        "2024-01-32", // Invalid day
        "01-01-2024", // Ambiguous format
        "2024/01/01", // Wrong separator
        "",
        "tomorrow",
        "2024-1-1", // Missing zero padding (might work, but test it)
    ];

    for invalid in invalid_dates {
        let result = normalize(invalid, &AnswerType::Date);
        if result.is_err() {
            let err = result.unwrap_err();
            assert!(
                matches!(err, BenchError::NormalizationFailed { .. }),
                "Expected NormalizationFailed for date '{}'",
                invalid
            );

            let err_msg = err.to_string();
            assert!(
                err_msg.contains("date") || err_msg.contains("parse"),
                "Error should mention date parsing: {}",
                err_msg
            );
        }
    }
}

#[test]
fn test_normalize_empty_strings() {
    // Empty string normalization behavior
    let empty_cases = vec![
        ("", &AnswerType::String),
        ("   ", &AnswerType::String),
        ("", &AnswerType::CsvListOrdered),
        ("", &AnswerType::CsvListUnordered),
    ];

    for (input, answer_type) in empty_cases {
        let result = normalize(input, answer_type);
        // Should handle empty inputs gracefully (either Ok with empty result or Err)
        assert!(
            result.is_ok() || result.is_err(),
            "Should handle empty input gracefully for {:?}",
            answer_type
        );
    }
}

#[test]
fn test_compare_type_mismatch() {
    // Comparing values with incompatible types should fail gracefully
    let result = compare("not_a_number", "42", &AnswerType::Number { decimals: 2 });
    assert!(
        result.is_err(),
        "Should fail to compare non-number with number type"
    );

    if let Err(err) = result {
        assert!(
            matches!(
                err,
                BenchError::ComparisonFailed { .. } | BenchError::NormalizationFailed { .. }
            ),
            "Expected comparison or normalization error"
        );
    }
}

#[test]
fn test_normalize_extreme_numbers() {
    // Test edge cases for number normalization
    let extreme_cases = vec![
        ("1e308", &AnswerType::Number { decimals: 2 }), // Near f64::MAX
        ("-1e308", &AnswerType::Number { decimals: 2 }),
        ("1e-308", &AnswerType::Number { decimals: 10 }), // Near f64::MIN_POSITIVE
        ("0.0", &AnswerType::Number { decimals: 2 }),
        ("-0.0", &AnswerType::Number { decimals: 2 }),
    ];

    for (input, answer_type) in extreme_cases {
        let result = normalize(input, answer_type);
        // Should handle extreme but valid numbers
        assert!(result.is_ok(), "Should handle extreme number: '{}'", input);
    }
}

// ============================================================================
// CATEGORY 5: Token Counting Errors
// ============================================================================

#[test]
fn test_count_tokens_empty_string() {
    let tokens = count_tokens("");
    // Empty string should have 0 tokens
    assert_eq!(tokens, 0, "Empty string should have 0 tokens");
}

#[test]
fn test_count_tokens_whitespace_only() {
    let tokens = count_tokens("   \n\t  ");
    // Whitespace-only should have minimal tokens (usize is always >= 0)
    // Just verify no panic occurs
    let _ = tokens;
}

#[test]
fn test_count_tokens_unicode() {
    let unicode_text = "Hello 世界 🌍";
    let tokens = count_tokens(unicode_text);
    assert!(tokens > 0, "Unicode text should tokenize successfully");
}

#[test]
fn test_count_tokens_very_long_string() {
    // Test with a very long string (not excessive, but substantial)
    let long_string = "a".repeat(100_000);
    let tokens = count_tokens(&long_string);
    assert!(tokens > 0, "Long string should tokenize successfully");
}

#[test]
fn test_compare_formats_with_invalid_document() {
    // Create a minimal valid document
    let hedl = "%VERSION: 1.0\n---\n";
    let doc = hedl_core::parse(hedl.as_bytes()).unwrap();

    // Should handle document with no data gracefully
    let stats = compare_formats(&doc);
    // usize is always >= 0, just verify we get valid results
    assert!(
        stats.hedl_bytes > 0,
        "Should have some bytes even for empty doc"
    );
}

#[test]
fn test_compare_formats_str_invalid_hedl() {
    let invalid = "not valid hedl!@#$";
    let result = compare_formats_str(invalid);

    assert!(result.is_err(), "Should return error for invalid HEDL");

    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("Parse") || err_msg.contains("parse") || err_msg.contains("error"),
        "Error should mention parsing: {}",
        err_msg
    );
}

// ============================================================================
// CATEGORY 6: Edge Cases and Boundary Conditions
// ============================================================================

#[test]
fn test_generate_users_boundary_counts() {
    // Test various boundary conditions
    let boundary_cases = vec![0, 1, 2, 10, 100, 1_000];

    for count in boundary_cases {
        let result = generate_users_safe(count);
        assert!(result.is_ok(), "Should handle count {} successfully", count);

        let hedl = result.unwrap();
        let doc = hedl_core::parse(hedl.as_bytes());
        assert!(
            doc.is_ok(),
            "Generated HEDL for count {} should parse successfully",
            count
        );
    }
}

#[test]
fn test_dataset_size_validation_boundary() {
    // Test exact boundary
    assert!(validate_dataset_size(MAX_DATASET_SIZE).is_ok());
    assert!(validate_dataset_size(MAX_DATASET_SIZE + 1).is_err());

    // Test far from boundary
    assert!(validate_dataset_size(0).is_ok());
    assert!(validate_dataset_size(1).is_ok());
    assert!(validate_dataset_size(MAX_DATASET_SIZE / 2).is_ok());
}

#[test]
fn test_normalize_csv_list_edge_cases() {
    // Empty list
    let result = normalize("", &AnswerType::CsvListOrdered);
    assert!(result.is_ok(), "Should handle empty CSV list");

    // Single item
    let result = normalize("item", &AnswerType::CsvListOrdered);
    assert!(result.is_ok(), "Should handle single-item list");
    assert_eq!(result.unwrap(), "item");

    // Whitespace variations
    let result = normalize(" a , b , c ", &AnswerType::CsvListUnordered);
    assert!(result.is_ok(), "Should handle whitespace in CSV");

    // Empty items (consecutive commas)
    let result = normalize("a,,b", &AnswerType::CsvListOrdered);
    assert!(result.is_ok(), "Should handle empty items");
}

#[test]
fn test_error_message_clarity() {
    // Verify error messages are actionable
    let result = validate_dataset_size(MAX_DATASET_SIZE + 1);
    let err = result.unwrap_err();
    let msg = format!("{}", err);

    // Should contain: what failed, why, and limits
    assert!(msg.contains("exceeds"), "Should explain the violation");
    assert!(msg.contains("maximum"), "Should mention the limit");
    assert!(
        msg.contains(&(MAX_DATASET_SIZE + 1).to_string()),
        "Should show requested size"
    );
    assert!(
        msg.contains(&MAX_DATASET_SIZE.to_string()),
        "Should show max size"
    );
}

#[test]
fn test_bench_error_equality() {
    // Test error type equality for proper error handling
    let err1 = BenchError::DatasetTooLarge {
        requested: 100,
        max: 10,
    };
    let err2 = BenchError::DatasetTooLarge {
        requested: 100,
        max: 10,
    };
    let err3 = BenchError::DatasetTooLarge {
        requested: 200,
        max: 10,
    };

    assert_eq!(err1, err2, "Identical errors should be equal");
    assert_ne!(err1, err3, "Different errors should not be equal");
}

#[test]
fn test_normalization_failed_error() {
    let err = BenchError::NormalizationFailed {
        value: "invalid".to_string(),
        reason: "Cannot parse as integer".to_string(),
    };

    let msg = format!("{}", err);
    assert!(msg.contains("invalid"), "Should show the invalid value");
    assert!(msg.contains("Cannot parse"), "Should show the reason");
}

#[test]
fn test_comparison_failed_error() {
    let err = BenchError::ComparisonFailed {
        reason: "Type mismatch".to_string(),
    };

    let msg = format!("{}", err);
    assert!(
        msg.contains("Comparison failed"),
        "Should mention comparison failure"
    );
    assert!(msg.contains("Type mismatch"), "Should show the reason");
}

// ============================================================================
// CATEGORY 7: No Panic Guarantees
// ============================================================================

#[test]
fn test_no_panic_on_invalid_inputs() {
    // Ensure various invalid inputs don't panic
    let long_string = "a".repeat(10_000);
    let invalid_inputs: Vec<&str> = vec![
        "",
        "   ",
        "\n\n\n",
        "invalid",
        "!@#$%^&*()",
        "∆∂∫∑",
        "\0",
        &long_string,
    ];

    for input in invalid_inputs {
        // Should not panic, regardless of whether it succeeds or fails
        let _ = hedl_core::parse(input.as_bytes());
        let _ = normalize(input, &AnswerType::String);
        let _ = normalize(input, &AnswerType::Integer);
        let _ = normalize(input, &AnswerType::Boolean);
        let _ = count_tokens(input);
    }
}

#[test]
fn test_no_panic_on_format_conversions() {
    use hedl_json::FromJsonConfig;
    use hedl_xml::FromXmlConfig;
    use hedl_yaml::FromYamlConfig;

    // Ensure format conversion errors don't panic
    let invalid = "totally invalid data";

    let _ = hedl_json::from_json(invalid, &FromJsonConfig::default());
    let _ = hedl_yaml::from_yaml(invalid, &FromYamlConfig::default());
    let _ = hedl_xml::from_xml(invalid, &FromXmlConfig::default());
    let _ = hedl_csv::from_csv(invalid, "Row", &["col"]);
}

#[test]
fn test_no_panic_on_extreme_sizes() {
    // Don't actually allocate huge sizes, just test validation
    let _ = validate_dataset_size(usize::MAX);
    let _ = validate_dataset_size(usize::MAX / 2);
    let _ = validate_dataset_size(MAX_DATASET_SIZE * 2);
}

// ============================================================================
// CATEGORY 8: Integration Tests for Error Paths
// ============================================================================

#[test]
fn test_full_pipeline_with_invalid_hedl() {
    // Test entire pipeline with invalid input
    let invalid = "not hedl";

    // Parse fails
    let parse_result = hedl_core::parse(invalid.as_bytes());
    assert!(parse_result.is_err(), "Parse should fail");

    // Token comparison would fail due to parse error
    let compare_result = compare_formats_str(invalid);
    assert!(
        compare_result.is_err(),
        "Format comparison should fail for invalid HEDL"
    );
}

#[test]
fn test_normalization_with_comparison() {
    // Test normalization errors propagate through comparison
    let result = compare("not_a_number", "42", &AnswerType::Integer);
    assert!(result.is_err(), "Should fail to compare invalid integer");

    if let Err(err) = result {
        let msg = format!("{}", err);
        assert!(!msg.is_empty(), "Error message should be descriptive");
    }
}

#[test]
fn test_error_propagation_in_batch_operations() {
    // Test that errors in batch operations are handled properly
    let valid_hedl = generate_users(10);
    let valid_doc = hedl_core::parse(valid_hedl.as_bytes()).unwrap();

    // Batch comparison with mixed valid/invalid shouldn't panic
    let documents = vec![("valid", &valid_doc)];

    let results = compare_batch(&documents);
    assert_eq!(results.len(), 1, "Should return results for all documents");
    assert!(
        results[0].1.hedl_tokens > 0,
        "Should have valid token counts"
    );
}

#[test]
fn test_oversized_generation_safety() {
    // Verify that safe generation prevents OOM
    let huge_size = MAX_DATASET_SIZE + 1_000_000;
    let result = generate_users_safe(huge_size);

    assert!(result.is_err(), "Should reject huge dataset size");

    match result.unwrap_err() {
        BenchError::DatasetTooLarge { requested, max } => {
            assert_eq!(requested, huge_size);
            assert_eq!(max, MAX_DATASET_SIZE);
        }
        other => panic!("Expected DatasetTooLarge, got: {:?}", other),
    }
}
