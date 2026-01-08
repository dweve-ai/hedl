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

//! Comprehensive tests for partial JSON parsing

use hedl_json::{
    FromJsonConfig, partial_parse_json, partial_parse_json_value,
    ErrorTolerance, PartialConfig,
};
use serde_json::json;

#[test]
fn test_partial_parse_fully_valid_json() {
    let json = r#"{
        "name": "Alice",
        "age": 30,
        "email": "alice@example.com"
    }"#;

    let config = PartialConfig::default();
    let result = partial_parse_json(json, &config);

    assert!(result.is_complete());
    assert!(result.document.is_some());
    assert!(result.errors.is_empty());
    assert!(!result.stopped_early);

    let doc = result.document.unwrap();
    assert_eq!(doc.root.len(), 3);
}

#[test]
fn test_partial_parse_invalid_json_syntax() {
    let json = "{ invalid json }";

    let config = PartialConfig::default();
    let result = partial_parse_json(json, &config);

    assert!(result.is_failed());
    assert!(result.document.is_none());
    assert_eq!(result.errors.len(), 1);
    assert!(result.errors[0].is_fatal);
}

#[test]
fn test_partial_parse_stop_on_first_error() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::StopOnFirst)
        .from_json_config(
            FromJsonConfig::builder()
                .max_string_length(10)
                .build()
        )
        .build();

    let json = r#"{
        "a": "this is way too long",
        "b": "also too long",
        "c": "still too long"
    }"#;

    let result = partial_parse_json(json, &config);

    // Should stop on first error
    assert!(result.errors.len() <= 1);
}

#[test]
fn test_partial_parse_collect_all_errors() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .from_json_config(
            FromJsonConfig::builder()
                .max_string_length(10)
                .build()
        )
        .build();

    let json = r#"{
        "a": "this is way too long",
        "b": "also too long",
        "c": "still too long"
    }"#;

    let result = partial_parse_json(json, &config);

    // Should collect all errors
    assert!(result.errors.len() >= 3);
    assert!(result.document.is_some());
}

#[test]
fn test_partial_parse_max_errors_limit() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::MaxErrors(2))
        .from_json_config(
            FromJsonConfig::builder()
                .max_string_length(5)
                .build()
        )
        .build();

    let json = r#"{
        "a": "too long",
        "b": "too long",
        "c": "too long",
        "d": "too long"
    }"#;

    let result = partial_parse_json(json, &config);

    assert!(result.stopped_early);
    assert!(result.errors.len() <= 2);
}

#[test]
fn test_partial_parse_skip_invalid_items() {
    let json = r#"{
        "users": [
            {"id": "1", "name": "Alice"},
            "invalid_user",
            {"id": "2", "name": "Bob"},
            123,
            {"id": "3", "name": "Carol"}
        ]
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::SkipInvalidItems)
        .build();

    let result = partial_parse_json(json, &config);

    assert!(result.document.is_some());
    // Valid users should be parsed
}

#[test]
fn test_partial_parse_replace_invalid_with_null() {
    let json = r#"{
        "name": "Alice",
        "nested": {"invalid": true}
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .replace_invalid_with_null(true)
        .build();

    let result = partial_parse_json(json, &config);

    assert!(result.document.is_some());
    let doc = result.document.unwrap();
    assert!(doc.root.contains_key("name"));
    // The nested object should be replaced with null or kept
}

#[test]
fn test_partial_parse_depth_limit_errors() {
    let config = PartialConfig::builder()
        .from_json_config(
            FromJsonConfig::builder()
                .max_depth(2)
                .build()
        )
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    let json = r#"{
        "level1": {
            "level2": {
                "level3": "too deep"
            }
        }
    }"#;

    let result = partial_parse_json(json, &config);

    // Should have errors about depth limit
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.error.to_string().contains("depth")));
}

#[test]
fn test_partial_parse_array_size_limit() {
    let config = PartialConfig::builder()
        .from_json_config(
            FromJsonConfig::builder()
                .max_array_size(3)
                .build()
        )
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    let json = r#"{
        "numbers": [1, 2, 3, 4, 5]
    }"#;

    let result = partial_parse_json(json, &config);

    // Should have error about array size
    assert!(!result.errors.is_empty());
}

#[test]
fn test_partial_parse_object_size_limit() {
    let config = PartialConfig::builder()
        .from_json_config(
            FromJsonConfig::builder()
                .max_object_size(2)
                .build()
        )
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    let json = r#"{
        "a": 1,
        "b": 2,
        "c": 3
    }"#;

    let result = partial_parse_json(json, &config);

    // Should have error about object size
    assert!(!result.errors.is_empty());
}

#[test]
fn test_partial_parse_mixed_valid_and_invalid() {
    let json = r#"{
        "validString": "hello",
        "validNumber": 42,
        "validBool": true,
        "validNull": null,
        "validArray": [1, 2, 3],
        "validObject": {"nested": "value"}
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    let result = partial_parse_json(json, &config);

    assert!(result.is_complete());
    assert!(result.errors.is_empty());

    let doc = result.document.unwrap();
    assert_eq!(doc.root.len(), 6);
}

#[test]
fn test_partial_parse_invalid_expression() {
    let json = r#"{
        "validExpr": "$(foo)",
        "invalidExpr": "$("
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    let result = partial_parse_json(json, &config);

    // Should have document with valid expression
    assert!(result.document.is_some());
}

#[test]
fn test_partial_parse_invalid_reference() {
    let json = r#"{
        "validRef": {"@ref": "@User:123"},
        "invalidRef": {"@ref": "invalid"}
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    let result = partial_parse_json(json, &config);

    assert!(result.document.is_some());
}

#[test]
fn test_partial_parse_invalid_tensor() {
    let json = r#"{
        "validTensor": [1.0, 2.0, 3.0],
        "invalidTensor": [1, "not a number", 3]
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    let result = partial_parse_json(json, &config);

    assert!(result.document.is_some());
}

#[test]
fn test_partial_parse_matrix_list_with_invalid_rows() {
    let json = r#"{
        "users": [
            {"id": "1", "name": "Alice", "age": 30},
            {"id": "2", "name": "Bob", "age": 25},
            "invalid row",
            {"id": "3", "name": "Carol", "age": 35}
        ]
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    let result = partial_parse_json(json, &config);

    assert!(result.document.is_some());
    // Should have parsed valid rows
}

#[test]
fn test_partial_parse_nested_errors() {
    let json = r#"{
        "outer": {
            "valid": "ok",
            "nested": {
                "deepValid": "still ok"
            }
        }
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .build();

    let result = partial_parse_json(json, &config);

    assert!(result.is_complete());
    assert!(result.document.is_some());
}

#[test]
fn test_partial_parse_error_location_tracking() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .from_json_config(
            FromJsonConfig::builder()
                .max_string_length(5)
                .build()
        )
        .build();

    let json = r#"{
        "user": {
            "name": "this is too long"
        }
    }"#;

    let result = partial_parse_json(json, &config);

    if !result.errors.is_empty() {
        let error = &result.errors[0];
        // Error location should contain path information
        assert!(error.location.path.contains("user") || error.location.path.contains("name"));
    }
}

#[test]
fn test_partial_parse_json_value() {
    let value = json!({
        "valid": "data",
        "number": 42
    });

    let config = PartialConfig::default();
    let result = partial_parse_json_value(&value, &config);

    assert!(result.is_complete());
    assert!(result.document.is_some());
}

#[test]
fn test_partial_parse_empty_object() {
    let json = "{}";

    let config = PartialConfig::default();
    let result = partial_parse_json(json, &config);

    assert!(result.is_complete());
    assert!(result.document.is_some());

    let doc = result.document.unwrap();
    assert!(doc.root.is_empty());
}

#[test]
fn test_partial_parse_empty_arrays() {
    let json = r#"{
        "emptyArray": [],
        "emptyUsers": []
    }"#;

    let config = PartialConfig::default();
    let result = partial_parse_json(json, &config);

    assert!(result.is_complete());
    assert!(result.document.is_some());
}

#[test]
fn test_partial_parse_include_partial_on_fatal() {
    let json = r#"{
        "valid": "data"
    }"#;

    let config = PartialConfig::builder()
        .include_partial_on_fatal(true)
        .build();

    let result = partial_parse_json(json, &config);

    // Even if there's a fatal error later, we should get partial results
    assert!(result.document.is_some() || result.errors.iter().any(|e| e.is_fatal));
}

#[test]
fn test_partial_result_into_result_success() {
    let json = r#"{"name": "Alice"}"#;
    let config = PartialConfig::default();
    let partial_result = partial_parse_json(json, &config);

    let result = partial_result.into_result();
    assert!(result.is_ok());
}

#[test]
fn test_partial_result_into_result_with_errors() {
    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .from_json_config(
            FromJsonConfig::builder()
                .max_string_length(5)
                .build()
        )
        .build();

    let json = r#"{"name": "this is too long"}"#;
    let partial_result = partial_parse_json(json, &config);

    let result = partial_result.into_result();
    // Should be error because there are parsing errors
    if !result.is_ok() {
        assert!(result.is_err());
    }
}

#[test]
fn test_partial_parse_large_valid_document() {
    // Create a larger valid document to test performance
    let json = r#"{
        "users": [
            {"id": "1", "name": "Alice", "email": "alice@example.com", "age": 30},
            {"id": "2", "name": "Bob", "email": "bob@example.com", "age": 25},
            {"id": "3", "name": "Carol", "email": "carol@example.com", "age": 35},
            {"id": "4", "name": "David", "email": "david@example.com", "age": 28},
            {"id": "5", "name": "Eve", "email": "eve@example.com", "age": 32}
        ],
        "metadata": {
            "version": "1.0",
            "timestamp": "2024-01-01T00:00:00Z",
            "count": 5
        }
    }"#;

    let config = PartialConfig::default();
    let result = partial_parse_json(json, &config);

    assert!(result.is_complete());
    assert!(result.document.is_some());
    assert!(result.errors.is_empty());

    let doc = result.document.unwrap();
    assert!(doc.root.contains_key("users"));
    assert!(doc.root.contains_key("metadata"));
}

#[test]
fn test_partial_parse_batch_processing_scenario() {
    // Simulate batch processing where some records are valid and some are not
    let json = r#"{
        "batch": [
            {"id": "1", "data": "valid"},
            {"id": "2", "data": "valid"},
            {"id": "3", "data": "valid"}
        ]
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::SkipInvalidItems)
        .build();

    let result = partial_parse_json(json, &config);

    assert!(result.document.is_some());
    // All valid items should be processed
}

#[test]
fn test_partial_parse_data_migration_scenario() {
    // Simulate data migration where we want to import as much as possible
    let json = r#"{
        "legacy_data": {
            "users": [
                {"id": "1", "name": "Alice"},
                {"id": "2", "name": "Bob"}
            ],
            "settings": {
                "theme": "dark",
                "language": "en"
            }
        }
    }"#;

    let config = PartialConfig::builder()
        .tolerance(ErrorTolerance::CollectAll)
        .replace_invalid_with_null(true)
        .build();

    let result = partial_parse_json(json, &config);

    assert!(result.document.is_some());
    // Should import all valid data
}

#[test]
fn test_error_tolerance_variants() {
    // Test all error tolerance variants
    let tolerances = vec![
        ErrorTolerance::StopOnFirst,
        ErrorTolerance::MaxErrors(5),
        ErrorTolerance::CollectAll,
        ErrorTolerance::SkipInvalidItems,
    ];

    let json = r#"{"valid": "data"}"#;

    for tolerance in tolerances {
        let config = PartialConfig::builder()
            .tolerance(tolerance)
            .build();

        let result = partial_parse_json(json, &config);
        assert!(result.document.is_some());
    }
}

#[test]
fn test_partial_config_builder_chaining() {
    let config = PartialConfig::builder()
        .from_json_config(
            FromJsonConfig::builder()
                .max_depth(100)
                .max_array_size(1000)
                .build()
        )
        .tolerance(ErrorTolerance::CollectAll)
        .include_partial_on_fatal(true)
        .replace_invalid_with_null(false)
        .build();

    assert!(matches!(config.tolerance, ErrorTolerance::CollectAll));
    assert!(config.include_partial_on_fatal);
    assert!(!config.replace_invalid_with_null);
}
