// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Utility visitors for common patterns.

use crate::visitor::{VisitDecision, Visitor, VisitorContext};
use crate::{Node, Reference, Value};
use std::collections::HashMap;

/// Visitor that counts maximum depth reached.
///
/// # Example
///
/// ```
/// use hedl_core::visitor::{collectors::DepthCounter, traverse, TraversalConfig};
/// use hedl_core::Document;
///
/// let doc = Document::new((2, 0));
/// let mut counter = DepthCounter::new();
/// traverse(&doc, &mut counter, &TraversalConfig::default());
/// assert_eq!(counter.max_depth(), 0);
/// ```
#[derive(Debug, Default)]
pub struct DepthCounter {
    max_depth: usize,
}

impl DepthCounter {
    /// Create a new depth counter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the maximum depth reached.
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
}

impl Visitor for DepthCounter {
    fn visit_node(&mut self, _node: &Node, ctx: &VisitorContext<'_>) -> VisitDecision {
        self.max_depth = self.max_depth.max(ctx.depth);
        VisitDecision::Continue
    }

    fn visit_scalar(
        &mut self,
        _key: &str,
        _value: &Value,
        ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        self.max_depth = self.max_depth.max(ctx.depth);
        VisitDecision::Continue
    }
}

/// Visitor that collects all nodes of a specific type.
///
/// # Example
///
/// ```
/// use hedl_core::visitor::{collectors::NodeCollector, traverse, TraversalConfig};
/// use hedl_core::{Document, Node};
///
/// let doc = Document::new((2, 0));
/// let mut collector = NodeCollector::new("User");
/// traverse(&doc, &mut collector, &TraversalConfig::default());
/// let users = collector.into_nodes();
/// ```
#[derive(Debug)]
pub struct NodeCollector {
    type_filter: Option<String>,
    nodes: Vec<Node>,
}

impl NodeCollector {
    /// Create a collector for all nodes.
    pub fn new_all() -> Self {
        Self {
            type_filter: None,
            nodes: Vec::new(),
        }
    }

    /// Create a collector for a specific node type.
    pub fn new(type_name: impl Into<String>) -> Self {
        Self {
            type_filter: Some(type_name.into()),
            nodes: Vec::new(),
        }
    }

    /// Get the collected nodes.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Consume the collector and return collected nodes.
    pub fn into_nodes(self) -> Vec<Node> {
        self.nodes
    }

    /// Get the count of collected nodes.
    pub fn count(&self) -> usize {
        self.nodes.len()
    }
}

impl Visitor for NodeCollector {
    fn visit_node(&mut self, node: &Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
        let should_collect = if let Some(ref filter) = self.type_filter {
            &node.type_name == filter
        } else {
            true
        };

        if should_collect {
            self.nodes.push(node.clone());
        }

        VisitDecision::Continue
    }
}

/// Visitor that collects all unique paths in the document.
///
/// # Example
///
/// ```
/// use hedl_core::visitor::{collectors::PathCollector, traverse, TraversalConfig};
/// use hedl_core::Document;
///
/// let doc = Document::new((2, 0));
/// let mut collector = PathCollector::new();
/// traverse(&doc, &mut collector, &TraversalConfig::default());
/// let paths = collector.paths();
/// ```
#[derive(Debug, Default)]
pub struct PathCollector {
    paths: Vec<String>,
}

impl PathCollector {
    /// Create a new path collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the collected paths.
    pub fn paths(&self) -> &[String] {
        &self.paths
    }

    /// Consume the collector and return collected paths.
    pub fn into_paths(self) -> Vec<String> {
        self.paths
    }
}

impl Visitor for PathCollector {
    fn visit_node(&mut self, _node: &Node, ctx: &VisitorContext<'_>) -> VisitDecision {
        self.paths.push(ctx.path_string());
        VisitDecision::Continue
    }

    fn visit_scalar(
        &mut self,
        _key: &str,
        _value: &Value,
        ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        self.paths.push(ctx.path_string());
        VisitDecision::Continue
    }
}

/// Visitor that collects all references in the document.
///
/// # Example
///
/// ```
/// use hedl_core::visitor::{collectors::ReferenceCollector, traverse, TraversalConfig};
/// use hedl_core::Document;
///
/// let doc = Document::new((2, 0));
/// let mut collector = ReferenceCollector::new();
/// traverse(&doc, &mut collector, &TraversalConfig::default());
/// let refs = collector.references();
/// ```
#[derive(Debug, Default)]
pub struct ReferenceCollector {
    references: Vec<Reference>,
    by_type: HashMap<String, Vec<String>>,
}

impl ReferenceCollector {
    /// Create a new reference collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get all collected references.
    pub fn references(&self) -> &[Reference] {
        &self.references
    }

    /// Get references grouped by type.
    pub fn by_type(&self) -> &HashMap<String, Vec<String>> {
        &self.by_type
    }

    /// Get the count of collected references.
    pub fn count(&self) -> usize {
        self.references.len()
    }

    /// Consume the collector and return collected references.
    pub fn into_references(self) -> Vec<Reference> {
        self.references
    }
}

impl Visitor for ReferenceCollector {
    fn visit_reference(
        &mut self,
        reference: &Reference,
        _ctx: &VisitorContext<'_>,
    ) -> VisitDecision {
        self.references.push(reference.clone());

        // Group by type if qualified
        if let Some(ref type_name) = reference.type_name {
            self.by_type
                .entry(type_name.to_string())
                .or_default()
                .push(reference.id.to_string());
        }

        VisitDecision::Continue
    }
}

/// Visitor that finds the first node matching a predicate.
///
/// # Example
///
/// ```
/// use hedl_core::visitor::{collectors::FindNode, traverse, TraversalConfig};
/// use hedl_core::{Document, Node};
///
/// let doc = Document::new((2, 0));
/// let mut finder = FindNode::new(|node: &Node| node.id == "target");
/// traverse(&doc, &mut finder, &TraversalConfig::default());
/// if let Some(node) = finder.found() {
///     // Found the target node
/// }
/// ```
pub struct FindNode<F>
where
    F: Fn(&Node) -> bool,
{
    predicate: F,
    found: Option<Node>,
}

impl<F> FindNode<F>
where
    F: Fn(&Node) -> bool,
{
    /// Create a new finder with a predicate.
    pub fn new(predicate: F) -> Self {
        Self {
            predicate,
            found: None,
        }
    }

    /// Get the found node, if any.
    pub fn found(&self) -> Option<&Node> {
        self.found.as_ref()
    }

    /// Consume the finder and return the found node.
    pub fn into_found(self) -> Option<Node> {
        self.found
    }
}

impl<F> Visitor for FindNode<F>
where
    F: Fn(&Node) -> bool,
{
    fn visit_node(&mut self, node: &Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
        if (self.predicate)(node) {
            self.found = Some(node.clone());
            VisitDecision::Stop // Early termination
        } else {
            VisitDecision::Continue
        }
    }
}

/// Visitor that counts nodes by type.
///
/// # Example
///
/// ```
/// use hedl_core::visitor::{collectors::TypeCounter, traverse, TraversalConfig};
/// use hedl_core::Document;
///
/// let doc = Document::new((2, 0));
/// let mut counter = TypeCounter::new();
/// traverse(&doc, &mut counter, &TraversalConfig::default());
/// let counts = counter.counts();
/// ```
#[derive(Debug, Default)]
pub struct TypeCounter {
    counts: HashMap<String, usize>,
}

impl TypeCounter {
    /// Create a new type counter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the counts map.
    pub fn counts(&self) -> &HashMap<String, usize> {
        &self.counts
    }

    /// Get the count for a specific type.
    pub fn count_for(&self, type_name: &str) -> usize {
        self.counts.get(type_name).copied().unwrap_or(0)
    }

    /// Get the total number of nodes counted.
    pub fn total(&self) -> usize {
        self.counts.values().sum()
    }

    /// Consume the counter and return the counts map.
    pub fn into_counts(self) -> HashMap<String, usize> {
        self.counts
    }
}

impl Visitor for TypeCounter {
    fn visit_node(&mut self, node: &Node, _ctx: &VisitorContext<'_>) -> VisitDecision {
        *self.counts.entry(node.type_name.to_string()).or_insert(0) += 1;
        VisitDecision::Continue
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;

    #[test]
    fn test_depth_counter() {
        let mut counter = DepthCounter::new();
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        assert_eq!(counter.max_depth(), 0);

        let node = Node::new("User", "1", vec![]);
        counter.visit_node(&node, &ctx);
        assert_eq!(counter.max_depth(), 0);

        let child_ctx = ctx.child(crate::visitor::PathSegment::Key("a".to_string()));
        counter.visit_node(&node, &child_ctx);
        assert_eq!(counter.max_depth(), 1);
    }

    #[test]
    fn test_node_collector_all() {
        let mut collector = NodeCollector::new_all();
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        let node1 = Node::new("User", "1", vec![]);
        let node2 = Node::new("Post", "2", vec![]);

        collector.visit_node(&node1, &ctx);
        collector.visit_node(&node2, &ctx);

        assert_eq!(collector.count(), 2);
    }

    #[test]
    fn test_node_collector_filtered() {
        let mut collector = NodeCollector::new("User");
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        let node1 = Node::new("User", "1", vec![]);
        let node2 = Node::new("Post", "2", vec![]);

        collector.visit_node(&node1, &ctx);
        collector.visit_node(&node2, &ctx);

        assert_eq!(collector.count(), 1);
        assert_eq!(collector.nodes()[0].type_name, "User");
    }

    #[test]
    fn test_path_collector() {
        let mut collector = PathCollector::new();
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        collector.visit_scalar("a", &Value::Int(1), &ctx);

        let child_ctx = ctx.child(crate::visitor::PathSegment::Key("b".to_string()));
        collector.visit_scalar("c", &Value::Int(2), &child_ctx);

        assert_eq!(collector.paths().len(), 2);
        assert_eq!(collector.paths()[0], "root");
        assert_eq!(collector.paths()[1], "b");
    }

    #[test]
    fn test_reference_collector() {
        let mut collector = ReferenceCollector::new();
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        let ref1 = Reference::qualified("User", "alice");
        let ref2 = Reference::qualified("User", "bob");
        let ref3 = Reference::qualified("Post", "post1");

        collector.visit_reference(&ref1, &ctx);
        collector.visit_reference(&ref2, &ctx);
        collector.visit_reference(&ref3, &ctx);

        assert_eq!(collector.count(), 3);
        assert_eq!(collector.by_type().get("User").unwrap().len(), 2);
        assert_eq!(collector.by_type().get("Post").unwrap().len(), 1);
    }

    #[test]
    fn test_find_node() {
        let mut finder = FindNode::new(|node: &Node| node.id == "target");
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        let node1 = Node::new("User", "alice", vec![]);
        let node2 = Node::new("User", "target", vec![]);

        assert_eq!(finder.visit_node(&node1, &ctx), VisitDecision::Continue);
        assert_eq!(finder.found(), None);

        assert_eq!(finder.visit_node(&node2, &ctx), VisitDecision::Stop);
        assert!(finder.found().is_some());
        assert_eq!(finder.found().unwrap().id, "target");
    }

    #[test]
    fn test_type_counter() {
        let mut counter = TypeCounter::new();
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        let node1 = Node::new("User", "1", vec![]);
        let node2 = Node::new("User", "2", vec![]);
        let node3 = Node::new("Post", "1", vec![]);

        counter.visit_node(&node1, &ctx);
        counter.visit_node(&node2, &ctx);
        counter.visit_node(&node3, &ctx);

        assert_eq!(counter.count_for("User"), 2);
        assert_eq!(counter.count_for("Post"), 1);
        assert_eq!(counter.total(), 3);
    }

    #[test]
    fn test_type_counter_empty() {
        let counter = TypeCounter::new();
        assert_eq!(counter.count_for("User"), 0);
        assert_eq!(counter.total(), 0);
    }
}
