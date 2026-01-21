// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www/apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Fix application engine

use crate::fix::{
    config::FixConfig, conflict::ConflictDetector, diff::DiffGenerator, error::FixError,
    ordering::FixOrderer, preview::FixPreview, statistics::FixStatistics, verifier::FixVerifier,
    Fix, FixResult,
};

/// Applies fixes to source text
pub struct FixApplicator {
    config: FixConfig,
    verifier: FixVerifier,
}

impl FixApplicator {
    /// Create a new fix applicator with configuration
    #[must_use]
    pub fn new(config: FixConfig) -> Self {
        Self {
            config,
            verifier: FixVerifier,
        }
    }

    /// Apply a single fix to source text
    pub fn apply_fix(&self, source: &str, fix: &Fix) -> Result<String, FixError> {
        // Validate fix
        self.verifier.verify_pre_application(source, fix)?;

        // Extract text by line and column
        let lines: Vec<&str> = source.lines().collect();
        let mut result = String::new();

        // Process lines before the fix range
        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;

            if line_num < fix.range.start.line || line_num > fix.range.end.line {
                // Lines outside the fix range - preserve as-is
                result.push_str(line);
                result.push('\n');
            } else if line_num == fix.range.start.line && line_num == fix.range.end.line {
                // Single line replacement
                let before = &line[..fix.range.start.column.min(line.len())];
                let after = &line[fix.range.end.column.min(line.len())..];
                result.push_str(before);
                result.push_str(&fix.replacement);
                result.push_str(after);
                result.push('\n');
            } else if line_num == fix.range.start.line {
                // Start of multi-line replacement
                let before = &line[..fix.range.start.column.min(line.len())];
                result.push_str(before);
                result.push_str(&fix.replacement);
            } else if line_num == fix.range.end.line {
                // End of multi-line replacement
                let after = &line[fix.range.end.column.min(line.len())..];
                result.push_str(after);
                result.push('\n');
            }
            // Skip lines in the middle of the range
        }

        // Ensure we have trailing newline if original had one
        if source.ends_with('\n') && !result.ends_with('\n') {
            result.push('\n');
        }

        // Handle missing trailing newline when source doesn't have one
        if !source.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }

        // Verify result unless verification is skipped
        if !self.config.skip_verification {
            self.verifier
                .verify_post_application(source, &result, std::slice::from_ref(fix))?;
        }

        Ok(result)
    }

    /// Apply multiple fixes with conflict resolution
    #[must_use]
    pub fn apply_fixes(&self, source: &str, mut fixes: Vec<Fix>) -> FixResult {
        // Filter by safety configuration
        if !self.config.apply_unsafe {
            fixes.retain(|f| f.is_safe);
        }

        // Apply max fixes limit
        if let Some(max) = self.config.max_fixes {
            fixes.truncate(max);
        }

        // Detect conflicts
        let detector = ConflictDetector::new(self.config.conflict_strategy);
        let conflicts = detector.detect_conflicts(&fixes);
        let resolved_conflicts = detector.resolve_conflicts(conflicts, &fixes);

        // Filter out conflicted fixes
        let applicable = detector.apply_resolutions(fixes, &resolved_conflicts);

        // Order fixes optimally
        let ordered = match FixOrderer::order_optimal(applicable) {
            Ok(ordered) => ordered,
            Err(e) => return FixResult::error(e),
        };

        // Apply fixes sequentially
        let mut result = source.to_string();
        let mut applied = Vec::new();
        let mut errors = Vec::new();

        for fix in ordered {
            match self.apply_fix(&result, &fix) {
                Ok(fixed) => {
                    result = fixed;
                    applied.push(fix);
                }
                Err(e) => {
                    errors.push(e);
                    if self.config.fail_on_error {
                        return FixResult {
                            success: false,
                            fixed_source: None,
                            errors,
                            conflicts: resolved_conflicts,
                            applied_fixes: applied,
                        };
                    }
                }
            }
        }

        FixResult {
            success: errors.is_empty(),
            fixed_source: Some(result),
            errors,
            conflicts: resolved_conflicts,
            applied_fixes: applied,
        }
    }

    /// Preview fixes without applying
    #[must_use]
    pub fn preview_fixes(&self, source: &str, fixes: Vec<Fix>) -> FixPreview {
        // Generate estimated result
        let result = self.apply_fixes(source, fixes.clone());

        let estimated_result = result.fixed_source.unwrap_or_else(|| source.to_string());

        // Generate diff
        let diff_gen = DiffGenerator::default();
        let diff = diff_gen.generate_diff(source, &estimated_result);

        // Calculate statistics
        let statistics = FixStatistics::from_fixes(&result.applied_fixes)
            .with_skipped_conflicts(result.conflicts.len());

        FixPreview::new(
            diff,
            result.applied_fixes.len(),
            result.conflicts,
            estimated_result,
            statistics,
        )
    }
}

impl Default for FixApplicator {
    fn default() -> Self {
        Self::new(FixConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fix::range::{SourcePosition, SourceRange};

    /// Helper function to create an applicator suitable for unit tests
    fn test_applicator() -> FixApplicator {
        FixApplicator::new(FixConfig::default().with_skip_verification(true))
    }

    #[test]
    fn test_apply_single_line_fix() {
        let applicator = test_applicator();
        let source = "User a Alice\n";

        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 6)),
            "alice",
            "Replace short ID",
        );

        let result = applicator.apply_fix(source, &fix).unwrap();
        assert_eq!(result, "User alice Alice\n");
    }

    #[test]
    fn test_apply_fix_invalid_range() {
        let applicator = test_applicator();
        let source = "User a Alice\n";

        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(10, 0), SourcePosition::new(10, 5)),
            "text",
            "desc",
        );

        assert!(applicator.apply_fix(source, &fix).is_err());
    }

    #[test]
    fn test_apply_multiple_fixes() {
        let applicator = test_applicator();
        let source = "User a Alice\nUser b Bob\n";

        let fix1 = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 6)),
            "alice",
            "Fix 1",
        );
        let fix2 = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(2, 5), SourcePosition::new(2, 6)),
            "bob",
            "Fix 2",
        );

        let result = applicator.apply_fixes(source, vec![fix1, fix2]);
        assert!(result.success);
        assert_eq!(result.applied_fixes.len(), 2);
    }

    #[test]
    fn test_apply_fixes_with_conflicts() {
        let applicator = test_applicator();
        let source = "User a Alice\n";

        let fix1 = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 6)),
            "alice",
            "Fix 1",
        );
        let fix2 = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 6)),
            "user_a",
            "Fix 2",
        );

        let result = applicator.apply_fixes(source, vec![fix1, fix2]);
        // One fix should be applied, one skipped due to conflict
        assert_eq!(result.applied_fixes.len(), 1);
        assert!(!result.conflicts.is_empty());
    }

    #[test]
    fn test_apply_fixes_respects_safety() {
        let config = FixConfig::safe_only().with_skip_verification(true);
        let applicator = FixApplicator::new(config);
        let source = "User a Alice\n";

        let safe_fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 6)),
            "alice",
            "Safe fix",
        );

        let unsafe_fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 7), SourcePosition::new(1, 12)),
            "Bob",
            "Unsafe fix",
        )
        .with_unsafe();

        let result = applicator.apply_fixes(source, vec![safe_fix, unsafe_fix]);
        // Only safe fix should be applied
        assert_eq!(result.applied_fixes.len(), 1);
        assert!(result.applied_fixes[0].is_safe);
    }

    #[test]
    fn test_apply_fixes_respects_max_limit() {
        let config = FixConfig::default()
            .with_max_fixes(1)
            .with_skip_verification(true);
        let applicator = FixApplicator::new(config);
        let source = "User a Alice\nUser b Bob\n";

        let fix1 = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 6)),
            "alice",
            "Fix 1",
        );
        let fix2 = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(2, 5), SourcePosition::new(2, 6)),
            "bob",
            "Fix 2",
        );

        let result = applicator.apply_fixes(source, vec![fix1, fix2]);
        // Only 1 fix should be applied due to limit
        assert_eq!(result.applied_fixes.len(), 1);
    }

    #[test]
    fn test_preview_fixes() {
        let applicator = test_applicator();
        let source = "User a Alice\n";

        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 6)),
            "alice",
            "Replace short ID",
        );

        let preview = applicator.preview_fixes(source, vec![fix]);
        assert_eq!(preview.fixes_count, 1);
        assert!(!preview.diff.is_empty());
        assert_eq!(preview.statistics.total_fixes, 1);
    }

    #[test]
    fn test_apply_fix_preserves_trailing_newline() {
        let applicator = test_applicator();
        let source = "User a Alice\n";

        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 6)),
            "alice",
            "desc",
        );

        let result = applicator.apply_fix(source, &fix).unwrap();
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_apply_fix_no_trailing_newline() {
        let applicator = test_applicator();
        let source = "User a Alice";

        let fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 6)),
            "alice",
            "desc",
        );

        let result = applicator.apply_fix(source, &fix).unwrap();
        assert!(!result.ends_with('\n'));
    }

    #[test]
    fn test_apply_fixes_ordering() {
        let applicator = test_applicator();
        let source = "line1\nline2\nline3\n";

        // Create fixes in wrong order
        let fix1 = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5)),
            "LINE1",
            "Fix line 1",
        );
        let fix2 = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(3, 0), SourcePosition::new(3, 5)),
            "LINE3",
            "Fix line 3",
        );

        let result = applicator.apply_fixes(source, vec![fix1, fix2]);
        assert!(result.success);
        assert_eq!(result.applied_fixes.len(), 2);
    }

    #[test]
    fn test_apply_fixes_fail_on_error() {
        let config = FixConfig::default()
            .with_fail_on_error(true)
            .with_skip_verification(true);
        let applicator = FixApplicator::new(config);
        let source = "User a Alice\n";

        let good_fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 6)),
            "alice",
            "Good fix",
        );

        let bad_fix = Fix::new(
            "test",
            SourceRange::new(SourcePosition::new(10, 0), SourcePosition::new(10, 5)),
            "bad",
            "Bad fix",
        );

        let result = applicator.apply_fixes(source, vec![good_fix, bad_fix]);
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }
}
