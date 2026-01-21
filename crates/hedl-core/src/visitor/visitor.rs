// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Immutable visitor trait for read-only document traversal.

use crate::visitor::{VisitDecision, VisitorContext};
use crate::{Document, MatrixList, Node, Value};

/// Immutable visitor trait for read-only tree traversal.
///
/// This is the primary visitor trait for analyzing and inspecting
/// HEDL documents without modification. All methods have default
/// implementations that return `Continue`, allowing implementations
/// to override only the methods they need.
///
/// # Control Flow
///
/// Methods return [`VisitDecision`] to control traversal:
/// - `Continue`: Visit this element and its children
/// - `SkipChildren`: Visit this element but skip its children
/// - `Stop`: Terminate traversal immediately
///
/// # Example: Count Nodes by Type
///
/// ```
/// use hedl_core::visitor::{Visitor, VisitDecision, VisitorContext};
/// use hedl_core::Node;
/// use std::collections::HashMap;
///
/// struct TypeCounter {
///     counts: HashMap<String, usize>,
/// }
///
/// impl Visitor for TypeCounter {
///     fn visit_node(&mut self, node: &Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
///         *self.counts.entry(node.type_name.clone()).or_insert(0) += 1;
///         VisitDecision::Continue
///     }
/// }
/// ```
///
/// # Example: Find First Match
///
/// ```
/// use hedl_core::visitor::{Visitor, VisitDecision, VisitorContext};
/// use hedl_core::Node;
///
/// struct FindUser {
///     target: String,
///     found: Option<String>,
/// }
///
/// impl Visitor for FindUser {
///     fn visit_node(&mut self, node: &Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
///         if node.type_name == "User" && node.id == self.target {
///             self.found = Some(node.id.clone());
///             VisitDecision::Stop  // Early termination
///         } else {
///             VisitDecision::Continue
///         }
///     }
/// }
/// ```
pub trait Visitor {
    /// Called at the start of document traversal.
    ///
    /// # Arguments
    ///
    /// - `doc`: The document being traversed
    /// - `ctx`: Visitor context with path and depth information
    ///
    /// # Returns
    ///
    /// `VisitDecision` to control whether to continue traversal.
    fn begin_document(&mut self, _doc: &Document, _ctx: &VisitorContext<'_>) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called at the end of document traversal.
    ///
    /// This is called after all root items have been visited, even if
    /// some traversal was skipped via `SkipChildren`.
    fn end_document(&mut self, _doc: &Document, _ctx: &VisitorContext<'_>) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called when visiting a scalar value.
    ///
    /// # Arguments
    ///
    /// - `key`: The key/field name for this scalar
    /// - `value`: The scalar value
    /// - `ctx`: Visitor context
    fn visit_scalar(
        &mut self,
        _key: &str,
        _value: &Value,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called before visiting an object's children.
    ///
    /// Return `SkipChildren` to skip the object's contents.
    fn begin_object(&mut self, _key: &str, _ctx: &VisitorContext<'_>) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called after visiting an object's children.
    fn end_object(&mut self, _key: &str, _ctx: &VisitorContext<'_>) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called before visiting a list's rows.
    ///
    /// # Arguments
    ///
    /// - `key`: The key for this list
    /// - `list`: The matrix list with schema and rows
    /// - `ctx`: Visitor context
    ///
    /// Return `SkipChildren` to skip all rows in the list.
    fn begin_list(
        &mut self,
        _key: &str,
        _list: &MatrixList,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called after visiting a list's rows.
    fn end_list(
        &mut self,
        _key: &str,
        _list: &MatrixList,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called when visiting a node (row) in a list.
    ///
    /// This is called for both top-level list rows and nested child nodes.
    ///
    /// # Arguments
    ///
    /// - `node`: The node being visited
    /// - `ctx`: Visitor context with current path and depth
    fn visit_node(&mut self, _node: &Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called before visiting a node's children.
    ///
    /// Return `SkipChildren` to skip nested child nodes.
    fn begin_node_children(&mut self, _node: &Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called after visiting a node's children.
    fn end_node_children(&mut self, _node: &Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called when visiting a reference value.
    ///
    /// This is called for `Value::Reference` instances.
    fn visit_reference(
        &mut self,
        _reference: &crate::Reference,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called when visiting an expression value.
    ///
    /// This is called for `Value::Expression` instances.
    fn visit_expression(
        &mut self,
        _expr: &crate::lex::Expression,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called when visiting a tensor value.
    ///
    /// This is called for `Value::Tensor` instances.
    fn visit_tensor(
        &mut self,
        _tensor: &crate::lex::Tensor,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoOpVisitor;
    impl Visitor for NoOpVisitor {}

    #[test]
    fn test_default_implementations_return_continue() {
        let mut visitor = NoOpVisitor;
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        assert_eq!(visitor.begin_document(&doc, &ctx), VisitDecision::Continue);
        assert_eq!(visitor.end_document(&doc, &ctx), VisitDecision::Continue);
        assert_eq!(
            visitor.visit_scalar("key", &Value::Null, &ctx),
            VisitDecision::Continue
        );
        assert_eq!(visitor.begin_object("key", &ctx), VisitDecision::Continue);
        assert_eq!(visitor.end_object("key", &ctx), VisitDecision::Continue);
    }

    struct CountingVisitor {
        scalar_count: usize,
        node_count: usize,
    }

    impl Visitor for CountingVisitor {
        fn visit_scalar(&mut self, _: &str, _: &Value, _: &VisitorContext<'_>) -> VisitDecision {
            self.scalar_count += 1;
            VisitDecision::Continue
        }

        fn visit_node(&mut self, _: &Node, _: &VisitorContext<'_>) -> VisitDecision {
            self.node_count += 1;
            VisitDecision::Continue
        }
    }

    #[test]
    fn test_visitor_can_count_elements() {
        let mut visitor = CountingVisitor {
            scalar_count: 0,
            node_count: 0,
        };

        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        visitor.visit_scalar("key", &Value::Int(42), &ctx);
        visitor.visit_scalar("key2", &Value::String("test".into()), &ctx);

        let node = Node::new("User", "1", vec![]);
        visitor.visit_node(&node, &ctx);

        assert_eq!(visitor.scalar_count, 2);
        assert_eq!(visitor.node_count, 1);
    }

    struct EarlyStopVisitor {
        stop_after: usize,
        count: usize,
    }

    impl Visitor for EarlyStopVisitor {
        fn visit_node(&mut self, _: &Node, _: &VisitorContext<'_>) -> VisitDecision {
            self.count += 1;
            if self.count >= self.stop_after {
                VisitDecision::Stop
            } else {
                VisitDecision::Continue
            }
        }
    }

    #[test]
    fn test_visitor_can_stop_early() {
        let mut visitor = EarlyStopVisitor {
            stop_after: 2,
            count: 0,
        };

        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        let node = Node::new("User", "1", vec![]);

        assert_eq!(visitor.visit_node(&node, &ctx), VisitDecision::Continue);
        assert_eq!(visitor.visit_node(&node, &ctx), VisitDecision::Stop);
        assert_eq!(visitor.count, 2);
    }

    struct SkipChildrenVisitor;

    impl Visitor for SkipChildrenVisitor {
        fn begin_object(&mut self, _: &str, _: &VisitorContext<'_>) -> VisitDecision {
            VisitDecision::SkipChildren
        }
    }

    #[test]
    fn test_visitor_can_skip_children() {
        let mut visitor = SkipChildrenVisitor;
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        assert_eq!(
            visitor.begin_object("obj", &ctx),
            VisitDecision::SkipChildren
        );
    }
}
