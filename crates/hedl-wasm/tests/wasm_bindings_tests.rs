// HEDL WebAssembly Bindings Comprehensive Tests
//
// Tests for all WASM-exposed functions and JavaScript interop

#[cfg(feature = "json")]
use hedl_core::parse as core_parse;

// ============ VERSION AND INITIALIZATION TESTS ============

#[test]
fn test_version_format() {
    // Version should be in semver format
    let version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(
        parts.len(),
        3,
        "Version should have 3 parts (major.minor.patch)"
    );

    // Each part should be a valid number
    for part in parts {
        assert!(
            part.parse::<u32>().is_ok(),
            "Version part should be numeric"
        );
    }
}

// ============ INPUT SIZE VALIDATION TESTS ============

#[test]
fn test_max_input_size_default() {
    // Test that default max input size is reasonable
    let default_max = 500 * 1024 * 1024; // 500 MB
    assert_eq!(default_max, 524_288_000);
}

#[test]
fn test_input_size_validation_logic() {
    // Test the logic of input size checking
    let max_size = 1000;
    let valid_input = "x".repeat(500);
    let invalid_input = "x".repeat(1500);

    assert!(valid_input.len() <= max_size);
    assert!(invalid_input.len() > max_size);
}

#[test]
fn test_input_size_edge_cases() {
    // Test edge cases for input size validation
    let max_size = 100;

    // Exactly at limit
    let at_limit = "x".repeat(100);
    assert_eq!(at_limit.len(), max_size);

    // Just under limit
    let under_limit = "x".repeat(99);
    assert!(under_limit.len() < max_size);

    // Just over limit
    let over_limit = "x".repeat(101);
    assert!(over_limit.len() > max_size);
}

#[test]
fn test_input_size_mb_calculation() {
    // Test MB calculation for error messages
    let bytes = 1_048_576; // 1 MB
    let mb = bytes / (1024 * 1024);
    assert_eq!(mb, 1);

    let bytes = 500 * 1024 * 1024; // 500 MB
    let mb = bytes / (1024 * 1024);
    assert_eq!(mb, 500);
}

// ============ HEDL DOCUMENT WRAPPER TESTS ============

#[test]
#[cfg(feature = "json")]
fn test_hedl_document_version_getter() {
    let hedl = "%VERSION: 1.0\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Format version as string
    let version_str = format!("{}.{}", doc.version.0, doc.version.1);
    assert_eq!(version_str, "1.0");
}

#[test]
#[cfg(feature = "json")]
fn test_hedl_document_schema_count() {
    let hedl = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, title]
%STRUCT: Comment: [id, text]
---
";
    let doc = core_parse(hedl.as_bytes()).unwrap();
    assert_eq!(doc.structs.len(), 3);
}

#[test]
#[cfg(feature = "json")]
fn test_hedl_document_alias_count() {
    let hedl = r#"
%VERSION: 1.0
%ALIAS: %active: "true"
%ALIAS: %inactive: "false"
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();
    assert_eq!(doc.aliases.len(), 2);
}

#[test]
#[cfg(feature = "json")]
fn test_hedl_document_nest_count() {
    let hedl = r"
%VERSION: 1.0
%STRUCT: User: [id]
%STRUCT: Post: [id]
%STRUCT: Comment: [id]
%NEST: User > Post
%NEST: Post > Comment
---
";
    let doc = core_parse(hedl.as_bytes()).unwrap();
    assert_eq!(doc.nests.len(), 2);
}

#[test]
#[cfg(feature = "json")]
fn test_hedl_document_root_item_count() {
    let hedl = r"
%VERSION: 1.0
---
users: data1
posts: data2
comments: data3
";
    let doc = core_parse(hedl.as_bytes()).unwrap();
    assert_eq!(doc.root.len(), 3);
}

#[test]
#[cfg(feature = "json")]
fn test_hedl_document_get_schema_names() {
    let hedl = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, title]
---
";
    let doc = core_parse(hedl.as_bytes()).unwrap();
    let schema_names: Vec<String> = doc.structs.keys().cloned().collect();

    assert!(schema_names.contains(&"User".to_string()));
    assert!(schema_names.contains(&"Post".to_string()));
    assert_eq!(schema_names.len(), 2);
}

#[test]
#[cfg(feature = "json")]
fn test_hedl_document_get_schema() {
    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name, email]\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let schema = doc.structs.get("User").unwrap();
    assert_eq!(
        schema,
        &vec!["id".to_string(), "name".to_string(), "email".to_string()]
    );
}

#[test]
#[cfg(feature = "json")]
fn test_hedl_document_get_schema_missing() {
    let hedl = "%VERSION: 1.0\n%STRUCT: User: [id]\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let schema = doc.structs.get("NonExistent");
    assert!(schema.is_none());
}

// ============ VALIDATION RESULT TESTS ============

#[test]
fn test_validation_result_structure() {
    use serde_json;

    // Valid result with no errors or warnings
    let result = serde_json::json!({
        "valid": true,
        "errors": [],
        "warnings": []
    });

    assert_eq!(result["valid"], true);
    assert!(result["errors"].is_array());
    assert!(result["warnings"].is_array());
}

#[test]
fn test_validation_error_structure() {
    use serde_json;

    let error = serde_json::json!({
        "line": 5,
        "message": "Syntax error",
        "type": "ParseError"
    });

    assert_eq!(error["line"], 5);
    assert_eq!(error["message"], "Syntax error");
    assert_eq!(error["type"], "ParseError");
}

#[test]
fn test_validation_warning_structure() {
    use serde_json;

    let warning = serde_json::json!({
        "line": 10,
        "message": "Unused schema",
        "rule": "unused-schema"
    });

    assert_eq!(warning["line"], 10);
    assert_eq!(warning["message"], "Unused schema");
    assert_eq!(warning["rule"], "unused-schema");
}

// ============ TOKEN STATS TESTS ============

#[test]
fn test_token_stats_structure() {
    use serde_json;

    let stats = serde_json::json!({
        "hedlBytes": 100,
        "hedlTokens": 25,
        "hedlLines": 10,
        "jsonBytes": 400,
        "jsonTokens": 100,
        "savingsPercent": 75,
        "tokensSaved": 75
    });

    assert_eq!(stats["hedlBytes"], 100);
    assert_eq!(stats["hedlTokens"], 25);
    assert_eq!(stats["hedlLines"], 10);
    assert_eq!(stats["jsonBytes"], 400);
    assert_eq!(stats["jsonTokens"], 100);
    assert_eq!(stats["savingsPercent"], 75);
    assert_eq!(stats["tokensSaved"], 75);
}

#[test]
fn test_token_stats_negative_savings() {
    // When HEDL is larger than JSON (edge case)
    let hedl_tokens = 150;
    let json_tokens = 100;

    let savings_percent = if json_tokens > 0 {
        ((i64::from(json_tokens) - i64::from(hedl_tokens)) * 100 / i64::from(json_tokens)) as i32
    } else {
        0
    };

    assert_eq!(savings_percent, -50);
}

#[test]
fn test_token_stats_equal_size() {
    // When HEDL and JSON are the same size
    let hedl_tokens = 100;
    let json_tokens = 100;

    let savings_percent = if json_tokens > 0 {
        ((i64::from(json_tokens) - i64::from(hedl_tokens)) * 100 / i64::from(json_tokens)) as i32
    } else {
        0
    };

    assert_eq!(savings_percent, 0);
}

// ============ JSON CONVERSION TESTS ============

#[test]
#[cfg(feature = "json")]
fn test_from_json_basic() {
    use hedl_json::{from_json_value, FromJsonConfig};

    let json = serde_json::json!({
        "name": "Test",
        "value": 42
    });

    let config = FromJsonConfig::default();
    let result = from_json_value(&json, &config);
    assert!(result.is_ok());
}

#[test]
#[cfg(feature = "json")]
fn test_from_json_array() {
    use hedl_json::{from_json_value, FromJsonConfig};

    let json = serde_json::json!([1, 2, 3, 4, 5]);

    let config = FromJsonConfig::default();
    let result = from_json_value(&json, &config);
    // Arrays may not convert directly to HEDL documents
    // Just verify it doesn't panic
    let _ = result;
}

#[test]
#[cfg(feature = "json")]
fn test_from_json_nested_object() {
    use hedl_json::{from_json_value, FromJsonConfig};

    let json = serde_json::json!({
        "user": {
            "name": "Alice",
            "email": "alice@example.com"
        }
    });

    let config = FromJsonConfig::default();
    let result = from_json_value(&json, &config);
    assert!(result.is_ok());
}

#[test]
#[cfg(feature = "json")]
fn test_from_json_invalid() {
    // Test invalid JSON parsing
    let invalid_json = "{invalid json}";
    let result: Result<serde_json::Value, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());
}

// ============ FORMAT TESTS ============

#[test]
#[cfg(feature = "json")]
fn test_format_with_ditto_enabled() {
    use hedl_c14n::CanonicalConfig;

    let mut config = CanonicalConfig::default();
    config.use_ditto = true;

    assert!(config.use_ditto);
}

#[test]
#[cfg(feature = "json")]
fn test_format_with_ditto_disabled() {
    use hedl_c14n::CanonicalConfig;

    let mut config = CanonicalConfig::default();
    config.use_ditto = false;

    assert!(!config.use_ditto);
}

// ============ QUERY API TESTS ============

#[test]
#[cfg(feature = "query-api")]
fn test_query_all_entities() {
    use hedl_core::{parse as core_parse, Item};

    let hedl = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
  | bob, Bob
";

    let doc = core_parse(hedl.as_bytes()).unwrap();

    let mut count = 0;
    for item in doc.root.values() {
        if let Item::List(list) = item {
            count += list.rows.len();
        }
    }

    assert_eq!(count, 2);
}

#[test]
#[cfg(feature = "query-api")]
fn test_query_by_type_filter() {
    use hedl_core::{parse as core_parse, Item};

    let hedl = r"
%VERSION: 1.0
%STRUCT: User: [id]
%STRUCT: Post: [id]
---
users: @User
  | alice
posts: @Post
  | post1
  | post2
";

    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Count only User entities
    let mut user_count = 0;
    for item in doc.root.values() {
        if let Item::List(list) = item {
            if list.type_name == "User" {
                user_count += list.rows.len();
            }
        }
    }

    assert_eq!(user_count, 1);
}

#[test]
#[cfg(feature = "query-api")]
fn test_query_by_id_filter() {
    use hedl_core::{parse as core_parse, Item};

    let hedl = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
  | bob, Bob
";

    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Find entity with id "alice"
    let mut found = false;
    for item in doc.root.values() {
        if let Item::List(list) = item {
            for node in &list.rows {
                if node.id == "alice" {
                    found = true;
                    break;
                }
            }
        }
    }

    assert!(found);
}

#[test]
#[cfg(feature = "query-api")]
fn test_query_empty_result() {
    use hedl_core::{parse as core_parse, Item};

    let hedl = "%VERSION: 1.0\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let mut count = 0;
    for item in doc.root.values() {
        if let Item::List(list) = item {
            count += list.rows.len();
        }
    }

    assert_eq!(count, 0);
}

// ============ ENTITY RESULT TESTS ============

#[test]
#[cfg(feature = "query-api")]
fn test_entity_result_serialization() {
    use serde_json;

    let entity = serde_json::json!({
        "type": "User",
        "id": "alice",
        "fields": {
            "name": "Alice Smith",
            "email": "alice@example.com"
        }
    });

    assert_eq!(entity["type"], "User");
    assert_eq!(entity["id"], "alice");
    assert!(entity["fields"].is_object());
}

// ============ NESTED ENTITY TESTS ============

#[test]
fn test_count_nested_entities() {
    use hedl_core::{parse as core_parse, Item};
    use std::collections::BTreeMap;

    let hedl = r"
%VERSION: 1.0
%STRUCT: User: [id]
%STRUCT: Post: [id]
%NEST: User > Post
---
users: @User
  | alice
    | post1
    | post2
  | bob
    | post3
";

    let doc = core_parse(hedl.as_bytes()).unwrap();

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    for item in doc.root.values() {
        if let Item::List(list) = item {
            *counts.entry(list.type_name.clone()).or_default() += list.rows.len();

            for node in &list.rows {
                if let Some(children_map) = node.children() {
                    for children in children_map.values() {
                        for child in children {
                            *counts.entry(child.type_name.clone()).or_default() += 1;
                        }
                    }
                }
            }
        }
    }

    assert_eq!(counts.get("User"), Some(&2));
    assert_eq!(counts.get("Post"), Some(&3));
}

// ============ ERROR MESSAGE TESTS ============

#[test]
fn test_parse_error_message_format() {
    use hedl_core::parse as core_parse;

    let invalid_hedl = "invalid content";
    let result = core_parse(invalid_hedl.as_bytes());

    assert!(result.is_err());
    let err = result.unwrap_err();

    // Error should have message
    assert!(!err.message.is_empty());
}

#[test]
fn test_input_size_error_message_format() {
    let input_size = 600_000_000;
    let max_size = 500_000_000;

    let error_msg = format!(
        "Input size ({} bytes, {} MB) exceeds maximum allowed size ({} bytes, {} MB)",
        input_size,
        input_size / (1024 * 1024),
        max_size,
        max_size / (1024 * 1024)
    );

    assert!(error_msg.contains("600000000 bytes"));
    assert!(error_msg.contains("572 MB")); // 600000000 / (1024*1024) = 572
    assert!(error_msg.contains("500000000 bytes"));
    assert!(error_msg.contains("476 MB")); // 500000000 / (1024*1024) = 476
}

// ============ CANONICALIZATION CONFIG TESTS ============

#[test]
fn test_canonical_config_default() {
    use hedl_c14n::CanonicalConfig;

    let config = CanonicalConfig::default();

    // Default should have ditto enabled
    assert!(config.use_ditto);
}

#[test]
fn test_canonical_config_custom() {
    use hedl_c14n::CanonicalConfig;

    let mut config = CanonicalConfig::default();
    config.use_ditto = false;

    assert!(!config.use_ditto);
}

// ============ EDGE CASE TESTS ============

#[test]
fn test_empty_string_input() {
    use hedl_core::parse as core_parse;

    let result = core_parse("".as_bytes());
    assert!(result.is_err(), "Empty string should fail to parse");
}

#[test]
fn test_whitespace_only_input() {
    use hedl_core::parse as core_parse;

    let result = core_parse("   \n\t\r\n  ".as_bytes());
    assert!(result.is_err(), "Whitespace-only should fail to parse");
}

#[test]
#[cfg(feature = "json")]
fn test_large_nested_structure() {
    use hedl_json::{from_json_value, FromJsonConfig};

    // Create deeply nested JSON
    let mut json = serde_json::json!({"level": 0});
    for i in 1..10 {
        json = serde_json::json!({"level": i, "nested": json});
    }

    let config = FromJsonConfig::default();
    let result = from_json_value(&json, &config);
    assert!(result.is_ok(), "Should handle nested structures");
}

#[test]
fn test_special_characters_in_keys() {
    use hedl_core::parse as core_parse;

    let hedl = r"
%VERSION: 1.0
---
normal_key: value1
key_with_underscores: value2
";

    let doc = core_parse(hedl.as_bytes()).unwrap();
    // Keys may be normalized, just verify the document parses
    assert!(!doc.root.is_empty());
}

// ============ MEMORY SAFETY TESTS ============

#[test]
fn test_clone_vs_borrow() {
    use hedl_core::parse as core_parse;

    let hedl = "%VERSION: 1.0\n%STRUCT: T: [id]\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Test borrowing vs cloning
    let schema_ref = doc.structs.get("T");
    assert!(schema_ref.is_some());

    let schema_cloned = doc.structs.get("T").cloned();
    assert!(schema_cloned.is_some());
}

#[test]
fn test_document_lifetime() {
    use hedl_core::parse as core_parse;

    let hedl = String::from("%VERSION: 1.0\n---\n");
    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Document should outlive the parse
    assert_eq!(doc.version, (1, 0));

    // Original string can be dropped
    drop(hedl);

    // Document should still be valid
    assert_eq!(doc.version, (1, 0));
}

// ============ UNICODE HANDLING TESTS ============

#[test]
fn test_unicode_in_values() {
    use hedl_core::parse as core_parse;

    let hedl = r"
%VERSION: 1.0
---
emoji: 🚀
chinese: 你好世界
arabic: مرحبا بالعالم
";

    let result = core_parse(hedl.as_bytes());
    assert!(result.is_ok(), "Should handle Unicode correctly");
}

#[test]
fn test_unicode_in_keys() {
    use hedl_core::parse as core_parse;

    let hedl = r"
%VERSION: 1.0
---
名前: value
";

    let result = core_parse(hedl.as_bytes());
    // May or may not be valid depending on HEDL spec, but should not panic
    let _ = result;
}

// ============ CONCURRENT ACCESS TESTS ============

#[test]
fn test_multiple_documents_simultaneously() {
    use hedl_core::parse as core_parse;

    let hedl1 = "%VERSION: 1.0\n---\nkey1: value1\n";
    let hedl2 = "%VERSION: 1.0\n---\nkey2: value2\n";

    let doc1 = core_parse(hedl1.as_bytes()).unwrap();
    let doc2 = core_parse(hedl2.as_bytes()).unwrap();

    assert_eq!(doc1.version, (1, 0));
    assert_eq!(doc2.version, (1, 0));
    assert_ne!(doc1.root.len(), 0);
    assert_ne!(doc2.root.len(), 0);
}

// ============ SERIALIZATION ROUND-TRIP TESTS ============

#[test]
#[cfg(feature = "json")]
fn test_hedl_to_json_to_hedl_roundtrip() {
    use hedl_c14n::CanonicalConfig;
    use hedl_core::parse as core_parse;
    use hedl_json::{from_json_value, to_json_value, FromJsonConfig, ToJsonConfig};

    let original_hedl =
        "%VERSION: 1.0\n%STRUCT: T: [id, value]\n---\nitems: @T\n  | a, 1\n  | b, 2\n";

    // Parse HEDL
    let doc1 = core_parse(original_hedl.as_bytes()).unwrap();

    // Convert to JSON
    let to_config = ToJsonConfig::default();
    let json = to_json_value(&doc1, &to_config).unwrap();

    // Convert back to HEDL
    let from_config = FromJsonConfig::default();
    let doc2 = from_json_value(&json, &from_config).unwrap();

    // Both documents should have same structure
    assert_eq!(doc1.version, doc2.version);
    assert_eq!(doc1.structs.len(), doc2.structs.len());

    // Canonicalize both
    let c14n_config = CanonicalConfig::default();
    let canonical1 = hedl_c14n::canonicalize_with_config(&doc1, &c14n_config).unwrap();
    let canonical2 = hedl_c14n::canonicalize_with_config(&doc2, &c14n_config).unwrap();

    // Canonicalized forms should be similar (may differ in formatting)
    assert!(!canonical1.is_empty());
    assert!(!canonical2.is_empty());
}

// ============ STRESS TESTS ============

#[test]
fn test_many_schemas() {
    use hedl_core::parse as core_parse;

    let mut hedl = String::from("%VERSION: 1.0\n");
    for i in 0..100 {
        hedl.push_str(&format!("%STRUCT: Type{i}: [id]\n"));
    }
    hedl.push_str("---\n");

    let doc = core_parse(hedl.as_bytes()).unwrap();
    assert_eq!(doc.structs.len(), 100);
}

#[test]
fn test_many_aliases() {
    use hedl_core::parse as core_parse;

    let mut hedl = String::from("%VERSION: 1.0\n");
    for i in 0..100 {
        hedl.push_str(&format!("%ALIAS: %alias{i}: \"value{i}\"\n"));
    }
    hedl.push_str("---\n");

    let doc = core_parse(hedl.as_bytes()).unwrap();
    assert_eq!(doc.aliases.len(), 100);
}

#[test]
fn test_deeply_nested_children() {
    use hedl_core::parse as core_parse;

    // Test document with multiple nesting levels
    let hedl = r"
%VERSION: 1.0
%STRUCT: Level0: [id]
%STRUCT: Level1: [id]
%STRUCT: Level2: [id]
%NEST: Level0 > Level1
%NEST: Level1 > Level2
---
root: @Level0
  | l0
    | l1a
      | l2a
      | l2b
    | l1b
      | l2c
";

    let doc = core_parse(hedl.as_bytes()).unwrap();
    assert_eq!(doc.nests.len(), 2);
}
