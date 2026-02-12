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

//! Input validation for HEDL to Cypher conversion.

use hedl_core::{Document, Item};

use crate::config::ToCypherConfig;
use crate::error::{Neo4jError, Result};

/// Default maximum NEST hierarchy depth to prevent stack overflow.
///
/// This limit protects against:
/// - Maliciously crafted deeply nested structures
/// - Infinite recursion from circular references
/// - Stack overflow attacks
///
/// The limit of 100 is sufficient for practical use cases while preventing
/// resource exhaustion.
pub(crate) const DEFAULT_MAX_NEST_DEPTH: usize = 100;

/// Count total nodes in a document, including NEST children.
///
/// This function traverses the entire document tree to count all nodes,
/// including nested children in NEST hierarchies. It is used for early
/// limit checking before any memory allocation occurs.
///
/// # Arguments
///
/// * `doc` - The HEDL document to count nodes in
///
/// # Returns
///
/// The total number of nodes in the document, including all nested children
///
/// # Performance
///
/// This is an O(n) operation where n is the number of nodes. It is only called
/// when a `max_nodes` limit is configured, so there is no overhead for trusted
/// input with unlimited processing.
fn count_total_nodes(doc: &Document) -> usize {
    let mut count = 0;

    for item in doc.root.values() {
        if let Item::List(matrix_list) = item {
            count += matrix_list.rows.len();
            // Also count nested children
            for node in &matrix_list.rows {
                count += count_children_recursive(node);
            }
        }
    }

    count
}

/// Recursively count children in a node's NEST hierarchy.
///
/// This function traverses all children of a node and recursively counts
/// their descendants, providing an accurate total for the entire subtree.
///
/// # Arguments
///
/// * `node` - The node whose children should be counted
///
/// # Returns
///
/// The total number of descendant nodes (children, grandchildren, etc.)
fn count_children_recursive(node: &hedl_core::Node) -> usize {
    let mut count = 0;
    if let Some(children_map) = node.children() {
        for children in children_map.values() {
            count += children.len();
            for child in children {
                count += count_children_recursive(child);
            }
        }
    }
    count
}

/// Validate node count against configured limit.
///
/// This function checks if the total node count (including NEST children)
/// exceeds the configured maximum. It should be called early in the conversion
/// process, before any memory allocation for node conversion occurs.
///
/// # Arguments
///
/// * `doc` - The HEDL document to validate
/// * `config` - Configuration containing the `max_nodes` limit
///
/// # Returns
///
/// * `Ok(())` if within limit or no limit set
/// * `Err(NodeCountExceeded)` if limit exceeded
///
/// # Security
///
/// This is a critical security function that prevents `DoS` attacks through
/// memory exhaustion. It MUST be called before any node processing begins.
pub(crate) fn validate_node_count(doc: &Document, config: &ToCypherConfig) -> Result<()> {
    if let Some(max_nodes) = config.max_nodes {
        let total_nodes = count_total_nodes(doc);
        if total_nodes > max_nodes {
            return Err(Neo4jError::NodeCountExceeded {
                count: total_nodes,
                max_count: max_nodes,
            });
        }
    }
    Ok(())
}
