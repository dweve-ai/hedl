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

use hedl_core::{Item, Value};
use hedl_xml::streaming::{from_xml_stream, StreamConfig};
use std::io::Cursor;

#[test]
fn test_streaming_empty_document() {
    let xml = r#"<?xml version="1.0"?><hedl></hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_streaming_single_scalar() {
    let xml = r#"<?xml version="1.0"?><hedl><name>test</name></hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].key, "name");
    assert_eq!(
        items[0].value.as_scalar(),
        Some(&Value::String("test".to_string()))
    );
}

#[test]
fn test_streaming_multiple_scalars() {
    let xml = r#"<?xml version="1.0"?><hedl>
        <id>123</id>
        <name>Alice</name>
        <active>true</active>
    </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 3);

    // Find items by key
    let id_item = items.iter().find(|i| i.key == "id").unwrap();
    let name_item = items.iter().find(|i| i.key == "name").unwrap();
    let active_item = items.iter().find(|i| i.key == "active").unwrap();

    assert_eq!(id_item.value.as_scalar(), Some(&Value::Int(123)));
    assert_eq!(name_item.value.as_scalar(), Some(&Value::String("Alice".to_string())));
    assert_eq!(active_item.value.as_scalar(), Some(&Value::Bool(true)));
}

#[test]
fn test_streaming_nested_object() {
    let xml = r#"<?xml version="1.0"?><hedl>
        <config>
            <name>test</name>
            <value>42</value>
        </config>
    </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].key, "config");

    if let Item::Object(obj) = &items[0].value {
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("value"));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_streaming_repeated_elements() {
    let xml = r#"<?xml version="1.0"?><hedl>
        <user id="1"><name>Alice</name></user>
        <user id="2"><name>Bob</name></user>
        <user id="3"><name>Charlie</name></user>
    </hedl>"#;
    let config = StreamConfig {
        infer_lists: true,
        ..Default::default()
    };
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Streaming parser yields items one by one at root level
    // Repeated elements are not automatically aggregated by the streaming parser
    // This differs from the non-streaming from_xml which accumulates and infers lists
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].key, "user");
    assert_eq!(items[1].key, "user");
    assert_eq!(items[2].key, "user");

    // Each item should be parsed as an object
    for item in items {
        assert!(item.value.as_object().is_some());
    }
}

#[test]
fn test_streaming_mixed_content() {
    let xml = r#"<?xml version="1.0"?><hedl>
        <string>hello</string>
        <number>42</number>
        <bool>true</bool>
        <float>3.14</float>
        <null></null>
    </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 5);
}

#[test]
fn test_streaming_deeply_nested() {
    let xml = r#"<?xml version="1.0"?><hedl>
        <outer>
            <inner>
                <deep>value</deep>
            </inner>
        </outer>
    </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].key, "outer");
}

#[test]
fn test_streaming_custom_buffer_size() {
    let xml = r#"<?xml version="1.0"?><hedl><item>test</item></hedl>"#;
    let config = StreamConfig {
        buffer_size: 32768,
        ..Default::default()
    };
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn test_streaming_custom_recursion_depth() {
    let xml = r#"<?xml version="1.0"?><hedl><a><b><c>deep</c></b></a></hedl>"#;
    let config = StreamConfig {
        max_recursion_depth: 10,
        ..Default::default()
    };
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn test_streaming_attributes_to_object() {
    let xml = r#"<?xml version="1.0"?><hedl>
        <item id="123" name="test" active="true"/>
    </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].key, "item");

    if let Item::Object(obj) = &items[0].value {
        assert_eq!(obj.len(), 3);
        assert!(obj.contains_key("id"));
        assert!(obj.contains_key("name"));
        assert!(obj.contains_key("active"));
    } else {
        panic!("Expected object");
    }
}

#[test]
fn test_streaming_unicode_content() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <hedl>
            <name>héllo 世界</name>
        </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0].value.as_scalar(),
        Some(&Value::String("héllo 世界".to_string()))
    );
}

#[test]
fn test_streaming_whitespace_handling() {
    let xml = r#"<?xml version="1.0"?>
        <hedl>
            <val>   hello world   </val>
        </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0].value.as_scalar(),
        Some(&Value::String("hello world".to_string()))
    );
}

#[test]
fn test_streaming_empty_element() {
    let xml = r#"<?xml version="1.0"?><hedl><empty/></hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].value.as_scalar(), Some(&Value::Null));
}

#[test]
fn test_streaming_numeric_types() {
    let xml = r#"<?xml version="1.0"?><hedl>
        <int>42</int>
        <float>3.14</float>
        <negative>-100</negative>
    </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 3);

    let int_item = items.iter().find(|i| i.key == "int").unwrap();
    let float_item = items.iter().find(|i| i.key == "float").unwrap();
    let negative_item = items.iter().find(|i| i.key == "negative").unwrap();

    assert_eq!(int_item.value.as_scalar(), Some(&Value::Int(42)));
    match float_item.value.as_scalar() {
        Some(Value::Float(f)) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected float"),
    }
    assert_eq!(negative_item.value.as_scalar(), Some(&Value::Int(-100)));
}

#[test]
fn test_streaming_boolean_values() {
    let xml = r#"<?xml version="1.0"?><hedl>
        <true_val>true</true_val>
        <false_val>false</false_val>
    </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 2);

    let true_item = items.iter().find(|i| i.key == "true_val").unwrap();
    let false_item = items.iter().find(|i| i.key == "false_val").unwrap();

    assert_eq!(true_item.value.as_scalar(), Some(&Value::Bool(true)));
    assert_eq!(false_item.value.as_scalar(), Some(&Value::Bool(false)));
}

#[test]
fn test_streaming_large_document_simulation() {
    // Create a large-ish XML document with 100 repeated elements
    let mut xml = String::from(r#"<?xml version="1.0"?><hedl>"#);
    for i in 0..100 {
        xml.push_str(&format!(
            r#"<item id="{}"><name>Item{}</name></item>"#,
            i, i
        ));
    }
    xml.push_str("</hedl>");

    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    // Streaming parser yields items one-by-one, not aggregated
    assert_eq!(items.len(), 100);

    // All items should have the key "item"
    for (idx, item) in items.iter().enumerate() {
        assert_eq!(item.key, "item");
        // Each should have name matching the index
        if let Item::Object(obj) = &item.value {
            if let Some(Item::Scalar(Value::String(name))) = obj.get("name") {
                assert_eq!(name, &format!("Item{}", idx));
            }
        }
    }
}

#[test]
fn test_streaming_default_config_values() {
    let config = StreamConfig::default();
    assert_eq!(config.buffer_size, 65536);
    assert_eq!(config.max_recursion_depth, 100);
    assert_eq!(config.max_batch_size, 1000);
    assert_eq!(config.default_type_name, "Item");
    assert_eq!(config.version, (1, 0));
    assert!(config.infer_lists);
}

#[test]
fn test_streaming_custom_config_values() {
    let config = StreamConfig {
        buffer_size: 131072,
        max_recursion_depth: 50,
        max_batch_size: 500,
        default_type_name: "CustomType".to_string(),
        version: (2, 1),
        infer_lists: false,
    };
    assert_eq!(config.buffer_size, 131072);
    assert_eq!(config.max_recursion_depth, 50);
    assert_eq!(config.max_batch_size, 500);
    assert_eq!(config.default_type_name, "CustomType");
    assert_eq!(config.version, (2, 1));
    assert!(!config.infer_lists);
}

#[test]
fn test_streaming_clone_config() {
    let config = StreamConfig::default();
    let cloned = config.clone();
    assert_eq!(config.buffer_size, cloned.buffer_size);
    assert_eq!(config.max_recursion_depth, cloned.max_recursion_depth);
    assert_eq!(config.max_batch_size, cloned.max_batch_size);
}

#[test]
fn test_streaming_preserves_parse_order() {
    let xml = r#"<?xml version="1.0"?><hedl>
        <first>1</first>
        <second>2</second>
        <third>3</third>
    </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items[0].key, "first");
    assert_eq!(items[1].key, "second");
    assert_eq!(items[2].key, "third");
}

#[test]
fn test_streaming_special_characters_escaping() {
    let xml = r#"<?xml version="1.0"?><hedl>
        <text>hello &amp; goodbye &lt;tag&gt;</text>
    </hedl>"#;
    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0].value.as_scalar(),
        Some(&Value::String("hello & goodbye <tag>".to_string()))
    );
}
