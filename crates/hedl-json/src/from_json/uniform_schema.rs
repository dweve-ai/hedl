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

//! Uniform schema detection
//!
//! These functions detect when an object's children all have similar schemas,
//! enabling conversion to a more compact MatrixList representation.
//!
//! For example, JSON Schema's "properties" object:
//! ```json
//! "properties": {
//!   "id": {"type": "string"},
//!   "name": {"type": "string"},
//!   "age": {"type": "number"}
//! }
//! ```
//! Can be converted to:
//! ```hedl
//! properties:@Property[type]
//!   |id|string
//!   |name|string
//!   |age|number
//! ```

use super::config::{FromJsonConfig, JsonConversionError, SchemaCache};
use super::value_conversion::json_scalar_to_value;
use hedl_core::lex::singularize_and_capitalize;
use hedl_core::{MatrixList, Node, Value};
use serde_json::{Map, Value as JsonValue};
use smallvec::SmallVec;
use std::collections::BTreeMap;

/// Minimum number of children required for uniform schema detection.
/// Objects with fewer children are kept as nested objects for readability.
const MIN_CHILDREN_FOR_UNIFORM_SCHEMA: usize = 2;

/// Maximum schema complexity (number of keys) for uniform schema conversion.
/// Objects with more complex children are kept nested to preserve structure.
const MAX_SCHEMA_KEYS_FOR_CONVERSION: usize = 8;

/// Get the schema signature of a JSON value (the sorted set of its keys if it's an object).
/// Returns None for non-objects.
fn get_object_schema_signature(value: &JsonValue) -> Option<Vec<&str>> {
    match value {
        JsonValue::Object(obj) => {
            let mut keys: Vec<&str> = obj
                .keys()
                .filter(|k| !k.starts_with("__"))
                .map(|s| s.as_str())
                .collect();
            keys.sort();
            Some(keys)
        }
        _ => None,
    }
}

/// Check if all children of an object have uniform schemas (same keys).
/// Returns the common schema if uniform, None otherwise.
fn detect_uniform_child_schema(map: &Map<String, JsonValue>) -> Option<Vec<&str>> {
    // Filter out metadata keys
    let children: Vec<_> = map.iter().filter(|(k, _)| !k.starts_with("__")).collect();

    // Need at least MIN_CHILDREN_FOR_UNIFORM_SCHEMA children
    if children.len() < MIN_CHILDREN_FOR_UNIFORM_SCHEMA {
        return None;
    }

    // All children must be objects
    let first_signature = get_object_schema_signature(children[0].1)?;

    // Schema must not be too complex
    if first_signature.len() > MAX_SCHEMA_KEYS_FOR_CONVERSION {
        return None;
    }

    // Schema must not be empty
    if first_signature.is_empty() {
        return None;
    }

    // Check all children have the same signature
    for (_, value) in &children[1..] {
        let sig = get_object_schema_signature(value)?;
        if sig != first_signature {
            return None;
        }
    }

    Some(first_signature)
}

/// Check if a value contains only scalar/simple values (no nested objects or arrays of objects).
/// This helps determine if an object's children are "flat" enough for MatrixList conversion.
pub fn is_flat_object(obj: &Map<String, JsonValue>) -> bool {
    for (key, value) in obj {
        if key.starts_with("__") {
            continue;
        }
        match value {
            JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_) => {
                // Scalars are fine
            }
            JsonValue::Array(arr) => {
                // Arrays of scalars are fine, arrays of objects are not
                if arr.iter().any(|v| matches!(v, JsonValue::Object(_))) {
                    return false;
                }
            }
            JsonValue::Object(nested) => {
                // Nested objects make this not flat
                // But allow empty objects or objects with only scalar values
                if !nested.is_empty()
                    && nested
                        .values()
                        .any(|v| matches!(v, JsonValue::Object(_) | JsonValue::Array(_)))
                {
                    return false;
                }
            }
        }
    }
    true
}

/// Try to convert an object with uniform-schema children to a MatrixList.
/// Returns Some(MatrixList) if conversion is possible, None otherwise.
pub fn try_convert_uniform_object_to_matrixlist(
    map: &Map<String, JsonValue>,
    parent_key: &str,
    config: &FromJsonConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    _schema_cache: &mut SchemaCache,
    depth: usize,
) -> Result<Option<MatrixList>, JsonConversionError> {
    // Check recursion depth
    if let Some(max_depth) = config.max_depth {
        if depth >= max_depth {
            return Ok(None);
        }
    }

    // Detect uniform schema
    let schema_keys = match detect_uniform_child_schema(map) {
        Some(keys) => keys,
        None => return Ok(None),
    };

    // Verify all children are flat enough for conversion
    let children: Vec<_> = map.iter().filter(|(k, _)| !k.starts_with("__")).collect();

    for (_, value) in &children {
        if let JsonValue::Object(obj) = value {
            if !is_flat_object(obj) {
                return Ok(None);
            }
        }
    }

    // Build the MatrixList
    let type_name = singularize_and_capitalize(parent_key);
    let schema: Vec<String> = schema_keys.iter().map(|s| (*s).to_string()).collect();

    // Register the struct
    structs.insert(type_name.clone(), schema.clone());

    // Convert each child to a row
    let mut rows = Vec::with_capacity(children.len());

    for (key, value) in &children {
        if let JsonValue::Object(obj) = value {
            // The key becomes the row ID
            let id = (*key).clone();

            // Extract field values in schema order
            let mut fields: SmallVec<[Value; 4]> = SmallVec::with_capacity(schema.len());
            for col in &schema {
                let field_value = obj
                    .get(col)
                    .map(|v| json_scalar_to_value(v, config))
                    .transpose()?
                    .unwrap_or(Value::Null);
                fields.push(field_value);
            }

            let node = Node {
                type_name: type_name.clone(),
                id,
                fields,
                children: None,
                child_count: 0,
            };
            rows.push(node);
        }
    }

    Ok(Some(MatrixList {
        type_name,
        schema,
        rows,
        count_hint: Some(children.len()),
    }))
}
