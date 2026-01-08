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

//! XML to HEDL conversion

use hedl_core::convert::parse_reference;
use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_core::lex::{parse_expression_token, singularize_and_capitalize};
use hedl_core::lex::Tensor;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::BTreeMap;

/// Maximum recursion depth for XML parsing (prevents stack overflow).
const MAX_RECURSION_DEPTH: usize = 100;

/// Configuration for XML import
#[derive(Debug, Clone)]
pub struct FromXmlConfig {
    /// Default type name for list items without metadata
    pub default_type_name: String,
    /// HEDL version to use
    pub version: (u32, u32),
    /// Try to infer list structures from repeated elements
    pub infer_lists: bool,
}

impl Default for FromXmlConfig {
    fn default() -> Self {
        Self {
            default_type_name: "Item".to_string(),
            version: (1, 0),
            infer_lists: true,
        }
    }
}

impl hedl_core::convert::ImportConfig for FromXmlConfig {
    fn default_type_name(&self) -> &str {
        &self.default_type_name
    }

    fn version(&self) -> (u32, u32) {
        self.version
    }
}

/// Convert XML string to HEDL Document
pub fn from_xml(xml: &str, config: &FromXmlConfig) -> Result<Document, String> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut doc = Document::new(config.version);

    // Skip XML declaration and find root element
    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                // Parse version from root if present
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    if key == "version" {
                        if let Some((major, minor)) = parse_version(&value) {
                            doc.version = (major, minor);
                        }
                    }
                }

                // Parse root content
                doc.root = parse_children(&mut reader, &name, config, &mut doc.structs, 0)?;
                break;
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(format!(
                    "XML parse error at position {}: {}",
                    reader.buffer_position(),
                    e
                ))
            }
            _ => {}
        }
    }

    Ok(doc)
}

fn parse_children(
    reader: &mut Reader<&[u8]>,
    parent_name: &str,
    config: &FromXmlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
    depth: usize,
) -> Result<BTreeMap<String, Item>, String> {
    // Security: Prevent stack overflow via deep recursion
    if depth > MAX_RECURSION_DEPTH {
        return Err(format!(
            "XML recursion depth exceeded (max: {})",
            MAX_RECURSION_DEPTH
        ));
    }
    let mut children = BTreeMap::new();
    let mut element_counts: BTreeMap<String, Vec<Item>> = BTreeMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let name = to_hedl_key(&raw_name);
                let elem_owned = e.to_owned();
                let item = parse_element(reader, &elem_owned, config, depth + 1)?;

                // Track repeated elements for list inference
                if config.infer_lists {
                    element_counts.entry(name.clone()).or_default().push(item);
                } else {
                    children.insert(name, item);
                }
            }
            Ok(Event::Empty(e)) => {
                let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let name = to_hedl_key(&raw_name);
                let elem_owned = e.to_owned();
                let item = parse_empty_element(&elem_owned)?;

                if config.infer_lists {
                    element_counts.entry(name.clone()).or_default().push(item);
                } else {
                    children.insert(name, item);
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == parent_name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
    }

    // Process element counts to infer lists
    if config.infer_lists {
        for (name, items) in element_counts {
            if items.len() > 1 {
                // Multiple elements with same name - convert to list
                let list = items_to_matrix_list(&name, items, config, structs)?;
                children.insert(name, Item::List(list));
            } else if let Some(item) = items.into_iter().next() {
                children.insert(name, item);
            }
        }
    }

    Ok(children)
}

fn parse_element(
    reader: &mut Reader<&[u8]>,
    elem: &quick_xml::events::BytesStart,
    config: &FromXmlConfig,
    depth: usize,
) -> Result<Item, String> {
    // Security: Prevent stack overflow via deep recursion
    if depth > MAX_RECURSION_DEPTH {
        return Err(format!(
            "XML recursion depth exceeded (max: {})",
            MAX_RECURSION_DEPTH
        ));
    }
    let name = String::from_utf8_lossy(elem.name().as_ref()).to_string();

    // Extract attributes (convert keys to valid HEDL format)
    let mut attributes = BTreeMap::new();
    let mut is_reference = false;
    for attr in elem.attributes().flatten() {
        let raw_key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let value = String::from_utf8_lossy(&attr.value).to_string();

        // Check for HEDL type marker (used to distinguish references from strings)
        if raw_key == "__hedl_type__" {
            if value == "ref" {
                is_reference = true;
            }
            continue; // Don't include in regular attributes
        }

        let key = to_hedl_key(&raw_key);
        attributes.insert(key, value);
    }

    // Parse content
    let mut text_content = String::new();
    let mut child_elements: BTreeMap<String, Vec<Item>> = BTreeMap::new();
    let mut marked_children: BTreeMap<String, Vec<Item>> = BTreeMap::new(); // Elements with __hedl_child__
    let mut has_children = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                has_children = true;
                let raw_child_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let child_name = to_hedl_key(&raw_child_name);

                // Check for __hedl_child__ marker attribute
                let is_marked_child = e.attributes().any(|attr| {
                    if let Ok(attr) = attr {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        let val = String::from_utf8_lossy(&attr.value);
                        key == "__hedl_child__" && val == "true"
                    } else {
                        false
                    }
                });

                let elem_owned = e.to_owned();
                let child_item = parse_element(reader, &elem_owned, config, depth + 1)?;

                if is_marked_child {
                    marked_children
                        .entry(raw_child_name)
                        .or_default()
                        .push(child_item);
                } else {
                    child_elements
                        .entry(child_name)
                        .or_default()
                        .push(child_item);
                }
            }
            Ok(Event::Empty(e)) => {
                has_children = true;
                let raw_child_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let child_name = to_hedl_key(&raw_child_name);

                // Check for __hedl_child__ marker attribute
                let is_marked_child = e.attributes().any(|attr| {
                    if let Ok(attr) = attr {
                        let key = String::from_utf8_lossy(attr.key.as_ref());
                        let val = String::from_utf8_lossy(&attr.value);
                        key == "__hedl_child__" && val == "true"
                    } else {
                        false
                    }
                });

                let elem_owned = e.to_owned();
                let child_item = parse_empty_element(&elem_owned)?;

                if is_marked_child {
                    marked_children
                        .entry(raw_child_name)
                        .or_default()
                        .push(child_item);
                } else {
                    child_elements
                        .entry(child_name)
                        .or_default()
                        .push(child_item);
                }
            }
            Ok(Event::Text(e)) => {
                text_content.push_str(
                    &e.unescape()
                        .map_err(|e| format!("Text unescape error: {}", e))?,
                );
            }
            Ok(Event::End(e)) => {
                let end_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if end_name == name {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML parse error: {}", e)),
            _ => {}
        }
    }

    // Determine item type
    if has_children {
        // Convert collected child elements, inferring lists for repeated elements
        let mut result_children = BTreeMap::new();
        for (child_name, items) in child_elements {
            if items.len() > 1 && config.infer_lists {
                // Check if all items are scalars/tensors (->tensor) or objects (->matrix list)
                if child_name == "item" && items_are_tensor_elements(&items) {
                    // Convert to tensor
                    let tensor = items_to_tensor(&items)?;
                    result_children.insert(child_name, Item::Scalar(Value::Tensor(tensor)));
                } else {
                    // Multiple elements with same name - convert to list
                    let list =
                        items_to_matrix_list(&child_name, items, config, &mut BTreeMap::new())?;
                    result_children.insert(child_name, Item::List(list));
                }
            } else if let Some(item) = items.into_iter().next() {
                result_children.insert(child_name, item);
            }
        }

        // Convert marked children (elements with __hedl_child__="true") to lists
        // These represent NEST hierarchical children that should be attached to nodes
        for (child_type_raw, child_items) in marked_children {
            if !child_items.is_empty() {
                // Convert to matrix list (even a single child becomes a list)
                let list = items_to_matrix_list(
                    &child_type_raw,
                    child_items,
                    config,
                    &mut BTreeMap::new(),
                )?;
                let child_key = to_hedl_key(&child_type_raw);
                result_children.insert(child_key, Item::List(list));
            }
        }

        // Check if we should flatten: if object has single child that's a list,
        // and the child name is the singular of the parent name, promote the list.
        // This handles XML patterns like <users><user>...</user><user>...</user></users>
        // which should become users: @User[...] not users: { user: @User[...] }
        // BUT: don't flatten if the list has hierarchical children (NEST structures)
        if result_children.len() == 1 {
            let (child_key, child_item) = result_children.iter().next().unwrap();
            if let Item::List(list) = child_item {
                // Don't flatten if any rows have children (hierarchical nesting)
                let has_nested_children = list.rows.iter().any(|node| !node.children.is_empty());
                if !has_nested_children {
                    // Check if child is singular form of parent
                    // Compare case-insensitively because XML element names may have different casing
                    // e.g., post_tags -> PostTag, but child element might be posttag -> Posttag
                    let parent_singular =
                        singularize_and_capitalize(&to_hedl_key(&name)).to_lowercase();
                    let child_type = singularize_and_capitalize(child_key).to_lowercase();
                    if parent_singular == child_type {
                        // Flatten: return the list directly
                        return Ok(result_children.into_values().next().unwrap());
                    }
                }
            }
        }

        // Object with nested elements
        Ok(Item::Object(result_children))
    } else if !text_content.trim().is_empty() {
        // Scalar with text content
        let value = if is_reference {
            // Explicitly marked as reference
            Value::Reference(parse_reference(text_content.trim())?)
        } else {
            parse_value(&text_content)?
        };
        Ok(Item::Scalar(value))
    } else if !attributes.is_empty() {
        // Empty element with attributes - convert to object
        let mut obj = BTreeMap::new();
        for (key, value_str) in attributes {
            let value = parse_value(&value_str)?;
            obj.insert(key, Item::Scalar(value));
        }
        Ok(Item::Object(obj))
    } else {
        // Empty element - null value
        Ok(Item::Scalar(Value::Null))
    }
}

fn parse_empty_element(elem: &quick_xml::events::BytesStart) -> Result<Item, String> {
    let mut attributes = BTreeMap::new();

    for attr in elem.attributes().flatten() {
        let raw_key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let key = to_hedl_key(&raw_key);
        let value = String::from_utf8_lossy(&attr.value).to_string();
        attributes.insert(key, value);
    }

    if attributes.is_empty() {
        Ok(Item::Scalar(Value::Null))
    } else if attributes.len() == 1 && attributes.contains_key("value") {
        // Special case: <elem value="x"/> -> scalar x
        let value_str = attributes.get("value").unwrap();
        let value = parse_value(value_str)?;
        Ok(Item::Scalar(value))
    } else {
        // Multiple attributes - convert to object
        let mut obj = BTreeMap::new();
        for (key, value_str) in attributes {
            let value = parse_value(&value_str)?;
            obj.insert(key, Item::Scalar(value));
        }
        Ok(Item::Object(obj))
    }
}

fn parse_value(s: &str) -> Result<Value, String> {
    let trimmed = s.trim();

    if trimmed.is_empty() {
        return Ok(Value::Null);
    }

    // Note: References are NOT auto-detected from @... pattern.
    // They must be explicitly marked with __hedl_type__="ref" attribute.
    // This prevents strings like "@not-a-ref" from being incorrectly parsed as references.

    // Check for expression pattern $(...)
    if trimmed.starts_with("$(") && trimmed.ends_with(')') {
        let expr =
            parse_expression_token(trimmed).map_err(|e| format!("Invalid expression: {}", e))?;
        return Ok(Value::Expression(expr));
    }

    // Try parsing as boolean
    if trimmed == "true" {
        return Ok(Value::Bool(true));
    }
    if trimmed == "false" {
        return Ok(Value::Bool(false));
    }

    // Try parsing as number
    if let Ok(i) = trimmed.parse::<i64>() {
        return Ok(Value::Int(i));
    }
    if let Ok(f) = trimmed.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    // Default to string
    Ok(Value::String(trimmed.to_string()))
}

fn parse_version(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() >= 2 {
        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        Some((major, minor))
    } else {
        None
    }
}

fn items_to_matrix_list(
    name: &str,
    items: Vec<Item>,
    _config: &FromXmlConfig,
    structs: &mut BTreeMap<String, Vec<String>>,
) -> Result<MatrixList, String> {
    // Infer type name from element name (singularize and capitalize)
    let type_name = singularize_and_capitalize(name);

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

fn infer_schema(items: &[Item]) -> Result<Vec<String>, String> {
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

fn item_to_node(
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
                fields,
                children,
                child_count: None,
            })
        }
        Item::Scalar(value) => {
            // Single scalar - create node with ID value and scalar value
            let id = format!("{}", idx);
            Ok(Node {
                type_name: type_name.to_string(),
                id: id.clone(),
                fields: vec![Value::String(id), value],
                children: BTreeMap::new(),
                child_count: None,
            })
        }
        Item::List(_) => Err("Cannot convert nested list to node".to_string()),
    }
}

/// Convert any string to a valid HEDL key (lowercase snake_case).
/// "Category" -> "category", "UserPost" -> "user_post", "XMLData" -> "xmldata"
fn to_hedl_key(s: &str) -> String {
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
        } else {
            result.push(c);
            prev_was_upper = false;
        }
    }

    // Clean up any double underscores
    while result.contains("__") {
        result = result.replace("__", "_");
    }

    // Remove leading/trailing underscores
    result.trim_matches('_').to_string()
}

/// Check if all items are suitable for tensor representation.
/// Items must be numeric scalars or objects containing only a tensor at the "item" key.
fn items_are_tensor_elements(items: &[Item]) -> bool {
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

/// Convert items to a tensor.
fn items_to_tensor(items: &[Item]) -> Result<Tensor, String> {
    let mut tensor_items = Vec::new();

    for item in items {
        let tensor = match item {
            Item::Scalar(Value::Int(n)) => Tensor::Scalar(*n as f64),
            Item::Scalar(Value::Float(f)) => Tensor::Scalar(*f),
            Item::Scalar(Value::Tensor(t)) => t.clone(),
            Item::Object(obj) if obj.len() == 1 => {
                // Nested tensor element (object with only "item" key containing tensor)
                if let Some(Item::Scalar(Value::Tensor(t))) = obj.get("item") {
                    t.clone()
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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== FromXmlConfig tests ====================

    #[test]
    fn test_from_xml_config_default() {
        let config = FromXmlConfig::default();
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (1, 0));
        assert!(config.infer_lists);
    }

    #[test]
    fn test_from_xml_config_debug() {
        let config = FromXmlConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("FromXmlConfig"));
        assert!(debug.contains("default_type_name"));
        assert!(debug.contains("version"));
        assert!(debug.contains("infer_lists"));
    }

    #[test]
    fn test_from_xml_config_clone() {
        let config = FromXmlConfig {
            default_type_name: "Custom".to_string(),
            version: (2, 1),
            infer_lists: false,
        };
        let cloned = config.clone();
        assert_eq!(cloned.default_type_name, "Custom");
        assert_eq!(cloned.version, (2, 1));
        assert!(!cloned.infer_lists);
    }

    #[test]
    fn test_from_xml_config_custom() {
        let config = FromXmlConfig {
            default_type_name: "MyType".to_string(),
            version: (3, 5),
            infer_lists: false,
        };
        assert_eq!(config.default_type_name, "MyType");
        assert_eq!(config.version, (3, 5));
        assert!(!config.infer_lists);
    }

    // ==================== parse_value tests ====================

    #[test]
    fn test_parse_value_empty() {
        assert_eq!(parse_value("").unwrap(), Value::Null);
        assert_eq!(parse_value("   ").unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_value_bool_true() {
        assert_eq!(parse_value("true").unwrap(), Value::Bool(true));
    }

    #[test]
    fn test_parse_value_bool_false() {
        assert_eq!(parse_value("false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_parse_value_int_positive() {
        assert_eq!(parse_value("42").unwrap(), Value::Int(42));
    }

    #[test]
    fn test_parse_value_int_negative() {
        assert_eq!(parse_value("-100").unwrap(), Value::Int(-100));
    }

    #[test]
    fn test_parse_value_int_zero() {
        assert_eq!(parse_value("0").unwrap(), Value::Int(0));
    }

    #[test]
    fn test_parse_value_float_simple() {
        if let Value::Float(f) = parse_value("3.5").unwrap() {
            assert!((f - 3.5).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_parse_value_float_negative() {
        if let Value::Float(f) = parse_value("-2.5").unwrap() {
            assert!((f + 2.5).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_parse_value_string() {
        assert_eq!(
            parse_value("hello").unwrap(),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_parse_value_string_with_spaces() {
        assert_eq!(
            parse_value("  hello world  ").unwrap(),
            Value::String("hello world".to_string())
        );
    }

    #[test]
    fn test_parse_value_expression_identifier() {
        if let Value::Expression(e) = parse_value("$(foo)").unwrap() {
            assert_eq!(e.to_string(), "foo");
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_value_expression_call() {
        if let Value::Expression(e) = parse_value("$(add(x, 1))").unwrap() {
            assert_eq!(e.to_string(), "add(x, 1)");
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_parse_value_at_string_not_reference() {
        // Strings starting with @ are just strings, not references
        if let Value::String(s) = parse_value("@not-a-ref").unwrap() {
            assert_eq!(s, "@not-a-ref");
        } else {
            panic!("Expected string");
        }
    }

    // ==================== parse_reference tests ====================

    #[test]
    fn test_parse_reference_local() {
        let ref_val = parse_reference("@user123").unwrap();
        assert_eq!(ref_val.type_name, None);
        assert_eq!(ref_val.id, "user123");
    }

    #[test]
    fn test_parse_reference_qualified() {
        let ref_val = parse_reference("@User:123").unwrap();
        assert_eq!(ref_val.type_name, Some("User".to_string()));
        assert_eq!(ref_val.id, "123");
    }

    #[test]
    fn test_parse_reference_with_special_chars() {
        let ref_val = parse_reference("@my-item_123").unwrap();
        assert_eq!(ref_val.type_name, None);
        assert_eq!(ref_val.id, "my-item_123");
    }

    #[test]
    fn test_parse_reference_invalid_no_at() {
        let result = parse_reference("user123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid reference format"));
    }

    // ==================== parse_version tests ====================

    #[test]
    fn test_parse_version_valid() {
        assert_eq!(parse_version("1.0"), Some((1, 0)));
        assert_eq!(parse_version("2.5"), Some((2, 5)));
        assert_eq!(parse_version("10.20"), Some((10, 20)));
    }

    #[test]
    fn test_parse_version_with_patch() {
        // Only major.minor are taken
        assert_eq!(parse_version("1.2.3"), Some((1, 2)));
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(parse_version("invalid"), None);
        assert_eq!(parse_version("1"), None);
        assert_eq!(parse_version(""), None);
        assert_eq!(parse_version("a.b"), None);
    }

    // ==================== to_hedl_key tests ====================

    #[test]
    fn test_to_hedl_key_pascal_case() {
        assert_eq!(to_hedl_key("Category"), "category");
        assert_eq!(to_hedl_key("UserPost"), "user_post");
        assert_eq!(to_hedl_key("UserProfileSettings"), "user_profile_settings");
    }

    #[test]
    fn test_to_hedl_key_acronyms() {
        assert_eq!(to_hedl_key("XMLData"), "xmldata");
        assert_eq!(to_hedl_key("HTTPResponse"), "httpresponse");
    }

    #[test]
    fn test_to_hedl_key_lowercase() {
        assert_eq!(to_hedl_key("users"), "users");
        assert_eq!(to_hedl_key("category"), "category");
    }

    #[test]
    fn test_to_hedl_key_mixed() {
        assert_eq!(to_hedl_key("someXMLData"), "some_xmldata");
        assert_eq!(to_hedl_key("getHTTPResponse"), "get_httpresponse");
    }

    #[test]
    fn test_to_hedl_key_with_underscores() {
        assert_eq!(to_hedl_key("user_name"), "user_name");
        assert_eq!(to_hedl_key("_private"), "private");
    }

    // ==================== items_are_tensor_elements tests ====================

    #[test]
    fn test_items_are_tensor_elements_int_scalars() {
        let items = vec![
            Item::Scalar(Value::Int(1)),
            Item::Scalar(Value::Int(2)),
            Item::Scalar(Value::Int(3)),
        ];
        assert!(items_are_tensor_elements(&items));
    }

    #[test]
    fn test_items_are_tensor_elements_float_scalars() {
        let items = vec![
            Item::Scalar(Value::Float(1.0)),
            Item::Scalar(Value::Float(2.0)),
        ];
        assert!(items_are_tensor_elements(&items));
    }

    #[test]
    fn test_items_are_tensor_elements_tensors() {
        let items = vec![
            Item::Scalar(Value::Tensor(Tensor::Scalar(1.0))),
            Item::Scalar(Value::Tensor(Tensor::Scalar(2.0))),
        ];
        assert!(items_are_tensor_elements(&items));
    }

    #[test]
    fn test_items_are_tensor_elements_mixed_numeric() {
        let items = vec![Item::Scalar(Value::Int(1)), Item::Scalar(Value::Float(2.0))];
        assert!(items_are_tensor_elements(&items));
    }

    #[test]
    fn test_items_are_tensor_elements_with_strings() {
        let items = vec![
            Item::Scalar(Value::Int(1)),
            Item::Scalar(Value::String("hello".to_string())),
        ];
        assert!(!items_are_tensor_elements(&items));
    }

    #[test]
    fn test_items_are_tensor_elements_empty() {
        let items: Vec<Item> = vec![];
        assert!(items_are_tensor_elements(&items));
    }

    // ==================== items_to_tensor tests ====================

    #[test]
    fn test_items_to_tensor_int_scalars() {
        let items = vec![
            Item::Scalar(Value::Int(1)),
            Item::Scalar(Value::Int(2)),
            Item::Scalar(Value::Int(3)),
        ];
        let tensor = items_to_tensor(&items).unwrap();
        if let Tensor::Array(arr) = tensor {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Tensor::Scalar(1.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_items_to_tensor_float_scalars() {
        let items = vec![
            Item::Scalar(Value::Float(1.5)),
            Item::Scalar(Value::Float(2.5)),
        ];
        let tensor = items_to_tensor(&items).unwrap();
        if let Tensor::Array(arr) = tensor {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Tensor::Scalar(1.5));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_items_to_tensor_invalid() {
        let items = vec![Item::Scalar(Value::String("hello".to_string()))];
        let result = items_to_tensor(&items);
        assert!(result.is_err());
    }

    // ==================== from_xml basic tests ====================

    #[test]
    fn test_empty_document() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><hedl></hedl>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert_eq!(doc.root.len(), 0);
    }

    #[test]
    fn test_empty_document_self_closing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><hedl/>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert_eq!(doc.root.len(), 0);
    }

    #[test]
    fn test_scalar_bool_true() {
        let xml = r#"<?xml version="1.0"?><hedl><val>true</val></hedl>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert_eq!(
            doc.root.get("val").and_then(|i| i.as_scalar()),
            Some(&Value::Bool(true))
        );
    }

    #[test]
    fn test_scalar_bool_false() {
        let xml = r#"<?xml version="1.0"?><hedl><val>false</val></hedl>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert_eq!(
            doc.root.get("val").and_then(|i| i.as_scalar()),
            Some(&Value::Bool(false))
        );
    }

    #[test]
    fn test_scalar_int() {
        let xml = r#"<?xml version="1.0"?><hedl><val>42</val></hedl>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert_eq!(
            doc.root.get("val").and_then(|i| i.as_scalar()),
            Some(&Value::Int(42))
        );
    }

    #[test]
    fn test_scalar_float() {
        let xml = r#"<?xml version="1.0"?><hedl><val>3.5</val></hedl>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        if let Some(Item::Scalar(Value::Float(f))) = doc.root.get("val") {
            assert!((f - 3.5).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_scalar_string() {
        let xml = r#"<?xml version="1.0"?><hedl><val>hello</val></hedl>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert_eq!(
            doc.root.get("val").and_then(|i| i.as_scalar()),
            Some(&Value::String("hello".to_string()))
        );
    }

    #[test]
    fn test_scalar_null_empty_element() {
        let xml = r#"<?xml version="1.0"?><hedl><val></val></hedl>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert_eq!(
            doc.root.get("val").and_then(|i| i.as_scalar()),
            Some(&Value::Null)
        );
    }

    #[test]
    fn test_scalar_expression() {
        let xml = r#"<?xml version="1.0"?><hedl><val>$(foo)</val></hedl>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        if let Some(Item::Scalar(Value::Expression(e))) = doc.root.get("val") {
            assert_eq!(e.to_string(), "foo");
        } else {
            panic!("Expected expression");
        }
    }

    // ==================== Nested object tests ====================

    #[test]
    fn test_nested_object() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <config>
                <name>test</name>
                <value>100</value>
            </config>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        let config_item = doc.root.get("config").unwrap();
        assert!(config_item.as_object().is_some());

        if let Item::Object(obj) = config_item {
            assert!(obj.contains_key("name"));
            assert!(obj.contains_key("value"));
        }
    }

    #[test]
    fn test_deeply_nested_object() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <outer>
                <inner>
                    <deep>42</deep>
                </inner>
            </outer>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Object(outer)) = doc.root.get("outer") {
            if let Some(Item::Object(inner)) = outer.get("inner") {
                if let Some(Item::Scalar(Value::Int(n))) = inner.get("deep") {
                    assert_eq!(*n, 42);
                } else {
                    panic!("Expected int");
                }
            } else {
                panic!("Expected inner object");
            }
        } else {
            panic!("Expected outer object");
        }
    }

    // ==================== List inference tests ====================

    #[test]
    fn test_infer_list_repeated_elements() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <user id="1"><name>Alice</name></user>
            <user id="2"><name>Bob</name></user>
        </hedl>"#;

        let config = FromXmlConfig {
            infer_lists: true,
            ..Default::default()
        };
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::List(list)) = doc.root.get("user") {
            assert_eq!(list.rows.len(), 2);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_no_infer_list_single_element() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <user id="1"><name>Alice</name></user>
        </hedl>"#;

        let config = FromXmlConfig {
            infer_lists: true,
            ..Default::default()
        };
        let doc = from_xml(xml, &config).unwrap();

        // Single element should remain as object
        assert!(doc.root.get("user").and_then(|i| i.as_object()).is_some());
    }

    #[test]
    fn test_infer_list_disabled() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <user id="1"><name>Alice</name></user>
            <user id="2"><name>Bob</name></user>
        </hedl>"#;

        let config = FromXmlConfig {
            infer_lists: false,
            ..Default::default()
        };
        let doc = from_xml(xml, &config).unwrap();

        // With infer_lists disabled, second element overwrites first
        assert!(doc.root.get("user").and_then(|i| i.as_object()).is_some());
    }

    // ==================== Attribute parsing tests ====================

    #[test]
    fn test_attributes_to_object() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <item id="123" name="test" active="true"/>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Object(obj)) = doc.root.get("item") {
            assert_eq!(
                obj.get("id").and_then(|i| i.as_scalar()),
                Some(&Value::Int(123))
            );
            assert_eq!(
                obj.get("name").and_then(|i| i.as_scalar()),
                Some(&Value::String("test".to_string()))
            );
            assert_eq!(
                obj.get("active").and_then(|i| i.as_scalar()),
                Some(&Value::Bool(true))
            );
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_single_value_attribute() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <item value="42"/>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        assert_eq!(
            doc.root.get("item").and_then(|i| i.as_scalar()),
            Some(&Value::Int(42))
        );
    }

    // ==================== Version parsing from root ====================

    #[test]
    fn test_version_from_root_attribute() {
        let xml = r#"<?xml version="1.0"?><hedl version="2.5"></hedl>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert_eq!(doc.version, (2, 5));
    }

    #[test]
    fn test_version_default() {
        let xml = r#"<?xml version="1.0"?><hedl></hedl>"#;
        let config = FromXmlConfig {
            version: (3, 1),
            ..Default::default()
        };
        let doc = from_xml(xml, &config).unwrap();
        assert_eq!(doc.version, (3, 1));
    }

    // ==================== Reference with marker attribute ====================

    #[test]
    fn test_reference_with_marker() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <ref __hedl_type__="ref">@user123</ref>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Scalar(Value::Reference(r))) = doc.root.get("ref") {
            assert_eq!(r.id, "user123");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_qualified_reference_with_marker() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <ref __hedl_type__="ref">@User:456</ref>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Scalar(Value::Reference(r))) = doc.root.get("ref") {
            assert_eq!(r.type_name, Some("User".to_string()));
            assert_eq!(r.id, "456");
        } else {
            panic!("Expected reference");
        }
    }

    // ==================== Error cases ====================

    #[test]
    fn test_empty_input() {
        // Empty input should produce an empty document
        let xml = "";
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert!(doc.root.is_empty());
    }

    #[test]
    fn test_only_declaration() {
        // Only XML declaration should produce an empty document
        let xml = r#"<?xml version="1.0"?>"#;
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert!(doc.root.is_empty());
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_unicode_content() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <hedl>
            <name>héllo 世界</name>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        assert_eq!(
            doc.root.get("name").and_then(|i| i.as_scalar()),
            Some(&Value::String("héllo 世界".to_string()))
        );
    }

    #[test]
    fn test_whitespace_handling() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <val>   hello world   </val>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        // Whitespace should be trimmed
        assert_eq!(
            doc.root.get("val").and_then(|i| i.as_scalar()),
            Some(&Value::String("hello world".to_string()))
        );
    }

    #[test]
    fn test_cdata_content() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <text><![CDATA[<not>xml</not>]]></text>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        // CDATA content should be preserved
        assert!(doc.root.contains_key("text"));
    }

    #[test]
    fn test_key_conversion_from_pascal_case() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <UserName>test</UserName>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        // UserName should be converted to user_name
        assert!(doc.root.contains_key("user_name"));
    }
}
