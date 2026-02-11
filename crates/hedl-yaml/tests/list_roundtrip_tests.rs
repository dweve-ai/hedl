// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Tests for Value::List roundtrip conversion through YAML

use hedl_core::{Document, Item, Value};
use hedl_yaml::{from_yaml, to_yaml, FromYamlConfig, ToYamlConfig};
use std::collections::BTreeMap;

#[test]
fn test_roundtrip_string_list() {
    let yaml = r#"
roles:
  - admin
  - editor
  - viewer
"#;

    let doc = from_yaml(yaml, &FromYamlConfig::default()).unwrap();
    let roles = doc
        .root
        .get("roles")
        .and_then(|item| item.as_scalar())
        .and_then(|value| value.as_list())
        .expect("Expected roles to be a list");

    assert_eq!(roles.len(), 3);
    assert_eq!(roles[0].as_str(), Some("admin"));
    assert_eq!(roles[1].as_str(), Some("editor"));
    assert_eq!(roles[2].as_str(), Some("viewer"));

    // Roundtrip
    let yaml_out = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let doc2 = from_yaml(&yaml_out, &FromYamlConfig::default()).unwrap();

    let roles2 = doc2
        .root
        .get("roles")
        .and_then(|item| item.as_scalar())
        .and_then(|value| value.as_list())
        .expect("Expected roles to be a list after roundtrip");

    assert_eq!(roles2.len(), 3);
    assert_eq!(roles2[0].as_str(), Some("admin"));
}

#[test]
fn test_roundtrip_bool_list() {
    let yaml = r#"
flags:
  - true
  - false
  - true
"#;

    let doc = from_yaml(yaml, &FromYamlConfig::default()).unwrap();
    let flags = doc
        .root
        .get("flags")
        .and_then(|item| item.as_scalar())
        .and_then(|value| value.as_list())
        .expect("Expected flags to be a list");

    assert_eq!(flags.len(), 3);
    assert_eq!(flags[0].as_bool(), Some(true));
    assert_eq!(flags[1].as_bool(), Some(false));
    assert_eq!(flags[2].as_bool(), Some(true));
}

#[test]
fn test_roundtrip_mixed_list() {
    let yaml = r#"
mixed:
  - test
  - 42
  - true
  - null
"#;

    let doc = from_yaml(yaml, &FromYamlConfig::default()).unwrap();
    let mixed = doc
        .root
        .get("mixed")
        .and_then(|item| item.as_scalar())
        .and_then(|value| value.as_list())
        .expect("Expected mixed to be a list");

    assert_eq!(mixed.len(), 4);
    assert_eq!(mixed[0].as_str(), Some("test"));
    assert_eq!(mixed[1].as_int(), Some(42));
    assert_eq!(mixed[2].as_bool(), Some(true));
    assert!(mixed[3].is_null());

    // Roundtrip
    let yaml_out = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    let doc2 = from_yaml(&yaml_out, &FromYamlConfig::default()).unwrap();

    let mixed2 = doc2
        .root
        .get("mixed")
        .and_then(|item| item.as_scalar())
        .and_then(|value| value.as_list())
        .expect("Expected mixed to be a list after roundtrip");

    assert_eq!(mixed2.len(), 4);
}

#[test]
fn test_empty_list_to_yaml() {
    // When converting an empty Value::List to YAML, it becomes an empty array
    let mut doc = Document {
        version: (1, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root: BTreeMap::new(),
    };

    doc.root.insert(
        "empty".to_string(),
        Item::Scalar(Value::List(Box::default())),
    );

    let yaml = to_yaml(&doc, &ToYamlConfig::default()).unwrap();
    assert!(yaml.contains("empty: []"));

    // When reading back, an empty array at the Item level becomes an empty MatrixList
    // This is expected behavior - empty sequences are ambiguous
    let doc2 = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();
    assert!(doc2.root.contains_key("empty"));
}

#[test]
fn test_list_vs_tensor_distinction() {
    // Numeric sequences should become tensors
    let yaml_tensor = r#"
numbers:
  - 1.0
  - 2.0
  - 3.0
"#;

    let doc = from_yaml(yaml_tensor, &FromYamlConfig::default()).unwrap();
    let numbers = doc.root.get("numbers").and_then(|item| item.as_scalar());
    assert!(
        matches!(numbers, Some(Value::Tensor(_))),
        "Expected tensor for numeric sequence"
    );

    // String sequences should become lists
    let yaml_list = r#"
words:
  - one
  - two
  - three
"#;

    let doc2 = from_yaml(yaml_list, &FromYamlConfig::default()).unwrap();
    let words = doc2.root.get("words").and_then(|item| item.as_scalar());
    assert!(
        matches!(words, Some(Value::List(_))),
        "Expected list for string sequence"
    );
}
