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

//! Tests verifying all re-exports work correctly.
//!
//! This test suite ensures that all types, functions, and modules re-exported
//! by the hedl crate are accessible and function as expected.

use hedl::{
    canonicalize,
    core_parse,
    from_json,
    lint,
    parse,
    parse_lenient,
    parse_with_limits,
    to_json,
    validate,
    // Core types
    Document,
    HedlError,
    HedlErrorKind,
    Item,
    Limits,
    MatrixList,
    Node,
    ParseOptions,
    Reference,
    ReferenceMode,
    Tensor,
    Value,
    // Constants
    SUPPORTED_VERSION,
    VERSION,
};

// =============================================================================
// Core Function Re-exports
// =============================================================================

#[test]
fn test_core_parse_function() {
    let input = b"%VERSION: 1.0\n---\nkey: value";
    let doc = core_parse(input).unwrap();
    assert_eq!(doc.version, (1, 0));
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_parse_with_limits_function() {
    let input = b"%VERSION: 1.0\n---\nkey: value";
    let options = ParseOptions::default();
    let doc = parse_with_limits(input, options).unwrap();
    assert_eq!(doc.version, (1, 0));
}

#[test]
fn test_parse_with_limits_custom_options() {
    let input = b"%VERSION: 1.0\n---\nref: @nonexistent";

    // Strict mode should fail
    let strict = ParseOptions {
        reference_mode: ReferenceMode::Strict,
        ..Default::default()
    };
    assert!(parse_with_limits(input, strict).is_err());

    // Lenient mode should succeed
    let lenient = ParseOptions {
        reference_mode: ReferenceMode::Lenient,
        ..Default::default()
    };
    assert!(parse_with_limits(input, lenient).is_ok());
}

#[test]
fn test_parse_with_limits_custom_limits() {
    let input = b"%VERSION: 1.0\n---\nkey: value";
    let options = ParseOptions {
        reference_mode: ReferenceMode::Strict,
        limits: Limits {
            max_indent_depth: 10,
            max_file_size: 1024 * 1024,
            ..Default::default()
        },
    };
    let doc = parse_with_limits(input, options).unwrap();
    assert_eq!(doc.root.len(), 1);
}

// =============================================================================
// Error Type Re-exports
// =============================================================================

#[test]
fn test_hedl_error_constructors() {
    let err = HedlError::syntax("bad token", 5);
    assert_eq!(err.kind, HedlErrorKind::Syntax);
    assert_eq!(err.line, 5);

    let err = HedlError::version("bad version", 1);
    assert_eq!(err.kind, HedlErrorKind::Version);

    let err = HedlError::schema("type mismatch", 3);
    assert_eq!(err.kind, HedlErrorKind::Schema);

    let err = HedlError::alias("duplicate alias", 7);
    assert_eq!(err.kind, HedlErrorKind::Alias);

    let err = HedlError::shape("wrong count", 10);
    assert_eq!(err.kind, HedlErrorKind::Shape);

    let err = HedlError::semantic("invalid", 2);
    assert_eq!(err.kind, HedlErrorKind::Semantic);

    let err = HedlError::reference("unresolved", 4);
    assert_eq!(err.kind, HedlErrorKind::Reference);

    let err = HedlError::collision("duplicate id", 6);
    assert_eq!(err.kind, HedlErrorKind::Collision);

    let err = HedlError::orphan_row("no parent", 8);
    assert_eq!(err.kind, HedlErrorKind::OrphanRow);

    let err = HedlError::security("depth exceeded", 0);
    assert_eq!(err.kind, HedlErrorKind::Security);

    let err = HedlError::io("read failed");
    assert_eq!(err.kind, HedlErrorKind::IO);

    let err = HedlError::conversion("convert failed");
    assert_eq!(err.kind, HedlErrorKind::Conversion);
}

#[test]
fn test_hedl_error_with_column() {
    let err = HedlError::syntax("error", 10).with_column(20);
    assert_eq!(err.column, Some(20));
}

#[test]
fn test_hedl_error_display() {
    let err = HedlError::syntax("unexpected token", 5);
    let msg = format!("{err}");
    assert!(msg.contains("unexpected token"));
    assert!(msg.contains('5'));
}

#[test]
fn test_hedl_error_kind_enum() {
    // Verify all enum variants are accessible
    let kinds = vec![
        HedlErrorKind::Syntax,
        HedlErrorKind::Version,
        HedlErrorKind::Schema,
        HedlErrorKind::Alias,
        HedlErrorKind::Shape,
        HedlErrorKind::Semantic,
        HedlErrorKind::Reference,
        HedlErrorKind::Collision,
        HedlErrorKind::OrphanRow,
        HedlErrorKind::Security,
        HedlErrorKind::IO,
        HedlErrorKind::Conversion,
    ];
    assert_eq!(kinds.len(), 12);
}

// =============================================================================
// Value Type Re-exports
// =============================================================================

#[test]
fn test_value_enum_variants() {
    let null_val = Value::Null;
    assert!(matches!(null_val, Value::Null));

    let bool_val = Value::Bool(true);
    assert!(matches!(bool_val, Value::Bool(true)));

    let int_val = Value::Int(42);
    assert!(matches!(int_val, Value::Int(42)));

    let float_val = Value::Float(3.25); // Not using PI constant
    assert!(matches!(float_val, Value::Float(_)));

    let str_val = Value::String("test".to_string().into());
    assert!(matches!(str_val, Value::String(_)));

    let tensor_val = Value::Tensor(Box::new(Tensor::Scalar(1.0)));
    assert!(matches!(tensor_val, Value::Tensor(_)));

    let ref_val = Value::Reference(Reference {
        type_name: None,
        id: "alice".to_string().into(),
    });
    assert!(matches!(ref_val, Value::Reference(_)));
}

#[test]
fn test_value_display() {
    let val = Value::String("hello".to_string().into());
    let s = format!("{val}");
    assert!(s.contains("hello"));
}

// =============================================================================
// Tensor Type Re-exports
// =============================================================================

#[test]
fn test_tensor_scalar() {
    let t = Tensor::Scalar(42.0);
    assert_eq!(t.shape(), Vec::<usize>::new());
    assert_eq!(t.flatten(), vec![42.0]);
    assert!(t.is_integer());
}

#[test]
fn test_tensor_array() {
    let t = Tensor::Array(vec![
        Tensor::Scalar(1.0),
        Tensor::Scalar(2.0),
        Tensor::Scalar(3.0),
    ]);
    assert_eq!(t.shape(), vec![3]);
    assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0]);
}

#[test]
fn test_tensor_2d() {
    let t = Tensor::Array(vec![
        Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
        Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
    ]);
    assert_eq!(t.shape(), vec![2, 2]);
    assert_eq!(t.flatten(), vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_tensor_is_integer() {
    let int_tensor = Tensor::Scalar(42.0);
    assert!(int_tensor.is_integer());

    let float_tensor = Tensor::Scalar(3.15); // Not using PI constant
    assert!(!float_tensor.is_integer());
}

// =============================================================================
// Reference Type Re-exports
// =============================================================================

#[test]
fn test_reference_without_type() {
    let r = Reference {
        type_name: None,
        id: "alice".to_string().into(),
    };
    assert_eq!(r.id.as_ref(), "alice");
    assert!(r.type_name.is_none());
}

#[test]
fn test_reference_with_type() {
    let r = Reference {
        type_name: Some("User".to_string().into()),
        id: "alice".to_string().into(),
    };
    assert_eq!(r.id.as_ref(), "alice");
    assert_eq!(r.type_name.as_deref(), Some("User"));
}

#[test]
fn test_reference_debug() {
    let r = Reference {
        type_name: Some("User".to_string().into()),
        id: "alice".to_string().into(),
    };
    let s = format!("{r:?}");
    assert!(s.contains("User"));
    assert!(s.contains("alice"));
}

// =============================================================================
// Document Type Re-exports
// =============================================================================

#[test]
fn test_document_new() {
    let doc = Document::new((1, 0));
    assert_eq!(doc.version, (1, 0));
    assert!(doc.root.is_empty());
    assert!(doc.aliases.is_empty());
    assert!(doc.structs.is_empty());
}

#[test]
fn test_document_with_data() {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("key".to_string(), Item::Scalar(Value::Int(42)));
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_document_clone() {
    let mut doc1 = Document::new((1, 0));
    doc1.root
        .insert("key".to_string(), Item::Scalar(Value::Int(42)));

    let doc2 = doc1.clone();
    assert_eq!(doc1.version, doc2.version);
    assert_eq!(doc1.root.len(), doc2.root.len());
}

// =============================================================================
// Node Type Re-exports
// =============================================================================

#[test]
fn test_node_creation() {
    let node = Node::new(
        "User",
        "alice",
        vec![Value::Int(1), Value::String("Alice".to_string().into())],
    );
    assert_eq!(node.type_name, "User");
    assert_eq!(node.id, "alice");
    assert_eq!(node.fields.len(), 2);
}

#[test]
fn test_node_display() {
    let node = Node::new("User", "alice", vec![Value::Int(1)]);
    let s = format!("{node:?}");
    assert!(!s.is_empty());
}

#[test]
fn test_node_get_field() {
    let node = Node::new("User", "alice", vec![Value::Int(1), Value::Int(2)]);
    assert!(node.get_field(0).is_some());
    assert!(node.get_field(10).is_none());
}

#[test]
fn test_node_add_child() {
    let mut parent = Node::new("Parent", "p1", vec![]);
    let child = Node::new("Child", "c1", vec![]);
    parent.add_child("Child", child);
    assert!(parent.children().is_some());
}

#[test]
fn test_node_child_count() {
    let mut node = Node::new("Test", "t1", vec![]);
    assert_eq!(node.get_child_count(), None);
    node.set_child_count(5);
    assert_eq!(node.get_child_count(), Some(5));
}

// =============================================================================
// MatrixList Type Re-exports
// =============================================================================

#[test]
fn test_matrix_list_creation() {
    let list = MatrixList::new(
        "Row",
        vec!["a".to_string(), "b".to_string(), "c".to_string()],
    );
    assert_eq!(list.type_name, "Row");
    assert_eq!(list.schema.len(), 3);
    assert_eq!(list.rows.len(), 0);
}

#[test]
fn test_matrix_list_add_row() {
    let mut list = MatrixList::new("Row", vec!["a".to_string(), "b".to_string()]);
    let node = Node::new("Row", "1", vec![Value::Int(1), Value::Int(2)]);
    list.add_row(node);
    assert_eq!(list.rows.len(), 1);
}

#[test]
fn test_matrix_list_with_rows() {
    let rows = vec![
        Node::new("Row", "1", vec![Value::Int(1)]),
        Node::new("Row", "2", vec![Value::Int(2)]),
    ];
    let list = MatrixList::with_rows("Row", vec!["a".to_string()], rows);
    assert_eq!(list.rows.len(), 2);
}

#[test]
fn test_matrix_list_count_hint() {
    let list = MatrixList::with_count_hint("Row", vec!["a".to_string()], 5);
    assert_eq!(list.count_hint, Some(5));
}

// =============================================================================
// Item Type Re-exports
// =============================================================================

#[test]
fn test_item_scalar() {
    let item = Item::Scalar(Value::Int(42));
    assert!(matches!(item, Item::Scalar(_)));
}

#[test]
fn test_item_object() {
    use std::collections::BTreeMap;
    let mut map = BTreeMap::new();
    map.insert("key".to_string(), Item::Scalar(Value::Int(42)));
    let item = Item::Object(map);
    assert!(matches!(item, Item::Object(_)));
}

#[test]
fn test_item_list() {
    let list = MatrixList::new("Row", vec!["a".to_string()]);
    let item = Item::List(list);
    assert!(matches!(item, Item::List(_)));
}

#[test]
fn test_item_as_scalar() {
    let item = Item::Scalar(Value::Int(42));
    assert!(item.as_scalar().is_some());

    let item = Item::List(MatrixList::new("Row", vec![]));
    assert!(item.as_scalar().is_none());
}

// =============================================================================
// ParseOptions and Limits Re-exports
// =============================================================================

#[test]
fn test_parse_options_default() {
    let options = ParseOptions::default();
    assert_eq!(options.reference_mode, ReferenceMode::Strict);
}

#[test]
fn test_parse_options_custom() {
    let options = ParseOptions {
        reference_mode: ReferenceMode::Lenient,
        limits: Limits {
            max_file_size: 1024 * 1024,
            max_indent_depth: 100,
            ..Default::default()
        },
    };
    assert_eq!(options.reference_mode, ReferenceMode::Lenient);
    assert_eq!(options.limits.max_indent_depth, 100);
}

#[test]
fn test_limits_struct() {
    let limits = Limits {
        max_indent_depth: 50,
        max_file_size: 2048,
        ..Default::default()
    };
    assert_eq!(limits.max_indent_depth, 50);
    assert_eq!(limits.max_file_size, 2048);
}

#[test]
fn test_limits_default() {
    let limits = Limits::default();
    assert!(limits.max_file_size > 0);
    assert!(limits.max_indent_depth > 0);
    assert!(limits.max_nodes > 0);
}

#[test]
fn test_limits_unlimited() {
    let limits = Limits::unlimited();
    assert_eq!(limits.max_file_size, usize::MAX);
}

#[test]
fn test_reference_mode_enum() {
    let strict = ReferenceMode::Strict;
    let lenient = ReferenceMode::Lenient;
    assert_eq!(strict, ReferenceMode::Strict);
    assert_eq!(lenient, ReferenceMode::Lenient);
}

// =============================================================================
// Constants Re-exports
// =============================================================================

#[test]
fn test_supported_version_constant() {
    assert_eq!(SUPPORTED_VERSION, (1, 0));
}

#[test]
fn test_version_constant() {
    assert!(!VERSION.is_empty());
    // Should be a valid semver string
    let parts: Vec<&str> = VERSION.split('.').collect();
    assert!(parts.len() >= 2, "VERSION should be semver format");
}

#[test]
fn test_version_can_be_parsed() {
    // Verify VERSION is parseable as semver
    let parts: Vec<&str> = VERSION.split('.').collect();
    for part in parts.iter().take(3) {
        let clean = part.split('-').next().unwrap();
        let _: u32 = clean.parse().expect("VERSION should contain valid numbers");
    }
}

// =============================================================================
// Module Re-exports
// =============================================================================

#[test]
fn test_lex_module_accessible() {
    use hedl::lex;

    assert!(lex::is_valid_key_token("key"));
    assert!(lex::is_valid_type_name("User"));
    assert!(lex::is_valid_id_token("alice"));
}

#[test]
fn test_csv_module_accessible() {
    use hedl::csv;

    let fields = csv::parse_csv_row("a, b, c").unwrap();
    assert_eq!(fields.len(), 3);
}

#[test]
fn test_tensor_module_accessible() {
    use hedl::tensor;

    let t = tensor::parse_tensor("[1, 2, 3]").unwrap();
    assert_eq!(t.shape(), vec![3]);
}

#[test]
fn test_c14n_module_accessible() {
    use hedl::c14n;

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let canonical = c14n::canonicalize(&doc).unwrap();
    assert!(canonical.contains("%VERSION: 1.0"));
}

#[test]
fn test_json_module_accessible() {
    use hedl::json;

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let config = json::ToJsonConfig::default();
    let json_str = json::to_json(&doc, &config).unwrap();
    assert!(json_str.contains("\"key\""));
}

#[test]
fn test_lint_module_accessible() {
    use hedl::lint;

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let config = lint::LintConfig::default();
    let diagnostics = lint::lint_with_config(&doc, config);
    let _ = diagnostics;
}

// =============================================================================
// Integration: All Re-exports Together
// =============================================================================

#[test]
fn test_full_pipeline_with_reexports() {
    // Parse
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    assert_eq!(doc.version, SUPPORTED_VERSION);

    // Canonicalize
    let canonical = canonicalize(&doc).unwrap();
    assert!(!canonical.is_empty());

    // Convert to JSON
    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"key\""));

    // Convert from JSON
    let doc2 = from_json(&json).unwrap();
    assert_eq!(doc2.version, SUPPORTED_VERSION);

    // Lint
    let diagnostics = lint(&doc);
    let _ = diagnostics;

    // Validate
    assert!(validate("%VERSION: 1.0\n---\nkey: value").is_ok());
}

#[test]
fn test_all_convenience_functions() {
    let input = "%VERSION: 1.0\n---\nkey: value";

    // parse
    let doc = parse(input).unwrap();
    assert!(doc.root.contains_key("key"));

    // parse_lenient
    let doc_lenient = parse_lenient(input).unwrap();
    assert_eq!(doc.version, doc_lenient.version);

    // canonicalize
    let canonical = canonicalize(&doc).unwrap();
    assert!(canonical.contains("key:"));

    // to_json
    let json = to_json(&doc).unwrap();
    assert!(json.contains("\"key\""));

    // from_json
    let doc_from_json = from_json(&json).unwrap();
    assert!(doc_from_json.root.contains_key("key"));

    // lint
    let diagnostics = lint(&doc);
    let _ = diagnostics;

    // validate
    assert!(validate(input).is_ok());
    assert!(validate("invalid").is_err());
}
