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

//! Round-trip tests for all format converters.
//!
//! These tests verify that documents survive conversion through various formats:
//! - HEDL -> JSON -> HEDL
//! - HEDL -> YAML -> HEDL
//! - HEDL -> XML -> HEDL
//! - Chained: HEDL -> JSON -> HEDL -> YAML -> HEDL -> XML -> HEDL

use hedl_c14n::canonicalize;
use hedl_core::{Document, Item};
use hedl_json::{from_json, to_json, FromJsonConfig, ToJsonConfig};
use hedl_test::fixtures;
use hedl_xml::{from_xml, to_xml, FromXmlConfig, ToXmlConfig};
use hedl_yaml::{from_yaml, to_yaml, FromYamlConfig, ToYamlConfig};
use std::fs;
use std::path::Path;

/// Verify the JSON fixture produces output structurally similar to hand-crafted HEDL.
///
/// Note: The comparison checks structural equivalence rather than exact string match
/// because JSON→HEDL produces alphabetically ordered sections while hand-crafted
/// uses semantic ordering. The column ordering within each struct matches when
/// using __hedl_schema hints in the JSON file.
#[test]
fn blog_json_to_hedl_matches_handcrafted_structure() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let json_path = fixtures_dir.join("blog.json");
    let handcrafted_path = fixtures_dir.join("blog.hedl");

    let json_str = fs::read_to_string(&json_path).expect("Failed to read blog.json");
    let from_json_doc = from_json(&json_str, &FromJsonConfig::default()).unwrap();
    let json_hedl = canonicalize(&from_json_doc).unwrap();

    let handcrafted =
        fs::read_to_string(&handcrafted_path).expect("Failed to read hand-crafted blog.hedl");

    // Check that we have the same STRUCT declarations (column schemas match)
    // Strip counts from STRUCT lines for comparison (e.g., "%STRUCT: Type (5):" -> "%STRUCT: Type:")
    fn strip_count(s: &str) -> String {
        // Match pattern like " (N):" and replace with ":"
        let re = regex::Regex::new(r" \(\d+\):").unwrap();
        re.replace(s, ":").to_string()
    }

    let json_structs: Vec<String> = json_hedl
        .lines()
        .filter(|l| l.starts_with("%STRUCT:"))
        .map(strip_count)
        .collect();
    let hand_structs: Vec<String> = handcrafted
        .lines()
        .filter(|l| l.starts_with("%STRUCT:"))
        .map(strip_count)
        .collect();

    // Sort for comparison since order differs (alphabetical vs semantic)
    let mut json_sorted = json_structs.clone();
    let mut hand_sorted = hand_structs.clone();
    json_sorted.sort();
    hand_sorted.sort();

    assert_eq!(
        json_sorted, hand_sorted,
        "STRUCT declarations don't match between JSON->HEDL and hand-crafted"
    );

    // Check same number of data rows (lines starting with "  |")
    let json_rows = json_hedl
        .lines()
        .filter(|l| l.trim().starts_with('|'))
        .count();
    let hand_rows = handcrafted
        .lines()
        .filter(|l| l.trim().starts_with('|'))
        .count();

    assert_eq!(
        json_rows, hand_rows,
        "Row counts differ: JSON->HEDL has {} rows, hand-crafted has {}",
        json_rows, hand_rows
    );
}

/// Test JSON round-trip for blog fixture.
#[test]
fn blog_json_roundtrip() {
    let original = fixtures::blog();
    let json = to_json(&original, &ToJsonConfig::default()).unwrap();
    let restored = from_json(&json, &FromJsonConfig::default()).unwrap();

    assert_lists_match(&original, &restored, "users", "JSON");
    assert_lists_match(&original, &restored, "posts", "JSON");
    assert_lists_match(&original, &restored, "comments", "JSON");
    assert_lists_match(&original, &restored, "categories", "JSON");
    assert_lists_match(&original, &restored, "tags", "JSON");
    assert_lists_match(&original, &restored, "reactions", "JSON");
    assert_lists_match(&original, &restored, "post_tags", "JSON");
    assert_lists_match(&original, &restored, "followers", "JSON");
}

/// Test YAML round-trip for blog fixture.
#[test]
fn blog_yaml_roundtrip() {
    let original = fixtures::blog();
    let yaml = to_yaml(&original, &ToYamlConfig::default()).unwrap();
    let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    assert_lists_match(&original, &restored, "users", "YAML");
    assert_lists_match(&original, &restored, "posts", "YAML");
    assert_lists_match(&original, &restored, "comments", "YAML");
    assert_lists_match(&original, &restored, "categories", "YAML");
    assert_lists_match(&original, &restored, "tags", "YAML");
    assert_lists_match(&original, &restored, "reactions", "YAML");
    assert_lists_match(&original, &restored, "post_tags", "YAML");
    assert_lists_match(&original, &restored, "followers", "YAML");
}

/// Test XML round-trip for blog fixture.
#[test]
fn blog_xml_roundtrip() {
    let original = fixtures::blog();
    let xml = to_xml(&original, &ToXmlConfig::default()).unwrap();
    let xml_config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let restored = from_xml(&xml, &xml_config).unwrap();

    assert_lists_match(&original, &restored, "users", "XML");
    assert_lists_match(&original, &restored, "posts", "XML");
    assert_lists_match(&original, &restored, "comments", "XML");
    assert_lists_match(&original, &restored, "categories", "XML");
    assert_lists_match(&original, &restored, "tags", "XML");
    assert_lists_match(&original, &restored, "reactions", "XML");
    assert_lists_match(&original, &restored, "post_tags", "XML");
    assert_lists_match(&original, &restored, "followers", "XML");
}

/// Test chained round-trip: JSON -> YAML -> XML -> back.
#[test]
fn blog_chained_roundtrip() {
    let original = fixtures::blog();

    // HEDL -> JSON -> HEDL
    let json = to_json(&original, &ToJsonConfig::default()).unwrap();
    let from_json_doc = from_json(&json, &FromJsonConfig::default()).unwrap();

    // -> YAML -> HEDL
    let yaml = to_yaml(&from_json_doc, &ToYamlConfig::default()).unwrap();
    let from_yaml_doc = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

    // -> XML -> HEDL
    let xml = to_xml(&from_yaml_doc, &ToXmlConfig::default()).unwrap();
    let xml_config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };
    let final_doc = from_xml(&xml, &xml_config).unwrap();

    // Verify all lists survived the chain
    assert_lists_match(&original, &final_doc, "users", "Chained");
    assert_lists_match(&original, &final_doc, "posts", "Chained");
    assert_lists_match(&original, &final_doc, "comments", "Chained");
    assert_lists_match(&original, &final_doc, "categories", "Chained");
    assert_lists_match(&original, &final_doc, "tags", "Chained");
    assert_lists_match(&original, &final_doc, "reactions", "Chained");
    assert_lists_match(&original, &final_doc, "post_tags", "Chained");
    assert_lists_match(&original, &final_doc, "followers", "Chained");
}

/// Test all fixtures round-trip through JSON.
#[test]
fn all_fixtures_json_roundtrip() {
    for (name, fixture_fn) in fixtures::all() {
        let original = fixture_fn();
        let json = to_json(&original, &ToJsonConfig::default()).unwrap();
        let restored = from_json(&json, &FromJsonConfig::default()).unwrap();

        // Check same number of root items
        assert_eq!(
            original.root.len(),
            restored.root.len(),
            "Fixture '{}' lost root items in JSON round-trip",
            name
        );

        // Check lists have same row counts
        for (key, item) in &original.root {
            if let Item::List(list) = item {
                if let Some(Item::List(restored_list)) = restored.root.get(key) {
                    assert_eq!(
                        list.rows.len(),
                        restored_list.rows.len(),
                        "Fixture '{}' list '{}' has wrong row count after JSON round-trip",
                        name,
                        key
                    );
                }
            }
        }
    }
}

/// Test all fixtures round-trip through YAML.
#[test]
fn all_fixtures_yaml_roundtrip() {
    for (name, fixture_fn) in fixtures::all() {
        let original = fixture_fn();
        let yaml = to_yaml(&original, &ToYamlConfig::default()).unwrap();
        let restored = from_yaml(&yaml, &FromYamlConfig::default()).unwrap();

        // Check same number of root items
        assert_eq!(
            original.root.len(),
            restored.root.len(),
            "Fixture '{}' lost root items in YAML round-trip",
            name
        );

        // Check lists have same row counts
        for (key, item) in &original.root {
            if let Item::List(list) = item {
                if let Some(Item::List(restored_list)) = restored.root.get(key) {
                    assert_eq!(
                        list.rows.len(),
                        restored_list.rows.len(),
                        "Fixture '{}' list '{}' has wrong row count after YAML round-trip",
                        name,
                        key
                    );
                }
            }
        }
    }
}

/// Test all fixtures round-trip through XML.
#[test]
fn all_fixtures_xml_roundtrip() {
    let xml_config = FromXmlConfig {
        infer_lists: true,
        ..Default::default()
    };

    for (name, fixture_fn) in fixtures::all() {
        let original = fixture_fn();
        let xml = to_xml(&original, &ToXmlConfig::default()).unwrap();
        let restored = from_xml(&xml, &xml_config).unwrap();

        // Check same number of root items
        assert_eq!(
            original.root.len(),
            restored.root.len(),
            "Fixture '{}' lost root items in XML round-trip (original: {:?}, restored: {:?})",
            name,
            original.root.keys().collect::<Vec<_>>(),
            restored.root.keys().collect::<Vec<_>>()
        );

        // Check lists have same row counts
        for (key, item) in &original.root {
            if let Item::List(list) = item {
                if let Some(Item::List(restored_list)) = restored.root.get(key) {
                    assert_eq!(
                        list.rows.len(),
                        restored_list.rows.len(),
                        "Fixture '{}' list '{}' has wrong row count after XML round-trip",
                        name,
                        key
                    );
                }
            }
        }
    }
}

/// Helper function to assert lists match between original and restored documents.
fn assert_lists_match(original: &Document, restored: &Document, list_name: &str, format: &str) {
    let orig_list = original
        .root
        .get(list_name)
        .and_then(|item| {
            if let Item::List(list) = item {
                Some(list)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("Original document missing '{}' list", list_name));

    let rest_list = restored
        .root
        .get(list_name)
        .and_then(|item| {
            if let Item::List(list) = item {
                Some(list)
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            panic!(
                "{} round-trip: '{}' is not a List in restored document (got {:?})",
                format,
                list_name,
                restored.root.get(list_name)
            )
        });

    assert_eq!(
        orig_list.rows.len(),
        rest_list.rows.len(),
        "{} round-trip: '{}' list has {} rows but expected {}",
        format,
        list_name,
        rest_list.rows.len(),
        orig_list.rows.len()
    );
}
