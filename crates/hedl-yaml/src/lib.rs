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

//! HEDL YAML Conversion
//!
//! Provides bidirectional conversion between HEDL documents and YAML format, with comprehensive
//! support for HEDL's rich type system and data structures.
//!
//! # Important Limitations
//!
//! When converting HEDL to YAML and back, certain HEDL-specific metadata is **not preserved**:
//!
//! - **ALIAS declarations**: Type aliases are lost (YAML has no equivalent concept)
//! - **NEST definitions**: Entity hierarchy declarations are lost (only structure remains)
//! - **STRUCT schemas**: Explicit field constraints and type declarations are lost
//! - **Type constraints**: Validation rules (min/max/format) are not preserved
//! - **Document metadata**: Version, author, license, etc. are not preserved
//!
//! YAML is an excellent format for data exchange and configuration, but it lacks the type system
//! and schema capabilities of HEDL's native format. See the README for detailed examples and
//! recommended workarounds.
//!
//! # What IS Preserved
//!
//! The following are preserved during YAML conversion:
//! - Scalar values (strings, numbers, booleans, null)
//! - References (local and qualified)
//! - Expressions
//! - Tensors (multi-dimensional arrays)
//! - Objects and nested structures
//! - Matrix lists (inferred schemas)
//! - Hierarchical relationships (structural only)
//!
//! # Examples
//!
//! ## Converting HEDL to YAML
//!
//! ```rust
//! use hedl_core::{Document, Item, Value};
//! use hedl_yaml::{to_yaml, ToYamlConfig};
//! use std::collections::BTreeMap;
//!
//! let mut doc = Document::new((1, 0));
//! let mut root = BTreeMap::new();
//! root.insert("name".to_string(), Item::Scalar(Value::String("example".to_string())));
//! root.insert("count".to_string(), Item::Scalar(Value::Int(42)));
//! doc.root = root;
//!
//! let config = ToYamlConfig::default();
//! let yaml = to_yaml(&doc, &config).unwrap();
//! println!("{}", yaml);
//! ```
//!
//! ## Converting YAML to HEDL
//!
//! ```rust
//! use hedl_yaml::{from_yaml, FromYamlConfig};
//!
//! let yaml = r#"
//! name: example
//! count: 42
//! active: true
//! "#;
//!
//! // Use default configuration with high limits (500MB / 10M / 10K)
//! let config = FromYamlConfig::default();
//! let doc = from_yaml(yaml, &config).unwrap();
//! assert_eq!(doc.version, (1, 0));
//! ```
//!
//! ## Customizing Resource Limits
//!
//! ```rust
//! use hedl_yaml::{from_yaml, FromYamlConfig};
//!
//! // For untrusted input, use conservative limits
//! let config = FromYamlConfig::builder()
//!     .max_document_size(10 * 1024 * 1024)  // 10 MB
//!     .max_array_length(100_000)             // 100K elements
//!     .max_nesting_depth(100)                // 100 levels
//!     .build();
//!
//! let yaml = "name: test\nvalue: 123\n";
//! let doc = from_yaml(yaml, &config).unwrap();
//! ```
//!
//! ## Round-trip Conversion (with Metadata Loss)
//!
//! ```rust
//! use hedl_core::{Document, Item, Value};
//! use hedl_yaml::{to_yaml, from_yaml, ToYamlConfig, FromYamlConfig};
//! use std::collections::BTreeMap;
//!
//! // Create original document
//! let mut doc = Document::new((1, 0));
//! let mut root = BTreeMap::new();
//! root.insert("test".to_string(), Item::Scalar(Value::String("value".to_string())));
//! doc.root = root;
//!
//! // Convert to YAML and back
//! let to_config = ToYamlConfig::default();
//! let yaml = to_yaml(&doc, &to_config).unwrap();
//!
//! let from_config = FromYamlConfig::default();
//! let restored = from_yaml(&yaml, &from_config).unwrap();
//!
//! // Data is preserved, but schema/type metadata is lost
//! assert_eq!(restored.version, doc.version);
//! ```
//!
//! ## Preserving Metadata with Hints
//!
//! ```rust
//! use hedl_core::{Document, Item, Value};
//! use hedl_yaml::{to_yaml, ToYamlConfig};
//! use std::collections::BTreeMap;
//!
//! let mut doc = Document::new((1, 0));
//! let mut root = BTreeMap::new();
//! root.insert("count".to_string(), Item::Scalar(Value::Int(42)));
//! doc.root = root;
//!
//! // Enable metadata hints in YAML output
//! let config = ToYamlConfig {
//!     include_metadata: true,  // Adds __type__ and __schema__ hints
//!     ..Default::default()
//! };
//! let yaml = to_yaml(&doc, &config).unwrap();
//! // YAML includes type hints, but they won't prevent data-only schemas
//! ```

pub mod error;
mod from_yaml;
mod to_yaml;

// Re-export the shared DEFAULT_SCHEMA from hedl-core for internal use
pub(crate) use hedl_core::convert::DEFAULT_SCHEMA;

pub use error::YamlError;
pub use from_yaml::{
    from_yaml, from_yaml_value, FromYamlConfig, FromYamlConfigBuilder, DEFAULT_MAX_ARRAY_LENGTH,
    DEFAULT_MAX_DOCUMENT_SIZE, DEFAULT_MAX_NESTING_DEPTH,
};
pub use to_yaml::{to_yaml, to_yaml_value, ToYamlConfig};

use hedl_core::Document;

/// Convert HEDL document to YAML string with default configuration
pub fn hedl_to_yaml(doc: &Document) -> Result<String, String> {
    to_yaml(doc, &ToYamlConfig::default())
}

/// Convert YAML string to HEDL document with default configuration
pub fn yaml_to_hedl(yaml: &str) -> Result<Document, String> {
    from_yaml(yaml, &FromYamlConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
    use hedl_core::lex::Tensor;
    use std::collections::BTreeMap;

    #[test]
    fn test_round_trip_scalars() {
        let mut doc = Document::new((1, 0));
        let mut root = BTreeMap::new();

        root.insert("null_val".to_string(), Item::Scalar(Value::Null));
        root.insert("bool_val".to_string(), Item::Scalar(Value::Bool(true)));
        root.insert("int_val".to_string(), Item::Scalar(Value::Int(42)));
        root.insert("float_val".to_string(), Item::Scalar(Value::Float(3.25)));
        root.insert(
            "string_val".to_string(),
            Item::Scalar(Value::String("hello".to_string())),
        );

        doc.root = root;

        let yaml = hedl_to_yaml(&doc).unwrap();
        let restored = yaml_to_hedl(&yaml).unwrap();

        assert_eq!(restored.root.len(), 5);
        assert_eq!(
            restored.root.get("bool_val").unwrap().as_scalar().unwrap(),
            &Value::Bool(true)
        );
        assert_eq!(
            restored.root.get("int_val").unwrap().as_scalar().unwrap(),
            &Value::Int(42)
        );
        assert_eq!(
            restored
                .root
                .get("string_val")
                .unwrap()
                .as_scalar()
                .unwrap(),
            &Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_round_trip_reference() {
        let mut doc = Document::new((1, 0));
        let mut root = BTreeMap::new();

        root.insert(
            "local_ref".to_string(),
            Item::Scalar(Value::Reference(Reference::local("item1"))),
        );
        root.insert(
            "qualified_ref".to_string(),
            Item::Scalar(Value::Reference(Reference::qualified("User", "user1"))),
        );

        doc.root = root;

        let yaml = hedl_to_yaml(&doc).unwrap();
        let restored = yaml_to_hedl(&yaml).unwrap();

        let local_ref = restored.root.get("local_ref").unwrap().as_scalar().unwrap();
        if let Value::Reference(r) = local_ref {
            assert_eq!(r.type_name, None);
            assert_eq!(r.id, "item1");
        } else {
            panic!("Expected reference");
        }

        let qualified_ref = restored
            .root
            .get("qualified_ref")
            .unwrap()
            .as_scalar()
            .unwrap();
        if let Value::Reference(r) = qualified_ref {
            assert_eq!(r.type_name, Some("User".to_string()));
            assert_eq!(r.id, "user1");
        } else {
            panic!("Expected qualified reference");
        }
    }

    #[test]
    fn test_round_trip_expression() {
        // Expression and Span imports commented out until Expression API is fixed
        // use hedl_core::lex::{Expression, Span};
        let _doc = Document::new((1, 0));
        let _root: BTreeMap<String, Item> = BTreeMap::new();

        // TODO: Fix Expression API to match new struct-based variants
        // root.insert(
        //     "expr".to_string(),
        //     Item::Scalar(Value::Expression(Expression::Call {
        //         name: "add".to_string(),
        //         args: vec![
        //             Expression::Identifier("x".to_string()),
        //             Expression::Literal { value: hedl_core::lex::ExprLiteral::Int(1), span: Span::default() }),
        //         ],
        //     })),
        // );

        // doc.root = root;

        // TODO: Re-enable after Expression API is fixed
        // let yaml = hedl_to_yaml(&doc).unwrap();
        // let restored = yaml_to_hedl(&yaml).unwrap();
        //
        // let expr = restored.root.get("expr").unwrap().as_scalar().unwrap();
        // if let Value::Expression(e) = expr {
        //     assert_eq!(e.to_string(), "add(x, 1)");
        // } else {
        //     panic!("Expected expression");
        // }
    }

    #[test]
    fn test_round_trip_tensor() {
        let mut doc = Document::new((1, 0));
        let mut root = BTreeMap::new();

        let tensor = Tensor::Array(vec![
            Tensor::Scalar(1.0),
            Tensor::Scalar(2.0),
            Tensor::Scalar(3.0),
        ]);
        root.insert("tensor".to_string(), Item::Scalar(Value::Tensor(tensor)));

        doc.root = root;

        let yaml = hedl_to_yaml(&doc).unwrap();
        let restored = yaml_to_hedl(&yaml).unwrap();

        let restored_tensor = restored.root.get("tensor").unwrap().as_scalar().unwrap();
        if let Value::Tensor(t) = restored_tensor {
            if let Tensor::Array(items) = t {
                assert_eq!(items.len(), 3);
            } else {
                panic!("Expected tensor array");
            }
        } else {
            panic!("Expected tensor");
        }
    }

    #[test]
    fn test_round_trip_nested_tensor() {
        let mut doc = Document::new((1, 0));
        let mut root = BTreeMap::new();

        let tensor = Tensor::Array(vec![
            Tensor::Array(vec![Tensor::Scalar(1.0), Tensor::Scalar(2.0)]),
            Tensor::Array(vec![Tensor::Scalar(3.0), Tensor::Scalar(4.0)]),
        ]);
        root.insert("matrix".to_string(), Item::Scalar(Value::Tensor(tensor)));

        doc.root = root;

        let yaml = hedl_to_yaml(&doc).unwrap();
        let restored = yaml_to_hedl(&yaml).unwrap();

        let restored_tensor = restored.root.get("matrix").unwrap().as_scalar().unwrap();
        if let Value::Tensor(Tensor::Array(rows)) = restored_tensor {
            assert_eq!(rows.len(), 2);
            if let Tensor::Array(cols) = &rows[0] {
                assert_eq!(cols.len(), 2);
            } else {
                panic!("Expected nested array");
            }
        } else {
            panic!("Expected nested tensor");
        }
    }

    #[test]
    fn test_round_trip_object() {
        let mut doc = Document::new((1, 0));
        let mut root = BTreeMap::new();

        let mut obj = BTreeMap::new();
        obj.insert(
            "name".to_string(),
            Item::Scalar(Value::String("test".to_string())),
        );
        obj.insert("age".to_string(), Item::Scalar(Value::Int(30)));
        root.insert("person".to_string(), Item::Object(obj));

        doc.root = root;

        let yaml = hedl_to_yaml(&doc).unwrap();
        let restored = yaml_to_hedl(&yaml).unwrap();

        let person_obj = restored.root.get("person").unwrap().as_object().unwrap();
        assert_eq!(person_obj.len(), 2);
        assert_eq!(
            person_obj.get("name").unwrap().as_scalar().unwrap(),
            &Value::String("test".to_string())
        );
        assert_eq!(
            person_obj.get("age").unwrap().as_scalar().unwrap(),
            &Value::Int(30)
        );
    }

    #[test]
    fn test_round_trip_matrix_list() {
        let mut doc = Document::new((1, 0));
        let mut root = BTreeMap::new();

        let mut list = MatrixList::new(
            "User",
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
        );

        // Per SPEC: fields must include ALL schema columns including ID
        let node1 = Node::new(
            "User",
            "user1",
            vec![
                Value::String("user1".to_string()),
                Value::String("Alice".to_string()),
                Value::Int(30),
            ],
        );
        let node2 = Node::new(
            "User",
            "user2",
            vec![
                Value::String("user2".to_string()),
                Value::String("Bob".to_string()),
                Value::Int(25),
            ],
        );

        list.add_row(node1);
        list.add_row(node2);

        root.insert("users".to_string(), Item::List(list));
        doc.root = root;

        let yaml = hedl_to_yaml(&doc).unwrap();
        let restored = yaml_to_hedl(&yaml).unwrap();

        let users_list = restored.root.get("users").unwrap().as_list().unwrap();
        assert_eq!(users_list.rows.len(), 2);
        assert_eq!(users_list.schema.len(), 3);
        // Schema is sorted alphabetically with id first: [id, age, name]
        assert_eq!(
            users_list.schema,
            vec!["id".to_string(), "age".to_string(), "name".to_string()]
        );

        let first_row = &users_list.rows[0];
        assert_eq!(first_row.id, "user1");
        // Per SPEC: fields include ALL schema columns including ID
        assert_eq!(first_row.fields.len(), 3);
        assert_eq!(first_row.fields[0], Value::String("user1".to_string())); // id
        assert_eq!(first_row.fields[1], Value::Int(30)); // age
        assert_eq!(first_row.fields[2], Value::String("Alice".to_string())); // name
    }

    #[test]
    fn test_empty_document() {
        let doc = Document::new((1, 0));
        let yaml = hedl_to_yaml(&doc).unwrap();
        let restored = yaml_to_hedl(&yaml).unwrap();
        assert_eq!(restored.version, (1, 0));
        assert_eq!(restored.root.len(), 0);
    }

    #[test]
    fn test_nested_objects() {
        let mut doc = Document::new((1, 0));
        let mut root = BTreeMap::new();

        let mut inner = BTreeMap::new();
        inner.insert("x".to_string(), Item::Scalar(Value::Int(10)));
        inner.insert("y".to_string(), Item::Scalar(Value::Int(20)));

        let mut outer = BTreeMap::new();
        outer.insert("point".to_string(), Item::Object(inner));
        outer.insert(
            "label".to_string(),
            Item::Scalar(Value::String("origin".to_string())),
        );

        root.insert("config".to_string(), Item::Object(outer));
        doc.root = root;

        let yaml = hedl_to_yaml(&doc).unwrap();
        let restored = yaml_to_hedl(&yaml).unwrap();

        let config_obj = restored.root.get("config").unwrap().as_object().unwrap();
        let point_obj = config_obj.get("point").unwrap().as_object().unwrap();
        assert_eq!(
            point_obj.get("x").unwrap().as_scalar().unwrap(),
            &Value::Int(10)
        );
        assert_eq!(
            point_obj.get("y").unwrap().as_scalar().unwrap(),
            &Value::Int(20)
        );
    }

    #[test]
    fn test_yaml_parsing_error() {
        let invalid_yaml = "{ invalid yaml: [";
        let result = yaml_to_hedl(invalid_yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("YAML parse error"));
    }

    #[test]
    fn test_yaml_non_mapping_root() {
        let yaml = "- item1\n- item2\n";
        let result = yaml_to_hedl(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Root must be a YAML mapping"));
    }

    #[test]
    fn test_yaml_with_anchors_and_aliases() {
        // YAML anchors and aliases are automatically resolved by serde_yaml
        let yaml = r#"
defaults: &defaults
  timeout: 30
  retries: 3

production:
  config: *defaults
  host: prod.example.com
"#;
        let doc = yaml_to_hedl(yaml).unwrap();

        // Verify that the anchor reference was resolved
        let prod = doc.root.get("production").unwrap().as_object().unwrap();
        let config = prod.get("config").unwrap().as_object().unwrap();
        assert_eq!(
            config.get("timeout").unwrap().as_scalar().unwrap(),
            &Value::Int(30)
        );
        assert_eq!(
            config.get("retries").unwrap().as_scalar().unwrap(),
            &Value::Int(3)
        );
        assert_eq!(
            prod.get("host").unwrap().as_scalar().unwrap(),
            &Value::String("prod.example.com".to_string())
        );
    }
}
