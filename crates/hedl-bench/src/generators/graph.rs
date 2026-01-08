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

//! Graph structure generators with cross-references.
//!
//! Generates HEDL documents with complex reference patterns including
//! directed acyclic graphs (DAGs), linked lists, and general graph structures.

use crate::datasets::{generate_graph, generate_reference_heavy};

/// Generates a directed acyclic graph (DAG) with specified nodes and edges.
///
/// Creates a graph structure where nodes reference other nodes without cycles.
///
/// # Arguments
///
/// * `nodes` - Number of nodes in the graph
/// * `edges` - Approximate number of edges (references)
///
/// # Returns
///
/// HEDL document string with DAG structure.
pub fn generate_dag(nodes: usize, edges: usize) -> String {
    if nodes == 0 {
        return "%VERSION: 1.0\n".to_string();
    }

    let edges_per_node = if nodes > 0 { (edges / nodes).max(1) } else { 1 };

    generate_graph(nodes, edges_per_node)
}

/// Generates a linked list structure.
///
/// Creates a chain of nodes where each node references the next.
///
/// # Arguments
///
/// * `length` - Number of nodes in the list
///
/// # Returns
///
/// HEDL document string with linked list structure.
pub fn generate_linked_list(length: usize) -> String {
    if length == 0 {
        return "%VERSION: 1.0\n".to_string();
    }

    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: Node: [id,value,next]\n");
    doc.push_str("list: @Node\n");

    for i in 0..length {
        let next_ref = if i < length - 1 {
            format!("@Node:{}", i + 1)
        } else {
            "null".to_string()
        };
        doc.push_str(&format!("| {}, value_{}, {}\n", i, i, next_ref));
    }

    doc
}

/// Generates a complex graph with specified density.
///
/// # Arguments
///
/// * `nodes` - Number of nodes
/// * `density` - Edge density (0.0 = no edges, 1.0 = complete graph)
///
/// # Returns
///
/// HEDL document string with complex graph structure.
pub fn generate_complex_graph(nodes: usize, density: f32) -> String {
    if nodes == 0 || density <= 0.0 {
        return generate_linked_list(nodes);
    }

    // Calculate edges based on density
    // Complete graph has n*(n-1)/2 edges
    let max_edges = nodes * (nodes - 1) / 2;
    let target_edges = (max_edges as f32 * density.min(1.0)) as usize;
    let edges_per_node = if nodes > 0 {
        (target_edges / nodes).max(1)
    } else {
        1
    };

    generate_graph(nodes, edges_per_node)
}

/// Generates a reference-heavy document with cross-references.
///
/// Creates a document with heavy use of @Type:id references for
/// complex relationships.
///
/// # Arguments
///
/// * `entity_count` - Number of entities
///
/// # Returns
///
/// HEDL document string with reference-heavy structure.
pub fn generate_reference_graph(entity_count: usize) -> String {
    generate_reference_heavy(entity_count)
}

/// Generates a bidirectional graph where edges go both ways.
///
/// # Arguments
///
/// * `nodes` - Number of nodes
/// * `edges_per_node` - Number of edges per node (both directions)
///
/// # Returns
///
/// HEDL document string with bidirectional graph.
pub fn generate_bidirectional_graph(nodes: usize, edges_per_node: usize) -> String {
    if nodes == 0 {
        return "%VERSION: 1.0\n".to_string();
    }

    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: Node: [id,name,edges]\n");
    doc.push_str("graph: @Node\n");

    for i in 0..nodes {
        let mut edges = Vec::new();
        for j in 1..=edges_per_node {
            let target = (i + j) % nodes;
            if target != i {
                edges.push(format!("@Node:{}", target));
            }
        }

        doc.push_str(&format!("| {}, node_{}, [{}]\n", i, i, edges.join(", ")));
    }

    doc
}

/// Generates a tree structure represented as a graph.
///
/// # Arguments
///
/// * `nodes` - Number of nodes in the tree
///
/// # Returns
///
/// HEDL document string with tree as graph.
pub fn generate_tree_graph(nodes: usize) -> String {
    if nodes == 0 {
        return "%VERSION: 1.0\n".to_string();
    }

    let mut doc = String::from("%VERSION: 1.0\n");
    doc.push_str("%STRUCT: TreeNode: [id,parent,children]\n");
    doc.push_str("tree: @TreeNode\n");

    // Root node
    doc.push_str("| 0, null, [");
    if nodes > 1 {
        doc.push_str("@TreeNode:1");
        if nodes > 2 {
            doc.push_str(", @TreeNode:2");
        }
    }
    doc.push_str("]\n");

    // Child nodes
    for i in 1..nodes {
        let parent = (i - 1) / 2;
        let left_child = 2 * i + 1;
        let right_child = 2 * i + 2;

        let mut children = Vec::new();
        if left_child < nodes {
            children.push(format!("@TreeNode:{}", left_child));
        }
        if right_child < nodes {
            children.push(format!("@TreeNode:{}", right_child));
        }

        doc.push_str(&format!(
            "| {}, @TreeNode:{}, [{}]\n",
            i,
            parent,
            children.join(", ")
        ));
    }

    doc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag() {
        let doc = generate_dag(10, 20);
        assert!(doc.contains("%VERSION: 1.0"));
        assert!(doc.contains("%STRUCT:"));
    }

    #[test]
    fn test_linked_list() {
        let doc = generate_linked_list(5);
        assert!(doc.contains("@Node:"));
        assert!(doc.contains("null"));
    }

    #[test]
    fn test_complex_graph() {
        let doc = generate_complex_graph(10, 0.5);
        assert!(doc.contains("%VERSION: 1.0"));
    }

    #[test]
    fn test_bidirectional_graph() {
        let doc = generate_bidirectional_graph(5, 2);
        assert!(doc.contains("@Node:"));
    }

    #[test]
    fn test_tree_graph() {
        let doc = generate_tree_graph(7);
        assert!(doc.contains("TreeNode"));
        assert!(doc.contains("parent"));
        assert!(doc.contains("children"));
    }
}
