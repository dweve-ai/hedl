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

//! Fixture file loading utilities.
//!
//! Provides functions for loading test fixtures from the fixtures directory,
//! with fallback to generated data.

use crate::{generate_users, sizes, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Loads a fixture file by name.
///
/// Attempts to load from `fixtures/{name}.hedl`. If not found, falls back
/// to generating appropriate data based on the name.
///
/// # Arguments
///
/// * `name` - Fixture name (without .hedl extension)
///
/// # Returns
///
/// Result containing fixture content as string.
pub fn load_fixture(name: &str) -> Result<String> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = PathBuf::from(manifest_dir)
        .join("fixtures")
        .join(format!("{}.hedl", name));

    if path.exists() {
        std::fs::read_to_string(&path).map_err(|e| crate::BenchError::IoError(e.to_string()))
    } else {
        // Fallback to generated fixture
        Ok(generate_fallback_fixture(name))
    }
}

/// Loads all fixtures from the fixtures directory.
///
/// # Returns
///
/// HashMap mapping fixture names to their content.
pub fn load_all_fixtures() -> HashMap<String, String> {
    let mut fixtures = HashMap::new();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fixtures_dir = PathBuf::from(manifest_dir).join("fixtures");

    if let Ok(entries) = std::fs::read_dir(&fixtures_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hedl") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        fixtures.insert(name.to_string(), content);
                    }
                }
            }
        }
    }

    // Add fallbacks for standard sizes if not present
    for &(name, size) in &[
        ("small", sizes::SMALL),
        ("medium", sizes::MEDIUM),
        ("large", sizes::LARGE),
    ] {
        fixtures
            .entry(name.to_string())
            .or_insert_with(|| generate_users(size));
    }

    fixtures
}

/// Generates fallback fixture data based on name.
///
/// # Arguments
///
/// * `name` - Fixture name
///
/// # Returns
///
/// Generated HEDL string.
fn generate_fallback_fixture(name: &str) -> String {
    match name {
        "small" => generate_users(sizes::SMALL),
        "medium" => generate_users(sizes::MEDIUM),
        "large" => generate_users(sizes::LARGE),
        "stress" => generate_users(sizes::STRESS),
        "extreme" => generate_users(sizes::EXTREME),
        _ => {
            // Try to parse as count
            if let Ok(count) = name.parse::<usize>() {
                generate_users(count)
            } else {
                generate_users(sizes::MEDIUM)
            }
        }
    }
}

/// Lists available fixtures.
///
/// # Returns
///
/// Vector of fixture names.
pub fn list_fixtures() -> Vec<String> {
    let mut names = Vec::new();
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fixtures_dir = PathBuf::from(manifest_dir).join("fixtures");

    if let Ok(entries) = std::fs::read_dir(&fixtures_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hedl") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    names.push(name.to_string());
                }
            }
        }
    }

    // Add standard sizes
    names.extend(["small", "medium", "large"].iter().map(|s| s.to_string()));
    names.sort();
    names.dedup();

    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_fixture_fallback() {
        let content = load_fixture("small").unwrap();
        assert!(content.contains("%VERSION: 1.0"));
    }

    #[test]
    fn test_generate_fallback_fixture() {
        let small = generate_fallback_fixture("small");
        assert!(small.contains("%STRUCT: User"));

        let numeric = generate_fallback_fixture("50");
        assert!(numeric.contains("%VERSION: 1.0"));
    }

    #[test]
    fn test_load_all_fixtures() {
        let fixtures = load_all_fixtures();
        assert!(fixtures.contains_key("small"));
        assert!(fixtures.contains_key("medium"));
    }

    #[test]
    fn test_list_fixtures() {
        let names = list_fixtures();
        assert!(!names.is_empty());
        assert!(names.contains(&"small".to_string()));
    }
}
