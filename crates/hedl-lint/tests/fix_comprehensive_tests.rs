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

//! Comprehensive tests for fix functionality

use hedl_lint::fix::{
    ConflictStrategy, ConflictType, Fix, FixApplicator, FixConfig, FixContext, FixError, FixResult,
    SourcePosition, SourceRange,
};
use hedl_lint::Severity;

// =============================================================================
// SourceRange tests
// =============================================================================

#[test]
fn test_source_range_overlaps_true() {
    let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
    let range2 = SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15));

    assert!(range1.overlaps(&range2));
    assert!(range2.overlaps(&range1));
}

#[test]
fn test_source_range_overlaps_false() {
    let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
    let range2 = SourceRange::new(SourcePosition::new(1, 10), SourcePosition::new(1, 20));

    assert!(!range1.overlaps(&range2));
    assert!(!range2.overlaps(&range1));
}

#[test]
fn test_source_range_overlaps_adjacent() {
    let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
    let range2 = SourceRange::new(SourcePosition::new(1, 10), SourcePosition::new(1, 20));

    assert!(!range1.overlaps(&range2));
}

#[test]
fn test_source_range_contains_range() {
    let outer = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 20));
    let inner = SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15));

    assert!(outer.contains(&inner));
    assert!(!inner.contains(&outer));
}

#[test]
fn test_source_range_contains_position() {
    let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));

    assert!(range.contains_position(&SourcePosition::new(1, 5)));
    assert!(!range.contains_position(&SourcePosition::new(1, 10))); // End is exclusive
    assert!(!range.contains_position(&SourcePosition::new(2, 0)));
}

#[test]
fn test_source_range_is_empty() {
    let point = SourceRange::point(SourcePosition::new(1, 5));
    assert!(point.is_empty());

    let normal = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
    assert!(!normal.is_empty());
}

#[test]
fn test_source_range_merge() {
    let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
    let range2 = SourceRange::new(SourcePosition::new(1, 5), SourcePosition::new(1, 15));

    let merged = range1.merge(&range2);

    assert_eq!(merged.start, SourcePosition::new(1, 0));
    assert_eq!(merged.end, SourcePosition::new(1, 15));
}

#[test]
fn test_source_range_merge_disjoint() {
    let range1 = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
    let range2 = SourceRange::new(SourcePosition::new(2, 0), SourcePosition::new(2, 5));

    let merged = range1.merge(&range2);

    assert_eq!(merged.start, SourcePosition::new(1, 0));
    assert_eq!(merged.end, SourcePosition::new(2, 5));
}

#[test]
fn test_source_range_line_count() {
    let single_line = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
    assert_eq!(single_line.line_count(), 0);

    let two_lines = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(2, 0));
    assert_eq!(two_lines.line_count(), 1);

    let three_lines = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(3, 0));
    assert_eq!(three_lines.line_count(), 2);
}

#[test]
fn test_source_range_is_valid() {
    let valid = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 10));
    assert!(valid.is_valid());

    let invalid = SourceRange::new(SourcePosition::new(1, 10), SourcePosition::new(1, 0));
    assert!(!invalid.is_valid());

    let point = SourceRange::point(SourcePosition::new(1, 5));
    assert!(point.is_valid());
}

#[test]
fn test_source_range_line_helper() {
    let range = SourceRange::line(5);

    assert_eq!(range.start, SourcePosition::new(5, 0));
    assert_eq!(range.end, SourcePosition::new(6, 0));
}

// =============================================================================
// FixContext tests
// =============================================================================

#[test]
fn test_fix_context_from_source() {
    let source = "line1\nline2\nline3";
    let context = FixContext::from_source(source);

    assert_eq!(context.line_count(), 3);
}

#[test]
fn test_fix_context_get_line() {
    let source = "line1\nline2\nline3";
    let context = FixContext::from_source(source);

    assert_eq!(context.get_line(1), Some("line1"));
    assert_eq!(context.get_line(2), Some("line2"));
    assert_eq!(context.get_line(3), Some("line3"));
    assert_eq!(context.get_line(4), None);
}

#[test]
fn test_fix_context_get_line_zero() {
    let source = "line1\nline2";
    let context = FixContext::from_source(source);

    assert_eq!(context.get_line(0), None);
}

#[test]
fn test_fix_context_empty_source() {
    let context = FixContext::from_source("");
    // Empty string represents one empty line
    assert_eq!(context.line_count(), 1);
}

#[test]
fn test_fix_context_single_line() {
    let context = FixContext::from_source("single line");
    assert_eq!(context.line_count(), 1);
    assert_eq!(context.get_line(1), Some("single line"));
}

#[test]
fn test_fix_context_trailing_newline() {
    let context = FixContext::from_source("line1\nline2\n");
    assert_eq!(context.line_count(), 2);
}

// =============================================================================
// FixConfig tests
// =============================================================================

#[test]
fn test_fix_config_default() {
    let config = FixConfig::default();

    assert!(!config.apply_unsafe);
    assert!(!config.fail_on_error);
    assert!(config.max_fixes.is_none());
    assert!(matches!(
        config.conflict_strategy,
        ConflictStrategy::PreferPriority
    ));
}

#[test]
fn test_fix_config_safe_only() {
    let config = FixConfig::safe_only();

    assert!(!config.apply_unsafe);
}

#[test]
fn test_fix_config_fail_on_error() {
    let config = FixConfig::default().with_fail_on_error(true);

    assert!(config.fail_on_error);
}

#[test]
fn test_fix_config_max_fixes() {
    let config = FixConfig::default().with_max_fixes(10);

    assert_eq!(config.max_fixes, Some(10));
}

#[test]
fn test_fix_config_conflict_strategy() {
    let config = FixConfig::default().with_conflict_strategy(ConflictStrategy::PreferFirst);

    assert!(matches!(
        config.conflict_strategy,
        ConflictStrategy::PreferFirst
    ));
}

#[test]
fn test_fix_config_permissive() {
    let config = FixConfig::permissive();

    assert!(config.apply_unsafe);
    assert!(!config.fail_on_error);
}

#[test]
fn test_fix_config_enable_rules() {
    let config = FixConfig::default()
        .enable_rule("rule1")
        .enable_rule("rule2");

    assert!(config.is_rule_enabled("rule1"));
    assert!(config.is_rule_enabled("rule2"));
    assert!(!config.is_rule_enabled("rule3"));
}

#[test]
fn test_fix_config_rule_enabled_default() {
    let config = FixConfig::default();

    // When enabled_rules is empty, all rules are enabled
    assert!(config.is_rule_enabled("any-rule"));
}

#[test]
fn test_fix_config_builder_chain() {
    let config = FixConfig::default()
        .enable_rule("test")
        .with_fail_on_error(true)
        .with_max_fixes(5)
        .with_conflict_strategy(ConflictStrategy::Skip);

    assert!(config.is_rule_enabled("test"));
    assert!(config.fail_on_error);
    assert_eq!(config.max_fixes, Some(5));
    assert!(matches!(config.conflict_strategy, ConflictStrategy::Skip));
}

// =============================================================================
// Fix creation and modification tests
// =============================================================================

#[test]
fn test_fix_with_severity() {
    let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
    let fix = Fix::new("test", range, "replacement", "description").with_severity(Severity::Error);

    assert_eq!(fix.severity, Severity::Error);
}

#[test]
fn test_fix_with_unsafe() {
    let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
    let fix = Fix::new("test", range, "replacement", "description").with_unsafe();

    assert!(!fix.is_safe);
}

#[test]
fn test_fix_with_dependency() {
    use uuid::Uuid;

    let dep_id = Uuid::new_v4();
    let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
    let fix = Fix::new("test", range, "replacement", "description").with_dependency(dep_id);

    assert_eq!(fix.dependencies.len(), 1);
    assert_eq!(fix.dependencies[0], dep_id);
}

#[test]
fn test_fix_with_multiple_dependencies() {
    use uuid::Uuid;

    let deps = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
    let range = SourceRange::new(SourcePosition::new(1, 0), SourcePosition::new(1, 5));
    let fix = Fix::new("test", range, "replacement", "description").with_dependencies(deps.clone());

    assert_eq!(fix.dependencies.len(), 3);
    assert_eq!(fix.dependencies, deps);
}

// =============================================================================
// FixResult tests
// =============================================================================

#[test]
fn test_fix_result_success_creation() {
    let result = FixResult::success("fixed source".to_string(), vec![]);

    assert!(result.success);
    assert_eq!(result.fixed_source, Some("fixed source".to_string()));
    assert!(result.errors.is_empty());
    assert!(result.conflicts.is_empty());
}

#[test]
fn test_fix_result_error_creation() {
    let error = FixError::InvalidRange("test range".to_string());
    let result = FixResult::error(error);

    assert!(!result.success);
    assert!(result.fixed_source.is_none());
    assert_eq!(result.errors.len(), 1);
}

#[test]
fn test_fix_result_with_conflicts() {
    let result = FixResult::with_conflicts(vec![]);

    assert!(!result.success);
    assert!(result.fixed_source.is_none());
}

// =============================================================================
// FixApplicator integration tests
// =============================================================================
// Note: These tests verify the API exists and basic behavior.
// Full applicator integration tests exist in the fix module unit tests.

#[test]
fn test_applicator_creation() {
    let applicator = FixApplicator::new(FixConfig::default());
    // Successfully created applicator
    let _ = applicator;
}

#[test]
fn test_applicator_empty_fixes() {
    let source = "test";
    let applicator = FixApplicator::new(FixConfig::default());

    let result = applicator.apply_fixes(source, vec![]);

    // Empty fix set should succeed
    assert!(result.success);
}

// =============================================================================
// Conflict detection edge cases
// =============================================================================

#[test]
fn test_conflict_type_display() {
    let overlapping = format!("{:?}", ConflictType::Overlapping);
    assert!(overlapping.contains("Overlapping"));

    let contradictory = format!("{:?}", ConflictType::Contradictory);
    assert!(contradictory.contains("Contradictory"));

    let dependent = format!("{:?}", ConflictType::Dependent);
    assert!(dependent.contains("Dependent"));
}

#[test]
fn test_conflict_strategy_display() {
    let skip = format!("{:?}", ConflictStrategy::Skip);
    assert!(skip.contains("Skip"));

    let prefer_first = format!("{:?}", ConflictStrategy::PreferFirst);
    assert!(prefer_first.contains("PreferFirst"));

    let prefer_priority = format!("{:?}", ConflictStrategy::PreferPriority);
    assert!(prefer_priority.contains("PreferPriority"));

    let fail = format!("{:?}", ConflictStrategy::Fail);
    assert!(fail.contains("Fail"));
}

// =============================================================================
// FixError tests
// =============================================================================

#[test]
fn test_fix_error_variants() {
    let invalid_range = FixError::InvalidRange("bad range".to_string());
    assert!(matches!(invalid_range, FixError::InvalidRange(_)));

    let application_failed = FixError::ApplicationFailed("failed".to_string());
    assert!(matches!(application_failed, FixError::ApplicationFailed(_)));

    let circular = FixError::CircularDependency("circular".to_string());
    assert!(matches!(circular, FixError::CircularDependency(_)));

    let encoding = FixError::EncodingError("bad utf8".to_string());
    assert!(matches!(encoding, FixError::EncodingError(_)));
}

#[test]
fn test_fix_error_display() {
    let error = FixError::InvalidRange("test".to_string());
    let display = format!("{error:?}");
    assert!(display.contains("InvalidRange"));
}

#[test]
fn test_fix_error_clone() {
    let error = FixError::ApplicationFailed("test".to_string());
    let cloned = error.clone();

    match (error, cloned) {
        (FixError::ApplicationFailed(msg1), FixError::ApplicationFailed(msg2)) => {
            assert_eq!(msg1, msg2);
        }
        _ => panic!("Clone failed"),
    }
}
