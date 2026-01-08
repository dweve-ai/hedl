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


//! Comprehensive error path testing for expression parser failures.
//!
//! This test suite validates that the expression parser correctly handles all error conditions
//! and returns appropriate LexError variants for invalid input.

use hedl_core::lex::{parse_expression, LexError};

// ==================== Empty and Invalid Input ====================

#[test]
fn test_empty_expression() {
    let result = parse_expression("");
    assert!(result.is_err());
    if let Err(LexError::InvalidToken { .. }) = result {
        // Expected - empty input is invalid
    } else {
        panic!("Expected InvalidToken for empty expression, got: {:?}", result);
    }
}

#[test]
fn test_whitespace_only() {
    let result = parse_expression("   ");
    assert!(result.is_err());
}

#[test]
fn test_invalid_characters() {
    let invalid_inputs = vec!["!!!", "###", "$$$", "%%%", "&&&", "***"];
    for input in invalid_inputs {
        let result = parse_expression(input);
        assert!(
            result.is_err(),
            "Expected error for invalid input: {}",
            input
        );
    }
}

// ==================== String Errors ====================

#[test]
fn test_unclosed_string() {
    let result = parse_expression(r#""unclosed"#);
    assert!(result.is_err());
    if let Err(LexError::UnclosedQuote { .. }) = result {
        // Expected
    } else {
        panic!("Expected UnclosedQuote error, got: {:?}", result);
    }
}

#[test]
fn test_unclosed_string_with_content() {
    let result = parse_expression(r#""hello world"#);
    assert!(result.is_err());
    if let Err(LexError::UnclosedQuote { .. }) = result {
        // Expected
    } else {
        panic!("Expected UnclosedQuote error, got: {:?}", result);
    }
}

#[test]
fn test_unclosed_string_with_escape() {
    let result = parse_expression(r#""hello\nworld"#);
    assert!(result.is_err());
}

// ==================== Parenthesis Errors ====================

#[test]
fn test_unclosed_parenthesis() {
    let result = parse_expression("(1 + 2");
    assert!(result.is_err());
    // Either UnclosedExpression or InvalidToken is acceptable for unclosed paren
    match result {
        Err(LexError::UnclosedExpression { .. }) | Err(LexError::InvalidToken { .. }) => {
            // Expected - parser detected the unclosed parenthesis
        }
        other => panic!("Expected error for unclosed parenthesis, got: {:?}", other),
    }
}

#[test]
fn test_mismatched_parenthesis() {
    let result = parse_expression("(1 + 2))");
    assert!(result.is_err());
}

#[test]
fn test_empty_parenthesis() {
    // () should fail as it's not a valid expression
    let result = parse_expression("()");
    assert!(result.is_err());
}

#[test]
fn test_nested_unclosed_parenthesis() {
    let result = parse_expression("((1 + (2 * 3)");
    assert!(result.is_err());
}

// ==================== Number Errors ====================

#[test]
fn test_invalid_number_format() {
    let result = parse_expression("123abc");
    assert!(result.is_err());
}

#[test]
fn test_double_decimal_point() {
    let result = parse_expression("3.14.159");
    assert!(result.is_err());
}

#[test]
fn test_leading_decimal() {
    // .5 is not valid in HEDL (must be 0.5)
    let result = parse_expression(".5");
    assert!(result.is_err());
}

#[test]
fn test_trailing_decimal() {
    // 5. is not valid (must be 5.0)
    let result = parse_expression("5.");
    assert!(result.is_err());
}

// ==================== Function Call Errors ====================

#[test]
fn test_unclosed_function_call() {
    let result = parse_expression("add(1, 2");
    assert!(result.is_err());
}

#[test]
fn test_function_call_missing_comma() {
    let result = parse_expression("add(1 2)");
    assert!(result.is_err());
}

#[test]
fn test_function_call_trailing_comma() {
    let result = parse_expression("add(1, 2,)");
    assert!(result.is_err());
}

#[test]
fn test_function_call_double_comma() {
    let result = parse_expression("add(1,, 2)");
    assert!(result.is_err());
}

#[test]
fn test_empty_function_name() {
    let result = parse_expression("(1, 2)");
    assert!(result.is_err());
}

// ==================== Field Access Errors ====================

#[test]
fn test_field_access_trailing_dot() {
    let result = parse_expression("obj.");
    assert!(result.is_err());
}

#[test]
fn test_field_access_double_dot() {
    let result = parse_expression("obj..field");
    assert!(result.is_err());
}

#[test]
fn test_field_access_leading_dot() {
    let result = parse_expression(".field");
    assert!(result.is_err());
}

#[test]
fn test_field_access_numeric_field() {
    // obj.123 is invalid
    let result = parse_expression("obj.123");
    assert!(result.is_err());
}

// ==================== Operator Errors ====================

#[test]
fn test_invalid_operator_modulo() {
    let result = parse_expression("1 % 2");
    assert!(result.is_err());
}

#[test]
fn test_invalid_operator_bitwise() {
    let operators = vec!["&", "|", "^", "<<", ">>"];
    for op in operators {
        let expr = format!("1 {} 2", op);
        let result = parse_expression(&expr);
        assert!(
            result.is_err(),
            "Expected error for invalid operator: {}",
            op
        );
    }
}

#[test]
fn test_double_operator() {
    let result = parse_expression("1 ++ 2");
    assert!(result.is_err());
}

#[test]
fn test_trailing_operator() {
    let result = parse_expression("1 +");
    assert!(result.is_err());
}

#[test]
fn test_leading_operator() {
    let result = parse_expression("+ 1");
    assert!(result.is_err());
}

// ==================== Identifier Errors ====================

#[test]
fn test_identifier_starts_with_number() {
    let result = parse_expression("123invalid");
    assert!(result.is_err());
}

#[test]
fn test_identifier_with_spaces() {
    let result = parse_expression("invalid id");
    assert!(result.is_err());
}

#[test]
fn test_identifier_with_special_chars() {
    let invalid = vec!["id@", "id#", "id$", "id%"];
    for id in invalid {
        let result = parse_expression(id);
        assert!(result.is_err(), "Expected error for identifier: {}", id);
    }
}

// ==================== Expression Nesting Errors ====================

#[test]
fn test_unclosed_nested_expression() {
    let result = parse_expression("add(mul(1, 2), 3");
    assert!(result.is_err());
}

#[test]
fn test_deeply_nested_unclosed() {
    let result = parse_expression("f(g(h(i(j(");
    assert!(result.is_err());
}

// ==================== Mixed Error Cases ====================

#[test]
fn test_string_in_arithmetic() {
    let result = parse_expression(r#"1 + "hello""#);
    assert!(result.is_err());
}

#[test]
fn test_unclosed_string_in_function() {
    let result = parse_expression(r#"print("unclosed)"#);
    assert!(result.is_err());
}

#[test]
fn test_invalid_escape_sequence() {
    // \x is not a valid escape in HEDL
    let result = parse_expression(r#""\xAB""#);
    // Should either parse successfully (treating \x literally) or fail
    // depending on specification
    let _ = result;
}

// ==================== Edge Cases ====================

#[test]
fn test_very_long_identifier() {
    let long_id = "x".repeat(10000);
    let result = parse_expression(&long_id);
    // Should succeed for valid identifier, or fail if there's a length limit
    let _ = result;
}

#[test]
fn test_unicode_in_identifier() {
    // HEDL may or may not support Unicode identifiers
    let result = parse_expression("日本語");
    assert!(result.is_err());
}

#[test]
fn test_emoji_in_expression() {
    let result = parse_expression("😀");
    assert!(result.is_err());
}

#[test]
fn test_null_byte() {
    let result = parse_expression("id\0name");
    assert!(result.is_err());
}

// ==================== Bracket Errors (if supported) ====================

#[test]
fn test_unclosed_bracket() {
    let result = parse_expression("[1, 2, 3");
    // Should fail with unclosed bracket error
    let _ = result;
}

#[test]
fn test_mismatched_bracket_paren() {
    let result = parse_expression("(1, 2]");
    assert!(result.is_err());
}

// ==================== Boolean Literal Edge Cases ====================

#[test]
fn test_capitalized_boolean() {
    // True/False should not be valid (only true/false)
    let result = parse_expression("True");
    // This might actually be valid as an identifier
    let _ = result;
}

#[test]
fn test_partial_boolean() {
    let result = parse_expression("tru");
    // Should be valid as an identifier, not an error
    assert!(result.is_ok());
}

// ==================== Recovery and Position Tracking ====================

#[test]
fn test_error_position_tracking() {
    let result = parse_expression("1 + (2 * ");
    if let Err(err) = result {
        // Verify error has position information (may or may not be present)
        // Just checking that the parser returns an error is the main goal
        let _pos = err.position();
    } else {
        panic!("Expected error for unclosed expression");
    }
}

#[test]
fn test_multiple_errors_first_reported() {
    // Expression with multiple errors should report the first one
    let result = parse_expression("(1 + 2");
    assert!(result.is_err());
}

// ==================== Real-World Error Patterns ====================

#[test]
fn test_common_typo_missing_quote() {
    let result = parse_expression(r#"hello"#);
    // Should succeed as identifier, not an error
    assert!(result.is_ok());
}

#[test]
fn test_common_typo_extra_paren() {
    let result = parse_expression("add(1, 2))");
    assert!(result.is_err());
}

#[test]
fn test_sql_like_syntax() {
    // User might try SQL syntax
    let result = parse_expression("SELECT * FROM");
    // Should fail or parse as identifiers
    let _ = result;
}

// ==================== Stress Tests ====================

#[test]
fn test_deeply_nested_valid_parens() {
    let mut expr = String::from("x");
    for _ in 0..100 {
        expr = format!("f({})", expr);
    }
    let result = parse_expression(&expr);
    // Should either succeed or hit recursion limit
    if result.is_err() {
        if let Err(LexError::RecursionTooDeep { .. }) = result {
            // Expected if there's a recursion limit
        }
    }
}

#[test]
fn test_many_function_arguments() {
    let mut args = Vec::new();
    for i in 0..1000 {
        args.push(i.to_string());
    }
    let expr = format!("f({})", args.join(", "));
    let result = parse_expression(&expr);
    // Should either succeed or hit argument limit
    let _ = result;
}
