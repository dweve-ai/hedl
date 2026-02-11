// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Fallible visitor trait with error handling.

use crate::visitor::{VisitDecision, VisitorContext};
use crate::{Document, MatrixList, Node, Value};

/// Fallible visitor trait with error handling.
///
/// Similar to [`Visitor`](crate::visitor::Visitor), but methods return
/// `Result<VisitDecision, E>` to support error propagation during traversal.
/// Useful for validation, parsing, or operations that may fail.
///
/// # Example: Validation Visitor
///
/// ```
/// use hedl_core::visitor::{FallibleVisitor, VisitDecision, VisitorContext};
/// use hedl_core::Node;
///
/// #[derive(Debug)]
/// enum ValidationError {
///     InvalidId(String),
///     MissingField,
/// }
///
/// struct NodeValidator;
///
/// impl FallibleVisitor for NodeValidator {
///     type Error = ValidationError;
///
///     fn visit_node(
///         &mut self,
///         node: &Node,
///         _ctx: &VisitorContext<'_>,
///     ) -> Result<VisitDecision, Self::Error> {
///         if node.id.is_empty() {
///             Err(ValidationError::InvalidId(node.type_name.clone()))
///         } else if node.fields.is_empty() {
///             Err(ValidationError::MissingField)
///         } else {
///             Ok(VisitDecision::Continue)
///         }
///     }
/// }
/// ```
///
/// # Example: Reference Resolver
///
/// ```
/// use hedl_core::visitor::{FallibleVisitor, VisitDecision, VisitorContext};
/// use hedl_core::Reference;
///
/// #[derive(Debug)]
/// struct ReferenceNotFound(String);
///
/// struct ReferenceResolver {
///     valid_ids: std::collections::HashSet<String>,
/// }
///
/// impl FallibleVisitor for ReferenceResolver {
///     type Error = ReferenceNotFound;
///
///     fn visit_reference(
///         &mut self,
///         reference: &Reference,
///         _ctx: &VisitorContext<'_>,
///     ) -> Result<VisitDecision, Self::Error> {
///         // reference.id is Box<str>, convert to &str for comparison
///         if self.valid_ids.contains(reference.id.as_ref()) {
///             Ok(VisitDecision::Continue)
///         } else {
///             Err(ReferenceNotFound(reference.id.to_string()))
///         }
///     }
/// }
/// ```
pub trait FallibleVisitor {
    /// Error type for this visitor.
    type Error;

    /// Called at the start of document traversal.
    fn begin_document(
        &mut self,
        _doc: &Document,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called at the end of document traversal.
    fn end_document(
        &mut self,
        _doc: &Document,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called when visiting a scalar value.
    fn visit_scalar(
        &mut self,
        _key: &str,
        _value: &Value,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called before visiting an object's children.
    fn begin_object(
        &mut self,
        _key: &str,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called after visiting an object's children.
    fn end_object(
        &mut self,
        _key: &str,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called before visiting a list's rows.
    fn begin_list(
        &mut self,
        _key: &str,
        _list: &MatrixList,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called after visiting a list's rows.
    fn end_list(
        &mut self,
        _key: &str,
        _list: &MatrixList,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called when visiting a node.
    fn visit_node(
        &mut self,
        _node: &Node,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called before visiting a node's children.
    fn begin_node_children(
        &mut self,
        _node: &Node,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called after visiting a node's children.
    fn end_node_children(
        &mut self,
        _node: &Node,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called when visiting a reference value.
    fn visit_reference(
        &mut self,
        _reference: &crate::Reference,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called when visiting an expression value.
    fn visit_expression(
        &mut self,
        _expr: &crate::lex::Expression,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }

    /// Called when visiting a tensor value.
    fn visit_tensor(
        &mut self,
        _tensor: &crate::lex::Tensor,
        _ctx: &VisitorContext<'_>,
    ) -> Result<VisitDecision, Self::Error> {
        Ok(VisitDecision::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoOpFallibleVisitor;
    impl FallibleVisitor for NoOpFallibleVisitor {
        type Error = String;
    }

    #[test]
    fn test_default_implementations_return_ok_continue() {
        let mut visitor = NoOpFallibleVisitor;
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        assert_eq!(
            visitor.begin_document(&doc, &ctx).unwrap(),
            VisitDecision::Continue
        );
        assert_eq!(
            visitor.end_document(&doc, &ctx).unwrap(),
            VisitDecision::Continue
        );
        assert_eq!(
            visitor.visit_scalar("key", &Value::Null, &ctx).unwrap(),
            VisitDecision::Continue
        );
    }

    #[derive(Debug, PartialEq)]
    enum TestError {
        NullNotAllowed,
        EmptyId,
    }

    struct StrictValidator;

    impl FallibleVisitor for StrictValidator {
        type Error = TestError;

        fn visit_scalar(
            &mut self,
            _key: &str,
            value: &Value,
            _ctx: &VisitorContext<'_>,
        ) -> Result<VisitDecision, Self::Error> {
            if matches!(value, Value::Null) {
                Err(TestError::NullNotAllowed)
            } else {
                Ok(VisitDecision::Continue)
            }
        }

        fn visit_node(
            &mut self,
            node: &Node,
            _ctx: &VisitorContext<'_>,
        ) -> Result<VisitDecision, Self::Error> {
            if node.id.is_empty() {
                Err(TestError::EmptyId)
            } else {
                Ok(VisitDecision::Continue)
            }
        }
    }

    #[test]
    fn test_fallible_visitor_returns_error_on_null() {
        let mut visitor = StrictValidator;
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        let result = visitor.visit_scalar("key", &Value::Null, &ctx);
        assert_eq!(result, Err(TestError::NullNotAllowed));
    }

    #[test]
    fn test_fallible_visitor_accepts_non_null() {
        let mut visitor = StrictValidator;
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        let result = visitor.visit_scalar("key", &Value::Int(42), &ctx);
        assert_eq!(result, Ok(VisitDecision::Continue));
    }

    #[test]
    fn test_fallible_visitor_validates_node_ids() {
        let mut visitor = StrictValidator;
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        let empty_id_node = Node::new("User", "", vec![]);
        let result = visitor.visit_node(&empty_id_node, &ctx);
        assert_eq!(result, Err(TestError::EmptyId));

        let valid_node = Node::new("User", "alice", vec![]);
        let result = visitor.visit_node(&valid_node, &ctx);
        assert_eq!(result, Ok(VisitDecision::Continue));
    }

    struct EarlyStopOnError;

    impl FallibleVisitor for EarlyStopOnError {
        type Error = String;

        fn visit_node(
            &mut self,
            node: &Node,
            _ctx: &VisitorContext<'_>,
        ) -> Result<VisitDecision, Self::Error> {
            if node.type_name == "BadType" {
                Ok(VisitDecision::Stop)
            } else {
                Ok(VisitDecision::Continue)
            }
        }
    }

    #[test]
    fn test_fallible_visitor_can_stop_early() {
        let mut visitor = EarlyStopOnError;
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        let normal_node = Node::new("User", "1", vec![]);
        assert_eq!(
            visitor.visit_node(&normal_node, &ctx).unwrap(),
            VisitDecision::Continue
        );

        let bad_node = Node::new("BadType", "1", vec![]);
        assert_eq!(
            visitor.visit_node(&bad_node, &ctx).unwrap(),
            VisitDecision::Stop
        );
    }

    struct CountingValidator {
        count: usize,
        max_count: usize,
    }

    impl FallibleVisitor for CountingValidator {
        type Error = String;

        fn visit_node(
            &mut self,
            _node: &Node,
            _ctx: &VisitorContext<'_>,
        ) -> Result<VisitDecision, Self::Error> {
            self.count += 1;
            if self.count > self.max_count {
                Err(format!("Too many nodes: {}", self.count))
            } else {
                Ok(VisitDecision::Continue)
            }
        }
    }

    #[test]
    fn test_fallible_visitor_error_propagation() {
        let mut visitor = CountingValidator {
            count: 0,
            max_count: 2,
        };
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        let node = Node::new("User", "1", vec![]);

        assert!(visitor.visit_node(&node, &ctx).is_ok());
        assert!(visitor.visit_node(&node, &ctx).is_ok());
        assert!(visitor.visit_node(&node, &ctx).is_err());
    }
}
