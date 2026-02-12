// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tests for YAML to HEDL conversion

#[cfg(test)]
mod yaml_conversion_tests {

    use crate::from_yaml::conversion::yaml_value_to_item;
    use crate::from_yaml::detection::{
        is_object_sequence, is_scalar_list_sequence, is_tensor_sequence,
    };
    use crate::from_yaml::matrix_list::yaml_sequence_to_matrix_list;
    use crate::from_yaml::tensor::yaml_sequence_to_tensor;
    use crate::from_yaml::value_conversion::yaml_to_value;
    use crate::{
        from_yaml, from_yaml_value, FromYamlConfig, FromYamlConfigBuilder,
        DEFAULT_MAX_ARRAY_LENGTH, DEFAULT_MAX_DOCUMENT_SIZE, DEFAULT_MAX_NESTING_DEPTH,
    };
    use hedl_core::convert::parse_reference;
    use hedl_core::lex::Tensor;
    use hedl_core::{Item, Value};
    use serde_yaml::{Mapping, Value as YamlValue};
    use std::collections::BTreeMap;

    // ==================== FromYamlConfig tests ====================

    #[test]
    fn test_from_yaml_config_default() {
        let config = FromYamlConfig::default();
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (2, 0));
    }

    #[test]
    fn test_from_yaml_config_debug() {
        let config = FromYamlConfig::default();
        let debug = format!("{config:?}");
        assert!(debug.contains("FromYamlConfig"));
        assert!(debug.contains("default_type_name"));
        assert!(debug.contains("version"));
    }

    #[test]
    fn test_from_yaml_config_clone() {
        let config = FromYamlConfig {
            default_type_name: "Custom".to_string(),
            version: (2, 1),
            ..Default::default()
        };
        let cloned = config.clone();
        assert_eq!(cloned.default_type_name, "Custom");
        assert_eq!(cloned.version, (2, 1));
    }

    #[test]
    fn test_from_yaml_config_custom() {
        let config = FromYamlConfig {
            default_type_name: "MyType".to_string(),
            version: (3, 0),
            ..Default::default()
        };
        assert_eq!(config.default_type_name, "MyType");
        assert_eq!(config.version, (3, 0));
    }

    // ==================== parse_reference tests ====================

    #[test]
    fn test_parse_reference_local() {
        let local_ref = parse_reference("@user1").unwrap();
        assert_eq!(local_ref.type_name, None);
        assert_eq!(local_ref.id.as_ref(), "user1");
    }

    #[test]
    fn test_parse_reference_qualified() {
        let qual_ref = parse_reference("@User:user1").unwrap();
        assert_eq!(qual_ref.type_name.as_deref(), Some("User"));
        assert_eq!(qual_ref.id.as_ref(), "user1");
    }

    #[test]
    fn test_parse_reference_invalid_no_at() {
        let result = parse_reference("user1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid reference format"));
    }

    #[test]
    fn test_parse_reference_with_special_chars() {
        let ref_val = parse_reference("@my-item_123").unwrap();
        assert_eq!(ref_val.type_name, None);
        assert_eq!(ref_val.id.as_ref(), "my-item_123");
    }

    #[test]
    fn test_parse_reference_qualified_with_dashes() {
        let ref_val = parse_reference("@My-Type:item-123").unwrap();
        assert_eq!(ref_val.type_name.as_deref(), Some("My-Type"));
        assert_eq!(ref_val.id.as_ref(), "item-123");
    }

    #[test]
    fn test_parse_reference_empty_id() {
        // @: is parsed as type "" and id ""
        let ref_val = parse_reference("@:").unwrap();
        assert_eq!(ref_val.type_name.as_deref(), Some(""));
        assert_eq!(ref_val.id.as_ref(), "");
    }

    // ==================== is_tensor_sequence tests ====================

    #[test]
    fn test_is_tensor_sequence_numbers() {
        let numbers = vec![
            YamlValue::Number(1.into()),
            YamlValue::Number(2.into()),
            YamlValue::Number(3.into()),
        ];
        assert!(is_tensor_sequence(&numbers));
    }

    #[test]
    fn test_is_tensor_sequence_nested() {
        let nested = vec![
            YamlValue::Sequence(vec![YamlValue::Number(1.into())]),
            YamlValue::Sequence(vec![YamlValue::Number(2.into())]),
        ];
        assert!(is_tensor_sequence(&nested));
    }

    #[test]
    fn test_is_tensor_sequence_mixed_numbers_and_nested() {
        let mixed = vec![
            YamlValue::Number(1.into()),
            YamlValue::Sequence(vec![YamlValue::Number(2.into())]),
        ];
        assert!(is_tensor_sequence(&mixed));
    }

    #[test]
    fn test_is_tensor_sequence_with_strings() {
        let mixed = vec![
            YamlValue::Number(1.into()),
            YamlValue::String("test".to_string()),
        ];
        assert!(!is_tensor_sequence(&mixed));
    }

    #[test]
    fn test_is_tensor_sequence_empty() {
        let empty: Vec<YamlValue> = vec![];
        assert!(!is_tensor_sequence(&empty));
    }

    #[test]
    fn test_is_tensor_sequence_all_strings() {
        let strings = vec![
            YamlValue::String("a".to_string()),
            YamlValue::String("b".to_string()),
        ];
        assert!(!is_tensor_sequence(&strings));
    }

    #[test]
    fn test_is_tensor_sequence_with_mappings() {
        let with_mapping = vec![
            YamlValue::Number(1.into()),
            YamlValue::Mapping(Mapping::new()),
        ];
        assert!(!is_tensor_sequence(&with_mapping));
    }

    // ==================== is_object_sequence tests ====================

    #[test]
    fn test_is_object_sequence_mappings() {
        let objects = vec![
            YamlValue::Mapping(Mapping::new()),
            YamlValue::Mapping(Mapping::new()),
        ];
        assert!(is_object_sequence(&objects));
    }

    #[test]
    fn test_is_object_sequence_mixed() {
        let mixed = vec![
            YamlValue::Mapping(Mapping::new()),
            YamlValue::Number(1.into()),
        ];
        assert!(!is_object_sequence(&mixed));
    }

    #[test]
    fn test_is_object_sequence_empty() {
        let empty: Vec<YamlValue> = vec![];
        assert!(!is_object_sequence(&empty));
    }

    #[test]
    fn test_is_object_sequence_all_numbers() {
        let numbers = vec![YamlValue::Number(1.into()), YamlValue::Number(2.into())];
        assert!(!is_object_sequence(&numbers));
    }

    #[test]
    fn test_is_object_sequence_with_nested_sequences() {
        let mixed = vec![
            YamlValue::Mapping(Mapping::new()),
            YamlValue::Sequence(vec![]),
        ];
        assert!(!is_object_sequence(&mixed));
    }

    // ==================== yaml_value_to_item tests ====================

    #[test]
    fn test_yaml_value_to_item_null() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(&YamlValue::Null, "test", &config, &mut structs, 0).unwrap();
        assert_eq!(item, Item::Scalar(Value::Null));
    }

    #[test]
    fn test_yaml_value_to_item_bool_true() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item =
            yaml_value_to_item(&YamlValue::Bool(true), "test", &config, &mut structs, 0).unwrap();
        assert_eq!(item, Item::Scalar(Value::Bool(true)));
    }

    #[test]
    fn test_yaml_value_to_item_bool_false() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item =
            yaml_value_to_item(&YamlValue::Bool(false), "test", &config, &mut structs, 0).unwrap();
        assert_eq!(item, Item::Scalar(Value::Bool(false)));
    }

    #[test]
    fn test_yaml_value_to_item_int() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::Number(42.into()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        assert_eq!(item, Item::Scalar(Value::Int(42)));
    }

    #[test]
    fn test_yaml_value_to_item_int_negative() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::Number((-100).into()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        assert_eq!(item, Item::Scalar(Value::Int(-100)));
    }

    #[test]
    fn test_yaml_value_to_item_float() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let yaml_num = YamlValue::Number(serde_yaml::Number::from(3.5));
        let item = yaml_value_to_item(&yaml_num, "test", &config, &mut structs, 0).unwrap();
        if let Item::Scalar(Value::Float(f)) = item {
            assert!((f - 3.5).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_yaml_value_to_item_string() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::String("hello".to_string()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        assert_eq!(
            item,
            Item::Scalar(Value::String("hello".to_string().into()))
        );
    }

    #[test]
    fn test_yaml_value_to_item_string_empty() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::String(String::new()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        assert_eq!(item, Item::Scalar(Value::String(String::new().into())));
    }

    #[test]
    fn test_yaml_value_to_item_string_with_at() {
        // Strings starting with @ are just strings, not references
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::String("@not-a-ref".to_string()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        if let Item::Scalar(Value::String(s)) = item {
            assert_eq!(s.as_ref(), "@not-a-ref");
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_yaml_value_to_item_expression() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::String("$(add(x, 1))".to_string()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        if let Item::Scalar(Value::Expression(e)) = item {
            assert_eq!(e.to_string(), "add(x, 1)");
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_yaml_value_to_item_expression_identifier() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let item = yaml_value_to_item(
            &YamlValue::String("$(foo)".to_string()),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        if let Item::Scalar(Value::Expression(e)) = item {
            assert_eq!(e.to_string(), "foo");
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_yaml_value_to_item_reference_local() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut ref_map = Mapping::new();
        ref_map.insert(
            YamlValue::String("@ref".to_string()),
            YamlValue::String("@user1".to_string()),
        );
        let item = yaml_value_to_item(
            &YamlValue::Mapping(ref_map),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        if let Item::Scalar(Value::Reference(r)) = item {
            assert_eq!(r.type_name, None);
            assert_eq!(r.id.as_ref(), "user1");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_yaml_value_to_item_reference_qualified() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut ref_map = Mapping::new();
        ref_map.insert(
            YamlValue::String("@ref".to_string()),
            YamlValue::String("@User:user1".to_string()),
        );
        let item = yaml_value_to_item(
            &YamlValue::Mapping(ref_map),
            "test",
            &config,
            &mut structs,
            0,
        )
        .unwrap();
        if let Item::Scalar(Value::Reference(r)) = item {
            assert_eq!(r.type_name.as_deref(), Some("User"));
            assert_eq!(r.id.as_ref(), "user1");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_yaml_value_to_item_tensor_1d() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let seq = YamlValue::Sequence(vec![
            YamlValue::Number(1.into()),
            YamlValue::Number(2.into()),
            YamlValue::Number(3.into()),
        ]);
        let item = yaml_value_to_item(&seq, "test", &config, &mut structs, 0).unwrap();
        if let Item::Scalar(Value::Tensor(tensor_box)) = item {
            if let Tensor::Array(arr) = *tensor_box {
                assert_eq!(arr.len(), 3);
            } else {
                panic!("Expected tensor array");
            }
        } else {
            panic!("Expected tensor");
        }
    }

    #[test]
    fn test_yaml_value_to_item_empty_sequence() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();
        let seq = YamlValue::Sequence(vec![]);
        let item = yaml_value_to_item(&seq, "items", &config, &mut structs, 0).unwrap();
        // Empty sequences become empty matrix lists
        if let Item::List(list) = item {
            assert!(list.rows.is_empty());
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_yaml_value_to_item_object_sequence() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut obj1 = Mapping::new();
        obj1.insert(
            YamlValue::String("id".to_string()),
            YamlValue::String("u1".to_string()),
        );
        obj1.insert(
            YamlValue::String("name".to_string()),
            YamlValue::String("Alice".to_string()),
        );

        let seq = YamlValue::Sequence(vec![YamlValue::Mapping(obj1)]);
        let item = yaml_value_to_item(&seq, "users", &config, &mut structs, 0).unwrap();
        if let Item::List(list) = item {
            assert_eq!(list.rows.len(), 1);
            assert_eq!(list.type_name, "User");
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_yaml_value_to_item_simple_object() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut obj = Mapping::new();
        obj.insert(
            YamlValue::String("name".to_string()),
            YamlValue::String("test".to_string()),
        );
        obj.insert(
            YamlValue::String("age".to_string()),
            YamlValue::Number(42.into()),
        );

        let item =
            yaml_value_to_item(&YamlValue::Mapping(obj), "test", &config, &mut structs, 0).unwrap();
        if let Item::Object(map) = item {
            assert_eq!(map.len(), 2);
            assert!(map.contains_key("name"));
            assert!(map.contains_key("age"));
        } else {
            panic!("Expected object");
        }
    }

    // ==================== yaml_to_value tests ====================

    #[test]
    fn test_yaml_to_value_null() {
        let value = yaml_to_value(&YamlValue::Null, &FromYamlConfig::default(), 0).unwrap();
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn test_yaml_to_value_bool() {
        assert_eq!(
            yaml_to_value(&YamlValue::Bool(true), &FromYamlConfig::default(), 0).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            yaml_to_value(&YamlValue::Bool(false), &FromYamlConfig::default(), 0).unwrap(),
            Value::Bool(false)
        );
    }

    #[test]
    fn test_yaml_to_value_int() {
        let value =
            yaml_to_value(&YamlValue::Number(42.into()), &FromYamlConfig::default(), 0).unwrap();
        assert_eq!(value, Value::Int(42));
    }

    #[test]
    fn test_yaml_to_value_float() {
        let yaml_num = YamlValue::Number(serde_yaml::Number::from(3.5));
        let value = yaml_to_value(&yaml_num, &FromYamlConfig::default(), 0).unwrap();
        if let Value::Float(f) = value {
            assert!((f - 3.5).abs() < 0.001);
        } else {
            panic!("Expected float");
        }
    }

    #[test]
    fn test_yaml_to_value_string() {
        let value = yaml_to_value(
            &YamlValue::String("hello".to_string()),
            &FromYamlConfig::default(),
            0,
        )
        .unwrap();
        assert_eq!(value, Value::String("hello".to_string().into()));
    }

    #[test]
    fn test_yaml_to_value_expression() {
        let value = yaml_to_value(
            &YamlValue::String("$(foo)".to_string()),
            &FromYamlConfig::default(),
            0,
        )
        .unwrap();
        if let Value::Expression(e) = value {
            assert_eq!(e.to_string(), "foo");
        } else {
            panic!("Expected expression");
        }
    }

    #[test]
    fn test_yaml_to_value_reference() {
        let mut ref_map = Mapping::new();
        ref_map.insert(
            YamlValue::String("@ref".to_string()),
            YamlValue::String("@user1".to_string()),
        );
        let value =
            yaml_to_value(&YamlValue::Mapping(ref_map), &FromYamlConfig::default(), 0).unwrap();
        if let Value::Reference(r) = value {
            assert_eq!(r.id.as_ref(), "user1");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_yaml_to_value_tensor() {
        let seq = YamlValue::Sequence(vec![
            YamlValue::Number(1.into()),
            YamlValue::Number(2.into()),
        ]);
        let value = yaml_to_value(&seq, &FromYamlConfig::default(), 0).unwrap();
        if let Value::Tensor(tensor_box) = value {
            if let Tensor::Array(arr) = *tensor_box {
                assert_eq!(arr.len(), 2);
            } else {
                panic!("Expected tensor array");
            }
        } else {
            panic!("Expected tensor");
        }
    }

    #[test]
    fn test_yaml_to_value_empty_sequence() {
        let seq = YamlValue::Sequence(vec![]);
        let value = yaml_to_value(&seq, &FromYamlConfig::default(), 0).unwrap();
        if let Value::List(items) = value {
            assert!(items.is_empty());
        } else {
            panic!("Expected empty list");
        }
    }

    #[test]
    fn test_yaml_to_value_nested_object_error() {
        // Regular nested objects are not allowed in scalar context
        let mut obj = Mapping::new();
        obj.insert(
            YamlValue::String("nested".to_string()),
            YamlValue::String("value".to_string()),
        );
        let result = yaml_to_value(&YamlValue::Mapping(obj), &FromYamlConfig::default(), 0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Nested objects not allowed"));
    }

    // ==================== Value::List tests ====================

    #[test]
    fn test_yaml_to_value_list_strings() {
        let seq = YamlValue::Sequence(vec![
            YamlValue::String("admin".to_string()),
            YamlValue::String("editor".to_string()),
            YamlValue::String("viewer".to_string()),
        ]);
        let value = yaml_to_value(&seq, &FromYamlConfig::default(), 0).unwrap();
        if let Value::List(items) = value {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].as_str(), Some("admin"));
            assert_eq!(items[1].as_str(), Some("editor"));
            assert_eq!(items[2].as_str(), Some("viewer"));
        } else {
            panic!("Expected list, got {:?}", value);
        }
    }

    #[test]
    fn test_yaml_to_value_list_bools() {
        let seq = YamlValue::Sequence(vec![
            YamlValue::Bool(true),
            YamlValue::Bool(false),
            YamlValue::Bool(true),
        ]);
        let value = yaml_to_value(&seq, &FromYamlConfig::default(), 0).unwrap();
        if let Value::List(items) = value {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].as_bool(), Some(true));
            assert_eq!(items[1].as_bool(), Some(false));
            assert_eq!(items[2].as_bool(), Some(true));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_yaml_to_value_list_mixed() {
        let seq = YamlValue::Sequence(vec![
            YamlValue::String("test".to_string()),
            YamlValue::Number(42.into()),
            YamlValue::Bool(true),
            YamlValue::Null,
        ]);
        let value = yaml_to_value(&seq, &FromYamlConfig::default(), 0).unwrap();
        if let Value::List(items) = value {
            assert_eq!(items.len(), 4);
            assert_eq!(items[0].as_str(), Some("test"));
            assert_eq!(items[1].as_int(), Some(42));
            assert_eq!(items[2].as_bool(), Some(true));
            assert!(items[3].is_null());
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_yaml_to_value_list_with_nulls() {
        let seq = YamlValue::Sequence(vec![
            YamlValue::Null,
            YamlValue::String("test".to_string()),
            YamlValue::Null,
        ]);
        let value = yaml_to_value(&seq, &FromYamlConfig::default(), 0).unwrap();
        if let Value::List(items) = value {
            assert_eq!(items.len(), 3);
            assert!(items[0].is_null());
            assert_eq!(items[1].as_str(), Some("test"));
            assert!(items[2].is_null());
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_yaml_to_value_list_with_references() {
        let mut ref_map1 = Mapping::new();
        ref_map1.insert(
            YamlValue::String("@ref".to_string()),
            YamlValue::String("@user1".to_string()),
        );
        let mut ref_map2 = Mapping::new();
        ref_map2.insert(
            YamlValue::String("@ref".to_string()),
            YamlValue::String("@User:user2".to_string()),
        );

        let seq = YamlValue::Sequence(vec![
            YamlValue::Mapping(ref_map1),
            YamlValue::Mapping(ref_map2),
        ]);
        let value = yaml_to_value(&seq, &FromYamlConfig::default(), 0).unwrap();
        if let Value::List(items) = value {
            assert_eq!(items.len(), 2);
            assert!(items[0].is_reference());
            assert!(items[1].is_reference());
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_is_scalar_list_sequence_strings() {
        let seq = vec![
            YamlValue::String("a".to_string()),
            YamlValue::String("b".to_string()),
        ];
        assert!(is_scalar_list_sequence(&seq));
    }

    #[test]
    fn test_is_scalar_list_sequence_bools() {
        let seq = vec![YamlValue::Bool(true), YamlValue::Bool(false)];
        assert!(is_scalar_list_sequence(&seq));
    }

    #[test]
    fn test_is_scalar_list_sequence_nulls() {
        let seq = vec![YamlValue::Null, YamlValue::Null];
        assert!(is_scalar_list_sequence(&seq));
    }

    #[test]
    fn test_is_scalar_list_sequence_mixed_with_numbers() {
        let seq = vec![
            YamlValue::String("test".to_string()),
            YamlValue::Number(42.into()),
        ];
        assert!(is_scalar_list_sequence(&seq));
    }

    #[test]
    fn test_is_scalar_list_sequence_only_numbers() {
        let seq = vec![YamlValue::Number(1.into()), YamlValue::Number(2.into())];
        assert!(!is_scalar_list_sequence(&seq));
    }

    #[test]
    fn test_is_scalar_list_sequence_empty() {
        let seq: Vec<YamlValue> = vec![];
        assert!(!is_scalar_list_sequence(&seq));
    }

    #[test]
    fn test_is_scalar_list_sequence_with_reference() {
        let mut ref_map = Mapping::new();
        ref_map.insert(
            YamlValue::String("@ref".to_string()),
            YamlValue::String("@user1".to_string()),
        );
        let seq = vec![YamlValue::Mapping(ref_map)];
        assert!(is_scalar_list_sequence(&seq));
    }

    // ==================== yaml_sequence_to_tensor tests ====================

    #[test]
    fn test_yaml_sequence_to_tensor_1d() {
        let seq = vec![
            YamlValue::Number(1.into()),
            YamlValue::Number(2.into()),
            YamlValue::Number(3.into()),
        ];
        let tensor = yaml_sequence_to_tensor(&seq, &FromYamlConfig::default(), "test", 0).unwrap();
        if let Tensor::Array(arr) = tensor {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Tensor::Scalar(1.0));
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_yaml_sequence_to_tensor_2d() {
        let seq = vec![
            YamlValue::Sequence(vec![
                YamlValue::Number(1.into()),
                YamlValue::Number(2.into()),
            ]),
            YamlValue::Sequence(vec![
                YamlValue::Number(3.into()),
                YamlValue::Number(4.into()),
            ]),
        ];
        let tensor = yaml_sequence_to_tensor(&seq, &FromYamlConfig::default(), "test", 0).unwrap();
        if let Tensor::Array(outer) = tensor {
            assert_eq!(outer.len(), 2);
            if let Tensor::Array(inner) = &outer[0] {
                assert_eq!(inner.len(), 2);
            } else {
                panic!("Expected nested array");
            }
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_yaml_sequence_to_tensor_empty() {
        let seq: Vec<YamlValue> = vec![];
        let tensor = yaml_sequence_to_tensor(&seq, &FromYamlConfig::default(), "test", 0).unwrap();
        if let Tensor::Array(arr) = tensor {
            assert!(arr.is_empty());
        } else {
            panic!("Expected empty array");
        }
    }

    #[test]
    fn test_yaml_sequence_to_tensor_invalid_element() {
        let seq = vec![
            YamlValue::Number(1.into()),
            YamlValue::String("invalid".to_string()),
        ];
        let result = yaml_sequence_to_tensor(&seq, &FromYamlConfig::default(), "test", 0);
        assert!(result.is_err());
    }

    // ==================== yaml_sequence_to_matrix_list tests ====================

    #[test]
    fn test_yaml_sequence_to_matrix_list_simple() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut obj = Mapping::new();
        obj.insert(
            YamlValue::String("id".to_string()),
            YamlValue::String("u1".to_string()),
        );
        obj.insert(
            YamlValue::String("name".to_string()),
            YamlValue::String("Alice".to_string()),
        );

        let seq = vec![YamlValue::Mapping(obj)];
        let list = yaml_sequence_to_matrix_list(&seq, "users", &config, &mut structs, 0).unwrap();

        assert_eq!(list.type_name, "User");
        assert_eq!(list.rows.len(), 1);
        assert_eq!(list.rows[0].id, "u1");
    }

    #[test]
    fn test_yaml_sequence_to_matrix_list_schema_inference() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut obj = Mapping::new();
        obj.insert(
            YamlValue::String("id".to_string()),
            YamlValue::String("u1".to_string()),
        );
        obj.insert(
            YamlValue::String("name".to_string()),
            YamlValue::String("Alice".to_string()),
        );
        obj.insert(
            YamlValue::String("age".to_string()),
            YamlValue::Number(30.into()),
        );

        let seq = vec![YamlValue::Mapping(obj)];
        let list = yaml_sequence_to_matrix_list(&seq, "users", &config, &mut structs, 0).unwrap();

        // Schema should be sorted with id first
        assert_eq!(list.schema[0], "id");
        assert!(list.schema.contains(&"name".to_string()));
        assert!(list.schema.contains(&"age".to_string()));
    }

    #[test]
    fn test_yaml_sequence_to_matrix_list_empty() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let seq: Vec<YamlValue> = vec![];
        let list = yaml_sequence_to_matrix_list(&seq, "users", &config, &mut structs, 0).unwrap();

        assert_eq!(list.type_name, "User");
        assert!(list.rows.is_empty());
        // Default schema for empty list
        assert!(list.schema.contains(&"id".to_string()));
    }

    #[test]
    fn test_yaml_sequence_to_matrix_list_type_name_singularization() {
        let config = FromYamlConfig::default();
        let mut structs = BTreeMap::new();

        let mut obj = Mapping::new();
        obj.insert(
            YamlValue::String("id".to_string()),
            YamlValue::String("1".to_string()),
        );

        let seq = vec![YamlValue::Mapping(obj)];

        // Test various pluralizations
        let list = yaml_sequence_to_matrix_list(&seq, "users", &config, &mut structs, 0).unwrap();
        assert_eq!(list.type_name, "User");

        let list =
            yaml_sequence_to_matrix_list(&seq, "companies", &config, &mut structs, 0).unwrap();
        assert_eq!(list.type_name, "Company");

        // "people" uses standard singularization (just removes 's' and capitalizes)
        let list = yaml_sequence_to_matrix_list(&seq, "people", &config, &mut structs, 0).unwrap();
        assert_eq!(list.type_name, "People");

        let list = yaml_sequence_to_matrix_list(&seq, "items", &config, &mut structs, 0).unwrap();
        assert_eq!(list.type_name, "Item");
    }

    // ==================== from_yaml integration tests ====================

    #[test]
    fn test_from_yaml_simple() {
        let yaml = "name: test\ncount: 42\n";
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        assert_eq!(doc.version, (2, 0));
        assert_eq!(doc.root.len(), 2);
    }

    #[test]
    fn test_from_yaml_invalid() {
        let yaml = "{ invalid yaml: [";
        let config = FromYamlConfig::default();
        let result = from_yaml(yaml, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("YAML parse error"));
    }

    #[test]
    fn test_from_yaml_non_mapping_root() {
        let yaml = "- item1\n- item2\n";
        let config = FromYamlConfig::default();
        let result = from_yaml(yaml, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Root must be a YAML mapping"));
    }

    #[test]
    fn test_from_yaml_with_list() {
        let yaml = r"
users:
  - id: u1
    name: Alice
  - id: u2
    name: Bob
";
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        if let Item::List(list) = &doc.root["users"] {
            assert_eq!(list.rows.len(), 2);
            assert_eq!(list.type_name, "User");
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_from_yaml_with_nested_object() {
        let yaml = r"
config:
  server:
    host: localhost
    port: 8080
";
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        if let Item::Object(config_obj) = &doc.root["config"] {
            if let Item::Object(server) = &config_obj["server"] {
                assert!(server.contains_key("host"));
                assert!(server.contains_key("port"));
            } else {
                panic!("Expected server object");
            }
        } else {
            panic!("Expected config object");
        }
    }

    #[test]
    fn test_from_yaml_with_tensor() {
        let yaml = r"
matrix:
  - [1, 2, 3]
  - [4, 5, 6]
";
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        if let Item::Scalar(Value::Tensor(tensor_box)) = &doc.root["matrix"] {
            if let Tensor::Array(ref outer) = **tensor_box {
                assert_eq!(outer.len(), 2);
            } else {
                panic!("Expected tensor array");
            }
        } else {
            panic!("Expected tensor");
        }
    }

    #[test]
    fn test_from_yaml_skips_metadata_keys() {
        let yaml = r#"
__type__: "MyType"
__schema__: ["id", "name"]
name: test
__other__: notskipped
"#;
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        // Only KNOWN HEDL metadata keys (__type__, __schema__) are skipped
        assert!(!doc.root.contains_key("__type__"));
        assert!(!doc.root.contains_key("__schema__"));
        // Regular keys and other __ keys are NOT skipped
        assert!(doc.root.contains_key("name"));
        assert!(doc.root.contains_key("__other__")); // Other __ keys are preserved
    }

    #[test]
    fn test_from_yaml_custom_version() {
        let yaml = "name: test\n";
        let config = FromYamlConfig {
            default_type_name: "Item".to_string(),
            version: (2, 5),
            ..Default::default()
        };
        let doc = from_yaml(yaml, &config).unwrap();
        assert_eq!(doc.version, (2, 5));
    }

    // ==================== from_yaml_value tests ====================

    #[test]
    fn test_from_yaml_value_mapping() {
        let mut map = Mapping::new();
        map.insert(
            YamlValue::String("key".to_string()),
            YamlValue::String("value".to_string()),
        );

        let config = FromYamlConfig::default();
        let doc = from_yaml_value(&YamlValue::Mapping(map), &config).unwrap();

        assert_eq!(doc.root.len(), 1);
        assert!(doc.root.contains_key("key"));
    }

    #[test]
    fn test_from_yaml_value_non_mapping() {
        let config = FromYamlConfig::default();
        let result = from_yaml_value(&YamlValue::Number(42.into()), &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Root must be a YAML mapping"));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_yaml_unicode_keys_and_values() {
        let yaml = "名前: テスト\nцена: 100\n";
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        assert!(doc.root.contains_key("名前"));
        assert!(doc.root.contains_key("цена"));
    }

    #[test]
    fn test_yaml_multiline_string() {
        let yaml = r"
description: |
  This is a
  multiline string
";
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        if let Item::Scalar(Value::String(s)) = &doc.root["description"] {
            assert!(s.contains('\n'));
        } else {
            panic!("Expected string");
        }
    }

    #[test]
    fn test_yaml_anchors_and_aliases() {
        // Simple anchor/alias reference (not merge key)
        let yaml = r"
defaults: &defaults
  timeout: 30
  retries: 3
production:
  config: *defaults
  host: prod.example.com
";
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();

        // The alias reference should be resolved as nested object
        if let Item::Object(prod) = &doc.root["production"] {
            assert!(prod.contains_key("config"));
            assert!(prod.contains_key("host"));
            // config should be an object with timeout and retries
            if let Item::Object(config_obj) = &prod["config"] {
                assert!(config_obj.contains_key("timeout"));
                assert!(config_obj.contains_key("retries"));
            } else {
                panic!("Expected config object");
            }
        } else {
            panic!("Expected object");
        }
    }

    // ==================== Resource Limit Tests (DoS Protection) ====================

    #[test]
    fn test_max_document_size_exceeded() {
        // Create a document larger than the limit
        let config = FromYamlConfig {
            max_document_size: 100, // Very small limit for testing
            ..Default::default()
        };

        let yaml = "a".repeat(200); // 200 bytes, exceeds 100 byte limit
        let result = from_yaml(&yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Document size"));
        assert!(err.contains("exceeds maximum"));
    }

    #[test]
    fn test_max_document_size_within_limit() {
        let config = FromYamlConfig {
            max_document_size: 1000,
            ..Default::default()
        };

        let yaml = "name: test\nvalue: 123\n";
        let result = from_yaml(yaml, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_array_length_exceeded() {
        let config = FromYamlConfig {
            max_array_length: 5, // Very small limit for testing
            ..Default::default()
        };

        // Create YAML with array longer than limit
        let yaml = r"
numbers:
  - 1
  - 2
  - 3
  - 4
  - 5
  - 6
";
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Array length"));
        assert!(err.contains("exceeds maximum"));
    }

    #[test]
    fn test_max_array_length_within_limit() {
        let config = FromYamlConfig {
            max_array_length: 10,
            ..Default::default()
        };

        let yaml = r"
numbers:
  - 1
  - 2
  - 3
";
        let result = from_yaml(yaml, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_array_length_exceeded_in_matrix_list() {
        let config = FromYamlConfig {
            max_array_length: 2, // Very small limit
            ..Default::default()
        };

        let yaml = r"
users:
  - id: u1
    name: Alice
  - id: u2
    name: Bob
  - id: u3
    name: Charlie
";
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Array length"));
        assert!(err.contains("exceeds maximum"));
    }

    #[test]
    fn test_max_nesting_depth_exceeded() {
        let config = FromYamlConfig {
            max_nesting_depth: 3, // Very shallow for testing
            ..Default::default()
        };

        // Create deeply nested structure
        let yaml = r"
level1:
  level2:
    level3:
      level4:
        level5: value
";
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Maximum nesting depth"));
        assert!(err.contains("exceeded"));
    }

    #[test]
    fn test_max_nesting_depth_within_limit() {
        let config = FromYamlConfig {
            max_nesting_depth: 10,
            ..Default::default()
        };

        let yaml = r"
level1:
  level2:
    level3: value
";
        let result = from_yaml(yaml, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_nesting_depth_exceeded_in_tensor() {
        let config = FromYamlConfig {
            max_nesting_depth: 2, // Very shallow
            ..Default::default()
        };

        // Nested tensor that's too deep
        let yaml = r"
matrix:
  - - - [1, 2]
";
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Maximum nesting depth"));
    }

    #[test]
    fn test_default_limits_are_reasonable() {
        let config = FromYamlConfig::default();

        // Verify default limits are set to high values
        assert_eq!(config.max_document_size, 500 * 1024 * 1024); // 500MB
        assert_eq!(config.max_array_length, 10_000_000); // 10 million
        assert_eq!(config.max_nesting_depth, 10_000); // 10,000 levels

        // Verify constants match defaults
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
        assert_eq!(config.max_array_length, DEFAULT_MAX_ARRAY_LENGTH);
        assert_eq!(config.max_nesting_depth, DEFAULT_MAX_NESTING_DEPTH);
    }

    #[test]
    fn test_custom_limits_configuration() {
        let config = FromYamlConfig {
            default_type_name: "Custom".to_string(),
            version: (2, 0),
            max_document_size: 50_000_000,
            max_array_length: 500_000,
            max_nesting_depth: 500,
        };

        assert_eq!(config.max_document_size, 50_000_000);
        assert_eq!(config.max_array_length, 500_000);
        assert_eq!(config.max_nesting_depth, 500);
    }

    #[test]
    fn test_nested_children_array_length_limit() {
        let config = FromYamlConfig {
            max_array_length: 2,
            ..Default::default()
        };

        let yaml = r"
users:
  - id: u1
    name: Alice
    posts:
      - id: p1
        title: First
      - id: p2
        title: Second
      - id: p3
        title: Third
";
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Array length"));
        assert!(err.contains("exceeds maximum"));
    }

    #[test]
    fn test_tensor_array_length_limit() {
        let config = FromYamlConfig {
            max_array_length: 3,
            ..Default::default()
        };

        let yaml = r"
matrix:
  - [1, 2, 3, 4, 5]
";
        let result = from_yaml(yaml, &config);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("Array length"));
    }

    #[test]
    fn test_zero_limits_blocks_everything() {
        let config = FromYamlConfig {
            max_document_size: 0,
            max_array_length: 0,
            max_nesting_depth: 0,
            ..Default::default()
        };

        let yaml = "name: test\n";
        let result = from_yaml(yaml, &config);

        // Should fail on document size
        assert!(result.is_err());
    }

    #[test]
    fn test_large_valid_document_within_limits() {
        let config = FromYamlConfig::default();

        // Create a reasonably large document that's still within limits
        let mut items = Vec::new();
        for i in 0..1000 {
            items.push(format!("  - id: item{}\n    value: {}", i, i * 2));
        }
        let yaml = format!("items:\n{}", items.join("\n"));

        let result = from_yaml(&yaml, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_combined_limits_enforcement() {
        let config = FromYamlConfig {
            max_document_size: 500,
            max_array_length: 5,
            max_nesting_depth: 3,
            ..Default::default()
        };

        // Test each limit independently

        // Document size exceeded
        let large_doc = "a".repeat(600);
        assert!(from_yaml(&large_doc, &config).is_err());

        // Array length exceeded
        let long_array = r"
items:
  - 1
  - 2
  - 3
  - 4
  - 5
  - 6
";
        assert!(from_yaml(long_array, &config).is_err());

        // Nesting depth exceeded
        let deep_nesting = r"
a:
  b:
    c:
      d:
        e: value
";
        assert!(from_yaml(deep_nesting, &config).is_err());
    }

    // ==================== FromYamlConfigBuilder tests ====================

    #[test]
    fn test_builder_default() {
        let config = FromYamlConfigBuilder::new().build();
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (2, 0));
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
        assert_eq!(config.max_array_length, DEFAULT_MAX_ARRAY_LENGTH);
        assert_eq!(config.max_nesting_depth, DEFAULT_MAX_NESTING_DEPTH);
    }

    #[test]
    fn test_builder_from_default() {
        let config = FromYamlConfigBuilder::default().build();
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (2, 0));
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
    }

    #[test]
    fn test_builder_via_config() {
        let config = FromYamlConfig::builder().build();
        assert_eq!(config.default_type_name, "Item");
        assert_eq!(config.version, (2, 0));
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
    }

    #[test]
    fn test_builder_custom_document_size() {
        let config = FromYamlConfig::builder()
            .max_document_size(100 * 1024 * 1024)
            .build();
        assert_eq!(config.max_document_size, 100 * 1024 * 1024);
        // Other values should be defaults
        assert_eq!(config.max_array_length, DEFAULT_MAX_ARRAY_LENGTH);
        assert_eq!(config.max_nesting_depth, DEFAULT_MAX_NESTING_DEPTH);
    }

    #[test]
    fn test_builder_custom_array_length() {
        let config = FromYamlConfig::builder()
            .max_array_length(5_000_000)
            .build();
        assert_eq!(config.max_array_length, 5_000_000);
        // Other values should be defaults
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
        assert_eq!(config.max_nesting_depth, DEFAULT_MAX_NESTING_DEPTH);
    }

    #[test]
    fn test_builder_custom_nesting_depth() {
        let config = FromYamlConfig::builder().max_nesting_depth(5000).build();
        assert_eq!(config.max_nesting_depth, 5000);
        // Other values should be defaults
        assert_eq!(config.max_document_size, DEFAULT_MAX_DOCUMENT_SIZE);
        assert_eq!(config.max_array_length, DEFAULT_MAX_ARRAY_LENGTH);
    }

    #[test]
    fn test_builder_all_custom() {
        let config = FromYamlConfig::builder()
            .default_type_name("Entity")
            .version(2, 0)
            .max_document_size(200 * 1024 * 1024)
            .max_array_length(20_000_000)
            .max_nesting_depth(20_000)
            .build();

        assert_eq!(config.default_type_name, "Entity");
        assert_eq!(config.version, (2, 0));
        assert_eq!(config.max_document_size, 200 * 1024 * 1024);
        assert_eq!(config.max_array_length, 20_000_000);
        assert_eq!(config.max_nesting_depth, 20_000);
    }

    #[test]
    fn test_builder_conservative_limits() {
        // Conservative limits for untrusted input
        let config = FromYamlConfig::builder()
            .max_document_size(10 * 1024 * 1024) // 10 MB
            .max_array_length(100_000)
            .max_nesting_depth(100)
            .build();

        assert_eq!(config.max_document_size, 10 * 1024 * 1024);
        assert_eq!(config.max_array_length, 100_000);
        assert_eq!(config.max_nesting_depth, 100);
    }

    #[test]
    fn test_builder_type_name_from_string() {
        let config = FromYamlConfig::builder()
            .default_type_name("CustomType".to_string())
            .build();
        assert_eq!(config.default_type_name, "CustomType");
    }

    #[test]
    fn test_builder_type_name_from_str() {
        let config = FromYamlConfig::builder()
            .default_type_name("CustomType")
            .build();
        assert_eq!(config.default_type_name, "CustomType");
    }

    #[test]
    fn test_builder_chaining() {
        // Test that builder methods can be chained in any order
        let config1 = FromYamlConfig::builder()
            .max_document_size(100_000_000)
            .max_array_length(1_000_000)
            .max_nesting_depth(1000)
            .build();

        let config2 = FromYamlConfig::builder()
            .max_nesting_depth(1000)
            .max_array_length(1_000_000)
            .max_document_size(100_000_000)
            .build();

        assert_eq!(config1.max_document_size, config2.max_document_size);
        assert_eq!(config1.max_array_length, config2.max_array_length);
        assert_eq!(config1.max_nesting_depth, config2.max_nesting_depth);
    }

    #[test]
    fn test_builder_debug() {
        let builder = FromYamlConfig::builder();
        let debug_str = format!("{builder:?}");
        assert!(debug_str.contains("FromYamlConfigBuilder"));
    }

    #[test]
    fn test_builder_clone() {
        let builder1 = FromYamlConfig::builder().max_document_size(100_000_000);
        let builder2 = builder1.clone();
        let config1 = builder1.build();
        let config2 = builder2.build();
        assert_eq!(config1.max_document_size, config2.max_document_size);
    }

    #[test]
    fn test_builder_with_yaml_parsing() {
        // Test that builder-configured limits work in actual parsing
        let config = FromYamlConfig::builder()
            .max_document_size(1000)
            .max_array_length(5)
            .build();

        let yaml = r"
numbers:
  - 1
  - 2
  - 3
";
        // Should succeed - within limits
        let result = from_yaml(yaml, &config);
        assert!(result.is_ok());

        // Test exceeding array length
        let yaml_long = r"
numbers:
  - 1
  - 2
  - 3
  - 4
  - 5
  - 6
";
        let result = from_yaml(yaml_long, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_constants_match_defaults() {
        // Verify that the constants have the expected values
        assert_eq!(DEFAULT_MAX_DOCUMENT_SIZE, 500 * 1024 * 1024);
        assert_eq!(DEFAULT_MAX_ARRAY_LENGTH, 10_000_000);
        assert_eq!(DEFAULT_MAX_NESTING_DEPTH, 10_000);
    }
}
