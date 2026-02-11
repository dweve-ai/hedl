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

use super::*;
use crate::cypher::StatementType;
use crate::error::Neo4jError;
use hedl_core::{Document, Item, MatrixList, Node, Value};
use smallvec::SmallVec;

fn make_simple_doc() -> Document {
    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                Node {
                    type_name: "User".to_string(),
                    id: "alice".to_string(),
                    fields: SmallVec::from_vec(vec![
                        Value::String("alice".to_string().into()),
                        Value::String("Alice Smith".to_string().into()),
                    ]),
                    children: None,
                    child_count: 0,
                },
                Node {
                    type_name: "User".to_string(),
                    id: "bob".to_string(),
                    fields: SmallVec::from_vec(vec![
                        Value::String("bob".to_string().into()),
                        Value::String("Bob Jones".to_string().into()),
                    ]),
                    children: None,
                    child_count: 0,
                },
            ],
            count_hint: None,
        }),
    );

    Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

#[test]
fn test_hedl_to_cypher_simple() {
    let doc = make_simple_doc();
    let result = hedl_to_cypher(&doc).unwrap();

    assert!(result.contains("CREATE CONSTRAINT"));
    assert!(result.contains(":User"));
    assert!(result.contains("UNWIND"));
    assert!(result.contains("MERGE"));
}

#[test]
fn test_to_cypher_with_create() {
    let doc = make_simple_doc();
    let config = ToCypherConfig::new().with_create();
    let result = to_cypher(&doc, &config).unwrap();

    assert!(result.contains("CREATE (n:User"));
    assert!(!result.contains("MERGE (n:User"));
}

#[test]
fn test_to_cypher_without_constraints() {
    let doc = make_simple_doc();
    let config = ToCypherConfig::new().without_constraints();
    let result = to_cypher(&doc, &config).unwrap();

    assert!(!result.contains("CREATE CONSTRAINT"));
}

#[test]
fn test_to_cypher_custom_id_property() {
    let doc = make_simple_doc();
    let config = ToCypherConfig::new().with_id_property("nodeId");
    let result = to_cypher(&doc, &config).unwrap();

    assert!(result.contains("nodeId"));
}

#[test]
fn test_to_cypher_statements() {
    let doc = make_simple_doc();
    let config = ToCypherConfig::default();
    let statements = to_cypher_statements(&doc, &config).unwrap();

    assert!(!statements.is_empty());

    // Should have at least a constraint and a node creation
    let has_constraint = statements
        .iter()
        .any(|s| s.statement_type == StatementType::Constraint);
    let has_node = statements
        .iter()
        .any(|s| s.statement_type == StatementType::CreateNode);

    assert!(has_constraint);
    assert!(has_node);
}

#[test]
fn test_to_cypher_with_references() {
    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("alice".to_string().into()),
                    Value::String("Alice".to_string().into()),
                ]),
                children: None,
                child_count: 0,
            }],
            count_hint: None,
        }),
    );
    root.insert(
        "posts".to_string(),
        Item::List(MatrixList {
            type_name: "Post".to_string(),
            schema: vec![
                "id".to_string(),
                "content".to_string(),
                "author".to_string(),
            ],
            rows: vec![Node {
                type_name: "Post".to_string(),
                id: "p1".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("p1".to_string().into()),
                    Value::String("Hello World".to_string().into()),
                    Value::Reference(hedl_core::Reference {
                        type_name: Some("User".to_string().into()),
                        id: "alice".to_string().into(),
                    }),
                ]),
                children: None,
                child_count: 0,
            }],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let result = hedl_to_cypher(&doc).unwrap();

    assert!(result.contains(":Post"));
    assert!(result.contains(":User"));
    assert!(result.contains(":AUTHOR")); // Relationship type
}

#[test]
fn test_node_to_cypher_inline() {
    let node = crate::mapping::Neo4jNode::new("User", "alice").with_property("name", "Alice Smith");
    let config = ToCypherConfig::default();

    let cypher = node_to_cypher_inline(&node, &config);

    assert!(cypher.contains("MERGE"));
    assert!(cypher.contains(":User"));
    assert!(cypher.contains("_hedl_id: 'alice'"));
    assert!(cypher.contains("name: 'Alice Smith'"));
}

#[test]
fn test_generate_constraints() {
    let mut node_types = BTreeMap::new();
    node_types.insert("User".to_string(), vec!["id".to_string()]);
    node_types.insert("Post".to_string(), vec!["id".to_string()]);

    let config = ToCypherConfig::default();
    let constraints = constraints::generate_constraints(&node_types, &config).unwrap();

    assert_eq!(constraints.len(), 2);
    assert!(constraints.iter().any(|c| c.query.contains(":User")));
    assert!(constraints.iter().any(|c| c.query.contains(":Post")));
}

#[test]
fn test_child_nodes_use_schema_column_names() {
    // Create a document with NEST hierarchy
    let mut alice_children = BTreeMap::new();
    alice_children.insert(
        "posts".to_string(),
        vec![Node {
            type_name: "Post".to_string(),
            id: "post1".to_string(),
            fields: SmallVec::from_vec(vec![
                Value::String("post1".to_string().into()),
                Value::String("First post".to_string().into()),
            ]),
            children: None,
            child_count: 0,
        }],
    );

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("alice".to_string().into()),
                    Value::String("Alice".to_string().into()),
                ]),
                children: Some(Box::new(alice_children)),
                child_count: 0,
            }],
            count_hint: None,
        }),
    );

    let mut structs = BTreeMap::new();
    structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    structs.insert(
        "Post".to_string(),
        vec!["id".to_string(), "title".to_string()],
    );

    let mut nests = BTreeMap::new();
    nests.insert("User".to_string(), vec!["Post".to_string()]);

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs,
        nests,
        root,
    };

    let cypher = to_cypher(&doc, &ToCypherConfig::default()).unwrap();

    // Verify that child Post nodes use 'title' property, not 'field_1'
    assert!(
        cypher.contains("title"),
        "Generated Cypher should contain 'title' property"
    );
    assert!(
        !cypher.contains("field_1"),
        "Generated Cypher should NOT contain 'field_1' property"
    );

    // Also verify the actual value is mapped
    assert!(
        cypher.contains("First post"),
        "Generated Cypher should contain the post title value"
    );
}

#[test]
fn test_streaming_api_basic() {
    let doc = make_simple_doc();
    let config = ToCypherConfig::default();

    // Generate using streaming API
    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
    let streaming_result = String::from_utf8(streaming_output).unwrap();

    // Generate using regular API
    let regular_result = to_cypher(&doc, &config).unwrap();

    // Both should produce identical output
    assert_eq!(streaming_result, regular_result);
}

#[test]
fn test_streaming_api_with_references() {
    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("alice".to_string().into()),
                    Value::String("Alice".to_string().into()),
                ]),
                children: None,
                child_count: 0,
            }],
            count_hint: None,
        }),
    );
    root.insert(
        "posts".to_string(),
        Item::List(MatrixList {
            type_name: "Post".to_string(),
            schema: vec![
                "id".to_string(),
                "content".to_string(),
                "author".to_string(),
            ],
            rows: vec![Node {
                type_name: "Post".to_string(),
                id: "p1".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("p1".to_string().into()),
                    Value::String("Hello World".to_string().into()),
                    Value::Reference(hedl_core::Reference {
                        type_name: Some("User".to_string().into()),
                        id: "alice".to_string().into(),
                    }),
                ]),
                children: None,
                child_count: 0,
            }],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let config = ToCypherConfig::default();

    // Generate using streaming API
    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
    let streaming_result = String::from_utf8(streaming_output).unwrap();

    // Generate using regular API
    let regular_result = to_cypher(&doc, &config).unwrap();

    // Both should produce identical output
    assert_eq!(streaming_result, regular_result);

    // Verify relationships are present
    assert!(streaming_result.contains(":AUTHOR"));
}

#[test]
fn test_streaming_api_with_nest() {
    let mut alice_children = BTreeMap::new();
    alice_children.insert(
        "posts".to_string(),
        vec![Node {
            type_name: "Post".to_string(),
            id: "post1".to_string(),
            fields: SmallVec::from_vec(vec![
                Value::String("post1".to_string().into()),
                Value::String("First post".to_string().into()),
            ]),
            children: None,
            child_count: 0,
        }],
    );

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("alice".to_string().into()),
                    Value::String("Alice".to_string().into()),
                ]),
                children: Some(Box::new(alice_children)),
                child_count: 0,
            }],
            count_hint: None,
        }),
    );

    let mut structs = BTreeMap::new();
    structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    structs.insert(
        "Post".to_string(),
        vec!["id".to_string(), "title".to_string()],
    );

    let mut nests = BTreeMap::new();
    nests.insert("User".to_string(), vec!["Post".to_string()]);

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs,
        nests,
        root,
    };

    let config = ToCypherConfig::default();

    // Generate using streaming API
    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
    let streaming_result = String::from_utf8(streaming_output).unwrap();

    // Generate using regular API
    let regular_result = to_cypher(&doc, &config).unwrap();

    // Both should produce identical output
    assert_eq!(streaming_result, regular_result);

    // Verify child nodes use schema column names
    assert!(streaming_result.contains("title"));
    assert!(!streaming_result.contains("field_1"));
}

#[test]
fn test_streaming_api_without_constraints() {
    let doc = make_simple_doc();
    let config = ToCypherConfig::new().without_constraints();

    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
    let streaming_result = String::from_utf8(streaming_output).unwrap();

    let regular_result = to_cypher(&doc, &config).unwrap();

    assert_eq!(streaming_result, regular_result);
    assert!(!streaming_result.contains("CREATE CONSTRAINT"));
}

#[test]
fn test_streaming_api_without_comments() {
    let doc = make_simple_doc();
    let config = ToCypherConfig::new().without_comments();

    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
    let streaming_result = String::from_utf8(streaming_output).unwrap();

    let regular_result = to_cypher(&doc, &config).unwrap();

    assert_eq!(streaming_result, regular_result);
    assert!(!streaming_result.contains("//"));
}

#[test]
fn test_streaming_api_with_create() {
    let doc = make_simple_doc();
    let config = ToCypherConfig::new().with_create();

    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
    let streaming_result = String::from_utf8(streaming_output).unwrap();

    let regular_result = to_cypher(&doc, &config).unwrap();

    assert_eq!(streaming_result, regular_result);
    assert!(streaming_result.contains("CREATE (n:User"));
    assert!(!streaming_result.contains("MERGE (n:User"));
}

#[test]
fn test_streaming_api_custom_batch_size() {
    let doc = make_simple_doc();
    let config = ToCypherConfig::new().with_batch_size(1);

    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
    let streaming_result = String::from_utf8(streaming_output).unwrap();

    let regular_result = to_cypher(&doc, &config).unwrap();

    assert_eq!(streaming_result, regular_result);
}

#[test]
fn test_streaming_api_empty_document() {
    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root: BTreeMap::new(),
    };

    let config = ToCypherConfig::default();

    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
    let streaming_result = String::from_utf8(streaming_output).unwrap();

    let regular_result = to_cypher(&doc, &config).unwrap();

    assert_eq!(streaming_result, regular_result);
}

#[test]
fn test_streaming_api_large_document() {
    // Create a document with many nodes to test batching
    let mut rows = Vec::new();
    for i in 0..5000 {
        rows.push(Node {
            type_name: "User".to_string(),
            id: format!("user{i}"),
            fields: SmallVec::from_vec(vec![
                Value::String(format!("user{i}").into()),
                Value::String(format!("User {i}").into()),
            ]),
            children: None,
            child_count: 0,
        });
    }

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows,
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let config = ToCypherConfig::new().with_batch_size(1000);

    // Generate using streaming API
    let mut streaming_output = Vec::new();
    to_cypher_stream(&doc, &config, &mut streaming_output).unwrap();
    let streaming_result = String::from_utf8(streaming_output).unwrap();

    // Generate using regular API
    let regular_result = to_cypher(&doc, &config).unwrap();

    // Both should produce identical output
    assert_eq!(streaming_result, regular_result);

    // Verify we have multiple batches
    // With 5000 nodes and batch_size 1000, we should have exactly 5 batches
    let batch_count = streaming_result.matches("UNWIND").count();
    assert_eq!(
        batch_count, 5,
        "Expected 5 batches for 5000 nodes with batch_size 1000"
    );
}

#[test]
fn test_deep_nested_children_use_schema_column_names() {
    // Create a 3-level NEST hierarchy: Organization > Department > Employee
    let mut dept_children = BTreeMap::new();
    dept_children.insert(
        "employees".to_string(),
        vec![Node {
            type_name: "Employee".to_string(),
            id: "emp1".to_string(),
            fields: SmallVec::from_vec(vec![
                Value::String("emp1".to_string().into()),
                Value::String("John".to_string().into()),
                Value::String("Engineer".to_string().into()),
            ]),
            children: None,
            child_count: 0,
        }],
    );

    let mut org_children = BTreeMap::new();
    org_children.insert(
        "departments".to_string(),
        vec![Node {
            type_name: "Department".to_string(),
            id: "eng".to_string(),
            fields: SmallVec::from_vec(vec![
                Value::String("eng".to_string().into()),
                Value::String("Engineering".to_string().into()),
            ]),
            children: Some(Box::new(dept_children)),
            child_count: 0,
        }],
    );

    let mut root = BTreeMap::new();
    root.insert(
        "organizations".to_string(),
        Item::List(MatrixList {
            type_name: "Organization".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![Node {
                type_name: "Organization".to_string(),
                id: "acme".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("acme".to_string().into()),
                    Value::String("ACME Corp".to_string().into()),
                ]),
                children: Some(Box::new(org_children)),
                child_count: 0,
            }],
            count_hint: None,
        }),
    );

    let mut structs = BTreeMap::new();
    structs.insert(
        "Organization".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    structs.insert(
        "Department".to_string(),
        vec!["id".to_string(), "dept_name".to_string()],
    );
    structs.insert(
        "Employee".to_string(),
        vec!["id".to_string(), "emp_name".to_string(), "role".to_string()],
    );

    let mut nests = BTreeMap::new();
    nests.insert("Organization".to_string(), vec!["Department".to_string()]);
    nests.insert("Department".to_string(), vec!["Employee".to_string()]);

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs,
        nests,
        root,
    };

    let cypher = to_cypher(&doc, &ToCypherConfig::default()).unwrap();

    // Verify Department uses 'dept_name' not 'field_1'
    assert!(
        cypher.contains("dept_name"),
        "Department should use 'dept_name' property"
    );

    // Verify Employee uses 'emp_name' and 'role' not 'field_1' and 'field_2'
    assert!(
        cypher.contains("emp_name"),
        "Employee should use 'emp_name' property"
    );
    assert!(
        cypher.contains("role"),
        "Employee should use 'role' property"
    );

    // Verify no generic field names
    assert!(!cypher.contains("field_1"), "Should not contain 'field_1'");
    assert!(!cypher.contains("field_2"), "Should not contain 'field_2'");

    // Verify actual values
    assert!(
        cypher.contains("Engineering"),
        "Should contain department name"
    );
    assert!(cypher.contains("John"), "Should contain employee name");
    assert!(cypher.contains("Engineer"), "Should contain employee role");
}

// SECURITY TESTS: Node count limit enforcement

#[test]
fn test_max_nodes_limit_enforced() {
    // Create document with 100 nodes
    let mut rows = Vec::new();
    for i in 0..100 {
        rows.push(Node {
            type_name: "User".to_string(),
            id: format!("user{i}"),
            fields: SmallVec::from_vec(vec![Value::String(format!("user{i}").into())]),
            children: None,
            child_count: 0,
        });
    }

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string()],
            rows,
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Set limit to 50 nodes
    let config = ToCypherConfig::builder().max_nodes(50).build();

    let result = to_cypher_statements(&doc, &config);

    assert!(result.is_err());
    match result.unwrap_err() {
        Neo4jError::NodeCountExceeded { count, max_count } => {
            assert_eq!(count, 100);
            assert_eq!(max_count, 50);
        }
        other => panic!("Expected NodeCountExceeded, got {other:?}"),
    }
}

#[test]
fn test_max_nodes_limit_exactly_at_boundary() {
    // Create document with exactly 50 nodes
    let mut rows = Vec::new();
    for i in 0..50 {
        rows.push(Node {
            type_name: "User".to_string(),
            id: format!("user{i}"),
            fields: SmallVec::from_vec(vec![Value::String(format!("user{i}").into())]),
            children: None,
            child_count: 0,
        });
    }

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string()],
            rows,
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Set limit to exactly 50 nodes (should succeed)
    let config = ToCypherConfig::builder().max_nodes(50).build();

    let result = to_cypher_statements(&doc, &config);
    assert!(result.is_ok());
}

#[test]
fn test_max_nodes_counts_nest_children() {
    // Create parent with 10 children
    let mut parent_children = BTreeMap::new();
    let mut child_nodes = Vec::new();
    for i in 0..10 {
        child_nodes.push(Node {
            type_name: "Post".to_string(),
            id: format!("post{i}"),
            fields: SmallVec::from_vec(vec![Value::String(format!("post{i}").into())]),
            children: None,
            child_count: 0,
        });
    }
    parent_children.insert("posts".to_string(), child_nodes);

    let parent = Node {
        type_name: "User".to_string(),
        id: "alice".to_string(),
        fields: SmallVec::from_vec(vec![Value::String("alice".to_string().into())]),
        children: Some(Box::new(parent_children)),
        child_count: 0,
    };

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string()],
            rows: vec![parent],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Total nodes: 1 parent + 10 children = 11
    // Set limit to 5, should fail
    let config = ToCypherConfig::builder().max_nodes(5).build();

    let result = to_cypher_statements(&doc, &config);
    assert!(result.is_err());

    match result.unwrap_err() {
        Neo4jError::NodeCountExceeded { count, max_count } => {
            assert_eq!(count, 11); // 1 + 10
            assert_eq!(max_count, 5);
        }
        other => panic!("Expected NodeCountExceeded, got {other:?}"),
    }
}

#[test]
fn test_max_nodes_counts_deeply_nested() {
    // Create 3-level hierarchy: Org -> Dept -> Employee
    // 1 org, 2 depts, 3 employees each = 1 + 2 + 6 = 9 nodes

    fn make_employees(count: usize) -> Vec<Node> {
        (0..count)
            .map(|i| Node {
                type_name: "Employee".to_string(),
                id: format!("emp{i}"),
                fields: SmallVec::from_vec(vec![Value::String(format!("emp{i}").into())]),
                children: None,
                child_count: 0,
            })
            .collect()
    }

    fn make_dept(id: &str, emp_count: usize) -> Node {
        let mut children = BTreeMap::new();
        children.insert("employees".to_string(), make_employees(emp_count));
        Node {
            type_name: "Department".to_string(),
            id: id.to_string(),
            fields: SmallVec::from_vec(vec![Value::String(id.to_string().into())]),
            children: Some(Box::new(children)),
            child_count: 0,
        }
    }

    let mut org_children = BTreeMap::new();
    org_children.insert(
        "departments".to_string(),
        vec![make_dept("dept1", 3), make_dept("dept2", 3)],
    );

    let org = Node {
        type_name: "Organization".to_string(),
        id: "org1".to_string(),
        fields: SmallVec::from_vec(vec![Value::String("org1".to_string().into())]),
        children: Some(Box::new(org_children)),
        child_count: 0,
    };

    let mut root = BTreeMap::new();
    root.insert(
        "organizations".to_string(),
        Item::List(MatrixList {
            type_name: "Organization".to_string(),
            schema: vec!["id".to_string()],
            rows: vec![org],
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Total: 1 org + 2 depts + 6 employees = 9 nodes
    let config = ToCypherConfig::builder().max_nodes(5).build();

    let result = to_cypher_statements(&doc, &config);
    assert!(result.is_err());

    if let Err(Neo4jError::NodeCountExceeded { count, .. }) = result {
        assert_eq!(count, 9);
    } else {
        panic!("Expected NodeCountExceeded error");
    }
}

#[test]
fn test_max_nodes_no_limit_by_default() {
    // Default config has no limit
    let config = ToCypherConfig::default();
    assert!(config.max_nodes.is_none());

    // Large document should succeed with default config
    let mut rows = Vec::new();
    for i in 0..10000 {
        rows.push(Node {
            type_name: "User".to_string(),
            id: format!("user{i}"),
            fields: SmallVec::from_vec(vec![Value::String(format!("user{i}").into())]),
            children: None,
            child_count: 0,
        });
    }

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string()],
            rows,
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let result = to_cypher_statements(&doc, &config);
    assert!(result.is_ok());
}

#[test]
fn test_for_untrusted_input_has_limit() {
    let config = ToCypherConfig::for_untrusted_input();
    assert_eq!(config.max_nodes, Some(100_000));
}

#[test]
fn test_max_nodes_streaming_api_enforced() {
    // Same test for streaming API
    let mut rows = Vec::new();
    for i in 0..100 {
        rows.push(Node {
            type_name: "User".to_string(),
            id: format!("user{i}"),
            fields: SmallVec::from_vec(vec![Value::String(format!("user{i}").into())]),
            children: None,
            child_count: 0,
        });
    }

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string()],
            rows,
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let config = ToCypherConfig::builder().max_nodes(50).build();

    let mut output = Vec::new();
    let result = to_cypher_stream(&doc, &config, &mut output);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        Neo4jError::NodeCountExceeded { .. }
    ));
}

#[test]
fn test_max_nodes_multiple_matrix_lists() {
    // Multiple lists should have combined count checked
    let mut root = BTreeMap::new();

    // Add 30 users
    let user_rows: Vec<Node> = (0..30)
        .map(|i| Node {
            type_name: "User".to_string(),
            id: format!("user{i}"),
            fields: SmallVec::from_vec(vec![Value::String(format!("user{i}").into())]),
            children: None,
            child_count: 0,
        })
        .collect();

    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string()],
            rows: user_rows,
            count_hint: None,
        }),
    );

    // Add 30 posts
    let post_rows: Vec<Node> = (0..30)
        .map(|i| Node {
            type_name: "Post".to_string(),
            id: format!("post{i}"),
            fields: SmallVec::from_vec(vec![Value::String(format!("post{i}").into())]),
            children: None,
            child_count: 0,
        })
        .collect();

    root.insert(
        "posts".to_string(),
        Item::List(MatrixList {
            type_name: "Post".to_string(),
            schema: vec!["id".to_string()],
            rows: post_rows,
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // Total: 30 + 30 = 60 nodes
    // Limit of 50 should fail
    let config = ToCypherConfig::builder().max_nodes(50).build();

    let result = to_cypher_statements(&doc, &config);
    assert!(result.is_err());

    if let Err(Neo4jError::NodeCountExceeded { count, .. }) = result {
        assert_eq!(count, 60);
    } else {
        panic!("Expected NodeCountExceeded error");
    }
}

// SECURITY TESTS: DoS protection

#[test]
fn test_security_dos_protection() {
    // Verify that untrusted config provides actual protection
    let config = ToCypherConfig::for_untrusted_input();

    // Create document exceeding limit
    let mut rows = Vec::new();
    for i in 0..100_001 {
        // One more than limit
        rows.push(Node {
            type_name: "User".to_string(),
            id: format!("user{i}"),
            fields: SmallVec::from_vec(vec![Value::String(format!("user{i}").into())]),
            children: None,
            child_count: 0,
        });
    }

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string()],
            rows,
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    // MUST fail for untrusted config
    let result = to_cypher_statements(&doc, &config);
    assert!(
        result.is_err(),
        "for_untrusted_input() MUST reject documents over 100K nodes"
    );
}

#[test]
fn test_security_error_message_is_clear() {
    let config = ToCypherConfig::builder().max_nodes(10).build();

    let rows: Vec<Node> = (0..20)
        .map(|i| Node {
            type_name: "User".to_string(),
            id: format!("user{i}"),
            fields: SmallVec::from_vec(vec![Value::String(format!("user{i}").into())]),
            children: None,
            child_count: 0,
        })
        .collect();

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec!["id".to_string()],
            rows,
            count_hint: None,
        }),
    );

    let doc = Document {
        version: (2, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    };

    let err = to_cypher_statements(&doc, &config).unwrap_err();
    let msg = err.to_string();

    // Error message should contain both counts for debugging
    assert!(msg.contains("20"), "Error should mention actual count");
    assert!(msg.contains("10"), "Error should mention limit");
}
