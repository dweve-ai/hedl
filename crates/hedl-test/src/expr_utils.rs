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

//! Expression utility functions.
//!
//! Helper functions for creating expressions in test fixtures.
//!
//! This module provides both panic-on-error and safe error-returning variants
//! for expression parsing. The `expr()` function panics on invalid input (useful
//! for concise test code), while `try_expr()` returns a `Result` for more
//! explicit error handling.

use hedl_core::{Expression, Value};
use hedl_core::lex::{parse_expression, LexError};
use std::fmt;

/// Error type for expression parsing failures.
///
/// This enum captures different categories of expression parsing errors
/// and provides descriptive error information for proper error handling.
///
/// # Examples
///
/// ```
/// use hedl_test::{try_expr, ExprError};
///
/// // Handle invalid syntax
/// match try_expr("invalid syntax!") {
///     Err(ExprError::ParseFailed { source, input }) => {
///         println!("Failed to parse: {}", input);
///         println!("Reason: {}", source);
///     }
///     Ok(expr) => println!("Success"),
///     _ => {}
/// }
///
/// // Handle empty input
/// match try_expr("") {
///     Err(ExprError::EmptyInput) => {
///         println!("Expression cannot be empty");
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone)]
pub enum ExprError {
    /// Expression string is empty.
    EmptyInput,

    /// Expression parsing failed with a lexical error.
    ///
    /// Contains the underlying `LexError` and the original input string
    /// for context in error reporting.
    ParseFailed {
        /// The underlying lexical error from the parser.
        source: LexError,
        /// The original input string that failed to parse.
        input: String,
    },

    /// Expression is missing (null or not provided).
    ///
    /// Used when an expression is expected but not found.
    Missing,
}

impl fmt::Display for ExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprError::EmptyInput => {
                write!(f, "Expression cannot be empty")
            }
            ExprError::ParseFailed { source, input } => {
                write!(
                    f,
                    "Failed to parse expression '{}': {}",
                    input, source
                )
            }
            ExprError::Missing => {
                write!(f, "Expression is missing or null")
            }
        }
    }
}

impl std::error::Error for ExprError {}

/// Helper to create an Expression from a valid expression string.
///
/// This function panics on invalid input, making it convenient for test code
/// where a concise failure is preferred over explicit error handling.
///
/// For error handling, use [`try_expr()`] instead.
///
/// # Panics
///
/// Panics if the expression string cannot be parsed, with a message showing
/// the invalid input.
///
/// # Example
///
/// ```
/// use hedl_test::expr;
///
/// // Simple literals
/// let e = expr("42");
/// let e = expr("3.14");
/// let e = expr("\"hello\"");
/// let e = expr("true");
///
/// // Identifiers
/// let e = expr("x");
/// let e = expr("my_variable");
///
/// // Function calls
/// let e = expr("now()");
/// let e = expr("add(x, y)");
/// let e = expr("max(1, 2, 3)");
///
/// // Field access
/// let e = expr("user.name");
/// let e = expr("data.values.first");
/// ```
pub fn expr(s: &str) -> Expression {
    try_expr(s).unwrap_or_else(|e| panic!("Invalid test expression: {}", e))
}

/// Safe variant of `expr()` that returns a `Result` instead of panicking.
///
/// This function provides explicit error handling for expression parsing,
/// returning `Ok(Expression)` on success or `Err(ExprError)` on failure.
///
/// Use this when you need to handle invalid expressions gracefully, such as
/// when processing user input or testing error conditions.
///
/// # Errors
///
/// Returns `Err(ExprError::EmptyInput)` if the expression string is empty.
///
/// Returns `Err(ExprError::ParseFailed)` if the expression string contains
/// invalid syntax or fails to parse for any reason.
///
/// # Examples
///
/// ## Basic usage
///
/// ```
/// use hedl_test::try_expr;
///
/// // Success cases
/// assert!(try_expr("42").is_ok());
/// assert!(try_expr("foo()").is_ok());
/// assert!(try_expr("x.y.z").is_ok());
///
/// // Failure cases
/// assert!(try_expr("").is_err());
/// assert!(try_expr("!@#$%").is_err());
/// ```
///
/// ## Error handling
///
/// ```
/// use hedl_test::{try_expr, ExprError};
///
/// match try_expr("invalid!!!") {
///     Ok(expr) => {
///         println!("Parsed: {:?}", expr);
///     }
///     Err(ExprError::EmptyInput) => {
///         println!("Empty expressions are not allowed");
///     }
///     Err(ExprError::ParseFailed { source, input }) => {
///         eprintln!("Failed to parse '{}': {}", input, source);
///     }
///     Err(ExprError::Missing) => {
///         println!("Expression was missing");
///     }
/// }
/// ```
///
/// ## Testing error conditions
///
/// ```
/// use hedl_test::try_expr;
///
/// // You can test error handling without panics
/// let inputs = vec!["", "!!!", ")(", "func("];
/// for input in inputs {
///     match try_expr(input) {
///         Err(_) => println!("'{}' correctly failed", input),
///         Ok(_) => panic!("'{}' should have failed", input),
///     }
/// }
/// ```
pub fn try_expr(s: &str) -> Result<Expression, ExprError> {
    if s.is_empty() {
        return Err(ExprError::EmptyInput);
    }

    parse_expression(s).map_err(|source| ExprError::ParseFailed {
        source,
        input: s.to_string(),
    })
}

/// Helper to create a Value::Expression from a valid expression string.
///
/// This function panics on invalid input. Use [`try_expr_value()`] for
/// error handling.
///
/// # Examples
///
/// ```
/// use hedl_test::expr_value;
/// use hedl_core::Value;
///
/// let v = expr_value("42");
/// let v = expr_value("foo()");
/// let v = expr_value("x.y");
/// ```
pub fn expr_value(s: &str) -> Value {
    Value::Expression(expr(s))
}

/// Safe variant of `expr_value()` that returns a `Result` instead of panicking.
///
/// This function provides explicit error handling for creating expression values,
/// returning `Ok(Value)` on success or `Err(ExprError)` on failure.
///
/// # Errors
///
/// Returns the same errors as [`try_expr()`].
///
/// # Examples
///
/// ```
/// use hedl_test::{try_expr_value, ExprError};
/// use hedl_core::Value;
///
/// match try_expr_value("my_func()") {
///     Ok(Value::Expression(_)) => println!("Success"),
///     Err(e) => println!("Failed: {}", e),
///     _ => {}
/// }
/// ```
pub fn try_expr_value(s: &str) -> Result<Value, ExprError> {
    try_expr(s).map(Value::Expression)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_literals() {
        assert!(expr("42").is_literal());
        assert!(expr("3.14").is_literal());
        assert!(expr("\"hello\"").is_literal());
        assert!(expr("true").is_literal());
        assert!(expr("false").is_literal());
    }

    #[test]
    fn test_expr_identifiers() {
        match expr("x") {
            Expression::Identifier { name, .. } => assert_eq!(name, "x"),
            _ => panic!("Expected identifier"),
        }

        match expr("my_var") {
            Expression::Identifier { name, .. } => assert_eq!(name, "my_var"),
            _ => panic!("Expected identifier"),
        }
    }

    #[test]
    fn test_expr_function_calls() {
        match expr("now()") {
            Expression::Call { name, args, .. } => {
                assert_eq!(name, "now");
                assert_eq!(args.len(), 0);
            }
            _ => panic!("Expected call"),
        }

        match expr("add(x, y)") {
            Expression::Call { name, args, .. } => {
                assert_eq!(name, "add");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("Expected call with 2 args"),
        }
    }

    #[test]
    fn test_expr_field_access() {
        match expr("user.name") {
            Expression::Access { field, .. } => assert_eq!(field, "name"),
            _ => panic!("Expected field access"),
        }

        match expr("obj.x.y") {
            Expression::Access { field, .. } => assert_eq!(field, "y"),
            _ => panic!("Expected nested field access"),
        }
    }

    #[test]
    fn test_try_expr_success() {
        assert!(try_expr("42").is_ok());
        assert!(try_expr("foo()").is_ok());
        assert!(try_expr("x.y").is_ok());
        assert!(try_expr("true").is_ok());
    }

    #[test]
    fn test_try_expr_empty_input() {
        match try_expr("") {
            Err(ExprError::EmptyInput) => {}
            other => panic!("Expected EmptyInput error, got: {:?}", other),
        }
    }

    #[test]
    fn test_try_expr_invalid_syntax() {
        match try_expr("!!!") {
            Err(ExprError::ParseFailed { input, .. }) => {
                assert_eq!(input, "!!!");
            }
            other => panic!("Expected ParseFailed error, got: {:?}", other),
        }

        match try_expr(")(") {
            Err(ExprError::ParseFailed { .. }) => {}
            other => panic!("Expected ParseFailed error, got: {:?}", other),
        }
    }

    #[test]
    fn test_expr_value_success() {
        let v = expr_value("42");
        match v {
            Value::Expression(_) => {}
            _ => panic!("Expected expression value"),
        }
    }

    #[test]
    fn test_try_expr_value_success() {
        let result = try_expr_value("my_func()");
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Expression(_) => {}
            _ => panic!("Expected expression value"),
        }
    }

    #[test]
    fn test_try_expr_value_empty() {
        match try_expr_value("") {
            Err(ExprError::EmptyInput) => {}
            other => panic!("Expected EmptyInput error, got: {:?}", other),
        }
    }

    #[test]
    fn test_expr_error_display() {
        let err = ExprError::EmptyInput;
        assert_eq!(err.to_string(), "Expression cannot be empty");

        let err = ExprError::Missing;
        assert_eq!(err.to_string(), "Expression is missing or null");

        let err = ExprError::ParseFailed {
            source: hedl_core::lex::LexError::InvalidToken {
                message: "test error".to_string(),
                pos: hedl_core::lex::SourcePos::new(1, 1),
            },
            input: "bad input".to_string(),
        };
        assert!(err.to_string().contains("bad input"));
        assert!(err.to_string().contains("test error"));
    }

    #[test]
    fn test_expr_is_expression_trait() {
        // Verify that Expression has necessary trait implementations
        let e = expr("42");
        let _ = format!("{:?}", e); // Debug
        let e2 = expr("42");
        let _ = e == e2; // PartialEq
        let _ = e.clone(); // Clone
    }

    #[test]
    #[should_panic(expected = "Invalid test expression")]
    fn test_expr_panics_on_invalid() {
        let _ = expr("!!!");
    }

    #[test]
    #[should_panic(expected = "Invalid test expression")]
    fn test_expr_panics_on_empty() {
        let _ = expr("");
    }

    #[test]
    fn test_try_expr_preserves_input_in_error() {
        let input = "invalid stuff!!!";
        match try_expr(input) {
            Err(ExprError::ParseFailed { input: err_input, .. }) => {
                assert_eq!(err_input, input);
            }
            _ => panic!("Expected ParseFailed with preserved input"),
        }
    }
}

// Helper trait for Expression to check literal type (test helper only)
#[allow(dead_code)]
trait ExpressionExt {
    fn is_literal(&self) -> bool;
}

impl ExpressionExt for Expression {
    fn is_literal(&self) -> bool {
        matches!(self, Expression::Literal { .. })
    }
}
