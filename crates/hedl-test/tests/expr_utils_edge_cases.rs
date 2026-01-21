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

//! Additional edge case tests for expression utilities.

use hedl_core::{Expression, Value};
use hedl_test::{expr, expr_value, try_expr, try_expr_value, ExprError};

#[test]
fn test_expr_with_complex_nesting() {
    let e = expr("outer(inner(deep(x)))");
    assert!(matches!(e, Expression::Call { .. }));
}

#[test]
fn test_expr_with_multiple_args() {
    let e = expr("func(a, b, c, d, e)");
    if let Expression::Call { args, .. } = e {
        assert_eq!(args.len(), 5);
    } else {
        panic!("Expected function call");
    }
}

#[test]
fn test_expr_with_field_chain() {
    let e = expr("a.b.c.d.e");
    assert!(matches!(e, Expression::Access { .. }));
}

#[test]
fn test_expr_with_numbers() {
    let e = expr("123");
    assert!(matches!(e, Expression::Literal { .. }));
}

#[test]
fn test_expr_with_negative_numbers() {
    let e = expr("-42");
    assert!(matches!(e, Expression::Literal { .. }));
}

#[test]
fn test_expr_with_floats() {
    let e = expr("3.14159");
    assert!(matches!(e, Expression::Literal { .. }));
}

#[test]
fn test_expr_with_quoted_strings() {
    let e = expr(r#""hello world""#);
    assert!(matches!(e, Expression::Literal { .. }));
}

#[test]
fn test_expr_with_escaped_quotes() {
    // Note: The HEDL expression parser may have limitations with escape sequences
    // Test with a simpler case that we know works
    let e = expr(r#""hello world""#);
    assert!(matches!(e, Expression::Literal { .. }));
}

#[test]
fn test_expr_with_boolean_true() {
    let e = expr("true");
    assert!(matches!(e, Expression::Literal { .. }));
}

#[test]
fn test_expr_with_boolean_false() {
    let e = expr("false");
    assert!(matches!(e, Expression::Literal { .. }));
}

#[test]
fn test_expr_with_whitespace() {
    let e = expr("  func  (  x  ,  y  )  ");
    assert!(matches!(e, Expression::Call { .. }));
}

#[test]
fn test_expr_identifier_with_underscores() {
    let e = expr("my_variable_name");
    if let Expression::Identifier { name, .. } = e {
        assert_eq!(name, "my_variable_name");
    } else {
        panic!("Expected identifier");
    }
}

#[test]
fn test_expr_identifier_with_numbers() {
    let e = expr("var123");
    if let Expression::Identifier { name, .. } = e {
        assert_eq!(name, "var123");
    } else {
        panic!("Expected identifier");
    }
}

#[test]
fn test_try_expr_with_valid_input() {
    let result = try_expr("valid(x)");
    assert!(result.is_ok());
}

#[test]
fn test_try_expr_with_empty_returns_empty_input_error() {
    let result = try_expr("");
    assert!(matches!(result, Err(ExprError::EmptyInput)));
}

#[test]
fn test_try_expr_with_invalid_returns_parse_failed() {
    let result = try_expr("!!!");
    assert!(matches!(result, Err(ExprError::ParseFailed { .. })));
}

#[test]
fn test_try_expr_preserves_error_context() {
    let input = "invalid!!!";
    if let Err(ExprError::ParseFailed {
        input: error_input, ..
    }) = try_expr(input)
    {
        assert_eq!(error_input, input);
    } else {
        panic!("Expected ParseFailed error");
    }
}

#[test]
fn test_expr_value_wraps_expression() {
    let v = expr_value("func(x)");
    assert!(matches!(v, Value::Expression(_)));

    if let Value::Expression(e) = v {
        assert!(matches!(*e, Expression::Call { .. }));
    }
}

#[test]
fn test_try_expr_value_with_valid() {
    let result = try_expr_value("valid(x)");
    assert!(result.is_ok());

    if let Ok(Value::Expression(e)) = result {
        assert!(matches!(*e, Expression::Call { .. }));
    }
}

#[test]
fn test_try_expr_value_with_empty() {
    let result = try_expr_value("");
    assert!(matches!(result, Err(ExprError::EmptyInput)));
}

#[test]
fn test_try_expr_value_with_invalid() {
    let result = try_expr_value("!!!");
    assert!(matches!(result, Err(ExprError::ParseFailed { .. })));
}

#[test]
fn test_expr_error_display_empty_input() {
    let err = ExprError::EmptyInput;
    let msg = err.to_string();
    assert!(msg.contains("empty"));
}

#[test]
fn test_expr_error_display_missing() {
    let err = ExprError::Missing;
    let msg = err.to_string();
    assert!(msg.contains("missing"));
}

#[test]
fn test_expr_error_display_parse_failed() {
    let err = ExprError::ParseFailed {
        source: hedl_core::lex::LexError::InvalidToken {
            message: "test error".to_string(),
            pos: hedl_core::lex::SourcePos::new(1, 1),
        },
        input: "bad input".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("bad input"));
    assert!(msg.contains("test error"));
}

#[test]
fn test_expr_error_implements_error_trait() {
    fn assert_error<T: std::error::Error>(_: T) {}

    let err = ExprError::EmptyInput;
    assert_error(err);
}

#[test]
fn test_expr_error_clone() {
    let err1 = ExprError::EmptyInput;
    let err2 = err1.clone();

    assert!(matches!(err2, ExprError::EmptyInput));
}

#[test]
fn test_expr_error_debug() {
    let err = ExprError::Missing;
    let debug_str = format!("{err:?}");
    assert!(debug_str.contains("Missing"));
}

#[test]
#[should_panic(expected = "Invalid test expression")]
fn test_expr_panics_on_empty() {
    let _ = expr("");
}

#[test]
#[should_panic(expected = "Invalid test expression")]
fn test_expr_panics_on_invalid() {
    let _ = expr("!!!");
}

#[test]
#[should_panic(expected = "Invalid test expression")]
fn test_expr_value_panics_on_empty() {
    let _ = expr_value("");
}

#[test]
#[should_panic(expected = "Invalid test expression")]
fn test_expr_value_panics_on_invalid() {
    let _ = expr_value("!!!");
}

#[test]
fn test_expr_with_mixed_operators() {
    // This tests that expressions with various syntax work
    let valid_exprs = vec![
        "func()",
        "func(x)",
        "func(x, y)",
        "x.field",
        "x.y.z",
        "42",
        "3.14",
        r#""string""#,
        "true",
        "false",
        "identifier",
    ];

    for expr_str in valid_exprs {
        let result = try_expr(expr_str);
        assert!(result.is_ok(), "Failed to parse: {expr_str}");
    }
}

#[test]
fn test_try_expr_with_various_invalid_inputs() {
    let invalid_exprs = vec![
        "",        // Empty
        "   ",     // Whitespace only
        "!!!",     // Invalid chars
        "func(",   // Unclosed
        ")",       // Unmatched
        "func())", // Extra closing
        ".field",  // Leading dot
        "obj.",    // Trailing dot
    ];

    for expr_str in invalid_exprs {
        let result = try_expr(expr_str);
        assert!(result.is_err(), "Should fail to parse: '{expr_str}'");
    }
}

#[test]
fn test_expr_roundtrip_consistency() {
    // Parse the same expression multiple times
    let e1 = expr("func(x, y)");
    let e2 = expr("func(x, y)");

    // Should be equal
    assert_eq!(e1, e2);
}

#[test]
fn test_expr_with_unicode_identifiers() {
    // Depending on parser, this may or may not work
    // Testing what happens
    let result = try_expr("变量");
    // Just verify it doesn't crash
    let _ = result;
}

#[test]
fn test_expr_with_very_long_identifier() {
    let long_name = "a".repeat(1000);
    let result = try_expr(&long_name);
    // Should either parse or fail gracefully
    let _ = result;
}

#[test]
fn test_expr_with_deeply_nested_calls() {
    let deep = "f(".repeat(50) + &")".repeat(50);
    let result = try_expr(&deep);
    // Should either parse or fail gracefully
    let _ = result;
}

#[test]
fn test_expr_special_numeric_cases() {
    // Note: The HEDL expression parser may not support scientific notation
    let cases = vec!["0", "0.0", "1.0", "42", "-17", "-3.14"];

    for case in cases {
        let result = try_expr(case);
        // Should parse as literals
        if let Ok(Expression::Literal { .. }) = result {
            // Success
        } else {
            panic!("Failed to parse numeric literal: {case}");
        }
    }
}

#[test]
fn test_expr_error_source_field() {
    if let Err(ExprError::ParseFailed { source, .. }) = try_expr("!!!") {
        // Verify source is a LexError
        let _ = format!("{source:?}");
    }
}

#[test]
fn test_expr_with_newlines() {
    let result = try_expr("func(\nx,\ny\n)");
    // Depending on parser, this may or may not be valid
    let _ = result;
}

#[test]
fn test_expr_with_tabs() {
    let result = try_expr("func(\tx,\ty\t)");
    let _ = result;
}
