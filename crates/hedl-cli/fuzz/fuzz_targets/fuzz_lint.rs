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


#![no_main]

//! Fuzz target for HEDL lint command.
//!
//! This fuzzer tests the linting functionality for crashes, panics, and memory
//! safety issues. It focuses on:
//! - Malformed HEDL input
//! - Edge cases in lint rules
//! - Diagnostic generation with unusual inputs
//! - JSON serialization of diagnostics
//!
//! The fuzzer ensures that linting operations never panic and handle all errors
//! gracefully.

use libfuzzer_sys::fuzz_target;
use hedl_core::parse;
use hedl_lint::{lint_with_config, LintConfig, Severity};

fuzz_target!(|data: &[u8]| {
    // Limit input size to prevent timeout (100 KB max for fuzzing)
    if data.len() > 100_000 {
        return;
    }

    if let Ok(text) = std::str::from_utf8(data) {
        test_linting(text);
    }
});

fn test_linting(text: &str) {
    // Parse HEDL (may fail, which is expected for malformed input)
    if let Ok(doc) = parse(text.as_bytes()) {
        // Test default linting configuration
        let config = LintConfig::default();
        let diagnostics = lint_with_config(&doc, config);

        // Test diagnostic processing (should never panic)
        for diag in &diagnostics {
            // Access all diagnostic fields
            let _ = diag.severity();
            let _ = diag.rule_id();
            let _ = diag.message();
            let _ = diag.line();
            let _ = diag.suggestion();

            // Test severity classification
            match diag.severity() {
                Severity::Error => {
                    // Errors should have meaningful messages
                    assert!(!diag.message().is_empty());
                }
                Severity::Warning => {
                    // Warnings should have meaningful messages
                    assert!(!diag.message().is_empty());
                }
                Severity::Hint => {
                    // Hints should have meaningful messages
                    assert!(!diag.message().is_empty());
                }
            }
        }

        // Test JSON serialization of diagnostics (for --format json)
        test_json_serialization(&diagnostics);

        // Test counting diagnostics by severity
        let has_errors = diagnostics.iter().any(|d| d.severity() == Severity::Error);
        let has_warnings = diagnostics.iter().any(|d| d.severity() == Severity::Warning);
        let has_hints = diagnostics.iter().any(|d| d.severity() == Severity::Hint);

        // Verify counts are consistent
        let error_count = diagnostics.iter().filter(|d| d.severity() == Severity::Error).count();
        let warning_count = diagnostics.iter().filter(|d| d.severity() == Severity::Warning).count();
        let hint_count = diagnostics.iter().filter(|d| d.severity() == Severity::Hint).count();

        assert_eq!(has_errors, error_count > 0);
        assert_eq!(has_warnings, warning_count > 0);
        assert_eq!(has_hints, hint_count > 0);
        assert_eq!(error_count + warning_count + hint_count, diagnostics.len());

        // Test lint configuration variations
        test_custom_config(&doc);
    }
}

fn test_json_serialization(diagnostics: &[hedl_lint::Diagnostic]) {
    // Create JSON representation (mirrors lint.rs)
    let json = serde_json::json!({
        "file": "fuzz_input.hedl",
        "diagnostics": diagnostics.iter().map(|d| {
            serde_json::json!({
                "severity": format!("{:?}", d.severity()),
                "rule": d.rule_id(),
                "message": d.message(),
                "line": d.line(),
                "suggestion": d.suggestion()
            })
        }).collect::<Vec<_>>()
    });

    // Serialize to string (should never panic)
    if let Ok(output) = serde_json::to_string_pretty(&json) {
        // Verify output is valid
        assert!(!output.is_empty());

        // Should be parseable back
        let _ = serde_json::from_str::<serde_json::Value>(&output);
    }
}

fn test_custom_config(doc: &hedl_core::Document) {
    // Test with various configurations to ensure robustness
    // This mirrors the LintConfig variations that might be used

    // Default config
    let config1 = LintConfig::default();
    let _ = lint_with_config(doc, config1);

    // Additional configurations can be added here as LintConfig evolves
    // For now, we just test the default to ensure no panics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        test_linting("");
    }

    #[test]
    fn test_minimal_valid_hedl() {
        test_linting("HEDL 1.0\n");
    }

    #[test]
    fn test_malformed_input() {
        test_linting("not valid hedl at all!!!");
    }

    #[test]
    fn test_deeply_nested() {
        let mut input = String::from("HEDL 1.0\n");
        for i in 0..100 {
            input.push_str(&format!("obj{} {{\n", i));
        }
        for _ in 0..100 {
            input.push_str("}\n");
        }
        test_linting(&input);
    }
}
