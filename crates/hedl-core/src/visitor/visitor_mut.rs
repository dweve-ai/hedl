// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Mutable visitor trait for in-place document modification.

use crate::visitor::{VisitDecision, VisitorContext};
use crate::{Document, Item, MatrixList, Node, Value};
use std::collections::BTreeMap;

/// Mutable visitor trait for in-place tree modification.
///
/// Allows visitors to modify nodes during traversal. Use with caution
/// as modifications affect subsequent visitor calls and may invalidate
/// assumptions about the tree structure.
///
/// # Safety Considerations
///
/// - Modifications are visible to subsequent method calls
/// - Changing node IDs may break reference integrity
/// - Removing children may affect nested traversal
/// - Be careful with structural changes during traversal
///
/// # Example: Normalize Field Values
///
/// ```
/// use hedl_core::visitor::{VisitorMut, VisitDecision, VisitorContext};
/// use hedl_core::{Node, Value};
///
/// struct FieldNormalizer;
///
/// impl VisitorMut for FieldNormalizer {
///     fn visit_scalar_mut(
///         &mut self,
///         _key: &str,
///         value: &mut Value,
///         _ctx: &VisitorContext<'_>,
///     ) -> VisitDecision {
///         // Normalize strings to lowercase
///         if let Value::String(s) = value {
///             *value = Value::String(s.to_lowercase().into());
///         }
///         VisitDecision::Continue
///     }
/// }
/// ```
///
/// # Example: Remove Empty Nodes
///
/// ```
/// use hedl_core::visitor::{VisitorMut, VisitDecision, VisitorContext};
/// use hedl_core::Node;
///
/// struct EmptyNodeRemover;
///
/// impl VisitorMut for EmptyNodeRemover {
///     fn visit_node_mut(&mut self, node: &mut Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
///         // Clear empty children
///         if let Some(children) = node.children_mut() {
///             children.retain(|_, nodes| !nodes.is_empty());
///         }
///         VisitDecision::Continue
///     }
/// }
/// ```
pub trait VisitorMut {
    /// Called at the start of document traversal.
    fn begin_document_mut(
        &mut self,
        _doc: &mut Document,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called at the end of document traversal.
    fn end_document_mut(
        &mut self,
        _doc: &mut Document,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called when visiting a scalar value (with mutable access).
    ///
    /// # Arguments
    ///
    /// - `key`: The key/field name for this scalar
    /// - `value`: Mutable reference to the scalar value
    /// - `ctx`: Visitor context
    fn visit_scalar_mut(
        &mut self,
        _key: &str,
        _value: &mut Value,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called before visiting an object's children (with mutable access).
    fn begin_object_mut(
        &mut self,
        _key: &str,
        _obj: &mut BTreeMap<String, Item>,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called after visiting an object's children (with mutable access).
    fn end_object_mut(
        &mut self,
        _key: &str,
        _obj: &mut BTreeMap<String, Item>,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called before visiting a list's rows (with mutable access).
    fn begin_list_mut(
        &mut self,
        _key: &str,
        _list: &mut MatrixList,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called after visiting a list's rows (with mutable access).
    fn end_list_mut(
        &mut self,
        _key: &str,
        _list: &mut MatrixList,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called when visiting a node (with mutable access).
    ///
    /// # Warning
    ///
    /// Modifying `node.id` may break reference integrity.
    /// Modifying `node.type_name` may break schema validation.
    fn visit_node_mut(&mut self, _node: &mut Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called before visiting a node's children (with mutable access).
    fn begin_node_children_mut(
        &mut self,
        _node: &mut Node,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called after visiting a node's children (with mutable access).
    fn end_node_children_mut(
        &mut self,
        _node: &mut Node,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called when visiting a reference value (with mutable access).
    fn visit_reference_mut(
        &mut self,
        _reference: &mut crate::Reference,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called when visiting an expression value (with mutable access).
    fn visit_expression_mut(
        &mut self,
        _expr: &mut crate::lex::Expression,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }

    /// Called when visiting a tensor value (with mutable access).
    fn visit_tensor_mut(
        &mut self,
        _tensor: &mut crate::lex::Tensor,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        VisitDecision::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoOpVisitorMut;
    impl VisitorMut for NoOpVisitorMut {}

    #[test]
    fn test_default_implementations() {
        let mut visitor = NoOpVisitorMut;
        let mut doc = Document::new((1, 0));
        let temp_doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&temp_doc);

        assert_eq!(
            visitor.begin_document_mut(&mut doc, &ctx),
            VisitDecision::Continue
        );
        assert_eq!(
            visitor.end_document_mut(&mut doc, &ctx),
            VisitDecision::Continue
        );
    }

    struct ValueModifier;

    impl VisitorMut for ValueModifier {
        fn visit_scalar_mut(
            &mut self,
            _key: &str,
            value: &mut Value,
            _ctx: &VisitorContext<'_>,
        ) -> VisitDecision {
            if let Value::Int(n) = value {
                *n *= 2;
            }
            VisitDecision::Continue
        }
    }

    #[test]
    fn test_mutable_visitor_can_modify_values() {
        let mut visitor = ValueModifier;
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        let mut value = Value::Int(21);
        visitor.visit_scalar_mut("key", &mut value, &ctx);

        assert_eq!(value, Value::Int(42));
    }

    struct NodeIdModifier;

    impl VisitorMut for NodeIdModifier {
        fn visit_node_mut(&mut self, node: &mut Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
            node.id = format!("modified_{}", node.id);
            VisitDecision::Continue
        }
    }

    #[test]
    fn test_mutable_visitor_can_modify_nodes() {
        let mut visitor = NodeIdModifier;
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        let mut node = Node::new("User", "alice", vec![]);
        visitor.visit_node_mut(&mut node, &ctx);

        assert_eq!(node.id, "modified_alice");
    }

    struct SkipEmptyLists;

    impl VisitorMut for SkipEmptyLists {
        fn begin_list_mut(
            &mut self,
            _key: &str,
            list: &mut MatrixList,
            _ctx: &VisitorContext<'_>,
        ) -> VisitDecision {
            if list.rows.is_empty() {
                VisitDecision::SkipChildren
            } else {
                VisitDecision::Continue
            }
        }
    }

    #[test]
    fn test_mutable_visitor_can_skip_empty_lists() {
        let mut visitor = SkipEmptyLists;
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        let mut empty_list = MatrixList::new("User", vec!["id".to_string()]);
        assert_eq!(
            visitor.begin_list_mut("users", &mut empty_list, &ctx),
            VisitDecision::SkipChildren
        );

        let mut non_empty_list = MatrixList::new("User", vec!["id".to_string()]);
        non_empty_list.add_row(Node::new("User", "1", vec![]));
        assert_eq!(
            visitor.begin_list_mut("users", &mut non_empty_list, &ctx),
            VisitDecision::Continue
        );
    }

    struct ObjectCleaner;

    impl VisitorMut for ObjectCleaner {
        fn end_object_mut(
            &mut self,
            _key: &str,
            obj: &mut BTreeMap<String, Item>,
            _ctx: &VisitorContext<'_>,
        ) -> VisitDecision {
            // Remove all null scalars
            obj.retain(|_, item| !matches!(item, Item::Scalar(Value::Null)));
            VisitDecision::Continue
        }
    }

    #[test]
    fn test_mutable_visitor_can_clean_objects() {
        let mut visitor = ObjectCleaner;
        let doc = Document::new((1, 0));
        let ctx = VisitorContext::new(&doc);

        let mut obj = BTreeMap::new();
        obj.insert("a".to_string(), Item::Scalar(Value::Int(1)));
        obj.insert("b".to_string(), Item::Scalar(Value::Null));
        obj.insert("c".to_string(), Item::Scalar(Value::String("test".into())));

        visitor.end_object_mut("obj", &mut obj, &ctx);

        assert_eq!(obj.len(), 2);
        assert!(obj.contains_key("a"));
        assert!(!obj.contains_key("b"));
        assert!(obj.contains_key("c"));
    }
}
