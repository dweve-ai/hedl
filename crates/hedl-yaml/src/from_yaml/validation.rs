// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Row validation and nested children processing

use crate::error::YamlError;
use crate::from_yaml::config::FromYamlConfig;
use crate::from_yaml::detection::is_object_sequence;
use crate::from_yaml::matrix_list::yaml_sequence_to_matrix_list;
use crate::from_yaml::schema::{build_matrix_columns, extract_row_id};
use hedl_core::Node;
use serde_yaml::{Mapping, Value as YamlValue};
use std::collections::BTreeMap;

pub(crate) fn process_nested_children(
    map: &Mapping,
    key: &str,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<BTreeMap<String, Vec<Node>>, String> {
    let mut children: BTreeMap<String, Vec<Node>> = BTreeMap::new();

    for (child_key, child_value) in map {
        if let (Some(child_key_str), YamlValue::Sequence(child_seq)) =
            (child_key.as_str(), child_value)
        {
            if is_object_sequence(child_seq) {
                // Check child array length
                if child_seq.len() > config.max_array_length {
                    return Err(YamlError::ArrayTooLong {
                        length: child_seq.len(),
                        max_length: config.max_array_length,
                        path: format!("{key}.{child_key_str}"),
                        context: None,
                    }
                    .to_string());
                }
                // This is a nested child list
                let child_list = yaml_sequence_to_matrix_list(
                    child_seq,
                    child_key_str,
                    config,
                    structs,
                    depth + 1,
                )?;
                children.insert(child_key_str.to_string(), child_list.rows);
            }
        }
    }

    Ok(children)
}

/// Parameters for converting a single YAML mapping into a HEDL Node.
///
/// Bundles the arguments needed by [`convert_sequence_item`] to keep
/// the public signature concise.
pub(crate) struct ConvertItemParams<'a> {
    /// The YAML mapping to convert
    pub(crate) map: &'a Mapping,
    /// The HEDL type name for this node
    pub(crate) type_name: &'a str,
    /// The column schema defining fields
    pub(crate) schema: &'a [String],
    /// The parent key (for error messages)
    pub(crate) key: &'a str,
    /// Configuration with depth/size limits
    pub(crate) config: &'a FromYamlConfig,
    /// Current nesting depth
    pub(crate) depth: usize,
}

/// Converts a single mapping in a sequence to a Node.
///
/// This function extracts all necessary data (ID, fields, children) from a
/// mapping and constructs a Node for inclusion in a matrix list.
///
/// # Arguments
///
/// * `params` - Bundled conversion parameters (see [`ConvertItemParams`])
/// * `structs` - Structure registry for child types (mutably borrowed separately
///   because it is updated during conversion)
///
/// # Returns
///
/// A Node with all fields and children properly populated, or an error if conversion fails.
pub(crate) fn convert_sequence_item(
    params: &ConvertItemParams<'_>,
    structs: &mut BTreeMap<String, Vec<String>>,
) -> Result<Node, String> {
    let id = extract_row_id(params.map, params.schema);
    let fields = build_matrix_columns(params.map, params.schema, params.config, params.depth)?;
    let children =
        process_nested_children(params.map, params.key, params.config, structs, params.depth)?;

    Ok(Node {
        type_name: params.type_name.to_string(),
        id,
        fields: fields.into(),
        children: if children.is_empty() {
            None
        } else {
            Some(Box::new(children))
        },
        child_count: 0,
    })
}

/// Parameters for validating and converting all sequence items to Nodes.
///
/// Bundles the arguments needed by [`validate_row_structure`] to keep
/// the public signature concise.
pub(crate) struct ValidateRowParams<'a> {
    /// The YAML sequence to process
    pub(crate) seq: &'a [YamlValue],
    /// The HEDL type name for nodes
    pub(crate) type_name: &'a str,
    /// The column schema defining fields
    pub(crate) schema: &'a [String],
    /// The parent key (for error messages)
    pub(crate) key: &'a str,
    /// Configuration with depth/size limits
    pub(crate) config: &'a FromYamlConfig,
    /// Current nesting depth
    pub(crate) depth: usize,
}

/// Validates row structure and converts all sequence items to Nodes.
///
/// This function processes all mappings in a sequence, validating them and
/// converting to Nodes. It filters out non-mapping items and collects all rows.
///
/// # Arguments
///
/// * `params` - Bundled validation parameters (see [`ValidateRowParams`])
/// * `structs` - Structure registry for child types (mutably borrowed separately
///   because it is updated during conversion)
///
/// # Returns
///
/// A vector of Nodes, or an error if any row conversion fails.
pub(crate) fn validate_row_structure(
    params: &ValidateRowParams<'_>,
    structs: &mut BTreeMap<String, Vec<String>>,
) -> Result<Vec<Node>, String> {
    // OPTIMIZATION: Pre-allocate capacity based on sequence length
    let mut rows = Vec::with_capacity(params.seq.len());
    for item in params.seq {
        if let YamlValue::Mapping(map) = item {
            let convert_params = ConvertItemParams {
                map,
                type_name: params.type_name,
                schema: params.schema,
                key: params.key,
                config: params.config,
                depth: params.depth,
            };
            let node = convert_sequence_item(&convert_params, structs)?;
            rows.push(node);
        }
    }
    Ok(rows)
}
