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

//! Tests for feature-gated functionality.
//!
//! This test suite verifies that feature-gated modules (YAML, XML, CSV, Parquet, Neo4j, TOON)
//! are properly exposed and functional when their respective features are enabled.

#[allow(unused_imports)] // Document is used in feature-gated code
use hedl::{parse, Document};

// =============================================================================
// YAML Feature Tests
// =============================================================================

#[cfg(feature = "yaml")]
mod yaml_feature_tests {
    use super::*;
    use hedl::yaml::{
        from_yaml, hedl_to_yaml, to_yaml, yaml_to_hedl, FromYamlConfig, ToYamlConfig,
    };

    #[test]
    fn test_yaml_module_exists() {
        // Just verify the module is accessible
        let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
        let config = ToYamlConfig::default();
        let yaml = to_yaml(&doc, &config).unwrap();
        assert!(!yaml.is_empty());
    }

    #[test]
    fn test_to_yaml_basic() {
        let doc = parse("%VERSION: 1.0\n---\nname: Alice\nage: 30").unwrap();
        let config = ToYamlConfig::default();
        let yaml = to_yaml(&doc, &config).unwrap();
        assert!(yaml.contains("name:") || yaml.contains("age:"));
    }

    #[test]
    fn test_from_yaml_basic() {
        let yaml = "name: Alice\nage: 30";
        let config = FromYamlConfig::default();
        let doc = from_yaml(yaml, &config).unwrap();
        assert_eq!(doc.version, (1, 0));
    }

    #[test]
    fn test_hedl_to_yaml_conversion() {
        let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
        let yaml = hedl_to_yaml(&doc).unwrap();
        assert!(!yaml.is_empty());
    }

    #[test]
    fn test_yaml_to_hedl_conversion() {
        let yaml = "key: value";
        let doc = yaml_to_hedl(yaml).unwrap();
        assert_eq!(doc.version, (1, 0));
    }

    #[test]
    fn test_yaml_round_trip() {
        let original = parse("%VERSION: 1.0\n---\nname: Bob\nactive: true").unwrap();
        let config = ToYamlConfig::default();
        let yaml = to_yaml(&original, &config).unwrap();

        let from_config = FromYamlConfig::default();
        let restored = from_yaml(&yaml, &from_config).unwrap();
        assert_eq!(original.version, restored.version);
    }

    #[test]
    fn test_yaml_config_default() {
        let to_config = ToYamlConfig::default();
        let from_config = FromYamlConfig::default();
        let _ = to_config;
        let _ = from_config;
    }
}

// =============================================================================
// XML Feature Tests
// =============================================================================

#[cfg(feature = "xml")]
mod xml_feature_tests {
    use super::*;
    use hedl::xml::{from_xml, hedl_to_xml, to_xml, xml_to_hedl, FromXmlConfig, ToXmlConfig};

    #[test]
    fn test_xml_module_exists() {
        let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(!xml.is_empty());
    }

    #[test]
    fn test_to_xml_basic() {
        let doc = parse("%VERSION: 1.0\n---\nname: Alice").unwrap();
        let config = ToXmlConfig::default();
        let xml = to_xml(&doc, &config).unwrap();
        assert!(xml.contains("<name>") || xml.contains("name="));
    }

    #[test]
    fn test_from_xml_basic() {
        let xml = "<root><name>Alice</name></root>";
        let config = FromXmlConfig::default();
        let doc = from_xml(xml, &config).unwrap();
        assert_eq!(doc.version, (1, 0));
    }

    #[test]
    fn test_hedl_to_xml_conversion() {
        let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
        let xml = hedl_to_xml(&doc).unwrap();
        assert!(!xml.is_empty());
    }

    #[test]
    fn test_xml_to_hedl_conversion() {
        let xml = "<root><key>value</key></root>";
        let doc = xml_to_hedl(xml).unwrap();
        assert_eq!(doc.version, (1, 0));
    }

    #[test]
    fn test_xml_config_default() {
        let to_config = ToXmlConfig::default();
        let from_config = FromXmlConfig::default();
        let _ = to_config;
        let _ = from_config;
    }
}

// =============================================================================
// CSV Feature Tests
// =============================================================================

#[cfg(feature = "csv")]
mod csv_file_feature_tests {
    use super::*;
    use hedl::csv_file::{
        from_csv, from_csv_with_config, to_csv, to_csv_with_config, FromCsvConfig, ToCsvConfig,
    };
    use hedl::{Item, MatrixList, Node, Value};

    #[test]
    fn test_csv_file_module_exists() {
        let doc = create_test_doc();
        let csv = to_csv(&doc).unwrap();
        assert!(!csv.is_empty());
    }

    #[test]
    fn test_to_csv_basic() {
        let doc = create_test_doc();
        let csv = to_csv(&doc).unwrap();
        assert!(csv.contains("a") || csv.contains("b") || csv.contains("c"));
    }

    #[test]
    fn test_from_csv_basic() {
        // Schema is columns EXCLUDING id, so data needs id column + schema columns
        let csv = "id,a,b,c\nrow1,1,2,3\nrow2,4,5,6";
        let doc = from_csv(csv, "Row", &["a", "b", "c"]).unwrap();
        assert_eq!(doc.version, (1, 0));
    }

    #[test]
    fn test_to_csv_with_config() {
        let doc = create_test_doc();
        let config = ToCsvConfig::default();
        let csv = to_csv_with_config(&doc, config).unwrap();
        assert!(!csv.is_empty());
    }

    #[test]
    fn test_from_csv_with_config() {
        // Schema is columns EXCLUDING id, so data needs id column + schema columns
        let csv = "id,a,b,c\nrow1,1,2,3";
        let config = FromCsvConfig::default();
        let doc = from_csv_with_config(csv, "Row", &["a", "b", "c"], config).unwrap();
        assert_eq!(doc.version, (1, 0));
    }

    #[test]
    fn test_csv_config_default() {
        let to_config = ToCsvConfig::default();
        let from_config = FromCsvConfig::default();
        let _ = to_config;
        let _ = from_config;
    }

    fn create_test_doc() -> Document {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new(
            "Row",
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        list.add_row(Node::new(
            "Row",
            "1",
            vec![Value::Int(1), Value::Int(2), Value::Int(3)],
        ));
        list.add_row(Node::new(
            "Row",
            "2",
            vec![Value::Int(4), Value::Int(5), Value::Int(6)],
        ));
        doc.root.insert("data".to_string(), Item::List(list));
        doc
    }
}

// =============================================================================
// Parquet Feature Tests
// =============================================================================

#[cfg(feature = "parquet")]
mod parquet_feature_tests {
    use super::*;
    use hedl::parquet::{from_parquet_bytes, to_parquet_bytes, ToParquetConfig};
    use hedl::{Item, MatrixList, Node, Value};

    #[test]
    fn test_parquet_module_exists() {
        let doc = create_test_doc();
        let bytes = to_parquet_bytes(&doc).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_to_parquet_bytes() {
        let doc = create_test_doc();
        let bytes = to_parquet_bytes(&doc).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_from_parquet_bytes() {
        let doc = create_test_doc();
        let bytes = to_parquet_bytes(&doc).unwrap();
        let restored = from_parquet_bytes(&bytes).unwrap();
        assert_eq!(restored.version, (1, 0));
    }

    #[test]
    fn test_parquet_round_trip() {
        let original = create_test_doc();
        let bytes = to_parquet_bytes(&original).unwrap();
        let restored = from_parquet_bytes(&bytes).unwrap();
        assert_eq!(original.version, restored.version);
    }

    #[test]
    fn test_parquet_config_default() {
        let config = ToParquetConfig::default();
        let _ = config;
    }

    fn create_test_doc() -> Document {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Row", vec!["x".to_string(), "y".to_string()]);
        list.add_row(Node::new("Row", "1", vec![Value::Int(10), Value::Int(20)]));
        list.add_row(Node::new("Row", "2", vec![Value::Int(30), Value::Int(40)]));
        doc.root.insert("points".to_string(), Item::List(list));
        doc
    }
}

// =============================================================================
// Neo4j Feature Tests
// =============================================================================

#[cfg(feature = "neo4j")]
mod neo4j_feature_tests {
    use super::*;
    use hedl::neo4j::{
        hedl_to_cypher, to_cypher, to_cypher_statements, ObjectHandling, RelationshipNaming,
        StatementType, ToCypherConfig,
    };
    use hedl::{Item, MatrixList, Node, Value};

    /// Create a document with matrix list data suitable for Cypher conversion.
    /// Neo4j's to_cypher only processes Item::List items, not simple scalars.
    fn create_cypher_doc() -> Document {
        let mut doc = Document::new((1, 0));
        let mut list = MatrixList::new("Person", vec!["name".to_string(), "age".to_string()]);
        list.add_row(Node::new(
            "Person",
            "alice",
            vec![Value::String("Alice".to_string().into()), Value::Int(30)],
        ));
        list.add_row(Node::new(
            "Person",
            "bob",
            vec![Value::String("Bob".to_string().into()), Value::Int(25)],
        ));
        doc.root.insert("people".to_string(), Item::List(list));
        doc
    }

    #[test]
    fn test_neo4j_module_exists() {
        let doc = create_cypher_doc();
        let config = ToCypherConfig::default();
        let cypher = to_cypher(&doc, &config).unwrap();
        assert!(!cypher.is_empty());
    }

    #[test]
    fn test_to_cypher_basic() {
        let doc = create_cypher_doc();
        let config = ToCypherConfig::default();
        let cypher = to_cypher(&doc, &config).unwrap();
        assert!(!cypher.is_empty());
        assert!(cypher.contains("Person") || cypher.contains("MERGE") || cypher.contains("CREATE"));
    }

    #[test]
    fn test_to_cypher_statements() {
        let doc = create_cypher_doc();
        let config = ToCypherConfig::default();
        let statements = to_cypher_statements(&doc, &config).unwrap();
        assert!(!statements.is_empty());
    }

    #[test]
    fn test_hedl_to_cypher_conversion() {
        let doc = create_cypher_doc();
        let cypher = hedl_to_cypher(&doc).unwrap();
        assert!(!cypher.is_empty());
    }

    #[test]
    fn test_cypher_config_default() {
        let config = ToCypherConfig::default();
        let _ = config;
    }

    #[test]
    fn test_cypher_config_with_object_handling() {
        let config = ToCypherConfig {
            object_handling: ObjectHandling::JsonString,
            ..Default::default()
        };
        let _ = config;
    }

    #[test]
    fn test_statement_type_enum() {
        let constraint = StatementType::Constraint;
        let index = StatementType::Index;
        let create_node = StatementType::CreateNode;
        let _ = constraint;
        let _ = index;
        let _ = create_node;
    }

    #[test]
    fn test_object_handling_enum() {
        let flatten = ObjectHandling::Flatten;
        let json_string = ObjectHandling::JsonString;
        let _ = flatten;
        let _ = json_string;
    }

    #[test]
    fn test_relationship_naming_enum() {
        let property_name = RelationshipNaming::PropertyName;
        let generic = RelationshipNaming::Generic;
        let target_type = RelationshipNaming::TargetType;
        let _ = property_name;
        let _ = generic;
        let _ = target_type;
    }
}

// =============================================================================
// TOON Feature Tests
// =============================================================================

#[cfg(feature = "toon")]
mod toon_feature_tests {
    use super::*;
    use hedl::toon::{hedl_to_toon, to_toon, Delimiter, ToToonConfig, ToToonConfigBuilder};

    #[test]
    fn test_toon_module_exists() {
        let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
        let config = ToToonConfig::default();
        let toon = to_toon(&doc, &config).unwrap();
        assert!(!toon.is_empty());
    }

    #[test]
    fn test_to_toon_basic() {
        let doc = parse("%VERSION: 1.0\n---\nname: Alice").unwrap();
        let config = ToToonConfig::default();
        let toon = to_toon(&doc, &config).unwrap();
        assert!(!toon.is_empty());
    }

    #[test]
    fn test_hedl_to_toon_conversion() {
        let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
        let toon = hedl_to_toon(&doc).unwrap();
        assert!(!toon.is_empty());
    }

    #[test]
    fn test_toon_config_default() {
        let config = ToToonConfig::default();
        let _ = config;
    }

    #[test]
    fn test_toon_config_builder() {
        let config = ToToonConfigBuilder::default()
            .delimiter(Delimiter::Tab)
            .build();
        let _ = config;
    }

    #[test]
    fn test_delimiter_enum() {
        let tab = Delimiter::Tab;
        let pipe = Delimiter::Pipe;
        let comma = Delimiter::Comma;
        let _ = tab;
        let _ = pipe;
        let _ = comma;
    }
}

// =============================================================================
// Feature Combination Tests
// =============================================================================

#[test]
fn test_no_feature_gates_required_for_core() {
    // These should always work regardless of features
    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
    let _ = hedl::canonicalize(&doc).unwrap();
    let _ = hedl::to_json(&doc).unwrap();
    let _ = hedl::lint(&doc);
}

#[cfg(all(feature = "yaml", feature = "xml"))]
#[test]
fn test_yaml_xml_interop() {
    use hedl::xml;
    use hedl::yaml;

    let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();

    let yaml_config = yaml::ToYamlConfig::default();
    let yaml_out = yaml::to_yaml(&doc, &yaml_config).unwrap();

    let xml_config = xml::ToXmlConfig::default();
    let xml_out = xml::to_xml(&doc, &xml_config).unwrap();

    assert!(!yaml_out.is_empty());
    assert!(!xml_out.is_empty());
}

#[cfg(all(feature = "csv", feature = "parquet"))]
#[test]
fn test_csv_parquet_interop() {
    use hedl::{csv_file, parquet, Item, MatrixList, Node, Value};

    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Row", vec!["a".to_string(), "b".to_string()]);
    list.add_row(Node::new("Row", "1", vec![Value::Int(1), Value::Int(2)]));
    doc.root.insert("data".to_string(), Item::List(list));

    let csv = csv_file::to_csv(&doc).unwrap();
    let parquet_bytes = parquet::to_parquet_bytes(&doc).unwrap();

    assert!(!csv.is_empty());
    assert!(!parquet_bytes.is_empty());
}
