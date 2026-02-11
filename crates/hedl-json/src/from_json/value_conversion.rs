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

//! JSON value to HEDL value conversion
//!
//! Functions for converting individual JSON values to HEDL Value types.

use super::array_conversion::{
    is_object_array, is_tensor_array, json_array_to_list, json_array_to_tensor,
};
use super::config::{json_number_to_value, FromJsonConfig, JsonConversionError};
use hedl_core::convert::parse_reference;
use hedl_core::lex::parse_expression_token;
use hedl_core::Value;
use serde_json::Value as JsonValue;

/// Convert a JSON value to a HEDL scalar Value (no recursion into objects/arrays).
pub fn json_scalar_to_value(
    value: &JsonValue,
    config: &FromJsonConfig,
) -> Result<Value, JsonConversionError> {
    match value {
        JsonValue::Null => Ok(Value::Null),
        JsonValue::Bool(b) => Ok(Value::Bool(*b)),
        JsonValue::Number(n) => json_number_to_value(n),
        JsonValue::String(s) => {
            if let Some(max_len) = config.max_string_length {
                if s.len() > max_len {
                    return Err(JsonConversionError::MaxStringLengthExceeded(
                        max_len,
                        s.len(),
                    ));
                }
            }
            if s.starts_with("$(") && s.ends_with(')') {
                let expr = parse_expression_token(s)
                    .map_err(|e| JsonConversionError::InvalidExpression(e.to_string()))?;
                Ok(Value::Expression(Box::new(expr)))
            } else {
                Ok(Value::String(s.clone().into_boxed_str()))
            }
        }
        JsonValue::Array(arr) => {
            // For arrays in scalar context, convert to tensor if numeric, else JSON string
            if is_tensor_array(arr) {
                let tensor = json_array_to_tensor(arr, config, 0)?;
                Ok(Value::Tensor(Box::new(tensor)))
            } else {
                // Serialize as JSON string for non-numeric arrays
                Ok(Value::String(
                    serde_json::to_string(value)
                        .unwrap_or_else(|_| "[]".to_string())
                        .into_boxed_str(),
                ))
            }
        }
        JsonValue::Object(obj) => {
            // For nested objects in scalar context, serialize as JSON string
            // unless it's a reference
            if let Some(JsonValue::String(r)) = obj.get("@ref") {
                Ok(Value::Reference(
                    parse_reference(r).map_err(JsonConversionError::InvalidReference)?,
                ))
            } else {
                Ok(Value::String(
                    serde_json::to_string(value)
                        .unwrap_or_else(|_| "{}".to_string())
                        .into_boxed_str(),
                ))
            }
        }
    }
}

/// Convert JSON value to HEDL Value for use in MatrixList fields.
///
/// This is the primary value conversion function used when populating MatrixList rows.
/// It handles all JSON types and converts them appropriately to HEDL values.
pub fn json_to_value(
    value: &JsonValue,
    config: &FromJsonConfig,
) -> Result<Value, JsonConversionError> {
    Ok(match value {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Number(n) => json_number_to_value(n)?,
        JsonValue::String(s) => {
            // Check string length limit
            if let Some(max_len) = config.max_string_length {
                if s.len() > max_len {
                    return Err(JsonConversionError::MaxStringLengthExceeded(
                        max_len,
                        s.len(),
                    ));
                }
            }

            // Check for expression pattern $( ... )
            if s.starts_with("$(") && s.ends_with(')') {
                let expr = parse_expression_token(s)
                    .map_err(|e| JsonConversionError::InvalidExpression(e.to_string()))?;
                Value::Expression(Box::new(expr))
            } else {
                Value::String(s.clone().into_boxed_str())
            }
        }
        JsonValue::Array(arr) => {
            // Check array size limit
            if let Some(max_size) = config.max_array_size {
                if arr.len() > max_size {
                    return Err(JsonConversionError::MaxArraySizeExceeded(
                        max_size,
                        arr.len(),
                    ));
                }
            }

            // Check if this is an array of objects (nested children) - skip as Null
            // Child arrays are handled separately in json_array_to_matrix_list
            if is_object_array(arr) {
                Value::Null // Children processed by json_array_to_matrix_list
            } else if is_tensor_array(arr) {
                let tensor = json_array_to_tensor(arr, config, 0)?;
                Value::Tensor(Box::new(tensor))
            } else if arr.is_empty() {
                // Empty array → empty list
                Value::List(Box::default())
            } else {
                // Non-numeric array (strings, bools, nulls, mixed types) → Value::List
                json_array_to_list(arr, config)?
            }
        }
        JsonValue::Object(obj) => {
            if let Some(JsonValue::String(r)) = obj.get("@ref") {
                Value::Reference(parse_reference(r).map_err(JsonConversionError::InvalidReference)?)
            } else {
                return Err(JsonConversionError::NestedObject);
            }
        }
    })
}
