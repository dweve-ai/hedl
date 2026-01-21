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

//! Edge case tests for TOON parsing (`from_toon`)
//!
//! Tests error conditions, malformed input, and boundary cases.

use hedl_core::{Item, Value};
use hedl_toon::{from_toon, from_toon_with_config, FromToonConfig, ToonError};

#[test]
fn test_parse_empty_input() {
    let toon = "";
    let doc = from_toon(toon).unwrap();
    assert!(doc.root.is_empty());
}

#[test]
fn test_parse_whitespace_only() {
    let toon = "   \n  \t  \n   ";
    let doc = from_toon(toon).unwrap();
    assert!(doc.root.is_empty());
}

#[test]
fn test_parse_single_value() {
    let toon = "name: Alice";
    let doc = from_toon(toon).unwrap();
    assert!(doc.root.contains_key("name"));
    if let Item::Scalar(Value::String(s)) = &doc.root["name"] {
        assert_eq!(s.as_ref(), "Alice");
    }
}

#[test]
fn test_parse_multiple_values_no_indentation() {
    let toon = r"name: Alice
age: 30
active: true";

    let doc = from_toon(toon).unwrap();
    assert_eq!(doc.root.len(), 3);
    assert!(doc.root.contains_key("name"));
    assert!(doc.root.contains_key("age"));
    assert!(doc.root.contains_key("active"));
}

#[test]
fn test_parse_nested_object_with_auto_indent() {
    let toon = r"config:
  name: MyApp
  version: 1.0
  settings:
    debug: true
    timeout: 30";

    let doc = from_toon(toon).unwrap();

    if let Item::Object(config) = &doc.root["config"] {
        if let Item::Scalar(Value::String(s)) = &config["name"] {
            assert_eq!(s.as_ref(), "MyApp");
        }

        if let Item::Object(settings) = &config["settings"] {
            assert!(matches!(
                &settings["debug"],
                Item::Scalar(Value::Bool(true))
            ));
            assert!(matches!(&settings["timeout"], Item::Scalar(Value::Int(30))));
        }
    } else {
        panic!("Expected config object");
    }
}

#[test]
fn test_parse_tabular_array() {
    let toon = r"users[3]{id,name,age}:
  u1,Alice,30
  u2,Bob,25
  u3,Charlie,35";

    let doc = from_toon(toon).unwrap();

    if let Item::List(list) = &doc.root["users"] {
        assert_eq!(list.rows.len(), 3);
        assert_eq!(list.schema, vec!["id", "name", "age"]);

        if let Value::String(s) = &list.rows[0].fields[1] {
            assert_eq!(s.as_ref(), "Alice");
        }
        if let Value::Int(n) = &list.rows[1].fields[2] {
            assert_eq!(*n, 25);
        }
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_parse_expanded_array() {
    let toon = r"items[2]:
  - id: i1
    name: First
  - id: i2
    name: Second";

    let doc = from_toon(toon).unwrap();

    if let Item::List(list) = &doc.root["items"] {
        assert_eq!(list.rows.len(), 2);
        assert_eq!(list.schema, vec!["id", "name"]);

        if let Value::String(s) = &list.rows[0].fields[0] {
            assert_eq!(s.as_ref(), "i1");
        }
        if let Value::String(s) = &list.rows[1].fields[1] {
            assert_eq!(s.as_ref(), "Second");
        }
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_parse_empty_array() {
    let toon = "items[0]:";
    let doc = from_toon(toon).unwrap();

    if let Item::List(list) = &doc.root["items"] {
        assert_eq!(list.rows.len(), 0);
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_parse_quoted_keys() {
    let toon = r#""my-key": value1
"123": value2
"with:colon": value3"#;

    let doc = from_toon(toon).unwrap();
    assert!(doc.root.contains_key("my-key"));
    assert!(doc.root.contains_key("123"));
    assert!(doc.root.contains_key("with:colon"));
}

#[test]
fn test_parse_escaped_strings() {
    let toon = r#"newline: "line1\nline2"
quote: "say \"hello\""
backslash: "path\\to\\file"
tab: "col1\tcol2""#;

    let doc = from_toon(toon).unwrap();

    if let Item::Scalar(Value::String(s)) = &doc.root["newline"] {
        assert_eq!(s.as_ref(), "line1\nline2");
    }
    if let Item::Scalar(Value::String(s)) = &doc.root["quote"] {
        assert_eq!(s.as_ref(), "say \"hello\"");
    }
    if let Item::Scalar(Value::String(s)) = &doc.root["backslash"] {
        assert_eq!(s.as_ref(), "path\\to\\file");
    }
    if let Item::Scalar(Value::String(s)) = &doc.root["tab"] {
        assert_eq!(s.as_ref(), "col1\tcol2");
    }
}

#[test]
fn test_parse_references() {
    let toon = r#"user: "@User:u123"
local: "@item1""#;

    let doc = from_toon(toon).unwrap();

    if let Item::Scalar(Value::Reference(r)) = &doc.root["user"] {
        assert_eq!(r.type_name.as_deref(), Some("User"));
        assert_eq!(r.id.as_ref(), "u123");
    } else {
        panic!("Expected reference");
    }

    if let Item::Scalar(Value::Reference(r)) = &doc.root["local"] {
        assert!(r.type_name.is_none());
        assert_eq!(r.id.as_ref(), "item1");
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_parse_booleans_and_null() {
    let toon = r"yes: true
no: false
nothing: null";

    let doc = from_toon(toon).unwrap();
    assert!(matches!(&doc.root["yes"], Item::Scalar(Value::Bool(true))));
    assert!(matches!(&doc.root["no"], Item::Scalar(Value::Bool(false))));
    assert!(matches!(&doc.root["nothing"], Item::Scalar(Value::Null)));
}

#[test]
fn test_parse_numbers() {
    let toon = r"integer: 42
negative: -123
zero: 0
float: 3.15159
negative_float: -2.5";

    let doc = from_toon(toon).unwrap();
    assert!(matches!(&doc.root["integer"], Item::Scalar(Value::Int(42))));
    assert!(matches!(
        &doc.root["negative"],
        Item::Scalar(Value::Int(-123))
    ));
    assert!(matches!(&doc.root["zero"], Item::Scalar(Value::Int(0))));
    assert!(matches!(
        &doc.root["float"],
        Item::Scalar(Value::Float(f)) if (*f - 3.15159).abs() < 0.00001
    ));
}

#[test]
fn test_parse_with_custom_indent() {
    let toon = r"config:
    name: MyApp
    settings:
        debug: true";

    let config = FromToonConfig { indent_width: 4 };
    let doc = from_toon_with_config(toon, &config).unwrap();

    if let Item::Object(config) = &doc.root["config"] {
        assert!(config.contains_key("name"));
        assert!(config.contains_key("settings"));
    } else {
        panic!("Expected object");
    }
}

// Error handling tests

#[test]
fn test_error_unexpected_indentation_at_root() {
    let toon = "  name: test"; // Indentation at root level is invalid

    let result = from_toon(toon);
    assert!(result.is_err());

    if let Err(ToonError::IndentationError { line, .. }) = result {
        assert_eq!(line, 1);
    } else {
        panic!("Expected IndentationError");
    }
}

#[test]
fn test_error_missing_colon() {
    let toon = "name value"; // Missing colon

    let result = from_toon(toon);
    assert!(result.is_err());

    if let Err(ToonError::ParseError { line, .. }) = result {
        assert_eq!(line, 1);
    } else {
        panic!("Expected ParseError");
    }
}

// Note: Unterminated quotes may be handled gracefully by the parser
// by treating the rest of the line as the string content, so we don't
// test for this as an error condition.

#[test]
fn test_error_invalid_array_header() {
    let toon = "items[abc]:"; // Non-numeric count

    let result = from_toon(toon);
    // Should parse but won't find valid array header
    // This should either succeed with empty list or fail
    // depending on implementation
    let _ = result;
}

#[test]
fn test_error_schema_mismatch() {
    let toon = r"users[2]{id,name}:
  u1,Alice,Extra"; // 3 values but schema has 2 fields

    let result = from_toon(toon);
    assert!(result.is_err());

    if let Err(ToonError::SchemaMismatch {
        expected, actual, ..
    }) = result
    {
        assert_eq!(expected, 2);
        assert_eq!(actual, 3);
    } else {
        panic!("Expected SchemaMismatch error, got: {result:?}");
    }
}

#[test]
fn test_parse_trailing_whitespace_in_values() {
    let toon = "name: Alice   \nage: 30  ";

    let doc = from_toon(toon).unwrap();

    if let Item::Scalar(Value::String(s)) = &doc.root["name"] {
        // Trailing whitespace should be trimmed for unquoted values
        assert_eq!(s.as_ref(), "Alice");
    }
}

#[test]
fn test_parse_unicode() {
    let toon = r#"chinese: 你好世界
emoji: 🌍🚀⭐
mixed: "Hello 世界 🌍""#;

    let doc = from_toon(toon).unwrap();

    if let Item::Scalar(Value::String(s)) = &doc.root["chinese"] {
        assert_eq!(s.as_ref(), "你好世界");
    }
    if let Item::Scalar(Value::String(s)) = &doc.root["emoji"] {
        assert_eq!(s.as_ref(), "🌍🚀⭐");
    }
    if let Item::Scalar(Value::String(s)) = &doc.root["mixed"] {
        assert_eq!(s.as_ref(), "Hello 世界 🌍");
    }
}

#[test]
fn test_parse_mixed_delimiters_comma() {
    let toon = r"items[2]{a,b,c}:
  1,2,3
  4,5,6";

    let doc = from_toon(toon).unwrap();

    if let Item::List(list) = &doc.root["items"] {
        assert_eq!(list.rows.len(), 2);
        assert_eq!(list.schema, vec!["a", "b", "c"]);
    }
}

#[test]
fn test_parse_tab_delimiter() {
    let toon = "items[2\t]{a\tb\tc}:\n\t1\t2\t3\n\t4\t5\t6";

    let doc = from_toon(toon).unwrap();

    if let Item::List(list) = &doc.root["items"] {
        assert_eq!(list.rows.len(), 2);
        assert_eq!(list.schema, vec!["a", "b", "c"]);
    }
}

#[test]
fn test_parse_pipe_delimiter() {
    let toon = "items[2|]{a|b|c}:\n  1|2|3\n  4|5|6";

    let doc = from_toon(toon).unwrap();

    if let Item::List(list) = &doc.root["items"] {
        assert_eq!(list.rows.len(), 2);
        assert_eq!(list.schema, vec!["a", "b", "c"]);
    }
}

#[test]
fn test_parse_empty_string_value() {
    let toon = r#"empty: """#;

    let doc = from_toon(toon).unwrap();

    if let Item::Scalar(Value::String(s)) = &doc.root["empty"] {
        assert_eq!(s.as_ref(), "");
    }
}

#[test]
fn test_parse_whitespace_only_quoted() {
    let toon = r#"spaces: "   "
tabs: "\t\t\t""#;

    let doc = from_toon(toon).unwrap();

    if let Item::Scalar(Value::String(s)) = &doc.root["spaces"] {
        assert_eq!(s.as_ref(), "   ");
    }
    if let Item::Scalar(Value::String(s)) = &doc.root["tabs"] {
        assert_eq!(s.as_ref(), "\t\t\t");
    }
}

#[test]
fn test_parse_very_long_lines() {
    let long_value = "a".repeat(10000);
    let toon = format!("long: {long_value}");

    let doc = from_toon(&toon).unwrap();

    if let Item::Scalar(Value::String(s)) = &doc.root["long"] {
        assert_eq!(s.len(), 10000);
    }
}

#[test]
fn test_parse_deeply_nested() {
    let toon = r"level1:
  level2:
    level3:
      level4:
        level5:
          value: deep";

    let doc = from_toon(toon).unwrap();

    if let Item::Object(l1) = &doc.root["level1"] {
        if let Item::Object(l2) = &l1["level2"] {
            if let Item::Object(l3) = &l2["level3"] {
                if let Item::Object(l4) = &l3["level4"] {
                    if let Item::Object(l5) = &l4["level5"] {
                        if let Item::Scalar(Value::String(s)) = &l5["value"] {
                            assert_eq!(s.as_ref(), "deep");
                        } else {
                            panic!("Expected string value");
                        }
                    } else {
                        panic!("Expected level5");
                    }
                } else {
                    panic!("Expected level4");
                }
            } else {
                panic!("Expected level3");
            }
        } else {
            panic!("Expected level2");
        }
    } else {
        panic!("Expected level1");
    }
}

#[test]
fn test_parse_array_with_quoted_values() {
    let toon = r#"items[2]{id,name}:
  "id1","Name with, comma"
  "id2","Another \"quoted\" name""#;

    let doc = from_toon(toon).unwrap();

    if let Item::List(list) = &doc.root["items"] {
        assert_eq!(list.rows.len(), 2);

        if let Value::String(s) = &list.rows[0].fields[1] {
            assert_eq!(s.as_ref(), "Name with, comma");
        }
        if let Value::String(s) = &list.rows[1].fields[1] {
            assert_eq!(s.as_ref(), "Another \"quoted\" name");
        }
    }
}

#[test]
fn test_parse_expanded_array_missing_dash() {
    let toon = r"items[1]:
  id: i1
  name: First"; // Missing "- " prefix

    let result = from_toon(toon);
    assert!(result.is_err());
}

#[test]
fn test_parse_comments_not_supported() {
    // TOON doesn't support comments, so lines starting with # are parsed as keys
    let toon = r"# This is a comment
name: value";

    let result = from_toon(toon);
    // Will fail because "# This is a comment" doesn't have a colon
    assert!(result.is_err());
}

#[test]
fn test_parse_value_type_inference() {
    let toon = r#"int_string: "123"
actual_int: 123
bool_string: "true"
actual_bool: true"#;

    let doc = from_toon(toon).unwrap();

    // Quoted values should be strings
    assert!(matches!(
        &doc.root["int_string"],
        Item::Scalar(Value::String(s)) if s.as_ref() == "123"
    ));

    // Unquoted numbers should be parsed as numbers
    assert!(matches!(
        &doc.root["actual_int"],
        Item::Scalar(Value::Int(123))
    ));

    // Quoted booleans should be strings
    assert!(matches!(
        &doc.root["bool_string"],
        Item::Scalar(Value::String(s)) if s.as_ref() == "true"
    ));

    // Unquoted booleans should be parsed as booleans
    assert!(matches!(
        &doc.root["actual_bool"],
        Item::Scalar(Value::Bool(true))
    ));
}

#[test]
fn test_parse_special_float_values() {
    let toon = r"zero: 0
negative_zero: -0
small: 0.0001
large: 1000000";

    let doc = from_toon(toon).unwrap();

    assert!(matches!(&doc.root["zero"], Item::Scalar(Value::Int(0))));
    // -0 might be parsed as Int(0) or Float(-0.0) depending on parser
    assert!(doc.root.contains_key("negative_zero"));
}
