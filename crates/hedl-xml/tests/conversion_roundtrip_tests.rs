//! Comprehensive conversion tests for hedl-xml
//!
//! Tests cover:
//! - Error conversions
//! - Edge cases in XML parsing
//! - Config variations
//! - Security scenarios
//! - Streaming edge cases

use hedl_core::{Document, Item, Value};
use hedl_xml::{from_xml, to_xml, EntityPolicy, FromXmlConfig, ToXmlConfig};
use std::collections::BTreeMap;

// Error tests are already covered in the error.rs module tests

// ==================== FromXmlConfig tests ====================

#[test]
fn test_from_xml_config_strict_security() {
    let config = FromXmlConfig::strict_security();
    assert_eq!(config.entity_policy, EntityPolicy::RejectDtd);
    assert!(config.log_security_events);
}

#[test]
fn test_from_xml_config_import_trait() {
    use hedl_core::convert::ImportConfig;

    let config = FromXmlConfig {
        default_type_name: "CustomType".to_string(),
        version: (3, 2),
        ..Default::default()
    };

    assert_eq!(config.default_type_name(), "CustomType");
    assert_eq!(config.version(), (3, 2));
}

#[test]
fn test_from_xml_config_entity_policy_variants() {
    let config = FromXmlConfig {
        entity_policy: EntityPolicy::RejectDtd,
        ..Default::default()
    };
    assert_eq!(config.entity_policy, EntityPolicy::RejectDtd);

    let config = FromXmlConfig {
        entity_policy: EntityPolicy::AllowDtdNoExternal,
        ..Default::default()
    };
    assert_eq!(config.entity_policy, EntityPolicy::AllowDtdNoExternal);

    let config = FromXmlConfig {
        entity_policy: EntityPolicy::WarnOnEntities,
        ..Default::default()
    };
    assert_eq!(config.entity_policy, EntityPolicy::WarnOnEntities);
}

// ==================== ToXmlConfig tests ====================

#[test]
fn test_to_xml_config_export_trait() {
    use hedl_core::convert::ExportConfig;

    let config = ToXmlConfig {
        include_metadata: true,
        pretty: false,
        ..Default::default()
    };

    assert!(config.include_metadata());
    assert!(!config.pretty());
}

#[test]
fn test_to_xml_config_variations() {
    let config = ToXmlConfig {
        pretty: true,
        indent: "\t".to_string(),
        root_element: "data".to_string(),
        include_metadata: false,
        use_attributes: true,
    };

    assert!(config.pretty);
    assert_eq!(config.indent, "\t");
    assert_eq!(config.root_element, "data");
    assert!(!config.include_metadata);
    assert!(config.use_attributes);
}

// ==================== Roundtrip conversion tests ====================

#[test]
fn test_roundtrip_all_scalar_types() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("null_val".to_string(), Item::Scalar(Value::Null));
    doc.root
        .insert("bool_val".to_string(), Item::Scalar(Value::Bool(false)));
    doc.root
        .insert("int_val".to_string(), Item::Scalar(Value::Int(-42)));
    doc.root
        .insert("float_val".to_string(), Item::Scalar(Value::Float(-3.15)));
    doc.root.insert(
        "string_val".to_string(),
        Item::Scalar(Value::String("test".to_string().into())),
    );

    let xml = to_xml(&doc, &ToXmlConfig::default()).unwrap();
    let doc2 = from_xml(&xml, &FromXmlConfig::default()).unwrap();

    assert!(doc2.root.contains_key("null_val"));
    assert_eq!(
        doc2.root.get("bool_val").and_then(|i| i.as_scalar()),
        Some(&Value::Bool(false))
    );
    assert_eq!(
        doc2.root.get("int_val").and_then(|i| i.as_scalar()),
        Some(&Value::Int(-42))
    );
    assert_eq!(
        doc2.root.get("string_val").and_then(|i| i.as_scalar()),
        Some(&Value::String("test".to_string().into()))
    );
}

#[test]
fn test_roundtrip_nested_objects() {
    let mut doc = Document::new((2, 0));
    let mut level2 = BTreeMap::new();
    level2.insert("deep".to_string(), Item::Scalar(Value::Int(99)));
    let mut level1 = BTreeMap::new();
    level1.insert("inner".to_string(), Item::Object(level2));
    doc.root.insert("outer".to_string(), Item::Object(level1));

    let xml = to_xml(&doc, &ToXmlConfig::default()).unwrap();
    let doc2 = from_xml(&xml, &FromXmlConfig::default()).unwrap();

    if let Some(Item::Object(outer)) = doc2.root.get("outer") {
        if let Some(Item::Object(inner)) = outer.get("inner") {
            assert_eq!(
                inner.get("deep").and_then(|i| i.as_scalar()),
                Some(&Value::Int(99))
            );
        } else {
            panic!("Expected inner object");
        }
    } else {
        panic!("Expected outer object");
    }
}

// ==================== XML parsing edge cases ====================

#[test]
fn test_xml_with_comments() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <!-- This is a comment -->
        <value>42</value>
        <!-- Another comment -->
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();
    assert_eq!(
        doc.root.get("value").and_then(|i| i.as_scalar()),
        Some(&Value::Int(42))
    );
}

#[test]
fn test_xml_with_processing_instructions() {
    let xml = r#"<?xml version="1.0"?>
    <?xml-stylesheet type="text/xsl" href="style.xsl"?>
    <hedl>
        <value>test</value>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();
    assert_eq!(
        doc.root.get("value").and_then(|i| i.as_scalar()),
        Some(&Value::String("test".to_string().into()))
    );
}

#[test]
fn test_xml_with_mixed_empty_elements() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <empty1/>
        <empty2></empty2>
        <with_value value="42"/>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    assert!(doc.root.contains_key("empty1"));
    assert!(doc.root.contains_key("empty2"));
    assert_eq!(
        doc.root.get("with_value").and_then(|i| i.as_scalar()),
        Some(&Value::Int(42))
    );
}

#[test]
fn test_xml_with_numeric_element_names() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <_123>value1</_123>
        <_9item>value2</_9item>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    assert!(doc.root.contains_key("_123"));
    assert!(doc.root.contains_key("_9item"));
}

#[test]
fn test_xml_large_numbers() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <max_i64>9223372036854775807</max_i64>
        <min_i64>-9223372036854775808</min_i64>
        <large_float>1.7976931348623157e308</large_float>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    assert_eq!(
        doc.root.get("max_i64").and_then(|i| i.as_scalar()),
        Some(&Value::Int(9223372036854775807))
    );
    assert_eq!(
        doc.root.get("min_i64").and_then(|i| i.as_scalar()),
        Some(&Value::Int(-9223372036854775808))
    );
}

#[test]
fn test_xml_special_float_values() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <val1>0.0</val1>
        <val2>-0.0</val2>
        <val3>1e10</val3>
        <val4>1e-10</val4>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    assert!(doc.root.contains_key("val1"));
    assert!(doc.root.contains_key("val2"));
    assert!(doc.root.contains_key("val3"));
    assert!(doc.root.contains_key("val4"));
}

// ==================== Security tests ====================

#[test]
fn test_entity_policy_reject_dtd_with_entity() {
    let xml = r#"<?xml version="1.0"?>
    <!DOCTYPE hedl [<!ENTITY test "value">]>
    <hedl><data>&test;</data></hedl>"#;

    let config = FromXmlConfig {
        entity_policy: EntityPolicy::RejectDtd,
        ..Default::default()
    };
    let result = from_xml(xml, &config);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("DOCTYPE"));
}

#[test]
fn test_entity_policy_allow_dtd_no_external() {
    let xml = r#"<?xml version="1.0"?>
    <!DOCTYPE hedl [<!ENTITY internal "value">]>
    <hedl><data>test</data></hedl>"#;

    let config = FromXmlConfig {
        entity_policy: EntityPolicy::AllowDtdNoExternal,
        log_security_events: false,
        ..Default::default()
    };
    // Should parse successfully (entities won't be expanded)
    let doc = from_xml(xml, &config).unwrap();
    assert!(doc.root.contains_key("data"));
}

#[test]
fn test_suspicious_entity_references_in_values() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <data>safe content</data>
    </hedl>"#;

    let config = FromXmlConfig {
        entity_policy: EntityPolicy::WarnOnEntities,
        log_security_events: true,
        ..Default::default()
    };
    // Should parse normally
    let doc = from_xml(xml, &config).unwrap();
    assert!(doc.root.contains_key("data"));
}

// ==================== XML generation edge cases ====================

#[test]
fn test_to_xml_empty_string_values() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "empty".to_string(),
        Item::Scalar(Value::String(String::new().into())),
    );
    doc.root.insert(
        "whitespace".to_string(),
        Item::Scalar(Value::String("   ".to_string().into())),
    );

    let config = ToXmlConfig::default();
    let xml = to_xml(&doc, &config).unwrap();

    assert!(xml.contains("<empty>"));
    assert!(xml.contains("<whitespace>"));
}

#[test]
fn test_to_xml_with_custom_indent() {
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("val".to_string(), Item::Scalar(Value::Int(42)));

    let config = ToXmlConfig {
        pretty: true,
        indent: "\t\t".to_string(),
        ..Default::default()
    };
    let xml = to_xml(&doc, &config).unwrap();

    // Should contain tabs for indentation
    assert!(xml.contains("<?xml"));
    assert!(xml.contains("<val>42</val>"));
}

#[test]
fn test_to_xml_very_long_element_name() {
    let mut doc = Document::new((2, 0));
    let long_name = "a".repeat(100);
    doc.root
        .insert(long_name.clone(), Item::Scalar(Value::Int(1)));

    let config = ToXmlConfig::default();
    let xml = to_xml(&doc, &config).unwrap();

    assert!(xml.contains(&format!("<{long_name}>")));
    assert!(xml.contains(&format!("</{long_name}>")));
}

// ==================== Version handling tests ====================

#[test]
fn test_version_parsing_edge_cases() {
    // Version from XML
    let xml = r#"<?xml version="1.0"?><hedl version="10.20"></hedl>"#;
    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();
    assert_eq!(doc.version, (10, 20));

    // Version with patch (only major.minor used)
    let xml = r#"<?xml version="1.0"?><hedl version="2.0.1"></hedl>"#;
    let doc = from_xml(xml, &config).unwrap();
    assert_eq!(doc.version, (2, 0));

    // Invalid version falls back to config
    let xml = r#"<?xml version="1.0"?><hedl version="invalid"></hedl>"#;
    let config = FromXmlConfig {
        version: (5, 0),
        ..Default::default()
    };
    let doc = from_xml(xml, &config).unwrap();
    assert_eq!(doc.version, (5, 0));
}

// ==================== Mixed content tests ====================

#[test]
fn test_mixed_text_and_children() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item>
            Text before
            <child>child value</child>
            Text after
        </item>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("item") {
        assert!(obj.contains_key("child"));
        assert!(obj.contains_key("_text"));
    } else {
        panic!("Expected object for mixed content");
    }
}

#[test]
fn test_attributes_only_empty_element() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item id="1" name="test"/>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    if let Some(Item::Object(obj)) = doc.root.get("item") {
        assert_eq!(
            obj.get("id").and_then(|i| i.as_scalar()),
            Some(&Value::Int(1))
        );
        assert_eq!(
            obj.get("name").and_then(|i| i.as_scalar()),
            Some(&Value::String("test".to_string().into()))
        );
    } else {
        panic!("Expected object");
    }
}

// ==================== Utility function tests ====================

#[test]
fn test_hedl_to_xml_convenience() {
    let doc = Document::new((2, 0));
    let xml = hedl_xml::hedl_to_xml(&doc).unwrap();
    assert!(xml.contains("<?xml"));
    assert!(xml.contains("<hedl"));
}

#[test]
fn test_xml_to_hedl_convenience() {
    let xml = r#"<?xml version="1.0"?><hedl><val>42</val></hedl>"#;
    let doc = hedl_xml::xml_to_hedl(xml).unwrap();
    assert_eq!(
        doc.root.get("val").and_then(|i| i.as_scalar()),
        Some(&Value::Int(42))
    );
}

// ==================== Unicode and encoding tests ====================

#[test]
fn test_unicode_in_all_positions() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <hedl>
        <emoji>🚀</emoji>
        <chinese>中文</chinese>
        <arabic>العربية</arabic>
        <mixed>Hello世界🌍</mixed>
    </hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();

    assert!(doc.root.contains_key("emoji"));
    assert!(doc.root.contains_key("chinese"));
    assert!(doc.root.contains_key("arabic"));
    assert!(doc.root.contains_key("mixed"));
}

#[test]
fn test_unicode_roundtrip() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "emoji".to_string(),
        Item::Scalar(Value::String("🚀🌟💻".to_string().into())),
    );
    doc.root.insert(
        "kanji".to_string(),
        Item::Scalar(Value::String("日本語".to_string().into())),
    );

    let xml = to_xml(&doc, &ToXmlConfig::default()).unwrap();
    let doc2 = from_xml(&xml, &FromXmlConfig::default()).unwrap();

    assert_eq!(
        doc2.root.get("emoji").and_then(|i| i.as_scalar()),
        Some(&Value::String("🚀🌟💻".to_string().into()))
    );
    assert_eq!(
        doc2.root.get("kanji").and_then(|i| i.as_scalar()),
        Some(&Value::String("日本語".to_string().into()))
    );
}

// ==================== Error handling tests ====================

#[test]
fn test_malformed_xml_unclosed_tag() {
    let xml = r#"<?xml version="1.0"?><hedl><item>value</hedl>"#;
    let config = FromXmlConfig::default();
    let result = from_xml(xml, &config);
    assert!(result.is_err());
}

#[test]
fn test_malformed_xml_invalid_character() {
    let xml = "<?xml version=\"1.0\"?><hedl><item>\x00</item></hedl>";
    let config = FromXmlConfig::default();
    let result = from_xml(xml, &config);
    // May or may not error depending on XML parser tolerance
    // Just ensure it doesn't panic
    let _ = result;
}

#[test]
fn test_xml_without_declaration() {
    let xml = r"<hedl><value>42</value></hedl>";
    let config = FromXmlConfig::default();
    let doc = from_xml(xml, &config).unwrap();
    assert_eq!(
        doc.root.get("value").and_then(|i| i.as_scalar()),
        Some(&Value::Int(42))
    );
}
