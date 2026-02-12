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

//! Conversion utilities between lexer and value types.

use crate::error::HedlResult;
use crate::value::{Reference, Value};

/// Convert a lex_inference::TensorValue to tensor::Tensor.
pub(super) fn convert_tensor_value(tv: Vec<crate::lex::TensorValue>) -> Vec<crate::lex::Tensor> {
    tv.into_iter()
        .map(|item| match item {
            crate::lex::TensorValue::Number(n) => crate::lex::Tensor::Scalar(n),
            crate::lex::TensorValue::Array(arr) => {
                crate::lex::Tensor::Array(convert_tensor_value(arr))
            }
        })
        .collect()
}

/// Convert a lex_inference::Value to value::Value.
///
/// This bridges the lexer-level Value type (used during parsing) with the
/// document-level Value type (used in the final AST).
pub(super) fn convert_lex_value_to_value(lex_val: crate::lex::Value) -> HedlResult<Value> {
    match lex_val {
        crate::lex::Value::Null => Ok(Value::Null),
        crate::lex::Value::Bool(b) => Ok(Value::Bool(b)),
        crate::lex::Value::Int(i) => Ok(Value::Int(i)),
        crate::lex::Value::Float(f) => Ok(Value::Float(f)),
        crate::lex::Value::String(s) => Ok(Value::String(s.into_boxed_str())),
        crate::lex::Value::Reference(r) => Ok(Value::Reference(Reference {
            type_name: r.type_name.map(|t| t.into_boxed_str()),
            id: r.id.into_boxed_str(),
        })),
        crate::lex::Value::Expression(e) => Ok(Value::Expression(Box::new(e))),
        crate::lex::Value::Tensor(tv) => {
            // Convert Vec<TensorValue> to Tensor
            let tensors = convert_tensor_value(tv);
            // Wrap in Array if multiple elements, or use the single element
            let tensor = if tensors.len() == 1 {
                // SAFETY: len == 1 guarantees next() returns Some
                tensors.into_iter().next().expect("single-element vec")
            } else {
                crate::lex::Tensor::Array(tensors)
            };
            Ok(Value::Tensor(Box::new(tensor)))
        }
        crate::lex::Value::List(items) => {
            // Recursively convert list items
            let converted_items: Result<Vec<Value>, crate::error::HedlError> =
                items.into_iter().map(convert_lex_value_to_value).collect();
            Ok(Value::List(Box::new(converted_items?)))
        }
    }
}
