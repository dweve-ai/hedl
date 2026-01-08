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

//! Generator validation utilities.
//!
//! Validates that generated HEDL documents meet expected criteria for
//! structure, complexity, and correctness.

use super::config::ComplexityLevel;
use crate::Result;

/// Validates that generated HEDL can be parsed successfully.
///
/// # Arguments
///
/// * `hedl` - The HEDL string to validate
///
/// # Returns
///
/// Result indicating success or parse error.
pub fn validate_generated(hedl: &str) -> Result<()> {
    hedl_core::parse(hedl.as_bytes())
        .map(|_| ())
        .map_err(|e| crate::BenchError::ParseError(e.to_string()))
}

/// Verifies that a document matches expected complexity level.
///
/// # Arguments
///
/// * `hedl` - The HEDL string to check
/// * `expected` - Expected complexity level
///
/// # Returns
///
/// true if complexity matches expectation.
pub fn verify_complexity(hedl: &str, expected: ComplexityLevel) -> bool {
    match expected {
        ComplexityLevel::Flat => {
            // Should not have %NEST or references
            !hedl.contains("%NEST") && !hedl.contains("@")
                || (hedl.contains("@") && hedl.contains(": @")) // Struct refs OK
        }
        ComplexityLevel::ModerateNesting => {
            // Should have some %NEST but not too deep
            let nest_count = hedl.matches("%NEST").count();
            nest_count > 0 && nest_count < 10
        }
        ComplexityLevel::DittoHeavy => {
            // Should have many ditto markers
            hedl.matches('^').count() > 5
        }
        ComplexityLevel::ReferenceHeavy => {
            // Should have many @Type:id references
            hedl.matches("@").count() > 10
        }
        ComplexityLevel::DeepHierarchy => {
            // Should have deep nesting
            let nest_count = hedl.matches("%NEST").count();
            nest_count >= 4
        }
    }
}

/// Counts the number of entities in a HEDL document.
///
/// # Arguments
///
/// * `hedl` - The HEDL string
///
/// # Returns
///
/// Approximate entity count.
pub fn count_entities(hedl: &str) -> usize {
    hedl.lines()
        .filter(|line| line.trim().starts_with('|'))
        .count()
}

/// Verifies that document has expected entity count.
///
/// # Arguments
///
/// * `hedl` - The HEDL string
/// * `expected` - Expected count
/// * `tolerance` - Acceptable variance (e.g., 0.1 = 10%)
///
/// # Returns
///
/// true if count is within tolerance.
pub fn verify_entity_count(hedl: &str, expected: usize, tolerance: f32) -> bool {
    let actual = count_entities(hedl);
    let lower = (expected as f32 * (1.0 - tolerance)) as usize;
    let upper = (expected as f32 * (1.0 + tolerance)) as usize;
    actual >= lower && actual <= upper
}

/// Validates that document structure is well-formed.
///
/// # Arguments
///
/// * `hedl` - The HEDL string
///
/// # Returns
///
/// Result indicating validation success.
pub fn validate_structure(hedl: &str) -> Result<()> {
    // Check for version header
    if !hedl.contains("%VERSION:") {
        return Err(crate::BenchError::ValidationError(
            "Missing %VERSION header".to_string(),
        ));
    }

    // Check for struct definitions
    if !hedl.contains("%STRUCT:") {
        return Err(crate::BenchError::ValidationError(
            "Missing %STRUCT definition".to_string(),
        ));
    }

    // Parse to verify syntax
    validate_generated(hedl)
}

/// Validates that document can round-trip through JSON.
///
/// # Arguments
///
/// * `hedl` - The HEDL string
///
/// # Returns
///
/// Result indicating round-trip success.
pub fn validate_roundtrip(hedl: &str) -> Result<()> {
    let doc = hedl_core::parse(hedl.as_bytes())
        .map_err(|e| crate::BenchError::ParseError(e.to_string()))?;

    let json = hedl_json::to_json(&doc, &hedl_json::ToJsonConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))?;

    let _doc2 = hedl_json::from_json(&json, &hedl_json::FromJsonConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))?;

    Ok(())
}

/// Estimates complexity score for a document.
///
/// Returns a numeric score based on nesting, references, and structure.
///
/// # Arguments
///
/// * `hedl` - The HEDL string
///
/// # Returns
///
/// Complexity score (higher = more complex).
pub fn estimate_complexity_score(hedl: &str) -> usize {
    let nest_count = hedl.matches("%NEST").count() * 10;
    let ref_count = hedl.matches("@").count();
    let ditto_count = hedl.matches('^').count();
    let line_count = hedl.lines().count();

    nest_count + ref_count + ditto_count + (line_count / 10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_users;

    #[test]
    fn test_validate_generated() {
        let hedl = generate_users(10);
        assert!(validate_generated(&hedl).is_ok());
    }

    #[test]
    fn test_count_entities() {
        let hedl = generate_users(10);
        let count = count_entities(&hedl);
        assert_eq!(count, 10);
    }

    #[test]
    fn test_verify_entity_count() {
        let hedl = generate_users(10);
        assert!(verify_entity_count(&hedl, 10, 0.1));
        assert!(!verify_entity_count(&hedl, 100, 0.1));
    }

    #[test]
    fn test_validate_structure() {
        let hedl = generate_users(10);
        assert!(validate_structure(&hedl).is_ok());

        let bad_hedl = "no version header";
        assert!(validate_structure(bad_hedl).is_err());
    }

    #[test]
    fn test_validate_roundtrip() {
        let hedl = generate_users(5);
        assert!(validate_roundtrip(&hedl).is_ok());
    }

    #[test]
    fn test_estimate_complexity_score() {
        let simple = generate_users(10);
        let score = estimate_complexity_score(&simple);
        assert!(score > 0);
    }
}
