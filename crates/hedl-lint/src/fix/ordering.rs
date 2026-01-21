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

//! Fix ordering for safe sequential application

use crate::fix::{Fix, FixError, FixId};
use std::collections::{HashMap, VecDeque};

/// Orders fixes for safe sequential application
pub struct FixOrderer;

impl FixOrderer {
    /// Order fixes by position (bottom-up to maintain positions)
    /// Applies fixes from end to start so positions remain valid
    #[must_use]
    pub fn order_by_position(mut fixes: Vec<Fix>) -> Vec<Fix> {
        fixes.sort_by(|a, b| b.range.start.cmp(&a.range.start));
        fixes
    }

    /// Order fixes respecting dependencies using topological sort
    pub fn order_by_dependencies(fixes: Vec<Fix>) -> Result<Vec<Fix>, FixError> {
        let graph = Self::build_dependency_graph(&fixes);
        let ordered_ids = Self::topological_sort(&graph)?;

        // Convert back to Fix objects maintaining order
        let fix_map: HashMap<_, _> = fixes.into_iter().map(|f| (f.id, f)).collect();
        let ordered = ordered_ids
            .into_iter()
            .filter_map(|id| fix_map.get(&id).cloned())
            .collect();

        Ok(ordered)
    }

    /// Order fixes by priority (errors first, then warnings, then hints)
    #[must_use]
    pub fn order_by_priority(mut fixes: Vec<Fix>) -> Vec<Fix> {
        fixes.sort_by_key(|f| std::cmp::Reverse(f.severity));
        fixes
    }

    /// Combine strategies: priority first, then dependencies, then position
    /// This ensures high-priority fixes are applied first, respecting dependencies,
    /// and within the same priority level, fixes are ordered by position (bottom-up)
    pub fn order_optimal(fixes: Vec<Fix>) -> Result<Vec<Fix>, FixError> {
        // First handle dependencies
        let by_deps = Self::order_by_dependencies(fixes)?;

        // Then sort by priority (stable sort preserves dependency order)
        // and position (bottom-up) as secondary key
        let mut result = by_deps;
        result.sort_by(|a, b| {
            use std::cmp::Ordering;
            // Primary: priority (Error > Warning > Hint)
            match b.severity.cmp(&a.severity) {
                Ordering::Equal => {
                    // Secondary: position (bottom-up for same priority)
                    b.range.start.cmp(&a.range.start)
                }
                other => other,
            }
        });

        Ok(result)
    }

    /// Build dependency graph
    fn build_dependency_graph(fixes: &[Fix]) -> HashMap<FixId, Vec<FixId>> {
        let mut graph = HashMap::new();

        for fix in fixes {
            graph
                .entry(fix.id)
                .or_insert_with(Vec::new)
                .extend(fix.dependencies.clone());
        }

        // Ensure all fixes are in the graph (even without dependencies)
        for fix in fixes {
            graph.entry(fix.id).or_insert_with(Vec::new);
        }

        graph
    }

    /// Topological sort using Kahn's algorithm
    /// The graph maps `fix_id` -> list of dependencies that must come before it
    fn topological_sort(graph: &HashMap<FixId, Vec<FixId>>) -> Result<Vec<FixId>, FixError> {
        // The graph has fix -> [deps], where deps must be processed before fix
        // We need to reverse this to get the correct in-degree
        // In-degree = number of dependencies for each fix
        let mut in_degree: HashMap<FixId, usize> = HashMap::new();
        for &node in graph.keys() {
            in_degree.entry(node).or_insert(0);
        }

        // Count in-degrees: each fix's in-degree is the number of its dependencies
        for (&node, deps) in graph {
            *in_degree.entry(node).or_insert(0) = deps.len();
        }

        // Find nodes with no dependencies (in-degree 0)
        let mut queue: VecDeque<FixId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node);

            // Find all nodes that depend on this node and reduce their in-degree
            for (&other_node, deps) in graph {
                if deps.contains(&node) {
                    if let Some(degree) = in_degree.get_mut(&other_node) {
                        if *degree > 0 {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(other_node);
                            }
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() != graph.len() {
            return Err(FixError::CircularDependency(
                "Circular dependency detected in fixes".to_string(),
            ));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Severity;
    use crate::fix::range::{SourcePosition, SourceRange};

    #[test]
    fn test_order_by_position() {
        let fix1 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(5, 0), SourcePosition::new(5, 5)),
            "text",
            "desc",
        );
        let fix2 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5)),
            "text",
            "desc",
        );
        let fix3 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(8, 0), SourcePosition::new(8, 5)),
            "text",
            "desc",
        );

        let ordered = FixOrderer::order_by_position(vec![fix1.clone(), fix2.clone(), fix3.clone()]);

        // Should be ordered from end to start (8, 5, 2)
        assert_eq!(ordered[0].id, fix3.id);
        assert_eq!(ordered[1].id, fix1.id);
        assert_eq!(ordered[2].id, fix2.id);
    }

    #[test]
    fn test_order_by_dependencies_no_deps() {
        let fix1 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5)),
            "text",
            "desc",
        );
        let fix2 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5)),
            "text",
            "desc",
        );

        let ordered = FixOrderer::order_by_dependencies(vec![fix1.clone(), fix2.clone()]).unwrap();
        assert_eq!(ordered.len(), 2);
    }

    #[test]
    fn test_order_by_dependencies_with_deps() {
        let fix1 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5)),
            "text",
            "desc1",
        );
        let fix2 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5)),
            "text",
            "desc2",
        )
        .with_dependency(fix1.id);

        let ordered = FixOrderer::order_by_dependencies(vec![fix2.clone(), fix1.clone()]).unwrap();

        // fix1 should come before fix2
        assert_eq!(ordered[0].id, fix1.id);
        assert_eq!(ordered[1].id, fix2.id);
    }

    #[test]
    fn test_order_by_dependencies_circular() {
        let mut fix1 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5)),
            "text",
            "desc1",
        );
        let mut fix2 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5)),
            "text",
            "desc2",
        );

        // Create circular dependency
        fix1.dependencies.push(fix2.id);
        fix2.dependencies.push(fix1.id);

        let result = FixOrderer::order_by_dependencies(vec![fix1, fix2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_order_by_priority() {
        let fix1 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5)),
            "text",
            "desc",
        )
        .with_severity(Severity::Hint);

        let fix2 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5)),
            "text",
            "desc",
        )
        .with_severity(Severity::Error);

        let fix3 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(3, 0), SourcePosition::new(3, 5)),
            "text",
            "desc",
        )
        .with_severity(Severity::Warning);

        let ordered = FixOrderer::order_by_priority(vec![fix1.clone(), fix2.clone(), fix3.clone()]);

        // Should be ordered: Error, Warning, Hint
        assert_eq!(ordered[0].id, fix2.id);
        assert_eq!(ordered[1].id, fix3.id);
        assert_eq!(ordered[2].id, fix1.id);
    }

    #[test]
    fn test_order_optimal() {
        let fix1 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(5, 0), SourcePosition::new(5, 5)),
            "text",
            "desc",
        )
        .with_severity(Severity::Hint);

        let fix2 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5)),
            "text",
            "desc",
        )
        .with_severity(Severity::Error);

        let ordered = FixOrderer::order_optimal(vec![fix1.clone(), fix2.clone()]).unwrap();

        // Higher priority (Error) should come first, then ordered by position
        assert_eq!(ordered[0].id, fix2.id);
        assert_eq!(ordered[1].id, fix1.id);
    }

    #[test]
    fn test_topological_sort_complex() {
        let fix1 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5)),
            "text",
            "a",
        );
        let fix2 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5)),
            "text",
            "b",
        )
        .with_dependency(fix1.id);
        let fix3 = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(3, 0), SourcePosition::new(3, 5)),
            "text",
            "c",
        )
        .with_dependency(fix2.id);

        let ordered =
            FixOrderer::order_by_dependencies(vec![fix3.clone(), fix2.clone(), fix1.clone()])
                .unwrap();

        // Should be ordered: fix1 -> fix2 -> fix3
        assert_eq!(ordered[0].id, fix1.id);
        assert_eq!(ordered[1].id, fix2.id);
        assert_eq!(ordered[2].id, fix3.id);
    }
}
