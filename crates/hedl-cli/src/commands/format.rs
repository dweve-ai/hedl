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

//! Format command - HEDL canonicalization and formatting

use super::{read_file, write_output};
use hedl_c14n::{canonicalize_with_config, CanonicalConfig};
use hedl_core::{parse, Document, Item};

/// Format a HEDL file to canonical form.
///
/// Parses a HEDL file and outputs it in canonical (standardized) form. Can be used
/// to check if a file is already canonical, or to add count hints to all matrix lists.
///
/// # Arguments
///
/// * `file` - Path to the HEDL file to format
/// * `output` - Optional output file path. If `None`, writes to stdout
/// * `check` - If `true`, only checks if the file is canonical without reformatting
/// * `ditto` - If `true`, uses ditto optimization (repeated values as `"`)
/// * `with_counts` - If `true`, automatically adds count hints to all matrix lists
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err` if:
/// - The file cannot be read or parsed
/// - In check mode, if the file is not in canonical form
/// - Output cannot be written
///
/// # Errors
///
/// Returns `Err` if:
/// - The file cannot be read
/// - The file contains syntax errors
/// - Canonicalization fails
/// - In check mode, if the file is not already canonical
/// - Output writing fails
///
/// # Examples
///
/// ```no_run
/// use hedl_cli::commands::format;
///
/// # fn main() -> Result<(), String> {
/// // Format to stdout
/// format("input.hedl", None, false, true, false)?;
///
/// // Format to file with count hints
/// format("input.hedl", Some("output.hedl"), false, true, true)?;
///
/// // Check if file is already canonical
/// let result = format("input.hedl", None, true, true, false);
/// if result.is_ok() {
///     println!("File is already canonical");
/// }
///
/// // Disable ditto optimization
/// format("input.hedl", Some("output.hedl"), false, false, false)?;
/// # Ok(())
/// # }
/// ```
///
/// # Output
///
/// In check mode, prints "File is in canonical form" if valid, or returns an error.
/// Otherwise, writes the canonical HEDL to the specified output or stdout.
pub fn format(
    file: &str,
    output: Option<&str>,
    check: bool,
    ditto: bool,
    with_counts: bool,
) -> Result<(), String> {
    let content = read_file(file)?;

    let mut doc = parse(content.as_bytes()).map_err(|e| format!("Parse error: {}", e))?;

    // Add count hints if requested
    if with_counts {
        add_count_hints(&mut doc);
    }

    let mut config = CanonicalConfig::default();
    config.use_ditto = ditto;

    let canonical = canonicalize_with_config(&doc, &config)
        .map_err(|e| format!("Canonicalization error: {}", e))?;

    if check {
        // Compare with original (normalized)
        let normalized_original = content.replace("\r\n", "\n");
        if canonical.trim() != normalized_original.trim() {
            return Err("File is not in canonical form".to_string());
        }
        println!("File is in canonical form");
        Ok(())
    } else {
        write_output(&canonical, output)
    }
}

/// Recursively add count hints to all matrix lists in the document
fn add_count_hints(doc: &mut Document) {
    for item in doc.root.values_mut() {
        add_count_hints_to_item(item);
    }
}

/// Recursively add count hints to an item
fn add_count_hints_to_item(item: &mut Item) {
    match item {
        Item::List(list) => {
            // Set count hint based on actual row count
            list.count_hint = Some(list.rows.len());

            // Recursively add child counts to each node
            for node in &mut list.rows {
                add_child_count_to_node(node);
            }
        }
        Item::Object(map) => {
            // Recursively process nested objects
            for nested_item in map.values_mut() {
                add_count_hints_to_item(nested_item);
            }
        }
        Item::Scalar(_) => {
            // Scalars don't have matrix lists
        }
    }
}

/// Recursively set child_count on nodes that have children
fn add_child_count_to_node(node: &mut hedl_core::Node) {
    // Calculate total number of direct children across all child types
    let total_children: usize = node.children.values().map(|v| v.len()).sum();

    if total_children > 0 {
        node.child_count = Some(total_children);

        // Recursively process all child nodes
        for child_list in node.children.values_mut() {
            for child_node in child_list {
                add_child_count_to_node(child_node);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{MatrixList, Node, Value};

    #[test]
    fn test_add_count_hints_to_empty_list() {
        let list = MatrixList::new("Team", vec!["id".to_string(), "name".to_string()]);
        assert_eq!(list.count_hint, None);

        let mut item = Item::List(list);
        add_count_hints_to_item(&mut item);

        if let Item::List(list) = item {
            assert_eq!(list.count_hint, Some(0));
        } else {
            panic!("Expected List item");
        }
    }

    #[test]
    fn test_add_count_hints_to_list_with_rows() {
        let mut list = MatrixList::new("Team", vec!["id".to_string(), "name".to_string()]);
        list.add_row(Node::new("Team", "t1", vec![Value::String("Team 1".into())]));
        list.add_row(Node::new("Team", "t2", vec![Value::String("Team 2".into())]));
        list.add_row(Node::new("Team", "t3", vec![Value::String("Team 3".into())]));
        assert_eq!(list.count_hint, None);

        let mut item = Item::List(list);
        add_count_hints_to_item(&mut item);

        if let Item::List(list) = item {
            assert_eq!(list.count_hint, Some(3));
            assert_eq!(list.rows.len(), 3);
        } else {
            panic!("Expected List item");
        }
    }

    #[test]
    fn test_add_count_hints_overwrites_existing() {
        let mut list = MatrixList::with_count_hint(
            "Team",
            vec!["id".to_string(), "name".to_string()],
            5,
        );
        list.add_row(Node::new("Team", "t1", vec![Value::String("Team 1".into())]));
        list.add_row(Node::new("Team", "t2", vec![Value::String("Team 2".into())]));
        assert_eq!(list.count_hint, Some(5)); // Old value

        let mut item = Item::List(list);
        add_count_hints_to_item(&mut item);

        if let Item::List(list) = item {
            assert_eq!(list.count_hint, Some(2)); // Updated to actual count
            assert_eq!(list.rows.len(), 2);
        } else {
            panic!("Expected List item");
        }
    }

    #[test]
    fn test_add_count_hints_to_nested_objects() {
        use std::collections::BTreeMap;

        let mut list1 = MatrixList::new("Team", vec!["id".to_string()]);
        list1.add_row(Node::new("Team", "t1", vec![]));

        let mut list2 = MatrixList::new("Player", vec!["id".to_string()]);
        list2.add_row(Node::new("Player", "p1", vec![]));
        list2.add_row(Node::new("Player", "p2", vec![]));

        let mut inner_map = BTreeMap::new();
        inner_map.insert("teams".to_string(), Item::List(list1));

        let mut outer_map = BTreeMap::new();
        outer_map.insert("sports".to_string(), Item::Object(inner_map));
        outer_map.insert("players".to_string(), Item::List(list2));

        let mut item = Item::Object(outer_map);
        add_count_hints_to_item(&mut item);

        // Verify nested structure has count hints
        if let Item::Object(map) = item {
            // Check teams nested in sports
            if let Some(Item::Object(sports)) = map.get("sports") {
                if let Some(Item::List(teams)) = sports.get("teams") {
                    assert_eq!(teams.count_hint, Some(1));
                } else {
                    panic!("Expected teams list in sports");
                }
            } else {
                panic!("Expected sports object");
            }

            // Check players at top level
            if let Some(Item::List(players)) = map.get("players") {
                assert_eq!(players.count_hint, Some(2));
            } else {
                panic!("Expected players list");
            }
        } else {
            panic!("Expected Object item");
        }
    }

    #[test]
    fn test_add_count_hints_to_scalar() {
        let mut item = Item::Scalar(Value::String("test".into()));
        add_count_hints_to_item(&mut item);
        // Should not panic, just do nothing
        assert!(matches!(item, Item::Scalar(_)));
    }

    #[test]
    fn test_add_count_hints_to_empty_object() {
        use std::collections::BTreeMap;

        let mut item = Item::Object(BTreeMap::new());
        add_count_hints_to_item(&mut item);
        // Should not panic, just do nothing
        assert!(matches!(item, Item::Object(_)));
    }

    #[test]
    fn test_add_count_hints_document() {
        let mut doc = Document::new((1, 0));

        let mut list1 = MatrixList::new("Team", vec!["id".to_string()]);
        list1.add_row(Node::new("Team", "t1", vec![]));
        list1.add_row(Node::new("Team", "t2", vec![]));

        let mut list2 = MatrixList::new("Player", vec!["id".to_string()]);
        list2.add_row(Node::new("Player", "p1", vec![]));

        doc.root.insert("teams".to_string(), Item::List(list1));
        doc.root.insert("players".to_string(), Item::List(list2));

        add_count_hints(&mut doc);

        // Verify both lists have count hints
        if let Some(Item::List(teams)) = doc.root.get("teams") {
            assert_eq!(teams.count_hint, Some(2));
        } else {
            panic!("Expected teams list");
        }

        if let Some(Item::List(players)) = doc.root.get("players") {
            assert_eq!(players.count_hint, Some(1));
        } else {
            panic!("Expected players list");
        }
    }
}
