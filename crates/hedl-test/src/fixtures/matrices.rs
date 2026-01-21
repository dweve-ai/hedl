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

//! Fixtures for `MatrixList` structures with various configurations.

use hedl_core::{Document, Item, MatrixList, Node, Reference, Value};
use smallvec::SmallVec;
use std::collections::BTreeMap;

/// Document with a simple user list.
///
/// Tests: `MatrixList` with string fields.
#[must_use]
pub fn user_list() -> Document {
    let mut root = BTreeMap::new();

    let users = MatrixList {
        type_name: "User".to_string(),
        schema: vec!["id".to_string(), "name".to_string(), "email".to_string()],
        rows: vec![
            Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("alice".to_string().into()),
                    Value::String("Alice Smith".to_string().into()),
                    Value::String("alice@example.com".to_string().into()),
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
                    Value::String("bob@example.com".to_string().into()),
                ]),
                children: None,
                child_count: 0,
            },
            Node {
                type_name: "User".to_string(),
                id: "charlie".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("charlie".to_string().into()),
                    Value::String("Charlie Brown".to_string().into()),
                    Value::String("charlie@example.com".to_string().into()),
                ]),
                children: None,
                child_count: 0,
            },
        ],
        count_hint: None,
    };

    root.insert("users".to_string(), Item::List(users));

    let mut structs = BTreeMap::new();
    structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string(), "email".to_string()],
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

/// Document with `MatrixList` containing various field types.
///
/// Tests: `MatrixList` with int, float, bool, null fields.
#[must_use]
pub fn mixed_type_list() -> Document {
    let mut root = BTreeMap::new();

    let items = MatrixList {
        type_name: "Item".to_string(),
        schema: vec![
            "id".to_string(),
            "name".to_string(),
            "count".to_string(),
            "price".to_string(),
            "active".to_string(),
            "notes".to_string(),
        ],
        rows: vec![
            Node {
                type_name: "Item".to_string(),
                id: "item1".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("item1".to_string().into()),
                    Value::String("Widget".to_string().into()),
                    Value::Int(100),
                    Value::Float(9.99),
                    Value::Bool(true),
                    Value::String("Best seller".to_string().into()),
                ]),
                children: None,
                child_count: 0,
            },
            Node {
                type_name: "Item".to_string(),
                id: "item2".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("item2".to_string().into()),
                    Value::String("Gadget".to_string().into()),
                    Value::Int(50),
                    Value::Float(19.99),
                    Value::Bool(false),
                    Value::Null,
                ]),
                children: None,
                child_count: 0,
            },
        ],
        count_hint: None,
    };

    root.insert("items".to_string(), Item::List(items));

    let mut structs = BTreeMap::new();
    structs.insert(
        "Item".to_string(),
        vec![
            "id".to_string(),
            "name".to_string(),
            "count".to_string(),
            "price".to_string(),
            "active".to_string(),
            "notes".to_string(),
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

/// Document with references between lists.
///
/// Tests: `MatrixList` with reference fields pointing to other nodes.
#[must_use]
pub fn with_references() -> Document {
    let mut root = BTreeMap::new();

    // Users
    let users = MatrixList {
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
    };

    // Posts with author references
    let posts = MatrixList {
        type_name: "Post".to_string(),
        schema: vec!["id".to_string(), "title".to_string(), "author".to_string()],
        rows: vec![
            Node {
                type_name: "Post".to_string(),
                id: "post1".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("post1".to_string().into()),
                    Value::String("Hello World".to_string().into()),
                    Value::Reference(Reference {
                        type_name: Some("User".to_string().into()),
                        id: "alice".to_string().into(),
                    }),
                ]),
                children: None,
                child_count: 0,
            },
            Node {
                type_name: "Post".to_string(),
                id: "post2".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("post2".to_string().into()),
                    Value::String("Rust is great".to_string().into()),
                    Value::Reference(Reference {
                        type_name: Some("User".to_string().into()),
                        id: "bob".to_string().into(),
                    }),
                ]),
                children: None,
                child_count: 0,
            },
            Node {
                type_name: "Post".to_string(),
                id: "post3".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("post3".to_string().into()),
                    Value::String("HEDL Tutorial".to_string().into()),
                    Value::Reference(Reference {
                        type_name: Some("User".to_string().into()),
                        id: "alice".to_string().into(),
                    }),
                ]),
                children: None,
                child_count: 0,
            },
        ],
        count_hint: None,
    };

    root.insert("users".to_string(), Item::List(users));
    root.insert("posts".to_string(), Item::List(posts));

    let mut structs = BTreeMap::new();
    structs.insert(
        "User".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    structs.insert(
        "Post".to_string(),
        vec!["id".to_string(), "title".to_string(), "author".to_string()],
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

/// Document with NEST hierarchy.
///
/// Tests: Parent-child relationships via NEST.
#[must_use]
pub fn with_nest() -> Document {
    let mut root = BTreeMap::new();

    // Users with nested posts
    let mut alice_children = BTreeMap::new();
    alice_children.insert(
        "posts".to_string(),
        vec![
            Node {
                type_name: "Post".to_string(),
                id: "post1".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("post1".to_string().into()),
                    Value::String("Alice's first post".to_string().into()),
                ]),
                children: None,
                child_count: 0,
            },
            Node {
                type_name: "Post".to_string(),
                id: "post2".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("post2".to_string().into()),
                    Value::String("Alice's second post".to_string().into()),
                ]),
                children: None,
                child_count: 0,
            },
        ],
    );

    let mut bob_children = BTreeMap::new();
    bob_children.insert(
        "posts".to_string(),
        vec![Node {
            type_name: "Post".to_string(),
            id: "post3".to_string(),
            fields: SmallVec::from_vec(vec![
                Value::String("post3".to_string().into()),
                Value::String("Bob's post".to_string().into()),
            ]),
            children: None,
            child_count: 0,
        }],
    );

    let users = MatrixList {
        type_name: "User".to_string(),
        schema: vec!["id".to_string(), "name".to_string()],
        rows: vec![
            Node {
                type_name: "User".to_string(),
                id: "alice".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("alice".to_string().into()),
                    Value::String("Alice".to_string().into()),
                ]),
                children: Some(Box::new(alice_children)),
                child_count: 0,
            },
            Node {
                type_name: "User".to_string(),
                id: "bob".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("bob".to_string().into()),
                    Value::String("Bob".to_string().into()),
                ]),
                children: Some(Box::new(bob_children)),
                child_count: 0,
            },
        ],
        count_hint: None,
    };

    root.insert("users".to_string(), Item::List(users));

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
    nests.insert("User".to_string(), "Post".to_string());

    Document {
        version: (1, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs,
        nests,
        root,
    }
}

/// Document with deep NEST hierarchy (3 levels).
///
/// Tests: Multi-level parent-child relationships.
#[must_use]
pub fn deep_nest() -> Document {
    let mut root = BTreeMap::new();

    // Organization > Department > Employee
    let mut dept_children = BTreeMap::new();
    dept_children.insert(
        "employees".to_string(),
        vec![
            Node {
                type_name: "Employee".to_string(),
                id: "emp1".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("emp1".to_string().into()),
                    Value::String("John Doe".to_string().into()),
                ]),
                children: None,
                child_count: 0,
            },
            Node {
                type_name: "Employee".to_string(),
                id: "emp2".to_string(),
                fields: SmallVec::from_vec(vec![
                    Value::String("emp2".to_string().into()),
                    Value::String("Jane Doe".to_string().into()),
                ]),
                children: None,
                child_count: 0,
            },
        ],
    );

    let mut org_children = BTreeMap::new();
    org_children.insert(
        "departments".to_string(),
        vec![Node {
            type_name: "Department".to_string(),
            id: "engineering".to_string(),
            fields: SmallVec::from_vec(vec![
                Value::String("engineering".to_string().into()),
                Value::String("Engineering".to_string().into()),
            ]),
            children: Some(Box::new(dept_children)),
            child_count: 0,
        }],
    );

    let orgs = MatrixList {
        type_name: "Organization".to_string(),
        schema: vec!["id".to_string(), "name".to_string()],
        rows: vec![Node {
            type_name: "Organization".to_string(),
            id: "acme".to_string(),
            fields: SmallVec::from_vec(vec![
                Value::String("acme".to_string().into()),
                Value::String("Acme Corp".to_string().into()),
            ]),
            children: Some(Box::new(org_children)),
            child_count: 0,
        }],
        count_hint: None,
    };

    root.insert("organizations".to_string(), Item::List(orgs));

    let mut structs = BTreeMap::new();
    structs.insert(
        "Organization".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    structs.insert(
        "Department".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );
    structs.insert(
        "Employee".to_string(),
        vec!["id".to_string(), "name".to_string()],
    );

    let mut nests = BTreeMap::new();
    nests.insert("Organization".to_string(), "Department".to_string());
    nests.insert("Department".to_string(), "Employee".to_string());

    Document {
        version: (1, 0),
        schema_versions: BTreeMap::new(),
        aliases: BTreeMap::new(),
        structs,
        nests,
        root,
    }
}
