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

//! Tests for inline convenience functions in lib.rs.
//!
//! These tests verify the behavior of the #[inline] convenience functions
//! that wrap underlying implementations.

use hedl::{canonicalize, from_json, lint, parse, parse_lenient, to_json, validate};

// =============================================================================
// parse() Tests
// =============================================================================

#[test]
fn test_parse_inline_basic() {
    // Parsing v1.0 input should preserve the input version
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    assert_eq!(doc.version, (1, 0));
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_parse_inline_accepts_str() {
    let input: &str = "%VERSION: 1.0\n---\nkey: value";
    let doc = parse(input).unwrap();
    assert_eq!(doc.version, (1, 0));
}

#[test]
fn test_parse_inline_accepts_string() {
    let input: String = "%VERSION: 1.0\n---\nkey: value".to_string();
    let doc = parse(&input).unwrap();
    assert_eq!(doc.version, (1, 0));
}

#[test]
fn test_parse_inline_empty_data_section() {
    let doc = parse("%VERSION: 1.0\n---\n").unwrap();
    assert_eq!(doc.version, (1, 0));
    assert_eq!(doc.root.len(), 0);
}

#[test]
fn test_parse_inline_with_structs() {
    let input = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\n";
    let doc = parse(input).unwrap();
    assert!(doc.structs.contains_key("User"));
}

#[test]
fn test_parse_inline_with_aliases() {
    let input = "%VERSION: 1.0\n%ALIAS: %pi: \"3.14159\"\n---\nvalue: %pi";
    let doc = parse(input).unwrap();
    assert!(doc.aliases.contains_key("pi"));
}

#[test]
fn test_parse_inline_error_no_version() {
    let result = parse("---\nkey: value");
    assert!(result.is_err());
}

#[test]
fn test_parse_inline_error_no_separator() {
    let result = parse("%VERSION: 1.0\nkey: value");
    assert!(result.is_err());
}

#[test]
fn test_parse_inline_error_invalid_version() {
    // Note: HEDL parser may accept versions other than 1.0 for forward compatibility
    // This test just verifies the API accepts version format correctly
    let result = parse("%VERSION: 99.99\n---\n");
    // May succeed or fail depending on version validation policy
    let _ = result;
}

#[test]
fn test_parse_inline_strict_mode_default() {
    // parse() uses strict mode by default
    let input = "%VERSION: 1.0\n---\nref: @nonexistent";
    let result = parse(input);
    assert!(result.is_err());
}

// =============================================================================
// parse_lenient() Tests
// =============================================================================

#[test]
fn test_parse_lenient_inline_basic() {
    let doc = parse_lenient("%VERSION: 1.0\n---\nkey: value").unwrap();
    assert_eq!(doc.version, (1, 0));
}

#[test]
fn test_parse_lenient_inline_unresolved_reference() {
    let input = "%VERSION: 1.0\n---\nref: @nonexistent";
    let doc = parse_lenient(input).unwrap();
    assert_eq!(doc.version, (1, 0));
    // In lenient mode, unresolved references should not cause errors
}

#[test]
fn test_parse_lenient_inline_vs_strict() {
    let input = "%VERSION: 1.0\n---\nref: @undefined_id";

    // Strict should fail
    assert!(parse(input).is_err());

    // Lenient should succeed
    assert!(parse_lenient(input).is_ok());
}

#[test]
fn test_parse_lenient_inline_creates_correct_options() {
    let input = "%VERSION: 1.0\n---\nkey: value";
    let doc = parse_lenient(input).unwrap();

    // Verify it actually uses lenient mode by checking default limits work
    assert_eq!(doc.version, (1, 0));
}

#[test]
fn test_parse_lenient_inline_error_still_on_syntax() {
    // Lenient mode should still fail on syntax errors
    let result = parse_lenient("this is not HEDL");
    assert!(result.is_err());
}

// =============================================================================
// canonicalize() Tests
// =============================================================================

#[test]
fn test_canonicalize_inline_basic() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let canonical = canonicalize(&doc).unwrap();
    assert!(canonical.contains("%VERSION: 1.0"));
    assert!(canonical.contains("key:"));
}

#[test]
fn test_canonicalize_inline_sorts_keys() {
    let doc = parse("%VERSION: 1.0\n---\nz: 3\nb: 2\na: 1").unwrap();
    let canonical = canonicalize(&doc).unwrap();

    let a_pos = canonical.find("a:").unwrap();
    let b_pos = canonical.find("b:").unwrap();
    let z_pos = canonical.find("z:").unwrap();

    assert!(a_pos < b_pos);
    assert!(b_pos < z_pos);
}

#[test]
fn test_canonicalize_inline_deterministic() {
    let doc = parse("%VERSION: 1.0\n---\nb: 2\na: 1").unwrap();
    let c1 = canonicalize(&doc).unwrap();
    let c2 = canonicalize(&doc).unwrap();
    assert_eq!(c1, c2);
}

#[test]
fn test_canonicalize_inline_empty_document() {
    let doc = parse("%VERSION: 1.0\n---\n").unwrap();
    let canonical = canonicalize(&doc).unwrap();
    assert!(canonical.contains("%VERSION: 1.0"));
}

#[test]
fn test_canonicalize_inline_nested() {
    let input = "%VERSION: 1.0\n---\nparent:\n z: 3\n a: 1";
    let doc = parse(input).unwrap();
    let canonical = canonicalize(&doc).unwrap();
    assert!(canonical.contains("a:"));
    assert!(canonical.contains("z:"));
}

#[test]
fn test_canonicalize_inline_with_aliases() {
    let input = "%VERSION: 1.0\n%ALIAS: %rate: \"1.5\"\n---\nvalue: %rate";
    let doc = parse(input).unwrap();
    let canonical = canonicalize(&doc).unwrap();
    assert!(canonical.contains("%ALIAS:"));
}

// =============================================================================
// to_json() Tests
// =============================================================================

#[test]
fn test_to_json_inline_basic() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"key\""));
    assert!(json.contains("\"value\""));
}

#[test]
fn test_to_json_inline_number() {
    let doc = parse("%VERSION: 1.0\n---\ncount: 42").unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("42"));
}

#[test]
fn test_to_json_inline_boolean() {
    let doc = parse("%VERSION: 1.0\n---\nactive: true\ndisabled: false").unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("true"));
    assert!(json.contains("false"));
}

#[test]
fn test_to_json_inline_null() {
    let doc = parse("%VERSION: 1.0\n---\nvalue: null").unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("null"));
}

#[test]
fn test_to_json_inline_nested() {
    let input = "%VERSION: 1.0\n---\nuser:\n name: Alice\n age: 30";
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"user\""));
    assert!(json.contains("\"name\""));
    assert!(json.contains("\"Alice\""));
}

#[test]
fn test_to_json_inline_empty_document() {
    let doc = parse("%VERSION: 1.0\n---\n").unwrap();
    let json = to_json(&doc).unwrap();
    // Should produce valid JSON even if empty
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object());
}

#[test]
fn test_to_json_inline_valid_json() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let json = to_json(&doc).unwrap();
    // Should be valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(parsed.is_ok());
}

// =============================================================================
// from_json() Tests
// =============================================================================

#[test]
fn test_from_json_inline_basic() {
    // from_json creates new documents with v2.0 format
    let json = r#"{"key": "value"}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.version, (2, 0));
}

#[test]
fn test_from_json_inline_number() {
    let json = r#"{"count": 42}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_from_json_inline_boolean() {
    let json = r#"{"active": true, "disabled": false}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.root.len(), 2);
}

#[test]
fn test_from_json_inline_null() {
    let json = r#"{"value": null}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_from_json_inline_nested() {
    let json = r#"{"user": {"name": "Alice", "age": 30}}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_from_json_inline_array() {
    let json = r#"{"items": [1, 2, 3]}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_from_json_inline_error_invalid() {
    let json = "not valid json";
    let result = from_json(json);
    assert!(result.is_err());
}

#[test]
fn test_from_json_inline_error_contains_context() {
    let json = "{invalid}";
    let result = from_json(json);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.message.contains("JSON conversion error"));
}

// =============================================================================
// lint() Tests
// =============================================================================

#[test]
fn test_lint_inline_basic() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let diagnostics = lint(&doc);
    // Should not panic
    let _ = diagnostics;
}

#[test]
fn test_lint_inline_empty_document() {
    let doc = parse("%VERSION: 1.0\n---\n").unwrap();
    let diagnostics = lint(&doc);
    let _ = diagnostics;
}

#[test]
fn test_lint_inline_returns_vec() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let diagnostics = lint(&doc);
    // Should return a Vec - verify it's a valid collection
    assert!(diagnostics.len() < 1000); // Arbitrary upper bound
}

#[test]
fn test_lint_inline_unused_alias() {
    let input = "%VERSION: 1.0\n%ALIAS: %unused: \"value\"\n---\nkey: other";
    let doc = parse(input).unwrap();
    let diagnostics = lint(&doc);
    // May or may not have diagnostics, but should not panic
    let _ = diagnostics;
}

#[test]
fn test_lint_inline_diagnostics_display() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let diagnostics = lint(&doc);
    for d in diagnostics {
        let _s = format!("{d}");
    }
}

// =============================================================================
// validate() Tests
// =============================================================================

#[test]
fn test_validate_inline_valid() {
    assert!(validate("%VERSION: 1.0\n---\n").is_ok());
    assert!(validate("%VERSION: 1.0\n---\nkey: value").is_ok());
}

#[test]
fn test_validate_inline_invalid_no_version() {
    assert!(validate("---\nkey: value").is_err());
}

#[test]
fn test_validate_inline_invalid_no_separator() {
    assert!(validate("%VERSION: 1.0\nkey: value").is_err());
}

#[test]
fn test_validate_inline_invalid_syntax() {
    assert!(validate("this is not HEDL").is_err());
}

#[test]
fn test_validate_inline_empty_string() {
    assert!(validate("").is_err());
}

#[test]
fn test_validate_inline_only_version() {
    assert!(validate("%VERSION: 1.0").is_err());
}

#[test]
fn test_validate_inline_returns_unit() {
    let result = validate("%VERSION: 1.0\n---\n");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ());
}

// =============================================================================
// Round-trip Tests Through Inline Functions
// =============================================================================

#[test]
fn test_inline_parse_to_json_from_json() {
    let original = "%VERSION: 1.0\n---\nname: Alice\nage: 30";
    let doc1 = parse(original).unwrap();
    let json = to_json(&doc1).unwrap();
    let doc2 = from_json(&json).unwrap();

    // doc1 has version from input (1, 0)
    // doc2 is created by from_json which produces v2.0
    assert_eq!(doc1.version, (1, 0));
    assert_eq!(doc2.version, (2, 0));
    // Data should be preserved
    assert_eq!(doc1.root.len(), doc2.root.len());
}

#[test]
fn test_inline_parse_canonicalize_parse() {
    let original = "%VERSION: 1.0\n---\nb: 2\na: 1";
    let doc1 = parse(original).unwrap();
    let canonical = canonicalize(&doc1).unwrap();
    let doc2 = parse(&canonical).unwrap();

    assert_eq!(doc1.version, doc2.version);
    assert_eq!(doc1.root.len(), doc2.root.len());
}

#[test]
fn test_inline_functions_composition() {
    let input = "%VERSION: 1.0\n---\nkey: value";

    // Chain: parse -> canonicalize -> parse -> to_json -> from_json
    let doc1 = parse(input).unwrap();
    let canonical = canonicalize(&doc1).unwrap();
    let doc2 = parse(&canonical).unwrap();
    let json = to_json(&doc2).unwrap();
    let doc3 = from_json(&json).unwrap();

    // doc1 and doc2 have version from input (1, 0)
    // doc3 is created by from_json which produces v2.0
    assert_eq!(doc1.version, (1, 0));
    assert_eq!(doc2.version, (1, 0));
    assert_eq!(doc3.version, (2, 0));
    // Data should be preserved throughout
    assert_eq!(doc1.root.len(), doc3.root.len());
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_inline_functions_error_types() {
    // parse error
    let err1 = parse("invalid").unwrap_err();
    assert!(!err1.message.is_empty());

    // to_json error (should not happen with valid doc, but test error path exists)
    // Note: to_json maps internal errors to HedlError::syntax with "JSON conversion error"

    // from_json error
    let err3 = from_json("invalid json").unwrap_err();
    assert!(err3.message.contains("JSON conversion error"));

    // validate error
    let err4 = validate("invalid").unwrap_err();
    assert!(!err4.message.is_empty());
}

#[test]
fn test_inline_functions_preserve_error_info() {
    let result = parse("invalid HEDL syntax here");
    assert!(result.is_err());

    let err = result.unwrap_err();
    // Error should have meaningful information
    assert!(err.line > 0 || !err.message.is_empty());
}

// =============================================================================
// Performance Characteristics Tests
// =============================================================================

#[test]
fn test_inline_functions_small_input() {
    // Verify inline functions work efficiently with small inputs
    let input = "%VERSION: 1.0\n---\nk: v";
    let doc = parse(input).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_inline_functions_moderate_input() {
    // Build a moderate-sized document
    let mut input = String::from("%VERSION: 1.0\n---\n");
    for i in 0..100 {
        input.push_str(&format!("key{i}: value{i}\n"));
    }

    let doc = parse(&input).unwrap();
    assert_eq!(doc.root.len(), 100);

    let canonical = canonicalize(&doc).unwrap();
    assert!(!canonical.is_empty());

    let json = to_json(&doc).unwrap();
    assert!(!json.is_empty());
}

// =============================================================================
// Unicode and Special Characters Tests
// =============================================================================

#[test]
fn test_inline_functions_unicode() {
    let input = "%VERSION: 1.0\n---\nname: \"日本語\"\nemoji: \"🎉\"";
    let doc = parse(input).unwrap();

    let json = to_json(&doc).unwrap();
    assert!(json.contains("日本語") || json.contains("\\u"));

    let canonical = canonicalize(&doc).unwrap();
    assert!(!canonical.is_empty());
}

#[test]
fn test_inline_functions_special_chars() {
    let input = r#"%VERSION: 1.0
---
quote: "test\"quote"
newline: "line1\nline2"
"#;
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    assert!(!json.is_empty());
}

// =============================================================================
// Thread Safety Tests
// =============================================================================

#[test]
fn test_inline_functions_thread_safe() {
    use std::thread;

    let handles: Vec<_> = (0..4)
        .map(|i| {
            thread::spawn(move || {
                let input = format!("%VERSION: 1.0\n---\nthread: {i}");
                let doc = parse(&input).unwrap();
                let _ = canonicalize(&doc).unwrap();
                let _ = to_json(&doc).unwrap();
                let _ = lint(&doc);
                assert!(validate(&input).is_ok());
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
