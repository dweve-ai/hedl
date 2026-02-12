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

//! Value inference and type conversion
//!
//! Handles automatic type inference from unquoted string values
//! and conversion between lexer-level and document-level value representations.

use crate::error::{StreamError, StreamResult};
use crate::event::HeaderInfo;
use hedl_core::Value;

/// Infer a Value from an unquoted string, handling special literals
/// like null, booleans, references, aliases, and numbers.
pub(crate) fn infer_value(
    s: &str,
    line_num: usize,
    header: Option<&HeaderInfo>,
) -> StreamResult<Value> {
    let s = s.trim();

    // Handle null values: empty, ~, or the keyword "null"
    if s.is_empty() || s == "~" || s == "null" {
        return Ok(Value::Null);
    }

    if s == "true" {
        return Ok(Value::Bool(true));
    }
    if s == "false" {
        return Ok(Value::Bool(false));
    }

    // List literal: (...)
    if s.starts_with('(') && s.ends_with(')') {
        match hedl_core::lex::parse_list_literal(s, 0) {
            Ok((lex_value, _)) => {
                // Convert lex::Value::List to Value::List
                if let hedl_core::lex::Value::List(items) = lex_value {
                    let converted_items: Result<Vec<Value>, StreamError> = items
                        .into_iter()
                        .map(|item| convert_lex_value(item, line_num))
                        .collect();
                    return Ok(Value::List(Box::new(converted_items?)));
                }
            }
            Err(_) => {
                // If list parsing fails, fall through to other inference
            }
        }
    }

    // Reference
    if let Some(ref_part) = s.strip_prefix('@') {
        if let Some(colon_pos) = ref_part.find(':') {
            let type_name = &ref_part[..colon_pos];
            let id = &ref_part[colon_pos + 1..];
            return Ok(Value::Reference(hedl_core::Reference {
                type_name: Some(type_name.to_string().into()),
                id: id.to_string().into(),
            }));
        }
        return Ok(Value::Reference(hedl_core::Reference {
            type_name: None,
            id: ref_part.to_string().into(),
        }));
    }

    // Alias
    if let Some(alias) = s.strip_prefix('$') {
        if let Some(header) = header {
            if let Some(value) = header.aliases.get(alias) {
                return Ok(Value::String(value.clone().into()));
            }
        }
        return Ok(Value::String(s.to_string().into()));
    }

    // Number
    if let Ok(i) = s.parse::<i64>() {
        return Ok(Value::Int(i));
    }
    if let Ok(f) = s.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    // Default to string
    Ok(Value::String(s.to_string().into()))
}

/// Convert a lex::Value to the main Value type.
///
/// This handles the conversion from the lexer-level value representation
/// to the document-level value representation used in streaming events.
pub(crate) fn convert_lex_value(
    lex_val: hedl_core::lex::Value,
    line_num: usize,
) -> StreamResult<Value> {
    match lex_val {
        hedl_core::lex::Value::Null => Ok(Value::Null),
        hedl_core::lex::Value::Bool(b) => Ok(Value::Bool(b)),
        hedl_core::lex::Value::Int(i) => Ok(Value::Int(i)),
        hedl_core::lex::Value::Float(f) => Ok(Value::Float(f)),
        hedl_core::lex::Value::String(s) => Ok(Value::String(s.into_boxed_str())),
        hedl_core::lex::Value::Reference(r) => Ok(Value::Reference(hedl_core::Reference {
            type_name: r.type_name.map(|t| t.into_boxed_str()),
            id: r.id.into_boxed_str(),
        })),
        hedl_core::lex::Value::Expression(e) => Ok(Value::Expression(Box::new(e))),
        hedl_core::lex::Value::Tensor(_) => Err(StreamError::syntax(
            line_num,
            "tensors not supported in list literals",
        )),
        hedl_core::lex::Value::List(items) => {
            let converted_items: Result<Vec<Value>, StreamError> = items
                .into_iter()
                .map(|item| convert_lex_value(item, line_num))
                .collect();
            Ok(Value::List(Box::new(converted_items?)))
        }
    }
}
