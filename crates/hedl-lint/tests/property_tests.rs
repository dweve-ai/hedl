// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Property-based tests for hedl-lint
//!
//! These tests verify systematic properties of lint rules across
//! large input spaces using automatically generated test cases.
//!
//! # Test Organization
//!
//! - **Core Properties**: Determinism, idempotence, configuration consistency
//! - **Rule-Specific Properties**: Properties for each lint rule
//! - **Integration Properties**: Cross-rule interactions and aggregation
//!
//! # Running Tests
//!
//! ```bash
//! # Run all property tests with default case count (100)
//! cargo test --test property_tests
//!
//! # Run with more cases for deeper testing
//! PROPTEST_CASES=1000 cargo test --test property_tests
//!
//! # Run a specific property test
//! cargo test --test property_tests test_lint_determinism
//! ```
//!
//! # Regression Files
//!
//! Proptest stores minimal failing test cases in `.proptest-regressions/`
//! directory. These files should be committed to git to ensure that
//! previously found bugs don't regress.

use hedl_lint::{lint, lint_with_config, DiagnosticKind, LintConfig, Severity};
use proptest::prelude::*;

mod proptest_generators;
use proptest_generators::*;

// ===== Core Properties =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Linting the same document twice produces identical results
    ///
    /// This tests determinism, which is critical for:
    /// - Reproducible builds
    /// - CI/CD consistency
    /// - Caching lint results
    ///
    /// # Invariant
    /// For all documents D: lint(D) = lint(D)
    #[test]
    fn test_lint_determinism(doc in simple_document()) {
        let diagnostics1 = lint(&doc);
        let diagnostics2 = lint(&doc);

        prop_assert_eq!(
            diagnostics1.len(),
            diagnostics2.len(),
            "Lint should produce same number of diagnostics"
        );

        // Compare diagnostic content (order and details)
        for (d1, d2) in diagnostics1.iter().zip(diagnostics2.iter()) {
            prop_assert_eq!(d1.severity(), d2.severity(), "Severity mismatch");
            prop_assert_eq!(format!("{:?}", d1.kind()), format!("{:?}", d2.kind()), "Kind mismatch");
            prop_assert_eq!(d1.message(), d2.message(), "Message mismatch");
        }
    }

    /// Property: Linting multiple times is idempotent
    ///
    /// Running lint 3 times should produce identical results each time.
    ///
    /// # Invariant
    /// For all documents D: lint(D) = lint(D) = lint(D)
    #[test]
    fn test_lint_idempotence(doc in multi_field_document()) {
        let run1 = lint(&doc);
        let run2 = lint(&doc);
        let run3 = lint(&doc);

        prop_assert_eq!(run1.len(), run2.len(), "Run 1 vs 2 count mismatch");
        prop_assert_eq!(run2.len(), run3.len(), "Run 2 vs 3 count mismatch");

        // All runs should produce identical diagnostics
        for ((d1, d2), d3) in run1.iter().zip(run2.iter()).zip(run3.iter()) {
            prop_assert_eq!(format!("{:?}", d1.kind()), format!("{:?}", d2.kind()), "Run 1 vs 2 kind mismatch");
            prop_assert_eq!(format!("{:?}", d2.kind()), format!("{:?}", d3.kind()), "Run 2 vs 3 kind mismatch");
        }
    }

    /// Property: Disabling a rule reduces or maintains diagnostic count
    ///
    /// Disabling rules should never INCREASE the number of diagnostics.
    ///
    /// # Invariant
    /// For all documents D and rules R:
    ///   count(lint_with_config(D, disabled=R)) <= count(lint(D))
    #[test]
    fn test_disable_rule_reduces_diagnostics(doc in multi_field_document()) {
        let all_diagnostics = lint(&doc);

        // Disable id-naming rule
        let mut config = LintConfig::default();
        config.disable_rule("id-naming");
        let filtered_diagnostics = lint_with_config(&doc, config);

        // Filtered count should be <= original count
        prop_assert!(
            filtered_diagnostics.len() <= all_diagnostics.len(),
            "Disabling rule increased diagnostics: {} -> {}",
            all_diagnostics.len(),
            filtered_diagnostics.len()
        );

        // No id-naming diagnostics should remain
        let id_naming_count = filtered_diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
            .count();

        prop_assert_eq!(id_naming_count, 0, "Disabled rule still triggered");
    }

    /// Property: Setting min_severity filters diagnostics correctly
    ///
    /// Higher severity thresholds should never increase diagnostic count.
    ///
    /// # Invariant
    /// For all documents D:
    ///   count(lint(D, min_severity=Error)) <= count(lint(D, min_severity=Warning)) <= count(lint(D, min_severity=Hint))
    #[test]
    fn test_severity_filtering(doc in multi_field_document()) {
        // Get all diagnostics (min_severity = Hint)
        let config_hint = LintConfig {
            min_severity: Severity::Hint,
            ..Default::default()
        };
        let hint_diagnostics = lint_with_config(&doc, config_hint);

        // Get warning+ diagnostics
        let config_warning = LintConfig {
            min_severity: Severity::Warning,
            ..Default::default()
        };
        let warning_diagnostics = lint_with_config(&doc, config_warning);

        // Get error-only diagnostics
        let config_error = LintConfig {
            min_severity: Severity::Error,
            ..Default::default()
        };
        let error_diagnostics = lint_with_config(&doc, config_error);

        // Property: error_count <= warning_count <= hint_count
        prop_assert!(
            error_diagnostics.len() <= warning_diagnostics.len(),
            "Error count ({}) > warning count ({})",
            error_diagnostics.len(),
            warning_diagnostics.len()
        );
        prop_assert!(
            warning_diagnostics.len() <= hint_diagnostics.len(),
            "Warning count ({}) > hint count ({})",
            warning_diagnostics.len(),
            hint_diagnostics.len()
        );

        // Verify all remaining diagnostics meet threshold
        for diag in &warning_diagnostics {
            prop_assert!(
                diag.severity() >= Severity::Warning,
                "Found {:?} diagnostic when min_severity=Warning",
                diag.severity()
            );
        }

        for diag in &error_diagnostics {
            prop_assert!(
                diag.severity() >= Severity::Error,
                "Found {:?} diagnostic when min_severity=Error",
                diag.severity()
            );
        }
    }

    /// Property: Well-formed documents produce no diagnostics
    ///
    /// Documents that follow all best practices should produce zero diagnostics.
    ///
    /// # Invariant
    /// For all well-formed documents D: lint(D) = []
    #[test]
    fn test_well_formed_no_diagnostics(doc in well_formed_document()) {
        let diagnostics = lint(&doc);

        prop_assert!(
            diagnostics.is_empty(),
            "Well-formed document produced {} diagnostics: {:?}",
            diagnostics.len(),
            diagnostics.iter().map(|d| format!("{:?}", d.kind())).collect::<Vec<_>>()
        );
    }

    /// Property: Rule escalation converts all severities to Error
    ///
    /// When a rule is set to error mode, all its diagnostics should be
    /// escalated to Error severity.
    ///
    /// # Invariant
    /// For all documents D and rules R:
    ///   For all diagnostics in lint_with_config(D, error=R):
    ///     If diagnostic.rule = R then diagnostic.severity = Error
    #[test]
    fn test_rule_escalation_to_error(doc in empty_list_document()) {
        let mut config = LintConfig::default();
        config.set_rule_error("empty-list");

        let diagnostics = lint_with_config(&doc, config);

        // All empty-list diagnostics should be escalated to Error
        for diag in &diagnostics {
            if matches!(diag.kind(), DiagnosticKind::EmptyList) {
                prop_assert_eq!(
                    diag.severity(),
                    Severity::Error,
                    "EmptyList diagnostic not escalated to Error"
                );
            }
        }
    }

    /// Property: Diagnostic count is monotonic with violations
    ///
    /// Adding more violations should increase or maintain diagnostic count.
    ///
    /// # Invariant
    /// For all documents D and violations V:
    ///   count(lint(D + V)) >= count(lint(D))
    #[test]
    fn test_monotonicity_adding_violations(
        base_doc in valid_list_document(),
        extra_short_ids in prop::collection::hash_set(short_id(), 1..10)
    ) {
        let base_diagnostics = lint(&base_doc);

        // Clone document and add short IDs (violations)
        let mut extended_doc = base_doc.clone();

        if let Some(hedl_core::Item::List(list)) = extended_doc.root.values_mut().next() {
            let type_name = list.type_name.clone();
            for id in extra_short_ids {
                list.add_row(hedl_core::Node::new(&type_name, &id, vec![]));
            }
        }

        let extended_diagnostics = lint(&extended_doc);

        // Extended should have >= diagnostics than base
        prop_assert!(
            extended_diagnostics.len() >= base_diagnostics.len(),
            "Adding violations decreased diagnostics: {} -> {}",
            base_diagnostics.len(),
            extended_diagnostics.len()
        );
    }

    /// Property: Diagnostics are sorted by severity (errors first)
    ///
    /// The lint runner should return diagnostics sorted by severity
    /// in descending order (Error > Warning > Hint).
    ///
    /// # Invariant
    /// For all documents D and diagnostics [d1, d2, ..., dn] in lint(D):
    ///   d1.severity >= d2.severity >= ... >= dn.severity
    #[test]
    fn test_diagnostics_sorted_by_severity(doc in mixed_violation_document()) {
        let diagnostics = lint(&doc);

        if diagnostics.len() > 1 {
            for i in 0..diagnostics.len() - 1 {
                prop_assert!(
                    diagnostics[i].severity() >= diagnostics[i + 1].severity(),
                    "Diagnostics not sorted: {:?} before {:?}",
                    diagnostics[i].severity(),
                    diagnostics[i + 1].severity()
                );
            }
        }
    }
}

// ===== Rule-Specific Properties =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Short IDs always trigger id-naming hints
    ///
    /// # Invariant
    /// For all documents D with short IDs:
    ///   exists diagnostic in lint(D) where
    ///     diagnostic.kind = IdNaming AND
    ///     diagnostic.severity = Hint
    #[test]
    fn test_short_ids_always_hint(doc in short_id_list_document()) {
        let diagnostics = lint(&doc);

        let id_naming_hints: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
            .collect();

        // Should have at least one id-naming hint for short IDs
        prop_assert!(
            !id_naming_hints.is_empty(),
            "Short IDs should trigger id-naming hints"
        );

        // All id-naming diagnostics should be Hints (not escalated)
        for diag in &id_naming_hints {
            prop_assert_eq!(
                diag.severity(),
                Severity::Hint,
                "IdNaming diagnostic should be Hint severity"
            );
        }
    }

    /// Property: Good IDs never trigger id-naming hints
    ///
    /// # Invariant
    /// For all documents D with only good IDs:
    ///   not exists diagnostic in lint(D) where diagnostic.kind = IdNaming
    #[test]
    fn test_good_ids_no_hint(doc in valid_list_document()) {
        let diagnostics = lint(&doc);

        let id_naming_hints: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
            .collect();

        // Should have zero id-naming hints for good IDs
        prop_assert!(
            id_naming_hints.is_empty(),
            "Good IDs should not trigger id-naming hints, but got: {:?}",
            id_naming_hints.iter().map(|d| d.message()).collect::<Vec<_>>()
        );
    }

    /// Property: Numeric IDs always trigger id-naming hints
    ///
    /// # Invariant
    /// For all documents D with numeric-only IDs:
    ///   exists diagnostic in lint(D) where
    ///     diagnostic.kind = IdNaming AND
    ///     diagnostic.message contains "numbers"
    #[test]
    fn test_numeric_ids_trigger_hint(doc in numeric_id_list_document()) {
        let diagnostics = lint(&doc);

        let numeric_hints: Vec<_> = diagnostics
            .iter()
            .filter(|d| {
                matches!(d.kind(), DiagnosticKind::IdNaming) &&
                d.message().contains("numbers")
            })
            .collect();

        prop_assert!(
            !numeric_hints.is_empty(),
            "Numeric IDs should trigger id-naming hints"
        );
    }

    /// Property: Unused schemas always trigger warnings
    ///
    /// # Invariant
    /// For all documents D with unused schemas:
    ///   count(diagnostics where kind = UnusedSchema) = count(unused schemas in D)
    #[test]
    fn test_unused_schemas_trigger_warnings(doc in unused_schema_document()) {
        let diagnostics = lint(&doc);

        let unused_warnings: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::UnusedSchema))
            .collect();

        // Count unused schemas in document
        let used_types: std::collections::HashSet<_> = doc
            .root
            .values()
            .filter_map(|item| {
                if let hedl_core::Item::List(list) = item {
                    Some(list.type_name.as_str())
                } else {
                    None
                }
            })
            .collect();

        let unused_count = doc
            .structs
            .keys()
            .filter(|k| !used_types.contains(k.as_str()))
            .count();

        // Should have warning for each unused schema
        prop_assert_eq!(
            unused_warnings.len(),
            unused_count,
            "Should warn for each unused schema"
        );

        // All should be Warning severity
        for diag in &unused_warnings {
            prop_assert_eq!(
                diag.severity(),
                Severity::Warning,
                "UnusedSchema should be Warning severity"
            );
        }
    }

    /// Property: Empty lists always trigger hints
    ///
    /// # Invariant
    /// For all documents D with empty lists:
    ///   count(diagnostics where kind = EmptyList) = count(empty lists in D)
    #[test]
    fn test_empty_lists_trigger_hints(doc in empty_list_document()) {
        let diagnostics = lint(&doc);

        let empty_hints: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
            .collect();

        // Count empty lists in document
        let empty_count = doc
            .root
            .values()
            .filter(|item| matches!(item, hedl_core::Item::List(list) if list.rows.is_empty()))
            .count();

        prop_assert_eq!(
            empty_hints.len(),
            empty_count,
            "Should hint for each empty list"
        );

        // All should be Hint severity
        for diag in &empty_hints {
            prop_assert_eq!(
                diag.severity(),
                Severity::Hint,
                "EmptyList should be Hint severity"
            );
        }
    }

    /// Property: Unqualified KV references always trigger warnings
    ///
    /// # Invariant
    /// For all documents D with unqualified references in KV context:
    ///   exists diagnostic in lint(D) where
    ///     diagnostic.kind = UnqualifiedKvReference AND
    ///     diagnostic.severity = Warning AND
    ///     diagnostic.suggestion is Some
    #[test]
    fn test_unqualified_refs_trigger_warnings(doc in unqualified_ref_document()) {
        let diagnostics = lint(&doc);

        let unqualified_warnings: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
            .collect();

        // Should have at least one warning for unqualified reference
        prop_assert!(
            !unqualified_warnings.is_empty(),
            "Unqualified KV references should trigger warnings"
        );

        // All should be Warning severity with suggestions
        for diag in &unqualified_warnings {
            prop_assert_eq!(
                diag.severity(),
                Severity::Warning,
                "UnqualifiedKvReference should be Warning severity"
            );
            prop_assert!(
                diag.suggestion().is_some(),
                "UnqualifiedKvReference should provide suggestion"
            );
        }
    }

    /// Property: Qualified references never trigger warnings
    ///
    /// # Invariant
    /// For all documents D with only qualified references:
    ///   not exists diagnostic in lint(D) where diagnostic.kind = UnqualifiedKvReference
    #[test]
    fn test_qualified_refs_no_warning(doc in qualified_ref_document()) {
        let diagnostics = lint(&doc);

        let unqualified_warnings: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::UnqualifiedKvReference))
            .collect();

        // Should have zero warnings for qualified references
        prop_assert!(
            unqualified_warnings.is_empty(),
            "Qualified references should not trigger warnings"
        );
    }

    /// Property: Nested violations are detected at any depth
    ///
    /// # Invariant
    /// For all documents D with violations at depth N (where N <= MAX_DEPTH):
    ///   exists diagnostic in lint(D) for the violation
    #[test]
    fn test_nested_violations_detected(doc in nested_violation_document()) {
        let diagnostics = lint(&doc);

        // Should detect the short ID violation even in nested structure
        let id_naming_hints: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::IdNaming))
            .collect();

        prop_assert!(
            !id_naming_hints.is_empty(),
            "Should detect violations in nested structures"
        );
    }

    /// Property: Rule respects enabled/disabled configuration
    ///
    /// # Invariant
    /// For all documents D and rules R where R is disabled:
    ///   not exists diagnostic in lint_with_config(D, disabled=R) where diagnostic.rule_id = R
    #[test]
    fn test_rule_respects_configuration(doc in mixed_violation_document()) {
        // Test each rule can be individually disabled
        for rule_id in &["id-naming", "unused-schema", "empty-list", "unqualified-kv-ref"] {
            let mut config = LintConfig::default();
            config.disable_rule(rule_id);

            let diagnostics = lint_with_config(&doc, config);

            // No diagnostics from disabled rule should appear
            let disabled_rule_diagnostics: Vec<_> = diagnostics
                .iter()
                .filter(|d| d.rule_id() == *rule_id)
                .collect();

            prop_assert!(
                disabled_rule_diagnostics.is_empty(),
                "Disabled rule '{}' still produced diagnostics",
                rule_id
            );
        }
    }
}

// ===== Integration Properties =====

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property: Multiple rules can run simultaneously without conflicts
    ///
    /// # Invariant
    /// For all documents D:
    ///   lint(D) = union of all individual rule results
    ///   (no duplicate diagnostics, no missing diagnostics)
    #[test]
    fn test_multiple_rules_no_conflicts(doc in mixed_violation_document()) {
        let all_diagnostics = lint(&doc);

        // Count diagnostics by kind
        let mut kind_counts = std::collections::HashMap::new();
        for diag in &all_diagnostics {
            *kind_counts.entry(format!("{:?}", diag.kind())).or_insert(0) += 1;
        }

        // Verify we have diagnostics from multiple rules
        // (mixed_violation_document should trigger multiple rules)
        prop_assert!(
            kind_counts.len() >= 2,
            "Should have diagnostics from multiple rules, got: {:?}",
            kind_counts.keys().collect::<Vec<_>>()
        );

        // Check for no duplicate diagnostics (same kind, same message)
        let mut seen = std::collections::HashSet::new();
        for diag in &all_diagnostics {
            let key = (format!("{:?}", diag.kind()), diag.message().to_string());
            prop_assert!(
                seen.insert(key.clone()),
                "Duplicate diagnostic found: {:?}",
                key
            );
        }
    }

    /// Property: Diagnostic limit is enforced
    ///
    /// # Invariant
    /// For all documents D and limits L:
    ///   count(lint_with_config(D, max_diagnostics=L)) <= L + 1
    ///   (The +1 accounts for the limit exceeded warning itself)
    #[test]
    fn test_diagnostic_limit_enforced(doc in mixed_violation_document()) {
        let limit = 5;
        let config = LintConfig {
            max_diagnostics: limit,
            ..Default::default()
        };

        let diagnostics = lint_with_config(&doc, config);

        // Allow limit + 1 because the limit exceeded warning is added after the limit
        prop_assert!(
            diagnostics.len() <= limit + 1,
            "Diagnostic count {} exceeds limit + 1 ({} + 1)",
            diagnostics.len(),
            limit
        );
    }

    /// Property: Escalation works across all rule types
    ///
    /// # Invariant
    /// For all documents D and rules R where error=true:
    ///   For all diagnostics d where d.rule_id = R:
    ///     d.severity = Error
    #[test]
    fn test_escalation_all_rules(doc in mixed_violation_document()) {
        for rule_id in &["id-naming", "unused-schema", "empty-list", "unqualified-kv-ref"] {
            let mut config = LintConfig::default();
            config.set_rule_error(rule_id);

            let diagnostics = lint_with_config(&doc, config);

            // All diagnostics from this rule should be Error
            for diag in &diagnostics {
                if diag.rule_id() == *rule_id {
                    prop_assert_eq!(
                        diag.severity(),
                        Severity::Error,
                        "Rule '{}' diagnostic not escalated to Error",
                        rule_id
                    );
                }
            }
        }
    }

    /// Property: Severity filtering works with escalation
    ///
    /// # Invariant
    /// For all documents D, rules R, and severity S:
    ///   If rule R is escalated to Error and min_severity = Error:
    ///     lint_with_config(D, error=R, min_severity=Error) includes diagnostics from R
    #[test]
    fn test_escalation_with_severity_filter(doc in empty_list_document()) {
        let mut config = LintConfig::default();
        config.set_rule_error("empty-list"); // Escalate Hint -> Error
        config.min_severity = Severity::Error; // Filter to Error only

        let diagnostics = lint_with_config(&doc, config);

        // Should still have empty-list diagnostics (escalated from Hint to Error)
        let empty_list_diags: Vec<_> = diagnostics
            .iter()
            .filter(|d| matches!(d.kind(), DiagnosticKind::EmptyList))
            .collect();

        if !doc.root.values().filter(|item| matches!(item, hedl_core::Item::List(list) if list.rows.is_empty())).collect::<Vec<_>>().is_empty() {
            prop_assert!(
                !empty_list_diags.is_empty(),
                "Escalated hints should pass Error severity filter"
            );
        }
    }

    /// Property: Performance is bounded (no exponential behavior)
    ///
    /// # Invariant
    /// For all documents D:
    ///   lint_time(D) is roughly linear in size(D)
    ///
    /// Note: This is a weak test - we just verify it completes in reasonable time.
    /// Real performance testing is done in benchmark suite.
    #[test]
    fn test_performance_bounded(doc in well_formed_document()) {
        use std::time::Instant;

        let start = Instant::now();
        let _diagnostics = lint(&doc);
        let duration = start.elapsed();

        // Should complete in under 1 second for small generated documents
        prop_assert!(
            duration.as_secs() < 1,
            "Lint took too long: {:?}",
            duration
        );
    }
}

#[cfg(test)]
mod test_generators {
    use super::*;

    /// Verify generators are actually exercising different code paths
    #[test]
    fn test_generator_coverage() {
        proptest!(|(
            _simple in simple_document(),
            _multi in multi_field_document(),
            _valid_list in valid_list_document(),
            short_list in short_id_list_document(),
            numeric_list in numeric_id_list_document(),
            unused in unused_schema_document(),
            empty in empty_list_document(),
            unqualified in unqualified_ref_document(),
            _qualified in qualified_ref_document(),
            well_formed in well_formed_document(),
            mixed in mixed_violation_document()
        )| {
            // Each generator should produce different patterns of diagnostics
            let d4 = lint(&short_list);
            let d5 = lint(&numeric_list);
            let d6 = lint(&unused);
            let d7 = lint(&empty);
            let d8 = lint(&unqualified);
            let d10 = lint(&well_formed);
            let d11 = lint(&mixed);

            // Well-formed should have no diagnostics
            prop_assert!(d10.is_empty(), "Well-formed document should have no diagnostics");

            // Documents designed to trigger violations should have diagnostics
            prop_assert!(!d4.is_empty() || !d5.is_empty() || !d6.is_empty() || !d7.is_empty() || !d8.is_empty() || !d11.is_empty(),
                "At least one violation document should produce diagnostics");
        });
    }
}
