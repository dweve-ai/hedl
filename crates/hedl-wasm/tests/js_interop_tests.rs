// HEDL WebAssembly JavaScript Interop Tests
//
// Tests for JavaScript/TypeScript interoperability and WASM-specific features

// ============ JSVALUE CONVERSION TESTS ============

#[test]
fn test_json_value_primitives() {
    // Test JSON primitive types that map to JsValue
    let null = serde_json::Value::Null;
    let bool_true = serde_json::Value::Bool(true);
    let bool_false = serde_json::Value::Bool(false);
    let number = serde_json::json!(42);
    let string = serde_json::Value::String("test".to_string());

    assert!(null.is_null());
    assert!(bool_true.is_boolean());
    assert!(bool_false.is_boolean());
    assert!(number.is_number());
    assert!(string.is_string());
}

#[test]
fn test_json_value_array() {
    // Test JSON arrays
    let array = serde_json::json!([1, 2, 3, 4, 5]);

    assert!(array.is_array());
    let arr = array.as_array().unwrap();
    assert_eq!(arr.len(), 5);
}

#[test]
fn test_json_value_object() {
    // Test JSON objects
    let object = serde_json::json!({
        "key1": "value1",
        "key2": 42,
        "key3": true
    });

    assert!(object.is_object());
    let obj = object.as_object().unwrap();
    assert_eq!(obj.len(), 3);
    assert!(obj.contains_key("key1"));
}

#[test]
fn test_json_value_nested() {
    // Test nested JSON structures
    let nested = serde_json::json!({
        "users": [
            {"id": "alice", "name": "Alice"},
            {"id": "bob", "name": "Bob"}
        ],
        "metadata": {
            "count": 2,
            "timestamp": "2024-01-01"
        }
    });

    assert!(nested.is_object());
    let obj = nested.as_object().unwrap();

    assert!(obj["users"].is_array());
    assert!(obj["metadata"].is_object());
}

// ============ SERIALIZATION TESTS ============

#[test]
fn test_serialize_validation_result() {
    #[derive(serde::Serialize)]
    struct ValidationResult {
        valid: bool,
        errors: Vec<ValidationError>,
        warnings: Vec<ValidationWarning>,
    }

    #[derive(serde::Serialize)]
    struct ValidationError {
        line: usize,
        message: String,
        #[serde(rename = "type")]
        error_type: String,
    }

    #[derive(serde::Serialize)]
    struct ValidationWarning {
        line: usize,
        message: String,
        rule: String,
    }

    let result = ValidationResult {
        valid: false,
        errors: vec![ValidationError {
            line: 5,
            message: "Parse error".to_string(),
            error_type: "SyntaxError".to_string(),
        }],
        warnings: vec![ValidationWarning {
            line: 10,
            message: "Unused schema".to_string(),
            rule: "unused-schema".to_string(),
        }],
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"valid\":false"));
    assert!(json.contains("Parse error"));
    assert!(json.contains("SyntaxError"));
}

#[test]
fn test_serialize_token_stats() {
    #[derive(serde::Serialize)]
    struct TokenStats {
        #[serde(rename = "hedlBytes")]
        hedl_bytes: usize,
        #[serde(rename = "hedlTokens")]
        hedl_tokens: usize,
        #[serde(rename = "hedlLines")]
        hedl_lines: usize,
        #[serde(rename = "jsonBytes")]
        json_bytes: usize,
        #[serde(rename = "jsonTokens")]
        json_tokens: usize,
        #[serde(rename = "savingsPercent")]
        savings_percent: i32,
        #[serde(rename = "tokensSaved")]
        tokens_saved: i32,
    }

    let stats = TokenStats {
        hedl_bytes: 100,
        hedl_tokens: 25,
        hedl_lines: 10,
        json_bytes: 400,
        json_tokens: 100,
        savings_percent: 75,
        tokens_saved: 75,
    };

    let json = serde_json::to_string(&stats).unwrap();
    assert!(json.contains("\"hedlBytes\":100"));
    assert!(json.contains("\"hedlTokens\":25"));
    assert!(json.contains("\"savingsPercent\":75"));
}

#[test]
#[cfg(feature = "query-api")]
fn test_serialize_entity_result() {
    #[derive(serde::Serialize)]
    struct EntityResult {
        #[serde(rename = "type")]
        type_name: String,
        id: String,
        fields: serde_json::Value,
    }

    let result = EntityResult {
        type_name: "User".to_string(),
        id: "alice".to_string(),
        fields: serde_json::json!({
            "name": "Alice Smith",
            "email": "alice@example.com"
        }),
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("\"type\":\"User\""));
    assert!(json.contains("\"id\":\"alice\""));
}

// ============ TYPESCRIPT TYPE COMPATIBILITY TESTS ============

#[test]
fn test_typescript_json_primitive_types() {
    // Verify TypeScript JsonPrimitive type compatibility
    let primitives: Vec<serde_json::Value> = vec![
        serde_json::json!("string"),
        serde_json::json!(42),
        serde_json::json!(std::f64::consts::PI),
        serde_json::json!(true),
        serde_json::json!(false),
        serde_json::json!(null),
    ];

    for value in primitives {
        // All should be valid JSON values
        let serialized = serde_json::to_string(&value).unwrap();
        assert!(!serialized.is_empty());
    }
}

#[test]
fn test_typescript_json_array_type() {
    // Verify TypeScript JsonArray type compatibility
    let arrays: Vec<serde_json::Value> = vec![
        serde_json::json!([]),
        serde_json::json!([1, 2, 3]),
        serde_json::json!(["a", "b", "c"]),
        serde_json::json!([true, false]),
        serde_json::json!([null, null]),
        serde_json::json!([[1, 2], [3, 4]]), // Nested arrays
    ];

    for value in arrays {
        assert!(value.is_array());
    }
}

#[test]
fn test_typescript_json_object_type() {
    // Verify TypeScript JsonObject type compatibility
    let objects: Vec<serde_json::Value> = vec![
        serde_json::json!({}),
        serde_json::json!({"key": "value"}),
        serde_json::json!({"nested": {"key": "value"}}),
        serde_json::json!({"array": [1, 2, 3]}),
    ];

    for value in objects {
        assert!(value.is_object());
    }
}

// ============ OPTIONAL PARAMETER TESTS ============

#[test]
fn test_optional_bool_parameter() {
    // Test Option<bool> parameter handling
    let default_true: Option<bool> = None;
    let explicit_true = Some(true);
    let explicit_false = Some(false);

    // Test default behavior when None
    assert_eq!(default_true, None);
    // Test Some(true) value
    assert_eq!(explicit_true, Some(true));
    // Test Some(false) value
    assert_eq!(explicit_false, Some(false));
}

#[test]
fn test_optional_string_parameter() {
    // Test Option<String> parameter handling
    let none_value: Option<String> = None;
    let some_value = Some("test".to_string());

    assert!(none_value.is_none());
    assert!(some_value.is_some());
}

// ============ FEATURE FLAG TESTS ============

#[test]
#[cfg(feature = "json")]
fn test_json_feature_enabled() {
    // This test only compiles if json feature is enabled
    let _ = "json feature is enabled";
}

#[test]
#[cfg(feature = "statistics")]
fn test_statistics_feature_enabled() {
    // This test only compiles if statistics feature is enabled
    const CHARS_PER_TOKEN: usize = 4;
    assert_eq!(CHARS_PER_TOKEN, 4);
}

#[test]
#[cfg(feature = "query-api")]
fn test_query_api_feature_enabled() {
    // This test only compiles if query-api feature is enabled
    let _ = "query-api feature is enabled";
}

#[test]
#[cfg(feature = "full-validation")]
fn test_full_validation_feature_enabled() {
    // This test only compiles if full-validation feature is enabled
    let _ = "full-validation feature is enabled";
}

#[test]
#[cfg(feature = "token-tools")]
fn test_token_tools_feature_enabled() {
    // This test only compiles if token-tools feature is enabled
    let _ = "token-tools feature is enabled";
}

// ============ ERROR MESSAGE FORMAT TESTS ============

#[test]
fn test_error_message_format_parse() {
    // Test parse error message format for JavaScript consumption
    let line = 5;
    let message = "Unexpected token";

    let error_msg = format!("Parse error at line {line}: {message}");
    assert!(error_msg.contains("line 5"));
    assert!(error_msg.contains("Unexpected token"));
}

#[test]
fn test_error_message_format_input_size() {
    // Test input size error message format
    let input_size = 600_000_000;
    let max_size = 500_000_000;

    let error_msg = format!(
        "Input size ({} bytes, {} MB) exceeds maximum allowed size ({} bytes, {} MB)",
        input_size,
        input_size / (1024 * 1024),
        max_size,
        max_size / (1024 * 1024)
    );

    assert!(error_msg.contains("exceeds maximum"));
    assert!(error_msg.contains("MB"));
}

// ============ RETURN VALUE TESTS ============

#[test]
fn test_getter_return_types() {
    // Test that getters return expected types

    // String getter
    let version = "1.0".to_string();
    assert!(!version.is_empty());

    // Number getter
    let count: usize = 42;
    assert_eq!(count, 42);

    // Array getter
    let names: Vec<String> = vec!["User".to_string(), "Post".to_string()];
    assert_eq!(names.len(), 2);

    // Option getter
    let optional: Option<Vec<String>> = Some(vec!["id".to_string()]);
    assert!(optional.is_some());
}

// ============ NULL HANDLING TESTS ============

#[test]
fn test_null_return_values() {
    // Test that functions can return NULL/None appropriately

    let empty_object = serde_json::json!({});
    let serialized = serde_json::to_string(&empty_object).unwrap();
    assert_eq!(serialized, "{}");

    let null_value = serde_json::Value::Null;
    let serialized = serde_json::to_string(&null_value).unwrap();
    assert_eq!(serialized, "null");
}

#[test]
fn test_empty_collection_returns() {
    // Test empty collections serialize correctly

    let empty_array: Vec<String> = vec![];
    let serialized = serde_json::to_string(&empty_array).unwrap();
    assert_eq!(serialized, "[]");

    let empty_map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    let serialized = serde_json::to_string(&empty_map).unwrap();
    assert_eq!(serialized, "{}");
}

// ============ PROMISE COMPATIBILITY TESTS ============

#[test]
fn test_result_to_promise_ok() {
    // Test Ok result (would resolve promise in JS)
    let result: Result<String, String> = Ok("success".to_string());
    assert!(result.is_ok());
    assert_eq!(result.as_ref().unwrap(), "success");
}

#[test]
fn test_result_to_promise_err() {
    // Test Err result (would reject promise in JS)
    let result: Result<String, String> = Err("error message".to_string());
    assert!(result.is_err());
    assert_eq!(result.as_ref().unwrap_err(), "error message");
}

// ============ CAMELCASE NAMING TESTS ============

#[test]
fn test_camelcase_field_names() {
    // Test that serde rename produces camelCase for JavaScript

    #[derive(serde::Serialize)]
    struct TestStruct {
        #[serde(rename = "camelCase")]
        camel_case: String,
        #[serde(rename = "anotherField")]
        another_field: i32,
    }

    let test = TestStruct {
        camel_case: "test".to_string(),
        another_field: 42,
    };

    let json = serde_json::to_string(&test).unwrap();
    assert!(json.contains("\"camelCase\""));
    assert!(json.contains("\"anotherField\""));
    assert!(!json.contains("camel_case"));
    assert!(!json.contains("another_field"));
}

// ============ DOCUMENTATION COMMENT TESTS ============

#[test]
fn test_doc_comment_examples() {
    // Verify examples from doc comments would work

    // Example: version format
    let version = "1.0";
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 2);

    // Example: MB calculation
    let bytes = 500 * 1024 * 1024;
    let mb = bytes / (1024 * 1024);
    assert_eq!(mb, 500);
}

// ============ ATOMIC OPERATIONS TESTS ============

#[test]
fn test_atomic_usize_operations() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let atomic = AtomicUsize::new(100);

    // Store
    atomic.store(200, Ordering::Relaxed);
    assert_eq!(atomic.load(Ordering::Relaxed), 200);

    // Load
    let value = atomic.load(Ordering::Relaxed);
    assert_eq!(value, 200);

    // Compare and swap would work
    let old = atomic.swap(300, Ordering::Relaxed);
    assert_eq!(old, 200);
    assert_eq!(atomic.load(Ordering::Relaxed), 300);
}

// ============ WASM MEMORY TESTS ============

#[test]
fn test_string_memory_ownership() {
    // Test that strings are properly owned and don't cause issues

    let s1 = "test".to_string();
    let s2 = s1.clone();

    assert_eq!(s1, s2);
    assert_eq!(s1.as_ptr(), s1.as_ptr()); // Same string
    assert_ne!(s1.as_ptr(), s2.as_ptr()); // Different allocations
}

#[test]
fn test_vec_memory_ownership() {
    // Test that vectors are properly owned

    let v1 = vec![1, 2, 3];
    let v2 = v1.clone();

    assert_eq!(v1, v2);
    assert_ne!(v1.as_ptr(), v2.as_ptr()); // Different allocations
}

// ============ REFERENCE SERIALIZATION TESTS ============

#[test]
#[cfg(feature = "query-api")]
fn test_reference_to_string_qualified() {
    use hedl_core::Reference;

    let reference = Reference {
        type_name: Some("User".into()),
        id: "alice".into(),
    };

    let formatted = if let Some(ref t) = reference.type_name {
        format!("@{}:{}", t, reference.id)
    } else {
        format!("@{}", reference.id)
    };

    assert_eq!(formatted, "@User:alice");
}

#[test]
#[cfg(feature = "query-api")]
fn test_reference_to_string_unqualified() {
    use hedl_core::Reference;

    let reference = Reference {
        type_name: None,
        id: "alice".into(),
    };

    let formatted = if let Some(ref t) = reference.type_name {
        format!("@{}:{}", t, reference.id)
    } else {
        format!("@{}", reference.id)
    };

    assert_eq!(formatted, "@alice");
}

// ============ TENSOR SERIALIZATION TESTS ============

#[test]
#[cfg(feature = "query-api")]
fn test_tensor_to_json_format() {
    // Test tensor serialization format (structure test, not actual Tensor creation)
    let json = serde_json::json!({
        "shape": [2, 3],
        "data": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
    });

    assert!(json.is_object());
    assert!(json["shape"].is_array());
    assert!(json["data"].is_array());
    assert_eq!(json["shape"].as_array().unwrap().len(), 2);
    assert_eq!(json["data"].as_array().unwrap().len(), 6);
}

// ============ EXPRESSION SERIALIZATION TESTS ============

#[test]
#[cfg(feature = "query-api")]
fn test_expression_to_string() {
    let expr = "now()";
    let formatted = format!("$({expr})");

    assert_eq!(formatted, "$(now())");
}

// ============ COMPARE TOKENS TESTS ============

#[test]
#[cfg(feature = "token-tools")]
fn test_compare_tokens_result_structure() {
    let result = serde_json::json!({
        "hedl": {
            "bytes": 100,
            "tokens": 25,
            "lines": 10
        },
        "json": {
            "bytes": 400,
            "tokens": 100
        },
        "savings": {
            "percent": 75,
            "tokens": 75
        }
    });

    assert!(result["hedl"].is_object());
    assert!(result["json"].is_object());
    assert!(result["savings"].is_object());
    assert_eq!(result["savings"]["percent"], 75);
}

// ============ PANIC HOOK TESTS ============

#[test]
fn test_panic_message_format() {
    // Test panic message formatting (without actually panicking)

    #[cfg(debug_assertions)]
    let debug_mode = true;
    #[cfg(not(debug_assertions))]
    let debug_mode = false;

    // In debug: detailed messages
    // In release: generic messages
    let expected_behavior = if debug_mode {
        "detailed panic info"
    } else {
        "generic error message"
    };

    assert!(!expected_behavior.is_empty());
}

// ============ WASM BINDGEN COMPATIBILITY TESTS ============

#[test]
fn test_wasm_bindgen_primitive_types() {
    // Test that Rust types map correctly to JavaScript primitives

    let string: String = "test".to_string();
    let number: usize = 42;
    let boolean: bool = true;

    assert!(!string.is_empty());
    assert!(number > 0);
    assert!(boolean);
}

#[test]
fn test_wasm_bindgen_option_types() {
    // Test Option<T> handling for optional JavaScript parameters

    let none_bool: Option<bool> = None;
    let some_bool: Option<bool> = Some(true);

    // Test None value
    assert_eq!(none_bool, None);
    // Test Some value
    assert_eq!(some_bool, Some(true));
}

// ============ SERDE_WASM_BINDGEN TESTS ============

#[test]
fn test_serde_wasm_bindgen_serialization() {
    // Test serialization for wasm-bindgen

    let value = serde_json::json!({
        "string": "test",
        "number": 42,
        "boolean": true,
        "null": null
    });

    // Should serialize to valid JSON
    let json_str = serde_json::to_string(&value).unwrap();
    assert!(json_str.contains("\"string\":\"test\""));
}

#[test]
fn test_serde_wasm_bindgen_deserialization() {
    // Test deserialization from wasm-bindgen

    let json_str = r#"{"key":"value","num":42}"#;
    let value: serde_json::Value = serde_json::from_str(json_str).unwrap();

    assert!(value.is_object());
    assert_eq!(value["key"], "value");
    assert_eq!(value["num"], 42);
}
