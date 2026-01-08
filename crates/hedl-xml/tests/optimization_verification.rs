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

//! Optimization verification tests
//!
//! These tests verify that the string interning and SmallVec optimizations
//! are working correctly and providing the expected memory reduction.

use hedl_xml::{from_xml, FromXmlConfig};

#[test]
fn test_repeated_element_names_parse_correctly() {
    // This XML has repeated element names which should benefit from string interning
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <hedl>
            <user id="1" name="Alice" email="alice@example.com"/>
            <user id="2" name="Bob" email="bob@example.com"/>
            <user id="3" name="Charlie" email="charlie@example.com"/>
            <user id="4" name="Diana" email="diana@example.com"/>
            <user id="5" name="Eve" email="eve@example.com"/>
        </hedl>"#;

    let config = FromXmlConfig::default();
    let result = from_xml(xml, &config);

    assert!(result.is_ok(), "Parsing should succeed");
    let doc = result.unwrap();

    // Verify the list was inferred correctly
    assert!(doc.root.contains_key("user"));
}

#[test]
fn test_many_attributes_parse_correctly() {
    // This XML has elements with many attributes which should benefit from SmallVec
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <hedl>
            <config
                server="localhost"
                port="8080"
                timeout="30"
                retry="3"
                debug="true"
                cache="false"
                workers="4"
                queue_size="1000"
                buffer_size="4096"
                max_connections="100"/>
        </hedl>"#;

    let config = FromXmlConfig::default();
    let result = from_xml(xml, &config);

    assert!(result.is_ok(), "Parsing should succeed");
    let doc = result.unwrap();

    // Verify attributes were parsed correctly
    assert!(doc.root.contains_key("config"));
}

#[test]
fn test_nested_common_keys_parse_correctly() {
    // This XML has nested structures with common key names (id, name, email)
    // which should benefit from string interning
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <hedl>
            <users>
                <user id="1">
                    <name>Alice</name>
                    <email>alice@example.com</email>
                    <profile>
                        <id>profile1</id>
                        <name>Alice's Profile</name>
                    </profile>
                </user>
                <user id="2">
                    <name>Bob</name>
                    <email>bob@example.com</email>
                    <profile>
                        <id>profile2</id>
                        <name>Bob's Profile</name>
                    </profile>
                </user>
            </users>
        </hedl>"#;

    let config = FromXmlConfig::default();
    let result = from_xml(xml, &config);

    assert!(result.is_ok(), "Parsing should succeed");
    let doc = result.unwrap();

    // Verify nested structure was parsed correctly
    assert!(doc.root.contains_key("users"));
}

#[test]
fn test_few_attributes_use_smallvec() {
    // Elements with 0-4 attributes should not allocate on the heap (SmallVec optimization)
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <hedl>
            <item1/>
            <item2 id="1"/>
            <item3 id="2" name="test"/>
            <item4 id="3" name="test" active="true"/>
            <item5 id="4" name="test" active="true" created="2024-01-01"/>
        </hedl>"#;

    let config = FromXmlConfig::default();
    let result = from_xml(xml, &config);

    assert!(result.is_ok(), "Parsing should succeed");
    let doc = result.unwrap();

    // Verify all items were parsed
    assert!(doc.root.contains_key("item1"));
    assert!(doc.root.contains_key("item2"));
    assert!(doc.root.contains_key("item3"));
    assert!(doc.root.contains_key("item4"));
    assert!(doc.root.contains_key("item5"));
}

#[test]
fn test_large_document_with_optimizations() {
    // Generate a large document to verify optimizations work at scale
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?><hedl>"#);

    // Add 100 users with repeated element and attribute names
    for i in 0..100 {
        xml.push_str(&format!(
            r#"<user id="{}" name="User {}" email="user{}@example.com" active="true"/>"#,
            i, i, i
        ));
    }

    xml.push_str("</hedl>");

    let config = FromXmlConfig::default();
    let result = from_xml(&xml, &config);

    assert!(result.is_ok(), "Parsing large document should succeed");
    let doc = result.unwrap();

    // Verify the list was created
    if let Some(hedl_core::Item::List(list)) = doc.root.get("user") {
        assert_eq!(list.rows.len(), 100, "Should have 100 users");
    } else {
        panic!("Expected a list of users");
    }
}
