// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Matrix list conversion from YAML sequences

use crate::from_yaml::config::FromYamlConfig;
use crate::from_yaml::schema::infer_row_schema;
use crate::from_yaml::validation::{validate_row_structure, ValidateRowParams};
use crate::YamlError;
use hedl_core::lex::singularize_and_capitalize;
use hedl_core::MatrixList;
use serde_yaml::Value as YamlValue;
use std::collections::BTreeMap;

pub(crate) fn yaml_sequence_to_matrix_list_with_schema(
    seq: &[YamlValue],
    key: &str,
    schema: Vec<String>,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<MatrixList, String> {
    // Validate nesting depth and array length
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
            path: key.to_string(),
            context: None,
        }
        .to_string());
    }
    if seq.len() > config.max_array_length {
        return Err(YamlError::ArrayTooLong {
            length: seq.len(),
            max_length: config.max_array_length,
            path: key.to_string(),
            context: None,
        }
        .to_string());
    }

    // Try to extract type_name from metadata (__type__ field in first row)
    let type_name = if let Some(YamlValue::Mapping(first_map)) = seq.first() {
        if let Some(YamlValue::String(type_str)) =
            first_map.get(YamlValue::String("__type__".to_string()))
        {
            type_str.clone()
        } else {
            singularize_and_capitalize(key)
        }
    } else {
        singularize_and_capitalize(key)
    };

    // Register the struct definition with explicit schema
    structs.insert(type_name.clone(), schema.clone());

    // Process all rows with the explicit schema
    let params = ValidateRowParams {
        seq,
        type_name: &type_name,
        schema: &schema,
        key,
        config,
        depth,
    };
    let rows = validate_row_structure(&params, structs)?;

    Ok(MatrixList {
        type_name,
        schema,
        rows,
        count_hint: None,
    })
}

/// Converts a YAML sequence to a HEDL `MatrixList`.
///
/// This function transforms a sequence of YAML mappings into a structured HEDL
/// `MatrixList`, inferring the schema from the first element. Child sequences are
/// recursively converted to nested matrix lists.
///
/// # How it works
///
/// 1. Validates nesting depth and array length constraints
/// 2. Infers type name from the key (singularized and capitalized)
/// 3. Infers schema from the first mapping, excluding child sequences
/// 4. Converts each mapping to a Node with fields and nested children
/// 5. Returns a `MatrixList` with all rows
///
/// # Arguments
///
/// * `seq` - The YAML sequence of mappings to convert
/// * `key` - The field name (used for type name inference)
/// * `config` - Configuration with depth/size limits
/// * `structs` - Structure registry (updated with new type definitions)
/// * `depth` - Current nesting depth for validation
///
/// # Examples
///
/// ```text
/// YAML:
/// users:
///   - id: u1
///     name: Alice
///   - id: u2
///     name: Bob
///
/// Result: MatrixList {
///     type_name: "User",
///     schema: ["id", "name"],
///     rows: [Node{id: "u1", ...}, Node{id: "u2", ...}]
/// }
/// ```
pub(crate) fn yaml_sequence_to_matrix_list(
    seq: &[YamlValue],
    key: &str,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<MatrixList, String> {
    // Validate nesting depth and array length
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
            path: key.to_string(),
            context: None,
        }
        .to_string());
    }
    if seq.len() > config.max_array_length {
        return Err(YamlError::ArrayTooLong {
            length: seq.len(),
            max_length: config.max_array_length,
            path: key.to_string(),
            context: None,
        }
        .to_string());
    }

    // Try to extract type_name from metadata (__type__ field in first row)
    // This preserves type_name during YAML roundtrip when include_metadata is used
    let type_name = if let Some(YamlValue::Mapping(first_map)) = seq.first() {
        if let Some(YamlValue::String(type_str)) =
            first_map.get(YamlValue::String("__type__".to_string()))
        {
            type_str.clone()
        } else {
            // Fallback to inferring from key
            singularize_and_capitalize(key)
        }
    } else {
        // Empty sequence or non-mapping first element - infer from key
        singularize_and_capitalize(key)
    };

    let schema = infer_row_schema(seq);

    // Register the struct definition
    structs.insert(type_name.clone(), schema.clone());

    // Process all rows and collect
    let params = ValidateRowParams {
        seq,
        type_name: &type_name,
        schema: &schema,
        key,
        config,
        depth,
    };
    let rows = validate_row_structure(&params, structs)?;

    Ok(MatrixList {
        type_name,
        schema,
        rows,
        count_hint: None,
    })
}
