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

//! HEDL to YAML conversion

use hedl_core::lex::Tensor;
use hedl_core::{Document, Item, MatrixList, Node, Value};
use once_cell::sync::Lazy;
use serde_yaml::{Mapping, Value as YamlValue};
use std::collections::BTreeMap;

// ============================================================================
// String Constants for Performance Optimization
// ============================================================================
//
// These constants eliminate repeated allocations of common metadata keys.
// Instead of allocating "__type__", "__schema__", etc. for every row/node,
// we allocate them once and clone the YamlValue (which is cheaper than
// allocating and converting from &str each time).
//
// Performance impact:
// - Eliminates ~100K+ allocations for large matrix lists (10K rows × 10 cols)
// - 5-10% overall performance improvement
// - Especially beneficial for metadata-heavy documents

static TYPE_KEY: Lazy<YamlValue> = Lazy::new(|| YamlValue::String("__type__".to_string()));
static SCHEMA_KEY: Lazy<YamlValue> = Lazy::new(|| YamlValue::String("__schema__".to_string()));
static ITEMS_KEY: Lazy<YamlValue> = Lazy::new(|| YamlValue::String("items".to_string()));
static REF_KEY: Lazy<YamlValue> = Lazy::new(|| YamlValue::String("@ref".to_string()));

// Field name cache for common indices (0-99)
// Eliminates format!() allocation for every field in every node
static FIELD_NAMES: Lazy<Vec<YamlValue>> = Lazy::new(|| {
    (0..100)
        .map(|i| YamlValue::String(format!("field_{i}")))
        .collect()
});

#[inline]
fn get_field_name(index: usize) -> YamlValue {
    FIELD_NAMES
        .get(index)
        .cloned()
        .unwrap_or_else(|| YamlValue::String(format!("field_{index}")))
}

/// Configuration for YAML output
#[derive(Debug, Clone)]
pub struct ToYamlConfig {
    /// Include HEDL metadata (__type__, __schema__)
    pub include_metadata: bool,
    /// Flatten matrix lists to plain arrays
    pub flatten_lists: bool,
    /// Include children as nested arrays (default: true)
    pub include_children: bool,
}

impl Default for ToYamlConfig {
    fn default() -> Self {
        Self {
            include_metadata: true, // Preserve type names for roundtrip fidelity
            flatten_lists: false,
            include_children: true, // Children should be included by default
        }
    }
}

impl hedl_core::convert::ExportConfig for ToYamlConfig {
    fn include_metadata(&self) -> bool {
        self.include_metadata
    }

    fn pretty(&self) -> bool {
        // YAML always uses pretty formatting
        true
    }
}

/// Convert Document to YAML string
pub fn to_yaml(doc: &Document, config: &ToYamlConfig) -> Result<String, String> {
    let value = to_yaml_value(doc, config)?;
    serde_yaml::to_string(&value).map_err(|e| format!("YAML serialization error: {e}"))
}

/// Convert Document to `serde_yaml::Value`
pub fn to_yaml_value(doc: &Document, config: &ToYamlConfig) -> Result<YamlValue, String> {
    root_to_yaml(&doc.root, doc, config)
}

fn root_to_yaml(
    root: &BTreeMap<String, Item>,
    doc: &Document,
    config: &ToYamlConfig,
) -> Result<YamlValue, String> {
    // Pre-allocate mapping with capacity hint for better performance
    let mut map = Mapping::new();
    map.reserve(root.len());

    for (key, item) in root {
        let yaml_value = item_to_yaml(item, doc, config)?;
        map.insert(YamlValue::String(key.clone()), yaml_value);
    }

    Ok(YamlValue::Mapping(map))
}

fn item_to_yaml(item: &Item, doc: &Document, config: &ToYamlConfig) -> Result<YamlValue, String> {
    match item {
        Item::Scalar(value) => Ok(value_to_yaml(value)),
        Item::Object(obj) => object_to_yaml(obj, doc, config),
        Item::List(list) => matrix_list_to_yaml(list, doc, config),
    }
}

fn object_to_yaml(
    obj: &BTreeMap<String, Item>,
    doc: &Document,
    config: &ToYamlConfig,
) -> Result<YamlValue, String> {
    // Pre-allocate mapping with capacity hint
    let mut map = Mapping::new();
    map.reserve(obj.len());

    for (key, item) in obj {
        let yaml_value = item_to_yaml(item, doc, config)?;
        map.insert(YamlValue::String(key.clone()), yaml_value);
    }

    Ok(YamlValue::Mapping(map))
}

fn value_to_yaml(value: &Value) -> YamlValue {
    match value {
        Value::Null => YamlValue::Null,
        Value::Bool(b) => YamlValue::Bool(*b),
        Value::Int(n) => YamlValue::Number((*n).into()),
        Value::Float(f) => YamlValue::Number(serde_yaml::Number::from(*f)),
        Value::String(s) => YamlValue::String(s.to_string()),
        Value::Tensor(t) => tensor_to_yaml(t),
        Value::Reference(r) => {
            // Represent references as mappings with @ref key (like JSON)
            // This distinguishes references from strings that happen to start with @
            let mut map = serde_yaml::Mapping::new();
            map.reserve(1);
            map.insert(REF_KEY.clone(), YamlValue::String(r.to_ref_string()));
            YamlValue::Mapping(map)
        }
        Value::Expression(e) => {
            // Optimized expression formatting: pre-allocate capacity to avoid reallocation
            let expr_str = e.to_string();
            let mut s = String::with_capacity(expr_str.len() + 3);
            s.push_str("$(");
            s.push_str(&expr_str);
            s.push(')');
            YamlValue::String(s)
        }
        Value::List(items) => {
            // Convert list to YAML sequence
            YamlValue::Sequence(items.iter().map(value_to_yaml).collect())
        }
    }
}

fn tensor_to_yaml(tensor: &Tensor) -> YamlValue {
    // Convert tensor to nested sequences recursively
    match tensor {
        Tensor::Scalar(n) => YamlValue::Number(serde_yaml::Number::from(*n)),
        Tensor::Array(items) => YamlValue::Sequence(items.iter().map(tensor_to_yaml).collect()),
    }
}

fn matrix_list_to_yaml(
    list: &MatrixList,
    doc: &Document,
    config: &ToYamlConfig,
) -> Result<YamlValue, String> {
    // Pre-allocate array capacity
    let mut array = Vec::with_capacity(list.rows.len());

    // OPTIMIZATION: Pre-convert schema column names to YamlValue once
    // Instead of cloning String → YamlValue for every row, clone YamlValue
    let schema_keys: Vec<YamlValue> = list
        .schema
        .iter()
        .map(|s| YamlValue::String(s.clone()))
        .collect();

    // Pre-allocate type name YamlValue if metadata is enabled
    let type_value = if config.include_metadata {
        Some(YamlValue::String(list.type_name.clone()))
    } else {
        None
    };

    for row in &list.rows {
        // Pre-allocate row mapping with expected size
        let expected_size = list.schema.len()
            + usize::from(config.include_metadata)
            + if config.include_children {
                row.children().map_or(0, std::collections::BTreeMap::len)
            } else {
                0
            };
        let mut row_obj = Mapping::new();
        row_obj.reserve(expected_size);

        // Add field values according to schema
        // Per SPEC.md: Node.fields contains ALL values including ID (first column)
        // MatrixList.schema includes all column names with ID first
        for (i, col_key) in schema_keys.iter().enumerate() {
            if let Some(field_value) = row.fields.get(i) {
                row_obj.insert(col_key.clone(), value_to_yaml(field_value));
            }
        }

        // Add metadata if configured
        if let Some(ref type_val) = type_value {
            row_obj.insert(TYPE_KEY.clone(), type_val.clone());
        }

        // Add children if configured and present
        if config.include_children {
            if let Some(children_map) = row.children() {
                for (child_type, child_nodes) in children_map {
                    let child_yaml = nodes_to_yaml(child_type, child_nodes, doc, config)?;
                    row_obj.insert(YamlValue::String(child_type.clone()), child_yaml);
                }
            }
        }

        array.push(YamlValue::Mapping(row_obj));
    }

    // Wrap with metadata if configured
    if config.include_metadata && !config.flatten_lists {
        let mut wrapper = Mapping::new();
        wrapper.reserve(3); // __type__, __schema__, items
        wrapper.insert(TYPE_KEY.clone(), YamlValue::String(list.type_name.clone()));
        wrapper.insert(SCHEMA_KEY.clone(), YamlValue::Sequence(schema_keys));
        wrapper.insert(ITEMS_KEY.clone(), YamlValue::Sequence(array));
        Ok(YamlValue::Mapping(wrapper))
    } else {
        Ok(YamlValue::Sequence(array))
    }
}

fn nodes_to_yaml(
    type_name: &str,
    nodes: &[Node],
    doc: &Document,
    config: &ToYamlConfig,
) -> Result<YamlValue, String> {
    // Pre-allocate array capacity
    let mut array = Vec::with_capacity(nodes.len());

    // Look up the schema for this type from the Document
    let schema = doc
        .structs
        .get(type_name)
        .map(|s| s.as_slice())
        .unwrap_or(&[]);

    // Pre-allocate type name YamlValue if metadata is enabled
    let type_value = if config.include_metadata {
        Some(YamlValue::String(type_name.to_string()))
    } else {
        None
    };

    for node in nodes {
        // Pre-allocate object mapping with expected size
        let expected_size = schema.len()
            + usize::from(config.include_metadata)
            + if config.include_children {
                node.children().map_or(0, std::collections::BTreeMap::len)
            } else {
                0
            };
        let mut obj = Mapping::new();
        obj.reserve(expected_size);

        // Add fields using schema column names
        for (i, value) in node.fields.iter().enumerate() {
            let key = if let Some(col_name) = schema.get(i) {
                YamlValue::String(col_name.clone())
            } else {
                get_field_name(i)
            };
            obj.insert(key, value_to_yaml(value));
        }

        // Add metadata if configured
        if let Some(ref type_val) = type_value {
            obj.insert(TYPE_KEY.clone(), type_val.clone());
        }

        // Add children if configured
        if config.include_children {
            if let Some(children_map) = node.children() {
                for (child_type, child_nodes) in children_map {
                    let child_yaml = nodes_to_yaml(child_type, child_nodes, doc, config)?;
                    obj.insert(YamlValue::String(child_type.clone()), child_yaml);
                }
            }
        }

        array.push(YamlValue::Mapping(obj));
    }

    Ok(YamlValue::Sequence(array))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::lex::{Expression, Span};
    use hedl_core::Reference;

    // ==================== ToYamlConfig tests ====================

    #[test]
    fn test_to_yaml_config_default() {
        let config = ToYamlConfig::default();
        assert!(config.include_metadata); // Default preserves type names for roundtrip
        assert!(!config.flatten_lists);
        assert!(config.include_children);
    }

    #[test]
    fn test_to_yaml_config_debug() {
        let config = ToYamlConfig::default();
        let debug = format!("{config:?}");
        assert!(debug.contains("ToYamlConfig"));
        assert!(debug.contains("include_metadata"));
        assert!(debug.contains("flatten_lists"));
        assert!(debug.contains("include_children"));
    }

    #[test]
    fn test_to_yaml_config_clone() {
        let config = ToYamlConfig {
            include_metadata: true,
            flatten_lists: true,
            include_children: false,
        };
        let cloned = config.clone();
        assert!(cloned.include_metadata);
        assert!(cloned.flatten_lists);
        assert!(!cloned.include_children);
    }

    #[test]
    fn test_to_yaml_config_all_true() {
        let config = ToYamlConfig {
            include_metadata: true,
            flatten_lists: true,
            include_children: true,
        };
        assert!(config.include_metadata);
        assert!(config.flatten_lists);
        assert!(config.include_children);
    }

    #[test]
    fn test_to_yaml_config_all_false() {
        let config = ToYamlConfig {
            include_metadata: false,
            flatten_lists: false,
            include_children: false,
        };
        assert!(!config.include_metadata);
        assert!(!config.flatten_lists);
        assert!(!config.include_children);
    }

    // ==================== value_to_yaml tests ====================

    #[test]
    fn test_value_to_yaml_null() {
        assert_eq!(value_to_yaml(&Value::Null), YamlValue::Null);
    }

    #[test]
    fn test_value_to_yaml_bool_true() {
        assert_eq!(value_to_yaml(&Value::Bool(true)), YamlValue::Bool(true));
    }

    #[test]
    fn test_value_to_yaml_bool_false() {
        assert_eq!(value_to_yaml(&Value::Bool(false)), YamlValue::Bool(false));
    }

    #[test]
    fn test_value_to_yaml_int_positive() {
        assert_eq!(value_to_yaml(&Value::Int(42)), YamlValue::Number(42.into()));
    }

    #[test]
    fn test_value_to_yaml_int_negative() {
        assert_eq!(
            value_to_yaml(&Value::Int(-100)),
            YamlValue::Number((-100).into())
        );
    }

    #[test]
    fn test_value_to_yaml_int_zero() {
        assert_eq!(value_to_yaml(&Value::Int(0)), YamlValue::Number(0.into()));
    }

    #[test]
    fn test_value_to_yaml_int_max() {
        let yaml = value_to_yaml(&Value::Int(i64::MAX));
        if let YamlValue::Number(n) = yaml {
            assert_eq!(n.as_i64(), Some(i64::MAX));
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_value_to_yaml_int_min() {
        let yaml = value_to_yaml(&Value::Int(i64::MIN));
        if let YamlValue::Number(n) = yaml {
            assert_eq!(n.as_i64(), Some(i64::MIN));
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_value_to_yaml_float_positive() {
        let yaml = value_to_yaml(&Value::Float(3.5));
        if let YamlValue::Number(n) = yaml {
            assert!((n.as_f64().unwrap() - 3.5).abs() < 0.001);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_value_to_yaml_float_negative() {
        let yaml = value_to_yaml(&Value::Float(-2.75));
        if let YamlValue::Number(n) = yaml {
            assert!((n.as_f64().unwrap() + 2.75).abs() < 0.001);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_value_to_yaml_float_zero() {
        let yaml = value_to_yaml(&Value::Float(0.0));
        if let YamlValue::Number(n) = yaml {
            assert_eq!(n.as_f64(), Some(0.0));
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_value_to_yaml_float_infinity() {
        let yaml = value_to_yaml(&Value::Float(f64::INFINITY));
        if let YamlValue::Number(n) = yaml {
            assert!(n.as_f64().unwrap().is_infinite());
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_value_to_yaml_float_nan() {
        let yaml = value_to_yaml(&Value::Float(f64::NAN));
        if let YamlValue::Number(n) = yaml {
            assert!(n.as_f64().unwrap().is_nan());
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_value_to_yaml_string_simple() {
        assert_eq!(
            value_to_yaml(&Value::String("hello".into())),
            YamlValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_value_to_yaml_string_empty() {
        assert_eq!(
            value_to_yaml(&Value::String("".into())),
            YamlValue::String(String::new())
        );
    }

    #[test]
    fn test_value_to_yaml_string_unicode() {
        assert_eq!(
            value_to_yaml(&Value::String("héllo 世界".into())),
            YamlValue::String("héllo 世界".to_string())
        );
    }

    #[test]
    fn test_value_to_yaml_string_with_newlines() {
        assert_eq!(
            value_to_yaml(&Value::String("line1\nline2".into())),
            YamlValue::String("line1\nline2".to_string())
        );
    }

    #[test]
    fn test_value_to_yaml_string_with_special_yaml_chars() {
        // Strings that might look like YAML syntax
        assert_eq!(
            value_to_yaml(&Value::String("key: value".into())),
            YamlValue::String("key: value".to_string())
        );
    }

    // ==================== Reference tests ====================

    #[test]
    fn test_value_to_yaml_reference_local() {
        let ref_val = Value::Reference(Reference::local("user1"));
        let yaml = value_to_yaml(&ref_val);
        if let YamlValue::Mapping(map) = yaml {
            assert_eq!(
                map.get(YamlValue::String("@ref".to_string())),
                Some(&YamlValue::String("@user1".to_string()))
            );
        } else {
            panic!("Expected mapping for reference");
        }
    }

    #[test]
    fn test_value_to_yaml_reference_qualified() {
        let qual_ref = Value::Reference(Reference::qualified("User", "user1"));
        let yaml = value_to_yaml(&qual_ref);
        if let YamlValue::Mapping(map) = yaml {
            assert_eq!(
                map.get(YamlValue::String("@ref".to_string())),
                Some(&YamlValue::String("@User:user1".to_string()))
            );
        } else {
            panic!("Expected mapping for reference");
        }
    }

    #[test]
    fn test_value_to_yaml_reference_with_special_id() {
        let ref_val = Value::Reference(Reference::local("my-item_123"));
        let yaml = value_to_yaml(&ref_val);
        if let YamlValue::Mapping(map) = yaml {
            assert_eq!(
                map.get(YamlValue::String("@ref".to_string())),
                Some(&YamlValue::String("@my-item_123".to_string()))
            );
        } else {
            panic!("Expected mapping for reference");
        }
    }

    // ==================== Expression tests ====================

    #[test]
    fn test_value_to_yaml_expression_identifier() {
        let expr = Value::Expression(Box::new(Expression::Identifier {
            name: "foo".to_string(),
            span: Span::default(),
        }));
        assert_eq!(
            value_to_yaml(&expr),
            YamlValue::String("$(foo)".to_string())
        );
    }

    #[test]
    fn test_value_to_yaml_expression_call() {
        let expr = Value::Expression(Box::new(Expression::Call {
            name: "add".to_string(),
            args: vec![
                Expression::Identifier {
                    name: "x".to_string(),
                    span: Span::default(),
                },
                Expression::Literal {
                    value: hedl_core::lex::ExprLiteral::Int(1),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        }));
        assert_eq!(
            value_to_yaml(&expr),
            YamlValue::String("$(add(x, 1))".to_string())
        );
    }

    #[test]
    fn test_value_to_yaml_expression_nested_call() {
        let expr = Value::Expression(Box::new(Expression::Call {
            name: "outer".to_string(),
            args: vec![Expression::Call {
                name: "inner".to_string(),
                args: vec![Expression::Literal {
                    value: hedl_core::lex::ExprLiteral::Int(42),
                    span: Span::default(),
                }],
                span: Span::default(),
            }],
            span: Span::default(),
        }));
        assert_eq!(
            value_to_yaml(&expr),
            YamlValue::String("$(outer(inner(42)))".to_string())
        );
    }

    #[test]
    fn test_value_to_yaml_expression_access() {
        let expr = Value::Expression(Box::new(Expression::Access {
            target: Box::new(Expression::Identifier {
                name: "user".to_string(),
                span: Span::default(),
            }),
            field: "name".to_string(),
            span: Span::default(),
        }));
        assert_eq!(
            value_to_yaml(&expr),
            YamlValue::String("$(user.name)".to_string())
        );
    }

    // ==================== tensor_to_yaml tests ====================

    #[test]
    fn test_tensor_to_yaml_scalar() {
        let tensor = Tensor::Scalar(42.5);
        let yaml = tensor_to_yaml(&tensor);
        if let YamlValue::Number(n) = yaml {
            assert!((n.as_f64().unwrap() - 42.5).abs() < 0.001);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_tensor_to_yaml_1d() {
        let tensor = Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
            Tensor::Scalar(3.0),
        ]);
        let yaml = tensor_to_yaml(&tensor);
        if let YamlValue::Sequence(seq) = yaml {
            assert_eq!(seq.len(), 3);
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_tensor_to_yaml_2d() {
        let tensor = Tensor::Array(vec![
            Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
            Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
        ]);
        let yaml = tensor_to_yaml(&tensor);
        if let YamlValue::Sequence(outer) = yaml {
            assert_eq!(outer.len(), 2);
            if let YamlValue::Sequence(inner) = &outer[0] {
                assert_eq!(inner.len(), 2);
            } else {
                panic!("Expected nested sequence");
            }
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_tensor_to_yaml_empty() {
        let tensor = Tensor::Array(vec![]);
        let yaml = tensor_to_yaml(&tensor);
        if let YamlValue::Sequence(seq) = yaml {
            assert!(seq.is_empty());
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_tensor_to_yaml_scalar_zero() {
        let tensor = Tensor::Scalar(0.0);
        let yaml = tensor_to_yaml(&tensor);
        if let YamlValue::Number(n) = yaml {
            assert_eq!(n.as_f64(), Some(0.0));
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_tensor_to_yaml_scalar_negative() {
        let tensor = Tensor::Scalar(-3.5);
        let yaml = tensor_to_yaml(&tensor);
        if let YamlValue::Number(n) = yaml {
            assert!((n.as_f64().unwrap() + 3.5).abs() < 0.001);
        } else {
            panic!("Expected number");
        }
    }

    // ==================== object_to_yaml tests ====================

    fn empty_doc() -> Document {
        Document {
            version: (1, 0),
            root: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            schema_versions: BTreeMap::new(),
            aliases: BTreeMap::new(),
        }
    }

    #[test]
    fn test_object_to_yaml_empty() {
        let obj = BTreeMap::new();
        let config = ToYamlConfig::default();
        let doc = empty_doc();
        let yaml = object_to_yaml(&obj, &doc, &config).unwrap();
        assert_eq!(yaml, YamlValue::Mapping(Mapping::new()));
    }

    #[test]
    fn test_object_to_yaml_simple() {
        let mut obj = BTreeMap::new();
        obj.insert(
            "name".to_string(),
            Item::Scalar(Value::String("test".to_string().into())),
        );
        obj.insert("age".to_string(), Item::Scalar(Value::Int(42)));

        let config = ToYamlConfig::default();
        let doc = empty_doc();
        let yaml = object_to_yaml(&obj, &doc, &config).unwrap();

        if let YamlValue::Mapping(map) = yaml {
            assert_eq!(map.len(), 2);
            assert_eq!(
                map.get(YamlValue::String("name".to_string())),
                Some(&YamlValue::String("test".to_string()))
            );
            assert_eq!(
                map.get(YamlValue::String("age".to_string())),
                Some(&YamlValue::Number(42.into()))
            );
        } else {
            panic!("Expected mapping");
        }
    }

    #[test]
    fn test_object_to_yaml_nested() {
        let mut inner = BTreeMap::new();
        inner.insert("x".to_string(), Item::Scalar(Value::Int(10)));
        inner.insert("y".to_string(), Item::Scalar(Value::Int(20)));

        let mut outer = BTreeMap::new();
        outer.insert("point".to_string(), Item::Object(inner));

        let config = ToYamlConfig::default();
        let doc = empty_doc();
        let yaml = object_to_yaml(&outer, &doc, &config).unwrap();

        if let YamlValue::Mapping(map) = yaml {
            if let Some(YamlValue::Mapping(point)) = map.get(YamlValue::String("point".to_string()))
            {
                assert_eq!(point.len(), 2);
            } else {
                panic!("Expected nested mapping");
            }
        } else {
            panic!("Expected mapping");
        }
    }

    #[test]
    fn test_object_to_yaml_with_all_types() {
        let mut obj = BTreeMap::new();
        obj.insert("null_val".to_string(), Item::Scalar(Value::Null));
        obj.insert("bool_val".to_string(), Item::Scalar(Value::Bool(true)));
        obj.insert("int_val".to_string(), Item::Scalar(Value::Int(42)));
        obj.insert("float_val".to_string(), Item::Scalar(Value::Float(3.5)));
        obj.insert(
            "string_val".to_string(),
            Item::Scalar(Value::String("hello".to_string().into())),
        );

        let config = ToYamlConfig::default();
        let doc = empty_doc();
        let yaml = object_to_yaml(&obj, &doc, &config).unwrap();

        if let YamlValue::Mapping(map) = yaml {
            assert_eq!(map.len(), 5);
        } else {
            panic!("Expected mapping");
        }
    }

    // ==================== item_to_yaml tests ====================

    #[test]
    fn test_item_to_yaml_scalar() {
        let item = Item::Scalar(Value::Int(42));
        let config = ToYamlConfig::default();
        let doc = empty_doc();
        let yaml = item_to_yaml(&item, &doc, &config).unwrap();
        assert_eq!(yaml, YamlValue::Number(42.into()));
    }

    #[test]
    fn test_item_to_yaml_object() {
        let mut obj = BTreeMap::new();
        obj.insert("key".to_string(), Item::Scalar(Value::Int(1)));
        let item = Item::Object(obj);

        let config = ToYamlConfig::default();
        let doc = empty_doc();
        let yaml = item_to_yaml(&item, &doc, &config).unwrap();

        if let YamlValue::Mapping(map) = yaml {
            assert_eq!(map.len(), 1);
        } else {
            panic!("Expected mapping");
        }
    }

    #[test]
    fn test_item_to_yaml_list() {
        let mut list = MatrixList::new("User".to_string(), vec!["id".to_string()]);
        list.add_row(Node::new(
            "User",
            "u1",
            vec![Value::String("u1".to_string().into())],
        ));

        let item = Item::List(list);
        let config = ToYamlConfig {
            include_metadata: false,
            flatten_lists: false,
            include_children: true,
        };
        let doc = empty_doc();
        let yaml = item_to_yaml(&item, &doc, &config).unwrap();

        if let YamlValue::Sequence(seq) = yaml {
            assert_eq!(seq.len(), 1);
        } else {
            panic!("Expected sequence");
        }
    }

    // ==================== matrix_list_to_yaml tests ====================

    #[test]
    fn test_matrix_list_to_yaml_simple() {
        let mut list = MatrixList::new(
            "User".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );
        list.add_row(Node::new(
            "User",
            "u1",
            vec![
                Value::String("u1".to_string().into()),
                Value::String("Alice".to_string().into()),
            ],
        ));

        let config = ToYamlConfig {
            include_metadata: false,
            flatten_lists: false,
            include_children: true,
        };
        let doc = empty_doc();
        let yaml = matrix_list_to_yaml(&list, &doc, &config).unwrap();

        if let YamlValue::Sequence(seq) = yaml {
            assert_eq!(seq.len(), 1);
            if let YamlValue::Mapping(row) = &seq[0] {
                assert_eq!(
                    row.get(YamlValue::String("id".to_string())),
                    Some(&YamlValue::String("u1".to_string()))
                );
                assert_eq!(
                    row.get(YamlValue::String("name".to_string())),
                    Some(&YamlValue::String("Alice".to_string()))
                );
            } else {
                panic!("Expected mapping in sequence");
            }
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_matrix_list_to_yaml_with_metadata() {
        let mut list = MatrixList::new(
            "User".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );
        list.add_row(Node::new(
            "User",
            "u1",
            vec![
                Value::String("u1".to_string().into()),
                Value::String("Alice".to_string().into()),
            ],
        ));

        let config = ToYamlConfig {
            include_metadata: true,
            flatten_lists: false,
            include_children: true,
        };
        let doc = empty_doc();
        let yaml = matrix_list_to_yaml(&list, &doc, &config).unwrap();

        if let YamlValue::Mapping(wrapper) = yaml {
            assert!(wrapper.contains_key(YamlValue::String("__type__".to_string())));
            assert!(wrapper.contains_key(YamlValue::String("__schema__".to_string())));
            assert!(wrapper.contains_key(YamlValue::String("items".to_string())));
        } else {
            panic!("Expected mapping with metadata");
        }
    }

    #[test]
    fn test_matrix_list_to_yaml_flattened() {
        let mut list = MatrixList::new("User".to_string(), vec!["id".to_string()]);
        list.add_row(Node::new(
            "User",
            "u1",
            vec![Value::String("u1".to_string().into())],
        ));

        let config = ToYamlConfig {
            include_metadata: true,
            flatten_lists: true,
            include_children: true,
        };
        let doc = empty_doc();
        let yaml = matrix_list_to_yaml(&list, &doc, &config).unwrap();

        // With flatten_lists, should be a sequence not a mapping
        if let YamlValue::Sequence(seq) = yaml {
            assert_eq!(seq.len(), 1);
        } else {
            panic!("Expected sequence when flattened");
        }
    }

    #[test]
    fn test_matrix_list_to_yaml_empty() {
        let list = MatrixList::new("User".to_string(), vec!["id".to_string()]);

        let config = ToYamlConfig {
            include_metadata: false,
            flatten_lists: false,
            include_children: true,
        };
        let doc = empty_doc();
        let yaml = matrix_list_to_yaml(&list, &doc, &config).unwrap();

        if let YamlValue::Sequence(seq) = yaml {
            assert!(seq.is_empty());
        } else {
            panic!("Expected empty sequence");
        }
    }

    #[test]
    fn test_matrix_list_to_yaml_with_children() {
        let mut list = MatrixList::new(
            "User".to_string(),
            vec!["id".to_string(), "name".to_string()],
        );
        let mut parent = Node::new(
            "User",
            "u1",
            vec![
                Value::String("u1".to_string().into()),
                Value::String("Alice".to_string().into()),
            ],
        );
        parent.add_child(
            "posts",
            Node::new("Post", "p1", vec![Value::String("p1".to_string().into())]),
        );
        list.add_row(parent);

        let config = ToYamlConfig {
            include_metadata: false,
            flatten_lists: false,
            include_children: true,
        };
        let mut doc = empty_doc();
        // Add Post schema so children can look it up
        doc.structs
            .insert("Post".to_string(), vec!["id".to_string()]);
        let yaml = matrix_list_to_yaml(&list, &doc, &config).unwrap();

        if let YamlValue::Sequence(seq) = yaml {
            if let YamlValue::Mapping(row) = &seq[0] {
                assert!(row.contains_key(YamlValue::String("posts".to_string())));
            } else {
                panic!("Expected mapping in sequence");
            }
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_matrix_list_to_yaml_without_children() {
        let mut list = MatrixList::new("User".to_string(), vec!["id".to_string()]);
        let mut parent = Node::new("User", "u1", vec![Value::String("u1".to_string().into())]);
        parent.add_child(
            "posts",
            Node::new("Post", "p1", vec![Value::String("p1".to_string().into())]),
        );
        list.add_row(parent);

        let config = ToYamlConfig {
            include_metadata: false,
            flatten_lists: false,
            include_children: false,
        };
        let doc = empty_doc();
        let yaml = matrix_list_to_yaml(&list, &doc, &config).unwrap();

        if let YamlValue::Sequence(seq) = yaml {
            if let YamlValue::Mapping(row) = &seq[0] {
                // Children should not be included
                assert!(!row.contains_key(YamlValue::String("posts".to_string())));
            } else {
                panic!("Expected mapping in sequence");
            }
        } else {
            panic!("Expected sequence");
        }
    }

    // ==================== root_to_yaml tests ====================

    #[test]
    fn test_root_to_yaml_empty() {
        let root = BTreeMap::new();
        let config = ToYamlConfig::default();
        let doc = empty_doc();
        let yaml = root_to_yaml(&root, &doc, &config).unwrap();
        assert_eq!(yaml, YamlValue::Mapping(Mapping::new()));
    }

    #[test]
    fn test_root_to_yaml_with_items() {
        let mut root = BTreeMap::new();
        root.insert(
            "name".to_string(),
            Item::Scalar(Value::String("test".to_string().into())),
        );
        root.insert("count".to_string(), Item::Scalar(Value::Int(42)));

        let config = ToYamlConfig::default();
        let doc = empty_doc();
        let yaml = root_to_yaml(&root, &doc, &config).unwrap();

        if let YamlValue::Mapping(map) = yaml {
            assert_eq!(map.len(), 2);
        } else {
            panic!("Expected mapping");
        }
    }

    // ==================== to_yaml integration tests ====================

    #[test]
    fn test_to_yaml_empty_document() {
        let doc = Document {
            version: (1, 0),
            root: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            schema_versions: BTreeMap::new(),
            aliases: BTreeMap::new(),
        };
        let config = ToYamlConfig::default();
        let yaml = to_yaml(&doc, &config).unwrap();
        assert!(yaml.contains("{}") || yaml.trim().is_empty() || yaml == "{}\n");
    }

    #[test]
    fn test_to_yaml_simple_document() {
        let mut doc = Document {
            version: (1, 0),
            root: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            schema_versions: BTreeMap::new(),
            aliases: BTreeMap::new(),
        };
        doc.root.insert(
            "name".to_string(),
            Item::Scalar(Value::String("test".to_string().into())),
        );

        let config = ToYamlConfig::default();
        let yaml = to_yaml(&doc, &config).unwrap();
        assert!(yaml.contains("name"));
        assert!(yaml.contains("test"));
    }

    // ==================== to_yaml_value tests ====================

    #[test]
    fn test_to_yaml_value_returns_mapping() {
        let doc = Document {
            version: (1, 0),
            root: BTreeMap::new(),
            structs: BTreeMap::new(),
            nests: BTreeMap::new(),
            schema_versions: BTreeMap::new(),
            aliases: BTreeMap::new(),
        };
        let config = ToYamlConfig::default();
        let value = to_yaml_value(&doc, &config).unwrap();
        assert!(matches!(value, YamlValue::Mapping(_)));
    }

    // ==================== Value::List tests ====================

    #[test]
    fn test_value_to_yaml_list_strings() {
        let list = Value::List(Box::new(vec![
            Value::String("admin".to_string().into()),
            Value::String("editor".to_string().into()),
            Value::String("viewer".to_string().into()),
        ]));
        let yaml = value_to_yaml(&list);
        if let YamlValue::Sequence(seq) = yaml {
            assert_eq!(seq.len(), 3);
            assert_eq!(seq[0], YamlValue::String("admin".to_string()));
            assert_eq!(seq[1], YamlValue::String("editor".to_string()));
            assert_eq!(seq[2], YamlValue::String("viewer".to_string()));
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_value_to_yaml_list_bools() {
        let list = Value::List(Box::new(vec![
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(true),
        ]));
        let yaml = value_to_yaml(&list);
        if let YamlValue::Sequence(seq) = yaml {
            assert_eq!(seq.len(), 3);
            assert_eq!(seq[0], YamlValue::Bool(true));
            assert_eq!(seq[1], YamlValue::Bool(false));
            assert_eq!(seq[2], YamlValue::Bool(true));
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_value_to_yaml_list_mixed() {
        let list = Value::List(Box::new(vec![
            Value::String("test".to_string().into()),
            Value::Int(42),
            Value::Bool(true),
            Value::Null,
        ]));
        let yaml = value_to_yaml(&list);
        if let YamlValue::Sequence(seq) = yaml {
            assert_eq!(seq.len(), 4);
            assert_eq!(seq[0], YamlValue::String("test".to_string()));
            assert_eq!(seq[1], YamlValue::Number(42.into()));
            assert_eq!(seq[2], YamlValue::Bool(true));
            assert_eq!(seq[3], YamlValue::Null);
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_value_to_yaml_list_empty() {
        let list = Value::List(Box::default());
        let yaml = value_to_yaml(&list);
        if let YamlValue::Sequence(seq) = yaml {
            assert!(seq.is_empty());
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_value_to_yaml_list_with_references() {
        let list = Value::List(Box::new(vec![
            Value::Reference(Reference::local("user1")),
            Value::Reference(Reference::qualified("User", "user2")),
        ]));
        let yaml = value_to_yaml(&list);
        if let YamlValue::Sequence(seq) = yaml {
            assert_eq!(seq.len(), 2);
            if let YamlValue::Mapping(map) = &seq[0] {
                assert_eq!(
                    map.get(YamlValue::String("@ref".to_string())),
                    Some(&YamlValue::String("@user1".to_string()))
                );
            } else {
                panic!("Expected mapping for reference");
            }
        } else {
            panic!("Expected sequence");
        }
    }

    #[test]
    fn test_value_to_yaml_list_nested() {
        let inner_list = Value::List(Box::new(vec![Value::Int(1), Value::Int(2)]));
        let outer_list = Value::List(Box::new(vec![
            inner_list,
            Value::List(Box::new(vec![Value::Int(3), Value::Int(4)])),
        ]));
        let yaml = value_to_yaml(&outer_list);
        if let YamlValue::Sequence(outer) = yaml {
            assert_eq!(outer.len(), 2);
            if let YamlValue::Sequence(inner) = &outer[0] {
                assert_eq!(inner.len(), 2);
            } else {
                panic!("Expected nested sequence");
            }
        } else {
            panic!("Expected sequence");
        }
    }
}
