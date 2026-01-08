// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Validation tests for hedl-neo4j security features.
//!
//! These tests verify that the security features like string length limits
//! properly prevent resource exhaustion attacks.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_core::lex::Span;
use hedl_neo4j::{to_cypher, Neo4jError, ToCypherConfig};
use std::collections::BTreeMap;

#[test]
fn test_string_length_validation_in_conversion() {
    // Create a document with a large string property
    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "bio".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: vec![
                    Value::String("alice".to_string()),
                    Value::String("x".repeat(2000)), // 2KB string
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Config with 1KB limit - should fail
    let config = ToCypherConfig::default().with_max_string_length(1000);
    let result = to_cypher(&doc, &config);
    assert!(result.is_err());

    if let Err(Neo4jError::StringLengthExceeded {
        length,
        max_length,
        property,
    }) = result
    {
        assert_eq!(length, 2000);
        assert_eq!(max_length, 1000);
        assert_eq!(property, "bio");
    } else {
        panic!("Expected StringLengthExceeded error");
    }

    // Config with 3KB limit - should succeed
    let config_large = ToCypherConfig::default().with_max_string_length(3000);
    let result_ok = to_cypher(&doc, &config_large);
    assert!(result_ok.is_ok());
}

#[test]
fn test_expression_string_length_validation() {
    use hedl_core::{Expression, ExprLiteral};

    // Create a document with an expression containing a large string
    let long_string = "x".repeat(2000);
    let mut root = BTreeMap::new();
    root.insert(
        "calcs".to_string(),
        Item::List(MatrixList {
            type_name: "Calc".to_string(),
            schema: vec!["id".to_string(), "formula".to_string()],
            rows: vec![Node {
                type_name: "Calc".to_string(),
                id: "c1".to_string(),
                fields: vec![
                    Value::String("c1".to_string()),
                    Value::Expression(Expression::Literal {
                        value: ExprLiteral::String(long_string),
                        span: Span::file_start(),
                    }),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // The expression string "$(String("xxx..."))" will be > 2000 bytes
    // Config with 1KB limit - should fail
    let config = ToCypherConfig::default().with_max_string_length(1000);
    let result = to_cypher(&doc, &config);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(Neo4jError::StringLengthExceeded { .. })
    ));

    // Config with 3KB limit - should succeed
    let config_large = ToCypherConfig::default().with_max_string_length(3000);
    let result_ok = to_cypher(&doc, &config_large);
    assert!(result_ok.is_ok());
}

#[test]
fn test_untrusted_input_config() {
    // Create a document with strings that would exceed untrusted limits
    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "data".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: vec![
                    Value::String("alice".to_string()),
                    Value::String("x".repeat(2_000_000)), // 2MB string
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Using for_untrusted_input() config (1MB limit)
    let config = ToCypherConfig::for_untrusted_input();
    let result = to_cypher(&doc, &config);
    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(Neo4jError::StringLengthExceeded { .. })
    ));
}

#[test]
fn test_no_string_length_limit() {
    // Create a document with a very large string
    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "data".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: vec![
                    Value::String("alice".to_string()),
                    Value::String("x".repeat(100_000)), // 100KB string
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Config without limit - should succeed
    let config = ToCypherConfig::default().without_string_length_limit();
    let result = to_cypher(&doc, &config);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_strings_within_limit() {
    // Create a document with multiple small strings
    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string(),
                "bio".to_string(),
            ],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: vec![
                    Value::String("alice".to_string()),
                    Value::String("Alice Smith".to_string()),
                    Value::String("alice@example.com".to_string()),
                    Value::String("Software engineer".to_string()),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Config with 100 byte limit - should succeed for all fields
    let config = ToCypherConfig::default().with_max_string_length(100);
    let result = to_cypher(&doc, &config);
    assert!(result.is_ok());
}

#[test]
fn test_unicode_string_length_bytes() {
    // Create a document with Unicode strings
    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "text".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: vec![
                    Value::String("alice".to_string()),
                    // Each emoji is 4 bytes in UTF-8
                    // 30 emojis = 120 bytes
                    Value::String("🔥".repeat(30)),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Config with 100 byte limit - should fail (120 bytes > 100)
    let config = ToCypherConfig::default().with_max_string_length(100);
    let result = to_cypher(&doc, &config);
    assert!(result.is_err());

    // Config with 150 byte limit - should succeed
    let config_large = ToCypherConfig::default().with_max_string_length(150);
    let result_ok = to_cypher(&doc, &config_large);
    assert!(result_ok.is_ok());
}

#[test]
fn test_empty_string_always_valid() {
    // Create a document with empty string
    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: vec![Value::String("alice".to_string()), Value::String("".to_string())],
                children: BTreeMap::new(),
                child_count: None,
            }],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Even with very strict limit, empty string should be OK
    let config = ToCypherConfig::default().with_max_string_length(1);
    let result = to_cypher(&doc, &config);
    assert!(result.is_ok());
}
