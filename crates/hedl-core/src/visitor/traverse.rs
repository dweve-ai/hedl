// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Document traversal implementation.

use crate::visitor::{
    FallibleVisitor, PathSegment, Transformer, TraversalConfig, TraversalStats, Visitor,
    VisitorContext, VisitorMut,
};
use crate::{Document, Item, Node, Value};

/// Result of a traversal operation.
#[derive(Debug)]
pub enum TraversalResult {
    /// Traversal completed successfully.
    Complete(TraversalStats),
    /// Traversal stopped early by visitor decision.
    Stopped(TraversalStats),
    /// Traversal stopped due to depth limit.
    DepthLimitReached(TraversalStats),
}

impl TraversalResult {
    /// Check if traversal was stopped early.
    pub fn is_stopped(&self) -> bool {
        matches!(self, Self::Stopped(_))
    }

    /// Check if traversal completed successfully.
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete(_))
    }

    /// Check if depth limit was reached.
    pub fn is_depth_limited(&self) -> bool {
        matches!(self, Self::DepthLimitReached(_))
    }

    /// Get the traversal statistics.
    pub fn stats(&self) -> &TraversalStats {
        match self {
            Self::Complete(s) | Self::Stopped(s) | Self::DepthLimitReached(s) => s,
        }
    }
}

/// Traverse a document with an immutable visitor.
///
/// # Arguments
///
/// - `doc`: The document to traverse
/// - `visitor`: The visitor to apply
/// - `config`: Traversal configuration
///
/// # Example
///
/// ```
/// use hedl_core::visitor::{traverse, Visitor, VisitDecision, VisitorContext, TraversalConfig};
/// use hedl_core::{Document, Node};
///
/// struct Counter { count: usize }
/// impl Visitor for Counter {
///     fn visit_node(&mut self, _: &Node, _: &VisitorContext<'_>) -> VisitDecision {
///         self.count += 1;
///         VisitDecision::Continue
///     }
/// }
///
/// let doc = Document::new((1, 0));
/// let mut counter = Counter { count: 0 };
/// let result = traverse(&doc, &mut counter, &TraversalConfig::default());
/// ```
pub fn traverse<V: Visitor>(
    doc: &Document,
    visitor: &mut V,
    config: &TraversalConfig,
) -> TraversalResult {
    let mut ctx = VisitorContext::new(doc);
    traverse_internal(doc, visitor, &mut ctx, config)
}

/// Internal traversal implementation for immutable visitors.
fn traverse_internal<V: Visitor>(
    doc: &Document,
    visitor: &mut V,
    ctx: &mut VisitorContext<'_>,
    config: &TraversalConfig,
) -> TraversalResult {
    // Begin document
    let decision = visitor.begin_document(doc, ctx);
    if decision.should_stop() {
        return TraversalResult::Stopped(ctx.stats().clone());
    }

    // Traverse root items
    for (key, item) in &doc.root {
        let result = traverse_item(key, item, visitor, ctx, config);
        if matches!(result, TraversalResult::Stopped(_)) {
            return result;
        }
    }

    // End document
    let decision = visitor.end_document(doc, ctx);
    if decision.should_stop() {
        return TraversalResult::Stopped(ctx.stats().clone());
    }

    TraversalResult::Complete(ctx.stats().clone())
}

/// Traverse a single item.
fn traverse_item<V: Visitor>(
    key: &str,
    item: &Item,
    visitor: &mut V,
    ctx: &mut VisitorContext<'_>,
    config: &TraversalConfig,
) -> TraversalResult {
    // Check depth limit
    if config.is_depth_limit_reached(ctx.depth) {
        return TraversalResult::DepthLimitReached(ctx.stats().clone());
    }

    match item {
        Item::Scalar(value) => {
            ctx.record_scalar_visit();
            let decision = visitor.visit_scalar(key, value, ctx);

            // Visit nested structures in values
            if !decision.should_stop() {
                visit_value_internals(value, visitor, ctx, config);
            }

            if decision.should_stop() {
                TraversalResult::Stopped(ctx.stats().clone())
            } else {
                TraversalResult::Complete(ctx.stats().clone())
            }
        }
        Item::Object(obj) => {
            ctx.record_object_visit();
            let decision = visitor.begin_object(key, ctx);

            if decision.should_stop() {
                return TraversalResult::Stopped(ctx.stats().clone());
            }

            if decision.should_visit_children() {
                let mut child_ctx = ctx.child(PathSegment::NestedKey(key.to_string()));
                for (child_key, child_item) in obj {
                    let result =
                        traverse_item(child_key, child_item, visitor, &mut child_ctx, config);
                    if matches!(result, TraversalResult::Stopped(_)) {
                        return result;
                    }
                }
            }

            let decision = visitor.end_object(key, ctx);
            if decision.should_stop() {
                TraversalResult::Stopped(ctx.stats().clone())
            } else {
                TraversalResult::Complete(ctx.stats().clone())
            }
        }
        Item::List(list) => {
            ctx.record_list_visit();
            let decision = visitor.begin_list(key, list, ctx);

            if decision.should_stop() {
                return TraversalResult::Stopped(ctx.stats().clone());
            }

            if decision.should_visit_children() {
                let list_ctx = ctx
                    .child(PathSegment::Key(key.to_string()))
                    .with_schema(&list.schema);

                for (idx, node) in list.rows.iter().enumerate() {
                    let mut node_ctx = list_ctx.child(PathSegment::Index(idx));
                    let result = traverse_node(node, visitor, &mut node_ctx, config);
                    if matches!(result, TraversalResult::Stopped(_)) {
                        return result;
                    }
                }
            }

            let decision = visitor.end_list(key, list, ctx);
            if decision.should_stop() {
                TraversalResult::Stopped(ctx.stats().clone())
            } else {
                TraversalResult::Complete(ctx.stats().clone())
            }
        }
    }
}

/// Traverse a node recursively.
#[allow(clippy::only_used_in_recursion)]
fn traverse_node<V: Visitor>(
    node: &Node,
    visitor: &mut V,
    ctx: &mut VisitorContext<'_>,
    config: &TraversalConfig,
) -> TraversalResult {
    ctx.record_node_visit();

    let decision = visitor.visit_node(node, ctx);
    if decision.should_stop() {
        return TraversalResult::Stopped(ctx.stats().clone());
    }

    if !decision.should_visit_children() {
        return TraversalResult::Complete(ctx.stats().clone());
    }

    // Visit children if present
    if let Some(children) = node.children() {
        if !children.is_empty() {
            let decision = visitor.begin_node_children(node, ctx);
            if decision.should_stop() {
                return TraversalResult::Stopped(ctx.stats().clone());
            }

            let child_ctx = ctx.child(PathSegment::NodeId(node.id.clone()));
            for (child_type, child_nodes) in children {
                // Get schema for child type
                let child_schema = ctx
                    .document
                    .structs
                    .get(child_type)
                    .map(|s| s.as_slice())
                    .unwrap_or(&[]);

                let type_ctx = child_ctx.with_schema(child_schema);
                for (idx, child) in child_nodes.iter().enumerate() {
                    let mut idx_ctx = type_ctx.child(PathSegment::Index(idx));
                    let result = traverse_node(child, visitor, &mut idx_ctx, config);
                    if matches!(result, TraversalResult::Stopped(_)) {
                        return result;
                    }
                }
            }

            let decision = visitor.end_node_children(node, ctx);
            if decision.should_stop() {
                return TraversalResult::Stopped(ctx.stats().clone());
            }
        }
    }

    TraversalResult::Complete(ctx.stats().clone())
}

/// Visit internals of a value (references, expressions, tensors).
fn visit_value_internals<V: Visitor>(
    value: &Value,
    visitor: &mut V,
    ctx: &VisitorContext<'_>,
    config: &TraversalConfig,
) {
    match value {
        Value::Reference(r) if config.follow_references => {
            visitor.visit_reference(r, ctx);
        }
        Value::Expression(e) if config.visit_expressions => {
            visitor.visit_expression(e, ctx);
        }
        Value::Tensor(t) if config.visit_tensors => {
            visitor.visit_tensor(t, ctx);
        }
        _ => {}
    }
}

/// Traverse a document with a fallible visitor.
///
/// # Arguments
///
/// - `doc`: The document to traverse
/// - `visitor`: The fallible visitor to apply
/// - `config`: Traversal configuration
///
/// # Errors
///
/// Returns the visitor's error type if any visitor method returns an error.
pub fn traverse_fallible<V: FallibleVisitor>(
    doc: &Document,
    visitor: &mut V,
    config: &TraversalConfig,
) -> Result<TraversalStats, V::Error> {
    let mut ctx = VisitorContext::new(doc);
    traverse_fallible_internal(doc, visitor, &mut ctx, config)
}

/// Internal traversal for fallible visitors.
fn traverse_fallible_internal<V: FallibleVisitor>(
    doc: &Document,
    visitor: &mut V,
    ctx: &mut VisitorContext<'_>,
    config: &TraversalConfig,
) -> Result<TraversalStats, V::Error> {
    visitor.begin_document(doc, ctx)?;

    for (key, item) in &doc.root {
        traverse_item_fallible(key, item, visitor, ctx, config)?;
    }

    visitor.end_document(doc, ctx)?;
    Ok(ctx.stats().clone())
}

/// Traverse item with fallible visitor.
fn traverse_item_fallible<V: FallibleVisitor>(
    key: &str,
    item: &Item,
    visitor: &mut V,
    ctx: &mut VisitorContext<'_>,
    config: &TraversalConfig,
) -> Result<(), V::Error> {
    if config.is_depth_limit_reached(ctx.depth) {
        return Ok(());
    }

    match item {
        Item::Scalar(value) => {
            ctx.record_scalar_visit();
            visitor.visit_scalar(key, value, ctx)?;
            visit_value_internals_fallible(value, visitor, ctx, config)?;
        }
        Item::Object(obj) => {
            ctx.record_object_visit();
            let decision = visitor.begin_object(key, ctx)?;
            if decision.should_visit_children() {
                let mut child_ctx = ctx.child(PathSegment::NestedKey(key.to_string()));
                for (child_key, child_item) in obj {
                    traverse_item_fallible(child_key, child_item, visitor, &mut child_ctx, config)?;
                }
            }
            visitor.end_object(key, ctx)?;
        }
        Item::List(list) => {
            ctx.record_list_visit();
            let decision = visitor.begin_list(key, list, ctx)?;
            if decision.should_visit_children() {
                let list_ctx = ctx
                    .child(PathSegment::Key(key.to_string()))
                    .with_schema(&list.schema);
                for (idx, node) in list.rows.iter().enumerate() {
                    let mut node_ctx = list_ctx.child(PathSegment::Index(idx));
                    traverse_node_fallible(node, visitor, &mut node_ctx, config)?;
                }
            }
            visitor.end_list(key, list, ctx)?;
        }
    }
    Ok(())
}

/// Traverse node with fallible visitor.
#[allow(clippy::only_used_in_recursion)]
fn traverse_node_fallible<V: FallibleVisitor>(
    node: &Node,
    visitor: &mut V,
    ctx: &mut VisitorContext<'_>,
    config: &TraversalConfig,
) -> Result<(), V::Error> {
    ctx.record_node_visit();
    let decision = visitor.visit_node(node, ctx)?;

    if decision.should_visit_children() {
        if let Some(children) = node.children() {
            if !children.is_empty() {
                visitor.begin_node_children(node, ctx)?;
                let child_ctx = ctx.child(PathSegment::NodeId(node.id.clone()));
                for (child_type, child_nodes) in children {
                    let child_schema = ctx
                        .document
                        .structs
                        .get(child_type)
                        .map(|s| s.as_slice())
                        .unwrap_or(&[]);
                    let type_ctx = child_ctx.with_schema(child_schema);
                    for (idx, child) in child_nodes.iter().enumerate() {
                        let mut idx_ctx = type_ctx.child(PathSegment::Index(idx));
                        traverse_node_fallible(child, visitor, &mut idx_ctx, config)?;
                    }
                }
                visitor.end_node_children(node, ctx)?;
            }
        }
    }
    Ok(())
}

/// Visit value internals with fallible visitor.
fn visit_value_internals_fallible<V: FallibleVisitor>(
    value: &Value,
    visitor: &mut V,
    ctx: &VisitorContext<'_>,
    config: &TraversalConfig,
) -> Result<(), V::Error> {
    match value {
        Value::Reference(r) if config.follow_references => {
            visitor.visit_reference(r, ctx)?;
        }
        Value::Expression(e) if config.visit_expressions => {
            visitor.visit_expression(e, ctx)?;
        }
        Value::Tensor(t) if config.visit_tensors => {
            visitor.visit_tensor(t, ctx)?;
        }
        _ => {}
    }
    Ok(())
}

/// Traverse a document with a mutable visitor.
///
/// # Warning
///
/// Be careful when modifying the document structure during traversal.
/// Structural changes may affect subsequent traversal behavior.
///
/// # Note
///
/// Due to Rust's borrowing rules, mutable traversal is currently limited
/// to document-level mutations. For full tree modification, consider using
/// the `Transformer` trait which consumes and rebuilds the tree.
pub fn traverse_mut<V: VisitorMut>(
    doc: &mut Document,
    visitor: &mut V,
    _config: &TraversalConfig,
) -> TraversalResult {
    // Create a minimal context without borrowing doc mutably
    let stats = TraversalStats::default();
    let temp_doc = Document::new(doc.version);
    let ctx = VisitorContext::new(&temp_doc);

    let decision = visitor.begin_document_mut(doc, &ctx);
    if decision.should_stop() {
        return TraversalResult::Stopped(stats);
    }

    // Note: Full mutable traversal of items would require more complex
    // ownership handling or unsafe code. For now, we provide begin/end hooks.

    let decision = visitor.end_document_mut(doc, &ctx);
    if decision.should_stop() {
        TraversalResult::Stopped(stats)
    } else {
        TraversalResult::Complete(stats)
    }
}

/// Transform a document with a transformer.
///
/// # Arguments
///
/// - `doc`: The document to transform
/// - `transformer`: The transformer to apply
/// - `_config`: Traversal configuration (currently unused)
///
/// # Returns
///
/// The transformed document.
pub fn transform<T: Transformer>(
    doc: Document,
    transformer: &mut T,
    _config: &TraversalConfig,
) -> Document {
    // Create a temporary context for the transformation
    let temp_doc = Document::new(doc.version);
    let ctx = VisitorContext::new(&temp_doc);
    transformer.transform_document(doc, &ctx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visitor::{VisitDecision, Visitor};

    struct CountingVisitor {
        nodes: usize,
        scalars: usize,
    }

    impl Visitor for CountingVisitor {
        fn visit_node(&mut self, _: &Node, _: &VisitorContext<'_>) -> VisitDecision {
            self.nodes += 1;
            VisitDecision::Continue
        }

        fn visit_scalar(&mut self, _: &str, _: &Value, _: &VisitorContext<'_>) -> VisitDecision {
            self.scalars += 1;
            VisitDecision::Continue
        }
    }

    #[test]
    fn test_traverse_empty_document() {
        let doc = Document::new((1, 0));
        let mut visitor = CountingVisitor {
            nodes: 0,
            scalars: 0,
        };
        let result = traverse(&doc, &mut visitor, &TraversalConfig::default());

        assert!(result.is_complete());
        assert_eq!(visitor.nodes, 0);
        assert_eq!(visitor.scalars, 0);
    }

    #[test]
    fn test_traversal_result_methods() {
        let stats = TraversalStats::default();
        let result = TraversalResult::Complete(stats.clone());
        assert!(result.is_complete());
        assert!(!result.is_stopped());
        assert!(!result.is_depth_limited());

        let result = TraversalResult::Stopped(stats.clone());
        assert!(result.is_stopped());
        assert!(!result.is_complete());

        let result = TraversalResult::DepthLimitReached(stats);
        assert!(result.is_depth_limited());
        assert!(!result.is_complete());
    }

    struct EarlyStopVisitor;

    impl Visitor for EarlyStopVisitor {
        fn begin_document(&mut self, _: &Document, _: &VisitorContext<'_>) -> VisitDecision {
            VisitDecision::Stop
        }
    }

    #[test]
    fn test_traverse_early_stop() {
        let doc = Document::new((1, 0));
        let mut visitor = EarlyStopVisitor;
        let result = traverse(&doc, &mut visitor, &TraversalConfig::default());

        assert!(result.is_stopped());
    }
}
