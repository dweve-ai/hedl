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
}

impl TypeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            by_type: BTreeMap::new(),
            by_id: HashMap::new(),
        }
    }

    /// Register an ID in a type, maintaining both indices
    pub fn register(
        &mut self,
        type_name: &str,
        id: &str,
        line_num: usize,
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

        type_registry.insert(id.to_string(), line_num);

        // Update inverted index
        self.by_id
            .entry(id.to_string())
            .or_default()
            .push(type_name.to_string());

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
) -> HedlResult<()> {
    registries.register(type_name, id, line_num)
}

/// Resolve all references in a document using default limits.
pub fn resolve_references(doc: &Document, strict: bool) -> HedlResult<()> {
    resolve_references_with_limits(doc, strict, &Limits::default())
}

/// Resolve all references in a document with configurable limits.
pub fn resolve_references_with_limits(
    doc: &Document,
    strict: bool,
    limits: &Limits,
) -> HedlResult<()> {
    // Build type registries from document
    let mut registries = TypeRegistry::new();
    collect_node_ids(&doc.root, &mut registries, 0, limits.max_nest_depth)?;

    // Validate all references
    validate_references(&doc.root, &registries, strict, None, 0, limits.max_nest_depth)
}

fn collect_node_ids(
    items: &BTreeMap<String, Item>,
    registries: &mut TypeRegistry,
    depth: usize,
    max_depth: usize,
) -> HedlResult<()> {
    check_nest_depth(depth, max_depth)?;

    for item in items.values() {
        match item {
            Item::List(list) => {
                collect_list_ids(list, registries, depth, max_depth)?;
            }
            Item::Object(obj) => {
                collect_node_ids(obj, registries, depth + 1, max_depth)?;
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
    max_depth: usize,
) -> HedlResult<()> {
    // Collect IDs from this list
    for node in &list.rows {
        // Node IDs were already validated during parsing, just collect them
        registries.register(&list.type_name, &node.id, 0)?; // line 0 = already parsed
    }

    // Then recurse into children
    for node in &list.rows {
        for child_list in node.children.values() {
            for child in child_list {
                collect_list_ids_from_node(child, registries, depth + 1, max_depth)?;
            }
        }
    }

    Ok(())
}

fn collect_list_ids_from_node(
    node: &Node,
    registries: &mut TypeRegistry,
    depth: usize,
    max_depth: usize,
) -> HedlResult<()> {
    check_nest_depth(depth, max_depth)?;

    registries.register(&node.type_name, &node.id, 0)?;

    for child_list in node.children.values() {
        for child in child_list {
            collect_list_ids_from_node(child, registries, depth + 1, max_depth)?;
        }
    }

    Ok(())
}

fn validate_references(
    items: &BTreeMap<String, Item>,
    registries: &TypeRegistry,
    strict: bool,
    current_type: Option<&str>,
    depth: usize,
    max_depth: usize,
) -> HedlResult<()> {
    check_nest_depth(depth, max_depth)?;

    for item in items.values() {
        match item {
            Item::Scalar(value) => {
                validate_value_reference(value, registries, strict, current_type)?;
            }
            Item::List(list) => {
                for node in &list.rows {
                    validate_node_references(node, registries, strict, depth, max_depth)?;
                }
            }
            Item::Object(obj) => {
                validate_references(obj, registries, strict, current_type, depth + 1, max_depth)?;
            }
        }
    }
    Ok(())
}

fn validate_node_references(
    node: &Node,
    registries: &TypeRegistry,
    strict: bool,
    depth: usize,
    max_depth: usize,
) -> HedlResult<()> {
    check_nest_depth(depth, max_depth)?;

    for value in &node.fields {
        validate_value_reference(value, registries, strict, Some(&node.type_name))?;
    }

    for child_list in node.children.values() {
        for child in child_list {
            validate_node_references(child, registries, strict, depth + 1, max_depth)?;
        }
    }

    Ok(())
}

fn validate_value_reference(
    value: &Value,
    registries: &TypeRegistry,
    strict: bool,
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
                        let matching_types = registries
                            .lookup_unqualified(&ref_val.id)
                            .unwrap_or(&[]);

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

        if !resolved && strict {
            return Err(HedlError::reference(
                format!("unresolved reference {}", ref_val.to_ref_string()),
                0, // Line number lost at this point
            ));
        }
    }

    Ok(())
}
