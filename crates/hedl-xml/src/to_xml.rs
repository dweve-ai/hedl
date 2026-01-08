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

//! HEDL to XML conversion

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_core::lex::Tensor;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::collections::BTreeMap;
use std::io::Cursor;

/// Configuration for XML output
#[derive(Debug, Clone)]
pub struct ToXmlConfig {
    /// Pretty-print with indentation
    pub pretty: bool,
    /// Indentation string (e.g., "  " or "\t")
    pub indent: String,
    /// Root element name
    pub root_element: String,
    /// Include HEDL metadata as attributes
    pub include_metadata: bool,
    /// Use attributes for scalar values where appropriate
    pub use_attributes: bool,
}

impl Default for ToXmlConfig {
    fn default() -> Self {
        Self {
            pretty: true,
            indent: "  ".to_string(),
            root_element: "hedl".to_string(),
            include_metadata: false,
            use_attributes: false,
        }
    }
}

impl hedl_core::convert::ExportConfig for ToXmlConfig {
    fn include_metadata(&self) -> bool {
        self.include_metadata
    }

    fn pretty(&self) -> bool {
        self.pretty
    }
}

/// Convert HEDL Document to XML string
pub fn to_xml(doc: &Document, config: &ToXmlConfig) -> Result<String, String> {
    let mut writer = if config.pretty {
        // new_with_indent takes (inner, indent_char, indent_size)
        Writer::new_with_indent(Cursor::new(Vec::new()), b' ', config.indent.len())
    } else {
        Writer::new(Cursor::new(Vec::new()))
    };

    // Write XML declaration
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(|e| format!("Failed to write XML declaration: {}", e))?;

    // Write root element
    let mut root = BytesStart::new(&config.root_element);
    if config.include_metadata {
        root.push_attribute((
            "version",
            format!("{}.{}", doc.version.0, doc.version.1).as_str(),
        ));
    }
    writer
        .write_event(Event::Start(root))
        .map_err(|e| format!("Failed to write root element: {}", e))?;

    // Write document content
    write_root(&mut writer, &doc.root, config)?;

    // Close root element
    writer
        .write_event(Event::End(BytesEnd::new(&config.root_element)))
        .map_err(|e| format!("Failed to close root element: {}", e))?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| format!("Invalid UTF-8 in XML output: {}", e))
}

fn write_root<W: std::io::Write>(
    writer: &mut Writer<W>,
    root: &BTreeMap<String, Item>,
    config: &ToXmlConfig,
) -> Result<(), String> {
    for (key, item) in root {
        write_item(writer, key, item, config)?;
    }
    Ok(())
}

fn write_item<W: std::io::Write>(
    writer: &mut Writer<W>,
    key: &str,
    item: &Item,
    config: &ToXmlConfig,
) -> Result<(), String> {
    match item {
        Item::Scalar(value) => write_scalar_element(writer, key, value, config)?,
        Item::Object(obj) => write_object(writer, key, obj, config)?,
        Item::List(list) => write_matrix_list(writer, key, list, config)?,
    }
    Ok(())
}

fn write_scalar_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    key: &str,
    value: &Value,
    config: &ToXmlConfig,
) -> Result<(), String> {
    let mut elem = BytesStart::new(key);

    // Add type marker for references to distinguish from strings starting with @
    if matches!(value, Value::Reference(_)) {
        elem.push_attribute(("__hedl_type__", "ref"));
    }

    // For simple values, we can use attributes if configured
    if config.use_attributes && is_simple_value(value) {
        elem.push_attribute(("value", escape_attribute_value(value).as_str()));
        writer
            .write_event(Event::Empty(elem))
            .map_err(|e| format!("Failed to write empty element: {}", e))?;
    } else {
        writer
            .write_event(Event::Start(elem.clone()))
            .map_err(|e| format!("Failed to write start element: {}", e))?;

        write_value_content(writer, value, config)?;

        writer
            .write_event(Event::End(BytesEnd::new(key)))
            .map_err(|e| format!("Failed to write end element: {}", e))?;
    }

    Ok(())
}

fn write_value_content<W: std::io::Write>(
    writer: &mut Writer<W>,
    value: &Value,
    config: &ToXmlConfig,
) -> Result<(), String> {
    match value {
        Value::Null => {
            // Empty element for null
        }
        Value::Bool(b) => write_text(writer, &b.to_string())?,
        Value::Int(n) => write_text(writer, &n.to_string())?,
        Value::Float(f) => write_text(writer, &f.to_string())?,
        Value::String(s) => write_text(writer, s)?,
        Value::Tensor(t) => write_tensor(writer, t, config)?,
        Value::Reference(r) => write_text(writer, &r.to_ref_string())?,
        Value::Expression(e) => write_text(writer, &format!("$({})", e))?,
    }
    Ok(())
}

fn write_object<W: std::io::Write>(
    writer: &mut Writer<W>,
    key: &str,
    obj: &BTreeMap<String, Item>,
    config: &ToXmlConfig,
) -> Result<(), String> {
    let elem = BytesStart::new(key);
    writer
        .write_event(Event::Start(elem))
        .map_err(|e| format!("Failed to write object start: {}", e))?;

    for (child_key, child_item) in obj {
        write_item(writer, child_key, child_item, config)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new(key)))
        .map_err(|e| format!("Failed to write object end: {}", e))?;

    Ok(())
}

fn write_matrix_list<W: std::io::Write>(
    writer: &mut Writer<W>,
    key: &str,
    list: &MatrixList,
    config: &ToXmlConfig,
) -> Result<(), String> {
    let mut list_elem = BytesStart::new(key);
    if config.include_metadata {
        list_elem.push_attribute(("type", list.type_name.as_str()));
    }

    writer
        .write_event(Event::Start(list_elem))
        .map_err(|e| format!("Failed to write list start: {}", e))?;

    // Write each row as an item element
    let item_name = list.type_name.to_lowercase();
    for row in &list.rows {
        write_node(writer, &item_name, row, &list.schema, config)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new(key)))
        .map_err(|e| format!("Failed to write list end: {}", e))?;

    Ok(())
}

fn write_node<W: std::io::Write>(
    writer: &mut Writer<W>,
    elem_name: &str,
    node: &Node,
    schema: &[String],
    config: &ToXmlConfig,
) -> Result<(), String> {
    let mut elem = BytesStart::new(elem_name);

    // Per SPEC.md: Node.fields contains ALL values including ID (first column)
    // MatrixList.schema includes all column names with ID first

    // Write simple values as attributes if configured
    if config.use_attributes {
        for (i, field) in node.fields.iter().enumerate() {
            if is_simple_value(field) && i < schema.len() {
                let attr_value = escape_attribute_value(field);
                elem.push_attribute((schema[i].as_str(), attr_value.as_str()));
            }
        }
    }

    // Check if we need element content (complex values or children)
    let has_complex_values = node.fields.iter().any(|v| !is_simple_value(v));
    let has_children = !node.children.is_empty();

    if !config.use_attributes || has_complex_values || has_children {
        writer
            .write_event(Event::Start(elem))
            .map_err(|e| format!("Failed to write node start: {}", e))?;

        // Write fields as elements if not using attributes or if complex
        if !config.use_attributes || has_complex_values {
            for (i, field) in node.fields.iter().enumerate() {
                if i < schema.len() {
                    write_scalar_element(writer, &schema[i], field, config)?;
                }
            }
        }

        // Write children with marker attribute so they can be recognized on import
        for (child_type, child_nodes) in &node.children {
            for child in child_nodes {
                // Determine schema for children (would need to be passed down in real implementation)
                let child_schema = vec!["id".to_string()]; // Simplified
                write_child_node(writer, child_type, child, &child_schema, config)?;
            }
        }

        writer
            .write_event(Event::End(BytesEnd::new(elem_name)))
            .map_err(|e| format!("Failed to write node end: {}", e))?;
    } else {
        // Empty element with all attributes
        writer
            .write_event(Event::Empty(elem))
            .map_err(|e| format!("Failed to write empty node: {}", e))?;
    }

    Ok(())
}

/// Write a child node with a marker attribute so it can be recognized as a NEST child on import.
fn write_child_node<W: std::io::Write>(
    writer: &mut Writer<W>,
    elem_name: &str,
    node: &Node,
    schema: &[String],
    config: &ToXmlConfig,
) -> Result<(), String> {
    let mut elem = BytesStart::new(elem_name);

    // Add marker attribute to indicate this is a NEST child
    elem.push_attribute(("__hedl_child__", "true"));

    // Write simple values as attributes if configured
    if config.use_attributes {
        for (i, field) in node.fields.iter().enumerate() {
            if is_simple_value(field) && i < schema.len() {
                let attr_value = escape_attribute_value(field);
                elem.push_attribute((schema[i].as_str(), attr_value.as_str()));
            }
        }
    }

    // Check if we need element content (complex values or children)
    let has_complex_values = node.fields.iter().any(|v| !is_simple_value(v));
    let has_children = !node.children.is_empty();

    if !config.use_attributes || has_complex_values || has_children {
        writer
            .write_event(Event::Start(elem))
            .map_err(|e| format!("Failed to write child node start: {}", e))?;

        // Write fields as elements if not using attributes or if complex
        if !config.use_attributes || has_complex_values {
            for (i, field) in node.fields.iter().enumerate() {
                if i < schema.len() {
                    write_scalar_element(writer, &schema[i], field, config)?;
                }
            }
        }

        // Write nested children recursively
        for (child_type, child_nodes) in &node.children {
            for child in child_nodes {
                let child_schema = vec!["id".to_string()];
                write_child_node(writer, child_type, child, &child_schema, config)?;
            }
        }

        writer
            .write_event(Event::End(BytesEnd::new(elem_name)))
            .map_err(|e| format!("Failed to write child node end: {}", e))?;
    } else {
        // Empty element with all attributes
        writer
            .write_event(Event::Empty(elem))
            .map_err(|e| format!("Failed to write empty child node: {}", e))?;
    }

    Ok(())
}

fn write_tensor<W: std::io::Write>(
    writer: &mut Writer<W>,
    tensor: &Tensor,
    _config: &ToXmlConfig,
) -> Result<(), String> {
    match tensor {
        Tensor::Scalar(n) => write_text(writer, &n.to_string())?,
        Tensor::Array(items) => {
            for item in items {
                let elem = BytesStart::new("item");
                writer
                    .write_event(Event::Start(elem))
                    .map_err(|e| format!("Failed to write tensor item start: {}", e))?;

                write_tensor(writer, item, _config)?;

                writer
                    .write_event(Event::End(BytesEnd::new("item")))
                    .map_err(|e| format!("Failed to write tensor item end: {}", e))?;
            }
        }
    }
    Ok(())
}

fn write_text<W: std::io::Write>(writer: &mut Writer<W>, text: &str) -> Result<(), String> {
    writer
        .write_event(Event::Text(BytesText::new(text)))
        .map_err(|e| format!("Failed to write text: {}", e))
}

fn is_simple_value(value: &Value) -> bool {
    matches!(
        value,
        Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_)
    )
}

fn escape_attribute_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => s.clone(),
        Value::Reference(r) => r.to_ref_string(),
        Value::Expression(e) => format!("$({})", e),
        Value::Tensor(_) => "[tensor]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Document, Reference};
    use hedl_core::lex::{Expression, Span};

    // ==================== ToXmlConfig tests ====================

    #[test]
    fn test_to_xml_config_default() {
        let config = ToXmlConfig::default();
        assert!(config.pretty);
        assert_eq!(config.indent, "  ");
        assert_eq!(config.root_element, "hedl");
        assert!(!config.include_metadata);
        assert!(!config.use_attributes);
    }

    #[test]
    fn test_to_xml_config_debug() {
        let config = ToXmlConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("ToXmlConfig"));
        assert!(debug.contains("pretty"));
        assert!(debug.contains("indent"));
        assert!(debug.contains("root_element"));
    }

    #[test]
    fn test_to_xml_config_clone() {
        let config = ToXmlConfig {
            pretty: false,
            indent: "\t".to_string(),
            root_element: "custom".to_string(),
            include_metadata: true,
            use_attributes: true,
        };
        let cloned = config.clone();
        assert!(!cloned.pretty);
        assert_eq!(cloned.indent, "\t");
        assert_eq!(cloned.root_element, "custom");
        assert!(cloned.include_metadata);
        assert!(cloned.use_attributes);
    }

    #[test]
    fn test_to_xml_config_all_options() {
        let config = ToXmlConfig {
            pretty: true,
            indent: "    ".to_string(),
            root_element: "document".to_string(),
            include_metadata: true,
            use_attributes: true,
        };
        assert!(config.pretty);
        assert_eq!(config.indent.len(), 4);
    }

    // ==================== to_xml basic tests ====================

    #[test]
    fn test_empty_document() {
        let doc = Document::new((1, 0));
        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<hedl"));
        assert!(xml.contains("</hedl>"));
    }

    #[test]
    fn test_empty_document_compact() {
        let doc = Document::new((1, 0));
        let config = ToXmlConfig {
            pretty: false,
            ..Default::default()
        };
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<hedl></hedl>"));
    }

    #[test]
    fn test_custom_root_element() {
        let doc = Document::new((1, 0));
        let config = ToXmlConfig {
            root_element: "custom_root".to_string(),
            ..Default::default()
        };
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<custom_root"));
        assert!(xml.contains("</custom_root>"));
    }

    #[test]
    fn test_with_metadata() {
        let doc = Document::new((2, 5));
        let config = ToXmlConfig {
            include_metadata: true,
            ..Default::default()
        };
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("version=\"2.5\""));
    }

    // ==================== Scalar value tests ====================

    #[test]
    fn test_scalar_null() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("null_val".to_string(), Item::Scalar(Value::Null));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        // Null values produce elements with empty content (may have whitespace in pretty mode)
        assert!(xml.contains("<null_val>") && xml.contains("</null_val>"));
    }

    #[test]
    fn test_scalar_bool_true() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("val".to_string(), Item::Scalar(Value::Bool(true)));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<val>true</val>"));
    }

    #[test]
    fn test_scalar_bool_false() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("val".to_string(), Item::Scalar(Value::Bool(false)));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<val>false</val>"));
    }

    #[test]
    fn test_scalar_int_positive() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("val".to_string(), Item::Scalar(Value::Int(42)));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<val>42</val>"));
    }

    #[test]
    fn test_scalar_int_negative() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("val".to_string(), Item::Scalar(Value::Int(-100)));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<val>-100</val>"));
    }

    #[test]
    fn test_scalar_int_zero() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("val".to_string(), Item::Scalar(Value::Int(0)));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<val>0</val>"));
    }

    #[test]
    fn test_scalar_float() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("val".to_string(), Item::Scalar(Value::Float(3.5)));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<val>3.5</val>"));
    }

    #[test]
    fn test_scalar_string() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "val".to_string(),
            Item::Scalar(Value::String("hello".to_string())),
        );

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<val>hello</val>"));
    }

    #[test]
    fn test_scalar_string_empty() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "val".to_string(),
            Item::Scalar(Value::String("".to_string())),
        );

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<val></val>") || xml.contains("<val/>"));
    }

    // ==================== Reference tests ====================

    #[test]
    fn test_scalar_reference_local() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "ref".to_string(),
            Item::Scalar(Value::Reference(Reference::local("user123"))),
        );

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("@user123"));
        assert!(xml.contains("__hedl_type__=\"ref\""));
    }

    #[test]
    fn test_scalar_reference_qualified() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "ref".to_string(),
            Item::Scalar(Value::Reference(Reference::qualified("User", "456"))),
        );

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("@User:456"));
    }

    // ==================== Expression tests ====================

    #[test]
    fn test_scalar_expression_identifier() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "expr".to_string(),
            Item::Scalar(Value::Expression(Expression::Identifier {
                name: "foo".to_string(),
                span: Span::default(),
            })),
        );

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("$(foo)"));
    }

    #[test]
    fn test_scalar_expression_call() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "expr".to_string(),
            Item::Scalar(Value::Expression(Expression::Call {
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
            })),
        );

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("$(add(x, 1))"));
    }

    // ==================== Tensor tests ====================

    #[test]
    fn test_tensor_1d() {
        let mut doc = Document::new((1, 0));
        let tensor = Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
            Tensor::Scalar(3.0),
        ]);
        doc.root
            .insert("tensor".to_string(), Item::Scalar(Value::Tensor(tensor)));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<tensor>"));
        assert!(xml.contains("<item>1</item>"));
        assert!(xml.contains("<item>2</item>"));
        assert!(xml.contains("<item>3</item>"));
    }

    #[test]
    fn test_tensor_scalar() {
        let mut doc = Document::new((1, 0));
        let tensor = Tensor::Scalar(42.5);
        doc.root
            .insert("tensor".to_string(), Item::Scalar(Value::Tensor(tensor)));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<tensor>42.5</tensor>"));
    }

    // ==================== Object tests ====================

    #[test]
    fn test_nested_object() {
        let mut doc = Document::new((1, 0));
        let mut inner = BTreeMap::new();
        inner.insert(
            "name".to_string(),
            Item::Scalar(Value::String("test".to_string())),
        );
        inner.insert("value".to_string(), Item::Scalar(Value::Int(100)));
        doc.root.insert("config".to_string(), Item::Object(inner));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();

        assert!(xml.contains("<config>"));
        assert!(xml.contains("<name>test</name>"));
        assert!(xml.contains("<value>100</value>"));
        assert!(xml.contains("</config>"));
    }

    #[test]
    fn test_deeply_nested_object() {
        let mut doc = Document::new((1, 0));

        let mut level3 = BTreeMap::new();
        level3.insert("deep".to_string(), Item::Scalar(Value::Int(42)));

        let mut level2 = BTreeMap::new();
        level2.insert("nested".to_string(), Item::Object(level3));

        let mut level1 = BTreeMap::new();
        level1.insert("inner".to_string(), Item::Object(level2));

        doc.root.insert("outer".to_string(), Item::Object(level1));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();

        assert!(xml.contains("<outer>"));
        assert!(xml.contains("<inner>"));
        assert!(xml.contains("<nested>"));
        assert!(xml.contains("<deep>42</deep>"));
    }

    // ==================== List tests ====================

    #[test]
    fn test_matrix_list() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);
        list.add_row(Node::new(
            "User",
            "u1",
            vec![
                Value::String("u1".to_string()),
                Value::String("Alice".to_string()),
            ],
        ));
        doc.root.insert("users".to_string(), Item::List(list));

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();

        assert!(xml.contains("<users>"));
        assert!(xml.contains("<user>"));
        assert!(xml.contains("</users>"));
    }

    #[test]
    fn test_matrix_list_with_metadata() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("User", vec!["id".to_string()]);
        list.add_row(Node::new(
            "User",
            "u1",
            vec![Value::String("u1".to_string())],
        ));
        doc.root.insert("users".to_string(), Item::List(list));

        let config = ToXmlConfig {
            include_metadata: true,
            ..Default::default()
        };
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("type=\"User\""));
    }

    // ==================== Special character tests ====================

    #[test]
    fn test_special_characters_ampersand() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "text".to_string(),
            Item::Scalar(Value::String("hello & goodbye".to_string())),
        );

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        // quick-xml handles escaping automatically
        assert!(xml.contains("<text>"));
    }

    #[test]
    fn test_special_characters_angle_brackets() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "text".to_string(),
            Item::Scalar(Value::String("hello <tag> goodbye".to_string())),
        );

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<text>"));
    }

    #[test]
    fn test_special_characters_quotes() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "text".to_string(),
            Item::Scalar(Value::String("hello \"quoted\"".to_string())),
        );

        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<text>"));
    }

    // ==================== Helper function tests ====================

    #[test]
    fn test_is_simple_value() {
        assert!(is_simple_value(&Value::Null));
        assert!(is_simple_value(&Value::Bool(true)));
        assert!(is_simple_value(&Value::Int(42)));
        assert!(is_simple_value(&Value::Float(3.5)));
        assert!(is_simple_value(&Value::String("hello".to_string())));
        assert!(!is_simple_value(&Value::Reference(Reference::local("x"))));
        assert!(!is_simple_value(&Value::Tensor(Tensor::Scalar(1.0))));
    }

    #[test]
    fn test_escape_attribute_value_null() {
        assert_eq!(escape_attribute_value(&Value::Null), "");
    }

    #[test]
    fn test_escape_attribute_value_bool() {
        assert_eq!(escape_attribute_value(&Value::Bool(true)), "true");
        assert_eq!(escape_attribute_value(&Value::Bool(false)), "false");
    }

    #[test]
    fn test_escape_attribute_value_int() {
        assert_eq!(escape_attribute_value(&Value::Int(42)), "42");
        assert_eq!(escape_attribute_value(&Value::Int(-100)), "-100");
    }

    #[test]
    fn test_escape_attribute_value_float() {
        assert_eq!(escape_attribute_value(&Value::Float(3.5)), "3.5");
    }

    #[test]
    fn test_escape_attribute_value_string() {
        assert_eq!(
            escape_attribute_value(&Value::String("hello".to_string())),
            "hello"
        );
    }

    #[test]
    fn test_escape_attribute_value_reference() {
        let ref_val = Value::Reference(Reference::local("user1"));
        assert_eq!(escape_attribute_value(&ref_val), "@user1");
    }

    #[test]
    fn test_escape_attribute_value_expression() {
        let expr = Value::Expression(Expression::Identifier {
            name: "foo".to_string(),
            span: Span::default(),
        });
        assert_eq!(escape_attribute_value(&expr), "$(foo)");
    }

    #[test]
    fn test_escape_attribute_value_tensor() {
        let tensor = Value::Tensor(Tensor::Scalar(1.0));
        assert_eq!(escape_attribute_value(&tensor), "[tensor]");
    }

    // ==================== Pretty vs compact tests ====================

    #[test]
    fn test_pretty_vs_compact() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("val".to_string(), Item::Scalar(Value::Int(42)));

        let config_pretty = ToXmlConfig {
            pretty: true,
            ..Default::default()
        };
        let config_compact = ToXmlConfig {
            pretty: false,
            ..Default::default()
        };

        let xml_pretty = to_xml(&doc, &config_pretty).unwrap();
        let xml_compact = to_xml(&doc, &config_compact).unwrap();

        assert!(xml_pretty.len() > xml_compact.len());
    }

    // ==================== use_attributes mode tests ====================

    #[test]
    fn test_use_attributes_simple() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("val".to_string(), Item::Scalar(Value::Int(42)));

        let config = ToXmlConfig {
            use_attributes: true,
            ..Default::default()
        };
        let xml = to_xml(&doc, &config).unwrap();
        // Simple values get value attribute in empty element
        assert!(xml.contains("value=\"42\""));
    }
}
