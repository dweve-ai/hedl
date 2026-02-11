// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Schema inference and matrix list column building

use crate::from_yaml::config::FromYamlConfig;
use crate::from_yaml::value_conversion::yaml_to_value;
use crate::DEFAULT_SCHEMA;
use hedl_core::Value;
use serde_yaml::{Mapping, Value as YamlValue};

pub(crate) fn infer_row_schema(seq: &[YamlValue]) -> Vec<String> {
    if let Some(YamlValue::Mapping(first)) = seq.first() {
        // OPTIMIZATION: Pre-allocate with estimated capacity to reduce reallocations
        let mut keys: Vec<String> = Vec::with_capacity(first.len());

        for (k, v) in first {
            let key_str = match k.as_str() {
                Some(s) => s,
                None => continue,
            };

            // Skip ONLY known HEDL metadata keys (not all keys starting with __)
            // This allows columns named "__", "___foo", etc. while filtering metadata
            if key_str == "__type__" || key_str == "__schema__" {
                continue;
            }

            // Exclude ALL sequences from schema - they represent either:
            // 1. Non-empty object sequences: nested children (matrix lists)
            // 2. Empty sequences: will become empty children, not fields
            // 3. Tensor sequences: stored separately, not in schema
            // This prevents empty child arrays from polluting the schema.
            if matches!(v, YamlValue::Sequence(_)) {
                continue;
            }

            keys.push(key_str.to_string());
        }

        keys.sort_unstable(); // OPTIMIZATION: Use unstable sort (faster for strings)

        // Ensure "id" is first if present
        if let Some(pos) = keys.iter().position(|k| k == "id") {
            keys.remove(pos);
            keys.insert(0, "id".to_string());
        }
        keys
    } else {
        DEFAULT_SCHEMA.iter().map(|s| (*s).to_string()).collect()
    }
}

/// Extracts field values from a mapping for a given schema.
///
/// This function retrieves all field values in schema order, converting each
/// YAML value to a HEDL Value. Missing fields are represented as `Value::Null`.
///
/// # Arguments
///
/// * `map` - The YAML mapping containing the row data
/// * `schema` - The column schema defining which fields to extract
/// * `config` - Configuration with depth/size limits
/// * `depth` - Current nesting depth
///
/// # Returns
///
/// A vector of HEDL Values in schema column order, or an error if conversion fails.
///
/// # Examples
///
/// ```text
/// Schema: ["id", "name"]
/// Mapping: { id: "u1", name: "Alice" }
/// Result: [Value::String("u1"), Value::String("Alice")]
/// ```
pub(crate) fn build_matrix_columns(
    map: &Mapping,
    schema: &[String],
    config: &FromYamlConfig,
    depth: usize,
) -> Result<Vec<Value>, String> {
    // OPTIMIZATION: Pre-allocate exact capacity needed
    let mut fields = Vec::with_capacity(schema.len());
    for col in schema {
        let value = map
            .get(YamlValue::String(col.clone()))
            .map(|v| yaml_to_value(v, config, depth + 1))
            .transpose()?
            .unwrap_or(Value::Null);
        fields.push(value);
    }
    Ok(fields)
}

/// Extracts the ID value from a mapping using the first schema column.
///
/// The ID is always derived from the first column in the schema. If missing
/// or not a string, an empty string is used as the default ID.
///
/// # Arguments
///
/// * `map` - The YAML mapping containing the row data
/// * `schema` - The column schema (first element is used for ID extraction)
///
/// # Returns
///
/// The ID string, or empty string if not found or not a string type.
///
/// # Examples
///
/// ```text
/// Schema: ["id", "name"]
/// Mapping: { id: "u1", name: "Alice" }
/// Result: "u1"
/// ```
///
/// # Optimization
///
/// Efficiently extracts the ID field from a YAML mapping according to the provided schema.
pub(crate) fn extract_row_id(map: &Mapping, schema: &[String]) -> String {
    map.get(YamlValue::String(schema[0].clone()))
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string) // Explicitly convert &str to String
        .unwrap_or_default()
}
