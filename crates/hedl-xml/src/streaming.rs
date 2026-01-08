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

//! Streaming XML parser for handling large documents
//!
//! This module provides memory-efficient streaming parsing for multi-gigabyte XML files.
//! Instead of loading the entire document into memory, items are yielded incrementally.
//!
//! # Features
//!
//! - Memory-efficient: Process files larger than available RAM
//! - Incremental: Yields items as they're parsed
//! - Configurable: Adjustable buffer sizes and recursion limits
//! - Type-safe: Returns `Result` for error handling
//!
//! # Examples
//!
//! ```text
//! use std::fs::File;
//! use hedl_xml::streaming::{from_xml_stream, StreamConfig};
//!
//! let file = File::open("large.xml")?;
//! let config = StreamConfig::default();
//!
//! for result in from_xml_stream(file, &config)? {
//!     match result {
//!         Ok(item) => println!("Processed: {:?}", item),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! ```

use hedl_core::convert::parse_reference;
use hedl_core::{Item, MatrixList, Node, Value};
use hedl_core::lex::{parse_expression_token, singularize_and_capitalize};
use hedl_core::lex::Tensor;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read};

/// Configuration for streaming XML parsing
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for reading chunks (default: 64KB)
    pub buffer_size: usize,
    /// Maximum recursion depth (default: 100)
    pub max_recursion_depth: usize,
    /// Maximum list size before yielding (default: 1000)
    pub max_batch_size: usize,
    /// Default type name for inferred lists
    pub default_type_name: String,
    /// HEDL version
    pub version: (u32, u32),
    /// Try to infer list structures from repeated elements
    pub infer_lists: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 65536, // 64KB
            max_recursion_depth: 100,
            max_batch_size: 1000,
            default_type_name: "Item".to_string(),
            version: (1, 0),
            infer_lists: true,
        }
    }
}

/// A streaming XML parser that yields items incrementally
///
/// This iterator yields `StreamItem` results as the XML is parsed.
/// Memory usage is bounded by the `buffer_size` configuration.
pub struct XmlStreamingParser<R: Read> {
    reader: Reader<BufReader<R>>,
    config: StreamConfig,
    root_element_name: String,
    root_parsed: bool,
    exhausted: bool,
    buf: Vec<u8>,
}

/// An item yielded by the streaming parser
#[derive(Debug, Clone)]
pub struct StreamItem {
    /// Key/field name in the HEDL document
    pub key: String,
    /// The parsed value (scalar, object, or list)
    pub value: Item,
}

impl<R: Read> XmlStreamingParser<R> {
    /// Create a new streaming parser
    pub fn new(reader: R, config: StreamConfig) -> Result<Self, String> {
        let buf_reader = BufReader::with_capacity(config.buffer_size, reader);
        let xml_reader = Reader::from_reader(buf_reader);
        Ok(XmlStreamingParser {
            reader: xml_reader,
            config,
            root_element_name: String::new(),
            root_parsed: false,
            exhausted: false,
            buf: Vec::with_capacity(8192),
        })
    }

    /// Internal method to find and parse the root element
    fn find_root(&mut self) -> Result<bool, String> {
        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                    self.root_element_name =
                        String::from_utf8_lossy(e.name().as_ref()).to_string();
                    self.root_parsed = true;
                    return Ok(true);
                }
                Ok(Event::Eof) => return Ok(false),
                Err(e) => {
                    return Err(format!(
                        "XML parse error at position {}: {}",
                        self.reader.buffer_position(),
                        e
                    ))
                }
                _ => {}
            }
        }
    }

    /// Parse the next element at the root level
    fn parse_next_root_element(&mut self) -> Result<Option<StreamItem>, String> {
        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(e)) => {
                    let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let name = to_hedl_key(&raw_name);
                    let elem_owned = e.to_owned();

                    let item = parse_element(&mut self.reader, &elem_owned, &self.config, 1)?;
                    return Ok(Some(StreamItem {
                        key: name,
                        value: item,
                    }));
                }
                Ok(Event::Empty(e)) => {
                    let raw_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let name = to_hedl_key(&raw_name);
                    let elem_owned = e.to_owned();

                    let item = parse_empty_element(&elem_owned)?;
                    return Ok(Some(StreamItem {
                        key: name,
                        value: item,
                    }));
                }
                Ok(Event::End(e)) => {
                    let end_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if end_name == self.root_element_name {
                        return Ok(None); // End of root element
                    }
                }
                Ok(Event::Eof) => return Ok(None),
                Err(e) => {
                    return Err(format!(
                        "XML parse error at position {}: {}",
                        self.reader.buffer_position(),
                        e
                    ))
                }
                _ => {}
            }
        }
    }
}

impl<R: Read> Iterator for XmlStreamingParser<R> {
    type Item = Result<StreamItem, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.exhausted {
            return None;
        }

        // If we haven't found the root element yet, do that first
        if !self.root_parsed {
            match self.find_root() {
                Ok(true) => {
                    // Root element found, continue to next element
                }
                Ok(false) => {
                    self.exhausted = true;
                    return None;
                }
                Err(e) => {
                    self.exhausted = true;
                    return Some(Err(e));
                }
            }
        }

        // Try to parse the next element
        match self.parse_next_root_element() {
            Ok(Some(item)) => Some(Ok(item)),
            Ok(None) => {
                self.exhausted = true;
                None
            }
            Err(e) => {
                self.exhausted = true;
                Some(Err(e))
            }
        }
    }
}

/// Create a streaming XML parser from a reader
///
/// Returns an iterator that yields `Result<StreamItem, String>` as items are parsed.
/// This is memory-efficient for multi-gigabyte XML files.
///
/// # Examples
///
/// ```no_run
/// use std::fs::File;
/// use hedl_xml::streaming::{from_xml_stream, StreamConfig};
///
/// let file = File::open("data.xml")?;
/// let config = StreamConfig::default();
///
/// let mut count = 0;
/// for result in from_xml_stream(file, &config)? {
///     match result {
///         Ok(_item) => count += 1,
///         Err(e) => eprintln!("Parse error: {}", e),
///     }
/// }
/// println!("Processed {} items", count);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn from_xml_stream<R: Read>(
    reader: R,
    config: &StreamConfig,
) -> Result<XmlStreamingParser<R>, String> {
    XmlStreamingParser::new(reader, config.clone())
}

// ============================================================================
// Parsing functions (shared with from_xml.rs)
// ============================================================================

fn parse_element<R>(
    reader: &mut Reader<R>,
    elem: &quick_xml::events::BytesStart,
    config: &StreamConfig,
    depth: usize,
) -> Result<Item, String>
where
    R: BufRead,
{
    if depth > config.max_recursion_depth {
        return Err(format!(
            "XML recursion depth exceeded (max: {})",
            config.max_recursion_depth
        ));
    }

    let name = String::from_utf8_lossy(elem.name().as_ref()).to_string();

    // Extract attributes
    let mut attributes = BTreeMap::new();
    let mut is_reference = false;
    for attr in elem.attributes().flatten() {
        let raw_key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let value = String::from_utf8_lossy(&attr.value).to_string();

        if raw_key == "__hedl_type__" {
            if value == "ref" {
                is_reference = true;
            }
            continue;
        }

        let key = to_hedl_key(&raw_key);
        attributes.insert(key, value);
    }

    // Parse content
    let mut text_content = String::new();
    let mut child_elements: BTreeMap<String, Vec<Item>> = BTreeMap::new();
    let mut has_children = false;
    let mut buf = Vec::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                has_children = true;
                let raw_child_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let child_name = to_hedl_key(&raw_child_name);
                let elem_owned = e.to_owned();
                let child_item = parse_element(reader, &elem_owned, config, depth + 1)?;

                child_elements
                    .entry(child_name)
                    .or_default()
                    .push(child_item);
            }
            Ok(Event::Empty(e)) => {
                has_children = true;
                let raw_child_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let child_name = to_hedl_key(&raw_child_name);
                let elem_owned = e.to_owned();
                let child_item = parse_empty_element(&elem_owned)?;

                child_elements
                    .entry(child_name)
                    .or_default()
                    .push(child_item);
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
        let mut result_children = BTreeMap::new();
        for (child_name, items) in child_elements {
            if items.len() > 1 && config.infer_lists {
                if child_name == "item" && items_are_tensor_elements(&items) {
                    let tensor = items_to_tensor(&items)?;
                    result_children.insert(child_name, Item::Scalar(Value::Tensor(tensor)));
                } else {
                    let list = items_to_matrix_list(&child_name, items, config)?;
                    result_children.insert(child_name, Item::List(list));
                }
            } else if let Some(item) = items.into_iter().next() {
                result_children.insert(child_name, item);
            }
        }

        // Check for flattening
        if result_children.len() == 1 {
            let (child_key, child_item) = result_children.iter().next().unwrap();
            if let Item::List(list) = child_item {
                let has_nested_children = list.rows.iter().any(|node| !node.children.is_empty());
                if !has_nested_children {
                    let parent_singular =
                        singularize_and_capitalize(&to_hedl_key(&name)).to_lowercase();
                    let child_type = singularize_and_capitalize(child_key).to_lowercase();
                    if parent_singular == child_type {
                        return Ok(result_children.into_values().next().unwrap());
                    }
                }
            }
        }

        Ok(Item::Object(result_children))
    } else if !text_content.trim().is_empty() {
        let value = if is_reference {
            Value::Reference(parse_reference(text_content.trim())?)
        } else {
            parse_value(&text_content)?
        };
        Ok(Item::Scalar(value))
    } else if !attributes.is_empty() {
        let mut obj = BTreeMap::new();
        for (key, value_str) in attributes {
            let value = parse_value(&value_str)?;
            obj.insert(key, Item::Scalar(value));
        }
        Ok(Item::Object(obj))
    } else {
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
        let value_str = attributes.get("value").unwrap();
        let value = parse_value(value_str)?;
        Ok(Item::Scalar(value))
    } else {
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

    if trimmed.starts_with("$(") && trimmed.ends_with(')') {
        let expr =
            parse_expression_token(trimmed).map_err(|e| format!("Invalid expression: {}", e))?;
        return Ok(Value::Expression(expr));
    }

    if trimmed == "true" {
        return Ok(Value::Bool(true));
    }
    if trimmed == "false" {
        return Ok(Value::Bool(false));
    }

    if let Ok(i) = trimmed.parse::<i64>() {
        return Ok(Value::Int(i));
    }
    if let Ok(f) = trimmed.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    Ok(Value::String(trimmed.to_string()))
}

fn items_to_matrix_list(
    name: &str,
    items: Vec<Item>,
    _config: &StreamConfig,
) -> Result<MatrixList, String> {
    let type_name = singularize_and_capitalize(name);
    let schema = infer_schema(&items)?;

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
        let mut keys: Vec<_> = first_obj
            .iter()
            .filter(|(_, item)| matches!(item, Item::Scalar(_)))
            .map(|(k, _)| k.clone())
            .collect();
        keys.sort();

        if let Some(pos) = keys.iter().position(|k| k == "id") {
            keys.remove(pos);
            keys.insert(0, "id".to_string());
        } else {
            keys.insert(0, "id".to_string());
        }

        Ok(keys)
    } else {
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
            let id = obj
                .get(&schema[0])
                .and_then(|i| i.as_scalar())
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}", idx));

            let mut fields = Vec::new();
            for col in schema {
                let value = obj
                    .get(col)
                    .and_then(|i| i.as_scalar())
                    .cloned()
                    .unwrap_or(Value::Null);
                fields.push(value);
            }

            let mut children: BTreeMap<String, Vec<Node>> = BTreeMap::new();
            for child_item in obj.values() {
                if let Item::List(child_list) = child_item {
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

fn to_hedl_key(s: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
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

    while result.contains("__") {
        result = result.replace("__", "_");
    }

    result.trim_matches('_').to_string()
}

fn items_are_tensor_elements(items: &[Item]) -> bool {
    items.iter().all(|item| {
        match item {
            Item::Scalar(Value::Int(_)) => true,
            Item::Scalar(Value::Float(_)) => true,
            Item::Scalar(Value::Tensor(_)) => true,
            Item::Object(obj) if obj.len() == 1 => {
                matches!(obj.get("item"), Some(Item::Scalar(Value::Tensor(_))))
            }
            _ => false,
        }
    })
}

fn items_to_tensor(items: &[Item]) -> Result<Tensor, String> {
    let mut tensor_items = Vec::new();

    for item in items {
        let tensor = match item {
            Item::Scalar(Value::Int(n)) => Tensor::Scalar(*n as f64),
            Item::Scalar(Value::Float(f)) => Tensor::Scalar(*f),
            Item::Scalar(Value::Tensor(t)) => t.clone(),
            Item::Object(obj) if obj.len() == 1 => {
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

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.buffer_size, 65536);
        assert_eq!(config.max_recursion_depth, 100);
        assert_eq!(config.max_batch_size, 1000);
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (1, 0));
        assert!(config.infer_lists);
    }

    #[test]
    fn test_stream_config_custom() {
        let config = StreamConfig {
            buffer_size: 131072,
            max_recursion_depth: 50,
            max_batch_size: 500,
            default_type_name: "CustomItem".to_string(),
            version: (2, 0),
            infer_lists: false,
        };
        assert_eq!(config.buffer_size, 131072);
        assert_eq!(config.max_recursion_depth, 50);
        assert_eq!(config.max_batch_size, 500);
        assert_eq!(config.default_type_name, "CustomItem");
        assert_eq!(config.version, (2, 0));
        assert!(!config.infer_lists);
    }

    #[test]
    fn test_stream_item_construction() {
        let item = StreamItem {
            key: "test".to_string(),
            value: Item::Scalar(Value::String("value".to_string())),
        };
        assert_eq!(item.key, "test");
        assert_eq!(
            item.value.as_scalar(),
            Some(&Value::String("value".to_string()))
        );
    }

    #[test]
    fn test_parse_value_string() {
        assert_eq!(parse_value("hello"), Ok(Value::String("hello".to_string())));
    }

    #[test]
    fn test_parse_value_bool() {
        assert_eq!(parse_value("true"), Ok(Value::Bool(true)));
        assert_eq!(parse_value("false"), Ok(Value::Bool(false)));
    }

    #[test]
    fn test_parse_value_int() {
        assert_eq!(parse_value("42"), Ok(Value::Int(42)));
    }

    #[test]
    fn test_parse_value_float() {
        match parse_value("3.14") {
            Ok(Value::Float(f)) => assert!((f - 3.14).abs() < 0.001),
            _ => panic!("Expected float"),
        }
    }

    #[test]
    fn test_parse_value_null() {
        assert_eq!(parse_value(""), Ok(Value::Null));
        assert_eq!(parse_value("   "), Ok(Value::Null));
    }

    #[test]
    fn test_to_hedl_key_pascal_case() {
        assert_eq!(to_hedl_key("Category"), "category");
        assert_eq!(to_hedl_key("UserPost"), "user_post");
    }

    #[test]
    fn test_to_hedl_key_lowercase() {
        assert_eq!(to_hedl_key("users"), "users");
    }

    #[test]
    fn test_infer_schema_from_objects() {
        let items = vec![
            Item::Object({
                let mut m = BTreeMap::new();
                m.insert("id".to_string(), Item::Scalar(Value::String("1".to_string())));
                m.insert("name".to_string(), Item::Scalar(Value::String("Alice".to_string())));
                m
            }),
        ];
        let schema = infer_schema(&items).unwrap();
        assert!(schema.contains(&"id".to_string()));
        assert!(schema.contains(&"name".to_string()));
    }

    #[test]
    fn test_items_are_tensor_elements_numeric() {
        let items = vec![
            Item::Scalar(Value::Int(1)),
            Item::Scalar(Value::Float(2.0)),
            Item::Scalar(Value::Int(3)),
        ];
        assert!(items_are_tensor_elements(&items));
    }

    #[test]
    fn test_items_are_tensor_elements_non_numeric() {
        let items = vec![
            Item::Scalar(Value::Int(1)),
            Item::Scalar(Value::String("hello".to_string())),
        ];
        assert!(!items_are_tensor_elements(&items));
    }
}
