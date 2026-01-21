// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Control flow decisions for visitor pattern traversal.

/// Control flow decision for document traversal.
///
/// Visitors return this from methods to control whether traversal
/// continues, skips subtrees, or terminates early. This enables
/// efficient traversal with fine-grained control over what gets visited.
///
/// # Examples
///
/// ```
/// use hedl_core::visitor::VisitDecision;
///
/// // Continue normal traversal
/// let decision = VisitDecision::Continue;
/// assert!(decision.should_continue());
/// assert!(decision.should_visit_children());
///
/// // Skip children but continue with siblings
/// let decision = VisitDecision::SkipChildren;
/// assert!(decision.should_continue());
/// assert!(!decision.should_visit_children());
///
/// // Stop all traversal
/// let decision = VisitDecision::Stop;
/// assert!(!decision.should_continue());
/// assert!(decision.should_stop());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VisitDecision {
    /// Continue normal traversal (visit children and siblings).
    #[default]
    Continue,
    /// Skip children of current node but continue with siblings.
    SkipChildren,
    /// Stop traversal entirely (early termination).
    Stop,
}

impl VisitDecision {
    /// Check if traversal should continue to next nodes.
    ///
    /// Returns `true` for `Continue` and `SkipChildren`, `false` for `Stop`.
    #[inline]
    pub fn should_continue(&self) -> bool {
        !matches!(self, Self::Stop)
    }

    /// Check if children of the current node should be visited.
    ///
    /// Returns `true` only for `Continue`.
    #[inline]
    pub fn should_visit_children(&self) -> bool {
        matches!(self, Self::Continue)
    }

    /// Check if traversal should stop immediately.
    ///
    /// Returns `true` only for `Stop`.
    #[inline]
    pub fn should_stop(&self) -> bool {
        matches!(self, Self::Stop)
    }

    /// Combine two decisions, taking the more restrictive one.
    ///
    /// Precedence: Stop > SkipChildren > Continue
    ///
    /// # Examples
    ///
    /// ```
    /// use hedl_core::visitor::VisitDecision;
    ///
    /// assert_eq!(
    ///     VisitDecision::Continue.combine(VisitDecision::Stop),
    ///     VisitDecision::Stop
    /// );
    /// assert_eq!(
    ///     VisitDecision::SkipChildren.combine(VisitDecision::Continue),
    ///     VisitDecision::SkipChildren
    /// );
    /// ```
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Stop, _) | (_, Self::Stop) => Self::Stop,
            (Self::SkipChildren, _) | (_, Self::SkipChildren) => Self::SkipChildren,
            _ => Self::Continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_continue() {
        assert!(VisitDecision::Continue.should_continue());
        assert!(VisitDecision::SkipChildren.should_continue());
        assert!(!VisitDecision::Stop.should_continue());
    }

    #[test]
    fn test_should_visit_children() {
        assert!(VisitDecision::Continue.should_visit_children());
        assert!(!VisitDecision::SkipChildren.should_visit_children());
        assert!(!VisitDecision::Stop.should_visit_children());
    }

    #[test]
    fn test_should_stop() {
        assert!(!VisitDecision::Continue.should_stop());
        assert!(!VisitDecision::SkipChildren.should_stop());
        assert!(VisitDecision::Stop.should_stop());
    }

    #[test]
    fn test_combine_stop_wins() {
        assert_eq!(
            VisitDecision::Stop.combine(VisitDecision::Continue),
            VisitDecision::Stop
        );
        assert_eq!(
            VisitDecision::Continue.combine(VisitDecision::Stop),
            VisitDecision::Stop
        );
        assert_eq!(
            VisitDecision::Stop.combine(VisitDecision::SkipChildren),
            VisitDecision::Stop
        );
    }

    #[test]
    fn test_combine_skip_children_wins() {
        assert_eq!(
            VisitDecision::SkipChildren.combine(VisitDecision::Continue),
            VisitDecision::SkipChildren
        );
        assert_eq!(
            VisitDecision::Continue.combine(VisitDecision::SkipChildren),
            VisitDecision::SkipChildren
        );
    }

    #[test]
    fn test_combine_continue() {
        assert_eq!(
            VisitDecision::Continue.combine(VisitDecision::Continue),
            VisitDecision::Continue
        );
    }

    #[test]
    fn test_default() {
        assert_eq!(VisitDecision::default(), VisitDecision::Continue);
    }

    #[test]
    fn test_equality() {
        assert_eq!(VisitDecision::Continue, VisitDecision::Continue);
        assert_ne!(VisitDecision::Continue, VisitDecision::Stop);
    }

    #[test]
    fn test_clone_copy() {
        let decision = VisitDecision::SkipChildren;
        let cloned = decision;
        assert_eq!(decision, cloned);
    }

    #[test]
    fn test_debug() {
        let debug = format!("{:?}", VisitDecision::Continue);
        assert!(debug.contains("Continue"));
    }

    #[test]
    fn test_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(VisitDecision::Continue);
        set.insert(VisitDecision::SkipChildren);
        set.insert(VisitDecision::Stop);
        assert_eq!(set.len(), 3);
    }
}
