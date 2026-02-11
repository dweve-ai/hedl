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

//! Tests for counting utilities.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use hedl_test::fixtures::builders::*;
use hedl_test::{count_nodes, count_references, fixtures};

#[test]
fn test_count_nodes_empty_document() {
    let doc = fixtures::empty();
    assert_eq!(count_nodes(&doc), 0);
}

#[test]
fn test_count_nodes_single_list() {
    let doc = fixtures::user_list();
    assert_eq!(count_nodes(&doc), 3);
}

#[test]
fn test_count_nodes_multiple_lists() {
    let doc = fixtures::with_references();
    // 2 users + 3 posts = 5
    assert_eq!(count_nodes(&doc), 5);
}

#[test]
fn test_count_nodes_with_nested() {
    let doc = fixtures::with_nest();
    // 2 users + 3 posts (nested) = 5
    assert_eq!(count_nodes(&doc), 5);
}

#[test]
fn test_count_nodes_deep_nesting() {
    let doc = fixtures::deep_nest();
    // 1 org + 1 dept + 2 employees = 4
    assert_eq!(count_nodes(&doc), 4);
}

#[test]
fn test_count_nodes_comprehensive() {
    let doc = fixtures::comprehensive();
    let count = count_nodes(&doc);
    // Should have multiple nodes
    assert!(count > 0);
}

#[test]
fn test_count_nodes_blog_fixture() {
    let doc = fixtures::blog();
    let count = count_nodes(&doc);
    // Blog has many entities
    assert!(count > 20);
}

#[test]
fn test_count_references_empty_document() {
    let doc = fixtures::empty();
    assert_eq!(count_references(&doc), 0);
}

#[test]
fn test_count_references_no_references() {
    let doc = fixtures::user_list();
    assert_eq!(count_references(&doc), 0);
}

#[test]
fn test_count_references_with_references() {
    let doc = fixtures::with_references();
    // 3 posts with author references
    assert_eq!(count_references(&doc), 3);
}

#[test]
fn test_count_references_comprehensive() {
    let doc = fixtures::comprehensive();
    let count = count_references(&doc);
    // Should have some references
    assert!(count > 0);
}

#[test]
fn test_count_references_blog() {
    let doc = fixtures::blog();
    let count = count_references(&doc);
    // Blog has many references
    assert!(count > 10);
}

#[test]
fn test_count_references_in_scalar_root() {
    let mut doc = Document::new((2, 0));

    doc.root.insert(
        "ref1".to_string(),
        Item::Scalar(Value::Reference(Reference {
            type_name: Some("User".to_string().into()),
            id: "alice".to_string().into(),
        })),
    );

    assert_eq!(count_references(&doc), 1);
}

#[test]
fn test_count_references_in_nested_children() {
    let child_with_ref = Node::new(
        "Post",
        "post1",
        vec![
            Value::String("post1".to_string().into()),
            Value::Reference(Reference {
                type_name: Some("User".to_string().into()),
                id: "alice".to_string().into(),
            }),
        ],
    );

    let parent = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .child("posts", child_with_ref)
        .build();

    let list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string()])
        .row(parent)
        .build();

    let doc = DocumentBuilder::new().list("users", list).build();

    // Should count reference in nested child
    assert_eq!(count_references(&doc), 1);
}

#[test]
fn test_count_references_multiple_levels_deep() {
    // Build deeply nested structure with references at each level
    let grandchild = Node::new(
        "Comment",
        "c1",
        vec![
            Value::String("c1".to_string().into()),
            Value::Reference(Reference {
                type_name: Some("User".to_string().into()),
                id: "bob".to_string().into(),
            }),
        ],
    );

    let child = NodeBuilder::new("Post", "post1")
        .field(ValueBuilder::string("post1"))
        .field(ValueBuilder::reference("User", "alice"))
        .child("comments", grandchild)
        .build();

    let parent = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .child("posts", child)
        .build();

    let list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string()])
        .row(parent)
        .build();

    let doc = DocumentBuilder::new().list("users", list).build();

    // Should count references at multiple levels: post.author + comment.author = 2
    assert_eq!(count_references(&doc), 2);
}

#[test]
fn test_count_nodes_with_multiple_child_types() {
    let post = Node::new("Post", "post1", vec![ValueBuilder::string("post1")]);
    let comment = Node::new("Comment", "c1", vec![ValueBuilder::string("c1")]);

    let user = NodeBuilder::new("User", "alice")
        .field(ValueBuilder::string("alice"))
        .child("posts", post)
        .child("comments", comment)
        .build();

    let list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string()])
        .row(user)
        .build();

    let doc = DocumentBuilder::new().list("users", list).build();

    // 1 user + 1 post + 1 comment = 3
    assert_eq!(count_nodes(&doc), 3);
}

#[test]
fn test_count_nodes_large_dataset() {
    let mut list = MatrixList::new("Item", vec!["id".to_string()]);

    for i in 0..1000 {
        list.add_row(Node::new(
            "Item",
            format!("item_{i}"),
            vec![ValueBuilder::string(format!("item_{i}"))],
        ));
    }

    let doc = DocumentBuilder::new().list("items", list).build();

    assert_eq!(count_nodes(&doc), 1000);
}

#[test]
fn test_count_references_large_dataset() {
    let mut list = MatrixList::new("Ref", vec!["id".to_string(), "target".to_string()]);

    for i in 0..500 {
        list.add_row(Node::new(
            "Ref",
            format!("ref_{i}"),
            vec![
                ValueBuilder::string(format!("ref_{i}")),
                ValueBuilder::reference("Target", format!("target_{i}")),
            ],
        ));
    }

    let doc = DocumentBuilder::new().list("refs", list).build();

    assert_eq!(count_references(&doc), 500);
}

#[test]
fn test_count_nodes_only_counts_list_nodes() {
    let doc = DocumentBuilder::new()
        .scalar("name", ValueBuilder::string("Alice"))
        .scalar("age", ValueBuilder::int(30))
        .scalar("active", ValueBuilder::bool_val(true))
        .build();

    // Scalars don't count as nodes
    assert_eq!(count_nodes(&doc), 0);
}

#[test]
fn test_count_references_ignores_non_reference_values() {
    let mut doc = Document::new((2, 0));

    doc.root.insert(
        "name".to_string(),
        Item::Scalar(ValueBuilder::string("Alice")),
    );
    doc.root
        .insert("age".to_string(), Item::Scalar(ValueBuilder::int(30)));
    doc.root.insert(
        "active".to_string(),
        Item::Scalar(ValueBuilder::bool_val(true)),
    );

    assert_eq!(count_references(&doc), 0);
}

#[test]
fn test_count_nodes_empty_list() {
    let empty_list = MatrixListBuilder::new("User")
        .schema(vec!["id".to_string()])
        .build();

    let doc = DocumentBuilder::new().list("users", empty_list).build();

    assert_eq!(count_nodes(&doc), 0);
}

#[test]
fn test_count_references_mixed_field_types() {
    let node = Node::new(
        "Entity",
        "e1",
        vec![
            ValueBuilder::string("e1"),
            ValueBuilder::int(42),
            ValueBuilder::reference("Target", "t1"),
            ValueBuilder::bool_val(true),
            ValueBuilder::reference("Target", "t2"),
        ],
    );

    let list = MatrixListBuilder::new("Entity")
        .schema(vec![
            "id".to_string(),
            "count".to_string(),
            "ref1".to_string(),
            "active".to_string(),
            "ref2".to_string(),
        ])
        .row(node)
        .build();

    let doc = DocumentBuilder::new().list("entities", list).build();

    // Should count 2 references
    assert_eq!(count_references(&doc), 2);
}

#[test]
fn test_count_nodes_with_siblings() {
    let child1 = Node::new("Child", "c1", vec![ValueBuilder::string("c1")]);
    let child2 = Node::new("Child", "c2", vec![ValueBuilder::string("c2")]);
    let child3 = Node::new("Child", "c3", vec![ValueBuilder::string("c3")]);

    let parent = NodeBuilder::new("Parent", "p1")
        .field(ValueBuilder::string("p1"))
        .children("children", vec![child1, child2, child3])
        .build();

    let list = MatrixListBuilder::new("Parent")
        .schema(vec!["id".to_string()])
        .row(parent)
        .build();

    let doc = DocumentBuilder::new().list("parents", list).build();

    // 1 parent + 3 children = 4
    assert_eq!(count_nodes(&doc), 4);
}
