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

//! Reference resolution for HEDL.

use crate::document::{Document, Item, MatrixList, Node};
use crate::error::{HedlError, HedlResult};
use crate::limits::Limits;
use crate::value::Value;
use std::collections::{BTreeMap, HashMap};

/// Reference resolution mode for controlling validation behavior.
///
/// Determines how the reference resolver handles unresolved or problematic references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReferenceMode {
    /// Strict mode: Unresolved references cause errors.
    ///
    /// This is the default and recommended mode for production parsing.
    /// Any reference that cannot be resolved will result in a parse error.
    ///
    /// # Behavior
    /// - Unresolved references → Error
    /// - Ambiguous references → Error (always, regardless of mode)
    #[default]
    Strict,

    /// Lenient mode: Unresolved references are ignored.
    ///
    /// Useful for partial parsing, work-in-progress documents, or when
    /// reference validation is deferred to a separate validation pass.
    ///
    /// # Behavior
    /// - Unresolved references → Silently ignored
    /// - Ambiguous references → Error (always, regardless of mode)
    ///
    /// # Use Cases
    /// - Parsing incomplete documents during development
    /// - Incremental parsing where not all nodes are loaded
    /// - Custom validation workflows
    Lenient,
}

impl ReferenceMode {
    /// Returns true if this mode should fail on unresolved references.
    #[inline]
    pub fn is_strict(self) -> bool {
        matches!(self, ReferenceMode::Strict)
    }

    /// Returns true if this mode allows unresolved references.
    #[inline]
    pub fn is_lenient(self) -> bool {
        matches!(self, ReferenceMode::Lenient)
    }
}

impl From<bool> for ReferenceMode {
    /// Convert from legacy boolean parameter.
    ///
    /// `true` → `Strict`, `false` → `Lenient`
    fn from(strict: bool) -> Self {
        if strict {
            ReferenceMode::Strict
        } else {
            ReferenceMode::Lenient
        }
    }
}

/// Type registries with both forward and inverted indices for efficient lookups.
///
/// P0 OPTIMIZATION: Inverted index for unqualified references (100-1000x speedup)
/// - Forward index: type -> (id -> line_num) for qualified lookups (O(log n))
/// - Inverted index: id -> [types] for unqualified lookups (O(1))
pub struct TypeRegistry {
    /// Forward index: type_name -> (id -> line_number)
    by_type: BTreeMap<String, BTreeMap<String, usize>>,
    /// Inverted index: id -> list of type names containing that ID
    by_id: HashMap<String, Vec<String>>,
    /// Total number of IDs registered across all types (for limit enforcement)
    total_ids: usize,
}

impl TypeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            by_type: BTreeMap::new(),
            by_id: HashMap::new(),
            total_ids: 0,
        }
    }

    /// Register an ID in a type, maintaining both indices
    pub fn register(
        &mut self,
        type_name: &str,
        id: &str,
        line_num: usize,
        limits: &Limits,
    ) -> HedlResult<()> {
        let type_registry = self.by_type.entry(type_name.to_string()).or_default();

        if let Some(&prev_line) = type_registry.get(id) {
            return Err(HedlError::collision(
                format!(
                    "duplicate ID '{}' in type '{}', previously defined at line {}",
                    id, type_name, prev_line
                ),
                line_num,
            ));
        }

        // Check total IDs limit before registration
        if self.total_ids >= limits.max_total_ids {
            return Err(HedlError::security(
                format!(
                    "total ID registrations {} exceeds limit {}",
                    self.total_ids, limits.max_total_ids
                ),
                line_num,
            ));
        }

        type_registry.insert(id.to_string(), line_num);

        // Update inverted index
        self.by_id
            .entry(id.to_string())
            .or_default()
            .push(type_name.to_string());

        // Increment total count
        self.total_ids += 1;

        Ok(())
    }

    /// Look up ID in a specific type (qualified reference)
    pub fn contains_in_type(&self, type_name: &str, id: &str) -> bool {
        self.by_type
            .get(type_name)
            .map(|r| r.contains_key(id))
            .unwrap_or(false)
    }

    /// Look up ID across all types (unqualified reference)
    /// Returns list of types containing this ID
    pub fn lookup_unqualified(&self, id: &str) -> Option<&[String]> {
        self.by_id.get(id).map(|v| v.as_slice())
    }

    /// Iterate over all IDs and their associated types in the inverted index.
    ///
    /// Used for merging registries during parallel parsing.
    pub fn by_id_iter(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.by_id.iter()
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Check NEST hierarchy depth against security limit.
///
/// Returns an error if the depth exceeds the maximum allowed depth.
fn check_nest_depth(depth: usize, max_depth: usize) -> HedlResult<()> {
    if depth > max_depth {
        return Err(HedlError::security(
            format!(
                "NEST hierarchy depth {} exceeds maximum allowed depth {}",
                depth, max_depth
            ),
            0,
        ));
    }
    Ok(())
}

/// Register a node ID, checking for collisions.
pub fn register_node(
    registries: &mut TypeRegistry,
    type_name: &str,
    id: &str,
    line_num: usize,
    limits: &Limits,
) -> HedlResult<()> {
    registries.register(type_name, id, line_num, limits)
}

/// Resolve all references in a document using default limits.
///
/// Validates that all references point to existing nodes according to the
/// specified reference resolution mode.
///
/// # Arguments
/// - `doc`: The document to validate
/// - `mode`: Reference resolution mode (strict or lenient)
///
/// # Errors
/// - In strict mode: Returns error if any reference cannot be resolved
/// - In any mode: Returns error if any reference is ambiguous
/// - Returns error if nesting depth exceeds limits
///
/// # Examples
/// ```
/// use hedl_core::{Document, ReferenceMode, resolve_references};
///
/// let doc = Document::new((1, 0));
/// // Strict mode - fail on unresolved references
/// resolve_references(&doc, ReferenceMode::Strict)?;
///
/// // Lenient mode - ignore unresolved references
/// resolve_references(&doc, ReferenceMode::Lenient)?;
/// # Ok::<(), hedl_core::HedlError>(())
/// ```
pub fn resolve_references(doc: &Document, mode: ReferenceMode) -> HedlResult<()> {
    resolve_references_with_limits(doc, mode, &Limits::default())
}

/// Resolve all references in a document with configurable limits.
///
/// Validates that all references point to existing nodes according to the
/// specified reference resolution mode, using custom security limits.
///
/// # Arguments
/// - `doc`: The document to validate
/// - `mode`: Reference resolution mode (strict or lenient)
/// - `limits`: Security limits for parsing
///
/// # Errors
/// - In strict mode: Returns error if any reference cannot be resolved
/// - In any mode: Returns error if any reference is ambiguous
/// - Returns error if nesting depth exceeds limits
pub fn resolve_references_with_limits(
    doc: &Document,
    mode: ReferenceMode,
    limits: &Limits,
) -> HedlResult<()> {
    // Build type registries from document
    let mut registries = TypeRegistry::new();
    collect_node_ids(&doc.root, &mut registries, 0, limits)?;

    // Validate all references
    validate_references(&doc.root, &registries, mode, None, 0, limits.max_nest_depth)
}

fn collect_node_ids(
    items: &BTreeMap<String, Item>,
    registries: &mut TypeRegistry,
    depth: usize,
    limits: &Limits,
) -> HedlResult<()> {
    check_nest_depth(depth, limits.max_nest_depth)?;

    for item in items.values() {
        match item {
            Item::List(list) => {
                collect_list_ids(list, registries, depth, limits)?;
            }
            Item::Object(obj) => {
                collect_node_ids(obj, registries, depth + 1, limits)?;
            }
            Item::Scalar(_) => {}
        }
    }
    Ok(())
}

fn collect_list_ids(
    list: &MatrixList,
    registries: &mut TypeRegistry,
    depth: usize,
    limits: &Limits,
) -> HedlResult<()> {
    // Collect IDs from this list
    for node in &list.rows {
        // Node IDs were already validated during parsing, just collect them
        registries.register(&list.type_name, &node.id, 0, limits)?; // line 0 = already parsed
    }

    // Then recurse into children
    for node in &list.rows {
        if let Some(children) = node.children() {
            for child_list in children.values() {
                for child in child_list {
                    collect_list_ids_from_node(child, registries, depth + 1, limits)?;
                }
            }
        }
    }

    Ok(())
}

fn collect_list_ids_from_node(
    node: &Node,
    registries: &mut TypeRegistry,
    depth: usize,
    limits: &Limits,
) -> HedlResult<()> {
    check_nest_depth(depth, limits.max_nest_depth)?;

    registries.register(&node.type_name, &node.id, 0, limits)?;

    if let Some(children) = node.children() {
        for child_list in children.values() {
            for child in child_list {
                collect_list_ids_from_node(child, registries, depth + 1, limits)?;
            }
        }
    }

    Ok(())
}

fn validate_references(
    items: &BTreeMap<String, Item>,
    registries: &TypeRegistry,
    mode: ReferenceMode,
    current_type: Option<&str>,
    depth: usize,
    max_depth: usize,
) -> HedlResult<()> {
    check_nest_depth(depth, max_depth)?;

    for item in items.values() {
        match item {
            Item::Scalar(value) => {
                validate_value_reference(value, registries, mode, current_type)?;
            }
            Item::List(list) => {
                for node in &list.rows {
                    validate_node_references(node, registries, mode, depth, max_depth)?;
                }
            }
            Item::Object(obj) => {
                validate_references(obj, registries, mode, current_type, depth + 1, max_depth)?;
            }
        }
    }
    Ok(())
}

fn validate_node_references(
    node: &Node,
    registries: &TypeRegistry,
    mode: ReferenceMode,
    depth: usize,
    max_depth: usize,
) -> HedlResult<()> {
    check_nest_depth(depth, max_depth)?;

    for value in &node.fields {
        validate_value_reference(value, registries, mode, Some(&node.type_name))?;
    }

    if let Some(children) = node.children() {
        for child_list in children.values() {
            for child in child_list {
                validate_node_references(child, registries, mode, depth + 1, max_depth)?;
            }
        }
    }

    Ok(())
}

fn validate_value_reference(
    value: &Value,
    registries: &TypeRegistry,
    mode: ReferenceMode,
    current_type: Option<&str>,
) -> HedlResult<()> {
    if let Value::Reference(ref_val) = value {
        // If reference has explicit type (@User:u1), look only in that type's registry
        let resolved = match &ref_val.type_name {
            Some(t) => registries.contains_in_type(t, &ref_val.id),
            None => {
                // No type qualifier - behavior depends on context
                match current_type {
                    // SPEC 10.2, 10.3: In matrix context, search ONLY current type
                    Some(type_name) => registries.contains_in_type(type_name, &ref_val.id),
                    // SPEC 10.3.1: In Key-Value context, search all types but detect ambiguity
                    // P0 OPTIMIZATION: Use inverted index for O(1) lookup instead of O(m) scan
                    None => {
                        let matching_types =
                            registries.lookup_unqualified(&ref_val.id).unwrap_or(&[]);

                        match matching_types.len() {
                            0 => false, // Not found
                            1 => true,  // Unambiguous match
                            _ => {
                                // Multiple matches - ambiguous reference
                                return Err(HedlError::reference(
                                    format!(
                                        "Ambiguous unqualified reference '@{}' matches multiple types: [{}]",
                                        ref_val.id,
                                        matching_types.join(", ")
                                    ),
                                    0, // Line number lost at this point
                                ));
                            }
                        }
                    }
                }
            }
        };

        if !resolved && mode.is_strict() {
            return Err(HedlError::reference(
                format!("unresolved reference {}", ref_val.to_ref_string()),
                0, // Line number lost at this point
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== max_total_ids limit tests ====================

    #[test]
    fn test_max_total_ids_limit() {
        let mut registry = TypeRegistry::new();
        let limits = Limits {
            max_total_ids: 3,
            ..Default::default()
        };

        // Register 3 IDs across different types (should succeed)
        assert!(registry.register("Type1", "id1", 1, &limits).is_ok());
        assert!(registry.register("Type2", "id2", 2, &limits).is_ok());
        assert!(registry.register("Type3", "id3", 3, &limits).is_ok());

        // 4th registration should fail
        let result = registry.register("Type4", "id4", 4, &limits);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("exceeds limit"),
            "Expected 'exceeds limit' in error message, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_max_total_ids_across_types() {
        let mut registry = TypeRegistry::new();
        let limits = Limits {
            max_total_ids: 10,
            ..Default::default()
        };

        // Register IDs in same type
        for i in 0..5 {
            assert!(registry
                .register("Type1", &format!("id{}", i), i, &limits)
                .is_ok());
        }

        // Register IDs in different type
        for i in 0..5 {
            assert!(registry
                .register("Type2", &format!("id{}", i), i + 5, &limits)
                .is_ok());
        }

        // 11th registration should fail
        let result = registry.register("Type3", "id_extra", 10, &limits);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("exceeds limit"),
            "Expected 'exceeds limit' in error message, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_unlimited_ids() {
        let mut registry = TypeRegistry::new();
        let limits = Limits::unlimited();

        // Should be able to register many IDs
        for i in 0..10000 {
            let result =
                registry.register(&format!("Type{}", i % 100), &format!("id{}", i), i, &limits);
            assert!(
                result.is_ok(),
                "Failed to register ID {} in unlimited mode",
                i
            );
        }
    }

    #[test]
    fn test_collision_detection_with_limits() {
        let mut registry = TypeRegistry::new();
        let limits = Limits::default();

        assert!(registry.register("Type1", "id1", 1, &limits).is_ok());

        // Duplicate in same type should fail with collision error, not limit error
        let result = registry.register("Type1", "id1", 2, &limits);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("duplicate"),
            "Expected 'duplicate' in error message, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_max_total_ids_exact_limit() {
        let mut registry = TypeRegistry::new();
        let limits = Limits {
            max_total_ids: 5,
            ..Default::default()
        };

        // Register exactly at limit (should succeed)
        for i in 0..5 {
            let result = registry.register("Type", &format!("id{}", i), i, &limits);
            assert!(result.is_ok(), "Failed to register ID {} at exact limit", i);
        }

        // One more should fail
        let result = registry.register("Type", "id5", 5, &limits);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("exceeds limit"),
            "Expected 'exceeds limit' in error message, got: {}",
            err_msg
        );
    }

    #[test]
    fn test_max_total_ids_just_under_limit() {
        let mut registry = TypeRegistry::new();
        let limits = Limits {
            max_total_ids: 5,
            ..Default::default()
        };

        // Register just under limit (should succeed)
        for i in 0..4 {
            assert!(registry
                .register("Type", &format!("id{}", i), i, &limits)
                .is_ok());
        }

        // Still have room for one more
        assert!(registry.register("Type", "id4", 4, &limits).is_ok());
    }

    #[test]
    fn test_max_total_ids_error_message_clarity() {
        let mut registry = TypeRegistry::new();
        let limits = Limits {
            max_total_ids: 2,
            ..Default::default()
        };

        registry.register("Type1", "id1", 1, &limits).unwrap();
        registry.register("Type2", "id2", 2, &limits).unwrap();

        let result = registry.register("Type3", "id3", 3, &limits);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("2"),
            "Error message should contain the count"
        );
        assert!(
            err_msg.contains("limit"),
            "Error message should mention 'limit'"
        );
    }

    #[test]
    fn test_total_ids_count_tracking() {
        let mut registry = TypeRegistry::new();
        let limits = Limits::unlimited();

        assert_eq!(registry.total_ids, 0);

        registry.register("Type1", "id1", 1, &limits).unwrap();
        assert_eq!(registry.total_ids, 1);

        registry.register("Type2", "id2", 2, &limits).unwrap();
        assert_eq!(registry.total_ids, 2);

        registry.register("Type1", "id3", 3, &limits).unwrap();
        assert_eq!(registry.total_ids, 3);
    }

    #[test]
    fn test_max_total_ids_with_multiple_types() {
        let mut registry = TypeRegistry::new();
        let limits = Limits {
            max_total_ids: 100,
            ..Default::default()
        };

        // Distribute IDs across 10 types
        for type_idx in 0..10 {
            for id_idx in 0..10 {
                let result = registry.register(
                    &format!("Type{}", type_idx),
                    &format!("id{}_{}", type_idx, id_idx),
                    type_idx * 10 + id_idx,
                    &limits,
                );
                assert!(result.is_ok(), "Failed at type {} id {}", type_idx, id_idx);
            }
        }

        // Now we're at limit (100), next should fail
        let result = registry.register("TypeExtra", "extra", 100, &limits);
        assert!(result.is_err());
    }

    #[test]
    fn test_collision_preserves_total_count() {
        let mut registry = TypeRegistry::new();
        let limits = Limits::unlimited();

        registry.register("Type1", "id1", 1, &limits).unwrap();
        assert_eq!(registry.total_ids, 1);

        // Attempt duplicate registration (should fail)
        let result = registry.register("Type1", "id1", 2, &limits);
        assert!(result.is_err());

        // Total count should not have changed
        assert_eq!(registry.total_ids, 1);
    }

    #[test]
    fn test_default_limits_max_total_ids() {
        let limits = Limits::default();
        assert_eq!(limits.max_total_ids, 10_000_000);
    }

    #[test]
    fn test_unlimited_limits_max_total_ids() {
        let limits = Limits::unlimited();
        assert_eq!(limits.max_total_ids, usize::MAX);
    }

    // ==================== TypeRegistry basic functionality tests ====================

    #[test]
    fn test_registry_new() {
        let registry = TypeRegistry::new();
        assert_eq!(registry.total_ids, 0);
        assert!(registry.by_type.is_empty());
        assert!(registry.by_id.is_empty());
    }

    #[test]
    fn test_registry_default() {
        let registry = TypeRegistry::default();
        assert_eq!(registry.total_ids, 0);
    }

    #[test]
    fn test_contains_in_type() {
        let mut registry = TypeRegistry::new();
        let limits = Limits::unlimited();

        registry.register("User", "u1", 1, &limits).unwrap();
        assert!(registry.contains_in_type("User", "u1"));
        assert!(!registry.contains_in_type("User", "u2"));
        assert!(!registry.contains_in_type("Post", "u1"));
    }

    #[test]
    fn test_lookup_unqualified() {
        let mut registry = TypeRegistry::new();
        let limits = Limits::unlimited();

        registry.register("User", "id1", 1, &limits).unwrap();
        registry.register("Post", "id1", 2, &limits).unwrap();

        let types = registry.lookup_unqualified("id1");
        assert!(types.is_some());
        let types = types.unwrap();
        assert_eq!(types.len(), 2);
        assert!(types.contains(&"User".to_string()));
        assert!(types.contains(&"Post".to_string()));

        let not_found = registry.lookup_unqualified("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_inverted_index_maintenance() {
        let mut registry = TypeRegistry::new();
        let limits = Limits::unlimited();

        // Same ID in multiple types should appear in inverted index
        registry.register("Type1", "shared_id", 1, &limits).unwrap();
        registry.register("Type2", "shared_id", 2, &limits).unwrap();
        registry.register("Type3", "shared_id", 3, &limits).unwrap();

        let types = registry.lookup_unqualified("shared_id").unwrap();
        assert_eq!(types.len(), 3);
    }

    // ==================== ReferenceMode tests ====================

    #[test]
    fn test_reference_mode_default() {
        assert_eq!(ReferenceMode::default(), ReferenceMode::Strict);
    }

    #[test]
    fn test_reference_mode_from_bool() {
        assert_eq!(ReferenceMode::from(true), ReferenceMode::Strict);
        assert_eq!(ReferenceMode::from(false), ReferenceMode::Lenient);
    }

    #[test]
    fn test_reference_mode_is_strict() {
        assert!(ReferenceMode::Strict.is_strict());
        assert!(!ReferenceMode::Lenient.is_strict());
    }

    #[test]
    fn test_reference_mode_is_lenient() {
        assert!(ReferenceMode::Lenient.is_lenient());
        assert!(!ReferenceMode::Strict.is_lenient());
    }

    #[test]
    fn test_reference_mode_equality() {
        assert_eq!(ReferenceMode::Strict, ReferenceMode::Strict);
        assert_eq!(ReferenceMode::Lenient, ReferenceMode::Lenient);
        assert_ne!(ReferenceMode::Strict, ReferenceMode::Lenient);
    }
}
