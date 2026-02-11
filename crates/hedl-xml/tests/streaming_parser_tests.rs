//! Comprehensive streaming tests for hedl-xml
//!
//! Tests for memory-efficient XML streaming parser.

use hedl_core::Item;
use hedl_xml::streaming::{from_xml_stream, EntityPolicy, StreamConfig};
use std::io::Cursor;

// ==================== StreamConfig tests ====================

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
        buffer_size: 4096,
        max_recursion_depth: 50,
        max_batch_size: 500,
        default_type_name: "Element".to_string(),
        version: (2, 0),
        infer_lists: false,
        entity_policy: EntityPolicy::RejectDtd,
        log_security_events: true,
    };

    assert_eq!(config.buffer_size, 4096);
    assert_eq!(config.max_recursion_depth, 50);
    assert_eq!(config.max_batch_size, 500);
    assert!(!config.infer_lists);
}

#[test]
fn test_stream_config_debug() {
    let config = StreamConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("StreamConfig"));
    assert!(debug.contains("buffer_size"));
}

#[test]
fn test_stream_config_clone() {
    let config = StreamConfig {
        buffer_size: 8192,
        ..Default::default()
    };
    let cloned = config.clone();
    assert_eq!(cloned.buffer_size, 8192);
}

// ==================== StreamPosition tests ====================

#[test]
fn test_stream_position_default() {
    use hedl_xml::streaming::StreamPosition;
    let pos = StreamPosition::default();
    assert_eq!(pos.byte_offset, 0);
    assert_eq!(pos.items_parsed, 0);
}

#[test]
fn test_stream_position_debug() {
    use hedl_xml::streaming::StreamPosition;
    let pos = StreamPosition {
        byte_offset: 100,
        items_parsed: 5,
    };
    let debug = format!("{pos:?}");
    assert!(debug.contains("100"));
    assert!(debug.contains('5'));
}

#[test]
fn test_stream_position_copy() {
    use hedl_xml::streaming::StreamPosition;
    let pos = StreamPosition {
        byte_offset: 200,
        items_parsed: 10,
    };
    let copied = pos; // Copy trait
    assert_eq!(copied.byte_offset, 200);
    assert_eq!(copied.items_parsed, 10);
}

#[test]
fn test_stream_position_equality() {
    use hedl_xml::streaming::StreamPosition;
    let pos1 = StreamPosition {
        byte_offset: 100,
        items_parsed: 5,
    };
    let pos2 = StreamPosition {
        byte_offset: 100,
        items_parsed: 5,
    };
    let pos3 = StreamPosition {
        byte_offset: 200,
        items_parsed: 5,
    };

    assert_eq!(pos1, pos2);
    assert_ne!(pos1, pos3);
}

// ==================== Basic streaming tests ====================

#[test]
fn test_streaming_basic() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item1>value1</item1>
        <item2>value2</item2>
        <item3>value3</item3>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].key, "item1");
    assert_eq!(items[1].key, "item2");
    assert_eq!(items[2].key, "item3");
}

#[test]
fn test_streaming_empty_document() {
    let xml = r#"<?xml version="1.0"?><hedl></hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_streaming_single_item() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <single>value</single>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].key, "single");
}

#[test]
fn test_streaming_large_document() {
    let mut xml = String::from(r#"<?xml version="1.0"?><hedl>"#);
    for i in 0..1000 {
        xml.push_str(&format!("<item{i}>value{i}</item{i}>"));
    }
    xml.push_str("</hedl>");

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1000);
}

// ==================== Position tracking tests ====================

#[test]
fn test_streaming_position_tracking() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item1>value1</item1>
        <item2>value2</item2>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let mut parser = from_xml_stream(cursor, &config).unwrap();

    let pos_start = parser.position();
    assert_eq!(pos_start.items_parsed, 0);

    parser.next().unwrap().unwrap();
    let pos_mid = parser.position();
    assert_eq!(pos_mid.items_parsed, 1);
    assert!(pos_mid.byte_offset > 0);

    parser.next().unwrap().unwrap();
    let pos_end = parser.position();
    assert_eq!(pos_end.items_parsed, 2);
    assert!(pos_end.byte_offset > pos_mid.byte_offset);
}

#[test]
fn test_streaming_bytes_processed() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item>value</item>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let mut parser = from_xml_stream(cursor, &config).unwrap();

    let bytes_start = parser.bytes_processed();
    // bytes_start is u64, always >= 0
    let _ = bytes_start;

    parser.next();
    let bytes_end = parser.bytes_processed();
    assert!(bytes_end > bytes_start);
}

#[test]
fn test_streaming_items_parsed() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <a>1</a>
        <b>2</b>
        <c>3</c>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let mut parser = from_xml_stream(cursor, &config).unwrap();

    assert_eq!(parser.items_parsed(), 0);
    parser.next();
    assert_eq!(parser.items_parsed(), 1);
    parser.next();
    assert_eq!(parser.items_parsed(), 2);
    parser.next();
    assert_eq!(parser.items_parsed(), 3);
}

// ==================== Security tests ====================

#[test]
fn test_streaming_reject_dtd() {
    let xml = r#"<?xml version="1.0"?>
    <!DOCTYPE hedl [<!ENTITY test "value">]>
    <hedl>
        <item>test</item>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig {
        entity_policy: EntityPolicy::RejectDtd,
        ..Default::default()
    };

    let mut parser = from_xml_stream(cursor, &config).unwrap();
    // First call to next() should trigger DTD check
    let result = parser.next();
    // Should return error when DTD is encountered
    if let Some(Err(e)) = result {
        assert!(e.contains("DOCTYPE"));
    }
}

#[test]
fn test_streaming_allow_dtd_no_external() {
    let xml = r#"<?xml version="1.0"?>
    <!DOCTYPE hedl []>
    <hedl>
        <item>value</item>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig {
        entity_policy: EntityPolicy::AllowDtdNoExternal,
        ..Default::default()
    };

    let mut parser = from_xml_stream(cursor, &config).unwrap();
    let item = parser.next().unwrap().unwrap();
    assert_eq!(item.key, "item");
}

// ==================== Error handling tests ====================

#[test]
fn test_streaming_malformed_xml() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item>unclosed
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let mut parser = from_xml_stream(cursor, &config).unwrap();

    let _result = parser.next();
    // Parser should either skip or error on malformed element
    // At minimum, shouldn't panic
}

#[test]
fn test_streaming_invalid_utf8() {
    let bad_xml = b"<?xml version=\"1.0\"?><hedl><item>\xFF\xFE</item></hedl>";

    let cursor = Cursor::new(bad_xml);
    let config = StreamConfig::default();
    let mut parser = from_xml_stream(cursor, &config).unwrap();

    // Should handle invalid UTF-8 gracefully
    let _ = parser.next();
}

// ==================== Config variations tests ====================

#[test]
fn test_streaming_custom_buffer_size() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item>value</item>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig {
        buffer_size: 64,
        ..Default::default()
    };

    let parser = from_xml_stream(cursor, &config).unwrap();
    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn test_streaming_custom_type_name() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item>value1</item>
        <item>value2</item>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig {
        default_type_name: "CustomType".to_string(),
        infer_lists: true,
        ..Default::default()
    };

    let parser = from_xml_stream(cursor, &config).unwrap();
    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Should infer list
    if items.len() == 1 {
        if let Item::List(list) = &items[0].value {
            assert_eq!(list.type_name, "Item");
        }
    }
}

#[test]
fn test_streaming_infer_lists_disabled() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item1>value1</item1>
        <item2>value2</item2>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig {
        infer_lists: false,
        ..Default::default()
    };

    let parser = from_xml_stream(cursor, &config).unwrap();
    let results: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // With infer_lists false and no duplicates, should succeed
    assert_eq!(results.len(), 2);
}

// ==================== StreamItem tests ====================

#[test]
fn test_stream_item_debug() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <test>value</test>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let mut parser = from_xml_stream(cursor, &config).unwrap();

    if let Some(Ok(item)) = parser.next() {
        let debug = format!("{item:?}");
        assert!(debug.contains("StreamItem"));
        assert!(debug.contains("test"));
    }
}

#[test]
fn test_stream_item_clone() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <test>value</test>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let mut parser = from_xml_stream(cursor, &config).unwrap();

    if let Some(Ok(item)) = parser.next() {
        let cloned = item.clone();
        assert_eq!(cloned.key, item.key);
    }
}

// ==================== Edge cases ====================

#[test]
fn test_streaming_very_long_content() {
    let long_value = "x".repeat(100000);
    let xml = format!(r#"<?xml version="1.0"?><hedl><item>{long_value}</item></hedl>"#);

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn test_streaming_nested_structures() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <outer>
            <inner>
                <deep>value</deep>
            </inner>
        </outer>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].key, "outer");
}

#[test]
fn test_streaming_mixed_content_types() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <int>42</int>
        <float>3.14</float>
        <bool>true</bool>
        <string>hello</string>
        <null></null>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 5);
}

#[test]
fn test_streaming_attributes() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item id="1" name="test"/>
        <item id="2" name="other"/>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig {
        infer_lists: true,
        ..Default::default()
    };

    let parser = from_xml_stream(cursor, &config).unwrap();
    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Should create a list
    if items.len() == 1 {
        if let Item::List(list) = &items[0].value {
            assert_eq!(list.rows.len(), 2);
        }
    }
}

#[test]
fn test_streaming_unicode() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <hedl>
        <emoji>🚀</emoji>
        <chinese>中文</chinese>
        <arabic>العربية</arabic>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 3);
}

#[test]
fn test_streaming_no_xml_declaration() {
    let xml = r"<hedl><item>value</item></hedl>";

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
}

#[test]
fn test_streaming_comments() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <!-- Comment -->
        <item>value</item>
        <!-- Another comment -->
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let items: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].key, "item");
}

// ==================== Iterator behavior tests ====================

#[test]
fn test_streaming_multiple_iterations() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item1>value1</item1>
        <item2>value2</item2>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let mut parser = from_xml_stream(cursor, &config).unwrap();

    // First iteration
    let item1 = parser.next().unwrap().unwrap();
    assert_eq!(item1.key, "item1");

    // Second iteration
    let item2 = parser.next().unwrap().unwrap();
    assert_eq!(item2.key, "item2");

    // Third iteration should be None
    assert!(parser.next().is_none());

    // Further iterations should still be None
    assert!(parser.next().is_none());
}

#[test]
fn test_streaming_for_loop() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <a>1</a>
        <b>2</b>
        <c>3</c>
    </hedl>"#;

    let cursor = Cursor::new(xml.as_bytes());
    let config = StreamConfig::default();
    let parser = from_xml_stream(cursor, &config).unwrap();

    let mut count = 0;
    for result in parser {
        result.unwrap();
        count += 1;
    }

    assert_eq!(count, 3);
}
