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

//! HEDL XML Conversion
//!
//! Provides bidirectional conversion between HEDL documents and XML format.
//!
//! # Features
//!
//! - Convert HEDL documents to well-formed XML
//! - Parse XML into HEDL documents with type inference
//! - **Streaming support** for large multi-gigabyte XML files
//! - **Async I/O** with Tokio (via `async` feature flag)
//! - **XSD schema validation** with comprehensive error messages
//! - **Schema caching** for high-performance validation
//! - Configurable output formatting (pretty print, attributes)
//! - Support for nested structures and matrix lists
//! - Reference and expression preservation
//!
//! # Examples
//!
//! ## Converting HEDL to XML
//!
//! ```rust
//! use hedl_core::{Document, Item, Value};
//! use hedl_xml::{to_xml, ToXmlConfig};
//! use std::collections::BTreeMap;
//!
//! let mut doc = Document::new((1, 0));
//! doc.root.insert("name".to_string(), Item::Scalar(Value::String("example".to_string())));
//!
//! let config = ToXmlConfig::default();
//! let xml = to_xml(&doc, &config).unwrap();
//! ```
//!
//! ## Converting XML to HEDL
//!
//! ```rust
//! use hedl_xml::{from_xml, FromXmlConfig};
//!
//! let xml = r#"<?xml version="1.0"?><hedl><name>example</name></hedl>"#;
//! let config = FromXmlConfig::default();
//! let doc = from_xml(xml, &config).unwrap();
//! ```
//!
//! ## Streaming large XML files
//!
//! For multi-gigabyte XML files, use the streaming API to process items incrementally
//! without loading the entire document into memory:
//!
//! ```rust,no_run
//! use hedl_xml::streaming::{from_xml_stream, StreamConfig};
//! use std::fs::File;
//!
//! let file = File::open("large.xml")?;
//! let config = StreamConfig::default();
//!
//! for result in from_xml_stream(file, &config)? {
//!     match result {
//!         Ok(item) => println!("Processing: {}", item.key),
//!         Err(e) => eprintln!("Error: {}", e),
//!     }
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## XSD Schema Validation
//!
//! Validate XML documents against XSD schemas:
//!
//! ```rust
//! use hedl_xml::schema::SchemaValidator;
//!
//! let schema = r#"<?xml version="1.0"?>
//! <xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
//!   <xs:element name="person">
//!     <xs:complexType>
//!       <xs:sequence>
//!         <xs:element name="name" type="xs:string"/>
//!         <xs:element name="age" type="xs:integer"/>
//!       </xs:sequence>
//!     </xs:complexType>
//!   </xs:element>
//! </xs:schema>"#;
//!
//! let validator = SchemaValidator::from_xsd(schema)?;
//!
//! let xml = r#"<?xml version="1.0"?>
//! <person>
//!   <name>Alice</name>
//!   <age>30</age>
//! </person>"#;
//!
//! validator.validate(xml)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Async I/O (with `async` feature)
//!
//! Enable async support in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! hedl-xml = { version = "*", features = ["async"] }
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! Then use async functions:
//!
//! ```rust,no_run
//! # #[cfg(feature = "async")]
//! # {
//! use hedl_xml::async_api::{from_xml_file_async, to_xml_file_async};
//! use hedl_xml::{FromXmlConfig, ToXmlConfig};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Read XML asynchronously
//! let doc = from_xml_file_async("input.xml", &FromXmlConfig::default()).await?;
//!
//! // Process document...
//!
//! // Write XML asynchronously
//! to_xml_file_async(&doc, "output.xml", &ToXmlConfig::default()).await?;
//! # Ok(())
//! # }
//! # }
//! ```

mod from_xml;
mod to_xml;
pub mod streaming;
pub mod schema;

#[cfg(feature = "async")]
pub mod async_api;

pub use from_xml::{from_xml, FromXmlConfig};
pub use to_xml::{to_xml, ToXmlConfig};
pub use streaming::{from_xml_stream, StreamConfig, StreamItem, XmlStreamingParser};
pub use schema::{SchemaValidator, SchemaCache, ValidationError};

use hedl_core::Document;

/// Convert HEDL document to XML string with default configuration
pub fn hedl_to_xml(doc: &Document) -> Result<String, String> {
    to_xml(doc, &ToXmlConfig::default())
}

/// Convert XML string to HEDL document with default configuration
pub fn xml_to_hedl(xml: &str) -> Result<Document, String> {
    from_xml(xml, &FromXmlConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
    use std::collections::BTreeMap;

    #[test]
    fn test_round_trip_scalars() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("null_val".to_string(), Item::Scalar(Value::Null));
        doc.root
            .insert("bool_val".to_string(), Item::Scalar(Value::Bool(true)));
        doc.root
            .insert("int_val".to_string(), Item::Scalar(Value::Int(42)));
        doc.root
            .insert("float_val".to_string(), Item::Scalar(Value::Float(3.25)));
        doc.root.insert(
            "string_val".to_string(),
            Item::Scalar(Value::String("hello".to_string())),
        );

        let xml = hedl_to_xml(&doc).unwrap();
        let doc2 = xml_to_hedl(&xml).unwrap();

        assert_eq!(
            doc2.root.get("bool_val").and_then(|i| i.as_scalar()),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            doc2.root.get("int_val").and_then(|i| i.as_scalar()),
            Some(&Value::Int(42))
        );
        assert_eq!(
            doc2.root.get("string_val").and_then(|i| i.as_scalar()),
            Some(&Value::String("hello".to_string()))
        );
    }

    #[test]
    fn test_round_trip_object() {
        let mut doc = Document::new((1, 0));
        let mut inner = BTreeMap::new();
        inner.insert(
            "name".to_string(),
            Item::Scalar(Value::String("test".to_string())),
        );
        inner.insert("value".to_string(), Item::Scalar(Value::Int(100)));
        doc.root.insert("config".to_string(), Item::Object(inner));

        let xml = hedl_to_xml(&doc).unwrap();
        let doc2 = xml_to_hedl(&xml).unwrap();

        let config_obj = doc2.root.get("config").and_then(|i| i.as_object()).unwrap();
        assert_eq!(
            config_obj.get("name").and_then(|i| i.as_scalar()),
            Some(&Value::String("test".to_string()))
        );
        assert_eq!(
            config_obj.get("value").and_then(|i| i.as_scalar()),
            Some(&Value::Int(100))
        );
    }

    #[test]
    fn test_round_trip_reference() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "ref1".to_string(),
            Item::Scalar(Value::Reference(Reference::local("user123"))),
        );
        doc.root.insert(
            "ref2".to_string(),
            Item::Scalar(Value::Reference(Reference::qualified("User", "456"))),
        );

        let xml = hedl_to_xml(&doc).unwrap();
        let doc2 = xml_to_hedl(&xml).unwrap();

        assert_eq!(
            doc2.root.get("ref1").and_then(|i| i.as_scalar()),
            Some(&Value::Reference(Reference::local("user123")))
        );
        assert_eq!(
            doc2.root.get("ref2").and_then(|i| i.as_scalar()),
            Some(&Value::Reference(Reference::qualified("User", "456")))
        );
    }

    #[test]
    fn test_round_trip_expression() {
        use hedl_core::lex::{ExprLiteral, Expression, Span};

        let mut doc = Document::new((1, 0));
        let expr = Expression::Call {
            name: "add".to_string(),
            args: vec![
                Expression::Identifier {
                    name: "x".to_string(),
                    span: Span::default(),
                },
                Expression::Literal {
                    value: ExprLiteral::Int(1),
                    span: Span::default(),
                },
            ],
            span: Span::default(),
        };
        doc.root.insert(
            "expr".to_string(),
            Item::Scalar(Value::Expression(expr.clone())),
        );

        let xml = hedl_to_xml(&doc).unwrap();
        let doc2 = xml_to_hedl(&xml).unwrap();

        assert_eq!(
            doc2.root.get("expr").and_then(|i| i.as_scalar()),
            Some(&Value::Expression(expr))
        );
    }

    #[test]
    fn test_matrix_list() {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("User", vec!["id".to_string(), "name".to_string()]);

        let node1 = Node::new(
            "User",
            "user1",
            vec![
                Value::String("user1".to_string()),
                Value::String("Alice".to_string()),
            ],
        );
        let node2 = Node::new(
            "User",
            "user2",
            vec![
                Value::String("user2".to_string()),
                Value::String("Bob".to_string()),
            ],
        );

        list.add_row(node1);
        list.add_row(node2);

        doc.root.insert("users".to_string(), Item::List(list));

        let xml = hedl_to_xml(&doc).unwrap();
        assert!(xml.contains("<users"));
        assert!(xml.contains("user1"));
        assert!(xml.contains("user2"));
    }

    #[test]
    fn test_special_characters_escaping() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "text".to_string(),
            Item::Scalar(Value::String(
                "hello & goodbye <tag> \"quoted\"".to_string(),
            )),
        );

        let xml = hedl_to_xml(&doc).unwrap();
        let doc2 = xml_to_hedl(&xml).unwrap();

        // XML escaping should be handled transparently
        let original = doc.root.get("text").and_then(|i| i.as_scalar());
        let parsed = doc2.root.get("text").and_then(|i| i.as_scalar());

        assert_eq!(original, parsed);
    }

    #[test]
    fn test_nested_objects() {
        let mut doc = Document::new((1, 0));

        let mut level2 = BTreeMap::new();
        level2.insert(
            "deep".to_string(),
            Item::Scalar(Value::String("value".to_string())),
        );

        let mut level1 = BTreeMap::new();
        level1.insert("nested".to_string(), Item::Object(level2));

        doc.root.insert("outer".to_string(), Item::Object(level1));

        let xml = hedl_to_xml(&doc).unwrap();
        let doc2 = xml_to_hedl(&xml).unwrap();

        assert!(doc2.root.contains_key("outer"));
    }

    #[test]
    fn test_config_pretty_print() {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "test".to_string(),
            Item::Scalar(Value::String("value".to_string())),
        );

        let config_pretty = ToXmlConfig {
            pretty: true,
            indent: "  ".to_string(),
            ..Default::default()
        };

        let config_compact = ToXmlConfig {
            pretty: false,
            ..Default::default()
        };

        let xml_pretty = to_xml(&doc, &config_pretty).unwrap();
        let xml_compact = to_xml(&doc, &config_compact).unwrap();

        // Pretty printed should have newlines and indentation
        assert!(xml_pretty.len() > xml_compact.len());
    }

    #[test]
    fn test_config_custom_root() {
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
    fn test_config_metadata() {
        let doc = Document::new((2, 1));

        let config = ToXmlConfig {
            include_metadata: true,
            ..Default::default()
        };

        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("version=\"2.1\""));
    }

    #[test]
    fn test_empty_values() {
        let mut doc = Document::new((1, 0));
        doc.root
            .insert("empty".to_string(), Item::Scalar(Value::Null));

        let xml = hedl_to_xml(&doc).unwrap();
        let doc2 = xml_to_hedl(&xml).unwrap();

        assert!(doc2.root.contains_key("empty"));
    }

    #[test]
    fn test_tensor_values() {
        use hedl_core::lex::Tensor;

        let mut doc = Document::new((1, 0));
        let tensor = Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
            Tensor::Scalar(3.0),
        ]);
        doc.root
            .insert("tensor".to_string(), Item::Scalar(Value::Tensor(tensor)));

        let xml = hedl_to_xml(&doc).unwrap();
        assert!(xml.contains("<tensor>"));
        assert!(xml.contains("<item>"));
    }

    #[test]
    fn test_infer_lists_config() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <hedl>
            <user id="1"><name>Alice</name></user>
            <user id="2"><name>Bob</name></user>
        </hedl>"#;

        let config = FromXmlConfig {
            infer_lists: true,
            ..Default::default()
        };

        let doc = from_xml(xml, &config).unwrap();

        // Should infer a list from repeated <user> elements
        assert!(doc.root.contains_key("user"));
        if let Some(Item::List(list)) = doc.root.get("user") {
            assert_eq!(list.rows.len(), 2);
        }
    }

    #[test]
    fn test_attributes_as_values() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <hedl>
            <item id="123" name="test" active="true"/>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        assert!(doc.root.contains_key("item"));
        if let Some(Item::Object(obj)) = doc.root.get("item") {
            // "123" is inferred as an integer (type inference is correct)
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
        }
    }
}
