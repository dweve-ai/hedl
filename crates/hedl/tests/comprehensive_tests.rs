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

//! Comprehensive tests for hedl facade crate.
//!
//! Tests all re-exported types, convenience functions, and modules including:
//! - Core parsing and types
//! - Canonicalization
//! - JSON conversion
//! - Linting
//! - Lexer utilities
//! - CSV row parsing
//! - Tensor parsing
//! - Feature-gated format converters

use hedl::{
    canonicalize,
    from_json,
    lint,
    // Convenience functions
    parse,
    parse_lenient,
    to_json,
    validate,
    // Core types
    Document,
    HedlError,
    ParseOptions,
    Reference,
    Tensor,
    Value,
    // Constants
    SUPPORTED_VERSION,
    VERSION,
};

// =============================================================================
// Constants Tests
// =============================================================================

#[test]
fn test_supported_version() {
    assert_eq!(SUPPORTED_VERSION, (1, 0));
}

#[test]
fn test_library_version() {
    assert!(!VERSION.is_empty());
    // Should match semver pattern
    let parts: Vec<&str> = VERSION.split('.').collect();
    assert!(parts.len() >= 2);
}

// =============================================================================
// parse() Tests
// =============================================================================

#[test]
fn test_parse_minimal_document() {
    let doc = parse("%VERSION: 1.0\n---\n").unwrap();
    assert_eq!(doc.version, (1, 0));
    assert!(doc.root.is_empty());
}

#[test]
fn test_parse_with_key_value() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    assert_eq!(doc.version, (1, 0));
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_parse_multiple_items() {
    let input = r#"%VERSION: 1.0
---
name: Alice
age: 30
active: true
"#;
    let doc = parse(input).unwrap();
    assert_eq!(doc.root.len(), 3);
}

#[test]
fn test_parse_nested_node() {
    let input = r#"%VERSION: 1.0
---
user:
  name: Alice
  age: 30
"#;
    let doc = parse(input).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_parse_struct_definition() {
    let input = r#"%VERSION: 1.0
%STRUCT: User: [id,name,email]
---
"#;
    let doc = parse(input).unwrap();
    assert!(doc.structs.contains_key("User"));
    let user_struct = &doc.structs["User"];
    assert_eq!(user_struct.len(), 3);
}

#[test]
fn test_parse_alias_definition() {
    let input = r#"%VERSION: 1.0
%ALIAS: %rate: "1.23456"
---
circle_const: %rate
"#;
    let doc = parse(input).unwrap();
    assert!(doc.aliases.contains_key("rate"));
}

#[test]
fn test_parse_matrix_list() {
    let input = r#"%VERSION: 1.0
%STRUCT: Point: [id,x,y]
---
points: @Point
  | p1, 1, 2
  | p2, 3, 4
  | p3, 5, 6
"#;
    let doc = parse(input).unwrap();
    assert!(doc.structs.contains_key("Point"));
}

#[test]
fn test_parse_reference() {
    let input = r#"%VERSION: 1.0
%STRUCT: User: [id,name]
---
users: @User
  | alice, Alice
friend: @alice
"#;
    let doc = parse(input).unwrap();
    // Reference should be resolved in strict mode
    assert_eq!(doc.root.len(), 2);
}

#[test]
fn test_parse_tensor() {
    let input = r#"%VERSION: 1.0
---
vector: [1, 2, 3]
matrix: [[1, 2], [3, 4]]
"#;
    let doc = parse(input).unwrap();
    assert_eq!(doc.root.len(), 2);
}

#[test]
fn test_parse_expression() {
    let input = r#"%VERSION: 1.0
---
formula: $(multiply(add(a, b), c))
"#;
    let doc = parse(input).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_parse_ditto_operator() {
    let input = r#"%VERSION: 1.0
%STRUCT: Product: [id,category,name,price]
---
products: @Product
  | p1, electronics, Phone, 999
  | p2, ^, Laptop, 1499
  | p3, ^, Tablet, 599
"#;
    let doc = parse(input).unwrap();
    assert!(doc.structs.contains_key("Product"));
}

#[test]
fn test_parse_error_no_version() {
    let result = parse("---\nkey: value");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_no_separator() {
    let result = parse("%VERSION: 1.0\nkey: value");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_invalid_syntax() {
    let result = parse("%VERSION: 1.0\n---\n[invalid");
    assert!(result.is_err());
}

// =============================================================================
// parse_lenient() Tests
// =============================================================================

#[test]
fn test_parse_lenient_minimal() {
    let doc = parse_lenient("%VERSION: 1.0\n---\n").unwrap();
    assert_eq!(doc.version, (1, 0));
}

#[test]
fn test_parse_lenient_unresolved_reference() {
    // In lenient mode, unresolved references become null
    let input = r#"%VERSION: 1.0
---
ref: @nonexistent
"#;
    let result = parse_lenient(input);
    // Should succeed in lenient mode
    assert!(result.is_ok());
}

#[test]
fn test_parse_lenient_vs_strict() {
    let input = r#"%VERSION: 1.0
---
ref: @nonexistent
"#;
    // Strict mode should fail
    let strict_result = parse(input);
    assert!(strict_result.is_err());

    // Lenient mode should succeed
    let lenient_result = parse_lenient(input);
    assert!(lenient_result.is_ok());
}

// =============================================================================
// validate() Tests
// =============================================================================

#[test]
fn test_validate_valid_document() {
    assert!(validate("%VERSION: 1.0\n---\n").is_ok());
    assert!(validate("%VERSION: 1.0\n---\nkey: value").is_ok());
}

#[test]
fn test_validate_invalid_document() {
    assert!(validate("invalid").is_err());
    assert!(validate("").is_err());
    assert!(validate("%VERSION: 1.0").is_err()); // Missing separator
}

// =============================================================================
// canonicalize() Tests
// =============================================================================

#[test]
fn test_canonicalize_basic() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let canonical = canonicalize(&doc).unwrap();
    assert!(canonical.contains("%VERSION: 1.0"));
    assert!(canonical.contains("key:"));
}

#[test]
fn test_canonicalize_sorts_keys() {
    let doc = parse("%VERSION: 1.0\n---\nz: 3\na: 1\nm: 2").unwrap();
    let canonical = canonicalize(&doc).unwrap();
    let a_pos = canonical.find("a:").unwrap();
    let m_pos = canonical.find("m:").unwrap();
    let z_pos = canonical.find("z:").unwrap();
    assert!(a_pos < m_pos);
    assert!(m_pos < z_pos);
}

#[test]
fn test_canonicalize_deterministic() {
    let input = "%VERSION: 1.0\n---\nb: 2\na: 1\nc: 3";
    let doc = parse(input).unwrap();
    let canonical1 = canonicalize(&doc).unwrap();
    let canonical2 = canonicalize(&doc).unwrap();
    assert_eq!(canonical1, canonical2);
}

#[test]
fn test_canonicalize_nested() {
    let input = r#"%VERSION: 1.0
---
outer:
  z: 3
  a: 1
"#;
    let doc = parse(input).unwrap();
    let canonical = canonicalize(&doc).unwrap();
    // Nested keys should also be sorted
    assert!(canonical.contains("a:"));
    assert!(canonical.contains("z:"));
}

// =============================================================================
// to_json() Tests
// =============================================================================

#[test]
fn test_to_json_basic() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"key\""));
    assert!(json.contains("\"value\""));
}

#[test]
fn test_to_json_number() {
    let doc = parse("%VERSION: 1.0\n---\nnum: 42").unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("42"));
}

#[test]
fn test_to_json_boolean() {
    let doc = parse("%VERSION: 1.0\n---\nactive: true\ndisabled: false").unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("true"));
    assert!(json.contains("false"));
}

#[test]
fn test_to_json_null() {
    let doc = parse("%VERSION: 1.0\n---\nvalue: null").unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("null"));
}

#[test]
fn test_to_json_nested() {
    let input = r#"%VERSION: 1.0
---
user:
  name: Alice
  age: 30
"#;
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"user\""));
    assert!(json.contains("\"name\""));
    assert!(json.contains("\"Alice\""));
}

#[test]
fn test_to_json_array() {
    let input = r#"%VERSION: 1.0
%STRUCT: Item: [value]
---
items: @Item
  | one
  | two
  | three
"#;
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"items\""));
}

// =============================================================================
// from_json() Tests
// =============================================================================

#[test]
fn test_from_json_basic() {
    let json = r#"{"key": "value"}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.version, (1, 0));
}

#[test]
fn test_from_json_number() {
    let json = r#"{"num": 42}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_from_json_boolean() {
    let json = r#"{"active": true}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_from_json_null() {
    let json = r#"{"value": null}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_from_json_nested() {
    let json = r#"{"user": {"name": "Alice", "age": 30}}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_from_json_array() {
    let json = r#"{"items": [1, 2, 3]}"#;
    let doc = from_json(json).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_from_json_error() {
    let json = "not valid json";
    let result = from_json(json);
    assert!(result.is_err());
}

// =============================================================================
// JSON Round-Trip Tests
// =============================================================================

#[test]
fn test_json_round_trip_basic() {
    let input = "%VERSION: 1.0\n---\nkey: value";
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    let doc2 = from_json(&json).unwrap();
    let json2 = to_json(&doc2).unwrap();
    // JSON should be equivalent (content may differ in formatting)
    assert!(json2.contains("\"key\""));
    assert!(json2.contains("\"value\""));
}

#[test]
fn test_json_round_trip_numbers() {
    let input = "%VERSION: 1.0\n---\nint: 42\nfloat: 3.25";
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    let doc2 = from_json(&json).unwrap();
    let json2 = to_json(&doc2).unwrap();
    assert!(json2.contains("42"));
    assert!(json2.contains("3.25"));
}

// =============================================================================
// lint() Tests
// =============================================================================

#[test]
fn test_lint_clean_document() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let diagnostics = lint(&doc);
    // Clean document may have no warnings
    // (depends on lint rules)
    let _ = diagnostics;
}

#[test]
fn test_lint_returns_diagnostics() {
    let input = r#"%VERSION: 1.0
%ALIAS: %unused_alias: "not_used"
---
key: value
"#;
    let doc = parse(input).unwrap();
    let diagnostics = lint(&doc);
    // May or may not have warnings depending on lint rules
    // At minimum, we verify it runs without error
    let _ = diagnostics;
}

#[test]
fn test_lint_diagnostic_display() {
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let diagnostics = lint(&doc);
    for d in diagnostics {
        // Verify Display is implemented
        let _s = format!("{}", d);
    }
}

// =============================================================================
// Lex Module Tests
// =============================================================================

#[test]
fn test_lex_is_valid_key_token() {
    use hedl::lex::is_valid_key_token;

    assert!(is_valid_key_token("key"));
    assert!(is_valid_key_token("my_key"));
    assert!(is_valid_key_token("key123"));
    assert!(!is_valid_key_token(""));
    assert!(!is_valid_key_token("123key"));
}

#[test]
fn test_lex_is_valid_type_name() {
    use hedl::lex::is_valid_type_name;

    assert!(is_valid_type_name("User"));
    assert!(is_valid_type_name("MyType"));
    assert!(!is_valid_type_name(""));
    assert!(!is_valid_type_name("user")); // Must start with uppercase
}

#[test]
fn test_lex_is_valid_id_token() {
    use hedl::lex::is_valid_id_token;

    assert!(is_valid_id_token("alice"));
    assert!(is_valid_id_token("user_123"));
    assert!(!is_valid_id_token(""));
}

#[test]
fn test_lex_parse_reference() {
    use hedl::lex::parse_reference;

    let ref1 = parse_reference("@alice").unwrap();
    assert_eq!(ref1.id, "alice");
    assert!(ref1.type_name.is_none());

    let ref2 = parse_reference("@User:alice").unwrap();
    assert_eq!(ref2.id, "alice");
    assert_eq!(ref2.type_name.as_deref(), Some("User"));
}

#[test]
fn test_lex_strip_comment() {
    use hedl::lex::strip_comment;

    // strip_comment removes the # and everything after it
    assert_eq!(strip_comment("key: value # comment"), "key: value");
    assert_eq!(strip_comment("no comment"), "no comment");
    assert_eq!(strip_comment("# full comment"), "");
}

#[test]
fn test_lex_indent_info() {
    use hedl::lex::IndentInfo;

    // Just verify the type is accessible
    let info = IndentInfo {
        level: 1,
        spaces: 2,
    };
    assert_eq!(info.level, 1);
    assert_eq!(info.spaces, 2);
}

// =============================================================================
// CSV Module Tests
// =============================================================================

#[test]
fn test_csv_parse_row_basic() {
    use hedl::csv::parse_csv_row;

    let fields = parse_csv_row("a, b, c").unwrap();
    assert_eq!(fields.len(), 3);
}

#[test]
fn test_csv_parse_row_quoted() {
    use hedl::csv::parse_csv_row;

    let fields = parse_csv_row(r#"a, "b, c", d"#).unwrap();
    assert_eq!(fields.len(), 3);
}

#[test]
fn test_csv_parse_row_numbers() {
    use hedl::csv::parse_csv_row;

    let fields = parse_csv_row("1, 2.5, 3").unwrap();
    assert_eq!(fields.len(), 3);
}

// =============================================================================
// Tensor Module Tests
// =============================================================================

#[test]
fn test_tensor_parse_1d() {
    use hedl::tensor::parse_tensor;

    let t = parse_tensor("[1, 2, 3]").unwrap();
    assert_eq!(t.shape(), vec![3]);
    assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0]);
}

#[test]
fn test_tensor_parse_2d() {
    use hedl::tensor::parse_tensor;

    let t = parse_tensor("[[1, 2], [3, 4]]").unwrap();
    assert_eq!(t.shape(), vec![2, 2]);
}

#[test]
fn test_tensor_parse_floats() {
    use hedl::tensor::parse_tensor;

    let t = parse_tensor("[1.5, 2.5, 3.5]").unwrap();
    assert_eq!(t.flatten(), vec![1.5, 2.5, 3.5]);
    assert!(!t.is_integer());
}

#[test]
fn test_tensor_parse_error() {
    use hedl::tensor::parse_tensor;

    let result = parse_tensor("[]");
    assert!(result.is_err());
}

// =============================================================================
// C14n Module Tests
// =============================================================================

#[test]
fn test_c14n_module_canonicalize() {
    use hedl::c14n::canonicalize;

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let canonical = canonicalize(&doc).unwrap();
    assert!(canonical.contains("%VERSION: 1.0"));
}

#[test]
fn test_c14n_module_with_config() {
    use hedl::c14n::{canonicalize_with_config, CanonicalConfig};

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let config = CanonicalConfig::default();
    let canonical = canonicalize_with_config(&doc, &config).unwrap();
    assert!(canonical.contains("key:"));
}

#[test]
fn test_c14n_quoting_strategy() {
    use hedl::c14n::QuotingStrategy;

    // Just verify the enum exists and can be used
    let _strategy = QuotingStrategy::Minimal;
    let _strategy = QuotingStrategy::Always;
}

// =============================================================================
// JSON Module Tests
// =============================================================================

#[test]
fn test_json_module_to_json() {
    use hedl::json::to_json;

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let config = hedl::json::ToJsonConfig::default();
    let json = to_json(&doc, &config).unwrap();
    assert!(json.contains("\"key\""));
}

#[test]
fn test_json_module_from_json() {
    use hedl::json::from_json;

    let json = r#"{"key": "value"}"#;
    let config = hedl::json::FromJsonConfig::default();
    let doc = from_json(json, &config).unwrap();
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_json_module_to_json_value() {
    use hedl::json::to_json_value;

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let config = hedl::json::ToJsonConfig::default();
    let value = to_json_value(&doc, &config).unwrap();
    assert!(value.is_object());
}

#[test]
fn test_json_module_from_json_value() {
    use hedl::json::from_json_value;

    let value = serde_json::json!({"key": "value"});
    let config = hedl::json::FromJsonConfig::default();
    let doc = from_json_value(&value, &config).unwrap();
    assert_eq!(doc.root.len(), 1);
}

// =============================================================================
// Lint Module Tests
// =============================================================================

#[test]
fn test_lint_module_lint() {
    use hedl::lint::lint;

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let diagnostics = lint(&doc);
    let _ = diagnostics;
}

#[test]
fn test_lint_module_with_config() {
    use hedl::lint::{lint_with_config, LintConfig};

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let config = LintConfig::default();
    let diagnostics = lint_with_config(&doc, config);
    let _ = diagnostics;
}

#[test]
fn test_lint_severity_enum() {
    use hedl::lint::Severity;

    let _hint = Severity::Hint;
    let _warning = Severity::Warning;
    let _error = Severity::Error;
}

// =============================================================================
// Type Re-export Tests
// =============================================================================

#[test]
fn test_document_type() {
    let doc = Document::new((1, 0));
    assert_eq!(doc.version, (1, 0));
    assert!(doc.root.is_empty());
    assert!(doc.aliases.is_empty());
    assert!(doc.structs.is_empty());
}

#[test]
fn test_value_enum() {
    let _null = Value::Null;
    let _bool = Value::Bool(true);
    let _int = Value::Int(42);
    let _float = Value::Float(3.25);
    let _str = Value::String("test".to_string());
}

#[test]
fn test_tensor_type_direct() {
    let t = Tensor::Scalar(42.0);
    assert_eq!(t.flatten(), vec![42.0]);

    let t2 = Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]);
    assert_eq!(t2.shape(), vec![2]);
}

#[test]
fn test_reference_type() {
    let r = Reference {
        type_name: Some("User".to_string()),
        id: "alice".to_string(),
    };
    assert_eq!(r.type_name.as_deref(), Some("User"));
    assert_eq!(r.id, "alice");
}

#[test]
fn test_hedl_error_type() {
    let err = HedlError::syntax("test error".to_string(), 1);
    let _msg = format!("{}", err);
}

#[test]
fn test_parse_options_type() {
    let options = ParseOptions {
        strict_refs: true,
        ..Default::default()
    };
    assert!(options.strict_refs);
}

// =============================================================================
// Feature-Gated Module Tests (YAML)
// =============================================================================

#[cfg(feature = "yaml")]
mod yaml_tests {
    use super::*;

    #[test]
    fn test_yaml_to_yaml() {
        use hedl::yaml::to_yaml;

        let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
        let config = hedl::yaml::ToYamlConfig::default();
        let yaml = to_yaml(&doc, &config).unwrap();
        assert!(yaml.contains("key:"));
    }

    #[test]
    fn test_yaml_from_yaml() {
        use hedl::yaml::from_yaml;

        let yaml = "key: value";
        let config = hedl::yaml::FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();
        assert_eq!(doc.root.len(), 1);
    }
}

// =============================================================================
// Feature-Gated Module Tests (XML)
// =============================================================================

#[cfg(feature = "xml")]
mod xml_tests {
    use super::*;

    #[test]
    fn test_xml_to_xml() {
        use hedl::xml::to_xml;

        let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
        let config = hedl::xml::ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<key>"));
    }

    #[test]
    fn test_xml_from_xml() {
        use hedl::xml::from_xml;

        let xml = "<root><key>value</key></root>";
        let config = hedl::xml::FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert!(!doc.root.is_empty());
    }
}

// =============================================================================
// Feature-Gated Module Tests (CSV)
// =============================================================================

#[cfg(feature = "csv")]
mod csv_file_tests {
    use super::*;

    #[test]
    fn test_csv_to_csv() {
        use hedl::csv_file::to_csv;
        use hedl_core::{Item, MatrixList, Node};

        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new(
            "Row",
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        list.add_row(Node::new(
            "Row",
            "1",
            vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        ));
        list.add_row(Node::new(
            "Row",
            "2",
            vec![Value::Int(4), Value::Int(5), Value::Int(6)],
        ));
        doc.root.insert("data".to_string(), Item::List(list));

        let csv = to_csv(&doc).unwrap();
        assert!(csv.contains("a,b,c"));
    }

    #[test]
    fn test_csv_from_csv() {
        use hedl::csv_file::from_csv;

        let csv = "id,a,b,c\n1,2,3,4\n2,5,6,7";
        let doc = from_csv(csv, "Row", &["a", "b", "c"]).unwrap();
        assert!(!doc.root.is_empty());
    }
}

// =============================================================================
// Feature-Gated Module Tests (Parquet)
// =============================================================================

#[cfg(feature = "parquet")]
mod parquet_tests {
    use super::*;
    use hedl_core::{Item, MatrixList, Node};

    #[test]
    fn test_parquet_round_trip() {
        use hedl::parquet::{from_parquet_bytes, to_parquet_bytes};

        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new(
            "Row",
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        list.add_row(Node::new(
            "Row",
            "1",
            vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        ));
        list.add_row(Node::new(
            "Row",
            "2",
            vec![Value::Int(4), Value::Int(5), Value::Int(6)],
        ));
        doc.root.insert("data".to_string(), Item::List(list));

        let bytes = to_parquet_bytes(&doc).unwrap();
        let doc2 = from_parquet_bytes(&bytes).unwrap();
        assert!(!doc2.root.is_empty());
    }
}

// =============================================================================
// Complex Document Tests
// =============================================================================

#[test]
fn test_complex_document_full_pipeline() {
    let input = r#"%VERSION: 1.0
%STRUCT: User: [id,name,age,active]
%STRUCT: Team: [id,name]
%ALIAS: %default_age: "25"
---
teams: @Team
  | engineering, Engineering
  | design, Design

users: @User
  | alice, Alice, 30, true
  | bob, Bob, %default_age, true
  | carol, Carol, 28, false

metadata:
  created: 2024-01-01
  version: 1
"#;

    // Parse
    let doc = parse(input).unwrap();
    assert_eq!(doc.version, (1, 0));
    assert!(doc.structs.contains_key("User"));
    assert!(doc.structs.contains_key("Team"));
    assert!(doc.aliases.contains_key("default_age"));

    // Canonicalize
    let canonical = canonicalize(&doc).unwrap();
    assert!(canonical.contains("%VERSION: 1.0"));

    // Convert to JSON
    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"teams\""));
    assert!(json.contains("\"users\""));
    assert!(json.contains("\"metadata\""));

    // Lint
    let diagnostics = lint(&doc);
    for d in &diagnostics {
        let _msg = format!("{}", d);
    }

    // Validate
    assert!(validate(input).is_ok());
}

#[test]
fn test_document_with_all_value_types() {
    let input = r#"%VERSION: 1.0
---
string_val: "hello world"
int_val: 42
float_val: 1.23456
bool_true: true
bool_false: false
null_val: null
tensor_val: [1, 2, 3]
expr_val: $(add(x, y))
nested:
  key: value
"#;

    let doc = parse(input).unwrap();
    assert_eq!(doc.root.len(), 9);

    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"hello world\""));
    assert!(json.contains("42"));
    assert!(json.contains("1.23456"));
}

#[test]
fn test_deeply_nested_structure() {
    let input = r#"%VERSION: 1.0
---
level1:
  level2:
    level3:
      level4:
        value: deep
"#;

    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"deep\""));

    let canonical = canonicalize(&doc).unwrap();
    assert!(canonical.contains("value:"));
}

#[test]
fn test_multiple_matrix_lists() {
    let input = r#"%VERSION: 1.0
%STRUCT: Point2D: [id,x,y]
%STRUCT: Point3D: [id,x,y,z]
---
points2d: @Point2D
  | p1, 1, 2
  | p2, 3, 4

points3d: @Point3D
  | q1, 1, 2, 3
  | q2, 4, 5, 6
"#;

    let doc = parse(input).unwrap();
    assert!(doc.structs.contains_key("Point2D"));
    assert!(doc.structs.contains_key("Point3D"));

    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"points2d\""));
    assert!(json.contains("\"points3d\""));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_error_messages() {
    let err = parse("invalid document");
    assert!(err.is_err());
    let error = err.unwrap_err();
    let msg = format!("{}", error);
    assert!(!msg.is_empty());
}

#[test]
fn test_json_conversion_error() {
    let result = from_json("{invalid json}");
    assert!(result.is_err());
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_empty_string_value() {
    let input = "%VERSION: 1.0\n---\nempty: \"\"";
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"\""));
}

#[test]
fn test_unicode_values() {
    let input = "%VERSION: 1.0\n---\nunicode: \"日本語 emoji 🎉\"";
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("日本語"));
    assert!(json.contains("🎉"));
}

#[test]
fn test_large_numbers() {
    let input = "%VERSION: 1.0\n---\nbig: 9999999999999";
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("9999999999999"));
}

#[test]
fn test_negative_numbers() {
    let input = "%VERSION: 1.0\n---\nneg: -42\nneg_float: -3.25";
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    assert!(json.contains("-42"));
    assert!(json.contains("-3.25"));
}

#[test]
fn test_scientific_notation() {
    let input = "%VERSION: 1.0\n---\nsci: 1.5e10";
    let doc = parse(input).unwrap();
    let json = to_json(&doc).unwrap();
    // May be formatted differently but should contain the value
    assert!(!json.is_empty());
}

#[test]
fn test_whitespace_handling() {
    let input = "%VERSION: 1.0\n---\n  key  :   value  ";
    // Whitespace in key/value should be handled
    let result = parse(input);
    // May or may not succeed depending on parser strictness
    let _ = result;
}

#[test]
fn test_comments() {
    let input = r#"%VERSION: 1.0
# This is a comment
---
key: value # inline comment
"#;
    let doc = parse(input).unwrap();
    assert_eq!(doc.root.len(), 1);
}

// =============================================================================
// Thread Safety Tests (basic)
// =============================================================================

#[test]
fn test_parse_is_thread_safe() {
    use std::thread;

    let handles: Vec<_> = (0..4)
        .map(|_| {
            thread::spawn(|| {
                let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
                assert_eq!(doc.version, (1, 0));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_canonicalize_is_thread_safe() {
    use std::thread;

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();

    let handles: Vec<_> = (0..4)
        .map(|_| {
            let doc_clone = doc.clone();
            thread::spawn(move || {
                let canonical = canonicalize(&doc_clone).unwrap();
                assert!(canonical.contains("key:"));
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}
