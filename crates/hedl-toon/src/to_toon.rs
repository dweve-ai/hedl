// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! HEDL to TOON conversion with correct indentation handling.
//!
//! This module uses a custom encoder that fixes indentation bugs present in
//! toon-format 0.4.x for nested tabular arrays in list items.

use crate::encoder;
use crate::error::{Result, ToonError, MAX_NESTING_DEPTH};
use hedl_core::{Document, Item, MatrixList, Node, Value};
use serde_json::json;

/// TOON delimiter options
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Delimiter {
    /// Comma delimiter (default)
    #[default]
    Comma,
    /// Tab delimiter
    Tab,
    /// Pipe delimiter
    Pipe,
}

/// Configuration for HEDL to TOON conversion
#[derive(Debug, Clone)]
pub struct ToToonConfig {
    /// Indentation width (default: 2)
    pub indent: usize,
    /// Delimiter for tabular arrays
    pub delimiter: Delimiter,
}

impl Default for ToToonConfig {
    fn default() -> Self {
        Self {
            indent: 2, // TOON format spec uses 2-space indentation (independent of HEDL indent)
            delimiter: Delimiter::Comma,
        }
    }
}

impl ToToonConfig {
    /// Create a new builder for configuration
    pub fn builder() -> ToToonConfigBuilder {
        ToToonConfigBuilder::default()
    }
}

/// Builder for ToToonConfig
#[derive(Debug, Default)]
pub struct ToToonConfigBuilder {
    indent: Option<usize>,
    delimiter: Option<Delimiter>,
}

impl ToToonConfigBuilder {
    /// Set indentation width
    pub fn indent(mut self, indent: usize) -> Self {
        self.indent = Some(indent);
        self
    }

    /// Set delimiter
    pub fn delimiter(mut self, delimiter: Delimiter) -> Self {
        self.delimiter = Some(delimiter);
        self
    }

    /// Build the configuration
    pub fn build(self) -> ToToonConfig {
        ToToonConfig {
            indent: self.indent.unwrap_or(2), // TOON format spec uses 2-space indentation
            delimiter: self.delimiter.unwrap_or_default(),
        }
    }
}

/// Convert HEDL document to TOON string
pub fn to_toon(doc: &Document, config: &ToToonConfig) -> Result<String> {
    // Convert HEDL to JSON value first
    let json_value = document_to_json(doc, doc, 0)?;

    // Get delimiter character
    let delimiter_char = match config.delimiter {
        Delimiter::Comma => ',',
        Delimiter::Tab => '\t',
        Delimiter::Pipe => '|',
    };

    // Use our custom encoder with correct indentation handling
    encoder::encode_to_toon(&json_value, config.indent, delimiter_char)
}

/// Convert HEDL Document to serde_json::Value
fn document_to_json(
    doc: &Document,
    full_doc: &Document,
    depth: usize,
) -> Result<serde_json::Value> {
    if depth > MAX_NESTING_DEPTH {
        return Err(ToonError::MaxDepthExceeded {
            depth,
            max: MAX_NESTING_DEPTH,
        });
    }

    let mut map = serde_json::Map::new();

    for (key, item) in &doc.root {
        let value = item_to_json(item, full_doc, depth + 1)?;
        map.insert(key.clone(), value);
    }

    Ok(serde_json::Value::Object(map))
}

/// Convert HEDL Item to JSON value
fn item_to_json(item: &Item, full_doc: &Document, depth: usize) -> Result<serde_json::Value> {
    if depth > MAX_NESTING_DEPTH {
        return Err(ToonError::MaxDepthExceeded {
            depth,
            max: MAX_NESTING_DEPTH,
        });
    }

    match item {
        Item::Scalar(value) => value_to_json(value),
        Item::Object(children) => {
            let mut map = serde_json::Map::new();
            for (key, child) in children {
                map.insert(key.clone(), item_to_json(child, full_doc, depth + 1)?);
            }
            Ok(serde_json::Value::Object(map))
        }
        Item::List(matrix) => matrix_to_json(matrix, full_doc, depth + 1),
    }
}

/// Convert HEDL MatrixList to JSON array
fn matrix_to_json(
    matrix: &MatrixList,
    full_doc: &Document,
    depth: usize,
) -> Result<serde_json::Value> {
    if depth > MAX_NESTING_DEPTH {
        return Err(ToonError::MaxDepthExceeded {
            depth,
            max: MAX_NESTING_DEPTH,
        });
    }

    let mut items = Vec::new();

    for node in &matrix.rows {
        let item = node_to_json(node, &matrix.schema, full_doc, depth + 1)?;
        items.push(item);
    }

    Ok(serde_json::Value::Array(items))
}

/// Convert HEDL Node to JSON object
fn node_to_json(
    node: &Node,
    schema: &[String],
    full_doc: &Document,
    depth: usize,
) -> Result<serde_json::Value> {
    if depth > MAX_NESTING_DEPTH {
        return Err(ToonError::MaxDepthExceeded {
            depth,
            max: MAX_NESTING_DEPTH,
        });
    }

    let mut map = serde_json::Map::new();

    // Add scalar fields using schema column names
    for (i, value) in node.fields.iter().enumerate() {
        if let Some(key) = schema.get(i) {
            map.insert(key.clone(), value_to_json(value)?);
        }
    }

    // Add nested children
    if let Some(children_map) = &node.children {
        for (type_name, child_nodes) in children_map.iter() {
            // Look up the child type's schema from the Document
            let child_schema = full_doc
                .structs
                .get(type_name)
                .map(|s| s.as_slice())
                .unwrap_or(&[]);

            let arr: Vec<serde_json::Value> = child_nodes
                .iter()
                .map(|n| node_to_json(n, child_schema, full_doc, depth + 1))
                .collect::<Result<_>>()?;
            map.insert(type_name.clone(), serde_json::Value::Array(arr));
        }
    }

    Ok(serde_json::Value::Object(map))
}

/// Convert HEDL Value to JSON value
fn value_to_json(value: &Value) -> Result<serde_json::Value> {
    Ok(match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => json!(*b),
        Value::Int(i) => json!(*i),
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                serde_json::Value::Null
            } else {
                json!(*f)
            }
        }
        Value::String(s) => json!(s.as_ref()),
        Value::Reference(r) => {
            // Format reference as @Type:id or @:id
            let s = match &r.type_name {
                Some(t) => format!("@{}:{}", t, r.id),
                None => format!("@:{}", r.id),
            };
            json!(s)
        }
        Value::Tensor(tensor) => tensor_enum_to_json(tensor),
        Value::List(items) => {
            let arr: Vec<serde_json::Value> =
                items.iter().map(value_to_json).collect::<Result<_>>()?;
            serde_json::Value::Array(arr)
        }
        Value::Expression(expr) => {
            // Convert expression to string representation
            json!(format!("${{{}}}", expr))
        }
    })
}

/// Convert Tensor enum to JSON value
fn tensor_enum_to_json(tensor: &hedl_core::lex::Tensor) -> serde_json::Value {
    use hedl_core::lex::Tensor;
    match tensor {
        Tensor::Scalar(n) => json!(*n),
        Tensor::Array(items) => {
            let arr: Vec<serde_json::Value> = items.iter().map(tensor_enum_to_json).collect();
            serde_json::Value::Array(arr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_to_toon() {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
name: MyApp
version: 1.0
"#;
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let toon = to_toon(&doc, &ToToonConfig::default()).unwrap();

        assert!(toon.contains("name:"));
        assert!(toon.contains("MyApp"));
    }

    #[test]
    fn test_array_to_toon() {
        let hedl = r#"%V:2.0
%NULL:~
%QUOTE:"
---
items: [1, 2, 3]
"#;
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let toon = to_toon(&doc, &ToToonConfig::default()).unwrap();

        assert!(toon.contains("items"));
    }
}
