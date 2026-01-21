// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Recursive traversal helpers for validation rules.
//!
//! This module provides simple, reusable traversal functions that properly
//! handle arbitrary nesting depth in HEDL documents. This fixes the issue
//! where validation rules only went 2-3 levels deep.
//!
//! # Example
//!
//! ```rust,ignore
//! use hedl_core::validation::traverse::{visit_all_nodes, NodeVisitContext};
//!
//! let mut seen_ids = HashSet::new();
//! visit_all_nodes(&doc, |node, ctx| {
//!     seen_ids.insert((ctx.type_name.clone(), node.id.clone()));
//! });
//! ```

use crate::{Document, Item, Node, Value};

/// Context provided when visiting a node.
#[derive(Debug, Clone)]
pub struct NodeVisitContext<'a> {
    /// The type name for this node.
    pub type_name: &'a str,
    /// Current nesting depth (0 = root level nodes).
    pub depth: usize,
    /// Path of parent node IDs leading to this node.
    pub path: Vec<&'a str>,
}

impl<'a> NodeVisitContext<'a> {
    /// Create a new root-level context.
    fn new(type_name: &'a str) -> Self {
        Self {
            type_name,
            depth: 0,
            path: Vec::new(),
        }
    }

    /// Create a child context for nested nodes.
    fn child(&self, parent_id: &'a str, child_type: &'a str) -> Self {
        let mut path = self.path.clone();
        path.push(parent_id);
        Self {
            type_name: child_type,
            depth: self.depth + 1,
            path,
        }
    }

    /// Get the full path as a string (for diagnostics).
    pub fn path_string(&self) -> String {
        if self.path.is_empty() {
            "root".to_string()
        } else {
            self.path.join(" > ")
        }
    }
}

/// Context provided when visiting a value.
#[derive(Debug, Clone)]
pub struct ValueVisitContext<'a> {
    /// The type name of the containing node.
    pub type_name: &'a str,
    /// The ID of the containing node.
    pub node_id: &'a str,
    /// Field index within the node (if applicable).
    pub field_index: Option<usize>,
    /// Current nesting depth.
    pub depth: usize,
}

impl<'a> ValueVisitContext<'a> {
    /// Create context for a node field value.
    fn from_node(node: &'a Node, field_index: usize, depth: usize) -> Self {
        Self {
            type_name: &node.type_name,
            node_id: &node.id,
            field_index: Some(field_index),
            depth,
        }
    }

    /// Create context for a scalar item value.
    fn from_scalar(key: &'a str) -> Self {
        Self {
            type_name: "",
            node_id: key,
            field_index: None,
            depth: 0,
        }
    }

    /// Create context for an object value.
    #[allow(dead_code)]
    fn from_object(key: &'a str, depth: usize) -> Self {
        Self {
            type_name: "",
            node_id: key,
            field_index: None,
            depth,
        }
    }
}

/// Visit all nodes in a document recursively.
///
/// This properly handles arbitrary nesting depth (NEST hierarchies of any depth).
///
/// # Arguments
///
/// * `doc` - The document to traverse
/// * `visitor` - Callback invoked for each node with context
pub fn visit_all_nodes<F>(doc: &Document, mut visitor: F)
where
    F: FnMut(&Node, &NodeVisitContext<'_>),
{
    for item in doc.root.values() {
        if let Item::List(matrix_list) = item {
            for node in &matrix_list.rows {
                let ctx = NodeVisitContext::new(&node.type_name);
                visit_node_recursive(node, &ctx, &mut visitor);
            }
        }
    }
}

/// Visit a node and all its descendants recursively.
fn visit_node_recursive<'a, F>(node: &'a Node, ctx: &NodeVisitContext<'a>, visitor: &mut F)
where
    F: FnMut(&Node, &NodeVisitContext<'_>),
{
    // Visit this node
    visitor(node, ctx);

    // Recursively visit children at any depth
    if let Some(children_map) = &node.children {
        for (child_type, child_nodes) in children_map.iter() {
            for child in child_nodes {
                let child_ctx = ctx.child(&node.id, child_type);
                visit_node_recursive(child, &child_ctx, visitor);
            }
        }
    }
}

/// Visit all values in a document recursively.
///
/// This visits values in:
/// - Root-level scalars
/// - Object values (recursively nested)
/// - Node field values (at any nesting depth)
///
/// # Arguments
///
/// * `doc` - The document to traverse
/// * `visitor` - Callback invoked for each value with context
pub fn visit_all_values<F>(doc: &Document, mut visitor: F)
where
    F: FnMut(&Value, &ValueVisitContext<'_>),
{
    for (key, item) in &doc.root {
        visit_item_values(key, item, 0, &mut visitor);
    }
}

/// Visit values within an item recursively.
fn visit_item_values<'a, F>(key: &'a str, item: &'a Item, depth: usize, visitor: &mut F)
where
    F: FnMut(&Value, &ValueVisitContext<'_>),
{
    match item {
        Item::Scalar(value) => {
            let ctx = ValueVisitContext::from_scalar(key);
            visitor(value, &ctx);
        }
        Item::Object(map) => {
            for (child_key, child_item) in map {
                visit_item_values(child_key, child_item, depth + 1, visitor);
            }
        }
        Item::List(matrix_list) => {
            for node in &matrix_list.rows {
                visit_node_values(node, depth, visitor);
            }
        }
    }
}

/// Visit values within a node and its children recursively.
fn visit_node_values<F>(node: &Node, depth: usize, visitor: &mut F)
where
    F: FnMut(&Value, &ValueVisitContext<'_>),
{
    // Visit field values
    for (idx, value) in node.fields.iter().enumerate() {
        let ctx = ValueVisitContext::from_node(node, idx, depth);
        visitor(value, &ctx);
    }

    // Recursively visit child node values
    if let Some(children_map) = &node.children {
        for child_nodes in children_map.values() {
            for child in child_nodes {
                visit_node_values(child, depth + 1, visitor);
            }
        }
    }
}

/// Visit all references in a document recursively.
///
/// This is a convenience wrapper around `visit_all_values` that only calls
/// the visitor for Reference values.
///
/// # Arguments
///
/// * `doc` - The document to traverse
/// * `visitor` - Callback invoked for each reference with context
pub fn visit_all_references<F>(doc: &Document, mut visitor: F)
where
    F: FnMut(&crate::Reference, &ValueVisitContext<'_>),
{
    visit_all_values(doc, |value, ctx| {
        if let Value::Reference(r) = value {
            visitor(r, ctx);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use std::collections::HashSet;

    #[test]
    fn test_visit_all_nodes_root_level() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
  | bob, Bob
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut visited = Vec::new();
        visit_all_nodes(&doc, |node, ctx| {
            visited.push((ctx.type_name.to_string(), node.id.clone(), ctx.depth));
        });

        assert_eq!(visited.len(), 2);
        assert!(visited.contains(&("User".to_string(), "alice".to_string(), 0)));
        assert!(visited.contains(&("User".to_string(), "bob".to_string(), 0)));
    }

    #[test]
    fn test_visit_all_nodes_nested() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: User: [id]
%STRUCT: Post: [id]
%NEST: User > Post
---
users: @User
  | alice
    | post1
    | post2
  | bob
    | post3
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut visited = Vec::new();
        visit_all_nodes(&doc, |node, ctx| {
            visited.push((ctx.type_name.to_string(), node.id.clone(), ctx.depth));
        });

        assert_eq!(visited.len(), 5);
        assert!(visited.contains(&("User".to_string(), "alice".to_string(), 0)));
        assert!(visited.contains(&("User".to_string(), "bob".to_string(), 0)));
        assert!(visited.contains(&("Post".to_string(), "post1".to_string(), 1)));
        assert!(visited.contains(&("Post".to_string(), "post2".to_string(), 1)));
        assert!(visited.contains(&("Post".to_string(), "post3".to_string(), 1)));
    }

    #[test]
    fn test_visit_all_nodes_deeply_nested() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%STRUCT: C: [id]
%STRUCT: D: [id]
%NEST: A > B
%NEST: B > C
%NEST: C > D
---
items: @A
  | a1
    | b1
      | c1
        | d1
        | d2
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut visited = Vec::new();
        visit_all_nodes(&doc, |node, ctx| {
            visited.push((ctx.type_name.to_string(), node.id.clone(), ctx.depth));
        });

        assert_eq!(visited.len(), 5);
        assert!(visited.contains(&("A".to_string(), "a1".to_string(), 0)));
        assert!(visited.contains(&("B".to_string(), "b1".to_string(), 1)));
        assert!(visited.contains(&("C".to_string(), "c1".to_string(), 2)));
        assert!(visited.contains(&("D".to_string(), "d1".to_string(), 3)));
        assert!(visited.contains(&("D".to_string(), "d2".to_string(), 3)));
    }

    #[test]
    fn test_visit_all_values_from_nodes() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: User: [id, score]
---
users: @User
  | alice, 100
  | bob, 200
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut values = Vec::new();
        visit_all_values(&doc, |value, ctx| {
            if let Value::Int(n) = value {
                values.push((ctx.node_id.to_string(), *n));
            }
        });

        assert_eq!(values.len(), 2);
        assert!(values.contains(&("alice".to_string(), 100)));
        assert!(values.contains(&("bob".to_string(), 200)));
    }

    #[test]
    fn test_visit_all_values_nested_nodes() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: User: [id]
%STRUCT: Post: [id, likes]
%NEST: User > Post
---
users: @User
  | alice
    | post1, 10
    | post2, 20
  | bob
    | post3, 30
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut likes_sum = 0;
        visit_all_values(&doc, |value, _ctx| {
            if let Value::Int(n) = value {
                likes_sum += n;
            }
        });

        assert_eq!(likes_sum, 60);
    }

    #[test]
    fn test_visit_all_references() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: User: [id]
%STRUCT: Post: [id, author]
%NEST: User > Post
---
users: @User
  | alice
    | post1, @User:alice
    | post2, @User:alice
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut refs: HashSet<String> = HashSet::new();
        visit_all_references(&doc, |r, _ctx| {
            refs.insert(r.id.to_string());
        });

        assert_eq!(refs.len(), 1);
        assert!(refs.contains("alice"));
    }

    #[test]
    fn test_visit_deeply_nested_references() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%STRUCT: C: [id]
%STRUCT: D: [id, ref]
%NEST: A > B
%NEST: B > C
%NEST: C > D
---
items: @A
  | a1
    | b1
      | c1
        | d1, @A:a1
        | d2, @B:b1
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut refs: HashSet<String> = HashSet::new();
        visit_all_references(&doc, |r, _ctx| {
            refs.insert(r.id.to_string());
        });

        assert_eq!(refs.len(), 2);
        assert!(refs.contains("a1"));
        assert!(refs.contains("b1"));
    }

    #[test]
    fn test_node_context_path() {
        let hedl = r#"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%STRUCT: C: [id]
%NEST: A > B
%NEST: B > C
---
items: @A
  | a1
    | b1
      | c1
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut paths = Vec::new();
        visit_all_nodes(&doc, |node, ctx| {
            if node.id == "c1" {
                paths.push(ctx.path_string());
            }
        });

        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], "a1 > b1");
    }

    #[test]
    fn test_visit_object_values() {
        let hedl = r#"
%VERSION: 1.0
---
config:
  nested:
    value: 42
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut found_value = false;
        visit_all_values(&doc, |value, _ctx| {
            if let Value::Int(42) = value {
                found_value = true;
            }
        });

        assert!(found_value);
    }

    #[test]
    fn test_visit_scalar_values() {
        let hedl = r#"
%VERSION: 1.0
---
name: Test
count: 42
"#;
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut count = 0;
        visit_all_values(&doc, |_value, _ctx| {
            count += 1;
        });

        assert_eq!(count, 2);
    }
}
