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

//! Test module for WASM bindings.

// --- WASM Tests (require browser) ---

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use crate::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_parse_basic() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
  | alice, Alice Smith
"#;
        let result = parse(hedl);
        assert!(result.is_ok());

        let doc = result.unwrap();
        assert_eq!(doc.version(), "1.0");
        assert_eq!(doc.schema_count(), 1);
    }

    #[cfg(feature = "json")]
    #[wasm_bindgen_test]
    fn test_to_json() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: Item: [id, value]
---
items:@Item
  | a, 1
  | b, 2
"#;
        let json = to_json(hedl, Some(false));
        assert!(json.is_ok());
    }

    #[wasm_bindgen_test]
    fn test_validate_valid() {
        let hedl = r#"
%VERSION: 1.0
---
name: Test
"#;
        let result = validate(hedl, Some(false));
        assert!(!result.is_null());
    }
}

// --- Native Rust Tests (run with cargo test) ---

#[cfg(test)]
mod native_tests {
    use hedl_c14n::CanonicalConfig;
    use hedl_core::parse as core_parse;

    #[cfg(feature = "query-api")]
    use hedl_core::{Item, Value};

    #[cfg(feature = "json")]
    use hedl_json::{to_json_value, ToJsonConfig};

    #[cfg(feature = "full-validation")]
    use hedl_lint::lint;

    use crate::document::*;
    #[cfg(any(feature = "statistics", feature = "token-tools"))]
    use crate::stats::*;
    use crate::validation::*;

    // ============ TOKEN ESTIMATION TESTS ============

    #[test]
    #[cfg(any(feature = "statistics", feature = "token-tools"))]
    fn test_estimate_tokens_empty() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    #[cfg(any(feature = "statistics", feature = "token-tools"))]
    fn test_estimate_tokens_simple() {
        // Rough approximation: ~4 chars per token
        let tokens = estimate_tokens("hello world");
        assert!(tokens > 0, "Should estimate some tokens");
        assert!(tokens < 10, "Should not over-estimate");
    }

    #[test]
    #[cfg(any(feature = "statistics", feature = "token-tools"))]
    fn test_estimate_tokens_punctuation() {
        // Punctuation counts extra
        let tokens_plain = estimate_tokens("hello world");
        let tokens_punct = estimate_tokens("hello, world!");
        assert!(
            tokens_punct >= tokens_plain,
            "Punctuation should add tokens"
        );
    }

    #[test]
    #[cfg(any(feature = "statistics", feature = "token-tools"))]
    fn test_estimate_tokens_whitespace() {
        // Whitespace counts extra
        let tokens_compact = estimate_tokens("abc");
        let tokens_spaced = estimate_tokens("a b c");
        assert!(
            tokens_spaced > tokens_compact,
            "Whitespace should add tokens"
        );
    }

    // ============ VALUE TO JSON TESTS ============

    #[cfg(feature = "query-api")]
    #[test]
    fn test_value_to_json_null() {
        let json = value_to_json(&Value::Null);
        assert!(json.is_null());
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_value_to_json_bool() {
        let json_true = value_to_json(&Value::Bool(true));
        assert_eq!(json_true, serde_json::Value::Bool(true));

        let json_false = value_to_json(&Value::Bool(false));
        assert_eq!(json_false, serde_json::Value::Bool(false));
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_value_to_json_int() {
        let json = value_to_json(&Value::Int(42));
        assert_eq!(json, serde_json::json!(42));
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_value_to_json_float() {
        let json = value_to_json(&Value::Float(3.5));
        assert_eq!(json, serde_json::json!(3.5));
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_value_to_json_string() {
        let json = value_to_json(&Value::String("hello".into()));
        assert_eq!(json, serde_json::Value::String("hello".to_string()));
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_value_to_json_reference_qualified() {
        let reference = hedl_core::Reference {
            type_name: Some("User".into()),
            id: "alice".into(),
        };
        let json = value_to_json(&Value::Reference(reference));
        assert_eq!(json, serde_json::json!("@User:alice"));
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_value_to_json_reference_unqualified() {
        let reference = hedl_core::Reference {
            type_name: None,
            id: "alice".into(),
        };
        let json = value_to_json(&Value::Reference(reference));
        assert_eq!(json, serde_json::json!("@alice"));
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_value_to_json_expression() {
        // Test expression conversion via parsing a complete HEDL document
        let hedl = "%VERSION: 1.0\n---\nx: $(now())\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();

        // Find the expression value in the parsed document
        if let Some(Item::Object(obj)) = doc.root.get("x") {
            // The expression parsing will be handled by core
            assert!(!obj.is_empty(), "Expression should parse successfully");
        } else if let Some(Item::Scalar(v)) = doc.root.get("x") {
            // Check if it's an expression value
            match v {
                Value::Expression(_) => {
                    let json = value_to_json(v);
                    assert!(json.is_string(), "Expression should serialize to string");
                    let s = json.as_str().unwrap();
                    assert!(s.starts_with("$("), "Expression should start with $(");
                    assert!(s.ends_with(')'), "Expression should end with )");
                }
                _ => panic!("Expected expression value"),
            }
        }
    }

    // ============ NODE FIELDS TO JSON TESTS ============

    #[cfg(feature = "query-api")]
    #[test]
    fn test_node_fields_to_json_with_schema() {
        let fields = vec![
            Value::String("alice".into()),
            Value::String("Alice Smith".into()),
        ];
        let schema = vec!["id".to_string(), "name".to_string()];

        let json = node_fields_to_json(&fields, &schema);
        assert!(json.is_object());
        let obj = json.as_object().unwrap();
        assert_eq!(obj.get("id"), Some(&serde_json::json!("alice")));
        assert_eq!(obj.get("name"), Some(&serde_json::json!("Alice Smith")));
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_node_fields_to_json_extra_fields() {
        // More fields than schema columns - uses field_N naming
        let fields = vec![
            Value::String("a".into()),
            Value::String("b".into()),
            Value::String("c".into()),
        ];
        let schema = vec!["id".to_string()];

        let json = node_fields_to_json(&fields, &schema);
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("field_1"));
        assert!(obj.contains_key("field_2"));
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_node_fields_to_json_empty() {
        let fields: Vec<Value> = vec![];
        let schema: Vec<String> = vec![];

        let json = node_fields_to_json(&fields, &schema);
        assert!(json.is_object());
        assert!(json.as_object().unwrap().is_empty());
    }

    // ============ PARSING TESTS ============

    #[test]
    fn test_parse_valid_document() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers:@User\n | alice, Alice\n";
        let doc = core_parse(hedl.as_bytes());
        assert!(doc.is_ok(), "Should parse valid HEDL");

        let doc = doc.unwrap();
        // Parsing v1.0 content preserves the version
        assert_eq!(doc.version, (1, 0));
        assert!(doc.structs.contains_key("User"));
    }

    #[test]
    fn test_parse_invalid_document() {
        let hedl = "invalid content without version";
        let doc = core_parse(hedl.as_bytes());
        assert!(doc.is_err(), "Should fail to parse invalid HEDL");
    }

    #[test]
    fn test_parse_empty_body() {
        let hedl = "%VERSION: 1.0\n---\n";
        let doc = core_parse(hedl.as_bytes());
        assert!(doc.is_ok(), "Should parse document with empty body");
    }

    #[test]
    fn test_parse_with_aliases() {
        let hedl = "%VERSION: 1.0\n%ALIAS: %active: \"true\"\n---\n";
        let doc = core_parse(hedl.as_bytes());
        assert!(doc.is_ok(), "Should parse document with aliases");
    }

    #[test]
    fn test_parse_with_nests() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n%STRUCT: Post: [id, title]\n%NEST: User > Post\n---\n";
        let doc = core_parse(hedl.as_bytes());
        assert!(doc.is_ok(), "Should parse document with nests");

        let doc = doc.unwrap();
        assert!(doc.nests.contains_key("User"), "Should have User nest");
    }

    // ============ JSON CONVERSION TESTS ============

    #[cfg(feature = "json")]
    #[test]
    fn test_to_json_value_basic() {
        let hedl = "%VERSION: 1.0\n---\nname: Test\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();
        let config = ToJsonConfig::default();

        let json = to_json_value(&doc, &config);
        assert!(json.is_ok(), "Should convert to JSON");
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_to_json_value_with_entities() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers:@User\n | alice, Alice\n | bob, Bob\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();
        let config = ToJsonConfig::default();

        let json = to_json_value(&doc, &config);
        assert!(json.is_ok(), "Should convert entities to JSON");
    }

    // ============ LINTING TESTS ============

    #[cfg(feature = "full-validation")]
    #[test]
    fn test_lint_valid_document() {
        let hedl = "%VERSION: 1.0\n---\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();
        let diagnostics = lint(&doc);
        // Valid document may still have hints/warnings
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.severity(), hedl_lint::Severity::Error))
            .collect();
        assert!(
            errors.is_empty(),
            "Should have no errors for valid document"
        );
    }

    // ============ ENTITY COUNTING TESTS ============

    #[test]
    fn test_count_item_entities_list() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers:@User\n | alice, Alice\n | bob, Bob\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();

        let mut counts = std::collections::BTreeMap::new();
        for item in doc.root.values() {
            count_item_entities(item, &mut counts);
        }

        assert_eq!(counts.get("User"), Some(&2), "Should count 2 User entities");
    }

    #[test]
    fn test_count_item_entities_nested() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id]\n%STRUCT: Post: [id]\n%NEST: User > Post\n---\nusers:@User\n | alice\n  | post1\n  | post2\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();

        let mut counts = std::collections::BTreeMap::new();
        for item in doc.root.values() {
            count_item_entities(item, &mut counts);
        }

        assert_eq!(counts.get("User"), Some(&1), "Should count 1 User");
        assert_eq!(counts.get("Post"), Some(&2), "Should count 2 Posts");
    }

    // ============ ENTITY FINDING TESTS ============

    #[cfg(feature = "query-api")]
    #[test]
    fn test_find_entities_all() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers:@User\n | alice, Alice\n | bob, Bob\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();

        let mut results = Vec::new();
        for item in doc.root.values() {
            find_entities(item, &None, &None, &mut results);
        }

        assert_eq!(results.len(), 2, "Should find 2 entities");
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_find_entities_by_type() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id]\n%STRUCT: Product: [id]\n---\nusers:@User\n | alice\nproducts:@Product\n | prod1\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();

        let mut results = Vec::new();
        let type_filter = Some("User".to_string());
        for item in doc.root.values() {
            find_entities(item, &type_filter, &None, &mut results);
        }

        assert_eq!(results.len(), 1, "Should find 1 User entity");
        assert_eq!(results[0].type_name, "User");
    }

    #[cfg(feature = "query-api")]
    #[test]
    fn test_find_entities_by_id() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers:@User\n | alice, Alice\n | bob, Bob\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();

        let mut results = Vec::new();
        let id_filter = Some("alice".to_string());
        for item in doc.root.values() {
            find_entities(item, &None, &id_filter, &mut results);
        }

        assert_eq!(results.len(), 1, "Should find 1 entity with id alice");
        assert_eq!(results[0].id, "alice");
    }

    // ============ STATS CALCULATION TESTS ============

    #[test]
    fn test_stats_savings_calculation() {
        // Test the savings calculation logic directly
        let hedl_tokens = 100usize;
        let json_tokens = 400usize;

        let savings_percent = if json_tokens > 0 {
            ((json_tokens as i64 - hedl_tokens as i64) * 100 / json_tokens as i64) as i32
        } else {
            0
        };

        assert_eq!(savings_percent, 75, "Should show 75% savings");
    }

    #[test]
    fn test_stats_negative_savings() {
        // When HEDL is larger than JSON (edge case)
        let hedl_tokens = 500usize;
        let json_tokens = 400usize;

        let savings_percent = if json_tokens > 0 {
            ((json_tokens as i64 - hedl_tokens as i64) * 100 / json_tokens as i64) as i32
        } else {
            0
        };

        assert!(
            savings_percent < 0,
            "Should show negative savings when HEDL is larger"
        );
    }

    #[test]
    fn test_stats_zero_json_tokens() {
        let json_tokens = 0usize;

        let savings_percent = if json_tokens > 0 { 100i32 } else { 0 };

        assert_eq!(savings_percent, 0, "Should be 0 when JSON tokens is 0");
    }

    // ============ CANONICALIZATION TESTS ============

    #[test]
    fn test_canonicalize_document() {
        let hedl = "%VERSION: 1.0\n---\nz: 3\na: 1\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();
        let config = CanonicalConfig::default();

        let canonical = hedl_c14n::canonicalize_with_config(&doc, &config);
        assert!(canonical.is_ok(), "Should canonicalize document");

        let canonical = canonical.unwrap();
        assert!(
            canonical.contains("%VERSION: 1.0"),
            "Should contain version"
        );
    }

    #[test]
    fn test_canonicalize_with_ditto() {
        let hedl = "%VERSION: 1.0\n%STRUCT: T: [id, value]\n---\ndata:@T\n | a, x\n | b, x\n";
        let doc = core_parse(hedl.as_bytes()).unwrap();

        let config = CanonicalConfig::default(); // use_ditto is true by default

        let canonical = hedl_c14n::canonicalize_with_config(&doc, &config);
        assert!(canonical.is_ok(), "Should canonicalize with ditto enabled");
    }

    // ============ VALIDATION RESULT STRUCTURE TESTS ============

    #[test]
    fn test_validation_result_serialization() {
        let result = ValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec![ValidationWarning {
                line: 1,
                message: "Test warning".to_string(),
                rule: "test-rule".to_string(),
            }],
        };

        let json = serde_json::to_string(&result);
        assert!(json.is_ok(), "ValidationResult should serialize");

        let json = json.unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("Test warning"));
    }

    #[test]
    fn test_validation_error_serialization() {
        let error = ValidationError {
            line: 5,
            message: "Parse error".to_string(),
            error_type: "SyntaxError".to_string(),
        };

        let json = serde_json::to_string(&error);
        assert!(json.is_ok(), "ValidationError should serialize");

        let json = json.unwrap();
        assert!(json.contains("\"line\":5"));
        assert!(json.contains("Parse error"));
    }

    // ============ TOKEN STATS STRUCTURE TESTS ============

    #[test]
    #[cfg(any(feature = "statistics", feature = "token-tools"))]
    fn test_token_stats_serialization() {
        let stats = TokenStats {
            hedl_bytes: 100,
            hedl_tokens: 25,
            hedl_lines: 10,
            json_bytes: 400,
            json_tokens: 100,
            savings_percent: 75,
            tokens_saved: 75,
        };

        let json = serde_json::to_string(&stats);
        assert!(json.is_ok(), "TokenStats should serialize");

        let json = json.unwrap();
        assert!(json.contains("\"hedlBytes\":100"));
        assert!(json.contains("\"savingsPercent\":75"));
    }
}
