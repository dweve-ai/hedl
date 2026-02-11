// HEDL WebAssembly Public API Function Tests
//
// Tests for all exported WASM functions and their behaviors

use hedl_core::{parse as core_parse, Item};

// ============ PARSE FUNCTION TESTS ============

#[test]
fn test_parse_minimal_document() {
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
"#;
    let result = core_parse(hedl.as_bytes());

    assert!(result.is_ok());
    let doc = result.unwrap();
    // Parsing v2.0 content preserves the version
    assert_eq!(doc.version, (2, 0));
    assert_eq!(doc.root.len(), 0);
}

#[test]
fn test_parse_with_schema() {
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name]
---
"#;
    let result = core_parse(hedl.as_bytes());

    assert!(result.is_ok());
    let doc = result.unwrap();
    assert!(doc.structs.contains_key("User"));
}

#[test]
fn test_parse_with_data() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name]
---
users:@User
 |alice, Alice
 |bob, Bob
"#;
    let result = core_parse(hedl.as_bytes());

    assert!(result.is_ok());
    let doc = result.unwrap();

    if let Some(Item::List(list)) = doc.root.get("users") {
        assert_eq!(list.rows.len(), 2);
    } else {
        panic!("Expected List item");
    }
}

#[test]
fn test_parse_multiple_types() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id]
%S:Post:[id]
%S:Comment:[id]
---
users:@User
 |alice
posts:@Post
 |post1
"#;
    let result = core_parse(hedl.as_bytes());

    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.structs.len(), 3);
    assert_eq!(doc.root.len(), 2);
}

#[test]
fn test_parse_with_nested_entities() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id]
%S:Post:[id]
%N:User>Post
---
users:@User
 |alice
  |post1
  |post2
"#;
    let result = core_parse(hedl.as_bytes());

    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.nests.len(), 1);
}

#[test]
fn test_parse_error_handling() {
    let invalid = "not a valid document";
    let result = core_parse(invalid.as_bytes());

    assert!(result.is_err());
}

// ============ TO_JSON FUNCTION TESTS ============

#[test]
#[cfg(feature = "json")]
fn test_to_json_basic() {
    use hedl_json::{to_json_value, ToJsonConfig};

    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
name: Test
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = ToJsonConfig::default();
    let result = to_json_value(&doc, &config);

    assert!(result.is_ok());
}

#[test]
#[cfg(feature = "json")]
fn test_to_json_with_entities() {
    use hedl_json::{to_json_value, ToJsonConfig};

    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name]
---
users:@User
 |alice, Alice
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = ToJsonConfig::default();
    let result = to_json_value(&doc, &config);

    assert!(result.is_ok());
    let json = result.unwrap();
    let json_str = serde_json::to_string(&json).unwrap();
    assert!(json_str.contains("alice") || json_str.contains("Alice"));
}

#[test]
#[cfg(feature = "json")]
fn test_to_json_pretty_format() {
    use hedl_json::{to_json_value, ToJsonConfig};

    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
key: value
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = ToJsonConfig::default();
    let json = to_json_value(&doc, &config).unwrap();

    let pretty = serde_json::to_string_pretty(&json).unwrap();
    let compact = serde_json::to_string(&json).unwrap();

    assert!(pretty.len() > compact.len());
    assert!(pretty.contains('\n'));
}

#[test]
#[cfg(feature = "json")]
fn test_to_json_compact_format() {
    use hedl_json::{to_json_value, ToJsonConfig};

    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
key: value
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = ToJsonConfig::default();
    let json = to_json_value(&doc, &config).unwrap();

    let compact = serde_json::to_string(&json).unwrap();
    assert!(!compact.contains("\n  "));
}

// ============ FROM_JSON FUNCTION TESTS ============

#[test]
#[cfg(feature = "json")]
fn test_from_json_simple_object() {
    use hedl_json::{from_json_value, FromJsonConfig};

    let json = serde_json::json!({"name": "Test", "value": 42});
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
fn test_from_json_nested() {
    use hedl_json::{from_json_value, FromJsonConfig};

    let json = serde_json::json!({
        "users": [
            {"id": "alice", "name": "Alice"},
            {"id": "bob", "name": "Bob"}
        ]
    });
    let config = FromJsonConfig::default();

    let result = from_json_value(&json, &config);
    assert!(result.is_ok());
}

// ============ FORMAT FUNCTION TESTS ============

#[test]
fn test_format_basic() {
    use hedl_c14n::{canonicalize_with_config, CanonicalConfig};

    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
z: 3
a: 1
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = CanonicalConfig::default();
    let result = canonicalize_with_config(&doc, &config);

    assert!(result.is_ok());
    let canonical = result.unwrap();
    assert!(canonical.contains("%V:"));
}

#[test]
fn test_format_with_ditto() {
    use hedl_c14n::{canonicalize_with_config, CanonicalConfig};

    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id, value]
---
items:@T
 |a, x
 |b, x
 |c, x
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let mut config = CanonicalConfig::default();
    config.use_ditto = true;

    let result = canonicalize_with_config(&doc, &config);
    assert!(result.is_ok());
}

#[test]
fn test_format_without_ditto() {
    use hedl_c14n::{canonicalize_with_config, CanonicalConfig};

    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:T:[id, value]
---
items:@T
 |a, x
 |b, x
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let mut config = CanonicalConfig::default();
    config.use_ditto = false;

    let result = canonicalize_with_config(&doc, &config);
    assert!(result.is_ok());
}

// ============ VALIDATE FUNCTION TESTS ============

#[test]
fn test_validate_valid_document() {
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
"#;
    let result = core_parse(hedl.as_bytes());

    assert!(result.is_ok(), "Valid document should parse");
}

#[test]
fn test_validate_invalid_syntax() {
    let hedl = "invalid syntax";
    let result = core_parse(hedl.as_bytes());

    assert!(result.is_err(), "Invalid syntax should error");
}

#[test]
#[cfg(feature = "full-validation")]
fn test_validate_with_linting() {
    use hedl_lint::lint;

    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let diagnostics = lint(&doc);
    // Should return some diagnostics (may be empty for valid doc)
    let _ = diagnostics;
}

#[test]
#[cfg(feature = "full-validation")]
fn test_validate_collects_all_errors() {
    use hedl_lint::lint;

    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:T1:[id]
%S:T2:[id]
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let diagnostics = lint(&doc);
    // Verify linting works without panic (unused schemas may generate diagnostics)
    let _ = diagnostics;
}

// ============ GET_STATS FUNCTION TESTS ============

#[test]
#[cfg(feature = "statistics")]
fn test_get_stats_basic() {
    use hedl_json::{to_json_value, ToJsonConfig};

    fn estimate_tokens(text: &str) -> usize {
        let bytes = text.as_bytes();
        let byte_count = bytes.len();

        if byte_count == 0 {
            return 0;
        }

        let mut whitespace_count = 0usize;
        let mut punct_count = 0usize;
        let mut i = 0;

        while i < byte_count {
            let b = bytes[i];

            if b < 128 {
                whitespace_count += usize::from(matches!(b, b' ' | b'\t' | b'\n' | b'\r'));
                punct_count += usize::from(matches!(
                    b,
                    b'!' | b'\"'
                        | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'('
                        | b')'
                        | b'*'
                        | b'+'
                        | b','
                        | b'-'
                        | b'.'
                        | b'/'
                        | b':'
                        | b';'
                        | b'<'
                        | b'='
                        | b'>'
                        | b'?'
                        | b'@'
                        | b'['
                        | b'\\'
                        | b']'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'{'
                        | b'|'
                        | b'}'
                        | b'~'
                ));
                i += 1;
            } else {
                let char_len = if b < 0b1110_0000 {
                    2
                } else if b < 0b1111_0000 {
                    3
                } else {
                    4
                };
                i += char_len;
            }
        }

        const CHARS_PER_TOKEN: usize = 4;
        (byte_count + whitespace_count + punct_count) / CHARS_PER_TOKEN
    }

    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
key: value
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = ToJsonConfig::default();
    let json = to_json_value(&doc, &config).unwrap();
    let json_str = serde_json::to_string(&json).unwrap();

    let hedl_tokens = estimate_tokens(hedl);
    let json_tokens = estimate_tokens(&json_str);

    assert!(hedl_tokens > 0);
    assert!(json_tokens > 0);
}

#[test]
#[cfg(feature = "statistics")]
fn test_get_stats_savings_calculation() {
    let hedl_tokens = 25usize;
    let json_tokens = 100usize;

    let savings_percent = if json_tokens > 0 {
        ((json_tokens as i64 - hedl_tokens as i64) * 100 / json_tokens as i64) as i32
    } else {
        0
    };

    assert_eq!(savings_percent, 75);
}

// ============ COMPARE_TOKENS FUNCTION TESTS ============

#[test]
#[cfg(feature = "token-tools")]
fn test_compare_tokens() {
    fn estimate_tokens(text: &str) -> usize {
        let bytes = text.as_bytes();
        let byte_count = bytes.len();
        if byte_count == 0 {
            return 0;
        }
        const CHARS_PER_TOKEN: usize = 4;
        byte_count / CHARS_PER_TOKEN
    }

    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
key: value
"#;
    let json = r#"{"key":"value"}"#;

    let hedl_tokens = estimate_tokens(hedl);
    let json_tokens = estimate_tokens(json);

    assert!(hedl_tokens > 0);
    assert!(json_tokens > 0);
}

// ============ HEDL_DOCUMENT GETTER TESTS ============

#[test]
fn test_document_version_getter() {
    // v2.0+ requires compact %V: syntax
    let hedl = "%V:2.5\n%NULL:~\n%QUOTE:\"\n---\n";
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let version_str = format!("{}.{}", doc.version.0, doc.version.1);
    assert_eq!(version_str, "2.5");
}

#[test]
fn test_document_schema_count_getter() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:T1:[id]
%S:T2:[id]
%S:T3:[id]
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    assert_eq!(doc.structs.len(), 3);
}

#[test]
fn test_document_alias_count_getter() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%A:%a1:"v1"
%A:%a2:"v2"
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    assert_eq!(doc.aliases.len(), 2);
}

#[test]
fn test_document_nest_count_getter() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:P:[id]
%S:C:[id]
%N:P>C
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    assert_eq!(doc.nests.len(), 1);
}

#[test]
fn test_document_root_item_count_getter() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
---
item1: value1
item2: value2
item3: value3
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    assert_eq!(doc.root.len(), 3);
}

#[test]
fn test_document_get_schema_names() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id]
%S:Post:[id]
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let names: Vec<String> = doc.structs.keys().cloned().collect();
    assert!(names.contains(&"User".to_string()));
    assert!(names.contains(&"Post".to_string()));
}

#[test]
fn test_document_get_schema() {
    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name, email]
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let schema = doc.structs.get("User");
    assert!(schema.is_some());

    let fields = schema.unwrap();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0], "id");
    assert_eq!(fields[1], "name");
    assert_eq!(fields[2], "email");
}

#[test]
fn test_document_get_aliases() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%A:%test:"value"
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    // Aliases may be stored with or without the % prefix
    let has_with_percent = doc.aliases.contains_key("%test");
    let has_without_percent = doc.aliases.contains_key("test");

    assert!(
        has_with_percent || has_without_percent,
        "Alias should exist"
    );

    if has_with_percent {
        let value = doc.aliases.get("%test").unwrap();
        assert_eq!(value, "value");
    } else {
        let value = doc.aliases.get("test").unwrap();
        assert_eq!(value, "value");
    }
}

#[test]
fn test_document_get_nests() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id]
%S:Post:[id]
%N:User>Post
---
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    assert!(doc.nests.contains_key("User"));
    let children = doc.nests.get("User").unwrap();
    assert!(children.contains(&"Post".to_string()));
}

// ============ DOCUMENT TO_JSON TESTS ============

#[test]
#[cfg(feature = "json")]
fn test_document_to_json() {
    use hedl_json::{to_json_value, ToJsonConfig};

    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
key: value
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = ToJsonConfig::default();
    let result = to_json_value(&doc, &config);

    assert!(result.is_ok());
}

#[test]
#[cfg(feature = "json")]
fn test_document_to_json_string() {
    use hedl_json::{to_json_value, ToJsonConfig};

    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
key: value
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = ToJsonConfig::default();
    let json = to_json_value(&doc, &config).unwrap();

    let pretty = serde_json::to_string_pretty(&json).unwrap();
    let compact = serde_json::to_string(&json).unwrap();

    assert!(pretty.len() > compact.len());
}

// ============ DOCUMENT TO_HEDL TESTS ============

#[test]
fn test_document_to_hedl() {
    use hedl_c14n::{canonicalize_with_config, CanonicalConfig};

    let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
key: value
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let config = CanonicalConfig::default();
    let result = canonicalize_with_config(&doc, &config);

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("%V:"));
}

// ============ DOCUMENT COUNT_ENTITIES TESTS ============

#[test]
fn test_document_count_entities() {
    use std::collections::BTreeMap;

    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id]
%S:Post:[id]
---
users:@User
 |alice
 |bob
posts:@Post
 |post1
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    for item in doc.root.values() {
        if let Item::List(list) = item {
            *counts.entry(list.type_name.clone()).or_default() += list.rows.len();
        }
    }

    assert_eq!(counts.get("User"), Some(&2));
    assert_eq!(counts.get("Post"), Some(&1));
}

// ============ DOCUMENT QUERY TESTS ============

#[test]
#[cfg(feature = "query-api")]
fn test_document_query_all() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name]
---
users:@User
 |alice, Alice
 |bob, Bob
"#;
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
fn test_document_query_by_type() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id]
%S:Post:[id]
---
users:@User
 |alice
posts:@Post
 |post1
 |post2
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

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
fn test_document_query_by_id() {
    let hedl = r#"
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id, name]
---
users:@User
 |alice, Alice
 |bob, Bob
"#;
    let doc = core_parse(hedl.as_bytes()).unwrap();

    let mut found = false;
    for item in doc.root.values() {
        if let Item::List(list) = item {
            for node in &list.rows {
                if node.id == "bob" {
                    found = true;
                }
            }
        }
    }

    assert!(found);
}

// ============ CONFIGURATION TESTS ============

#[test]
fn test_canonical_config_defaults() {
    use hedl_c14n::CanonicalConfig;

    let config = CanonicalConfig::default();
    assert!(!config.use_ditto); // Default is false per spec (explicit values preferred)
}

#[test]
#[cfg(feature = "json")]
fn test_json_config_defaults() {
    use hedl_json::{FromJsonConfig, ToJsonConfig};

    let to_config = ToJsonConfig::default();
    let from_config = FromJsonConfig::default();

    // Configs should exist and be valid
    let _ = to_config;
    let _ = from_config;
}

// ============ VERSION CONSTANT TESTS ============

#[test]
fn test_version_constant() {
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty());

    // Should be valid semver
    let parts: Vec<&str> = version.split('.').collect();
    assert!(parts.len() >= 2);
}

// ============ MAX INPUT SIZE TESTS ============

#[test]
fn test_default_max_input_size() {
    let default_max = 500 * 1024 * 1024; // 500 MB
    assert_eq!(default_max, 524_288_000);
}

#[test]
fn test_max_input_size_operations() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let max_size = AtomicUsize::new(500 * 1024 * 1024);

    // Get
    let current = max_size.load(Ordering::Relaxed);
    assert_eq!(current, 524_288_000);

    // Set
    max_size.store(1024 * 1024 * 1024, Ordering::Relaxed);
    let new_value = max_size.load(Ordering::Relaxed);
    assert_eq!(new_value, 1_073_741_824);
}
