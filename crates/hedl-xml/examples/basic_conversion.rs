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

//! Basic XML conversion example
//!
//! This example demonstrates bidirectional conversion between HEDL and XML formats.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_xml::{hedl_to_xml, xml_to_hedl, ToXmlConfig};
use std::collections::BTreeMap;

fn main() {
    // Create a HEDL document programmatically
    let mut doc = Document::new((1, 0));

    // Add some scalar values
    doc.root.insert(
        "name".to_string(),
        Item::Scalar(Value::String("HEDL XML Example".to_string())),
    );
    doc.root
        .insert("version".to_string(), Item::Scalar(Value::Int(1)));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(true)));

    // Add a nested object
    let mut config = BTreeMap::new();
    config.insert("debug".to_string(), Item::Scalar(Value::Bool(true)));
    config.insert("timeout".to_string(), Item::Scalar(Value::Int(30)));
    doc.root.insert("config".to_string(), Item::Object(config));

    // Add a matrix list (users table)
    let mut users = MatrixList::new(
        "User",
        vec!["id".to_string(), "name".to_string(), "email".to_string()],
    );

    users.add_row(Node::new(
        "User",
        "user1",
        vec![
            Value::String("Alice".to_string()),
            Value::String("alice@example.com".to_string()),
        ],
    ));

    users.add_row(Node::new(
        "User",
        "user2",
        vec![
            Value::String("Bob".to_string()),
            Value::String("bob@example.com".to_string()),
        ],
    ));

    doc.root.insert("users".to_string(), Item::List(users));

    // Convert to XML
    println!("=== HEDL to XML ===\n");
    let xml = hedl_to_xml(&doc).expect("Failed to convert to XML");
    println!("{}\n", xml);

    // Convert back to HEDL
    println!("=== XML to HEDL (round trip) ===\n");
    let doc2 = xml_to_hedl(&xml).expect("Failed to convert from XML");

    // Verify the round-trip
    assert_eq!(
        doc2.root.get("name").and_then(|i| i.as_scalar()),
        Some(&Value::String("HEDL XML Example".to_string()))
    );
    assert_eq!(
        doc2.root.get("version").and_then(|i| i.as_scalar()),
        Some(&Value::Int(1))
    );
    assert_eq!(
        doc2.root.get("active").and_then(|i| i.as_scalar()),
        Some(&Value::Bool(true))
    );

    println!("Round-trip successful!");

    // Demonstrate configuration options
    println!("\n=== Compact XML (no pretty print) ===\n");
    let config_compact = ToXmlConfig {
        pretty: false,
        ..Default::default()
    };
    let xml_compact =
        hedl_xml::to_xml(&doc, &config_compact).expect("Failed to convert to compact XML");
    println!("{}\n", xml_compact);

    // With metadata
    println!("=== XML with metadata ===\n");
    let config_meta = ToXmlConfig {
        include_metadata: true,
        ..Default::default()
    };
    let xml_meta =
        hedl_xml::to_xml(&doc, &config_meta).expect("Failed to convert to XML with metadata");
    println!("{}\n", xml_meta);

    // Custom root element
    println!("=== Custom root element ===\n");
    let config_custom = ToXmlConfig {
        root_element: "document".to_string(),
        ..Default::default()
    };
    let xml_custom =
        hedl_xml::to_xml(&doc, &config_custom).expect("Failed to convert to XML with custom root");
    println!("{}\n", xml_custom);

    println!("All examples completed successfully!");
}
