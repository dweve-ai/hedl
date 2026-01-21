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

//! Comprehensive tests for `LintConfig` validation

use hedl_lint::{LintConfig, RuleConfig};

#[test]
fn test_config_validate_empty() {
    let config = LintConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validate_with_rules() {
    let mut config = LintConfig::default();
    config.enable_rule("id-naming");
    config.disable_rule("empty-list");
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validate_empty_rule_id() {
    let mut config = LintConfig::default();
    config.rules.insert(String::new(), RuleConfig::default());

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Empty rule ID"));
}

#[test]
fn test_config_validate_too_long_rule_id() {
    let mut config = LintConfig::default();
    let long_id = "a".repeat(101); // Max is 100
    config.rules.insert(long_id, RuleConfig::default());

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Rule ID too long"));
}

#[test]
fn test_config_validate_max_length_rule_id() {
    let mut config = LintConfig::default();
    let max_id = "a".repeat(100); // Exactly at max
    config.rules.insert(max_id, RuleConfig::default());

    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validate_too_many_rules() {
    let mut config = LintConfig::default();

    // Add 1001 rules (max is 1000)
    for i in 0..1001 {
        config
            .rules
            .insert(format!("rule_{i}"), RuleConfig::default());
    }

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Too many rule configurations"));
}

#[test]
fn test_config_validate_max_rules() {
    let mut config = LintConfig::default();

    // Add exactly 1000 rules
    for i in 0..1000 {
        config
            .rules
            .insert(format!("rule_{i}"), RuleConfig::default());
    }

    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validate_unicode_rule_ids() {
    let mut config = LintConfig::default();
    config
        .rules
        .insert("规则_01".to_string(), RuleConfig::default());
    config
        .rules
        .insert("правило_02".to_string(), RuleConfig::default());

    assert!(config.validate().is_ok());
}

#[test]
fn test_config_validate_unicode_too_long() {
    let mut config = LintConfig::default();
    // Unicode characters are multi-byte, so fewer characters can exceed byte limit
    let long_unicode = "规".repeat(40); // Each char is 3 bytes
    config.rules.insert(long_unicode, RuleConfig::default());

    let result = config.validate();
    // Should fail because 40 * 3 = 120 bytes > 100 byte limit
    assert!(result.is_err());
}

#[test]
fn test_config_enable_then_disable() {
    let mut config = LintConfig::default();
    config.enable_rule("test-rule");
    assert!(config.rules.get("test-rule").unwrap().enabled);

    config.disable_rule("test-rule");
    assert!(!config.rules.get("test-rule").unwrap().enabled);
}

#[test]
fn test_config_set_rule_error_enables_rule() {
    let mut config = LintConfig::default();
    config.set_rule_error("strict-rule");

    let rule_config = config.rules.get("strict-rule").unwrap();
    assert!(rule_config.enabled);
    assert!(rule_config.error);
}

#[test]
fn test_config_multiple_operations_on_same_rule() {
    let mut config = LintConfig::default();

    config.enable_rule("test");
    assert!(config.rules.get("test").unwrap().enabled);
    assert!(!config.rules.get("test").unwrap().error);

    config.set_rule_error("test");
    assert!(config.rules.get("test").unwrap().enabled);
    assert!(config.rules.get("test").unwrap().error);

    config.disable_rule("test");
    assert!(!config.rules.get("test").unwrap().enabled);
    // Error flag should remain
    assert!(!config.rules.get("test").unwrap().error);
}

#[test]
fn test_config_max_diagnostics_default() {
    let config = LintConfig::default();
    assert_eq!(config.max_diagnostics, 10_000);
}

#[test]
fn test_config_max_diagnostics_custom() {
    let config = LintConfig {
        max_diagnostics: 100,
        ..Default::default()
    };
    assert_eq!(config.max_diagnostics, 100);
}

#[test]
fn test_config_clone_preserves_state() {
    let mut config = LintConfig::default();
    config.enable_rule("rule1");
    config.set_rule_error("rule2");
    config.max_diagnostics = 500;

    let cloned = config.clone();
    assert_eq!(cloned.rules.len(), 2);
    assert!(cloned.rules.get("rule1").unwrap().enabled);
    assert!(cloned.rules.get("rule2").unwrap().error);
    assert_eq!(cloned.max_diagnostics, 500);
}

#[cfg(feature = "parallel")]
#[test]
fn test_config_parallel_default_enabled() {
    let config = LintConfig::default();
    assert!(config.parallel);
}

#[cfg(feature = "parallel")]
#[test]
fn test_config_parallel_can_be_disabled() {
    let config = LintConfig {
        parallel: false,
        ..Default::default()
    };
    assert!(!config.parallel);
}
