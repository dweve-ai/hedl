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

//! Preview functionality for fixes

use crate::fix::conflict::FixConflict;
use crate::fix::statistics::FixStatistics;

/// Preview of fixes before application
#[derive(Debug, Clone)]
pub struct FixPreview {
    /// Unified diff of changes
    pub diff: String,
    /// Number of fixes to be applied
    pub fixes_count: usize,
    /// Conflicts detected
    pub conflicts: Vec<FixConflict>,
    /// Estimated result after applying fixes
    pub estimated_result: String,
    /// Statistics about the fixes
    pub statistics: FixStatistics,
}

impl FixPreview {
    /// Create a new preview
    #[must_use]
    pub fn new(
        diff: String,
        fixes_count: usize,
        conflicts: Vec<FixConflict>,
        estimated_result: String,
        statistics: FixStatistics,
    ) -> Self {
        Self {
            diff,
            fixes_count,
            conflicts,
            estimated_result,
            statistics,
        }
    }

    /// Check if there are any conflicts
    #[must_use]
    pub fn has_conflicts(&self) -> bool {
        !self.conflicts.is_empty()
    }

    /// Get the number of conflicts
    #[must_use]
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_new() {
        let preview = FixPreview::new(
            "diff content".to_string(),
            5,
            vec![],
            "result".to_string(),
            FixStatistics::default(),
        );

        assert_eq!(preview.diff, "diff content");
        assert_eq!(preview.fixes_count, 5);
        assert!(preview.conflicts.is_empty());
        assert_eq!(preview.estimated_result, "result");
    }

    #[test]
    fn test_has_conflicts() {
        let mut preview = FixPreview::new(
            String::new(),
            0,
            vec![],
            String::new(),
            FixStatistics::default(),
        );

        assert!(!preview.has_conflicts());

        preview.conflicts.push(crate::fix::conflict::FixConflict {
            fix_a: uuid::Uuid::new_v4(),
            fix_b: uuid::Uuid::new_v4(),
            conflict_type: crate::fix::conflict::ConflictType::Overlapping,
            resolution: None,
        });

        assert!(preview.has_conflicts());
    }

    #[test]
    fn test_conflict_count() {
        let preview = FixPreview::new(
            String::new(),
            0,
            vec![],
            String::new(),
            FixStatistics::default(),
        );

        assert_eq!(preview.conflict_count(), 0);
    }
}
