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

//! Document traversal trait for format converters.
//!
//! This module provides a shared abstraction for traversing HEDL documents,
//! reducing code duplication across format converters (JSON, YAML, XML, etc.).
//!
//! # Architecture
//!
//! The visitor pattern is used to separate traversal logic from conversion logic.
//! Format converters implement the `DocumentVisitor` trait to handle each element
//! type, while the `traverse` function handles the recursive structure.
//!
//! # Example
//!
//! ```text
//! use hedl_core::traverse::{DocumentVisitor, traverse, VisitorContext};
//!
//! struct JsonEmitter { output: String }
//!
//! impl DocumentVisitor for JsonEmitter {
//!     type Error = String;
//!
//!     fn visit_scalar(&mut self, key: &str, value: &Value, ctx: &VisitorContext) -> Result<(), Self::Error> {
//!         // Emit JSON scalar
//!         Ok(())
//!     }
//!     // ... other methods
//! }
//!
//! let mut emitter = JsonEmitter::default();
//! traverse(&doc, &mut emitter)?;
//! ```

use crate::{Document, Item, MatrixList, Node, Value};

/// Context provided to visitors during traversal.
#[derive(Debug, Clone)]
pub struct VisitorContext<'a> {
    /// Current nesting depth (0 = root level).
    pub depth: usize,
    /// Path from root to current element (key names).
    pub path: Vec<&'a str>,
    /// Reference to the document being traversed.
    pub document: &'a Document,
    /// Schema for the current list (if within a list context).
    pub current_schema: Option<&'a [String]>,
}

impl<'a> VisitorContext<'a> {
    /// Create a new context for the root level.
    pub fn new(document: &'a Document) -> Self {
        Self {
            depth: 0,
            path: Vec::new(),
            document,
            current_schema: None,
        }
    }

    /// Create a child context with incremented depth.
    pub fn child(&self, key: &'a str) -> Self {
        let mut path = self.path.clone();
        path.push(key);
        Self {
            depth: self.depth + 1,
            path,
            document: self.document,
            current_schema: self.current_schema,
        }
    }

    /// Create a child context with a list schema.
    pub fn with_schema(&self, schema: &'a [String]) -> Self {
        Self {
            depth: self.depth,
            path: self.path.clone(),
            document: self.document,
            current_schema: Some(schema),
        }
    }

    /// Get the current path as a string (for error messages).
    pub fn path_string(&self) -> String {
        if self.path.is_empty() {
            "root".to_string()
        } else {
            self.path.join(".")
        }
    }
}

/// Trait for visiting elements of a HEDL document.
///
/// Implement this trait to perform format conversion or analysis.
/// All methods have default implementations that do nothing, allowing
/// implementations to override only the methods they need.
pub trait DocumentVisitor {
    /// Error type returned by visitor methods.
    type Error;

    /// Called at the start of document traversal.
    fn begin_document(&mut self, _doc: &Document, _ctx: &VisitorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called at the end of document traversal.
    fn end_document(&mut self, _doc: &Document, _ctx: &VisitorContext) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called when visiting a scalar value.
    fn visit_scalar(
        &mut self,
        key: &str,
        value: &Value,
        ctx: &VisitorContext,
    ) -> Result<(), Self::Error>;

    /// Called at the start of an object (before visiting children).
    fn begin_object(
        &mut self,
        _key: &str,
        _ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called at the end of an object (after visiting children).
    fn end_object(
        &mut self,
        _key: &str,
        _ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called at the start of a matrix list (before visiting rows).
    fn begin_list(
        &mut self,
        _key: &str,
        _list: &MatrixList,
        _ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called at the end of a matrix list (after visiting rows).
    fn end_list(
        &mut self,
        _key: &str,
        _list: &MatrixList,
        _ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called when visiting a node (row) in a matrix list.
    fn visit_node(
        &mut self,
        node: &Node,
        schema: &[String],
        ctx: &VisitorContext,
    ) -> Result<(), Self::Error>;

    /// Called at the start of a node's children (nested entities).
    fn begin_node_children(
        &mut self,
        _node: &Node,
        _ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Called at the end of a node's children.
    fn end_node_children(
        &mut self,
        _node: &Node,
        _ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// Traverse a HEDL document, calling visitor methods for each element.
///
/// This function handles the recursive structure of documents, freeing
/// format converters from duplicating traversal logic.
pub fn traverse<V: DocumentVisitor>(
    doc: &Document,
    visitor: &mut V,
) -> Result<(), V::Error> {
    let ctx = VisitorContext::new(doc);
    visitor.begin_document(doc, &ctx)?;

    for (key, item) in &doc.root {
        traverse_item(key, item, visitor, &ctx)?;
    }

    visitor.end_document(doc, &ctx)?;
    Ok(())
}

/// Traverse a single item recursively.
fn traverse_item<V: DocumentVisitor>(
    key: &str,
    item: &Item,
    visitor: &mut V,
    ctx: &VisitorContext,
) -> Result<(), V::Error> {
    match item {
        Item::Scalar(value) => {
            visitor.visit_scalar(key, value, ctx)?;
        }
        Item::Object(map) => {
            visitor.begin_object(key, ctx)?;
            let child_ctx = ctx.child(key);
            for (child_key, child_item) in map {
                traverse_item(child_key, child_item, visitor, &child_ctx)?;
            }
            visitor.end_object(key, ctx)?;
        }
        Item::List(list) => {
            visitor.begin_list(key, list, ctx)?;
            let list_ctx = ctx.child(key).with_schema(&list.schema);
            for node in &list.rows {
                traverse_node(node, &list.schema, visitor, &list_ctx)?;
            }
            visitor.end_list(key, list, ctx)?;
        }
    }
    Ok(())
}

/// Traverse a node and its children recursively.
fn traverse_node<V: DocumentVisitor>(
    node: &Node,
    schema: &[String],
    visitor: &mut V,
    ctx: &VisitorContext,
) -> Result<(), V::Error> {
    visitor.visit_node(node, schema, ctx)?;

    if !node.children.is_empty() {
        visitor.begin_node_children(node, ctx)?;

        let child_ctx = ctx.child(&node.id);
        for (child_type, children) in &node.children {
            // Get schema for child type from document
            let child_schema = ctx.document.structs.get(child_type);
            let child_schema = child_schema.map(|s| s.as_slice()).unwrap_or(&[]);

            for child in children {
                traverse_node(child, child_schema, visitor, &child_ctx)?;
            }
        }

        visitor.end_node_children(node, ctx)?;
    }

    Ok(())
}

/// Statistics collector visitor for testing and analysis.
#[derive(Debug, Default)]
pub struct StatsCollector {
    /// Number of scalars visited.
    pub scalar_count: usize,
    /// Number of objects visited.
    pub object_count: usize,
    /// Number of lists visited.
    pub list_count: usize,
    /// Number of nodes visited.
    pub node_count: usize,
    /// Maximum depth reached.
    pub max_depth: usize,
}

impl DocumentVisitor for StatsCollector {
    type Error = std::convert::Infallible;

    fn visit_scalar(
        &mut self,
        _key: &str,
        _value: &Value,
        ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        self.scalar_count += 1;
        self.max_depth = self.max_depth.max(ctx.depth);
        Ok(())
    }

    fn begin_object(
        &mut self,
        _key: &str,
        ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        self.object_count += 1;
        self.max_depth = self.max_depth.max(ctx.depth);
        Ok(())
    }

    fn begin_list(
        &mut self,
        _key: &str,
        _list: &MatrixList,
        ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        self.list_count += 1;
        self.max_depth = self.max_depth.max(ctx.depth);
        Ok(())
    }

    fn visit_node(
        &mut self,
        _node: &Node,
        _schema: &[String],
        ctx: &VisitorContext,
    ) -> Result<(), Self::Error> {
        self.node_count += 1;
        self.max_depth = self.max_depth.max(ctx.depth);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, Value};

    #[test]
    fn test_traverse_empty_document() {
        let hedl = "%VERSION: 1.0\n---\n";
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut stats = StatsCollector::default();
        traverse(&doc, &mut stats).unwrap();

        assert_eq!(stats.scalar_count, 0);
        assert_eq!(stats.object_count, 0);
        assert_eq!(stats.list_count, 0);
        assert_eq!(stats.node_count, 0);
    }

    #[test]
    fn test_traverse_scalars() {
        let hedl = "%VERSION: 1.0\n---\nname: Test\ncount: 42\n";
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut stats = StatsCollector::default();
        traverse(&doc, &mut stats).unwrap();

        assert_eq!(stats.scalar_count, 2);
        assert_eq!(stats.max_depth, 0);
    }

    #[test]
    fn test_traverse_nested_objects() {
        let hedl = "%VERSION: 1.0\n---\nouter:\n  inner:\n    value: 42\n";
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut stats = StatsCollector::default();
        traverse(&doc, &mut stats).unwrap();

        assert_eq!(stats.object_count, 2);
        assert_eq!(stats.scalar_count, 1);
        assert_eq!(stats.max_depth, 2);
    }

    #[test]
    fn test_traverse_list() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\nusers: @User\n  | alice, Alice\n  | bob, Bob\n";
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut stats = StatsCollector::default();
        traverse(&doc, &mut stats).unwrap();

        assert_eq!(stats.list_count, 1);
        assert_eq!(stats.node_count, 2);
    }

    #[test]
    fn test_traverse_nested_nodes() {
        let hedl = "%VERSION: 1.0\n%STRUCT: User: [id]\n%STRUCT: Post: [id]\n%NEST: User > Post\n---\nusers: @User\n  | alice\n    | post1\n    | post2\n";
        let doc = parse(hedl.as_bytes()).unwrap();

        let mut stats = StatsCollector::default();
        traverse(&doc, &mut stats).unwrap();

        assert_eq!(stats.list_count, 1);
        assert_eq!(stats.node_count, 3); // 1 user + 2 posts
    }

    #[test]
    fn test_visitor_context_path() {
        let hedl = "%VERSION: 1.0\n---\na:\n  b:\n    c: 42\n";
        let doc = parse(hedl.as_bytes()).unwrap();

        struct PathCollector {
            paths: Vec<String>,
        }

        impl DocumentVisitor for PathCollector {
            type Error = std::convert::Infallible;

            fn visit_scalar(&mut self, _key: &str, _value: &Value, ctx: &VisitorContext) -> Result<(), Self::Error> {
                self.paths.push(ctx.path_string());
                Ok(())
            }

            fn begin_object(&mut self, _key: &str, ctx: &VisitorContext) -> Result<(), Self::Error> {
                self.paths.push(ctx.path_string());
                Ok(())
            }

            fn visit_node(&mut self, _node: &Node, _schema: &[String], ctx: &VisitorContext) -> Result<(), Self::Error> {
                self.paths.push(ctx.path_string());
                Ok(())
            }
        }

        let mut collector = PathCollector { paths: Vec::new() };
        traverse(&doc, &mut collector).unwrap();

        assert!(collector.paths.contains(&"root".to_string()));
        assert!(collector.paths.contains(&"a".to_string()));
        assert!(collector.paths.contains(&"a.b".to_string()));
    }
}
