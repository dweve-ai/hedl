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

//! Statistics tracking for fix operations

use crate::diagnostic::Severity;
use crate::fix::Fix;

/// Statistics for fix operations
#[derive(Debug, Clone, Default)]
pub struct FixStatistics {
    /// Total number of fixes available
    pub total_fixes: usize,
    /// Number of safe fixes
    pub safe_fixes: usize,
    /// Number of unsafe fixes
    pub unsafe_fixes: usize,
    /// Number of fixes skipped due to conflicts
    pub skipped_conflicts: usize,
    /// Number of error-level fixes
    pub errors_fixed: usize,
    /// Number of warning-level fixes
    pub warnings_fixed: usize,
    /// Number of hint-level fixes
    pub hints_fixed: usize,
}

impl FixStatistics {
    /// Create statistics from a list of fixes
    #[must_use]
    pub fn from_fixes(fixes: &[Fix]) -> Self {
        let mut stats = Self {
            total_fixes: fixes.len(),
            ..Default::default()
        };

        for fix in fixes {
            if fix.is_safe {
                stats.safe_fixes += 1;
            } else {
                stats.unsafe_fixes += 1;
            }

            match fix.severity {
                Severity::Error => stats.errors_fixed += 1,
                Severity::Warning => stats.warnings_fixed += 1,
                Severity::Hint => stats.hints_fixed += 1,
            }
        }

        stats
    }

    /// Add skipped conflicts count
    #[must_use]
    pub fn with_skipped_conflicts(mut self, count: usize) -> Self {
        self.skipped_conflicts = count;
        self
    }

    /// Get total number of fixes actually applied
    #[must_use]
    pub fn applied_count(&self) -> usize {
        self.total_fixes - self.skipped_conflicts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::range::{SourcePosition, SourceRange};

    #[test]
    fn test_statistics_default() {
        let stats = FixStatistics::default();
        assert_eq!(stats.total_fixes, 0);
        assert_eq!(stats.safe_fixes, 0);
        assert_eq!(stats.unsafe_fixes, 0);
        assert_eq!(stats.skipped_conflicts, 0);
    }

    #[test]
    fn test_statistics_from_fixes() {
        let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));

        let fix1 = Fix::new("rule", range.clone(), "text", "desc").with_severity(Severity::Error);

        let fix2 = Fix::new("rule", range.clone(), "text", "desc")
            .with_severity(Severity::Warning)
            .with_unsafe();

        let fix3 = Fix::new("rule", range, "text", "desc").with_severity(Severity::Hint);

        let stats = FixStatistics::from_fixes(&[fix1, fix2, fix3]);

        assert_eq!(stats.total_fixes, 3);
        assert_eq!(stats.safe_fixes, 2);
        assert_eq!(stats.unsafe_fixes, 1);
        assert_eq!(stats.errors_fixed, 1);
        assert_eq!(stats.warnings_fixed, 1);
        assert_eq!(stats.hints_fixed, 1);
    }

    #[test]
    fn test_statistics_with_skipped_conflicts() {
        let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
        let fix = Fix::new("rule", range, "text", "desc");

        let stats = FixStatistics::from_fixes(&[fix]).with_skipped_conflicts(1);

        assert_eq!(stats.total_fixes, 1);
        assert_eq!(stats.skipped_conflicts, 1);
        assert_eq!(stats.applied_count(), 0);
    }

    #[test]
    fn test_applied_count() {
        let stats = FixStatistics {
            total_fixes: 10,
            skipped_conflicts: 3,
            ..Default::default()
        };

        assert_eq!(stats.applied_count(), 7);
    }
}
