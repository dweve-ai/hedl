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

//! Streaming iterator for child nodes in NEST hierarchies.
//!
//! This module provides memory-efficient iteration over child nodes without
//! full materialization, reducing peak memory usage by ~99% for large hierarchies.

use hedl_core::Node;
use std::collections::BTreeMap;

use crate::config::ToCypherConfig;
use crate::error::{Neo4jError, Result};
use crate::mapping::node::Neo4jNode;
use crate::mapping::value::value_to_cypher;

const DEFAULT_MAX_NEST_DEPTH: usize = 100;

/// Iterator state for depth-first NEST traversal.
struct TraversalFrame<'a> {
    /// Iterator over children collections in the node
    children_iter: std::collections::btree_map::Iter<'a, String, Vec<Node>>,
    /// Current children vector being processed
    current_children: Option<(&'a String, std::slice::Iter<'a, Node>)>,
    /// Current depth in hierarchy
    depth: usize,
}

/// Iterator over child nodes from NEST hierarchies.
///
/// This iterator performs depth-first traversal of NEST structures,
/// yielding child nodes one at a time to avoid full materialization.
///
/// # Memory Usage
///
/// - Stack depth: `O(max_depth)` for recursion frames
/// - Per-node overhead: O(1) for current node conversion
/// - Total: `O(max_depth)` instead of `O(total_children)`
///
/// # Example
///
/// ```no_run
/// # use hedl_neo4j::to_cypher::child_iterator::ChildNodeIterator;
/// # use hedl_neo4j::ToCypherConfig;
/// # use std::collections::BTreeMap;
/// # let parent_nodes = vec![];
/// # let structs = BTreeMap::new();
/// # let config = ToCypherConfig::default();
/// let iter = ChildNodeIterator::new(&parent_nodes, &structs, &config);
/// for result in iter {
///     let (child_type, neo4j_node) = result.unwrap();
///     // Process child incrementally
/// }
/// ```
pub struct ChildNodeIterator<'a> {
    parent_nodes: &'a [Node],
    parent_index: usize,
    stack: Vec<TraversalFrame<'a>>,
    structs: &'a BTreeMap<String, Vec<String>>,
    config: &'a ToCypherConfig,
    max_depth: usize,
}

impl<'a> ChildNodeIterator<'a> {
    /// Create a new child node iterator.
    #[must_use]
    pub fn new(
        parent_nodes: &'a [Node],
        structs: &'a BTreeMap<String, Vec<String>>,
        config: &'a ToCypherConfig,
    ) -> Self {
        Self {
            parent_nodes,
            parent_index: 0,
            stack: Vec::new(),
            structs,
            config,
            max_depth: DEFAULT_MAX_NEST_DEPTH,
        }
    }

    /// Initialize stack with next parent's children.
    fn initialize_next_parent(&mut self) -> bool {
        while self.parent_index < self.parent_nodes.len() {
            let parent = &self.parent_nodes[self.parent_index];
            self.parent_index += 1;

            if let Some(children_map) = parent.children() {
                if !children_map.is_empty() {
                    // Push parent to stack for traversal
                    self.stack.push(TraversalFrame {
                        children_iter: children_map.iter(),
                        current_children: None,
                        depth: 0,
                    });
                    return true;
                }
            }
        }
        false
    }

    /// Advance to next child in depth-first order.
    fn advance_to_next_child(&mut self) -> Option<Result<(String, Neo4jNode)>> {
        loop {
            // Check depth limit first
            {
                let frame = self.stack.last()?;
                if frame.depth > self.max_depth {
                    return Some(Err(Neo4jError::RecursionLimitExceeded {
                        depth: frame.depth,
                        max_depth: self.max_depth,
                    }));
                }
            }

            // If we have a current children vector, try to get next child
            let current_depth = self.stack.last()?.depth;
            let child_opt = if let Some(frame) = self.stack.last_mut() {
                if let Some((_child_key, children_iter)) = &mut frame.current_children {
                    children_iter.next()
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(child) = child_opt {
                // Check if we would exceed depth limit with this child
                let child_depth = current_depth + 1;
                if child_depth > self.max_depth {
                    return Some(Err(Neo4jError::RecursionLimitExceeded {
                        depth: child_depth,
                        max_depth: self.max_depth,
                    }));
                }

                // Convert child to Neo4jNode
                let neo4j_node = match self.convert_child_to_neo4j(child, child_depth) {
                    Ok(node) => node,
                    Err(e) => return Some(Err(e)),
                };

                let child_type = child.type_name.clone();

                // If child has children, push it onto the stack for further traversal
                if let Some(child_children) = child.children() {
                    if !child_children.is_empty() {
                        self.stack.push(TraversalFrame {
                            children_iter: child_children.iter(),
                            current_children: None,
                            depth: child_depth,
                        });
                    }
                }

                return Some(Ok((child_type, neo4j_node)));
            }
            // Exhausted current children vector, get next one
            if let Some(frame) = self.stack.last_mut() {
                frame.current_children = None;
            }

            // Try to get next children collection
            let frame = self.stack.last_mut()?;
            if let Some((child_key, children_vec)) = frame.children_iter.next() {
                if !children_vec.is_empty() {
                    frame.current_children = Some((child_key, children_vec.iter()));
                }
                // Loop back to process this children vector
            } else {
                // Exhausted all children collections at this level, backtrack
                self.stack.pop();
            }
        }
    }

    /// Convert a child node to `Neo4jNode` format.
    fn convert_child_to_neo4j(&self, child: &Node, _depth: usize) -> Result<Neo4jNode> {
        let mut neo4j_node = Neo4jNode::new(&child.type_name, &child.id);

        // Look up schema for this child type
        if let Some(schema) = self.structs.get(&child.type_name) {
            // Use schema column names for properties
            for (i, field) in child.fields.iter().enumerate() {
                // Skip ID field (first column)
                if i == 0 {
                    continue;
                }

                // Get the column name from schema
                if let Some(column_name) = schema.get(i) {
                    // Skip references as they become relationships
                    if !matches!(field, hedl_core::Value::Reference(_)) {
                        let cypher_value = value_to_cypher(field, column_name, self.config)?;
                        neo4j_node
                            .properties
                            .insert(column_name.clone(), cypher_value);
                    }
                }
            }
        } else {
            // Fallback: use generic field names if schema not found
            for (i, field) in child.fields.iter().enumerate() {
                if i == 0 {
                    continue; // Skip ID field
                }
                if !matches!(field, hedl_core::Value::Reference(_)) {
                    let prop_name = format!("field_{i}");
                    let cypher_value = value_to_cypher(field, &prop_name, self.config)?;
                    neo4j_node.properties.insert(prop_name, cypher_value);
                }
            }
        }

        Ok(neo4j_node)
    }
}

impl Iterator for ChildNodeIterator<'_> {
    type Item = Result<(String, Neo4jNode)>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If stack is empty, try to initialize next parent
            if self.stack.is_empty() && !self.initialize_next_parent() {
                return None; // No more parents
            }

            // Advance to next child in depth-first order
            // If None, the current parent's children are exhausted - try next parent
            if let Some(result) = self.advance_to_next_child() {
                return Some(result);
            }
        }
    }
}

/// Adapter that batches child nodes by type for efficient streaming.
///
/// This iterator wraps `ChildNodeIterator` and accumulates nodes into
/// type-grouped batches. When any type reaches the batch size threshold,
/// that batch is flushed.
///
/// # Memory Usage
///
/// Peak memory: `O(batch_size` × `num_types`) in worst case
/// Typical: `O(batch_size)` when types are evenly distributed
///
/// # Example
///
/// ```no_run
/// # use hedl_neo4j::to_cypher::child_iterator::{ChildNodeIterator, TypeBatchedChildren};
/// # use hedl_neo4j::ToCypherConfig;
/// # use std::collections::BTreeMap;
/// # let parents = vec![];
/// # let structs = BTreeMap::new();
/// # let config = ToCypherConfig::default();
/// let child_iter = ChildNodeIterator::new(&parents, &structs, &config);
/// let batched = TypeBatchedChildren::new(child_iter, 1000);
///
/// for batch_result in batched {
///     let batch = batch_result.unwrap(); // BTreeMap<String, Vec<Neo4jNode>>
///     for (child_type, nodes) in batch {
///         // process_batch(&child_type, &nodes);
///     }
/// }
/// ```
pub struct TypeBatchedChildren<I>
where
    I: Iterator<Item = Result<(String, Neo4jNode)>>,
{
    inner: I,
    current_batch: BTreeMap<String, Vec<Neo4jNode>>,
    batch_size: usize,
    finished: bool,
}

impl<I> TypeBatchedChildren<I>
where
    I: Iterator<Item = Result<(String, Neo4jNode)>>,
{
    /// Create a new type-batched iterator.
    pub fn new(inner: I, batch_size: usize) -> Self {
        Self {
            inner,
            current_batch: BTreeMap::new(),
            batch_size,
            finished: false,
        }
    }

    /// Check if any type has reached the batch size threshold.
    fn has_full_batch(&self) -> bool {
        self.current_batch
            .values()
            .any(|nodes| nodes.len() >= self.batch_size)
    }

    /// Flush batches that have reached the threshold.
    fn flush_full_batches(&mut self) -> Option<BTreeMap<String, Vec<Neo4jNode>>> {
        if self.current_batch.is_empty() {
            return None;
        }

        let mut result = BTreeMap::new();
        let mut remaining = BTreeMap::new();

        for (type_name, nodes) in std::mem::take(&mut self.current_batch) {
            if nodes.len() >= self.batch_size {
                result.insert(type_name, nodes);
            } else {
                remaining.insert(type_name, nodes);
            }
        }

        self.current_batch = remaining;

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    /// Flush all remaining batches (at end of iteration).
    fn flush_remaining(&mut self) -> Option<BTreeMap<String, Vec<Neo4jNode>>> {
        if self.current_batch.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.current_batch))
        }
    }
}

impl<I> Iterator for TypeBatchedChildren<I>
where
    I: Iterator<Item = Result<(String, Neo4jNode)>>,
{
    type Item = Result<BTreeMap<String, Vec<Neo4jNode>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        loop {
            // Try to read next child from inner iterator
            match self.inner.next() {
                Some(Ok((child_type, node))) => {
                    // Accumulate into current batch
                    self.current_batch.entry(child_type).or_default().push(node);

                    // Check if any batch is full
                    if self.has_full_batch() {
                        if let Some(batch) = self.flush_full_batches() {
                            return Some(Ok(batch));
                        }
                    }
                }
                Some(Err(e)) => {
                    self.finished = true;
                    return Some(Err(e));
                }
                None => {
                    // End of input - flush remaining batches
                    self.finished = true;
                    return self.flush_remaining().map(Ok);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::Value;

    fn create_test_config() -> ToCypherConfig {
        ToCypherConfig::default()
    }

    #[test]
    fn test_empty_parents() {
        let config = create_test_config();
        let structs = BTreeMap::new();
        let parents = vec![];

        let iter = ChildNodeIterator::new(&parents, &structs, &config);
        assert_eq!(iter.count(), 0);
    }

    #[test]
    fn test_parents_without_children() {
        let config = create_test_config();
        let structs = BTreeMap::new();

        let parent = Node {
            type_name: "User".to_string(),
            id: "u1".to_string(),
            fields: vec![Value::String("u1".to_string().into())].into(),
            children: None,
            child_count: 0,
        };
        let parents = vec![parent];

        let iter = ChildNodeIterator::new(&parents, &structs, &config);
        assert_eq!(iter.count(), 0);
    }

    #[test]
    fn test_single_parent_with_children() {
        let config = create_test_config();
        let mut structs = BTreeMap::new();
        structs.insert(
            "Post".to_string(),
            vec!["id".to_string(), "title".to_string()],
        );

        let child = Node {
            type_name: "Post".to_string(),
            id: "p1".to_string(),
            fields: vec![
                Value::String("p1".to_string().into()),
                Value::String("Hello".to_string().into()),
            ]
            .into(),
            children: None,
            child_count: 0,
        };

        let mut parent_children = BTreeMap::new();
        parent_children.insert("posts".to_string(), vec![child]);

        let parent = Node {
            type_name: "User".to_string(),
            id: "u1".to_string(),
            fields: vec![Value::String("u1".to_string().into())].into(),
            children: Some(Box::new(parent_children)),
            child_count: 0,
        };

        let parents = vec![parent];
        let iter = ChildNodeIterator::new(&parents, &structs, &config);
        let children: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(children.len(), 1);
        assert_eq!(children[0].0, "Post");
        assert_eq!(children[0].1.id, "p1");
    }

    #[test]
    fn test_multiple_children_same_type() {
        let config = create_test_config();
        let mut structs = BTreeMap::new();
        structs.insert(
            "Post".to_string(),
            vec!["id".to_string(), "title".to_string()],
        );

        let child1 = Node {
            type_name: "Post".to_string(),
            id: "p1".to_string(),
            fields: vec![
                Value::String("p1".to_string().into()),
                Value::String("First".to_string().into()),
            ]
            .into(),
            children: None,
            child_count: 0,
        };

        let child2 = Node {
            type_name: "Post".to_string(),
            id: "p2".to_string(),
            fields: vec![
                Value::String("p2".to_string().into()),
                Value::String("Second".to_string().into()),
            ]
            .into(),
            children: None,
            child_count: 0,
        };

        let mut parent_children = BTreeMap::new();
        parent_children.insert("posts".to_string(), vec![child1, child2]);

        let parent = Node {
            type_name: "User".to_string(),
            id: "u1".to_string(),
            fields: vec![Value::String("u1".to_string().into())].into(),
            children: Some(Box::new(parent_children)),
            child_count: 0,
        };

        let parents = vec![parent];
        let iter = ChildNodeIterator::new(&parents, &structs, &config);
        let children: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(children.len(), 2);
        assert_eq!(children[0].1.id, "p1");
        assert_eq!(children[1].1.id, "p2");
    }

    #[test]
    fn test_nested_children() {
        let config = create_test_config();
        let mut structs = BTreeMap::new();
        structs.insert(
            "Post".to_string(),
            vec!["id".to_string(), "title".to_string()],
        );
        structs.insert(
            "Comment".to_string(),
            vec!["id".to_string(), "text".to_string()],
        );

        let grandchild = Node {
            type_name: "Comment".to_string(),
            id: "c1".to_string(),
            fields: vec![
                Value::String("c1".to_string().into()),
                Value::String("Great!".to_string().into()),
            ]
            .into(),
            children: None,
            child_count: 0,
        };

        let mut child_children = BTreeMap::new();
        child_children.insert("comments".to_string(), vec![grandchild]);

        let child = Node {
            type_name: "Post".to_string(),
            id: "p1".to_string(),
            fields: vec![
                Value::String("p1".to_string().into()),
                Value::String("Hello".to_string().into()),
            ]
            .into(),
            children: Some(Box::new(child_children)),
            child_count: 0,
        };

        let mut parent_children = BTreeMap::new();
        parent_children.insert("posts".to_string(), vec![child]);

        let parent = Node {
            type_name: "User".to_string(),
            id: "u1".to_string(),
            fields: vec![Value::String("u1".to_string().into())].into(),
            children: Some(Box::new(parent_children)),
            child_count: 0,
        };

        let parents = vec![parent];
        let iter = ChildNodeIterator::new(&parents, &structs, &config);
        let children: Vec<_> = iter.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(children.len(), 2); // 1 Post + 1 Comment
        assert!(children.iter().any(|(t, n)| t == "Post" && n.id == "p1"));
        assert!(children.iter().any(|(t, n)| t == "Comment" && n.id == "c1"));
    }

    #[test]
    fn test_depth_limit_protection() {
        let config = create_test_config();
        let structs = BTreeMap::new();

        // Create a very deep hierarchy (more than DEFAULT_MAX_NEST_DEPTH levels)
        // Build from the deepest level upward
        let mut deepest = Node {
            type_name: "Level102".to_string(),
            id: "n102".to_string(),
            fields: vec![Value::String("n102".to_string().into())].into(),
            children: None,
            child_count: 0,
        };

        // Build a chain of 102 levels (exceeds DEFAULT_MAX_NEST_DEPTH of 100)
        for i in (0..102).rev() {
            let mut children_map = BTreeMap::new();
            children_map.insert("child".to_string(), vec![deepest]);

            deepest = Node {
                type_name: format!("Level{i}"),
                id: format!("n{i}"),
                fields: vec![Value::String(format!("n{i}").into())].into(),
                children: Some(Box::new(children_map)),
                child_count: 0,
            };
        }

        let parents = vec![deepest];
        let iter = ChildNodeIterator::new(&parents, &structs, &config);

        // Should eventually hit depth limit error
        let mut found_error = false;
        for result in iter {
            if let Err(Neo4jError::RecursionLimitExceeded { .. }) = result {
                found_error = true;
                break;
            }
        }
        assert!(found_error, "Expected RecursionLimitExceeded error");
    }

    // TypeBatchedChildren tests

    #[test]
    fn test_batching_single_type() {
        // Create iterator yielding 2500 nodes of same type
        let nodes: Vec<_> = (0..2500)
            .map(|i| {
                let node = Neo4jNode::new("Post", format!("p{i}"));
                Ok(("Post".to_string(), node))
            })
            .collect();

        let iter = nodes.into_iter();
        let batched = TypeBatchedChildren::new(iter, 1000);
        let batches: Vec<_> = batched.collect::<Result<Vec<_>>>().unwrap();

        // Should have 3 batches: 1000, 1000, 500
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].get("Post").unwrap().len(), 1000);
        assert_eq!(batches[1].get("Post").unwrap().len(), 1000);
        assert_eq!(batches[2].get("Post").unwrap().len(), 500);
    }

    #[test]
    fn test_batching_multiple_types() {
        // Create iterator alternating between two types
        let nodes: Vec<_> = (0..1500)
            .map(|i| {
                let type_name = if i % 2 == 0 { "Post" } else { "Comment" };
                let node = Neo4jNode::new(type_name, format!("n{i}"));
                Ok((type_name.to_string(), node))
            })
            .collect();

        let iter = nodes.into_iter();
        let batched = TypeBatchedChildren::new(iter, 1000);
        let batches: Vec<_> = batched.collect::<Result<Vec<_>>>().unwrap();

        // Should flush when either type reaches 1000
        // Post: 750, Comment: 750
        // No batch should exceed 1000 nodes per type
        for batch in &batches {
            for nodes in batch.values() {
                assert!(nodes.len() <= 1000);
            }
        }
    }

    #[test]
    fn test_batching_empty_input() {
        let nodes: Vec<Result<(String, Neo4jNode)>> = vec![];
        let iter = nodes.into_iter();
        let batched = TypeBatchedChildren::new(iter, 1000);
        let batches: Vec<_> = batched.collect::<Result<Vec<_>>>().unwrap();

        assert_eq!(batches.len(), 0);
    }

    #[test]
    fn test_batching_small_input() {
        let nodes: Vec<_> = (0..10)
            .map(|i| {
                let node = Neo4jNode::new("Post", format!("p{i}"));
                Ok(("Post".to_string(), node))
            })
            .collect();

        let iter = nodes.into_iter();
        let batched = TypeBatchedChildren::new(iter, 1000);
        let batches: Vec<_> = batched.collect::<Result<Vec<_>>>().unwrap();

        // Should have 1 batch with 10 nodes
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].get("Post").unwrap().len(), 10);
    }
}
