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

//! Tests for list literal parsing in HEDL v1.1
//!
//! This module tests the parsing of list literals using parentheses syntax: `(...)`.
//! Lists can contain any scalar values including strings, bools, numbers, references,
//! expressions, and nulls, unlike tensors which are numeric-only.

use hedl_core::lex::{parse_list_literal, Value};

// ============================================================================
// Empty List Tests
// ============================================================================

#[test]
fn test_parse_empty_list_returns_empty_vec() {
    let (val, consumed) = parse_list_literal("()", 0).unwrap();
    match val {
        Value::List(items) => assert!(items.is_empty()),
        _ => panic!("Expected Value::List"),
    }
    assert_eq!(consumed, 2);
}

#[test]
fn test_parse_empty_list_with_whitespace_inside() {
    let (val, consumed) = parse_list_literal("(  )", 0).unwrap();
    match val {
        Value::List(items) => assert!(items.is_empty()),
        _ => panic!("Expected Value::List"),
    }
    assert_eq!(consumed, 4);
}

#[test]
fn test_parse_empty_list_with_tabs_inside() {
    let (val, _) = parse_list_literal("(\t\t)", 0).unwrap();
    match val {
        Value::List(items) => assert!(items.is_empty()),
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Single Element Tests
// ============================================================================

#[test]
fn test_parse_list_with_single_unquoted_string() {
    let (val, _) = parse_list_literal("(admin)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(&items[0], Value::String(s) if s == "admin"));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_single_quoted_string() {
    let (val, _) = parse_list_literal("(\"hello world\")", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(&items[0], Value::String(s) if s == "hello world"));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_single_integer() {
    let (val, _) = parse_list_literal("(42)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(items[0], Value::Int(42)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_single_negative_integer() {
    let (val, _) = parse_list_literal("(-123)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(items[0], Value::Int(-123)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_single_float() {
    let (val, _) = parse_list_literal("(4.56)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(items[0], Value::Float(f) if (f - 4.56).abs() < 0.001));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_single_boolean_true() {
    let (val, _) = parse_list_literal("(true)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(items[0], Value::Bool(true)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_single_boolean_false() {
    let (val, _) = parse_list_literal("(false)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(items[0], Value::Bool(false)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_single_null() {
    let (val, _) = parse_list_literal("(~)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(items[0], Value::Null));
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Multiple Element Tests
// ============================================================================

#[test]
fn test_parse_list_with_multiple_strings() {
    let (val, _) = parse_list_literal("(admin, editor, viewer)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s == "admin"));
            assert!(matches!(&items[1], Value::String(s) if s == "editor"));
            assert!(matches!(&items[2], Value::String(s) if s == "viewer"));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_multiple_quoted_strings() {
    let (val, _) = parse_list_literal("(\"hello\", \"world\", \"test\")", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s == "hello"));
            assert!(matches!(&items[1], Value::String(s) if s == "world"));
            assert!(matches!(&items[2], Value::String(s) if s == "test"));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_multiple_integers() {
    let (val, _) = parse_list_literal("(1, 2, 3, 4, 5)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 5);
            assert!(matches!(items[0], Value::Int(1)));
            assert!(matches!(items[1], Value::Int(2)));
            assert!(matches!(items[2], Value::Int(3)));
            assert!(matches!(items[3], Value::Int(4)));
            assert!(matches!(items[4], Value::Int(5)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_multiple_floats() {
    let (val, _) = parse_list_literal("(1.1, 2.2, 3.3)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], Value::Float(f) if (f - 1.1).abs() < 0.001));
            assert!(matches!(items[1], Value::Float(f) if (f - 2.2).abs() < 0.001));
            assert!(matches!(items[2], Value::Float(f) if (f - 3.3).abs() < 0.001));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_multiple_booleans() {
    let (val, _) = parse_list_literal("(true, false, true)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], Value::Bool(true)));
            assert!(matches!(items[1], Value::Bool(false)));
            assert!(matches!(items[2], Value::Bool(true)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_mixed_types() {
    let (val, _) = parse_list_literal("(1, \"two\", true, ~)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 4);
            assert!(matches!(items[0], Value::Int(1)));
            assert!(matches!(&items[1], Value::String(s) if s == "two"));
            assert!(matches!(items[2], Value::Bool(true)));
            assert!(matches!(items[3], Value::Null));
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Whitespace Handling Tests
// ============================================================================

#[test]
fn test_parse_list_with_no_spaces_after_commas() {
    let (val, _) = parse_list_literal("(a,b,c)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s == "a"));
            assert!(matches!(&items[1], Value::String(s) if s == "b"));
            assert!(matches!(&items[2], Value::String(s) if s == "c"));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_extra_spaces() {
    let (val, _) = parse_list_literal("(  a  ,  b  ,  c  )", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s == "a"));
            assert!(matches!(&items[1], Value::String(s) if s == "b"));
            assert!(matches!(&items[2], Value::String(s) if s == "c"));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_newlines_between_elements() {
    let (val, _) = parse_list_literal("(a,\nb,\nc)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Reference Tests
// ============================================================================

#[test]
fn test_parse_list_with_single_reference() {
    let (val, _) = parse_list_literal("(@user1)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(&items[0], Value::Reference(_)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_multiple_references() {
    let (val, _) = parse_list_literal("(@user1, @user2)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], Value::Reference(_)));
            assert!(matches!(&items[1], Value::Reference(_)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_qualified_references() {
    let (val, _) = parse_list_literal("(@User:user1, @Post:post2)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], Value::Reference(_)));
            assert!(matches!(&items[1], Value::Reference(_)));
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Expression Tests
// ============================================================================

#[test]
fn test_parse_list_with_single_expression() {
    let (val, _) = parse_list_literal("($(now()))", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(&items[0], Value::Expression(_)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_multiple_expressions() {
    let (val, _) = parse_list_literal("($(now()), $(concat(a, b)))", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], Value::Expression(_)));
            assert!(matches!(&items[1], Value::Expression(_)));
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Nested Structure Tests
// ============================================================================

#[test]
fn test_parse_list_with_tensor_element() {
    let (val, _) = parse_list_literal("([1, 2, 3], test)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], Value::Tensor(_)));
            assert!(matches!(&items[1], Value::String(s) if s == "test"));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_multiple_tensors() {
    let (val, _) = parse_list_literal("([1, 2], [3, 4], [5, 6])", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::Tensor(_)));
            assert!(matches!(&items[1], Value::Tensor(_)));
            assert!(matches!(&items[2], Value::Tensor(_)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_nested_tensor() {
    let (val, _) = parse_list_literal("([[1, 2], [3, 4]])", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(&items[0], Value::Tensor(_)));
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Special Character Tests
// ============================================================================

#[test]
fn test_parse_list_with_strings_containing_commas() {
    let (val, _) = parse_list_literal("(\"hello, world\", \"foo, bar\")", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], Value::String(s) if s == "hello, world"));
            assert!(matches!(&items[1], Value::String(s) if s == "foo, bar"));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_strings_containing_parentheses() {
    let (val, _) = parse_list_literal("(\"test (a)\", \"other (b)\")", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], Value::String(s) if s == "test (a)"));
            assert!(matches!(&items[1], Value::String(s) if s == "other (b)"));
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Unicode Tests
// ============================================================================

#[test]
fn test_parse_list_with_unicode_strings() {
    let (val, _) = parse_list_literal("(日本語, Émoji, Тест)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s == "日本語"));
            assert!(matches!(&items[1], Value::String(s) if s == "Émoji"));
            assert!(matches!(&items[2], Value::String(s) if s == "Тест"));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_emoji() {
    let (val, _) = parse_list_literal("(\"😀\", \"🎉\", \"🚀\")", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Error Cases
// ============================================================================

#[test]
fn test_parse_list_missing_closing_paren_returns_error() {
    let result = parse_list_literal("(a, b", 0);
    assert!(result.is_err());
}

#[test]
fn test_parse_list_missing_opening_paren_returns_error() {
    let result = parse_list_literal("a, b)", 0);
    assert!(result.is_err());
}

#[test]
fn test_parse_list_with_trailing_comma_returns_error() {
    let result = parse_list_literal("(a, b,)", 0);
    assert!(result.is_err());
}

#[test]
fn test_parse_list_with_leading_comma_returns_error() {
    let result = parse_list_literal("(,a, b)", 0);
    assert!(result.is_err());
}

#[test]
fn test_parse_list_with_consecutive_commas_returns_error() {
    let result = parse_list_literal("(a,,b)", 0);
    assert!(result.is_err());
}

#[test]
fn test_parse_list_with_only_comma_returns_error() {
    let result = parse_list_literal("(,)", 0);
    assert!(result.is_err());
}

#[test]
fn test_parse_list_with_double_comma_returns_error() {
    let result = parse_list_literal("(a,,b)", 0);
    assert!(result.is_err());
}

#[test]
fn test_parse_list_with_unmatched_bracket_returns_error() {
    let result = parse_list_literal("(a, ])", 0);
    assert!(result.is_err());
}

// ============================================================================
// Position Tracking Tests
// ============================================================================

#[test]
fn test_parse_list_returns_correct_consumed_bytes() {
    let (_, consumed) = parse_list_literal("(a, b, c) extra", 0).unwrap();
    assert_eq!(consumed, 9);
}

#[test]
fn test_parse_list_returns_correct_consumed_bytes_with_quotes() {
    let (_, consumed) = parse_list_literal("(\"hello\", \"world\")", 0).unwrap();
    assert_eq!(consumed, 18);
}

#[test]
fn test_parse_list_from_offset_position() {
    let input = "prefix (a, b)";
    let (val, consumed) = parse_list_literal(input, 7).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("Expected Value::List"),
    }
    assert_eq!(consumed, 6);
}

// ============================================================================
// Large List Tests
// ============================================================================

#[test]
fn test_parse_list_with_10_elements() {
    let (val, _) = parse_list_literal("(a, b, c, d, e, f, g, h, i, j)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 10);
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_50_elements() {
    let elements: Vec<String> = (0..50).map(|i| i.to_string()).collect();
    let list_str = format!("({})", elements.join(", "));
    let (val, _) = parse_list_literal(&list_str, 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 50);
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_100_elements() {
    let elements: Vec<String> = (0..100).map(|i| i.to_string()).collect();
    let list_str = format!("({})", elements.join(", "));
    let (val, _) = parse_list_literal(&list_str, 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 100);
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Scientific Notation Tests
// ============================================================================

#[test]
fn test_parse_list_with_scientific_notation_positive_exp() {
    let (val, _) = parse_list_literal("(1e10, 2e5, 3e3)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], Value::Float(_)));
            assert!(matches!(items[1], Value::Float(_)));
            assert!(matches!(items[2], Value::Float(_)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_scientific_notation_negative_exp() {
    let (val, _) = parse_list_literal("(1e-10, 2e-5, 3e-3)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], Value::Float(_)));
            assert!(matches!(items[1], Value::Float(_)));
            assert!(matches!(items[2], Value::Float(_)));
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Negative Number Tests
// ============================================================================

#[test]
fn test_parse_list_with_negative_integers() {
    let (val, _) = parse_list_literal("(-1, -2, -3)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], Value::Int(-1)));
            assert!(matches!(items[1], Value::Int(-2)));
            assert!(matches!(items[2], Value::Int(-3)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_negative_floats() {
    let (val, _) = parse_list_literal("(-1.5, -2.5, -3.5)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(items[0], Value::Float(f) if (f + 1.5).abs() < 0.001));
            assert!(matches!(items[1], Value::Float(f) if (f + 2.5).abs() < 0.001));
            assert!(matches!(items[2], Value::Float(f) if (f + 3.5).abs() < 0.001));
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Mixed Negative and Positive Tests
// ============================================================================

#[test]
fn test_parse_list_with_mixed_sign_numbers() {
    let (val, _) = parse_list_literal("(-1, 2, -3, 4)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 4);
            assert!(matches!(items[0], Value::Int(-1)));
            assert!(matches!(items[1], Value::Int(2)));
            assert!(matches!(items[2], Value::Int(-3)));
            assert!(matches!(items[3], Value::Int(4)));
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Integration with Value Inference Tests
// ============================================================================

#[test]
fn test_list_value_can_be_inferred_from_string() {
    use hedl_core::lex::infer_value;

    let val = infer_value("(a, b, c)", None).unwrap();
    assert!(matches!(val, Value::List(_)));
    if let Value::List(items) = val {
        assert_eq!(items.len(), 3);
    }
}

#[test]
fn test_list_value_inference_priority_over_string() {
    use hedl_core::lex::infer_value;

    // (test) should be a list, not a string
    let val = infer_value("(test)", None).unwrap();
    assert!(matches!(val, Value::List(_)));
}

#[test]
fn test_list_value_with_mixed_types_via_inference() {
    use hedl_core::lex::infer_value;

    let val = infer_value("(42, \"text\", true, ~, @ref)", None).unwrap();
    if let Value::List(items) = val {
        assert_eq!(items.len(), 5);
        assert!(matches!(items[0], Value::Int(42)));
        assert!(matches!(&items[1], Value::String(s) if s == "text"));
        assert!(matches!(items[2], Value::Bool(true)));
        assert!(matches!(items[3], Value::Null));
        assert!(matches!(&items[4], Value::Reference(_)));
    } else {
        panic!("Expected Value::List");
    }
}

// ============================================================================
// Value Pattern Matching Tests
// ============================================================================

#[test]
fn test_list_value_construction() {
    let val = Value::List(vec![]);
    assert!(matches!(val, Value::List(_)));
}

#[test]
fn test_list_value_with_elements() {
    let items = vec![Value::Int(1), Value::Int(2)];
    let val = Value::List(items.clone());

    if let Value::List(list) = val {
        assert_eq!(list.len(), 2);
    } else {
        panic!("Expected Value::List");
    }
}

#[test]
fn test_value_discriminant_checking() {
    let val = Value::List(vec![]);
    assert!(matches!(val, Value::List(_)));
    assert!(!matches!(val, Value::Null));
    assert!(!matches!(val, Value::Int(_)));
}

// ============================================================================
// NOTE: Nested list literals with parentheses are NOT currently supported
// The parser only tracks parenthesis depth for expressions `$(...)`, not
// for nested lists. Lists can contain tensors `[...]` which ARE nested structures.
// ============================================================================

// ============================================================================
// Empty String Tests
// ============================================================================

#[test]
fn test_parse_list_with_empty_string_quoted() {
    let (val, _) = parse_list_literal("(\"\", other)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 2);
            assert!(matches!(&items[0], Value::String(s) if s.is_empty()));
            assert!(matches!(&items[1], Value::String(s) if s == "other"));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_only_empty_string() {
    let (val, _) = parse_list_literal("(\"\")", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 1);
            assert!(matches!(&items[0], Value::String(s) if s.is_empty()));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_with_multiple_empty_strings() {
    let (val, _) = parse_list_literal("(\"\", \"\", \"\")", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            for item in items.iter() {
                assert!(matches!(item, Value::String(s) if s.is_empty()));
            }
        }
        _ => panic!("Expected Value::List"),
    }
}

// ============================================================================
// Quoted String Behavior Tests
// ============================================================================
// NOTE: Current implementation strips quotes and then infers the value type.
// This means "123" becomes Int(123), not String("123"). This is consistent
// with HEDL's philosophy that quotes are just a way to include special chars,
// not a type indicator. To get a string that looks like a number in the final
// output, users should use quoted strings in the key-value context where the
// inference happens at a different level.
// ============================================================================

#[test]
fn test_parse_list_quoted_values_are_inferred() {
    let (val, _) = parse_list_literal("(\"123\", \"45.6\", 789)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            // Quoted numbers are stripped and inferred: "123" -> 123 -> Int(123)
            assert!(matches!(&items[0], Value::Int(123)));
            assert!(matches!(&items[1], Value::Float(f) if (*f - 45.6).abs() < 0.001));
            // Unquoted number is also Int
            assert!(matches!(&items[2], Value::Int(789)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_quoted_booleans_are_inferred() {
    let (val, _) = parse_list_literal("(\"true\", \"false\", true, false)", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 4);
            // Quoted booleans are stripped and inferred: "true" -> true -> Bool(true)
            assert!(matches!(&items[0], Value::Bool(true)));
            assert!(matches!(&items[1], Value::Bool(false)));
            // Unquoted booleans are also Bool
            assert!(matches!(&items[2], Value::Bool(true)));
            assert!(matches!(&items[3], Value::Bool(false)));
        }
        _ => panic!("Expected Value::List"),
    }
}

#[test]
fn test_parse_list_quoted_strings_that_need_quotes() {
    // Quotes ARE preserved when they're necessary (e.g., for spaces, commas)
    let (val, _) = parse_list_literal("(\"hello world\", \"a,b\", \"normal\")", 0).unwrap();
    match val {
        Value::List(items) => {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s == "hello world"));
            assert!(matches!(&items[1], Value::String(s) if s == "a,b"));
            // "normal" becomes normal -> String("normal")
            assert!(matches!(&items[2], Value::String(s) if s == "normal"));
        }
        _ => panic!("Expected Value::List"),
    }
}
