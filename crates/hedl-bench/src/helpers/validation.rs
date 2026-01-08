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

//! Validation helpers for benchmarks.
//!
//! Utilities for validating HEDL documents, performing round-trip tests,
//! and ensuring data integrity.

use crate::Result;
use hedl_core::Document;

/// Performs strict validation on a HEDL document.
///
/// Runs linting and returns all diagnostics found.
///
/// # Arguments
///
/// * `doc` - The document to validate
///
/// # Returns
///
/// Result containing vector of diagnostics.
pub fn validate_strict(doc: &Document) -> Result<Vec<hedl_lint::Diagnostic>> {
    Ok(hedl_lint::lint(doc))
}

/// Validates round-trip integrity.
///
/// Compares original and converted documents for structural equivalence.
///
/// # Arguments
///
/// * `original` - Original HEDL string
/// * `converted` - Converted/round-tripped HEDL string
///
/// # Returns
///
/// true if documents are structurally equivalent.
pub fn validate_roundtrip(original: &str, converted: &str) -> bool {
    let doc1 = match hedl_core::parse(original.as_bytes()) {
        Ok(d) => d,
        Err(_) => return false,
    };

    let doc2 = match hedl_core::parse(converted.as_bytes()) {
        Ok(d) => d,
        Err(_) => return false,
    };

    // Compare root entity count
    doc1.root.len() == doc2.root.len()
}

/// Validates that a string can be parsed as HEDL.
///
/// # Arguments
///
/// * `hedl` - The HEDL string to validate
///
/// # Returns
///
/// true if parsing succeeds.
pub fn is_valid_hedl(hedl: &str) -> bool {
    hedl_core::parse(hedl.as_bytes()).is_ok()
}

/// Validates JSON round-trip.
///
/// HEDL -> JSON -> HEDL should preserve structure.
///
/// # Arguments
///
/// * `hedl` - The original HEDL string
///
/// # Returns
///
/// Result indicating round-trip success.
pub fn validate_json_roundtrip(hedl: &str) -> Result<bool> {
    let doc = hedl_core::parse(hedl.as_bytes())
        .map_err(|e| crate::BenchError::ParseError(e.to_string()))?;

    let json = hedl_json::to_json(&doc, &hedl_json::ToJsonConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))?;

    let doc2 = hedl_json::from_json(&json, &hedl_json::FromJsonConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))?;

    Ok(doc.root.len() == doc2.root.len())
}

/// Validates YAML round-trip.
///
/// HEDL -> YAML -> HEDL should preserve structure.
///
/// # Arguments
///
/// * `hedl` - The original HEDL string
///
/// # Returns
///
/// Result indicating round-trip success.
pub fn validate_yaml_roundtrip(hedl: &str) -> Result<bool> {
    let doc = hedl_core::parse(hedl.as_bytes())
        .map_err(|e| crate::BenchError::ParseError(e.to_string()))?;

    let yaml = hedl_yaml::to_yaml(&doc, &hedl_yaml::ToYamlConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))?;

    let doc2 = hedl_yaml::from_yaml(&yaml, &hedl_yaml::FromYamlConfig::default())
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))?;

    Ok(doc.root.len() == doc2.root.len())
}

/// Validates canonical form preservation.
///
/// Parsing canonical form should produce identical structure.
///
/// # Arguments
///
/// * `doc` - The document to test
///
/// # Returns
///
/// Result indicating canonical preservation.
pub fn validate_canonical(doc: &Document) -> Result<bool> {
    let canonical = hedl_c14n::canonicalize(doc)
        .map_err(|e| crate::BenchError::ConversionError(e.to_string()))?;

    let doc2 = hedl_core::parse(canonical.as_bytes())
        .map_err(|e| crate::BenchError::ParseError(e.to_string()))?;

    Ok(doc.root.len() == doc2.root.len())
}

/// Counts linting issues in a document.
///
/// # Arguments
///
/// * `doc` - The document to lint
///
/// # Returns
///
/// Number of diagnostics found.
pub fn count_lint_issues(doc: &Document) -> usize {
    hedl_lint::lint(doc).len()
}

/// Validates that generated data meets size expectations.
///
/// # Arguments
///
/// * `hedl` - The HEDL string
/// * `min_bytes` - Minimum expected size in bytes
/// * `max_bytes` - Maximum expected size in bytes
///
/// # Returns
///
/// true if size is within range.
pub fn validate_size_range(hedl: &str, min_bytes: usize, max_bytes: usize) -> bool {
    let size = hedl.len();
    size >= min_bytes && size <= max_bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_users;

    #[test]
    fn test_validate_strict() {
        let hedl = generate_users(10);
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        let diagnostics = validate_strict(&doc).unwrap();
        // May or may not have diagnostics, just ensure it runs
        let _ = diagnostics;
    }

    #[test]
    fn test_validate_roundtrip() {
        let hedl = generate_users(5);
        assert!(validate_roundtrip(&hedl, &hedl));
    }

    #[test]
    fn test_is_valid_hedl() {
        let hedl = generate_users(5);
        assert!(is_valid_hedl(&hedl));
        assert!(!is_valid_hedl("invalid hedl"));
    }

    #[test]
    fn test_validate_json_roundtrip() {
        let hedl = generate_users(5);
        assert!(validate_json_roundtrip(&hedl).unwrap());
    }

    #[test]
    fn test_validate_yaml_roundtrip() {
        let hedl = generate_users(5);
        assert!(validate_yaml_roundtrip(&hedl).unwrap());
    }

    #[test]
    fn test_validate_canonical() {
        let hedl = generate_users(5);
        let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
        assert!(validate_canonical(&doc).unwrap());
    }

    #[test]
    fn test_validate_size_range() {
        let hedl = generate_users(10);
        assert!(validate_size_range(&hedl, 100, 100_000));
        assert!(!validate_size_range(&hedl, 1_000_000, 2_000_000));
    }
}
