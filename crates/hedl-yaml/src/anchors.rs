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

//! YAML anchor and alias handling with cycle detection and forward reference validation.

use crate::error::YamlError;
use std::collections::{HashMap, HashSet};

/// Registry for YAML anchors, tracking definitions, usage sites, and dependencies.
///
/// This structure is used during two-pass YAML parsing to:
/// 1. Track all anchor definitions and their locations
/// 2. Detect circular references between anchors
/// 3. Validate that aliases only reference previously defined anchors (no forward references)
/// 4. Build a dependency graph for cycle detection
#[derive(Debug, Clone, Default)]
pub struct AnchorRegistry {
    /// Anchor name -> (anchor value as YAML string, source line number)
    definitions: HashMap<String, (String, usize)>,

    /// Anchor name -> list of line numbers where this anchor is aliased
    usage_sites: HashMap<String, Vec<usize>>,

    /// Directed graph: anchor -> set of anchors it references
    /// This is used for cycle detection
    dependency_graph: HashMap<String, HashSet<String>>,

    /// Anchors in order of definition (for validation and processing)
    definition_order: Vec<String>,
}

impl AnchorRegistry {
    /// Create a new empty anchor registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new anchor definition.
    ///
    /// # Arguments
    ///
    /// * `name` - The anchor name (without the & prefix)
    /// * `yaml_content` - The YAML content of the anchored value as a string
    /// * `line` - The line number where the anchor is defined
    ///
    /// # Errors
    ///
    /// Returns an error if the anchor name is already defined (redefinition).
    pub fn add_anchor(
        &mut self,
        name: String,
        yaml_content: String,
        line: usize,
    ) -> Result<(), YamlError> {
        // Check for redefinition
        if let Some((_, old_line)) = self.definitions.get(&name) {
            return Err(YamlError::AnchorRedefinition {
                name,
                old_line: *old_line,
                new_line: line,
            });
        }

        self.definitions.insert(name.clone(), (yaml_content, line));
        self.definition_order.push(name.clone());
        self.dependency_graph.insert(name, HashSet::new());

        Ok(())
    }

    /// Record an alias usage site.
    ///
    /// # Arguments
    ///
    /// * `alias_name` - The name of the anchor being aliased (without the * prefix)
    /// * `line` - The line number where the alias appears
    ///
    /// # Errors
    ///
    /// Returns an error if the alias references an undefined anchor (forward reference).
    pub fn add_alias(&mut self, alias_name: &str, line: usize) -> Result<(), YamlError> {
        // Validate that anchor is defined before this alias (no forward references)
        if !self.definitions.contains_key(alias_name) {
            return Err(YamlError::ForwardReference {
                alias: alias_name.to_string(),
                line,
            });
        }

        self.usage_sites
            .entry(alias_name.to_string())
            .or_default()
            .push(line);

        Ok(())
    }

    /// Add a dependency edge between two anchors.
    ///
    /// This records that `from_anchor` contains a reference to `to_anchor`.
    ///
    /// # Arguments
    ///
    /// * `from_anchor` - The anchor that contains a reference
    /// * `to_anchor` - The anchor being referenced
    pub fn add_dependency(&mut self, from_anchor: &str, to_anchor: &str) {
        self.dependency_graph
            .entry(from_anchor.to_string())
            .or_default()
            .insert(to_anchor.to_string());
    }

    /// Get the YAML content and line number for an anchor.
    pub fn get_anchor(&self, name: &str) -> Option<&(String, usize)> {
        self.definitions.get(name)
    }

    /// Check if an anchor is defined.
    pub fn has_anchor(&self, name: &str) -> bool {
        self.definitions.contains_key(name)
    }

    /// Get all anchor names in definition order.
    pub fn anchor_names(&self) -> &[String] {
        &self.definition_order
    }

    /// Check if the registry has any aliases.
    pub fn has_aliases(&self) -> bool {
        !self.usage_sites.is_empty()
    }

    /// Get the number of anchors defined.
    pub fn anchor_count(&self) -> usize {
        self.definitions.len()
    }
}

/// Cycle detector using depth-first search with recursion tracking.
///
/// This implements a modified Tarjan's algorithm for cycle detection in directed graphs,
/// optimized for early termination on first cycle found.
#[derive(Debug)]
pub struct CycleDetector<'a> {
    registry: &'a AnchorRegistry,
    visited: HashSet<String>,
    recursion_stack: Vec<String>,
}

impl<'a> CycleDetector<'a> {
    /// Create a new cycle detector for the given anchor registry.
    pub fn new(registry: &'a AnchorRegistry) -> Self {
        Self {
            registry,
            visited: HashSet::new(),
            recursion_stack: Vec::new(),
        }
    }

    /// Detect cycles in the anchor dependency graph.
    ///
    /// # Errors
    ///
    /// Returns an error if a cycle is detected, with a detailed cycle path.
    pub fn detect_cycles(&mut self) -> Result<(), YamlError> {
        // Only run cycle detection if there are actually aliases
        if !self.registry.has_aliases() {
            return Ok(());
        }

        // Visit all anchors in definition order
        for anchor in self.registry.anchor_names() {
            if !self.visited.contains(anchor) {
                self.visit_anchor(anchor)?;
            }
        }

        Ok(())
    }

    /// Visit an anchor node during DFS traversal.
    ///
    /// Returns an error if a cycle is detected from this node.
    fn visit_anchor(&mut self, anchor: &str) -> Result<(), YamlError> {
        // Mark as currently visiting
        self.visited.insert(anchor.to_string());
        self.recursion_stack.push(anchor.to_string());

        // Check all outgoing edges (anchors this anchor references)
        if let Some(dependencies) = self.registry.dependency_graph.get(anchor) {
            for referenced in dependencies {
                // Check if we've found a cycle
                if let Some(cycle_start_idx) =
                    self.recursion_stack.iter().position(|a| a == referenced)
                {
                    // Cycle detected - build the cycle path
                    return Err(self.build_cycle_error(cycle_start_idx));
                }

                // Recursively visit unvisited nodes
                if !self.visited.contains(referenced) {
                    self.visit_anchor(referenced)?;
                }
            }
        }

        // Done visiting this anchor - pop from recursion stack
        self.recursion_stack.pop();

        Ok(())
    }

    /// Build a detailed error message for a detected cycle.
    fn build_cycle_error(&self, cycle_start_idx: usize) -> YamlError {
        // Extract the cycle path from the recursion stack
        let cycle_path: Vec<String> = self.recursion_stack[cycle_start_idx..].to_vec();

        // Add the first element again to complete the cycle
        let mut full_cycle = cycle_path.clone();
        full_cycle.push(cycle_path[0].clone());

        // Format the cycle path
        let cycle_path_str = full_cycle.join(" -> ");

        // Collect anchor locations
        let mut anchors = Vec::new();
        let mut locations = Vec::new();

        for anchor_name in &cycle_path {
            anchors.push(anchor_name.clone());
            if let Some((_, line)) = self.registry.get_anchor(anchor_name) {
                locations.push(*line);
            }
        }

        YamlError::CircularReference {
            cycle_path: cycle_path_str,
            anchors,
            locations,
        }
    }
}

/// Detect cycles in an anchor registry.
///
/// This is a convenience function that creates a `CycleDetector` and runs cycle detection.
///
/// # Errors
///
/// Returns an error if a cycle is detected.
pub fn detect_cycles(registry: &AnchorRegistry) -> Result<(), YamlError> {
    let mut detector = CycleDetector::new(registry);
    detector.detect_cycles()
}

/// Validate anchor names for HEDL compatibility.
///
/// Ensures anchor names don't conflict with HEDL reserved identifiers or patterns.
///
/// # Errors
///
/// Returns an error if an anchor name is invalid.
pub fn validate_anchor_name(name: &str) -> Result<(), YamlError> {
    // Check for empty name
    if name.is_empty() {
        return Err(YamlError::InvalidAnchorName {
            name: name.to_string(),
            reason: "Anchor name cannot be empty".to_string(),
        });
    }

    // Check for HEDL reserved prefixes that could cause conflicts
    if name.starts_with("__") {
        return Err(YamlError::InvalidAnchorName {
            name: name.to_string(),
            reason: "Anchor names starting with '__' are reserved for HEDL metadata".to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_registry_basic() {
        let mut registry = AnchorRegistry::new();

        registry
            .add_anchor("anchor1".to_string(), "value: 42".to_string(), 1)
            .unwrap();
        registry
            .add_anchor("anchor2".to_string(), "value: 43".to_string(), 5)
            .unwrap();

        assert_eq!(registry.anchor_count(), 2);
        assert!(registry.has_anchor("anchor1"));
        assert!(registry.has_anchor("anchor2"));
        assert!(!registry.has_anchor("anchor3"));
    }

    #[test]
    fn test_anchor_redefinition_error() {
        let mut registry = AnchorRegistry::new();

        registry
            .add_anchor("anchor1".to_string(), "value: 42".to_string(), 1)
            .unwrap();

        let result = registry.add_anchor("anchor1".to_string(), "value: 43".to_string(), 5);

        assert!(result.is_err());
        match result.unwrap_err() {
            YamlError::AnchorRedefinition {
                name,
                old_line,
                new_line,
            } => {
                assert_eq!(name, "anchor1");
                assert_eq!(old_line, 1);
                assert_eq!(new_line, 5);
            }
            _ => panic!("Expected AnchorRedefinition error"),
        }
    }

    #[test]
    fn test_forward_reference_error() {
        let mut registry = AnchorRegistry::new();

        let result = registry.add_alias("undefined", 3);

        assert!(result.is_err());
        match result.unwrap_err() {
            YamlError::ForwardReference { alias, line } => {
                assert_eq!(alias, "undefined");
                assert_eq!(line, 3);
            }
            _ => panic!("Expected ForwardReference error"),
        }
    }

    #[test]
    fn test_alias_after_definition_succeeds() {
        let mut registry = AnchorRegistry::new();

        registry
            .add_anchor("anchor1".to_string(), "value: 42".to_string(), 1)
            .unwrap();
        registry.add_alias("anchor1", 5).unwrap();

        assert!(registry.has_aliases());
    }

    #[test]
    fn test_cycle_detection_no_cycle() {
        let mut registry = AnchorRegistry::new();

        registry
            .add_anchor("a".to_string(), "value: 1".to_string(), 1)
            .unwrap();
        registry
            .add_anchor("b".to_string(), "value: 2".to_string(), 2)
            .unwrap();

        registry.add_alias("a", 3).unwrap();
        registry.add_dependency("b", "a");

        let result = detect_cycles(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cycle_detection_direct_self_reference() {
        let mut registry = AnchorRegistry::new();

        registry
            .add_anchor("self".to_string(), "child: *self".to_string(), 1)
            .unwrap();
        registry.add_alias("self", 1).unwrap();
        registry.add_dependency("self", "self");

        let result = detect_cycles(&registry);
        assert!(result.is_err());

        match result.unwrap_err() {
            YamlError::CircularReference { cycle_path, .. } => {
                assert!(cycle_path.contains("self -> self"));
            }
            _ => panic!("Expected CircularReference error"),
        }
    }

    #[test]
    fn test_cycle_detection_two_node_cycle() {
        let mut registry = AnchorRegistry::new();

        registry
            .add_anchor("a".to_string(), "ref: *b".to_string(), 1)
            .unwrap();
        registry
            .add_anchor("b".to_string(), "ref: *a".to_string(), 5)
            .unwrap();

        registry.add_alias("b", 1).unwrap();
        registry.add_alias("a", 5).unwrap();

        registry.add_dependency("a", "b");
        registry.add_dependency("b", "a");

        let result = detect_cycles(&registry);
        assert!(result.is_err());
    }

    #[test]
    fn test_cycle_detection_three_node_cycle() {
        let mut registry = AnchorRegistry::new();

        registry
            .add_anchor("a".to_string(), "next: *b".to_string(), 1)
            .unwrap();
        registry
            .add_anchor("b".to_string(), "next: *c".to_string(), 2)
            .unwrap();
        registry
            .add_anchor("c".to_string(), "next: *a".to_string(), 3)
            .unwrap();

        registry.add_alias("b", 1).unwrap();
        registry.add_alias("c", 2).unwrap();
        registry.add_alias("a", 3).unwrap();

        registry.add_dependency("a", "b");
        registry.add_dependency("b", "c");
        registry.add_dependency("c", "a");

        let result = detect_cycles(&registry);
        assert!(result.is_err());

        match result.unwrap_err() {
            YamlError::CircularReference { cycle_path, .. } => {
                // Should contain a three-node cycle
                assert!(
                    cycle_path.contains('a')
                        && cycle_path.contains('b')
                        && cycle_path.contains('c')
                );
            }
            _ => panic!("Expected CircularReference error"),
        }
    }

    #[test]
    fn test_validate_anchor_name_valid() {
        assert!(validate_anchor_name("anchor1").is_ok());
        assert!(validate_anchor_name("my_anchor").is_ok());
        assert!(validate_anchor_name("_anchor").is_ok());
    }

    #[test]
    fn test_validate_anchor_name_empty() {
        let result = validate_anchor_name("");
        assert!(result.is_err());

        match result.unwrap_err() {
            YamlError::InvalidAnchorName { name, reason } => {
                assert_eq!(name, "");
                assert!(reason.contains("empty"));
            }
            _ => panic!("Expected InvalidAnchorName error"),
        }
    }

    #[test]
    fn test_validate_anchor_name_reserved_prefix() {
        let result = validate_anchor_name("__reserved");
        assert!(result.is_err());

        match result.unwrap_err() {
            YamlError::InvalidAnchorName { name, reason } => {
                assert_eq!(name, "__reserved");
                assert!(reason.contains("reserved"));
            }
            _ => panic!("Expected InvalidAnchorName error"),
        }
    }

    #[test]
    fn test_diamond_pattern_no_cycle() {
        let mut registry = AnchorRegistry::new();

        // Diamond pattern: left and right both reference base
        registry
            .add_anchor("base".to_string(), "version: 1.0".to_string(), 1)
            .unwrap();
        registry
            .add_anchor("left".to_string(), "base: *base".to_string(), 2)
            .unwrap();
        registry
            .add_anchor("right".to_string(), "base: *base".to_string(), 3)
            .unwrap();

        registry.add_alias("base", 2).unwrap();
        registry.add_alias("base", 3).unwrap();

        registry.add_dependency("left", "base");
        registry.add_dependency("right", "base");

        let result = detect_cycles(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_definition_order_preserved() {
        let mut registry = AnchorRegistry::new();

        registry
            .add_anchor("third".to_string(), "value: 3".to_string(), 10)
            .unwrap();
        registry
            .add_anchor("first".to_string(), "value: 1".to_string(), 5)
            .unwrap();
        registry
            .add_anchor("second".to_string(), "value: 2".to_string(), 7)
            .unwrap();

        let names = registry.anchor_names();
        assert_eq!(names, &["third", "first", "second"]);
    }

    #[test]
    fn test_no_aliases_skips_cycle_detection() {
        let mut registry = AnchorRegistry::new();

        // Add anchors but no aliases
        registry
            .add_anchor("a".to_string(), "value: 1".to_string(), 1)
            .unwrap();
        registry
            .add_anchor("b".to_string(), "value: 2".to_string(), 2)
            .unwrap();

        // Even if we add dependencies, without aliases there's no point checking cycles
        registry.add_dependency("a", "b");

        assert!(!registry.has_aliases());

        let result = detect_cycles(&registry);
        assert!(result.is_ok());
    }
}
