// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the validation framework.

use hedl_core::validation::{LintConfig, ValidationRunner};
use hedl_core::Document;

#[test]
fn test_validation_framework_basic() {
    let doc = Document::new((1, 0));
    let runner = ValidationRunner::new(LintConfig::default());
    let result = runner.validate(&doc);

    // Empty document should be valid
    assert!(result.is_valid);
    assert_eq!(result.diagnostics.len(), 0);
}

#[test]
fn test_validation_runner_with_custom_config() {
    let mut config = LintConfig::default();
    config.disable_rule("duplicate-key");

    let runner = ValidationRunner::new(config);
    let doc = Document::new((1, 0));
    let result = runner.validate(&doc);

    assert!(result.is_valid);
}

#[test]
fn test_validation_result_filters() {
    let doc = Document::new((1, 0));
    let runner = ValidationRunner::new(LintConfig::default());
    let result = runner.validate(&doc);

    // Test filter methods
    assert_eq!(result.errors().count(), 0);
    assert_eq!(result.warnings().count(), 0);
    assert_eq!(result.hints().count(), 0);
}

#[test]
fn test_validation_stats() {
    let doc = Document::new((1, 0));
    let runner = ValidationRunner::new(LintConfig::default());
    let result = runner.validate(&doc);

    // Should have executed some rules
    assert!(result.stats.rules_executed > 0);
    assert!(result.stats.total_duration.as_nanos() > 0);
}
