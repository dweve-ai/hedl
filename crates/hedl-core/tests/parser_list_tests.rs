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

//! Parser integration tests for list literals.
//!
//! Tests the integration of list literal parsing `(...)` into the full HEDL parser.

use hedl_core::{parse, Item, Value};

// ==================== Key-Value Pair Tests ====================

#[test]
fn test_parse_document_with_list_in_key_value_pair() {
    let input = br#"%VERSION: 1.1
---
roles: (admin, editor, viewer)
"#;
    let doc = parse(input).unwrap();

    // Verify roles is a List
    let roles = doc.root.get("roles").expect("roles key missing");
    match roles {
        Item::Scalar(Value::List(items)) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s.as_ref() == "admin"));
            assert!(matches!(&items[1], Value::String(s) if s.as_ref() == "editor"));
            assert!(matches!(&items[2], Value::String(s) if s.as_ref() == "viewer"));
        }
        _ => panic!("Expected List, got {:?}", roles),
    }
}

#[test]
fn test_parse_document_with_empty_list() {
    let input = br#"%VERSION: 1.1
---
tags: ()
"#;
    let doc = parse(input).unwrap();

    // Verify tags is an empty List
    let tags = doc.root.get("tags").expect("tags key missing");
    match tags {
        Item::Scalar(Value::List(items)) => {
            assert!(items.is_empty(), "Expected empty list");
        }
        _ => panic!("Expected List, got {:?}", tags),
    }
}

#[test]
fn test_parse_document_with_bool_list() {
    let input = br#"%VERSION: 1.1
---
flags: (true, false, true)
"#;
    let doc = parse(input).unwrap();

    // Verify flags is a List of bools
    let flags = doc.root.get("flags").expect("flags key missing");
    match flags {
        Item::Scalar(Value::List(items)) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::Bool(true)));
            assert!(matches!(&items[1], Value::Bool(false)));
            assert!(matches!(&items[2], Value::Bool(true)));
        }
        _ => panic!("Expected List, got {:?}", flags),
    }
}

#[test]
fn test_parse_document_with_int_list() {
    let input = br#"%VERSION: 1.1
---
numbers: (1, 2, 3, 42)
"#;
    let doc = parse(input).unwrap();

    // Verify numbers is a List of ints
    let numbers = doc.root.get("numbers").expect("numbers key missing");
    match numbers {
        Item::Scalar(Value::List(items)) => {
            assert_eq!(items.len(), 4);
            assert!(matches!(&items[0], Value::Int(1)));
            assert!(matches!(&items[1], Value::Int(2)));
            assert!(matches!(&items[2], Value::Int(3)));
            assert!(matches!(&items[3], Value::Int(42)));
        }
        _ => panic!("Expected List, got {:?}", numbers),
    }
}

#[test]
fn test_parse_document_with_mixed_type_list() {
    let input = br#"%VERSION: 1.1
---
mixed: (1, "two", true, ~)
"#;
    let doc = parse(input).unwrap();

    // Verify mixed contains int, string, bool, null
    let mixed = doc.root.get("mixed").expect("mixed key missing");
    match mixed {
        Item::Scalar(Value::List(items)) => {
            assert_eq!(items.len(), 4);
            assert!(matches!(&items[0], Value::Int(1)));
            assert!(matches!(&items[1], Value::String(s) if s.as_ref() == "two"));
            assert!(matches!(&items[2], Value::Bool(true)));
            assert!(matches!(&items[3], Value::Null));
        }
        _ => panic!("Expected List, got {:?}", mixed),
    }
}

#[test]
fn test_parse_document_with_nested_object_containing_list() {
    let input = br#"%VERSION: 1.1
---
user:
 name: Alice
 roles: (admin, editor)
"#;
    let doc = parse(input).unwrap();

    // Verify nested structure
    let user = doc.root.get("user").expect("user key missing");
    match user {
        Item::Object(obj) => {
            let name = obj.get("name").expect("name key missing");
            assert!(matches!(name, Item::Scalar(Value::String(s)) if s.as_ref() == "Alice"));

            let roles = obj.get("roles").expect("roles key missing");
            match roles {
                Item::Scalar(Value::List(items)) => {
                    assert_eq!(items.len(), 2);
                    assert!(matches!(&items[0], Value::String(s) if s.as_ref() == "admin"));
                    assert!(matches!(&items[1], Value::String(s) if s.as_ref() == "editor"));
                }
                _ => panic!("Expected List for roles"),
            }
        }
        _ => panic!("Expected Object for user"),
    }
}

#[test]
fn test_parse_document_with_list_and_tensor_distinguished() {
    let input = br#"%VERSION: 1.1
---
roles: (admin, editor)
weights: [0.5, 0.3, 0.2]
"#;
    let doc = parse(input).unwrap();

    // Verify roles is List
    let roles = doc.root.get("roles").expect("roles key missing");
    assert!(matches!(roles, Item::Scalar(Value::List(_))));

    // Verify weights is Tensor
    let weights = doc.root.get("weights").expect("weights key missing");
    assert!(matches!(weights, Item::Scalar(Value::Tensor(_))));
}

// ==================== Matrix Tests ====================

#[test]
fn test_parse_matrix_row_with_list_cell() {
    let input = br#"%VERSION: 1.1
%STRUCT: User: [id, name, roles]
---
users:@User
 |u1, Alice, (admin, editor)
 |u2, Bob, (viewer)
"#;
    let doc = parse(input).unwrap();

    // Verify matrix rows have List values
    let users = doc.root.get("users").expect("users key missing");
    match users {
        Item::List(matrix) => {
            assert_eq!(matrix.rows.len(), 2);

            // Check Alice's roles
            let alice = &matrix.rows[0];
            assert_eq!(alice.fields.len(), 3);
            match &alice.fields[2] {
                Value::List(roles) => {
                    assert_eq!(roles.len(), 2);
                    assert!(matches!(&roles[0], Value::String(s) if s.as_ref() == "admin"));
                    assert!(matches!(&roles[1], Value::String(s) if s.as_ref() == "editor"));
                }
                _ => panic!("Expected List for Alice's roles"),
            }

            // Check Bob's roles
            let bob = &matrix.rows[1];
            assert_eq!(bob.fields.len(), 3);
            match &bob.fields[2] {
                Value::List(roles) => {
                    assert_eq!(roles.len(), 1);
                    assert!(matches!(&roles[0], Value::String(s) if s.as_ref() == "viewer"));
                }
                _ => panic!("Expected List for Bob's roles"),
            }
        }
        _ => panic!("Expected List for users"),
    }
}

#[test]
fn test_parse_list_with_references() {
    let input = br#"%VERSION: 1.1
---
assignees: (@user1, @user2, @user3)
"#;
    let doc = parse(input).unwrap();

    // Verify list contains references
    let assignees = doc.root.get("assignees").expect("assignees key missing");
    match assignees {
        Item::Scalar(Value::List(items)) => {
            assert_eq!(items.len(), 3);
            for item in items.iter() {
                assert!(matches!(item, Value::Reference(_)));
            }
        }
        _ => panic!("Expected List, got {:?}", assignees),
    }
}

#[test]
fn test_parse_list_with_enum_directive_rejected() {
    // %ENUM was removed in v2.0
    let input = br#"%VERSION: 1.1
%ENUM: roles: {a:"admin", e:"editor", v:"viewer"}
---
user_roles: (a, e)
"#;
    let result = parse(input);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("removed"));
}

// ==================== Backward Compatibility Tests ====================

#[test]
fn test_parse_v10_document_still_works() {
    // Backward compatibility test
    let input = br#"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 |u1, Alice
 |u2, Bob
"#;
    let doc = parse(input).unwrap();

    // Verify v1.0 still parses correctly (version is preserved)
    assert_eq!(doc.version, (1, 0));
    let users = doc.root.get("users").expect("users key missing");
    match users {
        Item::List(matrix) => {
            assert_eq!(matrix.rows.len(), 2);
            assert_eq!(matrix.rows[0].fields.len(), 2);
            assert_eq!(matrix.rows[1].fields.len(), 2);
        }
        _ => panic!("Expected List for users"),
    }
}

// ==================== Edge Cases ====================

#[test]
fn test_parse_list_with_whitespace() {
    let input = br#"%VERSION: 1.1
---
items: ( a , b , c )
"#;
    let doc = parse(input).unwrap();

    let items = doc.root.get("items").expect("items key missing");
    match items {
        Item::Scalar(Value::List(list)) => {
            assert_eq!(list.len(), 3);
            assert!(matches!(&list[0], Value::String(s) if s.as_ref() == "a"));
            assert!(matches!(&list[1], Value::String(s) if s.as_ref() == "b"));
            assert!(matches!(&list[2], Value::String(s) if s.as_ref() == "c"));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_list_with_quoted_strings() {
    let input = br#"%VERSION: 1.1
---
messages: ("hello, world", "foo", "bar")
"#;
    let doc = parse(input).unwrap();

    let messages = doc.root.get("messages").expect("messages key missing");
    match messages {
        Item::Scalar(Value::List(list)) => {
            assert_eq!(list.len(), 3);
            assert!(matches!(&list[0], Value::String(s) if s.as_ref() == "hello, world"));
            assert!(matches!(&list[1], Value::String(s) if s.as_ref() == "foo"));
            assert!(matches!(&list[2], Value::String(s) if s.as_ref() == "bar"));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_list_single_element() {
    let input = br#"%VERSION: 1.1
---
single: (only)
"#;
    let doc = parse(input).unwrap();

    let single = doc.root.get("single").expect("single key missing");
    match single {
        Item::Scalar(Value::List(list)) => {
            assert_eq!(list.len(), 1);
            assert!(matches!(&list[0], Value::String(s) if s.as_ref() == "only"));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_list_with_expressions() {
    let input = br#"%VERSION: 1.1
---
calcs: ($(now()), $(x), $(concat(a, b)))
"#;
    let doc = parse(input).unwrap();

    let calcs = doc.root.get("calcs").expect("calcs key missing");
    match calcs {
        Item::Scalar(Value::List(list)) => {
            assert_eq!(list.len(), 3);
            assert!(matches!(&list[0], Value::Expression(_)));
            assert!(matches!(&list[1], Value::Expression(_)));
            assert!(matches!(&list[2], Value::Expression(_)));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_nested_lists_in_object() {
    let input = br#"%VERSION: 1.1
---
config:
 allowed: (read, write)
 denied: (delete, admin)
"#;
    let doc = parse(input).unwrap();

    let config = doc.root.get("config").expect("config key missing");
    match config {
        Item::Object(obj) => {
            let allowed = obj.get("allowed").expect("allowed key missing");
            assert!(matches!(allowed, Item::Scalar(Value::List(_))));

            let denied = obj.get("denied").expect("denied key missing");
            assert!(matches!(denied, Item::Scalar(Value::List(_))));
        }
        _ => panic!("Expected Object"),
    }
}

// ============================================================================
// NOTE: Nested list literals with parentheses like ((a, b), (c, d)) are NOT
// currently supported. The parser tracks parenthesis depth only for expressions.
// Lists CAN contain tensors which provide nested numeric arrays: ([1,2], [3,4])
// ============================================================================

// ==================== List in Matrix Tests ====================

#[test]
fn test_parse_matrix_with_list_and_tensor_cells() {
    let input = br#"%VERSION: 1.1
%STRUCT: Record: [id, tags, values]
---
data:@Record
 |r1, (tag1, tag2), [1, 2, 3]
 |r2, (tag3), [4, 5]
"#;
    let doc = parse(input).unwrap();

    let data = doc.root.get("data").expect("data key missing");
    match data {
        Item::List(matrix) => {
            assert_eq!(matrix.rows.len(), 2);

            // First row: list and tensor
            let row1 = &matrix.rows[0];
            assert!(matches!(&row1.fields[1], Value::List(tags) if tags.len() == 2));
            assert!(matches!(&row1.fields[2], Value::Tensor(_)));

            // Second row: list and tensor
            let row2 = &matrix.rows[1];
            assert!(matches!(&row2.fields[1], Value::List(tags) if tags.len() == 1));
            assert!(matches!(&row2.fields[2], Value::Tensor(_)));
        }
        _ => panic!("Expected List for data"),
    }
}

#[test]
fn test_parse_matrix_with_list_containing_tensors() {
    // Lists CAN contain tensors (nested numeric arrays)
    let input = br#"%VERSION: 1.1
%STRUCT: Record: [id, matrices]
---
data:@Record
 |r1, ([1, 2], [3, 4])
 |r2, ([5], [6, 7, 8])
"#;
    let doc = parse(input).unwrap();

    let data = doc.root.get("data").expect("data key missing");
    match data {
        Item::List(matrix) => {
            assert_eq!(matrix.rows.len(), 2);

            // First row: list containing two tensors
            match &matrix.rows[0].fields[1] {
                Value::List(items) => {
                    assert_eq!(items.len(), 2);
                    assert!(matches!(&items[0], Value::Tensor(_)));
                    assert!(matches!(&items[1], Value::Tensor(_)));
                }
                _ => panic!("Expected list of tensors in row 1"),
            }

            // Second row: list containing two tensors
            match &matrix.rows[1].fields[1] {
                Value::List(items) => {
                    assert_eq!(items.len(), 2);
                    assert!(matches!(&items[0], Value::Tensor(_)));
                    assert!(matches!(&items[1], Value::Tensor(_)));
                }
                _ => panic!("Expected list of tensors in row 2"),
            }
        }
        _ => panic!("Expected List for data"),
    }
}

// ==================== Empty String in List Tests ====================

#[test]
fn test_parse_list_with_empty_string_in_document() {
    let input = br#"%VERSION: 1.1
---
items: ("", filled, "")
"#;
    let doc = parse(input).unwrap();

    let items = doc.root.get("items").expect("items key missing");
    match items {
        Item::Scalar(Value::List(list)) => {
            assert_eq!(list.len(), 3);
            assert!(matches!(&list[0], Value::String(s) if s.is_empty()));
            assert!(matches!(&list[1], Value::String(s) if s.as_ref() == "filled"));
            assert!(matches!(&list[2], Value::String(s) if s.is_empty()));
        }
        _ => panic!("Expected List"),
    }
}

// ==================== Unicode in Lists Tests ====================

#[test]
fn test_parse_list_with_unicode_in_document() {
    let input = "%VERSION: 1.1\n---\nlanguages: (日本語, 中文, Русский)\n";
    let doc = parse(input.as_bytes()).unwrap();

    let languages = doc.root.get("languages").expect("languages key missing");
    match languages {
        Item::Scalar(Value::List(list)) => {
            assert_eq!(list.len(), 3);
            assert!(matches!(&list[0], Value::String(s) if s.as_ref() == "日本語"));
            assert!(matches!(&list[1], Value::String(s) if s.as_ref() == "中文"));
            assert!(matches!(&list[2], Value::String(s) if s.as_ref() == "Русский"));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_list_with_emoji_in_document() {
    let input = "%VERSION: 1.1\n---\nemojis: (😀, 🎉, 🚀)\n";
    let doc = parse(input.as_bytes()).unwrap();

    let emojis = doc.root.get("emojis").expect("emojis key missing");
    match emojis {
        Item::Scalar(Value::List(list)) => {
            assert_eq!(list.len(), 3);
            assert!(matches!(&list[0], Value::String(s) if s.as_ref() == "😀"));
            assert!(matches!(&list[1], Value::String(s) if s.as_ref() == "🎉"));
            assert!(matches!(&list[2], Value::String(s) if s.as_ref() == "🚀"));
        }
        _ => panic!("Expected List"),
    }
}

// ==================== Whitespace Handling Tests ====================

#[test]
fn test_parse_list_with_extensive_whitespace() {
    let input = br#"%VERSION: 1.1
---
spaced: (  a  ,  b  ,  c  )
"#;
    let doc = parse(input).unwrap();

    let spaced = doc.root.get("spaced").expect("spaced key missing");
    match spaced {
        Item::Scalar(Value::List(list)) => {
            assert_eq!(list.len(), 3);
            assert!(matches!(&list[0], Value::String(s) if s.as_ref() == "a"));
            assert!(matches!(&list[1], Value::String(s) if s.as_ref() == "b"));
            assert!(matches!(&list[2], Value::String(s) if s.as_ref() == "c"));
        }
        _ => panic!("Expected List"),
    }
}

// ==================== Error Cases ====================

#[test]
fn test_parse_unclosed_list_error() {
    let input = br#"%VERSION: 1.1
---
broken: (a, b, c
"#;
    let result = parse(input);
    assert!(result.is_err(), "Expected error for unclosed list");
    let err = result.unwrap_err();
    assert!(err.message.contains("unclosed") || err.message.contains("expected ')'"));
}

#[test]
fn test_parse_trailing_comma_in_list_error() {
    let input = br#"%VERSION: 1.1
---
bad: (a, b,)
"#;
    let result = parse(input);
    assert!(result.is_err(), "Expected error for trailing comma");
    let err = result.unwrap_err();
    assert!(err.message.contains("trailing comma") || err.message.contains("empty element"));
}

#[test]
fn test_parse_consecutive_commas_in_list_error() {
    let input = br#"%VERSION: 1.1
---
bad: (a,,b)
"#;
    let result = parse(input);
    assert!(result.is_err(), "Expected error for consecutive commas");
    let err = result.unwrap_err();
    assert!(err.message.contains("empty element") || err.message.contains("consecutive commas"));
}

// ==================== Multiple Lists in Document ====================

#[test]
fn test_parse_document_with_multiple_lists() {
    let input = br#"%VERSION: 1.1
---
permissions: (read, write, delete)
status_codes: (200, 404, 500)
flags: (true, false, true)
"#;
    let doc = parse(input).unwrap();

    // Verify all three lists parse correctly
    assert!(matches!(
        doc.root.get("permissions").unwrap(),
        Item::Scalar(Value::List(_))
    ));
    assert!(matches!(
        doc.root.get("status_codes").unwrap(),
        Item::Scalar(Value::List(_))
    ));
    assert!(matches!(
        doc.root.get("flags").unwrap(),
        Item::Scalar(Value::List(_))
    ));
}

#[test]
fn test_parse_list_preserves_value_types() {
    let input = br#"%VERSION: 1.1
---
mixed: (42, 3.5, true, false, ~, hello, @ref1)
"#;
    let doc = parse(input).unwrap();

    let mixed = doc.root.get("mixed").expect("mixed key missing");
    match mixed {
        Item::Scalar(Value::List(items)) => {
            assert_eq!(items.len(), 7);
            assert!(matches!(&items[0], Value::Int(42)));
            assert!(matches!(&items[1], Value::Float(f) if (*f - 3.5).abs() < 0.001));
            assert!(matches!(&items[2], Value::Bool(true)));
            assert!(matches!(&items[3], Value::Bool(false)));
            assert!(matches!(&items[4], Value::Null));
            assert!(matches!(&items[5], Value::String(s) if s.as_ref() == "hello"));
            assert!(matches!(&items[6], Value::Reference(_)));
        }
        _ => panic!("Expected List"),
    }
}
