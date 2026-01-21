// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Rule registry for managing validation rules.

use crate::validation::{Rule, RuleCategory};
use std::collections::HashMap;

/// Configuration for a single rule.
#[derive(Debug, Clone)]
pub struct RuleConfig {
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Whether to escalate diagnostics to errors.
    pub escalate_to_error: bool,
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            escalate_to_error: false,
        }
    }
}

/// Registry for managing validation rules.
///
/// The registry provides:
/// - Rule registration by category
/// - Rule lookup and filtering
/// - Enabled/disabled rule management
/// - Rule execution order determination
pub struct RuleRegistry {
    /// All registered rules.
    rules: Vec<Box<dyn Rule>>,
    /// Per-rule configuration.
    config: HashMap<String, RuleConfig>,
}

impl RuleRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            config: HashMap::new(),
        }
    }

    /// Register a rule.
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        let rule_id = rule.id().to_string();
        self.rules.push(rule);
        self.config.entry(rule_id).or_default();
    }

    /// Register multiple rules.
    pub fn register_all(&mut self, rules: Vec<Box<dyn Rule>>) {
        for rule in rules {
            self.register(rule);
        }
    }

    /// Get all registered rules.
    pub fn rules(&self) -> &[Box<dyn Rule>] {
        &self.rules
    }

    /// Get enabled rules, sorted by cost (cheap first).
    pub fn enabled_rules(&self) -> Vec<&dyn Rule> {
        let mut enabled: Vec<&dyn Rule> = self
            .rules
            .iter()
            .filter(|r| self.is_enabled(r.id()))
            .map(|r| r.as_ref())
            .collect();

        enabled.sort_by_key(|r| r.cost_estimate());
        enabled
    }

    /// Get rules by category.
    pub fn rules_by_category(&self, category: RuleCategory) -> Vec<&dyn Rule> {
        self.rules
            .iter()
            .filter(|r| r.category() == category)
            .map(|r| r.as_ref())
            .collect()
    }

    /// Enable a rule.
    pub fn enable_rule(&mut self, rule_id: &str) {
        self.config.entry(rule_id.to_string()).or_default().enabled = true;
    }

    /// Disable a rule.
    pub fn disable_rule(&mut self, rule_id: &str) {
        self.config.entry(rule_id.to_string()).or_default().enabled = false;
    }

    /// Set rule to escalate to error.
    pub fn set_rule_error(&mut self, rule_id: &str) {
        self.config
            .entry(rule_id.to_string())
            .or_default()
            .escalate_to_error = true;
    }

    /// Check if a rule is enabled.
    pub fn is_enabled(&self, rule_id: &str) -> bool {
        self.config.get(rule_id).map(|c| c.enabled).unwrap_or(true)
    }

    /// Check if a rule should escalate to error.
    pub fn should_escalate(&self, rule_id: &str) -> bool {
        self.config
            .get(rule_id)
            .map(|c| c.escalate_to_error)
            .unwrap_or(false)
    }

    /// Get rule configuration.
    pub fn get_config(&self, rule_id: &str) -> RuleConfig {
        self.config.get(rule_id).cloned().unwrap_or_default()
    }

    /// Set rule configuration.
    pub fn set_config(&mut self, rule_id: impl Into<String>, config: RuleConfig) {
        self.config.insert(rule_id.into(), config);
    }

    /// Clear all rules.
    pub fn clear(&mut self) {
        self.rules.clear();
        self.config.clear();
    }

    /// Get count of registered rules.
    pub fn count(&self) -> usize {
        self.rules.len()
    }

    /// Get count of enabled rules.
    pub fn enabled_count(&self) -> usize {
        self.rules
            .iter()
            .filter(|r| self.is_enabled(r.id()))
            .count()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::{Diagnostic, Severity, ValidationContext};
    use crate::{Document, HedlError};

    struct TestRule {
        id: String,
        category: RuleCategory,
        cost: u8,
    }

    impl Rule for TestRule {
        fn id(&self) -> &str {
            &self.id
        }
        fn description(&self) -> &str {
            "Test rule"
        }
        fn category(&self) -> RuleCategory {
            self.category
        }
        fn default_severity(&self) -> Severity {
            Severity::Warning
        }
        fn check(
            &self,
            _doc: &Document,
            _context: &mut ValidationContext,
        ) -> Result<Vec<Diagnostic>, HedlError> {
            Ok(vec![])
        }
        fn cost_estimate(&self) -> u8 {
            self.cost
        }
    }

    fn make_test_rule(id: &str, category: RuleCategory, cost: u8) -> Box<dyn Rule> {
        Box::new(TestRule {
            id: id.to_string(),
            category,
            cost,
        })
    }

    #[test]
    fn test_registry_new() {
        let registry = RuleRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_register_rule() {
        let mut registry = RuleRegistry::new();
        let rule = make_test_rule("test", RuleCategory::Structure, 50);

        registry.register(rule);
        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn test_register_all() {
        let mut registry = RuleRegistry::new();
        let rules = vec![
            make_test_rule("rule1", RuleCategory::TypeSafety, 10),
            make_test_rule("rule2", RuleCategory::References, 20),
        ];

        registry.register_all(rules);
        assert_eq!(registry.count(), 2);
    }

    #[test]
    fn test_enabled_rules() {
        let mut registry = RuleRegistry::new();
        registry.register(make_test_rule("rule1", RuleCategory::Structure, 50));
        registry.register(make_test_rule("rule2", RuleCategory::Structure, 25));

        let enabled = registry.enabled_rules();
        assert_eq!(enabled.len(), 2);
    }

    #[test]
    fn test_enabled_rules_sorted_by_cost() {
        let mut registry = RuleRegistry::new();
        registry.register(make_test_rule("expensive", RuleCategory::Structure, 90));
        registry.register(make_test_rule("cheap", RuleCategory::Structure, 10));
        registry.register(make_test_rule("medium", RuleCategory::Structure, 50));

        let enabled = registry.enabled_rules();
        assert_eq!(enabled[0].id(), "cheap");
        assert_eq!(enabled[1].id(), "medium");
        assert_eq!(enabled[2].id(), "expensive");
    }

    #[test]
    fn test_disable_rule() {
        let mut registry = RuleRegistry::new();
        registry.register(make_test_rule("rule1", RuleCategory::Structure, 50));

        registry.disable_rule("rule1");
        assert!(!registry.is_enabled("rule1"));
        assert_eq!(registry.enabled_count(), 0);
    }

    #[test]
    fn test_enable_rule() {
        let mut registry = RuleRegistry::new();
        registry.register(make_test_rule("rule1", RuleCategory::Structure, 50));

        registry.disable_rule("rule1");
        assert!(!registry.is_enabled("rule1"));

        registry.enable_rule("rule1");
        assert!(registry.is_enabled("rule1"));
    }

    #[test]
    fn test_rules_by_category() {
        let mut registry = RuleRegistry::new();
        registry.register(make_test_rule("type1", RuleCategory::TypeSafety, 50));
        registry.register(make_test_rule("ref1", RuleCategory::References, 50));
        registry.register(make_test_rule("type2", RuleCategory::TypeSafety, 50));

        let type_rules = registry.rules_by_category(RuleCategory::TypeSafety);
        assert_eq!(type_rules.len(), 2);

        let ref_rules = registry.rules_by_category(RuleCategory::References);
        assert_eq!(ref_rules.len(), 1);
    }

    #[test]
    fn test_escalate_to_error() {
        let mut registry = RuleRegistry::new();
        registry.register(make_test_rule("rule1", RuleCategory::Structure, 50));

        registry.set_rule_error("rule1");
        assert!(registry.should_escalate("rule1"));
    }

    #[test]
    fn test_get_config() {
        let mut registry = RuleRegistry::new();
        registry.register(make_test_rule("rule1", RuleCategory::Structure, 50));

        let config = registry.get_config("rule1");
        assert!(config.enabled);
        assert!(!config.escalate_to_error);
    }

    #[test]
    fn test_set_config() {
        let mut registry = RuleRegistry::new();
        registry.register(make_test_rule("rule1", RuleCategory::Structure, 50));

        let config = RuleConfig {
            enabled: false,
            escalate_to_error: true,
        };
        registry.set_config("rule1", config);

        assert!(!registry.is_enabled("rule1"));
        assert!(registry.should_escalate("rule1"));
    }

    #[test]
    fn test_clear() {
        let mut registry = RuleRegistry::new();
        registry.register(make_test_rule("rule1", RuleCategory::Structure, 50));
        registry.register(make_test_rule("rule2", RuleCategory::Structure, 50));

        registry.clear();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_enabled_count() {
        let mut registry = RuleRegistry::new();
        registry.register(make_test_rule("rule1", RuleCategory::Structure, 50));
        registry.register(make_test_rule("rule2", RuleCategory::Structure, 50));
        registry.register(make_test_rule("rule3", RuleCategory::Structure, 50));

        assert_eq!(registry.enabled_count(), 3);

        registry.disable_rule("rule2");
        assert_eq!(registry.enabled_count(), 2);
    }
}
