// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Traversal configuration options.

/// Traversal order strategy.
///
/// Controls whether nodes are visited before or after their children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TraversalOrder {
    /// Visit parent before children (default).
    ///
    /// This is the standard top-down traversal order, useful for
    /// emitting output or building structures from top to bottom.
    #[default]
    PreOrder,

    /// Visit children before parent.
    ///
    /// Useful for bottom-up analysis or aggregation, where parent
    /// processing depends on results from children.
    PostOrder,
}

/// Traversal mode strategy.
///
/// Controls the order in which siblings and descendants are visited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TraversalMode {
    /// Depth-first traversal (default).
    ///
    /// Explores as far as possible along each branch before backtracking.
    /// More memory-efficient for deep trees.
    #[default]
    DepthFirst,

    /// Breadth-first traversal.
    ///
    /// Visits all nodes at the same depth level before moving deeper.
    /// Useful for level-order processing.
    BreadthFirst,
}

/// Configuration for document traversal.
///
/// Controls how the document tree is traversed and what elements are visited.
///
/// # Examples
///
/// ```
/// use hedl_core::visitor::{TraversalConfig, TraversalOrder, TraversalMode};
///
/// // Default configuration (pre-order, depth-first, no limits)
/// let config = TraversalConfig::default();
///
/// // Custom configuration
/// let config = TraversalConfig {
///     order: TraversalOrder::PostOrder,
///     mode: TraversalMode::DepthFirst,
///     max_depth: Some(10),
///     follow_references: false,
///     visit_expressions: true,
///     visit_tensors: true,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct TraversalConfig {
    /// Traversal order (pre-order or post-order).
    pub order: TraversalOrder,

    /// Traversal mode (depth-first or breadth-first).
    pub mode: TraversalMode,

    /// Maximum depth to traverse (None = unlimited).
    ///
    /// When set, traversal stops descending once this depth is reached.
    /// Useful for limiting traversal of very deep trees.
    pub max_depth: Option<usize>,

    /// Whether to follow and validate references.
    ///
    /// When true, reference values trigger `visit_reference` calls.
    pub follow_references: bool,

    /// Whether to visit expression values.
    ///
    /// When true, expression values trigger `visit_expression` calls.
    pub visit_expressions: bool,

    /// Whether to visit tensor values.
    ///
    /// When true, tensor values trigger `visit_tensor` calls.
    pub visit_tensors: bool,
}

impl Default for TraversalConfig {
    fn default() -> Self {
        Self {
            order: TraversalOrder::PreOrder,
            mode: TraversalMode::DepthFirst,
            max_depth: None,
            follow_references: true,
            visit_expressions: true,
            visit_tensors: true,
        }
    }
}

impl TraversalConfig {
    /// Create a new configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the traversal order.
    pub fn with_order(mut self, order: TraversalOrder) -> Self {
        self.order = order;
        self
    }

    /// Set the traversal mode.
    pub fn with_mode(mut self, mode: TraversalMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the maximum depth.
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }

    /// Enable or disable reference following.
    pub fn with_follow_references(mut self, follow: bool) -> Self {
        self.follow_references = follow;
        self
    }

    /// Enable or disable expression visiting.
    pub fn with_visit_expressions(mut self, visit: bool) -> Self {
        self.visit_expressions = visit;
        self
    }

    /// Enable or disable tensor visiting.
    pub fn with_visit_tensors(mut self, visit: bool) -> Self {
        self.visit_tensors = visit;
        self
    }

    /// Check if depth limit has been reached.
    pub fn is_depth_limit_reached(&self, current_depth: usize) -> bool {
        if let Some(max) = self.max_depth {
            current_depth >= max
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_traversal_config() {
        let config = TraversalConfig::default();
        assert_eq!(config.order, TraversalOrder::PreOrder);
        assert_eq!(config.mode, TraversalMode::DepthFirst);
        assert!(config.max_depth.is_none());
        assert!(config.follow_references);
        assert!(config.visit_expressions);
        assert!(config.visit_tensors);
    }

    #[test]
    fn test_builder_pattern() {
        let config = TraversalConfig::new()
            .with_order(TraversalOrder::PostOrder)
            .with_mode(TraversalMode::BreadthFirst)
            .with_max_depth(5)
            .with_follow_references(false)
            .with_visit_expressions(false)
            .with_visit_tensors(false);

        assert_eq!(config.order, TraversalOrder::PostOrder);
        assert_eq!(config.mode, TraversalMode::BreadthFirst);
        assert_eq!(config.max_depth, Some(5));
        assert!(!config.follow_references);
        assert!(!config.visit_expressions);
        assert!(!config.visit_tensors);
    }

    #[test]
    fn test_depth_limit_check() {
        let config = TraversalConfig::new().with_max_depth(3);
        assert!(!config.is_depth_limit_reached(0));
        assert!(!config.is_depth_limit_reached(2));
        assert!(config.is_depth_limit_reached(3));
        assert!(config.is_depth_limit_reached(10));
    }

    #[test]
    fn test_no_depth_limit() {
        let config = TraversalConfig::new();
        assert!(!config.is_depth_limit_reached(0));
        assert!(!config.is_depth_limit_reached(1000));
    }

    #[test]
    fn test_traversal_order_default() {
        assert_eq!(TraversalOrder::default(), TraversalOrder::PreOrder);
    }

    #[test]
    fn test_traversal_mode_default() {
        assert_eq!(TraversalMode::default(), TraversalMode::DepthFirst);
    }

    #[test]
    fn test_traversal_order_equality() {
        assert_eq!(TraversalOrder::PreOrder, TraversalOrder::PreOrder);
        assert_ne!(TraversalOrder::PreOrder, TraversalOrder::PostOrder);
    }

    #[test]
    fn test_traversal_mode_equality() {
        assert_eq!(TraversalMode::DepthFirst, TraversalMode::DepthFirst);
        assert_ne!(TraversalMode::DepthFirst, TraversalMode::BreadthFirst);
    }

    #[test]
    fn test_clone() {
        let config = TraversalConfig::new().with_max_depth(5);
        let cloned = config.clone();
        assert_eq!(cloned.max_depth, Some(5));
    }

    #[test]
    fn test_debug() {
        let config = TraversalConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("TraversalConfig"));
    }
}
