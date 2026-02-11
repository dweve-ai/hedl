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

//! Conversion utilities for XML items to HEDL structures

use super::config::FromXmlConfig;
use hedl_core::lex::{singularize_and_capitalize, Tensor};
use hedl_core::{Item, MatrixList, Node, Value};
use std::collections::BTreeMap;

pub(crate) fn items_to_matrix_list(
    name: &str,
    items: Vec<Item>,
    config: &FromXmlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
) -> Result<MatrixList, String> {
    items_to_matrix_list_with_type(name, items, None, config, structs)
}

pub(crate) fn items_to_matrix_list_with_type(
    name: &str,
    items: Vec<Item>,
    explicit_type: Option<String>,
    _config: &FromXmlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
) -> Result<MatrixList, String> {
    // Use explicit type if provided, otherwise infer from element name
    let type_name = explicit_type.unwrap_or_else(|| singularize_and_capitalize(name));

    // Infer schema from first item
    let schema = infer_schema(&items)?;

    // Register the struct definition
    structs.insert(type_name.clone(), schema.clone());

    let mut rows = Vec::new();
    for (idx, item) in items.into_iter().enumerate() {
        let node = item_to_node(&type_name, &schema, item, idx)?;
        rows.push(node);
    }

    Ok(MatrixList {
        type_name,
        schema,
        rows,
        count_hint: None,
    })
}

pub(crate) fn infer_schema(items: &[Item]) -> Result<Vec<String>, String> {
    if let Some(Item::Object(first_obj)) = items.first() {
        // Only include scalar fields in the schema, not nested lists or child objects
        let mut keys: Vec<_> = first_obj
            .iter()
            .filter(|(_, item)| matches!(item, Item::Scalar(_)))
            .map(|(k, _)| k.clone())
            .collect();
        keys.sort();

        // Ensure "id" is first if present
        if let Some(pos) = keys.iter().position(|k| k == "id") {
            keys.remove(pos);
            keys.insert(0, "id".to_string());
        } else {
            // Add implicit id column
            keys.insert(0, "id".to_string());
        }

        Ok(keys)
    } else {
        // Default schema
        Ok(vec!["id".to_string(), "value".to_string()])
    }
}

pub(crate) fn item_to_node(
    type_name: &str,
    schema: &[String],
    item: Item,
    idx: usize,
) -> Result<Node, String> {
    match item {
        Item::Object(obj) => {
            // Extract ID from object or generate one
            let id = obj
                .get(&schema[0])
                .and_then(|i| i.as_scalar())
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}", idx));

            // Extract ALL field values (including ID) per SPEC
            let mut fields = Vec::new();
            for col in schema {
                let value = obj
                    .get(col)
                    .and_then(|i| i.as_scalar())
                    .cloned()
                    .unwrap_or(Value::Null);
                fields.push(value);
            }

            // Extract nested children (Item::List entries become child nodes)
            let mut children: BTreeMap<String, Vec<Node>> = BTreeMap::new();
            for child_item in obj.values() {
                if let Item::List(child_list) = child_item {
                    // Convert child list rows to nodes
                    children.insert(child_list.type_name.clone(), child_list.rows.clone());
                }
            }

            Ok(Node {
                type_name: type_name.to_string(),
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
        Item::Scalar(value) => {
            // Single scalar - create node with ID value and scalar value
            let id = format!("{}", idx);
            Ok(Node {
                type_name: type_name.to_string(),
                id: id.clone(),
                fields: vec![Value::String(id.into()), value].into(),
                children: None,
                child_count: 0,
            })
        }
        Item::List(_) => Err("Cannot convert nested list to node".to_string()),
    }
}

/// Convert any string to a valid HEDL key (lowercase snake_case).
/// "Category" -> "category", "UserPost" -> "user_post", "XMLData" -> "xmldata"
/// ISSUE 3 FIX: Also sanitizes namespaces and invalid characters
/// "x:tag" -> "x_tag", "my-key" -> "my_key", "key.name" -> "key_name"
pub(crate) fn to_hedl_key(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            // Add underscore before uppercase letter (except at start or after another uppercase)
            if i > 0 && !prev_was_upper && !result.ends_with('_') {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            prev_was_upper = true;
        } else if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
            // Valid HEDL key characters
            result.push(c);
            prev_was_upper = false;
        } else {
            // Invalid characters (namespace colons, hyphens, dots, etc.) -> underscore
            if !result.is_empty() && !result.ends_with('_') {
                result.push('_');
            }
            prev_was_upper = false;
        }
    }

    // Clean up any double underscores
    while result.contains("__") {
        result = result.replace("__", "_");
    }

    // Remove leading/trailing underscores
    let result = result.trim_matches('_').to_string();

    // Ensure result is not empty and starts with valid character
    if result.is_empty() {
        return "key".to_string();
    }

    // If first character is a digit, prepend underscore
    if result.as_bytes()[0].is_ascii_digit() {
        format!("_{}", result)
    } else {
        result
    }
}

/// Check if all items are suitable for tensor representation.
/// Items must be numeric scalars or objects containing only a tensor at the "item" key.
pub(crate) fn items_are_tensor_elements(items: &[Item]) -> bool {
    items.iter().all(|item| {
        match item {
            // Direct numeric scalars
            Item::Scalar(Value::Int(_)) => true,
            Item::Scalar(Value::Float(_)) => true,
            // Already-parsed tensors
            Item::Scalar(Value::Tensor(_)) => true,
            // Objects with single "item" key containing a tensor (nested arrays)
            Item::Object(obj) if obj.len() == 1 => {
                matches!(obj.get("item"), Some(Item::Scalar(Value::Tensor(_))))
            }
            _ => false,
        }
    })
}

/// Check if all items are suitable for list representation.
/// Items must be scalars (any type including strings, bools, refs, nulls, etc).
pub(crate) fn items_are_list_elements(items: &[Item]) -> bool {
    items.iter().all(|item| item.as_scalar().is_some())
}

/// Convert items to a tensor.
pub(crate) fn items_to_tensor(items: &[Item]) -> Result<Tensor, String> {
    let mut tensor_items = Vec::new();

    for item in items {
        let tensor = match item {
            Item::Scalar(Value::Int(n)) => Tensor::Scalar(*n as f64),
            Item::Scalar(Value::Float(f)) => Tensor::Scalar(*f),
            Item::Scalar(Value::Tensor(t)) => (**t).clone(),
            Item::Object(obj) if obj.len() == 1 => {
                // Nested tensor element (object with only "item" key containing tensor)
                if let Some(Item::Scalar(Value::Tensor(t))) = obj.get("item") {
                    (**t).clone()
                } else {
                    return Err("Cannot convert non-numeric item to tensor".to_string());
                }
            }
            _ => return Err("Cannot convert non-numeric item to tensor".to_string()),
        };
        tensor_items.push(tensor);
    }

    Ok(Tensor::Array(tensor_items))
}

/// Convert items to a list.
/// All items must be scalars (already checked by items_are_list_elements).
pub(crate) fn items_to_list(items: &[Item]) -> Result<Vec<Value>, String> {
    items
        .iter()
        .map(|item| {
            item.as_scalar()
                .cloned()
                .ok_or_else(|| "Cannot convert non-scalar item to list".to_string())
        })
        .collect()
}
