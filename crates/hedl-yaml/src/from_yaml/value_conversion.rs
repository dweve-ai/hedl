// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! YAML value to HEDL Value conversion

use crate::error::YamlError;
use crate::from_yaml::config::FromYamlConfig;
use crate::from_yaml::detection::{
    is_object_sequence, is_scalar_list_sequence, is_tensor_sequence,
};
use crate::from_yaml::tensor::yaml_sequence_to_tensor;
use hedl_core::convert::parse_reference;
use hedl_core::lex::parse_expression_token;
use hedl_core::Value;
use serde_yaml::Value as YamlValue;

pub(crate) fn yaml_to_value(
    value: &YamlValue,
    config: &FromYamlConfig,
    depth: usize,
) -> Result<Value, String> {
    // Check nesting depth
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
            path: "value".to_string(),
            context: None,
        }
        .to_string());
    }

    Ok(match value {
        YamlValue::Null => Value::Null,
        YamlValue::Bool(b) => Value::Bool(*b),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                return Err(format!("Invalid number: {n:?}"));
            }
        }
        YamlValue::String(s) => {
            // Check for expression pattern $( ... )
            if s.starts_with("$(") && s.ends_with(')') {
                let expr =
                    parse_expression_token(s).map_err(|e| format!("Invalid expression: {e}"))?;
                Value::Expression(Box::new(expr))
            } else {
                // Note: Strings that start with @ are just strings.
                // References use the @ref mapping format.
                Value::String(s.clone().into())
            }
        }
        YamlValue::Sequence(seq) => {
            // Check array length
            if seq.len() > config.max_array_length {
                return Err(YamlError::ArrayTooLong {
                    length: seq.len(),
                    max_length: config.max_array_length,
                    path: "value".to_string(),
                    context: None,
                }
                .to_string());
            }

            // Check if this is a sequence of mappings (nested children) - skip as Null
            // Child sequences are handled separately in yaml_sequence_to_matrix_list
            if is_object_sequence(seq) {
                Value::Null // Children processed by yaml_sequence_to_matrix_list
            } else if is_scalar_list_sequence(seq) {
                // Non-numeric sequence -> Value::List
                let items: Result<Vec<Value>, String> = seq
                    .iter()
                    .map(|v| yaml_to_value(v, config, depth + 1))
                    .collect();
                Value::List(Box::new(items?))
            } else if is_tensor_sequence(seq) {
                let tensor = yaml_sequence_to_tensor(seq, config, "tensor", depth + 1)?;
                Value::Tensor(Box::new(tensor))
            } else if seq.is_empty() {
                // Empty sequence -> empty list (changed from empty tensor for consistency)
                Value::List(Box::default())
            } else {
                // All numeric sequence -> tensor
                let tensor = yaml_sequence_to_tensor(seq, config, "tensor", depth + 1)?;
                Value::Tensor(Box::new(tensor))
            }
        }
        YamlValue::Mapping(map) => {
            // Check for reference marker (@ref key)
            if let Some(YamlValue::String(ref_str)) = map.get(YamlValue::String("@ref".to_string()))
            {
                Value::Reference(parse_reference(ref_str)?)
            } else {
                return Err("Nested objects not allowed in scalar context".into());
            }
        }
        YamlValue::Tagged(tagged) => {
            return yaml_to_value(&tagged.value, config, depth);
        }
    })
}
