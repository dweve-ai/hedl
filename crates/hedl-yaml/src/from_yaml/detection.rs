// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Type detection for YAML sequences

use serde_yaml::Value as YamlValue;

pub(crate) fn is_tensor_sequence(seq: &[YamlValue]) -> bool {
    // Empty sequences are not tensors - they're empty matrix lists
    !seq.is_empty()
        && seq
            .iter()
            .all(|v| matches!(v, YamlValue::Number(_) | YamlValue::Sequence(_)))
}

pub(crate) fn is_object_sequence(seq: &[YamlValue]) -> bool {
    !seq.is_empty()
        && seq.iter().all(|v| {
            if let YamlValue::Mapping(m) = v {
                // Exclude reference mappings (they have @ref key)
                !m.contains_key(YamlValue::String("@ref".to_string()))
            } else {
                false
            }
        })
}

/// Checks if a YAML sequence should be converted to a Value::List.
///
/// Returns true if the sequence contains non-numeric scalar values (strings, bools, nulls)
/// or references, indicating it should become a List rather than a Tensor.
pub(crate) fn is_scalar_list_sequence(seq: &[YamlValue]) -> bool {
    !seq.is_empty()
        && seq.iter().any(|v| {
            matches!(
                v,
                YamlValue::String(_) | YamlValue::Bool(_) | YamlValue::Null
            ) || (matches!(v, YamlValue::Mapping(m) if m.contains_key(YamlValue::String("@ref".to_string()))))
        })
}
