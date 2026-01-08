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

//! HEDL to JSON conversion

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_core::lex::Tensor;
use serde_json::{json, Map, Number, Value as JsonValue};
use std::collections::BTreeMap;

/// Configuration for JSON output
#[derive(Debug, Clone)]
pub struct ToJsonConfig {
    /// Include HEDL metadata (__type__, __schema__)
    pub include_metadata: bool,
    /// Flatten matrix lists to plain arrays
    pub flatten_lists: bool,
    /// Include children as nested arrays (default: true)
    pub include_children: bool,
}

impl Default for ToJsonConfig {
    fn default() -> Self {
        Self {
            include_metadata: false,
            flatten_lists: false,
            include_children: true, // Children should be included by default
        }
    }
}

impl hedl_core::convert::ExportConfig for ToJsonConfig {
    fn include_metadata(&self) -> bool {
        self.include_metadata
    }

    fn pretty(&self) -> bool {
        // JSON always uses pretty printing in to_json
        true
    }
}

/// Convert Document to JSON string
pub fn to_json(doc: &Document, config: &ToJsonConfig) -> Result<String, String> {
    let value = to_json_value(doc, config)?;
    serde_json::to_string_pretty(&value).map_err(|e| format!("JSON serialization error: {}", e))
}

/// Convert Document to serde_json::Value
pub fn to_json_value(doc: &Document, config: &ToJsonConfig) -> Result<JsonValue, String> {
    root_to_json(&doc.root, doc, config)
}

fn root_to_json(
    root: &BTreeMap<String, Item>,
    doc: &Document,
    config: &ToJsonConfig,
) -> Result<JsonValue, String> {
    // P1 OPTIMIZATION: Pre-allocate map capacity (1.05-1.1x speedup)
    let mut map = Map::with_capacity(root.len());

    for (key, item) in root {
        let json_value = item_to_json(item, doc, config)?;
        map.insert(key.clone(), json_value);
    }

    Ok(JsonValue::Object(map))
}

fn item_to_json(item: &Item, doc: &Document, config: &ToJsonConfig) -> Result<JsonValue, String> {
    match item {
        Item::Scalar(value) => Ok(value_to_json(value)),
        Item::Object(obj) => object_to_json(obj, doc, config),
        Item::List(list) => matrix_list_to_json(list, doc, config),
    }
}

fn object_to_json(
    obj: &BTreeMap<String, Item>,
    doc: &Document,
    config: &ToJsonConfig,
) -> Result<JsonValue, String> {
    // P1 OPTIMIZATION: Pre-allocate map capacity
    let mut map = Map::with_capacity(obj.len());

    for (key, item) in obj {
        let json_value = item_to_json(item, doc, config)?;
        map.insert(key.clone(), json_value);
    }

    Ok(JsonValue::Object(map))
}

fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Int(n) => JsonValue::Number(Number::from(*n)),
        Value::Float(f) => Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        Value::String(s) => JsonValue::String(s.clone()),
        Value::Tensor(t) => tensor_to_json(t),
        Value::Reference(r) => {
            // Represent references as objects with special key
            json!({ "@ref": r.to_ref_string() })
        }
        Value::Expression(e) => {
            // Represent expressions as strings with $() wrapper
            JsonValue::String(format!("$({})", e))
        }
    }
}

fn tensor_to_json(tensor: &Tensor) -> JsonValue {
    // Convert tensor to nested arrays recursively
    match tensor {
        Tensor::Scalar(n) => Number::from_f64(*n)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        Tensor::Array(items) => {
            // OPTIMIZATION: Pre-allocate array with exact capacity
            // Reduces reallocations during recursive tensor serialization
            let mut arr = Vec::with_capacity(items.len());
            for item in items {
                arr.push(tensor_to_json(item));
            }
            JsonValue::Array(arr)
        }
    }
}

fn matrix_list_to_json(
    list: &MatrixList,
    doc: &Document,
    config: &ToJsonConfig,
) -> Result<JsonValue, String> {
    // P1 OPTIMIZATION: Pre-allocate array capacity
    let mut array = Vec::with_capacity(list.rows.len());

    for row in &list.rows {
        // P1 OPTIMIZATION: Pre-allocate row object capacity
        let mut row_obj = Map::with_capacity(list.schema.len() + 2); // +2 for metadata fields

        // Add field values according to schema
        // Per SPEC.md: Node.fields contains ALL values including ID (first column)
        // MatrixList.schema includes all column names with ID first
        for (i, col_name) in list.schema.iter().enumerate() {
            if let Some(field_value) = row.fields.get(i) {
                row_obj.insert(col_name.clone(), value_to_json(field_value));
            }
        }

        // Add metadata if configured
        if config.include_metadata {
            row_obj.insert(
                "__type__".to_string(),
                JsonValue::String(list.type_name.clone()),
            );
        }

        // Add children if configured and present
        if config.include_children && !row.children.is_empty() {
            for (child_type, child_nodes) in &row.children {
                let child_json = nodes_to_json(child_type, child_nodes, doc, config)?;
                row_obj.insert(child_type.clone(), child_json);
            }
        }

        array.push(JsonValue::Object(row_obj));
    }

    // Wrap with metadata if configured
    if config.include_metadata && !config.flatten_lists {
        let mut metadata = json!({
            "__type__": list.type_name,
            "__schema__": list.schema,
            "items": array
        });

        // Include count_hint if present
        if let Some(count) = list.count_hint {
            if let Some(obj) = metadata.as_object_mut() {
                obj.insert("__count_hint__".to_string(), JsonValue::Number(count.into()));
            }
        }

        Ok(metadata)
    } else {
        Ok(JsonValue::Array(array))
    }
}

fn nodes_to_json(
    type_name: &str,
    nodes: &[Node],
    doc: &Document,
    config: &ToJsonConfig,
) -> Result<JsonValue, String> {
    // OPTIMIZATION: Pre-allocate array with exact capacity
    // Reduces reallocation during node processing
    let mut array = Vec::with_capacity(nodes.len());

    // Look up the schema for this type from the document
    let schema = doc.get_schema(type_name);

    for node in nodes {
        // OPTIMIZATION: Pre-allocate map capacity based on schema size + metadata + children
        let capacity = if let Some(field_names) = schema {
            field_names.len() + if config.include_metadata { 1 } else { 0 } + node.children.len()
        } else {
            node.fields.len() + if config.include_metadata { 1 } else { 0 } + node.children.len()
        };
        let mut obj = Map::with_capacity(capacity);

        // Add fields according to schema if available
        if let Some(field_names) = schema {
            for (i, col_name) in field_names.iter().enumerate() {
                if let Some(field_value) = node.fields.get(i) {
                    obj.insert(col_name.clone(), value_to_json(field_value));
                }
            }
        } else {
            // Fallback: use id + field_N naming when schema not available
            obj.insert("id".to_string(), JsonValue::String(node.id.clone()));
            for (i, value) in node.fields.iter().enumerate() {
                obj.insert(format!("field_{}", i), value_to_json(value));
            }
        }

        // Add metadata if configured
        if config.include_metadata {
            obj.insert(
                "__type__".to_string(),
                JsonValue::String(type_name.to_string()),
            );
        }

        // Add children if configured
        if config.include_children && !node.children.is_empty() {
            for (child_type, child_nodes) in &node.children {
                let child_json = nodes_to_json(child_type, child_nodes, doc, config)?;
                obj.insert(child_type.clone(), child_json);
            }
        }

        array.push(JsonValue::Object(obj));
    }

    Ok(JsonValue::Array(array))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Expression, Reference};

    // ==================== ToJsonConfig tests ====================

    #[test]
    fn test_to_json_config_default() {
        let config = ToJsonConfig::default();
        assert!(!config.include_metadata);
        assert!(!config.flatten_lists);
        assert!(config.include_children);
    }

    #[test]
    fn test_to_json_config_debug() {
        let config = ToJsonConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("ToJsonConfig"));
        assert!(debug.contains("include_metadata"));
        assert!(debug.contains("flatten_lists"));
        assert!(debug.contains("include_children"));
    }

    #[test]
    fn test_to_json_config_clone() {
        let config = ToJsonConfig {
            include_metadata: true,
            flatten_lists: true,
            include_children: false,
        };
        let cloned = config.clone();
        assert!(cloned.include_metadata);
        assert!(cloned.flatten_lists);
        assert!(!cloned.include_children);
    }

    // ==================== value_to_json tests ====================

    #[test]
    fn test_value_to_json() {
        assert_eq!(value_to_json(&Value::Null), JsonValue::Null);
        assert_eq!(value_to_json(&Value::Bool(true)), JsonValue::Bool(true));
        assert_eq!(value_to_json(&Value::Int(42)), json!(42));
        assert_eq!(
            value_to_json(&Value::String("hello".into())),
            json!("hello")
        );
    }

    #[test]
    fn test_value_to_json_null() {
        assert_eq!(value_to_json(&Value::Null), JsonValue::Null);
    }

    #[test]
    fn test_value_to_json_bool() {
        assert_eq!(value_to_json(&Value::Bool(true)), json!(true));
        assert_eq!(value_to_json(&Value::Bool(false)), json!(false));
    }

    #[test]
    fn test_value_to_json_int() {
        assert_eq!(value_to_json(&Value::Int(0)), json!(0));
        assert_eq!(value_to_json(&Value::Int(-42)), json!(-42));
        assert_eq!(value_to_json(&Value::Int(i64::MAX)), json!(i64::MAX));
    }

    #[test]
    fn test_value_to_json_float() {
        assert_eq!(value_to_json(&Value::Float(3.5)), json!(3.5));
        assert_eq!(value_to_json(&Value::Float(0.0)), json!(0.0));
        assert_eq!(value_to_json(&Value::Float(-1.5)), json!(-1.5));
    }

    #[test]
    fn test_value_to_json_float_nan() {
        // NaN cannot be represented in JSON, becomes null
        assert_eq!(value_to_json(&Value::Float(f64::NAN)), JsonValue::Null);
    }

    #[test]
    fn test_value_to_json_float_infinity() {
        // Infinity cannot be represented in JSON, becomes null
        assert_eq!(value_to_json(&Value::Float(f64::INFINITY)), JsonValue::Null);
        assert_eq!(
            value_to_json(&Value::Float(f64::NEG_INFINITY)),
            JsonValue::Null
        );
    }

    #[test]
    fn test_value_to_json_string() {
        assert_eq!(value_to_json(&Value::String("".into())), json!(""));
        assert_eq!(
            value_to_json(&Value::String("hello world".into())),
            json!("hello world")
        );
        assert_eq!(
            value_to_json(&Value::String("with\nnewline".into())),
            json!("with\nnewline")
        );
    }

    #[test]
    fn test_value_to_json_string_unicode() {
        assert_eq!(
            value_to_json(&Value::String("héllo 世界".into())),
            json!("héllo 世界")
        );
    }

    #[test]
    fn test_value_to_json_reference() {
        let reference = Reference::qualified("User", "123");
        let json = value_to_json(&Value::Reference(reference));
        assert_eq!(json, json!({"@ref": "@User:123"}));
    }

    #[test]
    fn test_value_to_json_reference_local() {
        let reference = Reference::local("123");
        let json = value_to_json(&Value::Reference(reference));
        assert_eq!(json, json!({"@ref": "@123"}));
    }

    #[test]
    fn test_value_to_json_expression() {
        use hedl_core::lex::Span;
        let expr = Expression::Identifier {
            name: "foo".to_string(),
            span: Span::default(),
        };
        let json = value_to_json(&Value::Expression(expr));
        assert_eq!(json, json!("$(foo)"));
    }

    // ==================== tensor_to_json tests ====================

    #[test]
    fn test_tensor_to_json_scalar() {
        assert_eq!(tensor_to_json(&Tensor::Scalar(1.0)), json!(1.0));
        assert_eq!(tensor_to_json(&Tensor::Scalar(3.5)), json!(3.5));
    }

    #[test]
    fn test_tensor_to_json_1d() {
        let tensor = Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
            Tensor::Scalar(3.0),
        ]);
        assert_eq!(tensor_to_json(&tensor), json!([1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_tensor_to_json_2d() {
        let tensor = Tensor::Array(vec![
            Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
            Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
        ]);
        assert_eq!(tensor_to_json(&tensor), json!([[1.0, 2.0], [3.0, 4.0]]));
    }

    #[test]
    fn test_tensor_to_json_empty() {
        let tensor = Tensor::Array(vec![]);
        assert_eq!(tensor_to_json(&tensor), json!([]));
    }

    #[test]
    fn test_tensor_to_json_nan_becomes_null() {
        let tensor = Tensor::Scalar(f64::NAN);
        assert_eq!(tensor_to_json(&tensor), JsonValue::Null);
    }

    // ==================== item_to_json tests ====================

    #[test]
    fn test_item_to_json_scalar() {
        let doc = Document::new((1, 0));
        let config = ToJsonConfig::default();
        let item = Item::Scalar(Value::Int(42));
        let result = item_to_json(&item, &doc, &config).unwrap();
        assert_eq!(result, json!(42));
    }

    #[test]
    fn test_item_to_json_object() {
        let doc = Document::new((1, 0));
        let config = ToJsonConfig::default();
        let mut obj = BTreeMap::new();
        obj.insert(
            "key".to_string(),
            Item::Scalar(Value::String("value".into())),
        );
        let item = Item::Object(obj);
        let result = item_to_json(&item, &doc, &config).unwrap();
        assert_eq!(result, json!({"key": "value"}));
    }

    // ==================== object_to_json tests ====================

    #[test]
    fn test_object_to_json_empty() {
        let doc = Document::new((1, 0));
        let config = ToJsonConfig::default();
        let obj = BTreeMap::new();
        let result = object_to_json(&obj, &doc, &config).unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_object_to_json_nested() {
        let doc = Document::new((1, 0));
        let config = ToJsonConfig::default();
        let mut inner = BTreeMap::new();
        inner.insert("nested".to_string(), Item::Scalar(Value::Bool(true)));
        let mut outer = BTreeMap::new();
        outer.insert("inner".to_string(), Item::Object(inner));
        let result = object_to_json(&outer, &doc, &config).unwrap();
        assert_eq!(result, json!({"inner": {"nested": true}}));
    }

    // ==================== root_to_json tests ====================

    #[test]
    fn test_root_to_json_empty() {
        let doc = Document::new((1, 0));
        let config = ToJsonConfig::default();
        let root = BTreeMap::new();
        let result = root_to_json(&root, &doc, &config).unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_root_to_json_with_items() {
        let doc = Document::new((1, 0));
        let config = ToJsonConfig::default();
        let mut root = BTreeMap::new();
        root.insert(
            "name".to_string(),
            Item::Scalar(Value::String("test".into())),
        );
        root.insert("count".to_string(), Item::Scalar(Value::Int(42)));
        let result = root_to_json(&root, &doc, &config).unwrap();
        assert_eq!(result, json!({"name": "test", "count": 42}));
    }

    // ==================== to_json tests ====================

    #[test]
    fn test_to_json_empty_document() {
        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root: BTreeMap::new(),
        };
        let config = ToJsonConfig::default();
        let result = to_json(&doc, &config).unwrap();
        assert_eq!(result.trim(), "{}");
    }

    #[test]
    fn test_to_json_with_scalars() {
        let mut root = BTreeMap::new();
        root.insert(
            "name".to_string(),
            Item::Scalar(Value::String("test".into())),
        );
        root.insert("active".to_string(), Item::Scalar(Value::Bool(true)));
        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };
        let config = ToJsonConfig::default();
        let result = to_json(&doc, &config).unwrap();
        let parsed: JsonValue = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["name"], json!("test"));
        assert_eq!(parsed["active"], json!(true));
    }

    // ==================== to_json_value tests ====================

    #[test]
    fn test_to_json_value_simple() {
        let mut root = BTreeMap::new();
        root.insert("key".to_string(), Item::Scalar(Value::Int(42)));
        let doc = Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            root,
        };
        let config = ToJsonConfig::default();
        let result = to_json_value(&doc, &config).unwrap();
        assert_eq!(result, json!({"key": 42}));
    }

    // ==================== matrix_list_to_json tests ====================

    #[test]
    fn test_matrix_list_to_json_simple() {
        let doc = Document::new((1, 0));
        let config = ToJsonConfig::default();
        let list = MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "1".to_string(),
                fields: vec![Value::String("1".into()), Value::String("Alice".into())],
                children: BTreeMap::new(),
                child_count: None,
            }],
            count_hint: None,
        };
        let result = matrix_list_to_json(&list, &doc, &config).unwrap();
        assert_eq!(result, json!([{"id": "1", "name": "Alice"}]));
    }

    #[test]
    fn test_matrix_list_to_json_with_metadata() {
        let doc = Document::new((1, 0));
        let config = ToJsonConfig {
            include_metadata: true,
            flatten_lists: false,
            include_children: true,
        };
        let list = MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "1".to_string(),
                fields: vec![Value::String("1".into())],
                children: BTreeMap::new(),
                child_count: None,
            }],
            count_hint: None,
        };
        let result = matrix_list_to_json(&list, &doc, &config).unwrap();
        assert!(result["__type__"] == json!("User"));
        assert!(result["__schema__"] == json!(["id"]));
    }

    #[test]
    fn test_matrix_list_to_json_empty() {
        let doc = Document::new((1, 0));
        let config = ToJsonConfig::default();
        let list = MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string()],
            rows: vec![],
            count_hint: None,
        };
        let result = matrix_list_to_json(&list, &doc, &config).unwrap();
        assert_eq!(result, json!([]));
    }

    #[test]
    fn test_matrix_list_to_json_with_count_hint() {
        let doc = Document::new((1, 0));
        let config = ToJsonConfig {
            include_metadata: true,
            flatten_lists: false,
            include_children: true,
        };
        let list = MatrixList {
            type_name: "Team".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![Node {
                type_name: "Team".to_string(),
                id: "1".to_string(),
                fields: vec![Value::String("1".into()), Value::String("Alpha".into())],
                children: BTreeMap::new(),
                child_count: None,
            }],
            count_hint: Some(5),
        };
        let result = matrix_list_to_json(&list, &doc, &config).unwrap();

        // Should include count_hint in metadata
        assert_eq!(result["__count_hint__"], json!(5));
        assert_eq!(result["__type__"], json!("Team"));
        assert_eq!(result["__schema__"], json!(["id", "name"]));
    }
}
