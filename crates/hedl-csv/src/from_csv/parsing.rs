// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Value parsing from CSV fields

use crate::error::{CsvError, Result};
use crate::from_csv::schema_inference::ColumnType;
use hedl_core::lex::{parse_expression_token, parse_tensor};
use hedl_core::Value;

pub(crate) fn parse_csv_value_with_type(field: &str, col_type: ColumnType) -> Result<Value> {
    let trimmed = field.trim();

    // Always handle null values regardless of inferred type
    if trimmed.is_empty() || trimmed == "~" {
        return Ok(Value::Null);
    }

    match col_type {
        ColumnType::Null => Ok(Value::Null),
        ColumnType::Bool => {
            if trimmed == "true" {
                Ok(Value::Bool(true))
            } else if trimmed == "false" {
                Ok(Value::Bool(false))
            } else {
                // Fallback to string if not a valid bool
                Ok(Value::String(field.to_string().into()))
            }
        }
        ColumnType::Int => {
            if let Ok(n) = trimmed.parse::<i64>() {
                Ok(Value::Int(n))
            } else {
                // Fallback to string if not a valid int
                Ok(Value::String(field.to_string().into()))
            }
        }
        ColumnType::Float => {
            if let Ok(f) = trimmed.parse::<f64>() {
                Ok(Value::Float(f))
            } else {
                // Fallback to string if not a valid float
                Ok(Value::String(field.to_string().into()))
            }
        }
        ColumnType::String => {
            // Use the original parse_csv_value for full type detection
            // (handles references, expressions, tensors, etc.)
            parse_csv_value(field)
        }
    }
}

/// Validate CSV headers against security limits.
///
/// This function checks:
/// - Column count does not exceed `max_columns`
/// - Total header size does not exceed `max_header_size`
/// - Individual column name size does not exceed `max_cell_size`
///
/// # Arguments
///
/// * `headers` - The CSV header record
/// * `config` - Configuration containing security limits
///
/// # Returns
///
/// `Ok(())` if all checks pass, otherwise an error.
pub(crate) fn parse_csv_value(field: &str) -> Result<Value> {
    let trimmed = field.trim();

    // Empty or null
    if trimmed.is_empty() || trimmed == "~" {
        return Ok(Value::Null);
    }

    // Boolean
    if trimmed == "true" {
        return Ok(Value::Bool(true));
    }
    if trimmed == "false" {
        return Ok(Value::Bool(false));
    }

    // Special float values
    match trimmed {
        "NaN" => return Ok(Value::Float(f64::NAN)),
        "Infinity" => return Ok(Value::Float(f64::INFINITY)),
        "-Infinity" => return Ok(Value::Float(f64::NEG_INFINITY)),
        _ => {}
    }

    // Reference
    if trimmed.starts_with('@') {
        return parse_reference(trimmed);
    }

    // Expression
    if trimmed.starts_with("$(") && trimmed.ends_with(')') {
        let expr = parse_expression_token(trimmed).map_err(|e| CsvError::ParseError {
            line: 0,
            message: format!("Invalid expression: {e}"),
        })?;
        return Ok(Value::Expression(Box::new(expr)));
    }

    // List literal (starts with '(' and ends with ')' but not an expression)
    if trimmed.starts_with('(') && trimmed.ends_with(')') && !trimmed.starts_with("$(") {
        return parse_list_value(trimmed);
    }

    // Try integer
    if let Ok(n) = trimmed.parse::<i64>() {
        return Ok(Value::Int(n));
    }

    // Try float
    if let Ok(f) = trimmed.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    // Tensor literal (starts with '[' and ends with ']')
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        if let Ok(tensor) = parse_tensor(trimmed) {
            return Ok(Value::Tensor(Box::new(tensor)));
        }
        // If parsing fails, fall through to string
    }

    // Default to string
    Ok(Value::String(field.to_string().into()))
}

/// Parse a reference string (e.g., "@user1" or "@User:user1").
pub(crate) fn parse_reference(s: &str) -> Result<Value> {
    let without_at = &s[1..];

    if let Some(colon_pos) = without_at.find(':') {
        // Qualified reference:@Type:id
        let type_name = &without_at[..colon_pos];
        let id = &without_at[colon_pos + 1..];

        if type_name.is_empty() || id.is_empty() {
            return Err(CsvError::ParseError {
                line: 0,
                message: format!("Invalid reference format: {s}"),
            });
        }

        Ok(Value::Reference(hedl_core::Reference::qualified(
            type_name, id,
        )))
    } else {
        // Local reference:@id
        if without_at.is_empty() {
            return Err(CsvError::ParseError {
                line: 0,
                message: "Empty reference ID".to_string(),
            });
        }

        Ok(Value::Reference(hedl_core::Reference::local(without_at)))
    }
}

/// Parse a list value from CSV (e.g., "(admin, editor, viewer)" or "(true, false)").
///
/// Lists use parentheses syntax and contain comma-separated values.
/// Each element is recursively parsed using the same value parsing logic.
///
/// # Examples
///
/// - `()` → empty list
/// - `(admin, editor)` → list of two strings
/// - `(true, false, true)` → list of three bools
/// - `(@user1, @user2)` → list of two references
/// - `(1, 2, 3)` → list of three integers
fn parse_list_value(s: &str) -> Result<Value> {
    // Remove opening '(' and closing ')'
    let inner = &s[1..s.len() - 1];
    let trimmed_inner = inner.trim();

    // Handle empty list
    if trimmed_inner.is_empty() {
        return Ok(Value::List(Box::default()));
    }

    // Split by comma and parse each element
    let mut items = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_expr = false;

    for ch in trimmed_inner.chars() {
        match ch {
            '(' if !in_expr => paren_depth += 1,
            ')' if !in_expr => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            '$' if current.trim().is_empty() => in_expr = true,
            ',' if paren_depth == 0 && bracket_depth == 0 && !in_expr => {
                // Found a separator at the top level
                let item_str = current.trim();
                if !item_str.is_empty() {
                    items.push(parse_csv_value(item_str)?);
                }
                current.clear();
                continue;
            }
            _ => {}
        }

        // Reset in_expr if we close the expression
        if in_expr && ch == ')' && paren_depth == 0 {
            in_expr = false;
        }

        current.push(ch);
    }

    // Parse the last item
    let item_str = current.trim();
    if !item_str.is_empty() {
        items.push(parse_csv_value(item_str)?);
    }

    Ok(Value::List(Box::new(items)))
}
