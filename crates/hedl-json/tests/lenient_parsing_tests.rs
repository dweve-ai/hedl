// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tests for lenient JSON parsing (JSON5-style trailing commas and comments)

#![cfg(feature = "lenient")]

use hedl_json::{from_json, FromJsonConfig};

// ============================================================================
// Trailing Comma Tests
// ============================================================================

#[test]
fn test_trailing_comma_object() {
    let json = r#"{
        "name": "Alice",
        "age": 30,
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert_eq!(doc.root.len(), 2);
    assert!(doc.root.contains_key("name"));
    assert!(doc.root.contains_key("age"));
}

#[test]
fn test_trailing_comma_array() {
    // Use objects in array for valid HEDL matrix list
    let json = r#"{
        "tags": [
            {"name": "rust",},
            {"name": "json",},
            {"name": "parsing",},
        ]
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("tags"));
}

#[test]
fn test_trailing_comma_nested() {
    let json = r#"{
        "users": [
            {
                "id": 1,
                "name": "Alice",
            },
            {
                "id": 2,
                "name": "Bob",
            },
        ],
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("users"));
}

#[test]
fn test_strict_mode_rejects_trailing_comma() {
    let json = r#"{"name": "Alice",}"#;

    // Default config is strict (lenient: false)
    let config = FromJsonConfig::builder().lenient(false).build();
    let result = from_json(json, &config);

    // Should fail with parse error
    assert!(result.is_err());
}

#[test]
fn test_lenient_mode_accepts_strict_json() {
    let json = r#"{"name": "Alice", "age": 30}"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    // Lenient mode should accept strict JSON
    assert_eq!(doc.root.len(), 2);
}

#[test]
fn test_multiple_trailing_commas() {
    let json = r#"{
        "a": [1, 2, 3,],
        "b": [4, 5, 6,],
        "c": {
            "x": 1,
            "y": 2,
        },
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert_eq!(doc.root.len(), 3);
}

#[test]
fn test_empty_array_with_trailing_elements() {
    let json = r#"{"items": [1,]}"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("items"));
}

// ============================================================================
// Comment Tests
// ============================================================================

#[test]
fn test_single_line_comment() {
    let json = r#"{
        // User configuration
        "name": "Alice",
        "age": 30
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert_eq!(doc.root.len(), 2);
}

#[test]
fn test_multi_line_comment() {
    let json = r#"{
        /*
         * User configuration
         * for the test
         */
        "name": "Alice",
        "age": 30
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert_eq!(doc.root.len(), 2);
}

#[test]
fn test_inline_comment() {
    let json = r#"{
        "name": "Alice", // First name
        "age": 30 /* years old */
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert_eq!(doc.root.len(), 2);
}

#[test]
fn test_comments_with_trailing_commas() {
    let json = r#"{
        // User data
        "name": "Alice",
        /* Age in years */
        "age": 30,
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert_eq!(doc.root.len(), 2);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_deeply_nested_with_trailing_commas() {
    let json = r#"{
        "level1": {
            "level2": {
                "level3": {
                    "value": 42,
                },
            },
        },
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("level1"));
}

#[test]
fn test_array_of_arrays_with_trailing_commas() {
    let json = r#"{
        "matrix": [
            [1, 2, 3,],
            [4, 5, 6,],
            [7, 8, 9,],
        ],
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("matrix"));
}

#[test]
fn test_mixed_types_with_trailing_commas() {
    let json = r#"{
        "string": "value",
        "number": 42,
        "float": 3.14,
        "bool": true,
        "null": null,
        "array": [1, 2, 3,],
        "object": {"nested": true,},
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert_eq!(doc.root.len(), 7);
}

// ============================================================================
// Default Behavior Tests
// ============================================================================

#[test]
fn test_default_config_is_strict() {
    // The default config should have lenient=false
    let config = FromJsonConfig::default();
    assert!(!config.lenient);
}

#[test]
fn test_builder_default_is_strict() {
    let config = FromJsonConfig::builder().build();
    assert!(!config.lenient);
}

// ============================================================================
// Real-World Configuration File Tests
// ============================================================================

#[test]
fn test_vscode_settings_style() {
    let json = r#"{
        // Editor settings
        "editor.fontSize": 14,
        "editor.tabSize": 2,
        "editor.insertSpaces": true,

        // Terminal settings
        "terminal.integrated.fontSize": 12,

        // Files
        "files.exclude": {
            "**/node_modules": true,
            "**/.git": true,
        },
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("editor.fontSize"));
    assert!(doc.root.contains_key("files.exclude"));
}

#[test]
fn test_package_json_style() {
    let json = r#"{
        "name": "example-package",
        "version": "1.0.0",
        "dependencies": {
            "lodash": "^4.17.21",
            "axios": "^0.27.2",
        },
        "scripts": {
            "build": "npm run compile",
            "test": "jest",
        },
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("name"));
    assert!(doc.root.contains_key("dependencies"));
    assert!(doc.root.contains_key("scripts"));
}

#[test]
fn test_tsconfig_style() {
    // Use objects in arrays for valid HEDL matrix list
    let json = r#"{
        // TypeScript compiler configuration
        "compilerOptions": {
            "target": "ES2020",
            "module": "commonjs",
            "strict": true,
            /* Output settings */
            "outDir": "./dist",
            "rootDir": "./src",
        },
        "include": [
            {"pattern": "src/**/*",},
        ],
        "exclude": [
            {"pattern": "node_modules",},
            {"pattern": "dist",},
        ],
    }"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let doc = from_json(json, &config).unwrap();

    assert!(doc.root.contains_key("compilerOptions"));
    assert!(doc.root.contains_key("include"));
    assert!(doc.root.contains_key("exclude"));
}

// ============================================================================
// Security Tests (DoS limits still apply)
// ============================================================================

#[test]
fn test_lenient_mode_respects_depth_limit() {
    let mut json = String::from("{\"a\":");
    for _ in 0..100 {
        json.push_str("{\"b\":");
    }
    json.push('1');
    for _ in 0..100 {
        json.push('}');
    }
    json.push_str(",}"); // Trailing comma

    let config = FromJsonConfig::builder()
        .lenient(true)
        .max_depth(50)
        .build();

    let result = from_json(&json, &config);

    // Should fail due to depth limit, not trailing comma
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("depth") || err.contains("Depth"));
}

#[test]
fn test_lenient_mode_respects_array_size_limit() {
    let mut json = String::from("{\"items\": [");
    for i in 0..1000 {
        if i > 0 {
            json.push_str(", ");
        }
        json.push_str(&i.to_string());
    }
    json.push_str(",]}"); // Trailing comma

    let config = FromJsonConfig::builder()
        .lenient(true)
        .max_array_size(100)
        .build();

    let result = from_json(&json, &config);

    // Should fail due to array size limit
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("array") || err.contains("Array") || err.contains("size"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_json_still_fails_in_lenient_mode() {
    let json = r#"{"name": "Alice" "age": 30}"#; // Missing comma

    let config = FromJsonConfig::builder().lenient(true).build();
    let result = from_json(json, &config);

    // Should still fail - lenient mode doesn't fix all errors
    assert!(result.is_err());
}

#[test]
fn test_unclosed_brace_fails_in_lenient_mode() {
    let json = r#"{"name": "Alice","#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let result = from_json(json, &config);

    assert!(result.is_err());
}

#[test]
fn test_unclosed_bracket_fails_in_lenient_mode() {
    let json = r#"{"items": [1, 2, 3}"#;

    let config = FromJsonConfig::builder().lenient(true).build();
    let result = from_json(json, &config);

    assert!(result.is_err());
}
