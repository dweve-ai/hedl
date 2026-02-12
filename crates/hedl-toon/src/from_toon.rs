// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! TOON to HEDL conversion using the official toon-format crate

use crate::error::{Result, ToonError, MAX_NESTING_DEPTH};
use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use smallvec::smallvec;
use std::collections::BTreeMap;
use toon_format::DecodeOptions;

/// Configuration for TOON parsing
#[derive(Debug, Clone, Default)]
pub struct FromToonConfig {
    /// Enable strict validation
    pub strict: bool,
}

/// Parse TOON string to HEDL Document
pub fn from_toon(input: &str) -> Result<Document> {
    from_toon_with_config(input, &FromToonConfig::default())
}

/// Parse TOON string to HEDL Document with configuration
pub fn from_toon_with_config(input: &str, config: &FromToonConfig) -> Result<Document> {
    // Use official TOON decoder
    let options = DecodeOptions {
        strict: config.strict,
        ..Default::default()
    };

    let json_value: serde_json::Value =
        toon_format::decode(input, &options).map_err(ToonError::from)?;

    // Convert JSON to HEDL Document
    json_to_document(&json_value, 0)
}

/// Convert JSON value to HEDL Document
fn json_to_document(value: &serde_json::Value, depth: usize) -> Result<Document> {
    if depth > MAX_NESTING_DEPTH {
        return Err(ToonError::MaxDepthExceeded {
            depth,
            max: MAX_NESTING_DEPTH,
        });
    }

    let mut doc = Document::new((2, 0));

    if let serde_json::Value::Object(map) = value {
        for (key, val) in map {
            let item = json_to_item_with_doc(val, Some(key), &mut doc, depth + 1)?;
            doc.root.insert(key.clone(), item);
        }
    }

    Ok(doc)
}

/// Convert JSON value to HEDL Item with document context for schema registration
fn json_to_item_with_doc(
    value: &serde_json::Value,
    key: Option<&str>,
    doc: &mut Document,
    depth: usize,
) -> Result<Item> {
    if depth > MAX_NESTING_DEPTH {
        return Err(ToonError::MaxDepthExceeded {
            depth,
            max: MAX_NESTING_DEPTH,
        });
    }

    match value {
        serde_json::Value::Null => Ok(Item::Scalar(Value::Null)),
        serde_json::Value::Bool(b) => Ok(Item::Scalar(Value::Bool(*b))),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Item::Scalar(Value::Int(i)))
            } else if let Some(f) = n.as_f64() {
                Ok(Item::Scalar(Value::Float(f)))
            } else {
                Ok(Item::Scalar(Value::String(n.to_string().into())))
            }
        }
        serde_json::Value::String(s) => {
            // Check for reference syntax
            if s.starts_with('@') {
                Ok(Item::Scalar(Value::Reference(parse_reference(s))))
            } else {
                Ok(Item::Scalar(Value::String(s.clone().into())))
            }
        }
        serde_json::Value::Array(arr) => {
            // Infer type name from key if available
            let type_name = key
                .map(infer_type_name)
                .unwrap_or_else(|| "Item".to_string());
            json_array_to_item_with_doc(arr, &type_name, doc, depth + 1)
        }
        serde_json::Value::Object(map) => {
            let mut children = BTreeMap::new();
            for (k, val) in map {
                children.insert(
                    k.clone(),
                    json_to_item_with_doc(val, Some(k), doc, depth + 1)?,
                );
            }
            Ok(Item::Object(children))
        }
    }
}

/// Convert JSON array to HEDL Item with document context for schema registration
fn json_array_to_item_with_doc(
    arr: &[serde_json::Value],
    type_name: &str,
    doc: &mut Document,
    depth: usize,
) -> Result<Item> {
    if depth > MAX_NESTING_DEPTH {
        return Err(ToonError::MaxDepthExceeded {
            depth,
            max: MAX_NESTING_DEPTH,
        });
    }

    if arr.is_empty() {
        return Ok(Item::List(MatrixList::new(type_name, vec![])));
    }

    // Check if all elements are objects (potential matrix)
    if let Some(serde_json::Value::Object(first)) = arr.first() {
        // Check all elements are objects
        let all_objects = arr.iter().all(|v| v.is_object());

        if all_objects {
            // Collect all primitive keys across all objects (arrays are children, not schema fields)
            let mut all_primitive_keys: std::collections::HashSet<String> =
                std::collections::HashSet::new();
            let mut all_child_keys: std::collections::HashSet<String> =
                std::collections::HashSet::new();

            for item in arr.iter() {
                if let serde_json::Value::Object(m) = item {
                    for (k, v) in m.iter() {
                        if v.is_array() {
                            all_child_keys.insert(k.clone());
                        } else {
                            all_primitive_keys.insert(k.clone());
                        }
                    }
                }
            }

            // Check if all elements have the same PRIMITIVE keys (children are optional)
            let all_same_primitives = arr.iter().all(|v| {
                if let serde_json::Value::Object(m) = v {
                    let item_primitive_keys: std::collections::HashSet<String> = m
                        .iter()
                        .filter(|(_, v)| !v.is_array())
                        .map(|(k, _)| k.clone())
                        .collect();
                    item_primitive_keys == all_primitive_keys
                } else {
                    false
                }
            });

            if all_same_primitives {
                // Preserve original key order from the first object for primitives
                let primitive_keys: Vec<String> = first
                    .keys()
                    .filter(|k| all_primitive_keys.contains(*k))
                    .cloned()
                    .collect();
                let child_keys: Vec<String> = all_child_keys.into_iter().collect();

                // Schema only includes primitive fields
                let schema: Vec<String> = primitive_keys;
                let mut list = MatrixList::new(type_name, schema.clone());

                // Register schema in document
                if !schema.is_empty() {
                    doc.structs.insert(type_name.to_string(), schema.clone());
                }

                for (idx, item) in arr.iter().enumerate() {
                    if let serde_json::Value::Object(m) = item {
                        let mut fields = smallvec![];

                        // Add primitive fields in schema order
                        for key in &schema {
                            if let Some(val) = m.get(key) {
                                let hedl_val = json_to_value(val)?;
                                fields.push(hedl_val);
                            }
                        }

                        // Get ID from first field or generate one
                        let id = if let Some(Value::String(s)) = fields.first() {
                            s.to_string()
                        } else if let Some(Value::Int(i)) = fields.first() {
                            i.to_string()
                        } else {
                            format!("item_{}", idx)
                        };

                        // Convert array fields to children
                        let children: Option<Box<BTreeMap<String, Vec<Node>>>> = if child_keys
                            .is_empty()
                        {
                            None
                        } else {
                            let mut children_map: BTreeMap<String, Vec<Node>> = BTreeMap::new();
                            for child_key in &child_keys {
                                if let Some(serde_json::Value::Array(child_arr)) = m.get(child_key)
                                {
                                    // Infer child type name from key
                                    let child_type = infer_type_name(child_key);
                                    // Recursively convert with doc for schema registration
                                    if let Item::List(child_list) = json_array_to_item_with_doc(
                                        child_arr,
                                        &child_type,
                                        doc,
                                        depth + 1,
                                    )? {
                                        children_map.insert(child_type, child_list.rows);
                                    }
                                }
                            }
                            if children_map.is_empty() {
                                None
                            } else {
                                Some(Box::new(children_map))
                            }
                        };

                        let child_count: u16 = children
                            .as_ref()
                            .map(|c| c.values().map(|v| v.len()).sum::<usize>())
                            .unwrap_or(0)
                            .try_into()
                            .unwrap_or(u16::MAX);

                        let node = Node {
                            type_name: type_name.to_string(),
                            id,
                            fields,
                            children,
                            child_count,
                        };
                        list.rows.push(node);
                    }
                }

                return Ok(Item::List(list));
            }
        }
    }

    // Check if all primitives (simple list)
    let all_primitives = arr.iter().all(|v| !v.is_object() && !v.is_array());

    if all_primitives {
        let values: Vec<Value> = arr.iter().map(json_to_value).collect::<Result<_>>()?;
        return Ok(Item::Scalar(Value::List(Box::new(values))));
    }

    // Mixed array - convert to list
    let mut list = MatrixList::new(type_name, vec![]);
    for (idx, item) in arr.iter().enumerate() {
        let hedl_val = json_to_value(item)?;
        let node = Node {
            type_name: type_name.to_string(),
            id: format!("item_{}", idx),
            fields: smallvec![hedl_val],
            children: None,
            child_count: 0,
        };
        list.rows.push(node);
    }

    Ok(Item::List(list))
}

/// Infer a type name from a key (handles pluralization)
fn infer_type_name(key: &str) -> String {
    // If key is PascalCase, use as-is (likely already a type name)
    if key
        .chars()
        .next()
        .map(|c| c.is_ascii_uppercase())
        .unwrap_or(false)
    {
        return key.to_string();
    }

    // Try to singularize and capitalize
    let singular = singularize(key);
    capitalize_first(&singular)
}

/// Simple singularization (handles common cases)
fn singularize(word: &str) -> String {
    if word.ends_with("ies") && word.len() > 3 {
        format!("{}y", &word[..word.len() - 3])
    } else if word.ends_with("es") && word.len() > 2 {
        // Handle cases like "classes" -> "class", "boxes" -> "box"
        let stem = &word[..word.len() - 2];
        if stem.ends_with("ss")
            || stem.ends_with('x')
            || stem.ends_with("ch")
            || stem.ends_with("sh")
        {
            stem.to_string()
        } else {
            // "es" might just be "s" plural: "types" doesn't end in "es"
            word[..word.len() - 1].to_string()
        }
    } else if word.ends_with('s') && word.len() > 1 {
        word[..word.len() - 1].to_string()
    } else {
        word.to_string()
    }
}

/// Capitalize the first character
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

/// Convert JSON value to HEDL Value
fn json_to_value(value: &serde_json::Value) -> Result<Value> {
    Ok(match value {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::String(n.to_string().into())
            }
        }
        serde_json::Value::String(s) => {
            if s.starts_with('@') {
                Value::Reference(parse_reference(s))
            } else {
                Value::String(s.clone().into())
            }
        }
        serde_json::Value::Array(arr) => {
            let values: Vec<Value> = arr.iter().map(json_to_value).collect::<Result<_>>()?;
            Value::List(Box::new(values))
        }
        serde_json::Value::Object(_) => {
            // Objects in value context become strings (JSON representation)
            Value::String(value.to_string().into())
        }
    })
}

/// Parse reference string to Reference struct
///
/// Reference formats:
/// - `@Type:id` -> qualified reference (type_name: Some("Type"), id: "id")
/// - `@:id` -> unqualified reference (type_name: None, id: "id")
/// - `@id` -> local reference (type_name: None, id: "id")
fn parse_reference(s: &str) -> Reference {
    let s = s.trim_start_matches('@');

    if let Some(colon_pos) = s.find(':') {
        let type_name = &s[..colon_pos];
        let id = &s[colon_pos + 1..];

        Reference {
            type_name: if type_name.is_empty() {
                None
            } else {
                Some(type_name.into())
            },
            id: id.into(),
        }
    } else {
        // No colon: local reference (@id) means type_name is None, id is the value
        Reference {
            type_name: None,
            id: s.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_from_toon() {
        let toon = r#"name: MyApp
version: 1
"#;
        let doc = from_toon(toon).unwrap();
        assert!(doc.root.contains_key("name"));
        assert!(doc.root.contains_key("version"));
    }

    #[test]
    fn test_nested_from_toon() {
        let toon = r#"config:
  name: MyApp
  settings:
    debug: true
"#;
        let doc = from_toon(toon).unwrap();
        assert!(doc.root.contains_key("config"));
    }
}
