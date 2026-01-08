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


// Standalone tests for needs_quoting optimization
//
// This file tests the needs_quoting function indirectly through encode_string
// since needs_quoting is private.

use hedl_toon::{to_toon, ToToonConfig, Delimiter};
use hedl_core::{Document, Item, Value};

#[test]
fn test_quoting_through_document() {
    let mut doc = Document::new((1, 0));

    // Test various string values that exercise needs_quoting
    let test_cases = vec![
        ("empty", ""),
        ("simple", "hello"),
        ("whitespace_leading", " hello"),
        ("whitespace_trailing", "hello "),
        ("boolean", "true"),
        ("numeric", "123"),
        ("structural", "foo:bar"),
        ("escaped", "hello\"world"),
        ("delimiter", "hello,world"),
        ("reference", "@User:123"),
        ("minus", "-item"),
        ("unicode_ws", "\u{00A0}hello"),
    ];

    for (key, value) in test_cases {
        doc.root.insert(
            key.to_string(),
            Item::Scalar(Value::String(value.to_string())),
        );
    }

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // Verify quoting behavior
    assert!(toon.contains("empty: \"\""));  // Empty needs quotes
    assert!(toon.contains("simple: hello"));  // Simple doesn't need quotes
    assert!(toon.contains("\" hello\""));  // Leading whitespace needs quotes
    assert!(toon.contains("\"hello \""));  // Trailing whitespace needs quotes
    assert!(toon.contains("\"true\""));  // Boolean literal needs quotes
    assert!(toon.contains("\"123\""));  // Numeric needs quotes
    assert!(toon.contains("\"foo:bar\""));  // Structural char needs quotes
    assert!(toon.contains("\"hello\\\"world\""));  // Escaped quote needs quotes
    assert!(toon.contains("\"hello,world\""));  // Delimiter needs quotes
    assert!(toon.contains("\"@User:123\""));  // Reference needs quotes
    assert!(toon.contains("\"-item\""));  // Minus needs quotes
    assert!(toon.contains("\"\u{00A0}hello\""));  // Unicode whitespace needs quotes
}

#[test]
fn test_delimiter_specific_quoting() {
    let mut doc = Document::new((1, 0));
    doc.root.insert("comma".to_string(), Item::Scalar(Value::String("a,b".to_string())));
    doc.root.insert("tab".to_string(), Item::Scalar(Value::String("a\tb".to_string())));
    doc.root.insert("pipe".to_string(), Item::Scalar(Value::String("a|b".to_string())));

    // Test with comma delimiter (default)
    let config_comma = ToToonConfig {
        indent: 2,
        delimiter: Delimiter::Comma,
    };
    let toon_comma = to_toon(&doc, &config_comma).unwrap();
    assert!(toon_comma.contains("\"a,b\""));  // Comma needs quoting with comma delimiter
    assert!(toon_comma.contains("\"a\\tb\""));  // Tab is escape sequence, always quoted
    assert!(!toon_comma.contains("\"a|b\"") || toon_comma.contains("a|b"));  // Pipe doesn't need quoting with comma delimiter

    // Test with tab delimiter
    let config_tab = ToToonConfig {
        indent: 2,
        delimiter: Delimiter::Tab,
    };
    let toon_tab = to_toon(&doc, &config_tab).unwrap();
    assert!(!toon_tab.contains("\"a,b\"") || toon_tab.contains("comma: a,b"));  // Comma doesn't need quoting with tab delimiter
    assert!(toon_tab.contains("\"a\\tb\""));  // Tab is escape sequence, always quoted

    // Test with pipe delimiter
    let config_pipe = ToToonConfig {
        indent: 2,
        delimiter: Delimiter::Pipe,
    };
    let toon_pipe = to_toon(&doc, &config_pipe).unwrap();
    assert!(!toon_pipe.contains("\"a,b\"") || toon_pipe.contains("comma: a,b"));  // Comma doesn't need quoting with pipe delimiter
    assert!(toon_pipe.contains("\"a|b\""));  // Pipe needs quoting with pipe delimiter
}

#[test]
fn test_no_quoting_needed() {
    let mut doc = Document::new((1, 0));

    // These strings should NOT need quoting
    let test_cases = vec![
        "hello",
        "world",
        "simple",
        "HelloWorld",
        "hello_world",
        "CONSTANT_NAME",
        "abc123",
        "test_123",
        "camelCase",
    ];

    for (i, value) in test_cases.iter().enumerate() {
        doc.root.insert(
            format!("key{}", i),
            Item::Scalar(Value::String(value.to_string())),
        );
    }

    let config = ToToonConfig::default();
    let toon = to_toon(&doc, &config).unwrap();

    // None of these should have quotes in the output
    for value in test_cases {
        // Check that value appears without quotes
        let pattern = format!(": {}", value);
        assert!(toon.contains(&pattern), "Expected '{}' to appear unquoted", value);
    }
}
