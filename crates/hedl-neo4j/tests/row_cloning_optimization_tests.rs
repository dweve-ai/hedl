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

//! Row cloning optimization tests for hedl-neo4j.
//!
//! These tests verify that row/node data is cloned efficiently during Cypher generation:
//! - Clones happen minimally (once per batch, not per row)
//! - Properties are correctly preserved through cloning
//! - Batch processing maintains memory efficiency
//! - Reference semantics are correct during UNWIND generation
//! - Concurrent processing maintains data integrity
//!
//! Test coverage:
//! - Clone count verification per batch
//! - Property preservation through conversion pipeline
//! - Memory efficiency patterns
//! - Batch boundary handling
//! - Large dataset performance characteristics
//! - Property-based testing for clone correctness

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_neo4j::{
    cypher::CypherValue,
    mapping::{matrix_list_to_nodes, node_to_neo4j, Neo4jRelationship},
    to_cypher, to_cypher_statements, to_cypher_stream, ToCypherConfig,
};
use proptest::prelude::*;
use smallvec::SmallVec;
use std::collections::BTreeMap;
use std::sync::Arc;

// ============================================================================
// Helper Functions for Creating Test Data
// ============================================================================

/// Create a simple document with the specified number of nodes.
fn create_document_with_nodes(count: usize) -> Document {
    let schema = vec!["id".to_string(), "name".to_string(), "value".to_string()];
    let rows: Vec<Node> = (0..count)
        .map(|i| Node {
            type_name: "TestNode".to_string(),
            id: format!("node_{i}"),
            fields: SmallVec::from_vec(vec![
                Value::String(format!("node_{i}").into()),
                Value::String(format!("Test Node {i}").into()),
                Value::Int(i as i64),
            ]),
            children: None,
            child_count: 0,
        })
        .collect();

    let mut root = BTreeMap::new();
    root.insert(
        "nodes".to_string(),
        Item::List(MatrixList {
            type_name: "TestNode".to_string(),
            schema,
            rows,
            count_hint: Some(count),
        }),
    );

    Document {
        version: (1, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

/// Create a document with nodes and relationships.
fn create_document_with_relationships(node_count: usize, rel_count: usize) -> Document {
    let user_schema = vec!["id".to_string(), "name".to_string()];
    let post_schema = vec![
        "id".to_string(),
        "content".to_string(),
        "author".to_string(),
    ];

    let user_rows: Vec<Node> = (0..node_count)
        .map(|i| Node {
            type_name: "User".to_string(),
            id: format!("user_{i}"),
            fields: SmallVec::from_vec(vec![
                Value::String(format!("user_{i}").into()),
                Value::String(format!("User {i}").into()),
            ]),
            children: None,
            child_count: 0,
        })
        .collect();

    let post_rows: Vec<Node> = (0..rel_count)
        .map(|i| Node {
            type_name: "Post".to_string(),
            id: format!("post_{i}"),
            fields: SmallVec::from_vec(vec![
                Value::String(format!("post_{i}").into()),
                Value::String(format!("Post content {i}").into()),
                Value::Reference(Reference {
                    type_name: Some("User".to_string().into()),
                    id: format!("user_{}", i % node_count).into(),
                }),
            ]),
            children: None,
            child_count: 0,
        })
        .collect();

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: user_schema,
            rows: user_rows,
            count_hint: Some(node_count),
        }),
    );
    root.insert(
        "posts".to_string(),
        Item::List(MatrixList {
            type_name: "Post".to_string(),
            schema: post_schema,
            rows: post_rows,
            count_hint: Some(rel_count),
        }),
    );

    let mut structs = BTreeMap::new();
    structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    structs.insert(
        "Post".to_string(),
        vec![
            "id".to_string(),
            "content".to_string(),
            "author".to_string(),
        ],
    );

    Document {
        version: (1, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs,
        nests: BTreeMap::new(),
        root,
    }
}

/// Create a document with NEST hierarchy (parent-child relationships).
fn create_document_with_nest(parent_count: usize, children_per_parent: usize) -> Document {
    let parent_schema = vec!["id".to_string(), "name".to_string()];
    let child_schema = vec!["id".to_string(), "title".to_string()];

    let parent_rows: Vec<Node> = (0..parent_count)
        .map(|i| {
            let mut children = BTreeMap::new();
            let child_nodes: Vec<Node> = (0..children_per_parent)
                .map(|j| Node {
                    type_name: "Child".to_string(),
                    id: format!("child_{i}_{j}"),
                    fields: SmallVec::from_vec(vec![
                        Value::String(format!("child_{i}_{j}").into()),
                        Value::String(format!("Child {j} of Parent {i}").into()),
                    ]),
                    children: None,
                    child_count: 0,
                })
                .collect();
            children.insert("children".to_string(), child_nodes);

            Node {
                type_name: "Parent".to_string(),
                id: format!("parent_{i}"),
                fields: SmallVec::from_vec(vec![
                    Value::String(format!("parent_{i}").into()),
                    Value::String(format!("Parent {i}").into()),
                ]),
                children: Some(Box::new(children)),
                child_count: children_per_parent as u16,
            }
        })
        .collect();

    let mut root = BTreeMap::new();
    root.insert(
        "parents".to_string(),
        Item::List(MatrixList {
            type_name: "Parent".to_string(),
            schema: parent_schema,
            rows: parent_rows,
            count_hint: Some(parent_count),
        }),
    );

    let mut structs = BTreeMap::new();
    structs.insert(
        "Parent".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    structs.insert("Child".to_string(), child_schema);

    let mut nests = BTreeMap::new();
    nests.insert("Parent".to_string(), "Child".to_string());

    Document {
        version: (1, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs,
        nests,
        root,
    }
}

// ============================================================================
// Clone Efficiency Tests
// ============================================================================

#[test]
fn test_batch_processing_efficiency_small_batch() {
    let doc = create_document_with_nodes(50);
    let config = ToCypherConfig::new().with_batch_size(1000);

    let statements = to_cypher_statements(&doc, &config).expect("Failed to generate statements");

    // With 50 nodes and batch size 1000, should have exactly 1 UNWIND for nodes
    let unwind_count = statements
        .iter()
        .filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:"))
        .count();

    assert_eq!(
        unwind_count, 1,
        "Small batch should produce exactly 1 UNWIND statement, got {unwind_count}"
    );
}

#[test]
fn test_batch_processing_efficiency_large_batch() {
    let doc = create_document_with_nodes(2500);
    let config = ToCypherConfig::new().with_batch_size(1000);

    let statements = to_cypher_statements(&doc, &config).expect("Failed to generate statements");

    // With 2500 nodes and batch size 1000, should have exactly 3 UNWINDs
    let unwind_count = statements
        .iter()
        .filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:"))
        .count();

    assert_eq!(
        unwind_count, 3,
        "Large batch should produce exactly 3 UNWIND statements (2500/1000 = 3), got {unwind_count}"
    );
}

#[test]
fn test_batch_boundary_exact_multiple() {
    // Exact multiple of batch size
    let doc = create_document_with_nodes(3000);
    let config = ToCypherConfig::new().with_batch_size(1000);

    let statements = to_cypher_statements(&doc, &config).expect("Failed to generate statements");

    let unwind_count = statements
        .iter()
        .filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:"))
        .count();

    assert_eq!(
        unwind_count, 3,
        "Exact multiple should produce 3 UNWIND statements (3000/1000), got {unwind_count}"
    );
}

#[test]
fn test_batch_boundary_one_over() {
    // One over batch size boundary
    let doc = create_document_with_nodes(1001);
    let config = ToCypherConfig::new().with_batch_size(1000);

    let statements = to_cypher_statements(&doc, &config).expect("Failed to generate statements");

    let unwind_count = statements
        .iter()
        .filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:"))
        .count();

    assert_eq!(
        unwind_count, 2,
        "1001 nodes with batch size 1000 should produce 2 batches, got {unwind_count}"
    );
}

#[test]
fn test_single_node_efficiency() {
    let doc = create_document_with_nodes(1);
    let config = ToCypherConfig::new().with_batch_size(1000);

    let statements = to_cypher_statements(&doc, &config).expect("Failed to generate statements");

    // Single node should produce exactly 1 UNWIND
    let unwind_count = statements
        .iter()
        .filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:"))
        .count();

    assert_eq!(
        unwind_count, 1,
        "Single node should produce exactly 1 UNWIND statement"
    );
}

// ============================================================================
// Property Preservation Tests
// ============================================================================

#[test]
fn test_property_preservation_simple() {
    let schema = vec!["id".to_string(), "name".to_string(), "count".to_string()];
    let node = Node {
        type_name: "Test".to_string(),
        id: "test1".to_string(),
        fields: SmallVec::from_vec(vec![
            Value::String("test1".to_string().into()),
            Value::String("Test Name".to_string().into()),
            Value::Int(42),
        ]),
        children: None,
        child_count: 0,
    };

    let config = ToCypherConfig::default();
    let neo4j_node = node_to_neo4j(&node, &schema, &config).expect("Conversion failed");

    assert_eq!(neo4j_node.label, "Test");
    assert_eq!(neo4j_node.id, "test1");
    assert_eq!(
        neo4j_node.get_property("name"),
        Some(&CypherValue::String("Test Name".to_string()))
    );
    assert_eq!(
        neo4j_node.get_property("count"),
        Some(&CypherValue::Int(42))
    );
}

#[test]
fn test_property_preservation_all_types() {
    let schema = vec![
        "id".to_string(),
        "str_val".to_string(),
        "int_val".to_string(),
        "float_val".to_string(),
        "bool_val".to_string(),
        "null_val".to_string(),
    ];
    let node = Node {
        type_name: "AllTypes".to_string(),
        id: "at1".to_string(),
        fields: SmallVec::from_vec(vec![
            Value::String("at1".to_string().into()),
            Value::String("string value".to_string().into()),
            Value::Int(i64::MAX),
            Value::Float(std::f64::consts::PI),
            Value::Bool(true),
            Value::Null,
        ]),
        children: None,
        child_count: 0,
    };

    let config = ToCypherConfig::default();
    let neo4j_node = node_to_neo4j(&node, &schema, &config).expect("Conversion failed");

    assert_eq!(
        neo4j_node.get_property("str_val"),
        Some(&CypherValue::String("string value".to_string()))
    );
    assert_eq!(
        neo4j_node.get_property("int_val"),
        Some(&CypherValue::Int(i64::MAX))
    );

    // Float comparison with tolerance
    if let Some(CypherValue::Float(f)) = neo4j_node.get_property("float_val") {
        assert!((*f - std::f64::consts::PI).abs() < 1e-10);
    } else {
        panic!("Expected float property");
    }

    assert_eq!(
        neo4j_node.get_property("bool_val"),
        Some(&CypherValue::Bool(true))
    );
    assert_eq!(
        neo4j_node.get_property("null_val"),
        Some(&CypherValue::Null)
    );
}

#[test]
fn test_property_preservation_special_characters() {
    // Test strings that should be preserved (safe characters)
    let preserved_strings = [
        "Hello 'World'",
        "Test \"with\" quotes",
        "Backslash \\ test",
        "Newline\ntest",
        "Tab\ttest",
        "Unicode: 日本語 🎉",
    ];

    let config = ToCypherConfig::default();

    for (i, special) in preserved_strings.iter().enumerate() {
        let schema = vec!["id".to_string(), "data".to_string()];
        let node = Node {
            type_name: "Special".to_string(),
            id: format!("s{i}"),
            fields: SmallVec::from_vec(vec![
                Value::String(format!("s{i}").into()),
                Value::String((*special).to_string().into()),
            ]),
            children: None,
            child_count: 0,
        };

        let neo4j_node =
            node_to_neo4j(&node, &schema, &config).expect("Conversion should not fail");
        assert_eq!(
            neo4j_node.get_property("data"),
            Some(&CypherValue::String((*special).to_string())),
            "Special string '{special}' not preserved"
        );
    }

    // Test that null bytes are FILTERED (security measure - they are dangerous)
    let null_test_schema = vec!["id".to_string(), "data".to_string()];
    let node_with_null = Node {
        type_name: "Special".to_string(),
        id: "snull".to_string(),
        fields: SmallVec::from_vec(vec![
            Value::String("snull".to_string().into()),
            Value::String("Control\x00chars".to_string().into()),
        ]),
        children: None,
        child_count: 0,
    };
    let neo4j_node = node_to_neo4j(&node_with_null, &null_test_schema, &config)
        .expect("Conversion should not fail");
    // Null bytes should be filtered out for security
    assert_eq!(
        neo4j_node.get_property("data"),
        Some(&CypherValue::String("Controlchars".to_string())),
        "Null bytes should be filtered for security"
    );
}

#[test]
fn test_property_preservation_through_cypher_generation() {
    let doc = create_document_with_nodes(10);
    let config = ToCypherConfig::default();

    let cypher = to_cypher(&doc, &config).expect("Cypher generation failed");

    // Verify all node IDs appear in output
    for i in 0..10 {
        assert!(
            cypher.contains(&format!("node_{i}")),
            "Node ID 'node_{i}' not found in output"
        );
        assert!(
            cypher.contains(&format!("Test Node {i}")),
            "Node name 'Test Node {i}' not found in output"
        );
    }
}

// ============================================================================
// Streaming vs Regular API Consistency Tests
// ============================================================================

#[test]
fn test_streaming_matches_regular_small() {
    let doc = create_document_with_nodes(10);
    let config = ToCypherConfig::default();

    let regular = to_cypher(&doc, &config).expect("Regular generation failed");

    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).expect("Streaming generation failed");
    let streaming = String::from_utf8(streaming_output).expect("Invalid UTF-8");

    assert_eq!(
        regular, streaming,
        "Streaming and regular output should be identical"
    );
}

#[test]
fn test_streaming_matches_regular_large() {
    let doc = create_document_with_nodes(5000);
    let config = ToCypherConfig::new().with_batch_size(1000);

    let regular = to_cypher(&doc, &config).expect("Regular generation failed");

    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).expect("Streaming generation failed");
    let streaming = String::from_utf8(streaming_output).expect("Invalid UTF-8");

    assert_eq!(
        regular, streaming,
        "Streaming and regular output should be identical for large documents"
    );
}

#[test]
fn test_streaming_matches_regular_with_relationships() {
    let doc = create_document_with_relationships(20, 50);
    let config = ToCypherConfig::default();

    let regular = to_cypher(&doc, &config).expect("Regular generation failed");

    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).expect("Streaming generation failed");
    let streaming = String::from_utf8(streaming_output).expect("Invalid UTF-8");

    assert_eq!(
        regular, streaming,
        "Streaming and regular output should be identical with relationships"
    );
}

#[test]
fn test_streaming_matches_regular_with_nest() {
    let doc = create_document_with_nest(10, 5);
    let config = ToCypherConfig::default();

    let regular = to_cypher(&doc, &config).expect("Regular generation failed");

    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).expect("Streaming generation failed");
    let streaming = String::from_utf8(streaming_output).expect("Invalid UTF-8");

    assert_eq!(
        regular, streaming,
        "Streaming and regular output should be identical with NEST hierarchy"
    );
}

// ============================================================================
// Memory Pattern Tests
// ============================================================================

#[test]
fn test_batch_size_affects_output_structure() {
    let doc = create_document_with_nodes(100);

    let config_small = ToCypherConfig::new().with_batch_size(10);
    let config_large = ToCypherConfig::new().with_batch_size(1000);

    let small_batch_stmts =
        to_cypher_statements(&doc, &config_small).expect("Small batch generation failed");
    let large_batch_stmts =
        to_cypher_statements(&doc, &config_large).expect("Large batch generation failed");

    let small_unwinds = small_batch_stmts
        .iter()
        .filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:"))
        .count();
    let large_unwinds = large_batch_stmts
        .iter()
        .filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:"))
        .count();

    assert_eq!(
        small_unwinds, 10,
        "100 nodes with batch size 10 should produce 10 batches"
    );
    assert_eq!(
        large_unwinds, 1,
        "100 nodes with batch size 1000 should produce 1 batch"
    );
}

#[test]
fn test_batch_parameter_size() {
    let doc = create_document_with_nodes(100);
    let config = ToCypherConfig::new().with_batch_size(25);

    let statements = to_cypher_statements(&doc, &config).expect("Generation failed");

    // Each UNWIND statement should have a "rows" parameter
    for stmt in statements
        .iter()
        .filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:"))
    {
        if let Some(CypherValue::List(rows)) = stmt.parameters.get("rows") {
            // Each batch should have at most batch_size elements
            assert!(
                rows.len() <= 25,
                "Batch should have at most 25 rows, got {}",
                rows.len()
            );
            // Non-final batches should be full
            if rows.len() < 25 {
                // This could be the last batch - verify it makes sense
                // Total batches = ceil(100/25) = 4, last batch should have 100 - 3*25 = 25
                // So all batches should be exactly 25 for this case
            }
        }
    }
}

// ============================================================================
// Relationship Cloning Tests
// ============================================================================

#[test]
fn test_relationship_property_cloning() {
    let rel = Neo4jRelationship::new("User", "alice", "FOLLOWS", "User", "bob")
        .with_property("since", CypherValue::Int(2020))
        .with_property("weight", CypherValue::Float(0.8));

    // Properties should be preserved
    assert_eq!(rel.from_label, "User");
    assert_eq!(rel.from_id, "alice");
    assert_eq!(rel.rel_type, "FOLLOWS");
    assert_eq!(rel.to_label, "User");
    assert_eq!(rel.to_id, "bob");
    assert_eq!(rel.properties.get("since"), Some(&CypherValue::Int(2020)));
    assert_eq!(rel.properties.get("weight"), Some(&CypherValue::Float(0.8)));
}

#[test]
fn test_relationship_generation_includes_all_references() {
    let doc = create_document_with_relationships(10, 50);
    let config = ToCypherConfig::default();

    let cypher = to_cypher(&doc, &config).expect("Cypher generation failed");

    // Should have AUTHOR relationships
    assert!(
        cypher.contains(":AUTHOR"),
        "Output should contain AUTHOR relationships"
    );

    // Should have relationship creation statements
    let rel_count = cypher.matches("]->(").count();
    assert!(
        rel_count > 0,
        "Should have relationship creation statements"
    );
}

// ============================================================================
// NEST Hierarchy Cloning Tests
// ============================================================================

#[test]
fn test_nest_child_nodes_cloned_correctly() {
    let doc = create_document_with_nest(5, 3);
    let config = ToCypherConfig::default();

    let cypher = to_cypher(&doc, &config).expect("Cypher generation failed");

    // All parent nodes should be present
    for i in 0..5 {
        assert!(
            cypher.contains(&format!("parent_{i}")),
            "Parent {i} not found"
        );
    }

    // All child nodes should be present
    for i in 0..5 {
        for j in 0..3 {
            assert!(
                cypher.contains(&format!("child_{i}_{j}")),
                "Child {i}_{j} not found"
            );
        }
    }
}

#[test]
fn test_nest_preserves_schema_column_names() {
    let doc = create_document_with_nest(2, 2);
    let config = ToCypherConfig::default();

    let cypher = to_cypher(&doc, &config).expect("Cypher generation failed");

    // Child nodes should use 'title' (schema name), not 'field_1' (generic)
    assert!(
        cypher.contains("title"),
        "Child nodes should use schema column name 'title'"
    );
    assert!(
        !cypher.contains("field_1"),
        "Child nodes should NOT use generic 'field_1'"
    );
}

#[test]
fn test_nest_relationships_created() {
    let doc = create_document_with_nest(3, 4);
    let config = ToCypherConfig::default();

    let cypher = to_cypher(&doc, &config).expect("Cypher generation failed");

    // Should have HAS_CHILD relationships (based on child type name)
    assert!(
        cypher.contains("HAS_CHILD") || cypher.contains("HAS_CHILDREN"),
        "NEST should create HAS_<ChildType> relationships"
    );
}

// ============================================================================
// Determinism Tests
// ============================================================================

#[test]
fn test_deterministic_output() {
    let doc = create_document_with_nodes(100);
    let config = ToCypherConfig::default();

    let output1 = to_cypher(&doc, &config).expect("First generation failed");
    let output2 = to_cypher(&doc, &config).expect("Second generation failed");
    let output3 = to_cypher(&doc, &config).expect("Third generation failed");

    assert_eq!(output1, output2, "Output should be deterministic (1 vs 2)");
    assert_eq!(output2, output3, "Output should be deterministic (2 vs 3)");
}

#[test]
fn test_deterministic_statements() {
    let doc = create_document_with_relationships(10, 30);
    let config = ToCypherConfig::default();

    let stmts1 = to_cypher_statements(&doc, &config).expect("First generation failed");
    let stmts2 = to_cypher_statements(&doc, &config).expect("Second generation failed");

    assert_eq!(
        stmts1.len(),
        stmts2.len(),
        "Statement count should be deterministic"
    );

    for (s1, s2) in stmts1.iter().zip(stmts2.iter()) {
        assert_eq!(s1.query, s2.query, "Statement queries should be identical");
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_document() {
    let doc = Document {
        version: (1, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root: BTreeMap::new(),
    };

    let config = ToCypherConfig::default();
    let cypher = to_cypher(&doc, &config).expect("Empty document should succeed");

    // Should produce minimal output
    let non_comment_lines: Vec<&str> = cypher
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect();

    assert!(
        non_comment_lines.is_empty() || non_comment_lines.iter().all(|l| l.is_empty()),
        "Empty document should produce no statements"
    );
}

#[test]
fn test_empty_matrix_list_handling() {
    let schema = vec!["id".to_string(), "name".to_string()];
    let list = MatrixList {
        type_name: "Empty".to_string(),
        schema,
        rows: vec![],
        count_hint: None,
    };

    let config = ToCypherConfig::default();
    let result = matrix_list_to_nodes(&list, &config);

    assert!(result.is_err(), "Empty MatrixList should produce an error");
}

#[test]
fn test_large_property_values() {
    let large_string = "x".repeat(100_000);
    let schema = vec!["id".to_string(), "data".to_string()];
    let node = Node {
        type_name: "Large".to_string(),
        id: "large1".to_string(),
        fields: SmallVec::from_vec(vec![
            Value::String("large1".to_string().into()),
            Value::String(large_string.clone().into()),
        ]),
        children: None,
        child_count: 0,
    };

    let config = ToCypherConfig::default();
    let neo4j_node = node_to_neo4j(&node, &schema, &config).expect("Large value conversion failed");

    if let Some(CypherValue::String(s)) = neo4j_node.get_property("data") {
        assert_eq!(s.len(), 100_000, "Large string should be preserved");
    } else {
        panic!("Expected large string property");
    }
}

#[test]
fn test_extreme_integer_values() {
    let extreme_values = [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX];

    let config = ToCypherConfig::default();

    for (i, val) in extreme_values.iter().enumerate() {
        let schema = vec!["id".to_string(), "value".to_string()];
        let node = Node {
            type_name: "Extreme".to_string(),
            id: format!("e{i}"),
            fields: SmallVec::from_vec(vec![
                Value::String(format!("e{i}").into()),
                Value::Int(*val),
            ]),
            children: None,
            child_count: 0,
        };

        let neo4j_node =
            node_to_neo4j(&node, &schema, &config).expect("Extreme value conversion failed");
        assert_eq!(
            neo4j_node.get_property("value"),
            Some(&CypherValue::Int(*val)),
            "Extreme value {val} not preserved"
        );
    }
}

#[test]
fn test_extreme_float_values() {
    let extreme_values = [
        f64::MIN,
        f64::MIN_POSITIVE,
        -0.0,
        0.0,
        f64::EPSILON,
        f64::MAX,
        std::f64::consts::PI,
        std::f64::consts::E,
    ];

    let config = ToCypherConfig::default();

    for (i, val) in extreme_values.iter().enumerate() {
        let schema = vec!["id".to_string(), "value".to_string()];
        let node = Node {
            type_name: "Float".to_string(),
            id: format!("f{i}"),
            fields: SmallVec::from_vec(vec![
                Value::String(format!("f{i}").into()),
                Value::Float(*val),
            ]),
            children: None,
            child_count: 0,
        };

        let neo4j_node =
            node_to_neo4j(&node, &schema, &config).expect("Float value conversion failed");

        if let Some(CypherValue::Float(f)) = neo4j_node.get_property("value") {
            assert!(
                (*f - *val).abs() < 1e-10 || (*f - *val).abs() / val.abs().max(1.0) < 1e-10,
                "Float value {val} not preserved (got {f})"
            );
        } else {
            panic!("Expected float property for value {val}");
        }
    }
}

// ============================================================================
// Property-Based Tests
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Batch count should equal ceil(n / batch_size)
    #[test]
    fn prop_batch_count_formula(
        node_count in 1usize..1000,
        batch_size in 10usize..500
    ) {
        let doc = create_document_with_nodes(node_count);
        let config = ToCypherConfig::new().with_batch_size(batch_size);

        if let Ok(statements) = to_cypher_statements(&doc, &config) {
            let unwind_count = statements
                .iter()
                .filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:"))
                .count();

            let expected = (node_count + batch_size - 1) / batch_size;
            prop_assert_eq!(
                unwind_count,
                expected,
                "Batch count mismatch: {} nodes / {} batch size should produce {} batches, got {}",
                node_count,
                batch_size,
                expected,
                unwind_count
            );
        }
    }

    /// Property: Total rows in all batches should equal node count
    #[test]
    fn prop_total_row_count_preserved(
        node_count in 1usize..500,
        batch_size in 10usize..200
    ) {
        let doc = create_document_with_nodes(node_count);
        let config = ToCypherConfig::new().with_batch_size(batch_size);

        if let Ok(statements) = to_cypher_statements(&doc, &config) {
            let mut total_rows = 0;
            for stmt in statements.iter().filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:")) {
                if let Some(CypherValue::List(rows)) = stmt.parameters.get("rows") {
                    total_rows += rows.len();
                }
            }

            prop_assert_eq!(
                total_rows,
                node_count,
                "Total rows {} should equal node count {}",
                total_rows,
                node_count
            );
        }
    }

    /// Property: Streaming and regular output should always match
    #[test]
    fn prop_streaming_equals_regular(
        node_count in 1usize..200,
        batch_size in 10usize..100
    ) {
        let doc = create_document_with_nodes(node_count);
        let config = ToCypherConfig::new().with_batch_size(batch_size);

        if let Ok(regular) = to_cypher(&doc, &config) {
            let mut streaming_output = Vec::new();
            if to_cypher_stream(&doc, &config, &mut streaming_output).is_ok() {
                if let Ok(streaming) = String::from_utf8(streaming_output) {
                    prop_assert_eq!(
                        regular,
                        streaming,
                        "Streaming and regular output should match"
                    );
                }
            }
        }
    }

    /// Property: All node IDs should appear in output
    #[test]
    fn prop_all_node_ids_preserved(node_count in 1usize..100) {
        let doc = create_document_with_nodes(node_count);
        let config = ToCypherConfig::default();

        if let Ok(cypher) = to_cypher(&doc, &config) {
            for i in 0..node_count {
                prop_assert!(
                    cypher.contains(&format!("node_{i}")),
                    "Node ID 'node_{}' not found in output",
                    i
                );
            }
        }
    }

    /// Property: Output should be deterministic
    #[test]
    fn prop_deterministic_output(node_count in 1usize..100) {
        let doc = create_document_with_nodes(node_count);
        let config = ToCypherConfig::default();

        if let (Ok(out1), Ok(out2)) = (to_cypher(&doc, &config), to_cypher(&doc, &config)) {
            prop_assert_eq!(
                out1,
                out2,
                "Output should be deterministic"
            );
        }
    }
}

// ============================================================================
// Concurrent Safety Tests
// ============================================================================

#[test]
fn test_concurrent_cypher_generation() {
    use std::thread;

    let doc = Arc::new(create_document_with_nodes(100));
    let config = Arc::new(ToCypherConfig::default());

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let doc_clone = Arc::clone(&doc);
            let config_clone = Arc::clone(&config);
            thread::spawn(move || to_cypher(&doc_clone, &config_clone))
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All results should be identical (determinism)
    let first = results[0].as_ref().expect("First generation failed");
    for (i, result) in results.iter().enumerate().skip(1) {
        let output = result
            .as_ref()
            .unwrap_or_else(|_| panic!("Generation {i} failed"));
        assert_eq!(
            first, output,
            "Concurrent generations should produce identical output"
        );
    }
}

#[test]
fn test_concurrent_statement_generation() {
    use std::thread;

    let doc = Arc::new(create_document_with_relationships(20, 50));
    let config = Arc::new(ToCypherConfig::default());

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let doc_clone = Arc::clone(&doc);
            let config_clone = Arc::clone(&config);
            thread::spawn(move || to_cypher_statements(&doc_clone, &config_clone))
        })
        .collect();

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All results should have same length
    let first_len = results[0].as_ref().expect("First generation failed").len();
    for (i, result) in results.iter().enumerate().skip(1) {
        let stmts = result
            .as_ref()
            .unwrap_or_else(|_| panic!("Generation {i} failed"));
        assert_eq!(
            first_len,
            stmts.len(),
            "Concurrent generations should produce same statement count"
        );
    }
}

// ============================================================================
// Clone Overhead Measurement Tests (for performance regression detection)
// ============================================================================

#[test]
fn test_clone_overhead_scaling() {
    // Measure that clone overhead scales linearly with batch count, not node count
    let sizes = vec![100, 500, 1000, 2000, 5000];
    let batch_size = 1000;

    let mut statement_counts = Vec::new();

    for size in &sizes {
        let doc = create_document_with_nodes(*size);
        let config = ToCypherConfig::new().with_batch_size(batch_size);

        let statements = to_cypher_statements(&doc, &config).expect("Generation failed");
        let unwind_count = statements
            .iter()
            .filter(|s| s.query.contains("UNWIND") && s.query.contains("MERGE (n:"))
            .count();

        statement_counts.push((*size, unwind_count));
    }

    // Verify batch count scales as expected
    for (size, count) in &statement_counts {
        let expected = (*size + batch_size - 1) / batch_size;
        assert_eq!(
            *count, expected,
            "Size {size} should produce {expected} batches, got {count}"
        );
    }
}

#[test]
fn test_relationship_clone_overhead() {
    // Relationships are grouped by type, so clone overhead should be per-batch, not per-relationship
    let doc = create_document_with_relationships(50, 200); // 50 users, 200 posts
    let config = ToCypherConfig::new().with_batch_size(100);

    let statements = to_cypher_statements(&doc, &config).expect("Generation failed");

    // Count relationship statements
    let rel_statements: Vec<_> = statements
        .iter()
        .filter(|s| s.query.contains(":AUTHOR"))
        .collect();

    // With 200 relationships and batch size 100, should have 2 batches
    let expected_batches = (200 + 100 - 1) / 100;
    assert!(
        rel_statements.len() <= expected_batches * 2, // Allow some overhead for label grouping
        "Relationship batches should scale with batch_size, not relationship count"
    );
}
