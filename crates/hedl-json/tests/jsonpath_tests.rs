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

//! Integration tests for JSONPath query functionality

use hedl_core::parse;
use hedl_json::jsonpath::{
    query, query_count, query_exists, query_first, query_single, QueryConfig, QueryConfigBuilder,
    QueryError,
};

/// Helper to parse HEDL from string for tests
fn parse_hedl(input: &str) -> hedl_core::Document {
    // Prepend HEDL header if not present, or separate header from body if needed
    let hedl = if input.contains("%VERSION") || input.starts_with("%HEDL") {
        input.to_string()
    } else if input.contains("%STRUCT") || input.contains("%NEST") {
        // Has directives but no VERSION - add VERSION and ensure separator
        let (header, body) = if input.contains("---") {
            let parts: Vec<&str> = input.splitn(2, "---").collect();
            (parts[0].trim().to_string(), parts.get(1).map(|s| s.trim().to_string()).unwrap_or_default())
        } else {
            // Extract directives to header
            let mut header_lines = Vec::new();
            let mut body_lines = Vec::new();
            for line in input.lines() {
                if line.trim().starts_with('%') {
                    header_lines.push(line.to_string());
                } else {
                    body_lines.push(line.to_string());
                }
            }
            (header_lines.join("\n"), body_lines.join("\n"))
        };
        format!("%VERSION: 1.0\n{}\n---\n{}", header, body)
    } else {
        format!("%VERSION: 1.0\n---\n{}", input)
    };
    parse(hedl.as_bytes()).unwrap()
}

// ==================== Basic Query Tests ====================

#[test]
fn test_simple_field_query() {
    let hedl = r#"
name: "Alice"
age: 30
email: "alice@example.com"
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results = query(&doc, "$.name", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("Alice"));

    let results = query(&doc, "$.age", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_i64(), Some(30));
}

#[test]
fn test_nested_object_query() {
    let hedl = r#"
user:
  profile:
    name: "Bob"
    age: 25
  settings:
    theme: "dark"
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results = query(&doc, "$.user.profile.name", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("Bob"));

    let results = query(&doc, "$.user.settings.theme", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("dark"));
}

#[test]
fn test_wildcard_query() {
    let hedl = r#"
a: 1
b: 2
c: 3
d: 4
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results = query(&doc, "$.*", &config).unwrap();
    assert_eq!(results.len(), 4);

    // Sum all values
    let sum: i64 = results.iter().filter_map(|v| v.as_i64()).sum();
    assert_eq!(sum, 10);
}

#[test]
fn test_recursive_descent_query() {
    let hedl = r#"
level1:
  level2:
    name: "Alice"
  other:
    name: "Bob"
top:
  name: "Charlie"
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    // Find all 'name' fields at any depth
    let results = query(&doc, "$..name", &config).unwrap();
    assert_eq!(results.len(), 3);

    let names: Vec<&str> = results.iter().filter_map(|v| v.as_str()).collect();
    assert!(names.contains(&"Alice"));
    assert!(names.contains(&"Bob"));
    assert!(names.contains(&"Charlie"));
}

// ==================== Query Helper Function Tests ====================

#[test]
fn test_query_first() {
    let hedl = "name: \"Alice\"\nage: 30";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query_first(&doc, "$.name", &config).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_str(), Some("Alice"));

    let result = query_first(&doc, "$.missing", &config).unwrap();
    assert!(result.is_none());
}

#[test]
fn test_query_single_success() {
    let hedl = "name: \"Alice\"";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query_single(&doc, "$.name", &config).unwrap();
    assert_eq!(result.as_str(), Some("Alice"));
}

#[test]
fn test_query_single_no_results() {
    let hedl = "name: \"Alice\"";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query_single(&doc, "$.missing", &config);
    assert!(result.is_err());
    match result.unwrap_err() {
        QueryError::ExecutionError(msg) => {
            assert!(msg.contains("no results"));
        }
        _ => panic!("Expected ExecutionError"),
    }
}

#[test]
fn test_query_single_multiple_results() {
    let hedl = "a: 1\nb: 2\nc: 3";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query_single(&doc, "$.*", &config);
    assert!(result.is_err());
    match result.unwrap_err() {
        QueryError::ExecutionError(msg) => {
            assert!(msg.contains("expected exactly 1"));
        }
        _ => panic!("Expected ExecutionError"),
    }
}

#[test]
fn test_query_exists() {
    let hedl = "name: \"Alice\"\nage: 30";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    assert!(query_exists(&doc, "$.name", &config).unwrap());
    assert!(query_exists(&doc, "$.age", &config).unwrap());
    assert!(!query_exists(&doc, "$.missing", &config).unwrap());
}

#[test]
fn test_query_count() {
    let hedl = "a: 1\nb: 2\nc: 3\nd: 4\ne: 5";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    assert_eq!(query_count(&doc, "$.*", &config).unwrap(), 5);
    assert_eq!(query_count(&doc, "$.a", &config).unwrap(), 1);
    assert_eq!(query_count(&doc, "$.missing", &config).unwrap(), 0);
}

// ==================== Configuration Tests ====================

#[test]
fn test_max_results_limit() {
    let hedl = "a: 1\nb: 2\nc: 3\nd: 4\ne: 5";
    let doc = parse_hedl(&hedl);
    let config = QueryConfigBuilder::new().max_results(3).build();

    let results = query(&doc, "$.*", &config).unwrap();
    assert_eq!(results.len(), 3);
}

#[test]
fn test_max_results_unlimited() {
    let hedl = "a: 1\nb: 2\nc: 3\nd: 4\ne: 5";
    let doc = parse_hedl(&hedl);
    let config = QueryConfigBuilder::new().max_results(0).build();

    let results = query(&doc, "$.*", &config).unwrap();
    assert_eq!(results.len(), 5);
}

#[test]
fn test_config_builder() {
    let config = QueryConfigBuilder::new()
        .include_metadata(true)
        .flatten_lists(true)
        .include_children(false)
        .max_results(10)
        .build();

    assert!(config.include_metadata);
    assert!(config.flatten_lists);
    assert!(!config.include_children);
    assert_eq!(config.max_results, 10);
}

// ==================== Data Type Tests ====================

#[test]
fn test_query_integers() {
    let hedl = "count: 42\nnegative: -100\nzero: 0";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query_single(&doc, "$.count", &config).unwrap();
    assert_eq!(result.as_i64(), Some(42));

    let result = query_single(&doc, "$.negative", &config).unwrap();
    assert_eq!(result.as_i64(), Some(-100));

    let result = query_single(&doc, "$.zero", &config).unwrap();
    assert_eq!(result.as_i64(), Some(0));
}

#[test]
fn test_query_floats() {
    let hedl = "price: 19.99\nratio: 0.5\nscientific: 1.23e10";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query_single(&doc, "$.price", &config).unwrap();
    assert_eq!(result.as_f64(), Some(19.99));

    let result = query_single(&doc, "$.ratio", &config).unwrap();
    assert_eq!(result.as_f64(), Some(0.5));
}

#[test]
fn test_query_booleans() {
    let hedl = "active: true\ndeleted: false";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query_single(&doc, "$.active", &config).unwrap();
    assert_eq!(result.as_bool(), Some(true));

    let result = query_single(&doc, "$.deleted", &config).unwrap();
    assert_eq!(result.as_bool(), Some(false));
}

#[test]
fn test_query_null() {
    let hedl = "value: ~\nother: \"not null\"";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query_single(&doc, "$.value", &config).unwrap();
    assert!(result.is_null());

    let result = query_single(&doc, "$.other", &config).unwrap();
    assert!(!result.is_null());
}

#[test]
fn test_query_strings() {
    let hedl = r#"
name: "Alice"
empty: ""
unicode: "日本語 🎉"
multiline: """
Line 1
Line 2
"""
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query_single(&doc, "$.name", &config).unwrap();
    assert_eq!(result.as_str(), Some("Alice"));

    let result = query_single(&doc, "$.empty", &config).unwrap();
    assert_eq!(result.as_str(), Some(""));

    let result = query_single(&doc, "$.unicode", &config).unwrap();
    assert_eq!(result.as_str(), Some("日本語 🎉"));
}

// ==================== Complex Query Tests ====================

#[test]
fn test_array_bracket_notation() {
    let hedl = r#"
field_with_underscore: "value1"
field_with_number2: "value2"
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results = query(&doc, "$['field_with_underscore']", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("value1"));
}

#[test]
fn test_deep_nesting() {
    let hedl = r#"
level1:
  level2:
    level3:
      level4:
        level5:
          value: "deep"
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results = query(&doc, "$.level1.level2.level3.level4.level5.value", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("deep"));
}

#[test]
fn test_multiple_paths() {
    let hedl = r#"
user:
  name: "Alice"
  email: "alice@example.com"
admin:
  name: "Bob"
  email: "bob@example.com"
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    // Get all names at any depth
    let results = query(&doc, "$..name", &config).unwrap();
    assert_eq!(results.len(), 2);

    let names: Vec<&str> = results.iter().filter_map(|v| v.as_str()).collect();
    assert!(names.contains(&"Alice"));
    assert!(names.contains(&"Bob"));
}

// ==================== Error Handling Tests ====================

#[test]
fn test_invalid_jsonpath_syntax() {
    let hedl = "name: \"Alice\"";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query(&doc, "$$invalid", &config);
    assert!(result.is_err());
    match result.unwrap_err() {
        QueryError::InvalidExpression(_) => {}
        _ => panic!("Expected InvalidExpression error"),
    }
}

#[test]
fn test_empty_path() {
    let hedl = "name: \"Alice\"";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let result = query(&doc, "", &config);
    assert!(result.is_err());
}

// ==================== Edge Cases ====================

#[test]
fn test_empty_document() {
    let hedl = "";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results = query(&doc, "$", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].is_object());
}

#[test]
fn test_root_query() {
    let hedl = "name: \"Alice\"\nage: 30";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results = query(&doc, "$", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].is_object());
}

#[test]
fn test_unicode_field_names() {
    // HEDL field names must be ASCII identifiers, but values can contain unicode
    let hedl = "name: \"太郎\"\nage: 25";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results = query(&doc, "$.name", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("太郎"));
}

#[test]
fn test_query_with_special_characters() {
    // HEDL field names must be valid identifiers (lowercase with underscores)
    let hedl = r#"
field_with_underscores: "value1"
another_field_name: "value2"
field123: "value3"
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results = query(&doc, "$['field_with_underscores']", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("value1"));
}

// ==================== Performance Tests ====================

#[test]
fn test_large_document_query() {
    // Generate a document with many fields
    let mut hedl = String::new();
    for i in 0..100 {
        hedl.push_str(&format!("field{}: {}\n", i, i));
    }

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    // Query specific field
    let results = query(&doc, "$.field42", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_i64(), Some(42));

    // Query all fields
    let results = query(&doc, "$.*", &config).unwrap();
    assert_eq!(results.len(), 100);
}

#[test]
fn test_deeply_nested_query() {
    // Create a deeply nested structure with proper indentation
    let hedl = r#"
root:
  l0:
    l1:
      l2:
        l3:
          l4:
            l5:
              value: "deep"
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results = query(&doc, "$.root.l0.l1.l2.l3.l4.l5.value", &config).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].as_str(), Some("deep"));
}

// ==================== Real-World Use Cases ====================

#[test]
fn test_extract_user_emails() {
    let hedl = r#"
users:
  alice:
    email: "alice@example.com"
    active: true
  bob:
    email: "bob@example.com"
    active: false
  charlie:
    email: "charlie@example.com"
    active: true
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    // Extract all emails
    let results = query(&doc, "$..email", &config).unwrap();
    assert_eq!(results.len(), 3);

    let emails: Vec<&str> = results.iter().filter_map(|v| v.as_str()).collect();
    assert!(emails.contains(&"alice@example.com"));
    assert!(emails.contains(&"bob@example.com"));
    assert!(emails.contains(&"charlie@example.com"));
}

#[test]
fn test_configuration_extraction() {
    let hedl = r#"
database:
  host: "localhost"
  port: 5432
  credentials:
    username: "admin"
    password: "secret"
cache:
  enabled: true
  ttl: 300
"#;

    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    // Extract database host
    let host = query_single(&doc, "$.database.host", &config).unwrap();
    assert_eq!(host.as_str(), Some("localhost"));

    // Extract database port
    let port = query_single(&doc, "$.database.port", &config).unwrap();
    assert_eq!(port.as_i64(), Some(5432));

    // Extract cache TTL
    let ttl = query_single(&doc, "$.cache.ttl", &config).unwrap();
    assert_eq!(ttl.as_i64(), Some(300));

    // Check if cache is enabled
    let enabled = query_single(&doc, "$.cache.enabled", &config).unwrap();
    assert_eq!(enabled.as_bool(), Some(true));
}

// ==================== Invariant Tests ====================

#[test]
fn test_query_idempotence() {
    let hedl = "name: \"Alice\"\nage: 30";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    let results1 = query(&doc, "$.name", &config).unwrap();
    let results2 = query(&doc, "$.name", &config).unwrap();

    assert_eq!(results1, results2);
}

#[test]
fn test_query_determinism() {
    let hedl = "a: 1\nb: 2\nc: 3";
    let doc = parse_hedl(&hedl);
    let config = QueryConfig::default();

    // Multiple queries should return same results
    for _ in 0..5 {
        let results = query(&doc, "$.*", &config).unwrap();
        assert_eq!(results.len(), 3);
    }
}
