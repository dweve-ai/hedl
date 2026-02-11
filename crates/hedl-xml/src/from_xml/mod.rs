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

mod config;
mod conversion;
mod parser;
mod values;

// Re-export public API
pub use config::{EntityPolicy, FromXmlConfig};
pub use parser::from_xml;

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::convert::parse_reference;
    use hedl_core::lex::Tensor;
    use hedl_core::{Item, Value};
    use std::collections::BTreeMap;

    // Helper to access internal functions for testing
    use crate::from_xml::conversion::{
        items_are_list_elements, items_are_tensor_elements, items_to_list, items_to_tensor,
        to_hedl_key,
    };
    use crate::from_xml::values::{parse_value, parse_version};

    // ==================== FromXmlConfig tests ====================

    #[test]
    fn test_from_xml_config_default() {
        let config = FromXmlConfig::default();
        assert_eq!(config.default_type_name, "Item");
        // v2.0 is now the default version for new documents
        assert_eq!(config.version, (2, 0));
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
            entity_policy: EntityPolicy::RejectDtd,
            log_security_events: true,
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
            entity_policy: EntityPolicy::default(),
            log_security_events: false,
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
            Value::String("hello".to_string().into())
        );
    }

    #[test]
    fn test_parse_value_string_with_spaces() {
        assert_eq!(
            parse_value("  hello world  ").unwrap(),
            Value::String("hello world".to_string().into())
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
            assert_eq!(s.as_ref(), "@not-a-ref");
        } else {
            panic!("Expected string");
        }
    }

    // ==================== parse_reference tests ====================

    #[test]
    fn test_parse_reference_local() {
        let ref_val = parse_reference("@user123").unwrap();
        assert_eq!(ref_val.type_name, None);
        assert_eq!(ref_val.id.as_ref(), "user123");
    }

    #[test]
    fn test_parse_reference_qualified() {
        let ref_val = parse_reference("@User:123").unwrap();
        assert_eq!(ref_val.type_name.as_deref(), Some("User"));
        assert_eq!(ref_val.id.as_ref(), "123");
    }

    #[test]
    fn test_parse_reference_with_special_chars() {
        let ref_val = parse_reference("@my-item_123").unwrap();
        assert_eq!(ref_val.type_name, None);
        assert_eq!(ref_val.id.as_ref(), "my-item_123");
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

    // ==================== Issue 3 tests: Namespace and invalid character sanitization ====================

    #[test]
    fn test_issue3_namespace_colon() {
        // Namespaced tags like "x:tag" should normalize to "x_tag"
        assert_eq!(to_hedl_key("x:tag"), "x_tag");
        assert_eq!(to_hedl_key("ns:element"), "ns_element");
        assert_eq!(to_hedl_key("xml:lang"), "xml_lang");
    }

    #[test]
    fn test_issue3_hyphens() {
        // Hyphens should convert to underscores
        assert_eq!(to_hedl_key("my-key"), "my_key");
        assert_eq!(to_hedl_key("multi-word-key"), "multi_word_key");
    }

    #[test]
    fn test_issue3_dots() {
        // Dots should convert to underscores
        assert_eq!(to_hedl_key("key.name"), "key_name");
        assert_eq!(to_hedl_key("config.value"), "config_value");
    }

    #[test]
    fn test_issue3_multiple_special_chars() {
        // Multiple invalid characters
        assert_eq!(to_hedl_key("my:key-name.value"), "my_key_name_value");
        assert_eq!(to_hedl_key("x:some-tag.attr"), "x_some_tag_attr");
    }

    #[test]
    fn test_issue3_leading_digit() {
        // Keys starting with digit get underscore prefix
        assert_eq!(to_hedl_key("123key"), "_123key");
        assert_eq!(to_hedl_key("9item"), "_9item");
    }

    #[test]
    fn test_issue3_empty_or_invalid_only() {
        // Empty or only invalid characters should return "key"
        assert_eq!(to_hedl_key(""), "key");
        assert_eq!(to_hedl_key(":::"), "key");
        assert_eq!(to_hedl_key("---"), "key");
    }

    #[test]
    fn test_issue3_namespace_with_pascal_case() {
        // Combination of namespace and pascal case
        assert_eq!(to_hedl_key("ns:UserName"), "ns_user_name");
        assert_eq!(to_hedl_key("xml:HTTPRequest"), "xml_httprequest");
    }

    #[test]
    fn test_issue3_xml_integration() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <x:tag>value1</x:tag>
            <my-attr>value2</my-attr>
            <config.item>value3</config.item>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        // All keys should be normalized
        assert!(doc.root.contains_key("x_tag"));
        assert!(doc.root.contains_key("my_attr"));
        assert!(doc.root.contains_key("config_item"));
    }

    #[test]
    fn test_issue3_no_collision_different_separators() {
        // Different separators should produce the same normalized key
        // This tests that the normalization is consistent
        assert_eq!(to_hedl_key("my:key"), to_hedl_key("my-key"));
        assert_eq!(to_hedl_key("my.key"), to_hedl_key("my_key"));
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
            Item::Scalar(Value::Tensor(Box::new(Tensor::Scalar(1.0)))),
            Item::Scalar(Value::Tensor(Box::new(Tensor::Scalar(2.0)))),
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
            Item::Scalar(Value::String("hello".to_string().into())),
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
        let items = vec![Item::Scalar(Value::String("hello".to_string().into()))];
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
            Some(&Value::String("hello".to_string().into()))
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
        let result = from_xml(xml, &config);

        // UPDATED for Issue 2 fix: With infer_lists disabled, duplicates now error
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Duplicate element"));
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
                Some(&Value::String("test".to_string().into()))
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
            assert_eq!(r.id.as_ref(), "user123");
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
            assert_eq!(r.type_name.as_deref(), Some("User"));
            assert_eq!(r.id.as_ref(), "456");
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
            Some(&Value::String("héllo 世界".to_string().into()))
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
            Some(&Value::String("hello world".to_string().into()))
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

    // ==================== Issue 1 tests: Attributes preserved ====================

    #[test]
    fn test_issue1_attributes_with_child_elements() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <item id="1"><name>A</name></item>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Object(obj)) = doc.root.get("item") {
            // Should have both id attribute and name child
            assert_eq!(
                obj.get("id").and_then(|i| i.as_scalar()),
                Some(&Value::Int(1))
            );
            assert_eq!(
                obj.get("name").and_then(|i| i.as_scalar()),
                Some(&Value::String("A".to_string().into()))
            );
        } else {
            panic!("Expected object with both id and name");
        }
    }

    #[test]
    fn test_issue1_attributes_with_text_content() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <item id="2" type="primary">Content text</item>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Object(obj)) = doc.root.get("item") {
            // Should have id, type attributes and _text for content
            assert_eq!(
                obj.get("id").and_then(|i| i.as_scalar()),
                Some(&Value::Int(2))
            );
            assert_eq!(
                obj.get("type").and_then(|i| i.as_scalar()),
                Some(&Value::String("primary".to_string().into()))
            );
            assert_eq!(
                obj.get("_text").and_then(|i| i.as_scalar()),
                Some(&Value::String("Content text".to_string().into()))
            );
        } else {
            panic!("Expected object with id, type and _text");
        }
    }

    #[test]
    fn test_issue1_attributes_with_both_children_and_text() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <item id="3" status="active">
                <name>Item 3</name>
                Some text content
            </item>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Object(obj)) = doc.root.get("item") {
            // Should have id, status attributes, name child, and _text
            assert_eq!(
                obj.get("id").and_then(|i| i.as_scalar()),
                Some(&Value::Int(3))
            );
            assert_eq!(
                obj.get("status").and_then(|i| i.as_scalar()),
                Some(&Value::String("active".to_string().into()))
            );
            assert_eq!(
                obj.get("name").and_then(|i| i.as_scalar()),
                Some(&Value::String("Item 3".to_string().into()))
            );
            assert!(obj.contains_key("_text"));
        } else {
            panic!("Expected object with attributes, children and text");
        }
    }

    #[test]
    fn test_issue1_multiple_attributes_preserved() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <product id="100" name="Widget" price="19.99" available="true">
                <description>A useful widget</description>
            </product>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Object(obj)) = doc.root.get("product") {
            assert_eq!(
                obj.get("id").and_then(|i| i.as_scalar()),
                Some(&Value::Int(100))
            );
            assert_eq!(
                obj.get("name").and_then(|i| i.as_scalar()),
                Some(&Value::String("Widget".to_string().into()))
            );
            if let Some(Item::Scalar(Value::Float(f))) = obj.get("price") {
                assert!((f - 19.99).abs() < 0.001);
            } else {
                panic!("Expected price float");
            }
            assert_eq!(
                obj.get("available").and_then(|i| i.as_scalar()),
                Some(&Value::Bool(true))
            );
            assert_eq!(
                obj.get("description").and_then(|i| i.as_scalar()),
                Some(&Value::String("A useful widget".to_string().into()))
            );
        } else {
            panic!("Expected object with all attributes and description");
        }
    }

    // ==================== Issue 2 tests: Duplicate elements handling ====================

    #[test]
    fn test_issue2_duplicate_elements_with_infer_lists_false() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <item>First</item>
            <item>Second</item>
        </hedl>"#;

        let config = FromXmlConfig {
            infer_lists: false,
            ..Default::default()
        };
        let result = from_xml(xml, &config);

        // Should error when duplicates found with infer_lists=false
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Duplicate element"));
        assert!(err.contains("infer_lists=false"));
    }

    #[test]
    fn test_issue2_duplicate_elements_with_infer_lists_true() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <item>First</item>
            <item>Second</item>
            <item>Third</item>
        </hedl>"#;

        let config = FromXmlConfig {
            infer_lists: true,
            ..Default::default()
        };
        let doc = from_xml(xml, &config).unwrap();

        // Should create a list when duplicates found with infer_lists=true
        if let Some(Item::List(list)) = doc.root.get("item") {
            assert_eq!(list.rows.len(), 3);
        } else {
            panic!("Expected list with 3 items");
        }
    }

    #[test]
    fn test_issue2_no_error_for_unique_elements_with_infer_lists_false() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <first>1</first>
            <second>2</second>
            <third>3</third>
        </hedl>"#;

        let config = FromXmlConfig {
            infer_lists: false,
            ..Default::default()
        };
        let doc = from_xml(xml, &config).unwrap();

        // Should succeed when all elements are unique
        assert_eq!(doc.root.len(), 3);
        assert!(doc.root.contains_key("first"));
        assert!(doc.root.contains_key("second"));
        assert!(doc.root.contains_key("third"));
    }

    #[test]
    fn test_issue2_duplicate_nested_elements_with_infer_lists_false() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <parent>
                <child>First</child>
                <child>Second</child>
            </parent>
        </hedl>"#;

        let config = FromXmlConfig {
            infer_lists: false,
            ..Default::default()
        };
        let result = from_xml(xml, &config);

        // Should error for nested duplicates too
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Duplicate element"));
    }

    // ==================== Value::List conversion tests ====================

    #[test]
    fn test_list_string_values_from_xml() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <roles>
                <item>admin</item>
                <item>editor</item>
                <item>viewer</item>
            </roles>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Scalar(Value::List(items))) = doc.root.get("roles") {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s.as_ref() == "admin"));
            assert!(matches!(&items[1], Value::String(s) if s.as_ref() == "editor"));
            assert!(matches!(&items[2], Value::String(s) if s.as_ref() == "viewer"));
        } else {
            panic!("Expected Value::List with string items");
        }
    }

    #[test]
    fn test_list_bool_values_from_xml() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <flags>
                <item>true</item>
                <item>false</item>
                <item>true</item>
            </flags>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Scalar(Value::List(items))) = doc.root.get("flags") {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Bool(true));
            assert_eq!(items[1], Value::Bool(false));
            assert_eq!(items[2], Value::Bool(true));
        } else {
            panic!("Expected Value::List with bool items");
        }
    }

    #[test]
    fn test_list_mixed_values_from_xml() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <mixed>
                <item>text</item>
                <item>42</item>
                <item>true</item>
            </mixed>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        if let Some(Item::Scalar(Value::List(items))) = doc.root.get("mixed") {
            assert_eq!(items.len(), 3);
            assert!(matches!(&items[0], Value::String(s) if s.as_ref() == "text"));
            assert_eq!(items[1], Value::Int(42));
            assert_eq!(items[2], Value::Bool(true));
        } else {
            panic!("Expected Value::List with mixed items");
        }
    }

    #[test]
    fn test_list_empty_from_xml() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <empty></empty>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        // Empty element becomes Null, not an empty list
        assert_eq!(
            doc.root.get("empty").and_then(|i| i.as_scalar()),
            Some(&Value::Null)
        );
    }

    #[test]
    fn test_list_numeric_still_becomes_tensor() {
        let xml = r#"<?xml version="1.0"?>
        <hedl>
            <numbers>
                <item>1</item>
                <item>2</item>
                <item>3</item>
            </numbers>
        </hedl>"#;

        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();

        // The <numbers> element contains <item> children which become a tensor.
        // The structure is: numbers -> Object { "item" -> Tensor }
        let numbers = doc.root.get("numbers").expect("numbers key missing");
        if let Item::Object(obj) = numbers {
            if let Some(Item::Scalar(Value::Tensor(_))) = obj.get("item") {
                // Success - numeric items become a tensor
            } else {
                panic!("Expected Value::Tensor for numeric items under 'item' key");
            }
        } else {
            panic!("Expected Object for 'numbers', got {:?}", numbers);
        }
    }

    #[test]
    fn test_items_are_list_elements() {
        let items = vec![
            Item::Scalar(Value::String("a".to_string().into())),
            Item::Scalar(Value::String("b".to_string().into())),
        ];
        assert!(items_are_list_elements(&items));

        let items = vec![
            Item::Scalar(Value::Bool(true)),
            Item::Scalar(Value::Bool(false)),
        ];
        assert!(items_are_list_elements(&items));

        let items = vec![
            Item::Scalar(Value::String("a".to_string().into())),
            Item::Object(BTreeMap::new()),
        ];
        assert!(!items_are_list_elements(&items));
    }

    #[test]
    fn test_items_to_list() {
        let items = vec![
            Item::Scalar(Value::String("a".to_string().into())),
            Item::Scalar(Value::Bool(true)),
            Item::Scalar(Value::Int(42)),
        ];
        let list = items_to_list(&items).unwrap();
        assert_eq!(list.len(), 3);
        assert!(matches!(&list[0], Value::String(s) if s.as_ref() == "a"));
        assert_eq!(list[1], Value::Bool(true));
        assert_eq!(list[2], Value::Int(42));
    }
}
