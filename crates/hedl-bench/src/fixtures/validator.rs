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

//! Fixture validation utilities.
//!
//! Validates that fixture files contain valid HEDL and meet quality standards.

use crate::Result;

/// Validates fixture content.
///
/// Checks that the fixture:
/// - Can be parsed as valid HEDL
/// - Contains required headers
/// - Has reasonable structure
///
/// # Arguments
///
/// * `content` - Fixture content to validate
///
/// # Returns
///
/// Result indicating validation success.
pub fn validate_fixture(content: &str) -> Result<()> {
    // Must not be empty
    if content.is_empty() {
        return Err(crate::BenchError::ValidationError(
            "Fixture is empty".to_string(),
        ));
    }

    // Must contain version header
    if !content.contains("%VERSION:") {
        return Err(crate::BenchError::ValidationError(
            "Fixture missing %VERSION header".to_string(),
        ));
    }

    // Must be valid HEDL
    hedl_core::parse(content.as_bytes())
        .map_err(|e| crate::BenchError::ParseError(e.to_string()))?;

    Ok(())
}

/// Validates all fixtures in a collection.
///
/// # Arguments
///
/// * `fixtures` - Map of fixture names to content
///
/// # Returns
///
/// Result with vector of validation errors (empty if all valid).
pub fn validate_all_fixtures(
    fixtures: &std::collections::HashMap<String, String>,
) -> Vec<(String, String)> {
    let mut errors = Vec::new();

    for (name, content) in fixtures {
        if let Err(e) = validate_fixture(content) {
            errors.push((name.clone(), e.to_string()));
        }
    }

    errors
}

/// Checks if fixture meets minimum size requirement.
///
/// # Arguments
///
/// * `content` - Fixture content
/// * `min_bytes` - Minimum size in bytes
///
/// # Returns
///
/// true if fixture meets minimum size.
pub fn meets_min_size(content: &str, min_bytes: usize) -> bool {
    content.len() >= min_bytes
}

/// Checks if fixture is within size limits.
///
/// # Arguments
///
/// * `content` - Fixture content
/// * `max_bytes` - Maximum size in bytes
///
/// # Returns
///
/// true if fixture is within size limit.
pub fn within_max_size(content: &str, max_bytes: usize) -> bool {
    content.len() <= max_bytes
}

/// Validates fixture has expected entity count.
///
/// # Arguments
///
/// * `content` - Fixture content
/// * `expected_count` - Expected number of entities
/// * `tolerance` - Acceptable variance (0.0-1.0)
///
/// # Returns
///
/// true if entity count is within tolerance.
pub fn has_expected_entity_count(content: &str, expected_count: usize, tolerance: f32) -> bool {
    let entity_count = content
        .lines()
        .filter(|line| line.trim().starts_with('|'))
        .count();

    let min = (expected_count as f32 * (1.0 - tolerance)) as usize;
    let max = (expected_count as f32 * (1.0 + tolerance)) as usize;

    entity_count >= min && entity_count <= max
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_users;

    #[test]
    fn test_validate_fixture() {
        let valid = generate_users(10);
        assert!(validate_fixture(&valid).is_ok());

        let invalid = "not valid hedl";
        assert!(validate_fixture(invalid).is_err());

        let empty = "";
        assert!(validate_fixture(empty).is_err());
    }

    #[test]
    fn test_meets_min_size() {
        let content = "hello world";
        assert!(meets_min_size(content, 5));
        assert!(!meets_min_size(content, 100));
    }

    #[test]
    fn test_within_max_size() {
        let content = "hello world";
        assert!(within_max_size(content, 100));
        assert!(!within_max_size(content, 5));
    }

    #[test]
    fn test_has_expected_entity_count() {
        let hedl = generate_users(10);
        assert!(has_expected_entity_count(&hedl, 10, 0.1));
        assert!(!has_expected_entity_count(&hedl, 100, 0.1));
    }

    #[test]
    fn test_validate_all_fixtures() {
        let mut fixtures = std::collections::HashMap::new();
        fixtures.insert("valid".to_string(), generate_users(5));
        fixtures.insert("invalid".to_string(), "bad hedl".to_string());

        let errors = validate_all_fixtures(&fixtures);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "invalid");
    }
}
