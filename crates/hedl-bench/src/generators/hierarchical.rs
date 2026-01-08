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

//! Hierarchical and nested structure generators.
//!
//! Generates deeply nested and tree-structured HEDL documents for testing
//! recursive parsing, deep nesting performance, and hierarchical relationships.

use crate::datasets::{generate_blog, generate_deep_hierarchy, generate_nested, generate_orders};

/// Generates deep nesting with specified depth and fields per level.
///
/// Creates a document with recursive nesting where each level contains
/// the specified number of fields.
///
/// # Arguments
///
/// * `depth` - Number of nesting levels
/// * `fields_per_level` - Number of fields at each level
///
/// # Returns
///
/// HEDL document string with deep nesting.
pub fn generate_deep_nesting(depth: usize, fields_per_level: usize) -> String {
    if depth == 0 {
        return "%VERSION: 1.0\n---\n".to_string();
    }

    let mut doc = String::from("%VERSION: 1.0\n---\nroot:\n");

    fn add_level(
        doc: &mut String,
        current_level: usize,
        max_depth: usize,
        fields_per_level: usize,
        indent: usize,
    ) {
        let prefix = "  ".repeat(indent);

        for field_idx in 0..fields_per_level {
            doc.push_str(&format!(
                "{}field{}: val{}_{}\n",
                prefix, field_idx, current_level, field_idx
            ));
        }

        if current_level < max_depth {
            doc.push_str(&format!("{}nested:\n", prefix));
            add_level(
                doc,
                current_level + 1,
                max_depth,
                fields_per_level,
                indent + 1,
            );
        }
    }

    add_level(&mut doc, 0, depth - 1, fields_per_level, 1);
    doc
}

/// Generates a wide tree with specified breadth and depth.
///
/// Creates a tree structure where each node has `breadth` children
/// up to the specified depth.
///
/// # Arguments
///
/// * `breadth` - Number of children per node
/// * `depth` - Tree depth
///
/// # Returns
///
/// HEDL document string with wide tree structure.
pub fn generate_wide_tree(breadth: usize, depth: usize) -> String {
    if depth == 0 || breadth == 0 {
        return "%VERSION: 1.0\n---\n".to_string();
    }

    let mut doc = String::from("%VERSION: 1.0\n---\nroot:\n");
    doc.push_str("  id: 0\n");
    doc.push_str("  name: root\n");

    fn generate_children(
        doc: &mut String,
        breadth: usize,
        current_depth: usize,
        max_depth: usize,
        id_counter: &mut usize,
        indent: usize,
    ) {
        if current_depth >= max_depth {
            return;
        }

        let prefix = "  ".repeat(indent);

        for child_idx in 0..breadth {
            *id_counter += 1;
            doc.push_str(&format!("{}child{}:\n", prefix, child_idx));
            doc.push_str(&format!("{}  id: {}\n", prefix, id_counter));
            doc.push_str(&format!("{}  name: node_{}\n", prefix, id_counter));

            if current_depth + 1 < max_depth {
                generate_children(
                    doc,
                    breadth,
                    current_depth + 1,
                    max_depth,
                    id_counter,
                    indent + 1,
                );
            }
        }
    }

    let mut id_counter = 0;
    generate_children(&mut doc, breadth, 1, depth, &mut id_counter, 1);

    doc
}

/// Generates a balanced tree with approximately the specified number of nodes.
///
/// # Arguments
///
/// * `nodes` - Target number of nodes in the tree
///
/// # Returns
///
/// HEDL document string with balanced tree.
pub fn generate_balanced_tree(nodes: usize) -> String {
    if nodes == 0 {
        return "%VERSION: 1.0\n".to_string();
    }

    // Calculate depth and breadth for balanced tree
    let depth = (nodes as f64).log2().ceil() as usize;
    let breadth = if depth > 0 {
        ((nodes as f64).powf(1.0 / depth as f64)).ceil() as usize
    } else {
        1
    };

    generate_wide_tree(breadth.max(2), depth.max(1))
}

/// Generates organizational hierarchy (company/division/department/team).
///
/// Uses the existing deep_hierarchy generator with specified entity count.
///
/// # Arguments
///
/// * `entity_count` - Number of leaf entities to generate
///
/// # Returns
///
/// HEDL document string with organizational hierarchy.
pub fn generate_org_hierarchy(entity_count: usize) -> String {
    generate_deep_hierarchy(entity_count)
}

/// Generates blog posts with nested comments hierarchy.
///
/// # Arguments
///
/// * `post_count` - Number of blog posts
/// * `comments_per_post` - Average comments per post
///
/// # Returns
///
/// HEDL document string with blog post hierarchy.
pub fn generate_blog_hierarchy(post_count: usize, comments_per_post: usize) -> String {
    generate_blog(post_count, comments_per_post)
}

/// Generates orders with nested items.
///
/// # Arguments
///
/// * `order_count` - Number of orders to generate
///
/// # Returns
///
/// HEDL document string with order hierarchy.
pub fn generate_order_hierarchy(order_count: usize) -> String {
    generate_orders(order_count)
}

/// Generates custom nested structure with specified depth.
///
/// # Arguments
///
/// * `depth` - Nesting depth
///
/// # Returns
///
/// HEDL document string with custom nesting.
pub fn generate_custom_nested(depth: usize) -> String {
    generate_nested(depth)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_nesting() {
        let doc = generate_deep_nesting(3, 2);
        assert!(doc.contains("%VERSION: 1.0"));
        assert!(doc.contains("field0"));
        assert!(doc.contains("nested"));
    }

    #[test]
    fn test_wide_tree() {
        let doc = generate_wide_tree(3, 2);
        assert!(doc.contains("%VERSION: 1.0"));
        assert!(doc.contains("child0"));
        assert!(doc.contains("name:"));
    }

    #[test]
    fn test_balanced_tree() {
        let doc = generate_balanced_tree(10);
        assert!(doc.contains("%VERSION: 1.0"));
    }

    #[test]
    fn test_blog_hierarchy() {
        let doc = generate_blog_hierarchy(5, 3);
        assert!(doc.contains("%STRUCT:"));
    }
}
