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

//! Configuration for fix behavior

use std::collections::HashSet;

/// Strategy for resolving conflicts between fixes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictStrategy {
    /// Skip conflicting fixes
    Skip,
    /// Prefer higher priority (error > warning > hint)
    #[default]
    PreferPriority,
    /// Fail on any conflict
    Fail,
    /// Apply first in order
    PreferFirst,
}

/// Configuration for fix generation and application
#[derive(Debug, Clone, Default)]
pub struct FixConfig {
    /// Whether to apply unsafe fixes
    pub apply_unsafe: bool,
    /// Maximum number of fixes to apply (None for unlimited)
    pub max_fixes: Option<usize>,
    /// Rules to generate fixes for (empty means all)
    pub enabled_rules: HashSet<String>,
    /// Conflict resolution strategy
    pub conflict_strategy: ConflictStrategy,
    /// Whether to fail on first error
    pub fail_on_error: bool,
    /// Whether to skip post-application verification (useful for testing)
    pub skip_verification: bool,
}

impl FixConfig {
    /// Create a config that applies all fixes including unsafe ones
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            apply_unsafe: true,
            ..Default::default()
        }
    }

    /// Create a config that only applies safe fixes
    #[must_use]
    pub fn safe_only() -> Self {
        Self {
            apply_unsafe: false,
            ..Default::default()
        }
    }

    /// Enable a specific rule
    pub fn enable_rule(mut self, rule_id: impl Into<String>) -> Self {
        self.enabled_rules.insert(rule_id.into());
        self
    }

    /// Set conflict strategy
    #[must_use]
    pub fn with_conflict_strategy(mut self, strategy: ConflictStrategy) -> Self {
        self.conflict_strategy = strategy;
        self
    }

    /// Set maximum number of fixes
    #[must_use]
    pub fn with_max_fixes(mut self, max: usize) -> Self {
        self.max_fixes = Some(max);
        self
    }

    /// Set whether to fail on first error
    #[must_use]
    pub fn with_fail_on_error(mut self, fail: bool) -> Self {
        self.fail_on_error = fail;
        self
    }

    /// Set whether to skip verification (for testing incomplete fragments)
    #[must_use]
    pub fn with_skip_verification(mut self, skip: bool) -> Self {
        self.skip_verification = skip;
        self
    }

    /// Check if a rule is enabled
    #[must_use]
    pub fn is_rule_enabled(&self, rule_id: &str) -> bool {
        self.enabled_rules.is_empty() || self.enabled_rules.contains(rule_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_strategy_default() {
        assert_eq!(
            ConflictStrategy::default(),
            ConflictStrategy::PreferPriority
        );
    }

    #[test]
    fn test_conflict_strategy_equality() {
        assert_eq!(ConflictStrategy::Skip, ConflictStrategy::Skip);
        assert_ne!(ConflictStrategy::Skip, ConflictStrategy::Fail);
    }

    #[test]
    fn test_fix_config_default() {
        let config = FixConfig::default();
        assert!(!config.apply_unsafe);
        assert!(config.max_fixes.is_none());
        assert!(config.enabled_rules.is_empty());
        assert_eq!(config.conflict_strategy, ConflictStrategy::PreferPriority);
        assert!(!config.fail_on_error);
    }

    #[test]
    fn test_fix_config_permissive() {
        let config = FixConfig::permissive();
        assert!(config.apply_unsafe);
    }

    #[test]
    fn test_fix_config_safe_only() {
        let config = FixConfig::safe_only();
        assert!(!config.apply_unsafe);
    }

    #[test]
    fn test_fix_config_enable_rule() {
        let config = FixConfig::default()
            .enable_rule("rule1")
            .enable_rule("rule2");

        assert!(config.enabled_rules.contains("rule1"));
        assert!(config.enabled_rules.contains("rule2"));
    }

    #[test]
    fn test_fix_config_with_conflict_strategy() {
        let config = FixConfig::default().with_conflict_strategy(ConflictStrategy::Fail);

        assert_eq!(config.conflict_strategy, ConflictStrategy::Fail);
    }

    #[test]
    fn test_fix_config_with_max_fixes() {
        let config = FixConfig::default().with_max_fixes(10);
        assert_eq!(config.max_fixes, Some(10));
    }

    #[test]
    fn test_fix_config_with_fail_on_error() {
        let config = FixConfig::default().with_fail_on_error(true);
        assert!(config.fail_on_error);
    }

    #[test]
    fn test_is_rule_enabled_empty_set() {
        let config = FixConfig::default();
        assert!(config.is_rule_enabled("any-rule"));
    }

    #[test]
    fn test_is_rule_enabled_with_rules() {
        let config = FixConfig::default()
            .enable_rule("rule1")
            .enable_rule("rule2");

        assert!(config.is_rule_enabled("rule1"));
        assert!(config.is_rule_enabled("rule2"));
        assert!(!config.is_rule_enabled("rule3"));
    }

    #[test]
    fn test_fix_config_builder_chain() {
        let config = FixConfig::default()
            .enable_rule("test")
            .with_max_fixes(5)
            .with_conflict_strategy(ConflictStrategy::Skip)
            .with_fail_on_error(true);

        assert!(config.is_rule_enabled("test"));
        assert_eq!(config.max_fixes, Some(5));
        assert_eq!(config.conflict_strategy, ConflictStrategy::Skip);
        assert!(config.fail_on_error);
    }
}
