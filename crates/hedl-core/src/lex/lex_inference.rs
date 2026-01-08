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

//! Value inference for HEDL scalars.
//!
//! Implements the inference ladder from SPEC Section 8.2 and 9.3.

use super::error::LexError;
use super::expression::{parse_expression_token, Expression};
use super::span::SourcePos;
use super::tokens::{parse_reference, Reference};
use std::collections::HashMap;

/// A parsed scalar value after inference.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Null value (~).
    Null,
    /// Boolean (true/false).
    Bool(bool),
    /// Integer number.
    Int(i64),
    /// Floating point number.
    Float(f64),
    /// String value.
    String(String),
    /// Reference to another node (@id or @Type:id).
    Reference(Reference),
    /// Parsed expression from $(...).
    Expression(Expression),
    /// Tensor/array literal.
    Tensor(Vec<TensorValue>),
}

/// Recursive tensor value structure.
#[derive(Debug, Clone, PartialEq)]
pub enum TensorValue {
    /// Scalar number.
    Number(f64),
    /// Nested array.
    Array(Vec<TensorValue>),
}

/// Infer value from an unquoted string in Key-Value context.
///
/// Follows the inference ladder (Section 8.2):
/// 1. Null: ~ → null
/// 2. Tensor: starts with [ → parse as tensor
/// 3. Reference: starts with @ → parse as reference
/// 4. Expression: starts with $( → parse as expression
/// 5. Alias: matches %key pattern → expand (if aliases provided)
/// 6. Boolean: true/false → bool
/// 7. Number: matches number pattern → int or float
/// 8. String: everything else
///
/// Note: Ditto (^) in Key-Value context is treated as literal string "^".
pub fn infer_value(s: &str, aliases: Option<&HashMap<String, String>>) -> Result<Value, LexError> {
    infer_value_impl(s, aliases, false, None)
}

/// Infer value from an unquoted string in matrix cell context.
///
/// Follows the inference ladder (Section 9.3):
/// 1. Null: ~ → null
/// 2. Ditto: ^ → copy from previous row (if prev_values provided)
/// 3. Tensor: starts with [ → parse as tensor
/// 4. Reference: starts with @ → parse as reference
/// 5. Expression: starts with $( → parse as expression
/// 6. Alias: matches %key pattern → expand (if aliases provided)
/// 7. Boolean: true/false → bool
/// 8. Number: matches number pattern → int or float
/// 9. String: everything else
pub fn infer_cell_value(
    s: &str,
    column_idx: usize,
    prev_row: Option<&[Value]>,
    aliases: Option<&HashMap<String, String>>,
) -> Result<Value, LexError> {
    infer_value_impl(s, aliases, true, prev_row.and_then(|r| r.get(column_idx)))
}

/// Internal implementation of value inference.
fn infer_value_impl(
    s: &str,
    aliases: Option<&HashMap<String, String>>,
    allow_ditto: bool,
    prev_value: Option<&Value>,
) -> Result<Value, LexError> {
    let trimmed = s.trim();

    // Detect the value type
    let value_type = detect_value_type(trimmed, allow_ditto, prev_value.is_some(), aliases)?;

    // Construct the value based on detected type
    construct_value(value_type, trimmed, aliases, prev_value)
}

/// Value type detected from the input string.
#[derive(Debug, Clone, PartialEq)]
enum ValueType {
    Null,
    Ditto,
    Tensor,
    Reference,
    Expression,
    Alias(String),
    Boolean(bool),
    Number,
    String,
}

/// Detect the type of value from the input string.
///
/// Follows the inference ladder priority order.
fn detect_value_type(
    trimmed: &str,
    allow_ditto: bool,
    has_prev_value: bool,
    aliases: Option<&HashMap<String, String>>,
) -> Result<ValueType, LexError> {
    // 1. Null
    if trimmed == "~" {
        return Ok(ValueType::Null);
    }

    // 2. Ditto (only in matrix cells)
    if allow_ditto && trimmed == "^" {
        if has_prev_value {
            return Ok(ValueType::Ditto);
        } else {
            return Err(LexError::InvalidToken { message: "ditto operator (^) used without previous row value".to_string(), pos: SourcePos::default() });
        }
    }

    // 3. Tensor literal
    if trimmed.starts_with('[') {
        return Ok(ValueType::Tensor);
    }

    // 4. Reference
    if trimmed.starts_with('@') {
        return Ok(ValueType::Reference);
    }

    // 5. Expression
    if trimmed.starts_with("$(") {
        return Ok(ValueType::Expression);
    }

    // 6. Alias expansion
    if let Some(key) = trimmed.strip_prefix('%') {
        if let Some(aliases_map) = aliases {
            if aliases_map.contains_key(key) {
                return Ok(ValueType::Alias(key.to_string()));
            }
        }
        // If no alias found, fall through to other inference steps
    }

    // 7. Boolean
    if trimmed == "true" {
        return Ok(ValueType::Boolean(true));
    }
    if trimmed == "false" {
        return Ok(ValueType::Boolean(false));
    }

    // 8. Number
    if is_number(trimmed) {
        return Ok(ValueType::Number);
    }

    // 9. String (default)
    Ok(ValueType::String)
}

/// Construct a value from the detected type.
fn construct_value(
    value_type: ValueType,
    trimmed: &str,
    aliases: Option<&HashMap<String, String>>,
    prev_value: Option<&Value>,
) -> Result<Value, LexError> {
    match value_type {
        ValueType::Null => Ok(Value::Null),
        ValueType::Ditto => {
            // Safety: prev_value is guaranteed to be Some by detect_value_type
            Ok(prev_value.unwrap().clone())
        }
        ValueType::Tensor => parse_tensor(trimmed).map(Value::Tensor),
        ValueType::Reference => parse_reference(trimmed).map(Value::Reference),
        ValueType::Expression => parse_expression_token(trimmed).map(Value::Expression),
        ValueType::Alias(key) => {
            // Safety: alias key is guaranteed to exist by detect_value_type
            let expanded = aliases.unwrap().get(&key).unwrap();
            // Recursively infer the expanded value
            infer_value_impl(expanded, None, false, None)
        }
        ValueType::Boolean(b) => Ok(Value::Bool(b)),
        ValueType::Number => parse_number(trimmed),
        ValueType::String => Ok(Value::String(trimmed.to_string())),
    }
}

/// Check if a string represents a valid number.
fn is_number(s: &str) -> bool {
    // Try integer first
    if !s.contains('.')
        && s.parse::<i64>().is_ok() {
            return true;
        }

    // Try float
    s.parse::<f64>().is_ok()
}

/// Parse a number: integer or float.
///
/// Pattern: `^-?[0-9]+(\.[0-9]+)?$`
fn parse_number(s: &str) -> Result<Value, LexError> {
    // Try integer first
    if !s.contains('.') {
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Value::Int(i));
        }
    }

    // Try float
    if let Ok(f) = s.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    Err(LexError::InvalidToken { message: format!("invalid number: {}", s), pos: SourcePos::default() })
}

/// Parse a tensor literal.
///
/// Must be balanced brackets with numbers only.
fn parse_tensor(s: &str) -> Result<Vec<TensorValue>, LexError> {
    let trimmed = s.trim();

    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(LexError::InvalidToken {
            message: "tensor must be enclosed in []".to_string(),
            pos: SourcePos::default(),
        });
    }

    let content = &trimmed[1..trimmed.len() - 1].trim();

    if content.is_empty() {
        return Ok(Vec::new());
    }

    parse_tensor_content(content)
}

/// Parse tensor content (inside brackets).
fn parse_tensor_content(s: &str) -> Result<Vec<TensorValue>, LexError> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut depth = 0;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        match ch {
            '[' => {
                depth += 1;
                current.push(ch);
            }
            ']' => {
                depth -= 1;
                current.push(ch);
                if depth < 0 {
                    return Err(LexError::InvalidToken {
                        message: "unbalanced brackets in tensor".to_string(),
                        pos: SourcePos::default(),
                    });
                }
            }
            ',' if depth == 0 => {
                // Top-level comma, process current element
                let elem = current.trim();
                if !elem.is_empty() {
                    result.push(parse_tensor_element(elem)?);
                }
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }

        i += 1;
    }

    // Process last element
    let elem = current.trim();
    if !elem.is_empty() {
        result.push(parse_tensor_element(elem)?);
    }

    if depth != 0 {
        return Err(LexError::InvalidToken {
            message: "unbalanced brackets in tensor".to_string(),
            pos: SourcePos::default(),
        });
    }

    Ok(result)
}

/// Parse a single tensor element (number or nested array).
fn parse_tensor_element(s: &str) -> Result<TensorValue, LexError> {
    let trimmed = s.trim();

    if trimmed.starts_with('[') {
        // Nested array
        let nested = parse_tensor(trimmed)?;
        Ok(TensorValue::Array(nested))
    } else {
        // Scalar number
        let num = trimmed.parse::<f64>().map_err(|_| {
            LexError::InvalidToken { message: format!("invalid number in tensor: {}", trimmed), pos: SourcePos::default() }
        })?;
        Ok(TensorValue::Number(num))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Null inference ====================

    #[test]
    fn test_infer_null() {
        let v = infer_value("~", None).unwrap();
        assert!(matches!(v, Value::Null));
    }

    #[test]
    fn test_infer_null_with_whitespace() {
        let v = infer_value("  ~  ", None).unwrap();
        assert!(matches!(v, Value::Null));
    }

    // ==================== Boolean inference ====================

    #[test]
    fn test_infer_bool_true() {
        let v = infer_value("true", None).unwrap();
        assert!(matches!(v, Value::Bool(true)));
    }

    #[test]
    fn test_infer_bool_false() {
        let v = infer_value("false", None).unwrap();
        assert!(matches!(v, Value::Bool(false)));
    }

    #[test]
    fn test_infer_bool_with_whitespace() {
        let v = infer_value("  true  ", None).unwrap();
        assert!(matches!(v, Value::Bool(true)));
    }

    #[test]
    fn test_infer_bool_case_sensitive() {
        // TRUE, True, FALSE, False should be strings
        let v = infer_value("TRUE", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "TRUE"));

        let v = infer_value("True", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "True"));

        let v = infer_value("FALSE", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "FALSE"));
    }

    // ==================== Integer inference ====================

    #[test]
    fn test_infer_int() {
        let v = infer_value("42", None).unwrap();
        assert!(matches!(v, Value::Int(42)));
    }

    #[test]
    fn test_infer_negative_int() {
        let v = infer_value("-123", None).unwrap();
        assert!(matches!(v, Value::Int(-123)));
    }

    #[test]
    fn test_infer_zero() {
        let v = infer_value("0", None).unwrap();
        assert!(matches!(v, Value::Int(0)));
    }

    #[test]
    fn test_infer_large_int() {
        let v = infer_value("9223372036854775807", None).unwrap(); // i64::MAX
        assert!(matches!(v, Value::Int(9223372036854775807)));
    }

    #[test]
    fn test_infer_min_int() {
        let v = infer_value("-9223372036854775808", None).unwrap(); // i64::MIN
        assert!(matches!(v, Value::Int(-9223372036854775808)));
    }

    #[test]
    fn test_infer_int_with_whitespace() {
        let v = infer_value("  42  ", None).unwrap();
        assert!(matches!(v, Value::Int(42)));
    }

    // ==================== Float inference ====================

    #[test]
    fn test_infer_float() {
        let v = infer_value("3.25", None).unwrap();
        assert!(matches!(v, Value::Float(f) if (f - 3.25).abs() < 0.001));
    }

    #[test]
    fn test_infer_float_explicit() {
        let v = infer_value("42.0", None).unwrap();
        assert!(matches!(v, Value::Float(f) if (f - 42.0).abs() < 0.001));
    }

    #[test]
    fn test_infer_negative_float() {
        let v = infer_value("-3.5", None).unwrap();
        assert!(matches!(v, Value::Float(f) if (f + 3.5).abs() < 0.001));
    }

    #[test]
    fn test_infer_float_scientific() {
        let v = infer_value("1e10", None).unwrap();
        assert!(matches!(v, Value::Float(f) if (f - 1e10).abs() < 1e5));
    }

    #[test]
    fn test_infer_float_scientific_negative_exp() {
        let v = infer_value("1e-10", None).unwrap();
        assert!(matches!(v, Value::Float(f) if (f - 1e-10).abs() < 1e-15));
    }

    #[test]
    fn test_infer_float_very_small() {
        let v = infer_value("0.000001", None).unwrap();
        assert!(matches!(v, Value::Float(f) if (f - 0.000001).abs() < 1e-12));
    }

    // ==================== String inference ====================

    #[test]
    fn test_infer_string() {
        let v = infer_value("hello", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "hello"));
    }

    #[test]
    fn test_infer_string_with_spaces() {
        let v = infer_value("  hello world  ", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "hello world"));
    }

    #[test]
    fn test_infer_string_with_special_chars() {
        let v = infer_value("hello@world.com", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "hello@world.com"));
    }

    #[test]
    fn test_infer_string_looks_like_number_but_isnt() {
        // Numbers with invalid format become strings
        let v = infer_value("42abc", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "42abc"));

        let v = infer_value("3.5.15", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "3.5.15"));
    }

    #[test]
    fn test_infer_string_unicode() {
        let v = infer_value("日本語テキスト", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "日本語テキスト"));

        let v = infer_value("émoji 😀", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "émoji 😀"));
    }

    #[test]
    fn test_infer_string_empty_after_trim() {
        let v = infer_value("   ", None).unwrap();
        assert!(matches!(v, Value::String(s) if s.is_empty()));
    }

    // ==================== Reference inference ====================

    #[test]
    fn test_infer_reference() {
        let v = infer_value("@user_1", None).unwrap();
        assert!(matches!(v, Value::Reference(r) if r.id == "user_1"));
    }

    #[test]
    fn test_infer_qualified_reference() {
        let v = infer_value("@User:user_1", None).unwrap();
        assert!(matches!(v, Value::Reference(r)
            if r.type_name == Some("User".to_string()) && r.id == "user_1"));
    }

    #[test]
    fn test_infer_reference_with_hyphen() {
        let v = infer_value("@user-1", None).unwrap();
        assert!(matches!(v, Value::Reference(r) if r.id == "user-1"));
    }

    #[test]
    fn test_infer_reference_with_whitespace() {
        let v = infer_value("  @user_1  ", None).unwrap();
        assert!(matches!(v, Value::Reference(r) if r.id == "user_1"));
    }

    // ==================== Expression inference ====================

    #[test]
    fn test_infer_expression() {
        let v = infer_value("$(now())", None).unwrap();
        match v {
            Value::Expression(expr) => {
                assert!(
                    matches!(expr, Expression::Call { name, args, .. } if name == "now" && args.is_empty())
                );
            }
            _ => panic!("expected Expression"),
        }
    }

    #[test]
    fn test_infer_expression_with_args() {
        let v = infer_value("$(concat(a, b))", None).unwrap();
        match v {
            Value::Expression(expr) => {
                assert!(matches!(expr, Expression::Call { name, .. } if name == "concat"));
            }
            _ => panic!("expected Expression"),
        }
    }

    #[test]
    fn test_infer_expression_with_whitespace() {
        let v = infer_value("  $(now())  ", None).unwrap();
        assert!(matches!(v, Value::Expression(_)));
    }

    // ==================== Tensor inference ====================

    #[test]
    fn test_infer_tensor_simple() {
        let v = infer_value("[1, 2, 3]", None).unwrap();
        assert!(matches!(v, Value::Tensor(_)));
    }

    #[test]
    fn test_infer_tensor_nested() {
        let v = infer_value("[[1, 2], [3, 4]]", None).unwrap();
        assert!(matches!(v, Value::Tensor(_)));
    }

    #[test]
    fn test_infer_tensor_with_whitespace() {
        let v = infer_value("  [ 1 , 2 , 3 ]  ", None).unwrap();
        assert!(matches!(v, Value::Tensor(_)));
    }

    #[test]
    fn test_infer_tensor_floats() {
        let v = infer_value("[1.5, 2.5, 3.5]", None).unwrap();
        if let Value::Tensor(t) = v {
            assert_eq!(t.len(), 3);
            assert!(matches!(t[0], TensorValue::Number(n) if (n - 1.5).abs() < 0.001));
        } else {
            panic!("expected Tensor");
        }
    }

    #[test]
    fn test_infer_tensor_negative() {
        let v = infer_value("[-1, -2, -3]", None).unwrap();
        if let Value::Tensor(t) = v {
            assert_eq!(t.len(), 3);
            assert!(matches!(t[0], TensorValue::Number(n) if (n + 1.0).abs() < 0.001));
        } else {
            panic!("expected Tensor");
        }
    }

    // ==================== Alias expansion ====================

    #[test]
    fn test_infer_alias_expansion() {
        let mut aliases = HashMap::new();
        aliases.insert("active".to_string(), "true".to_string());

        let v = infer_value("%active", Some(&aliases)).unwrap();
        assert!(matches!(v, Value::Bool(true)));
    }

    #[test]
    fn test_infer_alias_expansion_number() {
        let mut aliases = HashMap::new();
        aliases.insert("rate".to_string(), "1.23456".to_string());

        let v = infer_value("%rate", Some(&aliases)).unwrap();
        assert!(matches!(v, Value::Float(_)));
    }

    #[test]
    fn test_infer_alias_not_found_becomes_string() {
        let v = infer_value("%unknown", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "%unknown"));
    }

    #[test]
    fn test_infer_alias_expansion_to_string() {
        let mut aliases = HashMap::new();
        aliases.insert("greeting".to_string(), "hello world".to_string());

        let v = infer_value("%greeting", Some(&aliases)).unwrap();
        assert!(matches!(v, Value::String(s) if s == "hello world"));
    }

    // ==================== Ditto in Key-Value context ====================

    #[test]
    fn test_infer_ditto_in_keyvalue_is_string() {
        let v = infer_value("^", None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "^"));
    }

    // ==================== Ditto in cell context ====================

    #[test]
    fn test_infer_cell_ditto() {
        let prev_row = vec![Value::Int(42), Value::String("test".to_string())];
        let v = infer_cell_value("^", 0, Some(&prev_row), None).unwrap();
        assert!(matches!(v, Value::Int(42)));
    }

    #[test]
    fn test_infer_cell_ditto_second_column() {
        let prev_row = vec![Value::Int(42), Value::String("test".to_string())];
        let v = infer_cell_value("^", 1, Some(&prev_row), None).unwrap();
        assert!(matches!(v, Value::String(s) if s == "test"));
    }

    #[test]
    fn test_infer_cell_ditto_bool() {
        let prev_row = vec![Value::Bool(true)];
        let v = infer_cell_value("^", 0, Some(&prev_row), None).unwrap();
        assert!(matches!(v, Value::Bool(true)));
    }

    #[test]
    fn test_infer_cell_ditto_null() {
        let prev_row = vec![Value::Null];
        let v = infer_cell_value("^", 0, Some(&prev_row), None).unwrap();
        assert!(matches!(v, Value::Null));
    }

    #[test]
    fn test_infer_cell_ditto_no_prev_row_error() {
        let result = infer_cell_value("^", 0, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_infer_cell_ditto_out_of_bounds() {
        let prev_row = vec![Value::Int(42)];
        // Column 1 doesn't exist in prev_row
        let result = infer_cell_value("^", 1, Some(&prev_row), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_infer_cell_regular_value() {
        let prev_row = vec![Value::Int(42)];
        // Regular value, not ditto
        let v = infer_cell_value("100", 0, Some(&prev_row), None).unwrap();
        assert!(matches!(v, Value::Int(100)));
    }

    // ==================== Tensor parsing ====================

    #[test]
    fn test_parse_tensor_simple() {
        let result = parse_tensor("[1, 2, 3]").unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_parse_tensor_nested() {
        let result = parse_tensor("[[1, 2], [3, 4]]").unwrap();
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], TensorValue::Array(_)));
    }

    #[test]
    fn test_parse_tensor_empty() {
        let result = parse_tensor("[]").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_tensor_deeply_nested() {
        let result = parse_tensor("[[[1, 2], [3, 4]], [[5, 6], [7, 8]]]").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_tensor_single_element() {
        let result = parse_tensor("[42]").unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], TensorValue::Number(n) if (n - 42.0).abs() < 0.001));
    }

    #[test]
    fn test_parse_tensor_floats() {
        let result = parse_tensor("[1.5, 2.5]").unwrap();
        assert_eq!(result.len(), 2);
        assert!(matches!(result[0], TensorValue::Number(n) if (n - 1.5).abs() < 0.001));
    }

    #[test]
    fn test_parse_tensor_unbalanced_error() {
        assert!(parse_tensor("[1, 2, [3, 4]").is_err());
        assert!(parse_tensor("[1, 2]]").is_err());
    }

    #[test]
    fn test_parse_tensor_invalid_content() {
        assert!(parse_tensor("[1, abc, 3]").is_err());
    }

    // ==================== Value enum tests ====================

    #[test]
    fn test_value_clone() {
        let v1 = Value::Int(42);
        let v2 = v1.clone();
        assert_eq!(v1, v2);

        let v1 = Value::String("test".to_string());
        let v2 = v1.clone();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_value_equality() {
        assert_eq!(Value::Null, Value::Null);
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(Value::Int(42), Value::Int(42));
        assert_ne!(Value::Int(42), Value::Int(43));
        assert_ne!(Value::Int(42), Value::Float(42.0));
    }

    #[test]
    fn test_tensor_value_equality() {
        let t1 = TensorValue::Number(1.0);
        let t2 = TensorValue::Number(1.0);
        assert_eq!(t1, t2);

        let t3 = TensorValue::Array(vec![TensorValue::Number(1.0)]);
        let t4 = TensorValue::Array(vec![TensorValue::Number(1.0)]);
        assert_eq!(t3, t4);
    }

    // ==================== Inference ladder order ====================

    #[test]
    fn test_inference_ladder_null_first() {
        // ~ should be null, not a string
        let v = infer_value("~", None).unwrap();
        assert!(matches!(v, Value::Null));
    }

    #[test]
    fn test_inference_ladder_tensor_before_string() {
        // [1] should be tensor, not string "[1]"
        let v = infer_value("[1]", None).unwrap();
        assert!(matches!(v, Value::Tensor(_)));
    }

    #[test]
    fn test_inference_ladder_reference_before_string() {
        // @id should be reference, not string "@id"
        let v = infer_value("@valid_id", None).unwrap();
        assert!(matches!(v, Value::Reference(_)));
    }

    #[test]
    fn test_inference_ladder_bool_before_string() {
        // true should be bool, not string "true"
        let v = infer_value("true", None).unwrap();
        assert!(matches!(v, Value::Bool(true)));
    }

    #[test]
    fn test_inference_ladder_number_before_string() {
        // 42 should be int, not string "42"
        let v = infer_value("42", None).unwrap();
        assert!(matches!(v, Value::Int(42)));
    }
}
