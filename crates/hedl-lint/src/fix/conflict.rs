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

//! Conflict detection and resolution for fixes

use crate::fix::config::ConflictStrategy;
use crate::fix::{Fix, FixId};

/// Type of conflict between two fixes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictType {
    /// Ranges overlap
    Overlapping,
    /// One fix depends on the other
    Dependent,
    /// Fixes have opposing effects
    Contradictory,
}

/// Resolution for a conflict
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Prefer the first fix
    PreferFirst,
    /// Prefer the second fix
    PreferSecond,
    /// Merge both fixes
    Merge(String),
    /// Skip both fixes
    Skip,
}

/// A conflict between two fixes
#[derive(Debug, Clone)]
pub struct FixConflict {
    /// ID of first fix
    pub fix_a: FixId,
    /// ID of second fix
    pub fix_b: FixId,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Proposed resolution
    pub resolution: Option<ConflictResolution>,
}

/// Detects conflicts between fixes
pub struct ConflictDetector {
    strategy: ConflictStrategy,
}

impl ConflictDetector {
    /// Create a new conflict detector with the given strategy
    #[must_use]
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self { strategy }
    }

    /// Find all conflicts in a set of fixes
    #[must_use]
    pub fn detect_conflicts(&self, fixes: &[Fix]) -> Vec<FixConflict> {
        let mut conflicts = Vec::new();

        for (i, fix_a) in fixes.iter().enumerate() {
            for fix_b in &fixes[i + 1..] {
                if let Some(conflict) = self.check_conflict(fix_a, fix_b) {
                    conflicts.push(conflict);
                }
            }
        }

        conflicts
    }

    /// Check if two fixes conflict
    fn check_conflict(&self, fix_a: &Fix, fix_b: &Fix) -> Option<FixConflict> {
        // Check for contradictory fixes first (same range, different replacement)
        // This is more specific than overlapping, so check it first
        if fix_a.range == fix_b.range && fix_a.replacement != fix_b.replacement {
            return Some(FixConflict {
                fix_a: fix_a.id,
                fix_b: fix_b.id,
                conflict_type: ConflictType::Contradictory,
                resolution: None,
            });
        }

        // Check for dependencies
        if fix_a.dependencies.contains(&fix_b.id) || fix_b.dependencies.contains(&fix_a.id) {
            return Some(FixConflict {
                fix_a: fix_a.id,
                fix_b: fix_b.id,
                conflict_type: ConflictType::Dependent,
                resolution: None,
            });
        }

        // Check for overlapping ranges
        if fix_a.range.overlaps(&fix_b.range) {
            return Some(FixConflict {
                fix_a: fix_a.id,
                fix_b: fix_b.id,
                conflict_type: ConflictType::Overlapping,
                resolution: None,
            });
        }

        None
    }

    /// Resolve conflicts based on strategy
    #[must_use]
    pub fn resolve_conflicts(
        &self,
        mut conflicts: Vec<FixConflict>,
        fixes: &[Fix],
    ) -> Vec<FixConflict> {
        for conflict in &mut conflicts {
            conflict.resolution = self.resolve_conflict(conflict, fixes);
        }
        conflicts
    }

    /// Resolve a single conflict
    fn resolve_conflict(
        &self,
        conflict: &FixConflict,
        fixes: &[Fix],
    ) -> Option<ConflictResolution> {
        match self.strategy {
            ConflictStrategy::Skip => Some(ConflictResolution::Skip),
            ConflictStrategy::PreferFirst => Some(ConflictResolution::PreferFirst),
            ConflictStrategy::PreferPriority => {
                let fix_a = fixes.iter().find(|f| f.id == conflict.fix_a)?;
                let fix_b = fixes.iter().find(|f| f.id == conflict.fix_b)?;

                if fix_a.severity > fix_b.severity {
                    Some(ConflictResolution::PreferFirst)
                } else if fix_b.severity > fix_a.severity {
                    Some(ConflictResolution::PreferSecond)
                } else {
                    Some(ConflictResolution::PreferFirst)
                }
            }
            ConflictStrategy::Fail => None,
        }
    }

    /// Filter fixes based on conflict resolution
    #[must_use]
    pub fn apply_resolutions(&self, fixes: Vec<Fix>, conflicts: &[FixConflict]) -> Vec<Fix> {
        let mut skipped_ids = std::collections::HashSet::new();

        for conflict in conflicts {
            match &conflict.resolution {
                Some(ConflictResolution::Skip) => {
                    skipped_ids.insert(conflict.fix_a);
                    skipped_ids.insert(conflict.fix_b);
                }
                Some(ConflictResolution::PreferFirst) => {
                    skipped_ids.insert(conflict.fix_b);
                }
                Some(ConflictResolution::PreferSecond) => {
                    skipped_ids.insert(conflict.fix_a);
                }
                Some(ConflictResolution::Merge(_)) => {
                    // Merging not yet implemented
                    skipped_ids.insert(conflict.fix_a);
                    skipped_ids.insert(conflict.fix_b);
                }
                None => {
                    // No resolution - skip both
                    skipped_ids.insert(conflict.fix_a);
                    skipped_ids.insert(conflict.fix_b);
                }
            }
        }

        fixes
            .into_iter()
            .filter(|f| !skipped_ids.contains(&f.id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Severity;
    use crate::fix::range::{SourcePosition, SourceRange};

    #[test]
    fn test_detect_overlapping_conflicts() {
        let detector = ConflictDetector::new(ConflictStrategy::Skip);

        let fix_a = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10)),
            "text",
            "desc",
        );
        let fix_b = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15)),
            "text",
            "desc",
        );

        let conflicts = detector.detect_conflicts(&[fix_a, fix_b]);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::Overlapping);
    }

    #[test]
    fn test_detect_no_conflicts() {
        let detector = ConflictDetector::new(ConflictStrategy::Skip);

        let fix_a = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5)),
            "text",
            "desc",
        );
        let fix_b = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 10), SourcePosition::new(1, 15)),
            "text",
            "desc",
        );

        let conflicts = detector.detect_conflicts(&[fix_a, fix_b]);
        assert_eq!(conflicts.len(), 0);
    }

    #[test]
    fn test_detect_dependent_conflicts() {
        let detector = ConflictDetector::new(ConflictStrategy::Skip);

        let fix_a = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5)),
            "text",
            "desc",
        );
        let mut fix_b = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5)),
            "text",
            "desc",
        );
        fix_b.dependencies.push(fix_a.id);

        let conflicts = detector.detect_conflicts(&[fix_a, fix_b]);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::Dependent);
    }

    #[test]
    fn test_detect_contradictory_conflicts() {
        let detector = ConflictDetector::new(ConflictStrategy::Skip);

        let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
        let fix_a = Fix::new("rule", range.clone(), "replacement1", "desc");
        let fix_b = Fix::new("rule", range, "replacement2", "desc");

        let conflicts = detector.detect_conflicts(&[fix_a, fix_b]);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].conflict_type, ConflictType::Contradictory);
    }

    #[test]
    fn test_resolve_with_skip_strategy() {
        let detector = ConflictDetector::new(ConflictStrategy::Skip);

        let fix_a = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10)),
            "text",
            "desc",
        );
        let fix_b = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15)),
            "text",
            "desc",
        );

        let fixes = vec![fix_a, fix_b];
        let conflicts = detector.detect_conflicts(&fixes);
        let resolved = detector.resolve_conflicts(conflicts, &fixes);

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].resolution, Some(ConflictResolution::Skip));
    }

    #[test]
    fn test_resolve_with_priority_strategy() {
        let detector = ConflictDetector::new(ConflictStrategy::PreferPriority);

        let fix_a = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10)),
            "text",
            "desc",
        )
        .with_severity(Severity::Error);

        let fix_b = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15)),
            "text",
            "desc",
        )
        .with_severity(Severity::Hint);

        let fixes = vec![fix_a, fix_b];
        let conflicts = detector.detect_conflicts(&fixes);
        let resolved = detector.resolve_conflicts(conflicts, &fixes);

        assert_eq!(resolved.len(), 1);
        assert_eq!(
            resolved[0].resolution,
            Some(ConflictResolution::PreferFirst)
        );
    }

    #[test]
    fn test_resolve_with_prefer_first_strategy() {
        let detector = ConflictDetector::new(ConflictStrategy::PreferFirst);

        let fix_a = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10)),
            "text",
            "desc",
        );
        let fix_b = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15)),
            "text",
            "desc",
        );

        let fixes = vec![fix_a, fix_b];
        let conflicts = detector.detect_conflicts(&fixes);
        let resolved = detector.resolve_conflicts(conflicts, &fixes);

        assert_eq!(resolved.len(), 1);
        assert_eq!(
            resolved[0].resolution,
            Some(ConflictResolution::PreferFirst)
        );
    }

    #[test]
    fn test_apply_resolutions_skip() {
        let detector = ConflictDetector::new(ConflictStrategy::Skip);

        let fix_a = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10)),
            "text",
            "desc",
        );
        let fix_b = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15)),
            "text",
            "desc",
        );
        let fix_c = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5)),
            "text",
            "desc",
        );

        let fixes = vec![fix_a.clone(), fix_b.clone(), fix_c.clone()];
        let conflicts = detector.detect_conflicts(&fixes);
        let resolved = detector.resolve_conflicts(conflicts, &fixes);
        let filtered = detector.apply_resolutions(fixes, &resolved);

        // fix_a and fix_b conflict and are skipped, fix_c remains
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, fix_c.id);
    }

    #[test]
    fn test_apply_resolutions_prefer_first() {
        let detector = ConflictDetector::new(ConflictStrategy::PreferFirst);

        let fix_a = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10)),
            "text",
            "desc",
        );
        let fix_b = Fix::new(
            "rule",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15)),
            "text",
            "desc",
        );

        let fixes = vec![fix_a.clone(), fix_b.clone()];
        let conflicts = detector.detect_conflicts(&fixes);
        let resolved = detector.resolve_conflicts(conflicts, &fixes);
        let filtered = detector.apply_resolutions(fixes, &resolved);

        // fix_a is preferred, fix_b is skipped
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, fix_a.id);
    }
}
