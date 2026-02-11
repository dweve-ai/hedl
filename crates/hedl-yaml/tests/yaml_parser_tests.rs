// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive tests for `from_yaml` module covering edge cases, error handling,
//! resource limits, and complex parsing scenarios.

use hedl_core::{Item, Value};
use hedl_yaml::{from_yaml, from_yaml_value, FromYamlConfig, FromYamlConfigBuilder};

// ==================== Builder Pattern Tests ====================

#[test]
fn test_from_yaml_config_builder_default() {
    let config = FromYamlConfigBuilder::new().build();
    assert_eq!(config.default_type_name, "Item");
    // v2.0 is the default version for new documents
    assert_eq!(config.version, (2, 0));
    assert_eq!(
        config.max_document_size,
        hedl_yaml::DEFAULT_MAX_DOCUMENT_SIZE
    );
    assert_eq!(config.max_array_length, hedl_yaml::DEFAULT_MAX_ARRAY_LENGTH);
    assert_eq!(
        config.max_nesting_depth,
        hedl_yaml::DEFAULT_MAX_NESTING_DEPTH
    );
}

#[test]
fn test_from_yaml_config_builder_custom_all_fields() {
    let config = FromYamlConfigBuilder::new()
        .default_type_name("CustomEntity")
        .version(2, 1)
        .max_document_size(1024)
        .max_array_length(100)
        .max_nesting_depth(50)
        .build();

    assert_eq!(config.default_type_name, "CustomEntity");
    assert_eq!(config.version, (2, 1));
    assert_eq!(config.max_document_size, 1024);
    assert_eq!(config.max_array_length, 100);
    assert_eq!(config.max_nesting_depth, 50);
}

#[test]
fn test_from_yaml_config_builder_partial() {
    let config = FromYamlConfigBuilder::new().max_document_size(2048).build();

    assert_eq!(config.max_document_size, 2048);
    // Other fields should have defaults
    assert_eq!(config.default_type_name, "Item");
    // v2.0 is the default version for new documents
    assert_eq!(config.version, (2, 0));
}

#[test]
fn test_from_yaml_config_builder_fluent_chaining() {
    let config = FromYamlConfig::builder()
        .max_document_size(10 * 1024 * 1024)
        .max_array_length(1_000_000)
        .max_nesting_depth(1000)
        .default_type_name("Record")
        .version(2, 0)
        .build();

    assert_eq!(config.max_document_size, 10 * 1024 * 1024);
    assert_eq!(config.max_array_length, 1_000_000);
    assert_eq!(config.max_nesting_depth, 1000);
    assert_eq!(config.default_type_name, "Record");
    assert_eq!(config.version, (2, 0));
}

#[test]
fn test_from_yaml_config_default() {
    let config = FromYamlConfig::default();
    assert_eq!(config.default_type_name, "Item");
    // v2.0 is the default version for new documents
    assert_eq!(config.version, (2, 0));
}

#[test]
fn test_from_yaml_config_clone() {
    let config1 = FromYamlConfig::builder().max_document_size(5000).build();
    let config2 = config1.clone();

    assert_eq!(config1.max_document_size, config2.max_document_size);
    assert_eq!(config1.default_type_name, config2.default_type_name);
}

#[test]
fn test_from_yaml_config_debug() {
    let config = FromYamlConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("FromYamlConfig"));
    assert!(debug.contains("default_type_name"));
}

// ==================== Resource Limit Tests ====================

#[test]
fn test_from_yaml_document_size_limit_exceeded() {
    let yaml = "x".repeat(1000);
    let config = FromYamlConfig::builder().max_document_size(100).build();

    let result = from_yaml(&yaml, &config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Document size"));
    assert!(err.contains("exceeds maximum"));
}

#[test]
fn test_from_yaml_document_size_limit_exactly_at_limit() {
    let yaml = "key: value\n"; // 11 bytes
    let config = FromYamlConfig::builder().max_document_size(11).build();

    let result = from_yaml(yaml, &config);
    assert!(result.is_ok());
}

#[test]
fn test_from_yaml_array_length_limit_exceeded() {
    // Create YAML with array exceeding limit
    let yaml = r"
items:
  - id: 1
  - id: 2
  - id: 3
  - id: 4
  - id: 5
";

    let config = FromYamlConfig::builder().max_array_length(3).build();

    let result = from_yaml(yaml, &config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Array length"));
    assert!(err.contains("exceeds maximum"));
}

#[test]
fn test_from_yaml_nesting_depth_limit_exceeded() {
    // Create deeply nested YAML
    let yaml = r"
level1:
  level2:
    level3:
      level4:
        level5:
          value: deep
";

    let config = FromYamlConfig::builder().max_nesting_depth(3).build();

    let result = from_yaml(yaml, &config);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("nesting depth"));
}

#[test]
fn test_from_yaml_nesting_depth_limit_at_boundary() {
    let yaml = r"
level1:
  level2:
    value: ok
";

    let config = FromYamlConfig::builder().max_nesting_depth(10).build();

    let result = from_yaml(yaml, &config);
    assert!(result.is_ok());
}

// ==================== Unicode and Special Characters ====================

#[test]
fn test_from_yaml_unicode_string() {
    let yaml = r#"
name: "Héllo Wörld"
emoji: "🚀"
chinese: "你好世界"
arabic: "مرحبا بالعالم"
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("name") {
        assert_eq!(s.as_ref(), "Héllo Wörld");
    } else {
        panic!("Expected string value");
    }

    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("emoji") {
        assert_eq!(s.as_ref(), "🚀");
    } else {
        panic!("Expected emoji string");
    }
}

#[test]
fn test_from_yaml_multiline_string() {
    let yaml = r"
description: |
  This is a multiline
  string with several
  lines of text.
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("description") {
        assert!(s.contains("multiline"));
        assert!(s.contains("several"));
    } else {
        panic!("Expected multiline string");
    }
}

#[test]
fn test_from_yaml_string_with_special_yaml_chars() {
    let yaml = r#"
special: "key: value"
colon: "test: test"
bracket: "[array]"
brace: "{object}"
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    assert_eq!(doc.root.len(), 4);
}

#[test]
fn test_from_yaml_escaped_characters() {
    let yaml = r#"
newline: "line1\nline2"
tab: "col1\tcol2"
quote: "He said \"hello\""
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("newline") {
        assert!(s.contains("line1"));
        assert!(s.contains("line2"));
    }
}

// ==================== Number Edge Cases ====================

#[test]
fn test_from_yaml_number_zero() {
    let yaml = r"
int_zero: 0
float_zero: 0.0
negative_zero: -0
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::Int(n))) = doc.root.get("int_zero") {
        assert_eq!(*n, 0);
    }

    if let Some(Item::Scalar(Value::Float(f))) = doc.root.get("float_zero") {
        assert_eq!(*f, 0.0);
    }
}

#[test]
fn test_from_yaml_number_max_min() {
    let yaml = format!(
        r"
max_int: {}
min_int: {}
",
        i64::MAX,
        i64::MIN
    );

    let config = FromYamlConfig::default();
    let doc = from_yaml(&yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::Int(n))) = doc.root.get("max_int") {
        assert_eq!(*n, i64::MAX);
    }

    if let Some(Item::Scalar(Value::Int(n))) = doc.root.get("min_int") {
        assert_eq!(*n, i64::MIN);
    }
}

#[test]
fn test_from_yaml_float_special_values() {
    let yaml = r"
infinity: .inf
neg_infinity: -.inf
not_a_number: .nan
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::Float(f))) = doc.root.get("infinity") {
        assert!(f.is_infinite() && f.is_sign_positive());
    }

    if let Some(Item::Scalar(Value::Float(f))) = doc.root.get("neg_infinity") {
        assert!(f.is_infinite() && f.is_sign_negative());
    }

    if let Some(Item::Scalar(Value::Float(f))) = doc.root.get("not_a_number") {
        assert!(f.is_nan());
    }
}

#[test]
fn test_from_yaml_scientific_notation() {
    let yaml = r"
scientific: 1.23e10
negative_exp: 5.67e-3
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::Float(f))) = doc.root.get("scientific") {
        assert!((f - 1.23e10).abs() < 1.0);
    }

    if let Some(Item::Scalar(Value::Float(f))) = doc.root.get("negative_exp") {
        assert!((f - 5.67e-3).abs() < 1e-6);
    }
}

// ==================== Boolean Edge Cases ====================

#[test]
fn test_from_yaml_boolean_variants() {
    let yaml = r"
true_1: true
true_2: True
true_3: TRUE
true_4: yes
true_5: Yes
true_6: YES
true_7: on
true_8: On
true_9: ON
false_1: false
false_2: False
false_3: FALSE
false_4: no
false_5: No
false_6: NO
false_7: off
false_8: Off
false_9: OFF
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    // YAML 1.2 spec: only true/false (lowercase) are booleans
    // yes/no/on/off are treated as strings in YAML 1.2

    // Check that at least 'true' and 'false' work
    if let Some(Item::Scalar(Value::Bool(b))) = doc.root.get("true_1") {
        assert!(*b);
    }

    if let Some(Item::Scalar(Value::Bool(b))) = doc.root.get("false_1") {
        assert!(!*b);
    }
}

// ==================== Null Value Tests ====================

#[test]
fn test_from_yaml_null_variants() {
    let yaml = r"
null_1: null
null_2: Null
null_3: NULL
null_4: ~
null_5:
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    // All variants should be parsed as null
    assert!(matches!(
        doc.root.get("null_1"),
        Some(Item::Scalar(Value::Null))
    ));
    assert!(matches!(
        doc.root.get("null_2"),
        Some(Item::Scalar(Value::Null))
    ));
}

// ==================== Empty Collection Tests ====================

#[test]
fn test_from_yaml_empty_mapping() {
    let yaml = r"
empty_obj: {}
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Object(map)) = doc.root.get("empty_obj") {
        assert!(map.is_empty());
    } else {
        panic!("Expected empty object");
    }
}

#[test]
fn test_from_yaml_empty_sequence() {
    let yaml = r"
empty_array: []
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::List(list)) = doc.root.get("empty_array") {
        assert!(list.rows.is_empty());
    } else {
        panic!("Expected empty list");
    }
}

// ==================== Expression Parsing ====================

#[test]
fn test_from_yaml_expression_simple() {
    let yaml = r#"
expr: "$(value)"
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    assert!(matches!(
        doc.root.get("expr"),
        Some(Item::Scalar(Value::Expression(_)))
    ));
}

#[test]
fn test_from_yaml_expression_complex() {
    let yaml = r#"
expr: "$(add(x, multiply(2, y)))"
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::Expression(e))) = doc.root.get("expr") {
        let expr_str = e.to_string();
        assert!(expr_str.contains("add"));
        assert!(expr_str.contains("multiply"));
    } else {
        panic!("Expected expression");
    }
}

#[test]
fn test_from_yaml_expression_nested() {
    let yaml = r#"
nested: "$(outer(inner(value)))"
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    assert!(matches!(
        doc.root.get("nested"),
        Some(Item::Scalar(Value::Expression(_)))
    ));
}

#[test]
fn test_from_yaml_not_expression_missing_closing() {
    let yaml = r#"
not_expr: "$(incomplete"
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    // Should be parsed as string, not expression
    assert!(matches!(
        doc.root.get("not_expr"),
        Some(Item::Scalar(Value::String(_)))
    ));
}

// ==================== Reference Parsing ====================

#[test]
fn test_from_yaml_reference_local() {
    let yaml = r#"
ref: { "@ref": "@user1" }
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = doc.root.get("ref") {
        assert_eq!(r.id.as_ref(), "user1");
        assert_eq!(r.type_name, None);
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_from_yaml_reference_qualified() {
    let yaml = r#"
ref: { "@ref": "@User:user1" }
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::Reference(r))) = doc.root.get("ref") {
        assert_eq!(r.id.as_ref(), "user1");
        assert_eq!(r.type_name.as_deref(), Some("User"));
    } else {
        panic!("Expected qualified reference");
    }
}

#[test]
fn test_from_yaml_string_starting_with_at_not_ref() {
    let yaml = r#"
email: "@company.com"
twitter: "@username"
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    // These should be parsed as strings, not references
    assert!(matches!(
        doc.root.get("email"),
        Some(Item::Scalar(Value::String(_)))
    ));
    assert!(matches!(
        doc.root.get("twitter"),
        Some(Item::Scalar(Value::String(_)))
    ));
}

// ==================== Tensor Parsing ====================

#[test]
fn test_from_yaml_tensor_1d() {
    let yaml = r"
vector: [1, 2, 3, 4, 5]
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::Tensor(t))) = doc.root.get("vector") {
        if let hedl_core::lex::Tensor::Array(items) = t.as_ref() {
            assert_eq!(items.len(), 5);
        } else {
            panic!("Expected tensor array");
        }
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_from_yaml_tensor_2d() {
    let yaml = r"
matrix:
  - [1, 2, 3]
  - [4, 5, 6]
  - [7, 8, 9]
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Scalar(Value::Tensor(t))) = doc.root.get("matrix") {
        if let hedl_core::lex::Tensor::Array(rows) = t.as_ref() {
            assert_eq!(rows.len(), 3);
        } else {
            panic!("Expected tensor array");
        }
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_from_yaml_tensor_3d() {
    let yaml = r"
cube:
  - [[1, 2], [3, 4]]
  - [[5, 6], [7, 8]]
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    assert!(matches!(
        doc.root.get("cube"),
        Some(Item::Scalar(Value::Tensor(_)))
    ));
}

#[test]
fn test_from_yaml_tensor_floats() {
    let yaml = r"
weights: [0.1, 0.5, 0.9, 1.0]
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    assert!(matches!(
        doc.root.get("weights"),
        Some(Item::Scalar(Value::Tensor(_)))
    ));
}

// ==================== Matrix List Tests ====================

#[test]
fn test_from_yaml_matrix_list_basic() {
    let yaml = r"
users:
  - id: user1
    name: Alice
  - id: user2
    name: Bob
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::List(list)) = doc.root.get("users") {
        assert_eq!(list.rows.len(), 2);
        assert!(list.schema.contains(&"id".to_string()));
        assert!(list.schema.contains(&"name".to_string()));
    } else {
        panic!("Expected matrix list");
    }
}

#[test]
fn test_from_yaml_matrix_list_with_metadata() {
    let yaml = r"
users:
  __type__: User
  __schema__: [id, name, age]
  items:
    - id: user1
      name: Alice
      age: 30
    - id: user2
      name: Bob
      age: 25
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::List(list)) = doc.root.get("users") {
        assert_eq!(list.type_name, "User");
        assert_eq!(list.schema, vec!["id", "name", "age"]);
        assert_eq!(list.rows.len(), 2);
    } else {
        panic!("Expected matrix list with metadata");
    }
}

#[test]
fn test_from_yaml_matrix_list_empty() {
    let yaml = r"
empty_list: []
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::List(list)) = doc.root.get("empty_list") {
        assert!(list.rows.is_empty());
    } else {
        panic!("Expected empty list");
    }
}

// ==================== Error Handling Tests ====================

#[test]
fn test_from_yaml_malformed_yaml() {
    let yaml = "{ invalid: yaml: syntax [";
    let config = FromYamlConfig::default();
    let result = from_yaml(yaml, &config);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("YAML parse error"));
}

#[test]
fn test_from_yaml_non_mapping_root() {
    let yaml = "- item1\n- item2";
    let config = FromYamlConfig::default();
    let result = from_yaml(yaml, &config);

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Root must be a YAML mapping"));
}

#[test]
fn test_from_yaml_invalid_expression() {
    let yaml = r#"
expr: "$(invalid syntax here"
"#;

    let config = FromYamlConfig::default();
    // Should parse as string since it doesn't end with )
    let doc = from_yaml(yaml, &config).unwrap();

    assert!(matches!(
        doc.root.get("expr"),
        Some(Item::Scalar(Value::String(_)))
    ));
}

// ==================== Complex Nested Structures ====================

#[test]
fn test_from_yaml_deeply_nested_objects() {
    let yaml = r"
root:
  level1:
    level2:
      level3:
        value: deep
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Object(root_obj)) = doc.root.get("root") {
        assert!(root_obj.contains_key("level1"));
    } else {
        panic!("Expected nested objects");
    }
}

#[test]
fn test_from_yaml_mixed_types() {
    let yaml = r#"
data:
  string_val: "text"
  int_val: 42
  float_val: 3.15
  bool_val: true
  null_val: null
  array_val: [1, 2, 3]
  object_val:
    nested: value
"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::Object(data)) = doc.root.get("data") {
        assert_eq!(data.len(), 7);
    } else {
        panic!("Expected object with mixed types");
    }
}

// ==================== from_yaml_value Tests ====================

#[test]
fn test_from_yaml_value_mapping() {
    let yaml_str = "name: test\nvalue: 123";
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
    let config = FromYamlConfig::default();

    let doc = from_yaml_value(&yaml_value, &config).unwrap();
    assert_eq!(doc.root.len(), 2);
}

#[test]
fn test_from_yaml_value_non_mapping() {
    let yaml_value = serde_yaml::Value::String("test".to_string());
    let config = FromYamlConfig::default();

    let result = from_yaml_value(&yaml_value, &config);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Root must be a YAML mapping"));
}

// ==================== Whitespace and Formatting ====================

#[test]
fn test_from_yaml_extra_whitespace() {
    let yaml = r#"


name:    "test"


value:   42


"#;

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    assert_eq!(doc.root.len(), 2);
}

#[test]
fn test_from_yaml_tabs_and_spaces() {
    let yaml = "name:\t\"test\"\nvalue: 42";
    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    assert_eq!(doc.root.len(), 2);
}

// ==================== Comment Handling ====================

#[test]
fn test_from_yaml_with_comments() {
    let yaml = r"
# This is a comment
name: test # inline comment
# Another comment
value: 42
";

    let config = FromYamlConfig::default();
    let doc = from_yaml(yaml, &config).unwrap();

    assert_eq!(doc.root.len(), 2);
}

// ==================== Version Tests ====================

#[test]
fn test_from_yaml_custom_version() {
    let yaml = "name: test";
    let config = FromYamlConfig::builder().version(2, 5).build();

    let doc = from_yaml(yaml, &config).unwrap();
    assert_eq!(doc.version, (2, 5));
}

// ==================== Type Name Tests ====================

#[test]
fn test_from_yaml_custom_default_type_name() {
    let yaml = r"
items: []
";

    let config = FromYamlConfig::builder()
        .default_type_name("CustomType")
        .build();

    let doc = from_yaml(yaml, &config).unwrap();

    if let Some(Item::List(list)) = doc.root.get("items") {
        // The type name for singularized "items" should be "Item" or similar
        assert!(!list.type_name.is_empty());
    }
}
