// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Count hint utilities for HEDL documents.
//!
//! Provides functions to automatically add count hints to matrix lists and
//! child counts to nodes. Count hints improve parsing performance by allowing
//! pre-allocation of data structures.
//!
//! # Examples
//!
//! ```no_run
//! use hedl_c14n::add_count_hints;
//! use hedl_core::Document;
//!
//! # fn example(mut doc: Document) {
//! // Add count hints to all matrix lists in the document
//! add_count_hints(&mut doc);
//! # }
//! ```

use hedl_core::{Document, Item, Node};

/// Recursively add count hints to all matrix lists in the document.
///
/// This function walks the entire document tree and:
/// - Sets `count_hint` on each `MatrixList` to match the actual row count
/// - Sets `child_count` on each `Node` that has children
///
/// Count hints improve parsing performance by allowing parsers to pre-allocate
/// data structures. This is particularly useful for large documents with many
/// matrix lists.
///
/// # Arguments
///
/// * `doc` - The HEDL document to modify in-place
///
/// # Examples
///
/// ```no_run
/// use hedl_c14n::add_count_hints;
/// use hedl_core::{parse, Document};
///
/// # fn example(content: &[u8]) -> Result<(), hedl_core::HedlError> {
/// let mut doc = parse(content)?;
/// add_count_hints(&mut doc);
/// # Ok(())
/// # }
/// ```
///
/// # Performance
///
/// This function has O(n) complexity where n is the total number of nodes and
/// items in the document tree. It performs a single traversal of the tree.
pub fn add_count_hints(doc: &mut Document) {
    for item in doc.root.values_mut() {
        add_count_hints_to_item(item);
    }
}

/// Recursively add count hints to an item.
///
/// Internal helper function that processes a single item and recursively
/// handles nested structures.
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

/// Recursively set `child_count` on nodes that have children.
///
/// Internal helper function that calculates and sets the total number of
/// direct children across all child types for each node.
fn add_child_count_to_node(node: &mut Node) {
    // Calculate total number of direct children across all child types
    let total_children: usize = node
        .children()
        .map_or(0, |c| c.values().map(std::vec::Vec::len).sum());

    if total_children > 0 {
        node.child_count = total_children as u16;

        // Recursively process all child nodes
        if let Some(children) = node.children_mut() {
            for child_list in children.values_mut() {
                for child_node in child_list {
                    add_child_count_to_node(child_node);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{MatrixList, Value};
    use std::collections::BTreeMap;

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
        list.add_row(Node::new(
            "Team",
            "t1",
            vec![Value::String("Team 1".into())],
        ));
        list.add_row(Node::new(
            "Team",
            "t2",
            vec![Value::String("Team 2".into())],
        ));
        list.add_row(Node::new(
            "Team",
            "t3",
            vec![Value::String("Team 3".into())],
        ));
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
        let mut list =
            MatrixList::with_count_hint("Team", vec!["id".to_string(), "name".to_string()], 5);
        list.add_row(Node::new(
            "Team",
            "t1",
            vec![Value::String("Team 1".into())],
        ));
        list.add_row(Node::new(
            "Team",
            "t2",
            vec![Value::String("Team 2".into())],
        ));
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
            // Check inner nested list
            if let Some(Item::Object(inner)) = map.get("sports") {
                if let Some(Item::List(teams)) = inner.get("teams") {
                    assert_eq!(teams.count_hint, Some(1));
                } else {
                    panic!("Expected teams list");
                }
            } else {
                panic!("Expected sports object");
            }

            // Check top-level list
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
    fn test_add_child_count_to_node() {
        let mut parent = Node::new("Team", "t1", vec![]);
        assert_eq!(parent.child_count, 0);

        // Add children
        parent.add_child("Player", Node::new("Player", "p1", vec![]));
        parent.add_child("Player", Node::new("Player", "p2", vec![]));
        parent.add_child("Coach", Node::new("Coach", "c1", vec![]));

        add_child_count_to_node(&mut parent);

        assert_eq!(parent.child_count, 3);
    }

    #[test]
    fn test_add_child_count_recursive() {
        let mut grandparent = Node::new("League", "l1", vec![]);
        let mut parent = Node::new("Team", "t1", vec![]);

        parent.add_child("Player", Node::new("Player", "p1", vec![]));
        parent.add_child("Player", Node::new("Player", "p2", vec![]));

        grandparent.add_child("Team", parent);

        add_child_count_to_node(&mut grandparent);

        assert_eq!(grandparent.child_count, 1); // 1 team

        // Check nested child count
        if let Some(children) = grandparent.children() {
            if let Some(teams) = children.get("Team") {
                if let Some(team) = teams.first() {
                    assert_eq!(team.child_count, 2); // 2 players
                } else {
                    panic!("Expected team node");
                }
            } else {
                panic!("Expected Team children");
            }
        } else {
            panic!("Expected children");
        }
    }

    #[test]
    fn test_node_without_children_has_no_count() {
        let mut node = Node::new("Player", "p1", vec![]);

        add_child_count_to_node(&mut node);

        assert_eq!(node.child_count, 0);
    }
}
