// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Visitor context for tracking traversal state.

use crate::Document;
use std::collections::HashMap;

/// Path segment in the document tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSegment {
    /// Root-level key
    Key(String),
    /// Object nested key
    NestedKey(String),
    /// List row index
    Index(usize),
    /// Node ID
    NodeId(String),
}

impl PathSegment {
    /// Convert to string representation.
    pub fn as_str(&self) -> String {
        match self {
            PathSegment::Key(s) | PathSegment::NestedKey(s) | PathSegment::NodeId(s) => s.clone(),
            PathSegment::Index(i) => format!("[{}]", i),
        }
    }
}

/// Context provided to visitors during traversal.
///
/// This context tracks the current position in the document tree,
/// provides access to the document metadata, and supports custom
/// data storage for visitor state.
///
/// # Examples
///
/// ```
/// use hedl_core::visitor::VisitorContext;
/// use hedl_core::Document;
///
/// let doc = Document::new((2, 0));
/// let ctx = VisitorContext::new(&doc);
///
/// assert_eq!(ctx.depth, 0);
/// assert!(ctx.path.is_empty());
/// assert_eq!(ctx.path_string(), "root");
/// ```
#[derive(Debug)]
pub struct VisitorContext<'a> {
    /// Current nesting depth (0 = root level).
    pub depth: usize,

    /// Path from root to current element.
    pub path: Vec<PathSegment>,

    /// Reference to the document being traversed.
    pub document: &'a Document,

    /// Schema for the current list (if within a list context).
    pub current_schema: Option<&'a [String]>,

    /// Custom metadata storage for visitor-specific data.
    metadata: HashMap<String, String>,

    /// Statistics tracking.
    stats: TraversalStats,
}

/// Traversal statistics collected during document traversal.
#[derive(Debug, Clone, Default)]
pub struct TraversalStats {
    /// Number of nodes visited.
    pub nodes_visited: usize,
    /// Number of scalars visited.
    pub scalars_visited: usize,
    /// Number of lists visited.
    pub lists_visited: usize,
    /// Number of objects visited.
    pub objects_visited: usize,
    /// Maximum depth reached.
    pub max_depth_reached: usize,
}

impl<'a> VisitorContext<'a> {
    /// Create a new context for the root level.
    ///
    /// # Arguments
    ///
    /// - `document`: Reference to the document being traversed
    pub fn new(document: &'a Document) -> Self {
        Self {
            depth: 0,
            path: Vec::new(),
            document,
            current_schema: None,
            metadata: HashMap::new(),
            stats: TraversalStats::default(),
        }
    }

    /// Create a child context with incremented depth.
    ///
    /// # Arguments
    ///
    /// - `segment`: Path segment for the child element
    pub fn child(&self, segment: PathSegment) -> Self {
        let mut path = self.path.clone();
        path.push(segment);
        Self {
            depth: self.depth + 1,
            path,
            document: self.document,
            current_schema: self.current_schema,
            metadata: self.metadata.clone(),
            stats: self.stats.clone(),
        }
    }

    /// Create a child context with a different schema.
    ///
    /// Used when entering a list with its own schema.
    pub fn with_schema(&self, schema: &'a [String]) -> Self {
        Self {
            depth: self.depth,
            path: self.path.clone(),
            document: self.document,
            current_schema: Some(schema),
            metadata: self.metadata.clone(),
            stats: self.stats.clone(),
        }
    }

    /// Get the current path as a string (for error messages).
    ///
    /// # Example
    ///
    /// ```
    /// use hedl_core::visitor::{VisitorContext, PathSegment};
    /// use hedl_core::Document;
    ///
    /// let doc = Document::new((2, 0));
    /// let ctx = VisitorContext::new(&doc);
    /// assert_eq!(ctx.path_string(), "root");
    ///
    /// let child_ctx = ctx.child(PathSegment::Key("users".to_string()));
    /// assert_eq!(child_ctx.path_string(), "users");
    /// ```
    pub fn path_string(&self) -> String {
        if self.path.is_empty() {
            "root".to_string()
        } else {
            self.path
                .iter()
                .map(|seg| seg.as_str())
                .collect::<Vec<_>>()
                .join(".")
        }
    }

    /// Store custom metadata.
    ///
    /// This allows visitors to attach arbitrary string metadata
    /// to specific paths during traversal.
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Retrieve custom metadata.
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Get traversal statistics.
    pub fn stats(&self) -> &TraversalStats {
        &self.stats
    }

    /// Record a node visit in statistics.
    pub(crate) fn record_node_visit(&mut self) {
        self.stats.nodes_visited += 1;
        self.stats.max_depth_reached = self.stats.max_depth_reached.max(self.depth);
    }

    /// Record a scalar visit in statistics.
    pub(crate) fn record_scalar_visit(&mut self) {
        self.stats.scalars_visited += 1;
        self.stats.max_depth_reached = self.stats.max_depth_reached.max(self.depth);
    }

    /// Record a list visit in statistics.
    pub(crate) fn record_list_visit(&mut self) {
        self.stats.lists_visited += 1;
    }

    /// Record an object visit in statistics.
    pub(crate) fn record_object_visit(&mut self) {
        self.stats.objects_visited += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_context() {
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);

        assert_eq!(ctx.depth, 0);
        assert!(ctx.path.is_empty());
        assert_eq!(ctx.path_string(), "root");
        assert!(ctx.current_schema.is_none());
    }

    #[test]
    fn test_child_context_increments_depth() {
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);
        let child = ctx.child(PathSegment::Key("users".to_string()));

        assert_eq!(child.depth, 1);
        assert_eq!(child.path.len(), 1);
    }

    #[test]
    fn test_path_string_with_nested_keys() {
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);
        let ctx = ctx.child(PathSegment::Key("a".to_string()));
        let ctx = ctx.child(PathSegment::NestedKey("b".to_string()));
        let ctx = ctx.child(PathSegment::Key("c".to_string()));

        assert_eq!(ctx.path_string(), "a.b.c");
    }

    #[test]
    fn test_path_string_with_index() {
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);
        let ctx = ctx.child(PathSegment::Key("users".to_string()));
        let ctx = ctx.child(PathSegment::Index(0));

        assert_eq!(ctx.path_string(), "users.[0]");
    }

    #[test]
    fn test_with_schema() {
        let doc = Document::new((2, 0));
        let ctx = VisitorContext::new(&doc);
        let schema = vec!["id".to_string(), "name".to_string()];
        let ctx_with_schema = ctx.with_schema(&schema);

        assert!(ctx_with_schema.current_schema.is_some());
        assert_eq!(ctx_with_schema.current_schema.unwrap().len(), 2);
        assert_eq!(ctx_with_schema.depth, ctx.depth);
    }

    #[test]
    fn test_metadata_storage() {
        let doc = Document::new((2, 0));
        let mut ctx = VisitorContext::new(&doc);

        ctx.set_metadata("key", "value");
        assert_eq!(ctx.get_metadata("key"), Some("value"));
        assert_eq!(ctx.get_metadata("missing"), None);
    }

    #[test]
    fn test_metadata_persists_in_child() {
        let doc = Document::new((2, 0));
        let mut ctx = VisitorContext::new(&doc);
        ctx.set_metadata("key", "value");

        let child = ctx.child(PathSegment::Key("child".to_string()));
        assert_eq!(child.get_metadata("key"), Some("value"));
    }

    #[test]
    fn test_stats_tracking() {
        let doc = Document::new((2, 0));
        let mut ctx = VisitorContext::new(&doc);

        ctx.record_node_visit();
        ctx.record_scalar_visit();
        ctx.record_list_visit();
        ctx.record_object_visit();

        let stats = ctx.stats();
        assert_eq!(stats.nodes_visited, 1);
        assert_eq!(stats.scalars_visited, 1);
        assert_eq!(stats.lists_visited, 1);
        assert_eq!(stats.objects_visited, 1);
    }

    #[test]
    fn test_stats_max_depth() {
        let doc = Document::new((2, 0));
        let mut ctx = VisitorContext::new(&doc);

        ctx.record_node_visit();
        assert_eq!(ctx.stats().max_depth_reached, 0);

        let mut child = ctx.child(PathSegment::Key("a".to_string()));
        child.record_node_visit();
        assert_eq!(child.stats().max_depth_reached, 1);
    }

    #[test]
    fn test_path_segment_as_str() {
        assert_eq!(PathSegment::Key("test".to_string()).as_str(), "test");
        assert_eq!(
            PathSegment::NestedKey("nested".to_string()).as_str(),
            "nested"
        );
        assert_eq!(PathSegment::Index(5).as_str(), "[5]");
        assert_eq!(PathSegment::NodeId("id123".to_string()).as_str(), "id123");
    }

    #[test]
    fn test_stats_default() {
        let stats = TraversalStats::default();
        assert_eq!(stats.nodes_visited, 0);
        assert_eq!(stats.scalars_visited, 0);
        assert_eq!(stats.lists_visited, 0);
        assert_eq!(stats.objects_visited, 0);
        assert_eq!(stats.max_depth_reached, 0);
    }

    #[test]
    fn test_stats_clone() {
        let stats = TraversalStats {
            nodes_visited: 5,
            ..Default::default()
        };
        let cloned = stats.clone();
        assert_eq!(cloned.nodes_visited, 5);
    }
}
