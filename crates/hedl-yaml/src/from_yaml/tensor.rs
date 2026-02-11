// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tensor conversion from YAML sequences

use crate::error::YamlError;
use crate::from_yaml::config::FromYamlConfig;
use hedl_core::lex::Tensor;
use serde_yaml::Value as YamlValue;

pub(crate) fn yaml_sequence_to_tensor(
    seq: &[YamlValue],
    config: &FromYamlConfig,
    path: &str,
    depth: usize,
) -> Result<Tensor, String> {
    // Check nesting depth
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
            path: path.to_string(),
            context: None,
        }
        .to_string());
    }

    // Check array length
    if seq.len() > config.max_array_length {
        return Err(YamlError::ArrayTooLong {
            length: seq.len(),
            max_length: config.max_array_length,
            path: path.to_string(),
            context: None,
        }
        .to_string());
    }

    // Convert YAML sequence to Tensor recursively
    let items: Result<Vec<Tensor>, String> = seq
        .iter()
        .map(|v| match v {
            YamlValue::Number(n) => n
                .as_f64()
                .map(Tensor::Scalar)
                .ok_or_else(|| format!("Invalid tensor number: {n:?}")),
            YamlValue::Sequence(nested) => yaml_sequence_to_tensor(nested, config, path, depth + 1),
            _ => Err("Invalid tensor element - must be number or sequence".into()),
        })
        .collect();

    Ok(Tensor::Array(items?))
}
