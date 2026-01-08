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

//! Expression AST for HEDL `$(...)` expressions.
//!
//! Expressions follow a minimal function-call grammar:
//! - Identifiers: `x`, `foo_bar`
//! - Literals: `42`, `3.5`, `"hello"`, `true`, `false`
//! - Function calls: `func(arg1, arg2)`
//! - Field access: `target.field`

use super::error::LexError;
use super::span::{SourcePos, Span};

/// A parsed expression from `$(...)`.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// A literal value: number, string, or boolean.
    Literal {
        value: ExprLiteral,
        span: Span,
    },
    /// An identifier: `foo`, `bar_baz`.
    Identifier {
        name: String,
        span: Span,
    },
    /// A function call: `func(arg1, arg2)`.
    Call {
        name: String,
        args: Vec<Expression>,
        span: Span,
    },
    /// Field access: `target.field`.
    Access {
        target: Box<Expression>,
        field: String,
        span: Span,
    },
}

/// A literal value within an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum ExprLiteral {
    /// Integer literal.
    Int(i64),
    /// Float literal.
    Float(f64),
    /// String literal (unquoted content).
    String(String),
    /// Boolean literal.
    Bool(bool),
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Literal { value, .. } => write!(f, "{}", value),
            Expression::Identifier { name, .. } => write!(f, "{}", name),
            Expression::Call { name, args, .. } => {
                write!(f, "{}(", name)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expression::Access { target, field, .. } => {
                write!(f, "{}.{}", target, field)
            }
        }
    }
}

impl std::fmt::Display for ExprLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExprLiteral::Int(n) => write!(f, "{}", n),
            ExprLiteral::Float(n) => write!(f, "{}", n),
            ExprLiteral::String(s) => {
                // Re-escape the string for display
                write!(f, "\"")?;
                for ch in s.chars() {
                    if ch == '"' {
                        write!(f, "\"\"")?;
                    } else {
                        write!(f, "{}", ch)?;
                    }
                }
                write!(f, "\"")
            }
            ExprLiteral::Bool(b) => write!(f, "{}", b),
        }
    }
}

/// Parse an expression from the content inside `$(...)`.
///
/// # Grammar (informal):
/// ```text
/// expr     = call | access | atom
/// call     = identifier "(" args ")"
/// access   = expr "." identifier
/// atom     = identifier | literal
/// args     = (expr ("," expr)*)?
/// literal  = number | string | bool
/// ```
pub fn parse_expression(s: &str) -> Result<Expression, LexError> {
    let mut parser = ExprParser::new(s);
    let expr = parser.parse_expr()?;
    parser.skip_whitespace();
    if parser.pos < parser.chars.len() {
        return Err(LexError::InvalidToken { message: format!(
            "unexpected character '{}' at position {}",
            parser.chars[parser.pos], parser.pos
        ), pos: SourcePos::default() });
    }
    Ok(expr)
}

/// Parse expression content from a `$(...)` token.
///
/// This extracts the content between `$(` and `)` and parses it.
pub fn parse_expression_token(s: &str) -> Result<Expression, LexError> {
    if !s.starts_with("$(") {
        return Err(LexError::InvalidToken { message: "expression must start with $(".to_string(), pos: SourcePos::default() });
    }

    // Find matching closing paren
    let content = extract_expression_content(s)?;
    parse_expression(&content)
}

/// Extract expression content from `$(...)`, handling nested parens and quotes.
fn extract_expression_content(s: &str) -> Result<String, LexError> {
    if !s.starts_with("$(") {
        return Err(LexError::InvalidToken { message: "expression must start with $(".to_string(), pos: SourcePos::default() });
    }

    let mut in_quotes = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 2; // Skip "$("
    let mut depth = 1;
    let mut content_end = None;

    while i < chars.len() {
        let ch = chars[i];

        if ch == '"' {
            // Check for escaped quote
            if in_quotes && i + 1 < chars.len() && chars[i + 1] == '"' {
                i += 2;
                continue;
            }
            in_quotes = !in_quotes;
        } else if !in_quotes {
            if ch == '(' {
                depth += 1;
            } else if ch == ')' {
                depth -= 1;
                if depth == 0 {
                    content_end = Some(i);
                    break;
                }
            }
        }

        i += 1;
    }

    if depth != 0 {
        return Err(LexError::UnclosedExpression { pos: SourcePos::default() });
    }

    let content_end = content_end.ok_or(LexError::UnclosedExpression { pos: SourcePos::default() })?;
    let content: String = chars[2..content_end].iter().collect();
    Ok(content)
}

struct ExprParser {
    chars: Vec<char>,
    pos: usize,
}

impl ExprParser {
    fn new(s: &str) -> Self {
        Self {
            chars: s.chars().collect(),
            pos: 0,
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn parse_expr(&mut self) -> Result<Expression, LexError> {
        self.skip_whitespace();

        let mut expr = self.parse_atom()?;

        // Handle chained operations (call or access)
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('(') => {
                    // Function call - expr must be identifier
                    let name = match expr {
                        Expression::Identifier { name, .. } => name,
                        _ => {
                            return Err(LexError::InvalidToken { message: "function call on non-identifier".to_string(), pos: SourcePos::default() });
                        }
                    };
                    self.advance(); // consume '('
                    let args = self.parse_args()?;
                    expr = Expression::Call {
                        name,
                        args,
                        span: Span::default(), // TODO: Track actual span
                    };
                }
                Some('.') => {
                    // Field access
                    self.advance(); // consume '.'
                    self.skip_whitespace();
                    let field = self.parse_identifier()?;
                    expr = Expression::Access {
                        target: Box::new(expr),
                        field,
                        span: Span::default(), // TODO: Track actual span
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_atom(&mut self) -> Result<Expression, LexError> {
        self.skip_whitespace();

        match self.peek() {
            Some('"') => {
                // String literal
                let s = self.parse_string()?;
                Ok(Expression::Literal {
                    value: ExprLiteral::String(s),
                    span: Span::default(), // TODO: Track actual span
                })
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                // Number literal (or negative)
                self.parse_number()
            }
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                // Identifier or boolean
                let ident = self.parse_identifier()?;
                match ident.as_str() {
                    "true" => Ok(Expression::Literal {
                        value: ExprLiteral::Bool(true),
                        span: Span::default(), // TODO: Track actual span
                    }),
                    "false" => Ok(Expression::Literal {
                        value: ExprLiteral::Bool(false),
                        span: Span::default(), // TODO: Track actual span
                    }),
                    _ => Ok(Expression::Identifier {
                        name: ident,
                        span: Span::default(), // TODO: Track actual span
                    }),
                }
            }
            Some('(') => {
                // Parenthesized expression
                self.advance(); // consume '('
                let expr = self.parse_expr()?;
                self.skip_whitespace();
                if self.peek() != Some(')') {
                    return Err(LexError::InvalidToken { message: "expected ')' after parenthesized expression".to_string(), pos: SourcePos::default() });
                }
                self.advance(); // consume ')'
                Ok(expr)
            }
            Some(ch) => Err(LexError::InvalidToken { message: format!(
                "unexpected character '{}' in expression",
                ch
            ), pos: SourcePos::default() }),
            None => Err(LexError::InvalidToken { message: "unexpected end of expression".to_string(), pos: SourcePos::default() }),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, LexError> {
        let mut ident = String::new();

        match self.peek() {
            Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                if let Some(c) = self.advance() {
                    ident.push(c);
                } else {
                    return Err(LexError::InvalidToken { message: "unexpected end of input while parsing identifier".to_string(), pos: SourcePos::default() });
                }
            }
            _ => {
                return Err(LexError::InvalidToken { message: "expected identifier".to_string(), pos: SourcePos::default() });
            }
        }

        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                if let Some(c) = self.advance() {
                    ident.push(c);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(ident)
    }

    fn parse_string(&mut self) -> Result<String, LexError> {
        if self.advance() != Some('"') {
            return Err(LexError::InvalidToken {
                message: "expected '\"'".to_string(),
                pos: SourcePos::default(),
            });
        }

        let mut result = String::new();

        loop {
            match self.advance() {
                Some('"') => {
                    // Check for escaped quote
                    if self.peek() == Some('"') {
                        self.advance();
                        result.push('"');
                    } else {
                        // End of string
                        return Ok(result);
                    }
                }
                Some(ch) => result.push(ch),
                None => return Err(LexError::UnclosedQuote { pos: SourcePos::default() }),
            }
        }
    }

    fn parse_number(&mut self) -> Result<Expression, LexError> {
        let mut num_str = String::new();
        let mut has_dot = false;

        // Handle negative sign
        if self.peek() == Some('-') {
            if let Some(c) = self.advance() {
                num_str.push(c);
            } else {
                return Err(LexError::InvalidToken { message: "unexpected end of input while parsing number".to_string(), pos: SourcePos::default() });
            }
        }

        // Parse digits
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                if let Some(c) = self.advance() {
                    num_str.push(c);
                } else {
                    break;
                }
            } else if ch == '.' && !has_dot {
                // Check if this is a decimal point (not field access)
                // Look ahead to see if it's followed by digits
                let next_pos = self.pos + 1;
                if next_pos < self.chars.len() && self.chars[next_pos].is_ascii_digit() {
                    has_dot = true;
                    if let Some(c) = self.advance() {
                        num_str.push(c);
                    } else {
                        break;
                    }
                } else {
                    // This is field access, stop here
                    break;
                }
            } else {
                break;
            }
        }

        if has_dot {
            let f: f64 = num_str
                .parse()
                .map_err(|_| LexError::InvalidToken { message: format!("invalid float: {}", num_str), pos: SourcePos::default() })?;
            Ok(Expression::Literal {
                value: ExprLiteral::Float(f),
                span: Span::default(), // TODO: Track actual span
            })
        } else {
            let i: i64 = num_str
                .parse()
                .map_err(|_| LexError::InvalidToken { message: format!("invalid integer: {}", num_str), pos: SourcePos::default() })?;
            Ok(Expression::Literal {
                value: ExprLiteral::Int(i),
                span: Span::default(), // TODO: Track actual span
            })
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expression>, LexError> {
        let mut args = Vec::new();

        self.skip_whitespace();
        if self.peek() == Some(')') {
            self.advance(); // consume ')'
            return Ok(args);
        }

        loop {
            let arg = self.parse_expr()?;
            args.push(arg);

            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.advance(); // consume ','
                }
                Some(')') => {
                    self.advance(); // consume ')'
                    return Ok(args);
                }
                Some(ch) => {
                    return Err(LexError::InvalidToken {
                        message: format!("expected ',' or ')' in argument list, got '{}'", ch),
                        pos: SourcePos::new(1, 1),
                    });
                }
                None => {
                    return Err(LexError::InvalidToken {
                        message: "unexpected end of expression in argument list".to_string(),
                        pos: SourcePos::new(1, 1),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier() {
        let expr = parse_expression("foo").unwrap();
        assert!(matches!(expr, Expression::Identifier { name, .. } if name == "foo"));
    }

    #[test]
    fn test_parse_identifier_with_underscore() {
        let expr = parse_expression("foo_bar").unwrap();
        assert!(matches!(expr, Expression::Identifier { name, .. } if name == "foo_bar"));
    }

    #[test]
    fn test_parse_integer() {
        let expr = parse_expression("42").unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::Int(42), .. }));
    }

    #[test]
    fn test_parse_negative_integer() {
        let expr = parse_expression("-123").unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::Int(-123), .. }));
    }

    #[test]
    fn test_parse_float() {
        let expr = parse_expression("3.25").unwrap();
        assert!(
            matches!(expr, Expression::Literal { value: ExprLiteral::Float(f), .. } if (f - 3.25).abs() < 0.001)
        );
    }

    #[test]
    fn test_parse_string() {
        let expr = parse_expression(r#""hello""#).unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::String(s), .. } if s == "hello"));
    }

    #[test]
    fn test_parse_string_with_escaped_quote() {
        let expr = parse_expression(r#""say ""hello""""#).unwrap();
        assert!(
            matches!(expr, Expression::Literal { value: ExprLiteral::String(s), .. } if s == "say \"hello\"")
        );
    }

    #[test]
    fn test_parse_bool_true() {
        let expr = parse_expression("true").unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::Bool(true), .. }));
    }

    #[test]
    fn test_parse_bool_false() {
        let expr = parse_expression("false").unwrap();
        assert!(matches!(
            expr,
            Expression::Literal { value: ExprLiteral::Bool(false), .. }
        ));
    }

    #[test]
    fn test_parse_call_no_args() {
        let expr = parse_expression("now()").unwrap();
        assert!(
            matches!(expr, Expression::Call { name, args, .. } if name == "now" && args.is_empty())
        );
    }

    #[test]
    fn test_parse_call_one_arg() {
        let expr = parse_expression("upper(x)").unwrap();
        match expr {
            Expression::Call { name, args, .. } => {
                assert_eq!(name, "upper");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Expression::Identifier { name, .. } if name == "x"));
            }
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn test_parse_call_multiple_args() {
        let expr = parse_expression("concat(a, b, c)").unwrap();
        match expr {
            Expression::Call { name, args, .. } => {
                assert_eq!(name, "concat");
                assert_eq!(args.len(), 3);
            }
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn test_parse_call_string_args() {
        let expr = parse_expression(r#"concat("hello", "world")"#).unwrap();
        match expr {
            Expression::Call { name, args, .. } => {
                assert_eq!(name, "concat");
                assert_eq!(args.len(), 2);
                assert!(
                    matches!(&args[0], Expression::Literal { value: ExprLiteral::String(s), .. } if s == "hello")
                );
                assert!(
                    matches!(&args[1], Expression::Literal { value: ExprLiteral::String(s), .. } if s == "world")
                );
            }
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn test_parse_nested_call() {
        let expr = parse_expression("outer(inner(x))").unwrap();
        match expr {
            Expression::Call { name, args, .. } => {
                assert_eq!(name, "outer");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    Expression::Call {
                        name: inner_name, ..
                    } => {
                        assert_eq!(inner_name, "inner");
                    }
                    _ => panic!("expected nested Call"),
                }
            }
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn test_parse_field_access() {
        let expr = parse_expression("user.name").unwrap();
        match expr {
            Expression::Access { target, field, .. } => {
                assert!(matches!(*target, Expression::Identifier { name, .. } if name == "user"));
                assert_eq!(field, "name");
            }
            _ => panic!("expected Access"),
        }
    }

    #[test]
    fn test_parse_chained_access() {
        let expr = parse_expression("a.b.c").unwrap();
        match expr {
            Expression::Access { target, field, .. } => {
                assert_eq!(field, "c");
                match *target {
                    Expression::Access {
                        target: inner,
                        field: inner_field,
                        ..
                    } => {
                        assert!(matches!(*inner, Expression::Identifier { name, .. } if name == "a"));
                        assert_eq!(inner_field, "b");
                    }
                    _ => panic!("expected nested Access"),
                }
            }
            _ => panic!("expected Access"),
        }
    }

    #[test]
    fn test_parse_call_then_access() {
        let expr = parse_expression("get_user().name").unwrap();
        match expr {
            Expression::Access { target, field, .. } => {
                assert_eq!(field, "name");
                assert!(matches!(*target, Expression::Call { name, .. } if name == "get_user"));
            }
            _ => panic!("expected Access"),
        }
    }

    #[test]
    fn test_parse_expression_token() {
        let expr = parse_expression_token("$(now())").unwrap();
        assert!(
            matches!(expr, Expression::Call { name, args, .. } if name == "now" && args.is_empty())
        );
    }

    #[test]
    fn test_parse_expression_token_nested_parens() {
        let expr = parse_expression_token("$(concat(a, b))").unwrap();
        assert!(matches!(expr, Expression::Call { name, .. } if name == "concat"));
    }

    #[test]
    fn test_display_identifier() {
        let expr = Expression::Identifier { name: "foo".to_string(), span: Span::default() };
        assert_eq!(format!("{}", expr), "foo");
    }

    #[test]
    fn test_display_call() {
        let expr = Expression::Call {
            name: "func".to_string(),
            args: vec![
                Expression::Identifier { name: "x".to_string(), span: Span::default() },
                Expression::Literal { value: ExprLiteral::Int(42), span: Span::default() },
            ],
            span: Span::default(),
        };
        assert_eq!(format!("{}", expr), "func(x, 42)");
    }

    #[test]
    fn test_display_access() {
        let expr = Expression::Access {
            target: Box::new(Expression::Identifier { name: "user".to_string(), span: Span::default() }),
            field: "name".to_string(),
            span: Span::default(),
        };
        assert_eq!(format!("{}", expr), "user.name");
    }

    #[test]
    fn test_display_string_with_quotes() {
        let expr = Expression::Literal { value: ExprLiteral::String("say \"hi\"".to_string()), span: Span::default() };
        assert_eq!(format!("{}", expr), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_whitespace_handling() {
        let expr = parse_expression("  func(  a  ,  b  )  ").unwrap();
        assert!(
            matches!(expr, Expression::Call { name, args, .. } if name == "func" && args.len() == 2)
        );
    }

    #[test]
    fn test_error_unclosed_paren() {
        let result = parse_expression("func(x");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unclosed_string() {
        let result = parse_expression(r#""unclosed"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unexpected_char() {
        let result = parse_expression("func@");
        assert!(result.is_err());
    }

    #[test]
    fn test_parenthesized_expr() {
        let expr = parse_expression("(foo)").unwrap();
        assert!(matches!(expr, Expression::Identifier { name, .. } if name == "foo"));
    }

    // ==================== Additional literal tests ====================

    #[test]
    fn test_parse_zero() {
        let expr = parse_expression("0").unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::Int(0), .. }));
    }

    #[test]
    fn test_parse_large_integer() {
        let expr = parse_expression("9223372036854775807").unwrap();
        assert!(matches!(
            expr,
            Expression::Literal { value: ExprLiteral::Int(9223372036854775807), .. }
        ));
    }

    #[test]
    fn test_parse_negative_zero() {
        // -0 is valid and should parse as -0 (which equals 0)
        let expr = parse_expression("-0").unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::Int(0), .. }));
    }

    #[test]
    fn test_parse_float_zero() {
        let expr = parse_expression("0.0").unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::Float(f), .. } if f == 0.0));
    }

    #[test]
    fn test_parse_negative_float() {
        let expr = parse_expression("-3.5").unwrap();
        assert!(
            matches!(expr, Expression::Literal { value: ExprLiteral::Float(f), .. } if (f - (-3.5)).abs() < 0.001)
        );
    }

    #[test]
    fn test_parse_float_small() {
        let expr = parse_expression("0.001").unwrap();
        assert!(
            matches!(expr, Expression::Literal { value: ExprLiteral::Float(f), .. } if (f - 0.001).abs() < 0.0001)
        );
    }

    #[test]
    fn test_parse_string_empty() {
        let expr = parse_expression(r#""""#).unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::String(s), .. } if s.is_empty()));
    }

    #[test]
    fn test_parse_string_with_spaces() {
        let expr = parse_expression(r#""hello world""#).unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::String(s), .. } if s == "hello world"));
    }

    #[test]
    fn test_parse_string_with_special_chars() {
        let expr = parse_expression(r#""hello!@#$%""#).unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::String(s), .. } if s == "hello!@#$%"));
    }

    #[test]
    fn test_parse_string_unicode() {
        let expr = parse_expression(r#""日本語 🎉""#).unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::String(s), .. } if s.contains('🎉')));
    }

    #[test]
    fn test_parse_string_multiple_escaped_quotes() {
        let expr = parse_expression(r#""""""""#).unwrap();
        assert!(matches!(expr, Expression::Literal { value: ExprLiteral::String(s), .. } if s == "\"\""));
    }

    // ==================== Additional identifier tests ====================

    #[test]
    fn test_parse_identifier_single_char() {
        let expr = parse_expression("x").unwrap();
        assert!(matches!(expr, Expression::Identifier { name, .. } if name == "x"));
    }

    #[test]
    fn test_parse_identifier_underscore_only() {
        let expr = parse_expression("_").unwrap();
        assert!(matches!(expr, Expression::Identifier { name, .. } if name == "_"));
    }

    #[test]
    fn test_parse_identifier_leading_underscore() {
        let expr = parse_expression("_private").unwrap();
        assert!(matches!(expr, Expression::Identifier { name, .. } if name == "_private"));
    }

    #[test]
    fn test_parse_identifier_double_underscore() {
        let expr = parse_expression("__dunder__").unwrap();
        assert!(matches!(expr, Expression::Identifier { name, .. } if name == "__dunder__"));
    }

    #[test]
    fn test_parse_identifier_with_numbers() {
        let expr = parse_expression("var123").unwrap();
        assert!(matches!(expr, Expression::Identifier { name, .. } if name == "var123"));
    }

    // ==================== Additional call tests ====================

    #[test]
    fn test_parse_call_with_literals() {
        let expr = parse_expression("func(42, 3.5, true)").unwrap();
        match expr {
            Expression::Call { name, args, .. } => {
                assert_eq!(name, "func");
                assert_eq!(args.len(), 3);
                assert!(matches!(args[0], Expression::Literal { value: ExprLiteral::Int(42), .. }));
                assert!(matches!(
                    args[1],
                    Expression::Literal { value: ExprLiteral::Float(_), .. }
                ));
                assert!(matches!(
                    args[2],
                    Expression::Literal { value: ExprLiteral::Bool(true), .. }
                ));
            }
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn test_parse_deeply_nested_calls() {
        let expr = parse_expression("a(b(c(d(e))))").unwrap();
        fn count_depth(e: &Expression) -> usize {
            match e {
                Expression::Call { args, .. } => {
                    if args.is_empty() {
                        1
                    } else {
                        1 + count_depth(&args[0])
                    }
                }
                Expression::Identifier { .. } => 1,
                _ => 0,
            }
        }
        assert_eq!(count_depth(&expr), 5);
    }

    #[test]
    fn test_parse_call_with_string_containing_comma() {
        let expr = parse_expression(r#"func("a, b", c)"#).unwrap();
        match expr {
            Expression::Call { name, args, .. } => {
                assert_eq!(name, "func");
                assert_eq!(args.len(), 2);
                assert!(
                    matches!(&args[0], Expression::Literal { value: ExprLiteral::String(s), .. } if s == "a, b")
                );
            }
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn test_parse_call_with_string_containing_paren() {
        let expr = parse_expression(r#"func("(test)")"#).unwrap();
        match expr {
            Expression::Call { name, args, .. } => {
                assert_eq!(name, "func");
                assert_eq!(args.len(), 1);
                assert!(
                    matches!(&args[0], Expression::Literal { value: ExprLiteral::String(s), .. } if s == "(test)")
                );
            }
            _ => panic!("expected Call"),
        }
    }

    // ==================== Additional access tests ====================

    #[test]
    fn test_parse_deeply_chained_access() {
        let expr = parse_expression("a.b.c.d.e").unwrap();
        fn count_access(e: &Expression) -> usize {
            match e {
                Expression::Access { target, .. } => 1 + count_access(target),
                _ => 0,
            }
        }
        assert_eq!(count_access(&expr), 4);
    }

    #[test]
    fn test_parse_access_with_numbers_in_field() {
        let expr = parse_expression("obj.field123").unwrap();
        match expr {
            Expression::Access { field, .. } => {
                assert_eq!(field, "field123");
            }
            _ => panic!("expected Access"),
        }
    }

    #[test]
    fn test_parse_access_then_call_error() {
        // Access result cannot be called directly - it's not an identifier
        // This is a parser limitation: obj.method() doesn't work because
        // after access, we have an Access expression, not an identifier
        let result = parse_expression("obj.method()");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_method_call_works() {
        // But a call on a plain identifier works
        let expr = parse_expression("method()").unwrap();
        match expr {
            Expression::Call { name, args, .. } => {
                assert_eq!(name, "method");
                assert!(args.is_empty());
            }
            _ => panic!("expected Call"),
        }
    }

    #[test]
    fn test_parse_number_then_access() {
        // 42.field should parse 42 as int, then try to access .field
        // But actually, this parses as a float attempt which fails
        let expr = parse_expression("42.5").unwrap();
        assert!(matches!(
            expr,
            Expression::Literal { value: ExprLiteral::Float(f), .. } if (f - 42.5).abs() < 0.001
        ));
    }

    // ==================== Expression token tests ====================

    #[test]
    fn test_parse_expression_token_simple() {
        let expr = parse_expression_token("$(x)").unwrap();
        assert!(matches!(expr, Expression::Identifier { name, .. } if name == "x"));
    }

    #[test]
    fn test_parse_expression_token_complex() {
        let expr = parse_expression_token("$(user.profile.name)").unwrap();
        match expr {
            Expression::Access { field, .. } => {
                assert_eq!(field, "name");
            }
            _ => panic!("expected Access"),
        }
    }

    #[test]
    fn test_parse_expression_token_with_quotes() {
        let expr = parse_expression_token(r#"$(concat("a", "b"))"#).unwrap();
        assert!(matches!(expr, Expression::Call { .. }));
    }

    #[test]
    fn test_parse_expression_token_not_starting_with_dollar() {
        let result = parse_expression_token("(x)");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_expression_token_unclosed() {
        let result = parse_expression_token("$(x");
        assert!(result.is_err());
    }

    // ==================== Display tests ====================

    #[test]
    fn test_display_int() {
        let lit = ExprLiteral::Int(42);
        assert_eq!(format!("{}", lit), "42");
    }

    #[test]
    fn test_display_float() {
        let lit = ExprLiteral::Float(3.5);
        assert!(format!("{}", lit).starts_with("3.5"));
    }

    #[test]
    fn test_display_bool() {
        assert_eq!(format!("{}", ExprLiteral::Bool(true)), "true");
        assert_eq!(format!("{}", ExprLiteral::Bool(false)), "false");
    }

    #[test]
    fn test_display_nested_call() {
        let expr = Expression::Call {
            name: "outer".to_string(),
            args: vec![Expression::Call {
                name: "inner".to_string(),
                args: vec![Expression::Literal { value: ExprLiteral::Int(42), span: Span::default() }],
                span: Span::default(),
            }],
            span: Span::default(),
        };
        assert_eq!(format!("{}", expr), "outer(inner(42))");
    }

    #[test]
    fn test_display_access_chain() {
        let expr = Expression::Access {
            target: Box::new(Expression::Access {
                target: Box::new(Expression::Identifier { name: "a".to_string(), span: Span::default() }),
                field: "b".to_string(),
                span: Span::default(),
            }),
            field: "c".to_string(),
            span: Span::default(),
        };
        assert_eq!(format!("{}", expr), "a.b.c");
    }

    // ==================== Error handling tests ====================

    #[test]
    fn test_error_empty_expression() {
        let result = parse_expression("");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_only_whitespace() {
        let result = parse_expression("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_trailing_garbage() {
        let result = parse_expression("foo bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_call_on_literal() {
        // This parses "42" as literal, then sees "()" and fails
        // because 42 is not an identifier
        let result = parse_expression("42()");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unclosed_string_in_call() {
        let result = parse_expression(r#"func("unclosed)"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_missing_comma_in_args() {
        let result = parse_expression("func(a b)");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_trailing_comma_in_args() {
        let result = parse_expression("func(a,)");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unclosed_paren_in_nested() {
        let result = parse_expression("outer(inner(x)");
        assert!(result.is_err());
    }

    // ==================== Struct equality and clone tests ====================

    #[test]
    fn test_expression_equality() {
        let a = Expression::Identifier { name: "foo".to_string(), span: Span::default() };
        let b = Expression::Identifier { name: "foo".to_string(), span: Span::default() };
        assert_eq!(a, b);
    }

    #[test]
    fn test_expression_clone() {
        let original = Expression::Call {
            name: "func".to_string(),
            args: vec![Expression::Literal { value: ExprLiteral::Int(42), span: Span::default() }],
            span: Span::default(),
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_expr_literal_equality() {
        assert_eq!(ExprLiteral::Int(42), ExprLiteral::Int(42));
        assert_ne!(ExprLiteral::Int(42), ExprLiteral::Int(43));
        assert_eq!(ExprLiteral::Bool(true), ExprLiteral::Bool(true));
        assert_eq!(
            ExprLiteral::String("test".to_string()),
            ExprLiteral::String("test".to_string())
        );
    }

    #[test]
    fn test_expr_literal_clone() {
        let original = ExprLiteral::String("hello".to_string());
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_expression_debug() {
        let expr = Expression::Call {
            name: "func".to_string(),
            args: vec![],
            span: Span::default(),
        };
        let debug = format!("{:?}", expr);
        assert!(debug.contains("func"));
    }

    // ==================== Parenthesized expression tests ====================

    #[test]
    fn test_deeply_nested_parens() {
        let expr = parse_expression("(((foo)))").unwrap();
        assert!(matches!(expr, Expression::Identifier { name, .. } if name == "foo"));
    }

    #[test]
    fn test_parens_around_call() {
        let expr = parse_expression("(func(x))").unwrap();
        assert!(matches!(expr, Expression::Call { name, .. } if name == "func"));
    }

    #[test]
    fn test_parens_around_access() {
        let expr = parse_expression("(a.b)").unwrap();
        assert!(matches!(expr, Expression::Access { field, .. } if field == "b"));
    }

    #[test]
    fn test_unclosed_inner_paren() {
        let result = parse_expression("((foo)");
        assert!(result.is_err());
    }
}
