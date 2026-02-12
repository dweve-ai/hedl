// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Main YAML to HEDL conversion logic

use crate::anchors::detect_cycles;
use crate::error::YamlError;
use crate::from_yaml::cache::{AnchorCache, CycleDetector, ItemOrRef};
use crate::from_yaml::config::FromYamlConfig;
use crate::from_yaml::detection::{
    is_object_sequence, is_scalar_list_sequence, is_tensor_sequence,
};
use crate::from_yaml::matrix_list::{
    yaml_sequence_to_matrix_list, yaml_sequence_to_matrix_list_with_schema,
};
use crate::from_yaml::tensor::yaml_sequence_to_tensor;
use crate::from_yaml::value_conversion::yaml_to_value;
use crate::yaml_scanner::scan_yaml_anchors;
use hedl_core::convert::parse_reference;
use hedl_core::lex::{parse_expression_token, singularize_and_capitalize};
use hedl_core::{Document, Item, MatrixList, Value};
use serde_yaml::{Mapping, Value as YamlValue};
use std::collections::BTreeMap;

/// Convert YAML string to HEDL Document
///
/// # Security
///
/// This function enforces resource limits to prevent `DoS` attacks:
/// - Checks document size against `max_document_size`
/// - Enforces array length limits during conversion
/// - Enforces nesting depth limits during conversion
///
/// # Errors
///
/// Returns an error if:
/// - The YAML document exceeds `max_document_size`
/// - Any array exceeds `max_array_length`
/// - Nesting depth exceeds `max_nesting_depth`
/// - The YAML is malformed or cannot be parsed
pub fn from_yaml(yaml: &str, config: &FromYamlConfig) -> Result<Document, String> {
    // Check document size before parsing
    if yaml.len() > config.max_document_size {
        return Err(YamlError::DocumentTooLarge {
            size: yaml.len(),
            max_size: config.max_document_size,
            context: None,
        }
        .to_string());
    }

    // Phase 1: Scan for anchors and aliases, build registry, detect cycles
    let anchor_registry = scan_yaml_anchors(yaml).map_err(|e| e.to_string())?;
    detect_cycles(&anchor_registry).map_err(|e| e.to_string())?;

    // Phase 2: Parse YAML with serde_yaml
    // serde_yaml will also detect some issues (forward refs, cycles via recursion limit)
    // We enhance its error messages to be more user-friendly
    let value: YamlValue = serde_yaml::from_str(yaml).map_err(|e| {
        let err_msg = e.to_string();

        // Enhance error messages for common anchor/alias issues
        if err_msg.contains("unknown anchor") {
            format!(
                "Forward reference detected: {err_msg}. In YAML, anchors must be defined before aliases that reference them."
            )
        } else if err_msg.contains("recursion limit") {
            format!(
                "Circular reference detected: {err_msg}. YAML anchors cannot form circular references."
            )
        } else {
            format!("YAML parse error: {err_msg}")
        }
    })?;

    // Phase 3: Convert to HEDL
    // Note: serde_yaml automatically resolves aliases to their anchor values,
    // so we can't preserve them as HEDL References in the current implementation.
    // The anchor_registry is used for validation only (cycle detection, forward ref detection).
    // Future enhancement: use a different YAML parser that preserves anchor/alias structure.
    from_yaml_value(&value, config)
}

/// Convert `serde_yaml::Value` to HEDL Document
pub fn from_yaml_value(value: &YamlValue, config: &FromYamlConfig) -> Result<Document, String> {
    let mut structs = BTreeMap::new();
    let mut cache = AnchorCache::new();
    let mut cycle_detector = CycleDetector::new();

    let root = match value {
        YamlValue::Mapping(map) => yaml_mapping_to_root(
            map,
            config,
            &mut structs,
            &mut cache,
            &mut cycle_detector,
            0,
        )?,
        _ => return Err("Root must be a YAML mapping".into()),
    };

    Ok(Document {
        version: config.version,
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs,
        nests: BTreeMap::new(),
        root,
    })
}

fn yaml_mapping_to_root(
    map: &Mapping,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    cache: &mut AnchorCache,
    cycle_detector: &mut CycleDetector,
    depth: usize,
) -> Result<BTreeMap<String, Item>, String> {
    // Check nesting depth
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
            path: "root".to_string(),
            context: None,
        }
        .to_string());
    }

    let mut root = BTreeMap::new();

    for (key, value) in map {
        // Skip ONLY known HEDL metadata keys (not all keys starting with __)
        let key_str = key.as_str().ok_or("Non-string keys not supported")?;
        if key_str == "__type__" || key_str == "__schema__" {
            continue;
        }

        let item_or_ref = yaml_value_to_item_cached(
            value,
            key_str,
            config,
            structs,
            cache,
            cycle_detector,
            depth,
        )?;
        root.insert(key_str.to_string(), item_or_ref.into_item());
    }

    Ok(root)
}

pub(crate) fn yaml_mapping_to_item_map(
    map: &Mapping,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    cache: &mut AnchorCache,
    cycle_detector: &mut CycleDetector,
    depth: usize,
) -> Result<BTreeMap<String, Item>, String> {
    // Check nesting depth
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
            path: "mapping".to_string(),
            context: None,
        }
        .to_string());
    }

    let mut result = BTreeMap::new();

    for (key, value) in map {
        // Skip ONLY known HEDL metadata keys (not all keys starting with __)
        let key_str = key.as_str().ok_or("Non-string keys not supported")?;
        if key_str == "__type__" || key_str == "__schema__" {
            continue;
        }

        let item_or_ref = yaml_value_to_item_cached(
            value,
            key_str,
            config,
            structs,
            cache,
            cycle_detector,
            depth,
        )?;
        result.insert(key_str.to_string(), item_or_ref.into_item());
    }

    Ok(result)
}

/// Wrapper around `yaml_value_to_item` for internal use by `yaml_mapping_to_item_map`.
///
/// The `_cache` and `_cycle_detector` parameters are unused because serde_yaml
/// automatically resolves YAML anchors and aliases before the values reach this
/// code. These parameters are retained for API consistency with the calling code
/// that tracks them for other purposes (e.g., future manual parsing backends).
fn yaml_value_to_item_cached(
    value: &YamlValue,
    key: &str,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    _cache: &mut AnchorCache,
    _cycle_detector: &mut CycleDetector,
    depth: usize,
) -> Result<ItemOrRef, String> {
    let item = yaml_value_to_item(value, key, config, structs, depth)?;
    Ok(ItemOrRef::Owned(item))
}

pub(crate) fn yaml_value_to_item(
    value: &YamlValue,
    key: &str,
    config: &FromYamlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<Item, String> {
    // Check nesting depth
    if depth > config.max_nesting_depth {
        return Err(YamlError::MaxDepthExceeded {
            max_depth: config.max_nesting_depth,
            actual_depth: depth,
            path: key.to_string(),
            context: None,
        }
        .to_string());
    }

    match value {
        YamlValue::Null => Ok(Item::Scalar(Value::Null)),
        YamlValue::Bool(b) => Ok(Item::Scalar(Value::Bool(*b))),
        YamlValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Item::Scalar(Value::Int(i)))
            } else if let Some(f) = n.as_f64() {
                Ok(Item::Scalar(Value::Float(f)))
            } else {
                Err(format!("Invalid number: {n:?}"))
            }
        }
        YamlValue::String(s) => {
            // Check for expression pattern $( ... )
            if s.starts_with("$(") && s.ends_with(')') {
                let expr =
                    parse_expression_token(s).map_err(|e| format!("Invalid expression: {e}"))?;
                return Ok(Item::Scalar(Value::Expression(Box::new(expr))));
            }
            // Note: We no longer auto-convert @... strings to references here.
            // References are now encoded as mappings with @ref key (like JSON).
            // This allows strings that happen to start with @ to round-trip correctly.
            Ok(Item::Scalar(Value::String(s.clone().into())))
        }
        YamlValue::Sequence(seq) => {
            // Check array length limit
            if seq.len() > config.max_array_length {
                return Err(YamlError::ArrayTooLong {
                    length: seq.len(),
                    max_length: config.max_array_length,
                    path: key.to_string(),
                    context: None,
                }
                .to_string());
            }

            // Handle empty sequences as empty matrix lists
            if seq.is_empty() {
                let type_name = singularize_and_capitalize(key);
                let list = MatrixList::new(type_name.clone(), vec!["id".to_string()]);
                structs.insert(type_name, vec!["id".to_string()]);
                Ok(Item::List(list))
            } else if is_scalar_list_sequence(seq) {
                // Non-numeric scalar sequence (strings, bools, refs, etc.) -> Value::List
                let items: Result<Vec<Value>, String> = seq
                    .iter()
                    .map(|v| yaml_to_value(v, config, depth + 1))
                    .collect();
                Ok(Item::Scalar(Value::List(Box::new(items?))))
            } else if is_tensor_sequence(seq) {
                // Check if it's a tensor (array of numbers)
                let tensor = yaml_sequence_to_tensor(seq, config, key, depth)?;
                Ok(Item::Scalar(Value::Tensor(Box::new(tensor))))
            } else if is_object_sequence(seq) {
                // Convert to matrix list
                let list = yaml_sequence_to_matrix_list(seq, key, config, structs, depth)?;
                Ok(Item::List(list))
            } else {
                // All numeric sequence -> tensor
                let tensor = yaml_sequence_to_tensor(seq, config, key, depth)?;
                Ok(Item::Scalar(Value::Tensor(Box::new(tensor))))
            }
        }
        YamlValue::Mapping(map) => {
            // Check for reference marker (@ref key)
            if let Some(YamlValue::String(ref_str)) = map.get(YamlValue::String("@ref".to_string()))
            {
                return Ok(Item::Scalar(Value::Reference(parse_reference(ref_str)?)));
            }
            // Check for special metadata indicating a matrix list
            // A mapping is a list wrapper ONLY if it has 'items' AND at least one metadata key
            // (__type__ or __schema__). This prevents treating legitimate 'items' fields as lists.
            let has_items = map.contains_key(YamlValue::String("items".to_string()));
            let has_type_metadata = map.contains_key(YamlValue::String("__type__".to_string()));
            let has_schema_metadata = map.contains_key(YamlValue::String("__schema__".to_string()));

            if has_items && (has_type_metadata || has_schema_metadata) {
                // Structured matrix list with metadata (__type__, __schema__, items)
                let items = map
                    .get(YamlValue::String("items".to_string()))
                    .ok_or("Missing items array")?;
                if let YamlValue::Sequence(seq) = items {
                    // Check array length
                    if seq.len() > config.max_array_length {
                        return Err(YamlError::ArrayTooLong {
                            length: seq.len(),
                            max_length: config.max_array_length,
                            path: format!("{key}.items"),
                            context: None,
                        }
                        .to_string());
                    }

                    // Extract schema from wrapper metadata if present
                    // This preserves field ordering when YAML was exported with __schema__ metadata
                    let wrapper_schema = if let Some(YamlValue::Sequence(schema_seq)) =
                        map.get(YamlValue::String("__schema__".to_string()))
                    {
                        let schema: Result<Vec<String>, String> = schema_seq
                            .iter()
                            .map(|v| {
                                v.as_str()
                                    .map(std::string::ToString::to_string)
                                    .ok_or_else(|| "Schema must contain strings".to_string())
                            })
                            .collect();
                        Some(schema?)
                    } else {
                        None
                    };

                    let mut list = if let Some(schema) = wrapper_schema {
                        // Use explicit schema from metadata
                        yaml_sequence_to_matrix_list_with_schema(
                            seq, key, schema, config, structs, depth,
                        )?
                    } else {
                        // Infer schema from data
                        yaml_sequence_to_matrix_list(seq, key, config, structs, depth)?
                    };

                    // Override type_name with wrapper metadata if present
                    if let Some(YamlValue::String(wrapper_type)) =
                        map.get(YamlValue::String("__type__".to_string()))
                    {
                        list.type_name = wrapper_type.clone();
                        // Re-register with correct type name and schema
                        structs.insert(wrapper_type.clone(), list.schema.clone());
                    }

                    return Ok(Item::List(list));
                }
            }
            // Regular object
            let mut cache = AnchorCache::new();
            let mut cycle_detector = CycleDetector::new();
            let item_map = yaml_mapping_to_item_map(
                map,
                config,
                structs,
                &mut cache,
                &mut cycle_detector,
                depth + 1,
            )?;
            Ok(Item::Object(item_map))
        }
        YamlValue::Tagged(tagged) => {
            // Handle YAML tags (anchors/aliases)
            yaml_value_to_item(&tagged.value, key, config, structs, depth)
        }
    }
}
