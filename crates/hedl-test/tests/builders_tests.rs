// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Comprehensive tests for builder patterns.

use hedl_core::{Item, Node, Value};
use hedl_test::fixtures::builders::*;

#[test]
fn test_document_builder_default() {
    let builder = DocumentBuilder::default();
    let doc = builder.build();

    assert_eq!(doc.version, (1, 2));
    assert!(doc.root.is_empty());
}

#[test]
fn test_document_builder_with_version() {
    let doc = DocumentBuilder::new().version(2, 5).build();

    assert_eq!(doc.version, (2, 5));
}

#[test]
fn test_document_builder_with_scalars() {
    let doc = DocumentBuilder::new()
        .scalar("name", ValueBuilder::string("Alice"))
        .scalar("age", ValueBuilder::int(30))
        .scalar("active", ValueBuilder::bool_val(true))
        .scalar("score", ValueBuilder::float(95.5))
        .scalar("optional", ValueBuilder::null())
        .build();

    assert_eq!(doc.root.len(), 5);
    assert!(doc.root.contains_key("name"));
    assert!(doc.root.contains_key("age"));
    assert!(doc.root.contains_key("active"));
    assert!(doc.root.contains_key("score"));
    assert!(doc.root.contains_key("optional"));
}

#[test]
fn test_document_builder_with_alias() {
    let doc = DocumentBuilder::new()
        .alias("u", "users")
        .alias("p", "posts")
        .build();

    assert_eq!(doc.aliases.len(), 2);
    assert_eq!(doc.aliases.get("u"), Some(&"users".to_string()));
    assert_eq!(doc.aliases.get("p"), Some(&"posts".to_string()));
}

#[test]
fn test_document_builder_with_struct_def() {
    let doc = DocumentBuilder::new()
        .struct_def("User", vec!["id".to_string(), "name".to_string()])
        .struct_def(
            "Post",
            vec!["id".to_string(), "title".to_string(), "author".to_string()],
        )
        .build();

    assert_eq!(doc.structs.len(), 2);
    assert_eq!(
        doc.structs.get("User"),
        Some(&vec!["id".to_string(), "name".to_string()])
    );
}

#[test]
fn test_document_builder_with_nest() {
    let doc = DocumentBuilder::new()
        .nest("User", "Post")
        .nest("Post", "Comment")
        .build();

    assert_eq!(doc.nests.len(), 2);
    assert_eq!(doc.nests.get("User"), Some(&vec!["Post".to_string()]));
    assert_eq!(doc.nests.get("Post"), Some(&vec!["Comment".to_string()]));
}

#[test]
fn test_document_builder_with_list() {
    let list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string(), "name".to_string()])
        .build();

    let doc = DocumentBuilder::new().list("users", list).build();

    assert!(doc.root.contains_key("users"));
    if let Some(Item::List(l)) = doc.root.get("users") {
        assert_eq!(l.type_name, "User");
    } else {
        panic!("Expected list");
    }
}

#[test]
fn test_document_builder_chaining() {
    let doc = DocumentBuilder::new()
        .version(1, 2)
        .alias("u", "users")
        .struct_def("User", vec!["id".to_string(), "name".to_string()])
        .nest("User", "Post")
        .scalar("config", ValueBuilder::string("production"))
        .build();

    assert_eq!(doc.version, (1, 2));
    assert_eq!(doc.aliases.len(), 1);
    assert_eq!(doc.structs.len(), 1);
    assert_eq!(doc.nests.len(), 1);
    assert_eq!(doc.root.len(), 1);
}

#[test]
fn test_matrix_list_builder_basic() {
    let list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string(), "name".to_string()])
        .build();

    assert_eq!(list.type_name, "User");
    assert_eq!(list.schema, vec!["id", "name"]);
    assert!(list.rows.is_empty());
}

#[test]
fn test_matrix_list_builder_with_field() {
    let list = MatrixListBuilder::new("User")
        .field("id")
        .field("name")
        .field("email")
        .build();

    assert_eq!(list.schema.len(), 3);
    assert_eq!(list.schema, vec!["id", "name", "email"]);
}

#[test]
fn test_matrix_list_builder_with_rows() {
    let node1 = Node::new("User", "alice", vec![ValueBuilder::string("alice")]);
    let node2 = Node::new("User", "bob", vec![ValueBuilder::string("bob")]);

    let list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string()])
        .row(node1)
        .row(node2)
        .build();

    assert_eq!(list.rows.len(), 2);
    assert_eq!(list.rows[0].id, "alice");
    assert_eq!(list.rows[1].id, "bob");
}

#[test]
fn test_matrix_list_builder_with_rows_batch() {
    let nodes = vec![
        Node::new("User", "alice", vec![ValueBuilder::string("alice")]),
        Node::new("User", "bob", vec![ValueBuilder::string("bob")]),
        Node::new("User", "charlie", vec![ValueBuilder::string("charlie")]),
    ];

    let list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string()])
        .rows(nodes)
        .build();

    assert_eq!(list.rows.len(), 3);
}

#[test]
fn test_matrix_list_builder_with_count_hint() {
    let list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string()])
        .count_hint(100)
        .build();

    assert_eq!(list.count_hint, Some(100));
}

#[test]
fn test_node_builder_basic() {
    let node = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .field(ValueBuilder::string("Alice Smith"))
        .build();

    assert_eq!(node.type_name, "User");
    assert_eq!(node.id, "alice");
    assert_eq!(node.fields.len(), 2);
}

#[test]
fn test_node_builder_with_fields_batch() {
    let node = NodeBuilder::new("User", "alice")
        .fields(vec![
            ValueBuilder::string("alice"),
            ValueBuilder::string("Alice Smith"),
            ValueBuilder::int(30),
        ])
        .build();

    assert_eq!(node.fields.len(), 3);
}

#[test]
fn test_node_builder_with_children() {
    let child = Node::new("Post", "post1", vec![ValueBuilder::string("post1")]);

    let node = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .child("posts", child)
        .build();

    assert!(node.children.is_some());
    if let Some(ref children) = node.children {
        assert!(children.contains_key("posts"));
        assert_eq!(children.get("posts").unwrap().len(), 1);
    }
}

#[test]
fn test_node_builder_with_children_batch() {
    let children = vec![
        Node::new("Post", "post1", vec![ValueBuilder::string("post1")]),
        Node::new("Post", "post2", vec![ValueBuilder::string("post2")]),
    ];

    let node = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .children("posts", children)
        .build();

    if let Some(ref children) = node.children {
        assert_eq!(children.get("posts").unwrap().len(), 2);
    } else {
        panic!("Expected children");
    }
}

#[test]
fn test_node_builder_with_child_count() {
    let node = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .child_count(5)
        .build();

    assert_eq!(node.child_count, 5);
}

#[test]
fn test_node_builder_without_children() {
    let node = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .build();

    assert!(node.children.is_none());
}

#[test]
fn test_value_builder_null() {
    assert!(matches!(ValueBuilder::null(), Value::Null));
}

#[test]
fn test_value_builder_bool() {
    assert!(matches!(ValueBuilder::bool_val(true), Value::Bool(true)));
    assert!(matches!(ValueBuilder::bool_val(false), Value::Bool(false)));
}

#[test]
fn test_value_builder_int() {
    assert!(matches!(ValueBuilder::int(42), Value::Int(42)));
    assert!(matches!(ValueBuilder::int(-17), Value::Int(-17)));
    assert!(matches!(ValueBuilder::int(0), Value::Int(0)));
}

#[test]
fn test_value_builder_float() {
    if let Value::Float(f) = ValueBuilder::float(2.5) {
        assert!((f - 2.5).abs() < 0.001);
    } else {
        panic!("Expected float");
    }
}

#[test]
fn test_value_builder_string() {
    if let Value::String(s) = ValueBuilder::string("hello") {
        assert_eq!(s.as_ref(), "hello");
    } else {
        panic!("Expected string");
    }
}

#[test]
fn test_value_builder_reference() {
    if let Value::Reference(r) = ValueBuilder::reference("User", "alice") {
        assert_eq!(r.type_name.as_ref().unwrap().as_ref(), "User");
        assert_eq!(r.id.as_ref(), "alice");
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_value_builder_local_ref() {
    if let Value::Reference(r) = ValueBuilder::local_ref("local_id") {
        assert!(r.type_name.is_none());
        assert_eq!(r.id.as_ref(), "local_id");
    } else {
        panic!("Expected local reference");
    }
}

#[test]
fn test_value_builder_tensor_1d() {
    if let Value::Tensor(_t) = ValueBuilder::tensor_1d(vec![1.0, 2.0, 3.0]) {
        // Verify it's a tensor (can't dereference Box<Tensor> in match)
        // Successfully created tensor
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_value_builder_tensor_2d() {
    if let Value::Tensor(_t) = ValueBuilder::tensor_2d(vec![vec![1.0, 2.0], vec![3.0, 4.0]]) {
        // Verify it's a tensor (can't dereference Box<Tensor> in match)
        // Successfully created tensor
    } else {
        panic!("Expected tensor");
    }
}

#[test]
fn test_quick_simple_scalars() {
    let doc = quick::simple_scalars(vec![("name", "Alice"), ("city", "NYC"), ("country", "USA")]);

    assert_eq!(doc.root.len(), 3);
    assert!(doc.root.contains_key("name"));
    assert!(doc.root.contains_key("city"));
    assert!(doc.root.contains_key("country"));

    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("name") {
        assert_eq!(s.as_ref(), "Alice");
    }
}

#[test]
fn test_quick_simple_user_list() {
    let doc = quick::simple_user_list(vec![
        ("alice", "Alice Smith", "alice@example.com"),
        ("bob", "Bob Jones", "bob@example.com"),
        ("charlie", "Charlie Brown", "charlie@example.com"),
    ]);

    assert!(doc.root.contains_key("users"));
    assert!(doc.structs.contains_key("User"));

    if let Some(Item::List(list)) = doc.root.get("users") {
        assert_eq!(list.rows.len(), 3);
        assert_eq!(list.rows[0].id, "alice");
        assert_eq!(list.rows[1].id, "bob");
        assert_eq!(list.rows[2].id, "charlie");
    } else {
        panic!("Expected users list");
    }
}

#[test]
fn test_quick_with_references() {
    let doc = quick::with_references(
        vec![("alice", "Alice"), ("bob", "Bob")],
        vec![
            ("post1", "First Post", "alice"),
            ("post2", "Second Post", "bob"),
            ("post3", "Third Post", "alice"),
        ],
    );

    assert!(doc.root.contains_key("users"));
    assert!(doc.root.contains_key("posts"));
    assert!(doc.structs.contains_key("User"));
    assert!(doc.structs.contains_key("Post"));

    if let Some(Item::List(posts)) = doc.root.get("posts") {
        assert_eq!(posts.rows.len(), 3);

        // Verify references
        for post in &posts.rows {
            assert!(matches!(post.fields[2], Value::Reference(_)));
        }

        // Check specific reference
        if let Value::Reference(r) = &posts.rows[0].fields[2] {
            assert_eq!(r.id.as_ref(), "alice");
        }
    }
}

#[test]
fn test_builder_complex_nested_structure() {
    let post1 = NodeBuilder::new("Post", "post1")
        .field(ValueBuilder::string("post1"))
        .field(ValueBuilder::string("First Post"))
        .build();

    let post2 = NodeBuilder::new("Post", "post2")
        .field(ValueBuilder::string("post2"))
        .field(ValueBuilder::string("Second Post"))
        .build();

    let user = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .field(ValueBuilder::string("Alice Smith"))
        .children("posts", vec![post1, post2])
        .build();

    let users_list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string(), "name".to_string()])
        .row(user)
        .build();

    let doc = DocumentBuilder::new()
        .struct_def("User", vec!["id".to_string(), "name".to_string()])
        .struct_def("Post", vec!["id".to_string(), "title".to_string()])
        .nest("User", "Post")
        .list("users", users_list)
        .build();

    assert_eq!(doc.nests.get("User"), Some(&vec!["Post".to_string()]));

    if let Some(Item::List(list)) = doc.root.get("users") {
        let user = &list.rows[0];
        if let Some(ref children) = user.children {
            assert_eq!(children.get("posts").unwrap().len(), 2);
        } else {
            panic!("Expected children");
        }
    }
}

#[test]
fn test_builder_empty_lists() {
    let empty_list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string()])
        .build();

    assert!(empty_list.rows.is_empty());
    assert_eq!(empty_list.type_name, "User");
}

#[test]
fn test_builder_string_types() {
    // Test different string input types
    let doc = DocumentBuilder::new()
        .scalar("owned", ValueBuilder::string("owned".to_string()))
        .scalar("borrowed", ValueBuilder::string("borrowed"))
        .scalar("static", ValueBuilder::string("static"))
        .build();

    assert_eq!(doc.root.len(), 3);
}

#[test]
fn test_node_builder_multiple_child_groups() {
    let post = Node::new("Post", "post1", vec![ValueBuilder::string("post1")]);
    let comment = Node::new("Comment", "c1", vec![ValueBuilder::string("c1")]);

    let node = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .child("posts", post)
        .child("comments", comment)
        .build();

    if let Some(ref children) = node.children {
        assert_eq!(children.len(), 2);
        assert!(children.contains_key("posts"));
        assert!(children.contains_key("comments"));
    } else {
        panic!("Expected children");
    }
}

#[test]
fn test_node_builder_child_count_overflow() {
    let node = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .child_count(u16::MAX as usize + 100) // Exceeds u16::MAX
        .build();

    assert_eq!(node.child_count, u16::MAX);
}
