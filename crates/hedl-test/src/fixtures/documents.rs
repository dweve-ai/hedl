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

//! Fixtures for complex multi-entity documents.

use hedl_core::{Document, ExprLiteral, Expression, Item, MatrixList, Node, Reference, Tensor, Value};
use hedl_core::lex::Span;
use std::collections::BTreeMap;

    /// Comprehensive document with all HEDL features.
    ///
    /// Tests: All value types, references, NEST, multiple lists.
    pub fn comprehensive() -> Document {
        let mut root = BTreeMap::new();

        // Scalar values
        root.insert("config_debug".to_string(), Item::Scalar(Value::Bool(true)));
        root.insert(
            "config_version".to_string(),
            Item::Scalar(Value::String("1.0.0".to_string())),
        );
        root.insert(
            "config_max_items".to_string(),
            Item::Scalar(Value::Int(1000)),
        );
        root.insert(
            "config_threshold".to_string(),
            Item::Scalar(Value::Float(0.95)),
        );

        // Expression: $(multiply(config_max_items, 2))
        root.insert(
            "computed".to_string(),
            Item::Scalar(Value::Expression(Expression::Call {
                name: "multiply".to_string(),
                args: vec![
                    Expression::Identifier {
                        name: "config_max_items".to_string(),
                        span: Span::default(),
                    },
                    Expression::Literal {
                        value: ExprLiteral::Int(2),
                        span: Span::default(),
                    },
                ],
                span: Span::default(),
            })),
        );

        // Tensor
        root.insert(
            "weights".to_string(),
            Item::Scalar(Value::Tensor(Tensor::Array(vec![
                Tensor::Scalar(0.1),
                Tensor::Scalar(0.2),
                Tensor::Scalar(0.3),
            ]))),
        );

        // Additional metadata as separate scalars (HEDL doesn't have object type)
        root.insert(
            "meta_created_by".to_string(),
            Item::Scalar(Value::String("system".to_string())),
        );
        root.insert("meta_version".to_string(), Item::Scalar(Value::Int(1)));

        // Users with nested posts (NEST)
        let mut alice_children = BTreeMap::new();
        alice_children.insert(
            "posts".to_string(),
            vec![Node {
                type_name: "Post".to_string(),
                id: "p1".to_string(),
                fields: vec![
                    Value::String("p1".to_string()),
                    Value::String("Introduction to HEDL".to_string()),
                    Value::Int(100),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
        );

        let users = MatrixList {
            type_name: "User".to_string(),
            schema: vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string(),
                "age".to_string(),
            ],
            rows: vec![
                Node {
                    type_name: "User".to_string(),
                    id: "alice".to_string(),
                    fields: vec![
                        Value::String("alice".to_string()),
                        Value::String("Alice Smith".to_string()),
                        Value::String("alice@example.com".to_string()),
                        Value::Int(30),
                    ],
                    children: alice_children,
                    child_count: None,
                },
                Node {
                    type_name: "User".to_string(),
                    id: "bob".to_string(),
                    fields: vec![
                        Value::String("bob".to_string()),
                        Value::String("Bob Jones".to_string()),
                        Value::String("bob@example.com".to_string()),
                        Value::Int(25),
                    ],
                    children: BTreeMap::new(),
                    child_count: None,
                },
            ],
            count_hint: None,
        };

        // Comments with references to users and posts
        let comments = MatrixList {
            type_name: "Comment".to_string(),
            schema: vec![
                "id".to_string(),
                "text".to_string(),
                "author".to_string(),
                "post".to_string(),
            ],
            rows: vec![Node {
                type_name: "Comment".to_string(),
                id: "c1".to_string(),
                fields: vec![
                    Value::String("c1".to_string()),
                    Value::String("Great article!".to_string()),
                    Value::Reference(Reference {
                        type_name: Some("User".to_string()),
                        id: "bob".to_string(),
                    }),
                    Value::Reference(Reference {
                        type_name: Some("Post".to_string()),
                        id: "p1".to_string(),
                    }),
                ],
                children: BTreeMap::new(),
                child_count: None,
            }],
        count_hint: None,
        };

        // Tags (simple list without references)
        let tags = MatrixList {
            type_name: "Tag".to_string(),
            schema: vec!["id".to_string(), "name".to_string(), "color".to_string()],
            rows: vec![
                Node {
                    type_name: "Tag".to_string(),
                    id: "rust".to_string(),
                    fields: vec![
                        Value::String("rust".to_string()),
                        Value::String("Rust".to_string()),
                        Value::String("#FF4500".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Tag".to_string(),
                    id: "hedl".to_string(),
                    fields: vec![
                        Value::String("hedl".to_string()),
                        Value::String("HEDL".to_string()),
                        Value::String("#00BFFF".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
            ],
        count_hint: None,
        };

        root.insert("users".to_string(), Item::List(users));
        root.insert("comments".to_string(), Item::List(comments));
        root.insert("tags".to_string(), Item::List(tags));

        let mut structs = BTreeMap::new();
        structs.insert(
            "User".to_string(),
            vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string(),
                "age".to_string(),
            ],
        );
        structs.insert(
            "Post".to_string(),
            vec!["id".to_string(), "title".to_string(), "views".to_string()],
        );
        structs.insert(
            "Comment".to_string(),
            vec![
                "id".to_string(),
                "text".to_string(),
                "author".to_string(),
                "post".to_string(),
            ],
        );
        structs.insert(
            "Tag".to_string(),
            vec!["id".to_string(), "name".to_string(), "color".to_string()],
        );

        let mut nests = BTreeMap::new();
        nests.insert("User".to_string(), "Post".to_string());

        Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs,
            nests,
            root,
        }
    }

    /// Comprehensive blog platform fixture.
    ///
    /// Tests: Complex relational data with users, posts, comments, reactions,
    /// tags, categories, and various cross-references.
    pub fn blog() -> Document {
        let mut root = BTreeMap::new();

        // Users
        let users = MatrixList {
            type_name: "User".to_string(),
            schema: vec![
                "id".to_string(),
                "username".to_string(),
                "email".to_string(),
                "display_name".to_string(),
                "bio".to_string(),
                "avatar_url".to_string(),
                "joined_at".to_string(),
                "is_verified".to_string(),
                "follower_count".to_string(),
            ],
            rows: vec![
                Node {
                    type_name: "User".to_string(),
                    id: "user1".to_string(),
                    fields: vec![
                        Value::String("user1".to_string()),
                        Value::String("alice_dev".to_string()),
                        Value::String("alice@techblog.com".to_string()),
                        Value::String("Alice Johnson".to_string()),
                        Value::String(
                            "Full-stack developer passionate about React and Rust".to_string(),
                        ),
                        Value::String("https://avatars.example.com/alice.jpg".to_string()),
                        Value::String("2023-01-15T10:00:00Z".to_string()),
                        Value::Bool(true),
                        Value::Int(1250),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "User".to_string(),
                    id: "user2".to_string(),
                    fields: vec![
                        Value::String("user2".to_string()),
                        Value::String("bob_writes".to_string()),
                        Value::String("bob@techblog.com".to_string()),
                        Value::String("Bob Smith".to_string()),
                        Value::String("Technical writer and documentation enthusiast".to_string()),
                        Value::String("https://avatars.example.com/bob.jpg".to_string()),
                        Value::String("2023-02-20T14:30:00Z".to_string()),
                        Value::Bool(true),
                        Value::Int(890),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "User".to_string(),
                    id: "user3".to_string(),
                    fields: vec![
                        Value::String("user3".to_string()),
                        Value::String("charlie_ml".to_string()),
                        Value::String("charlie@techblog.com".to_string()),
                        Value::String("Charlie Chen".to_string()),
                        Value::String("Machine learning engineer at BigTech".to_string()),
                        Value::String("https://avatars.example.com/charlie.jpg".to_string()),
                        Value::String("2023-03-10T09:15:00Z".to_string()),
                        Value::Bool(false),
                        Value::Int(2100),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "User".to_string(),
                    id: "user4".to_string(),
                    fields: vec![
                        Value::String("user4".to_string()),
                        Value::String("diana_design".to_string()),
                        Value::String("diana@techblog.com".to_string()),
                        Value::String("Diana Rodriguez".to_string()),
                        Value::String("UX designer crafting delightful experiences".to_string()),
                        Value::String("https://avatars.example.com/diana.jpg".to_string()),
                        Value::String("2023-04-05T11:45:00Z".to_string()),
                        Value::Bool(true),
                        Value::Int(1680),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "User".to_string(),
                    id: "user5".to_string(),
                    fields: vec![
                        Value::String("user5".to_string()),
                        Value::String("evan_backend".to_string()),
                        Value::String("evan@techblog.com".to_string()),
                        Value::String("Evan Park".to_string()),
                        Value::String("Backend architect, Go and Kubernetes expert".to_string()),
                        Value::String("https://avatars.example.com/evan.jpg".to_string()),
                        Value::String("2023-05-12T16:20:00Z".to_string()),
                        Value::Bool(false),
                        Value::Int(750),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
            ],
        count_hint: None,
        };

        // Categories
        let categories = MatrixList {
            type_name: "Category".to_string(),
            schema: vec![
                "id".to_string(),
                "name".to_string(),
                "slug".to_string(),
                "description".to_string(),
                "color".to_string(),
                "post_count".to_string(),
            ],
            rows: vec![
                Node {
                    type_name: "Category".to_string(),
                    id: "cat1".to_string(),
                    fields: vec![
                        Value::String("cat1".to_string()),
                        Value::String("Programming".to_string()),
                        Value::String("programming".to_string()),
                        Value::String(
                            "Articles about programming languages and techniques".to_string(),
                        ),
                        Value::String("#3498db".to_string()),
                        Value::Int(45),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Category".to_string(),
                    id: "cat2".to_string(),
                    fields: vec![
                        Value::String("cat2".to_string()),
                        Value::String("Web Development".to_string()),
                        Value::String("web-dev".to_string()),
                        Value::String("Frontend and backend web development topics".to_string()),
                        Value::String("#2ecc71".to_string()),
                        Value::Int(38),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Category".to_string(),
                    id: "cat3".to_string(),
                    fields: vec![
                        Value::String("cat3".to_string()),
                        Value::String("Machine Learning".to_string()),
                        Value::String("ml".to_string()),
                        Value::String("AI, ML, and data science articles".to_string()),
                        Value::String("#9b59b6".to_string()),
                        Value::Int(22),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Category".to_string(),
                    id: "cat4".to_string(),
                    fields: vec![
                        Value::String("cat4".to_string()),
                        Value::String("DevOps".to_string()),
                        Value::String("devops".to_string()),
                        Value::String("CI/CD, containers, and infrastructure".to_string()),
                        Value::String("#e74c3c".to_string()),
                        Value::Int(31),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Category".to_string(),
                    id: "cat5".to_string(),
                    fields: vec![
                        Value::String("cat5".to_string()),
                        Value::String("Design".to_string()),
                        Value::String("design".to_string()),
                        Value::String("UI/UX design principles and practices".to_string()),
                        Value::String("#f39c12".to_string()),
                        Value::Int(19),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
            ],
        count_hint: None,
        };

        // Tags
        let tags = MatrixList {
            type_name: "Tag".to_string(),
            schema: vec![
                "id".to_string(),
                "name".to_string(),
                "usage_count".to_string(),
            ],
            rows: vec![
                Node {
                    type_name: "Tag".to_string(),
                    id: "tag1".to_string(),
                    fields: vec![
                        Value::String("tag1".to_string()),
                        Value::String("rust".to_string()),
                        Value::Int(28),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Tag".to_string(),
                    id: "tag2".to_string(),
                    fields: vec![
                        Value::String("tag2".to_string()),
                        Value::String("javascript".to_string()),
                        Value::Int(52),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Tag".to_string(),
                    id: "tag3".to_string(),
                    fields: vec![
                        Value::String("tag3".to_string()),
                        Value::String("python".to_string()),
                        Value::Int(41),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Tag".to_string(),
                    id: "tag4".to_string(),
                    fields: vec![
                        Value::String("tag4".to_string()),
                        Value::String("react".to_string()),
                        Value::Int(35),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Tag".to_string(),
                    id: "tag5".to_string(),
                    fields: vec![
                        Value::String("tag5".to_string()),
                        Value::String("docker".to_string()),
                        Value::Int(29),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Tag".to_string(),
                    id: "tag6".to_string(),
                    fields: vec![
                        Value::String("tag6".to_string()),
                        Value::String("kubernetes".to_string()),
                        Value::Int(24),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Tag".to_string(),
                    id: "tag7".to_string(),
                    fields: vec![
                        Value::String("tag7".to_string()),
                        Value::String("typescript".to_string()),
                        Value::Int(33),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Tag".to_string(),
                    id: "tag8".to_string(),
                    fields: vec![
                        Value::String("tag8".to_string()),
                        Value::String("machine-learning".to_string()),
                        Value::Int(18),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Tag".to_string(),
                    id: "tag9".to_string(),
                    fields: vec![
                        Value::String("tag9".to_string()),
                        Value::String("api-design".to_string()),
                        Value::Int(15),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Tag".to_string(),
                    id: "tag10".to_string(),
                    fields: vec![
                        Value::String("tag10".to_string()),
                        Value::String("testing".to_string()),
                        Value::Int(21),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
            ],
        count_hint: None,
        };

        // Posts
        let posts = MatrixList {
            type_name: "Post".to_string(),
            schema: vec![
                "id".to_string(),
                "title".to_string(),
                "slug".to_string(),
                "content".to_string(),
                "excerpt".to_string(),
                "author_id".to_string(),
                "category_id".to_string(),
                "status".to_string(),
                "is_featured".to_string(),
                "view_count".to_string(),
                "read_time_minutes".to_string(),
                "created_at".to_string(),
                "published_at".to_string(),
                "updated_at".to_string(),
            ],
            rows: vec![
                Node {
                    type_name: "Post".to_string(),
                    id: "post1".to_string(),
                    fields: vec![
                        Value::String("post1".to_string()),
                        Value::String("Getting Started with Rust in 2024".to_string()),
                        Value::String("rust-getting-started-2024".to_string()),
                        Value::String("Rust has become one of the most loved programming languages. In this comprehensive guide, we'll explore why Rust is gaining popularity and how to get started with your first project.".to_string()),
                        Value::String("A beginner's guide to Rust programming".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user1".to_string() }),
                        Value::Reference(Reference { type_name: Some("Category".to_string()), id: "cat1".to_string() }),
                        Value::String("published".to_string()),
                        Value::Bool(true),
                        Value::Int(4520),
                        Value::Int(12),
                        Value::String("2024-01-10T09:00:00Z".to_string()),
                        Value::String("2024-01-10T16:00:00Z".to_string()),
                        Value::String("2024-01-10T15:30:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Post".to_string(),
                    id: "post2".to_string(),
                    fields: vec![
                        Value::String("post2".to_string()),
                        Value::String("Building Scalable APIs with GraphQL".to_string()),
                        Value::String("scalable-graphql-apis".to_string()),
                        Value::String("GraphQL offers a flexible approach to API design. Learn how to build performant, scalable GraphQL APIs that can handle millions of requests.".to_string()),
                        Value::String("Master GraphQL API development".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user2".to_string() }),
                        Value::Reference(Reference { type_name: Some("Category".to_string()), id: "cat2".to_string() }),
                        Value::String("published".to_string()),
                        Value::Bool(false),
                        Value::Int(3280),
                        Value::Int(15),
                        Value::String("2024-01-12T11:00:00Z".to_string()),
                        Value::String("2024-01-13T12:00:00Z".to_string()),
                        Value::String("2024-01-13T10:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Post".to_string(),
                    id: "post3".to_string(),
                    fields: vec![
                        Value::String("post3".to_string()),
                        Value::String("Introduction to Neural Networks".to_string()),
                        Value::String("intro-neural-networks".to_string()),
                        Value::String("Neural networks are the foundation of modern AI. This article breaks down the concepts in an accessible way for developers new to machine learning.".to_string()),
                        Value::String("Understanding neural networks from scratch".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user3".to_string() }),
                        Value::Reference(Reference { type_name: Some("Category".to_string()), id: "cat3".to_string() }),
                        Value::String("published".to_string()),
                        Value::Bool(true),
                        Value::Int(5890),
                        Value::Int(20),
                        Value::String("2024-01-15T08:30:00Z".to_string()),
                        Value::String("2024-01-15T10:00:00Z".to_string()),
                        Value::String("2024-01-15T08:30:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Post".to_string(),
                    id: "post4".to_string(),
                    fields: vec![
                        Value::String("post4".to_string()),
                        Value::String("Kubernetes Best Practices for Production".to_string()),
                        Value::String("k8s-production-best-practices".to_string()),
                        Value::String("Running Kubernetes in production requires careful planning. Learn the best practices for security, scaling, and monitoring your clusters.".to_string()),
                        Value::String("Production-ready Kubernetes deployments".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user5".to_string() }),
                        Value::Reference(Reference { type_name: Some("Category".to_string()), id: "cat4".to_string() }),
                        Value::String("published".to_string()),
                        Value::Bool(false),
                        Value::Int(2150),
                        Value::Int(18),
                        Value::String("2024-01-18T14:00:00Z".to_string()),
                        Value::String("2024-01-19T10:00:00Z".to_string()),
                        Value::String("2024-01-19T09:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Post".to_string(),
                    id: "post5".to_string(),
                    fields: vec![
                        Value::String("post5".to_string()),
                        Value::String("Design Systems That Scale".to_string()),
                        Value::String("scalable-design-systems".to_string()),
                        Value::String("A well-built design system can transform how your team works. Discover the principles behind design systems used by top tech companies.".to_string()),
                        Value::String("Building enterprise design systems".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user4".to_string() }),
                        Value::Reference(Reference { type_name: Some("Category".to_string()), id: "cat5".to_string() }),
                        Value::String("published".to_string()),
                        Value::Bool(false),
                        Value::Int(1890),
                        Value::Int(14),
                        Value::String("2024-01-20T10:00:00Z".to_string()),
                        Value::String("2024-01-21T18:00:00Z".to_string()),
                        Value::String("2024-01-21T16:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Post".to_string(),
                    id: "post6".to_string(),
                    fields: vec![
                        Value::String("post6".to_string()),
                        Value::String("Advanced TypeScript Patterns".to_string()),
                        Value::String("advanced-typescript-patterns".to_string()),
                        Value::String("Take your TypeScript skills to the next level with advanced type patterns, conditional types, and mapped types that make your code safer.".to_string()),
                        Value::String("Level up your TypeScript game".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user1".to_string() }),
                        Value::Reference(Reference { type_name: Some("Category".to_string()), id: "cat1".to_string() }),
                        Value::String("published".to_string()),
                        Value::Bool(true),
                        Value::Int(3450),
                        Value::Int(16),
                        Value::String("2024-01-25T09:00:00Z".to_string()),
                        Value::String("2024-01-25T12:00:00Z".to_string()),
                        Value::String("2024-01-25T09:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
            ],
        count_hint: None,
        };

        // Comments (with threading via parent_id)
        let comments = MatrixList {
            type_name: "Comment".to_string(),
            schema: vec![
                "id".to_string(),
                "content".to_string(),
                "author_id".to_string(),
                "post_id".to_string(),
                "parent_id".to_string(),
                "created_at".to_string(),
                "is_edited".to_string(),
                "is_deleted".to_string(),
            ],
            rows: vec![
                Node {
                    type_name: "Comment".to_string(),
                    id: "comment1".to_string(),
                    fields: vec![
                        Value::String("comment1".to_string()),
                        Value::String("Great introduction! I've been meaning to learn Rust. The memory safety features are what attracted me.".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user2".to_string() }),
                        Value::Reference(Reference { type_name: Some("Post".to_string()), id: "post1".to_string() }),
                        Value::Null,
                        Value::String("2024-01-10T17:30:00Z".to_string()),
                        Value::Bool(false),
                        Value::Bool(false),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Comment".to_string(),
                    id: "comment2".to_string(),
                    fields: vec![
                        Value::String("comment2".to_string()),
                        Value::String("Same here! The borrow checker was tricky at first, but it really helps catch bugs early.".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user3".to_string() }),
                        Value::Reference(Reference { type_name: Some("Post".to_string()), id: "post1".to_string() }),
                        Value::Reference(Reference { type_name: Some("Comment".to_string()), id: "comment1".to_string() }),
                        Value::String("2024-01-10T18:15:00Z".to_string()),
                        Value::Bool(false),
                        Value::Bool(false),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Comment".to_string(),
                    id: "comment3".to_string(),
                    fields: vec![
                        Value::String("comment3".to_string()),
                        Value::String("Thanks for the kind words! Yes, the borrow checker is your friend once you get used to it.".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user1".to_string() }),
                        Value::Reference(Reference { type_name: Some("Post".to_string()), id: "post1".to_string() }),
                        Value::Reference(Reference { type_name: Some("Comment".to_string()), id: "comment2".to_string() }),
                        Value::String("2024-01-10T19:00:00Z".to_string()),
                        Value::Bool(false),
                        Value::Bool(false),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Comment".to_string(),
                    id: "comment4".to_string(),
                    fields: vec![
                        Value::String("comment4".to_string()),
                        Value::String("How does this compare to REST for smaller projects? Is GraphQL overkill for simple APIs?".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user5".to_string() }),
                        Value::Reference(Reference { type_name: Some("Post".to_string()), id: "post2".to_string() }),
                        Value::Null,
                        Value::String("2024-01-13T14:00:00Z".to_string()),
                        Value::Bool(false),
                        Value::Bool(false),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Comment".to_string(),
                    id: "comment5".to_string(),
                    fields: vec![
                        Value::String("comment5".to_string()),
                        Value::String("Good question! For simple CRUD apps, REST might be simpler. GraphQL shines when you have complex data relationships.".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user2".to_string() }),
                        Value::Reference(Reference { type_name: Some("Post".to_string()), id: "post2".to_string() }),
                        Value::Reference(Reference { type_name: Some("Comment".to_string()), id: "comment4".to_string() }),
                        Value::String("2024-01-13T15:30:00Z".to_string()),
                        Value::Bool(true),
                        Value::Bool(false),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Comment".to_string(),
                    id: "comment6".to_string(),
                    fields: vec![
                        Value::String("comment6".to_string()),
                        Value::String("Finally an ML article I can understand! The diagrams really helped explain backpropagation.".to_string()),
                        Value::Reference(Reference { type_name: Some("User".to_string()), id: "user4".to_string() }),
                        Value::Reference(Reference { type_name: Some("Post".to_string()), id: "post3".to_string() }),
                        Value::Null,
                        Value::String("2024-01-15T12:00:00Z".to_string()),
                        Value::Bool(false),
                        Value::Bool(false),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
            ],
        count_hint: None,
        };

        // Reactions (likes, loves, etc.)
        let reactions = MatrixList {
            type_name: "Reaction".to_string(),
            schema: vec![
                "id".to_string(),
                "post_id".to_string(),
                "user_id".to_string(),
                "type".to_string(),
                "created_at".to_string(),
            ],
            rows: vec![
                Node {
                    type_name: "Reaction".to_string(),
                    id: "react1".to_string(),
                    fields: vec![
                        Value::String("react1".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post1".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user2".to_string(),
                        }),
                        Value::String("like".to_string()),
                        Value::String("2024-01-10T17:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Reaction".to_string(),
                    id: "react2".to_string(),
                    fields: vec![
                        Value::String("react2".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post1".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user3".to_string(),
                        }),
                        Value::String("love".to_string()),
                        Value::String("2024-01-10T17:05:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Reaction".to_string(),
                    id: "react3".to_string(),
                    fields: vec![
                        Value::String("react3".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post1".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user4".to_string(),
                        }),
                        Value::String("like".to_string()),
                        Value::String("2024-01-10T18:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Reaction".to_string(),
                    id: "react4".to_string(),
                    fields: vec![
                        Value::String("react4".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post1".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user5".to_string(),
                        }),
                        Value::String("insightful".to_string()),
                        Value::String("2024-01-10T19:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Reaction".to_string(),
                    id: "react5".to_string(),
                    fields: vec![
                        Value::String("react5".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post2".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user1".to_string(),
                        }),
                        Value::String("like".to_string()),
                        Value::String("2024-01-13T12:30:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Reaction".to_string(),
                    id: "react6".to_string(),
                    fields: vec![
                        Value::String("react6".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post2".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user3".to_string(),
                        }),
                        Value::String("like".to_string()),
                        Value::String("2024-01-13T13:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Reaction".to_string(),
                    id: "react7".to_string(),
                    fields: vec![
                        Value::String("react7".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post3".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user1".to_string(),
                        }),
                        Value::String("love".to_string()),
                        Value::String("2024-01-15T10:30:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Reaction".to_string(),
                    id: "react8".to_string(),
                    fields: vec![
                        Value::String("react8".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post3".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user2".to_string(),
                        }),
                        Value::String("like".to_string()),
                        Value::String("2024-01-15T11:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
            ],
        count_hint: None,
        };

        // Post-Tag relationships (many-to-many)
        let post_tags = MatrixList {
            type_name: "PostTag".to_string(),
            schema: vec![
                "id".to_string(),
                "post_id".to_string(),
                "tag_id".to_string(),
            ],
            rows: vec![
                Node {
                    type_name: "PostTag".to_string(),
                    id: "pt1".to_string(),
                    fields: vec![
                        Value::String("pt1".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post1".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("Tag".to_string()),
                            id: "tag1".to_string(),
                        }),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "PostTag".to_string(),
                    id: "pt2".to_string(),
                    fields: vec![
                        Value::String("pt2".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post1".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("Tag".to_string()),
                            id: "tag3".to_string(),
                        }),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "PostTag".to_string(),
                    id: "pt3".to_string(),
                    fields: vec![
                        Value::String("pt3".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post2".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("Tag".to_string()),
                            id: "tag2".to_string(),
                        }),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "PostTag".to_string(),
                    id: "pt4".to_string(),
                    fields: vec![
                        Value::String("pt4".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post2".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("Tag".to_string()),
                            id: "tag9".to_string(),
                        }),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "PostTag".to_string(),
                    id: "pt5".to_string(),
                    fields: vec![
                        Value::String("pt5".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post3".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("Tag".to_string()),
                            id: "tag3".to_string(),
                        }),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "PostTag".to_string(),
                    id: "pt6".to_string(),
                    fields: vec![
                        Value::String("pt6".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post3".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("Tag".to_string()),
                            id: "tag8".to_string(),
                        }),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "PostTag".to_string(),
                    id: "pt7".to_string(),
                    fields: vec![
                        Value::String("pt7".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post4".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("Tag".to_string()),
                            id: "tag5".to_string(),
                        }),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "PostTag".to_string(),
                    id: "pt8".to_string(),
                    fields: vec![
                        Value::String("pt8".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("Post".to_string()),
                            id: "post4".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("Tag".to_string()),
                            id: "tag6".to_string(),
                        }),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
            ],
        count_hint: None,
        };

        // Followers (user relationships)
        let followers = MatrixList {
            type_name: "Follower".to_string(),
            schema: vec![
                "id".to_string(),
                "follower_id".to_string(),
                "following_id".to_string(),
                "created_at".to_string(),
            ],
            rows: vec![
                Node {
                    type_name: "Follower".to_string(),
                    id: "follow1".to_string(),
                    fields: vec![
                        Value::String("follow1".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user2".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user1".to_string(),
                        }),
                        Value::String("2023-02-25T10:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Follower".to_string(),
                    id: "follow2".to_string(),
                    fields: vec![
                        Value::String("follow2".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user3".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user1".to_string(),
                        }),
                        Value::String("2023-03-15T14:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Follower".to_string(),
                    id: "follow3".to_string(),
                    fields: vec![
                        Value::String("follow3".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user4".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user1".to_string(),
                        }),
                        Value::String("2023-04-10T09:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Follower".to_string(),
                    id: "follow4".to_string(),
                    fields: vec![
                        Value::String("follow4".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user1".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user2".to_string(),
                        }),
                        Value::String("2023-02-22T11:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Follower".to_string(),
                    id: "follow5".to_string(),
                    fields: vec![
                        Value::String("follow5".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user3".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user2".to_string(),
                        }),
                        Value::String("2023-03-20T16:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
                Node {
                    type_name: "Follower".to_string(),
                    id: "follow6".to_string(),
                    fields: vec![
                        Value::String("follow6".to_string()),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user1".to_string(),
                        }),
                        Value::Reference(Reference {
                            type_name: Some("User".to_string()),
                            id: "user3".to_string(),
                        }),
                        Value::String("2023-03-12T08:00:00Z".to_string()),
                    ],
                    children: BTreeMap::new(),
                child_count: None,
                },
            ],
        count_hint: None,
        };

        root.insert("users".to_string(), Item::List(users));
        root.insert("categories".to_string(), Item::List(categories));
        root.insert("tags".to_string(), Item::List(tags));
        root.insert("posts".to_string(), Item::List(posts));
        root.insert("comments".to_string(), Item::List(comments));
        root.insert("reactions".to_string(), Item::List(reactions));
        root.insert("post_tags".to_string(), Item::List(post_tags));
        root.insert("followers".to_string(), Item::List(followers));

        let mut structs = BTreeMap::new();
        structs.insert(
            "User".to_string(),
            vec![
                "id".to_string(),
                "username".to_string(),
                "email".to_string(),
                "display_name".to_string(),
                "bio".to_string(),
                "avatar_url".to_string(),
                "joined_at".to_string(),
                "is_verified".to_string(),
                "follower_count".to_string(),
            ],
        );
        structs.insert(
            "Category".to_string(),
            vec![
                "id".to_string(),
                "name".to_string(),
                "slug".to_string(),
                "description".to_string(),
                "color".to_string(),
                "post_count".to_string(),
            ],
        );
        structs.insert(
            "Tag".to_string(),
            vec![
                "id".to_string(),
                "name".to_string(),
                "usage_count".to_string(),
            ],
        );
        structs.insert(
            "Post".to_string(),
            vec![
                "id".to_string(),
                "title".to_string(),
                "slug".to_string(),
                "content".to_string(),
                "excerpt".to_string(),
                "author_id".to_string(),
                "category_id".to_string(),
                "status".to_string(),
                "is_featured".to_string(),
                "view_count".to_string(),
                "read_time_minutes".to_string(),
                "created_at".to_string(),
                "published_at".to_string(),
                "updated_at".to_string(),
            ],
        );
        structs.insert(
            "Comment".to_string(),
            vec![
                "id".to_string(),
                "content".to_string(),
                "author_id".to_string(),
                "post_id".to_string(),
                "parent_id".to_string(),
                "created_at".to_string(),
                "is_edited".to_string(),
                "is_deleted".to_string(),
            ],
        );
        structs.insert(
            "Reaction".to_string(),
            vec![
                "id".to_string(),
                "post_id".to_string(),
                "user_id".to_string(),
                "type".to_string(),
                "created_at".to_string(),
            ],
        );
        structs.insert(
            "PostTag".to_string(),
            vec![
                "id".to_string(),
                "post_id".to_string(),
                "tag_id".to_string(),
            ],
        );
        structs.insert(
            "Follower".to_string(),
            vec![
                "id".to_string(),
                "follower_id".to_string(),
                "following_id".to_string(),
                "created_at".to_string(),
            ],
        );

        Document {
            version: (1, 0),
            aliases: BTreeMap::new(),
            structs,
            nests: BTreeMap::new(),
            root,
        }
    }
