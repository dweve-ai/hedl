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

//! Automatic FK detection and nesting
//!
//! These functions automatically detect foreign key relationships in flat JSON
//! and build nested hierarchies without manual configuration.

use super::config::JsonConversionError;
use hedl_core::lex::singularize_and_capitalize;
use hedl_core::{Document, Item, Node, Value};
use smallvec::SmallVec;
use std::collections::{BTreeMap, HashMap, HashSet};

/// Automatically detect foreign key relationships and build nested hierarchies.
///
/// This function scans all top-level collections looking for FK patterns:
/// - Field names like `{collection_singular}_id` (e.g., `device_id` → `devices`)
/// - Field names like `{collection}` that match another collection's singular (e.g., `device` → `devices`)
///
/// When FKs are detected, children are nested under their parents and removed from root.
///
/// # Algorithm
///
/// 1. Build index of all collections and their IDs
/// 2. For each collection, detect potential FK fields
/// 3. Topologically sort relationships (deepest children first)
/// 4. Apply nesting transformations
pub fn auto_nest_by_fk(mut doc: Document) -> Result<Document, JsonConversionError> {
    // Build index of collection names and their row IDs
    let mut collection_ids: HashMap<String, HashSet<String>> = HashMap::new();
    for (key, item) in &doc.root {
        if let Item::List(list) = item {
            // Clone necessary: building owned index for FK validation
            let ids: HashSet<String> = list.rows.iter().map(|r| r.id.clone()).collect();
            collection_ids.insert(key.clone(), ids);
        }
    }

    // Early exit: FK relationships require at least 2 collections
    if collection_ids.len() < 2 {
        return Ok(doc);
    }

    // Detect FK relationships: (parent_collection, child_collection, fk_field, fk_idx)
    let mut detected_fks: Vec<(String, String, String, usize)> = Vec::new();

    for (child_collection, item) in &doc.root {
        if let Item::List(list) = item {
            // Check each field in schema for FK patterns
            for (idx, field_name) in list.schema.iter().enumerate() {
                // Skip 'id' field itself
                if field_name == "id" {
                    continue;
                }

                // Try to find a matching parent collection
                let potential_parent =
                    detect_fk_target(field_name, &collection_ids, &list.rows, idx);

                if let Some(parent_collection) = potential_parent {
                    // Don't nest a collection under itself
                    if &parent_collection != child_collection {
                        // Clone necessary: FK tuple needs owned strings for sorting/processing
                        detected_fks.push((
                            parent_collection,
                            child_collection.clone(),
                            field_name.clone(),
                            idx,
                        ));
                    }
                }
            }
        }
    }

    // No FKs detected, return document as-is
    if detected_fks.is_empty() {
        return Ok(doc);
    }

    // Topologically sort: process leaf children first
    let sorted_fks = topological_sort_fks(&detected_fks);

    // Track which collections become children (remove from root later)
    let mut child_collections: HashSet<String> = HashSet::new();

    // Apply nesting for each FK relationship
    for (parent_collection, child_collection, fk_field, fk_idx) in sorted_fks {
        let parent_type_name = singularize_and_capitalize(&parent_collection);
        let child_type_name = singularize_and_capitalize(&child_collection);

        // Build set of valid parent IDs for quick lookup
        let parent_ids: HashSet<String> =
            if let Some(Item::List(parent_list)) = doc.root.get(&parent_collection) {
                parent_list.rows.iter().map(|r| r.id.clone()).collect()
            } else {
                HashSet::new()
            };

        // First pass: check if ALL children can be matched to parents
        // If any would be orphaned, skip this FK relationship entirely to preserve data
        let mut has_orphans = false;
        if let Some(Item::List(child_list)) = doc.root.get(&child_collection) {
            for row in &child_list.rows {
                if let Some(fk_value) = row.fields.get(fk_idx) {
                    let parent_id = match fk_value {
                        Value::String(s) => s.to_string(),
                        Value::Reference(r) => r.id.to_string(),
                        _ => {
                            has_orphans = true;
                            break;
                        }
                    };
                    if !parent_ids.contains(&parent_id) {
                        has_orphans = true;
                        break;
                    }
                } else {
                    has_orphans = true;
                    break;
                }
            }
        }

        // Skip this FK relationship if it would create orphans
        if has_orphans {
            continue;
        }

        child_collections.insert(child_collection.clone());

        // Group children by their parent ID (all will match since we verified above)
        let mut children_by_parent: HashMap<String, Vec<Node>> = HashMap::new();

        if let Some(Item::List(child_list)) = doc.root.get(&child_collection) {
            for row in &child_list.rows {
                if let Some(fk_value) = row.fields.get(fk_idx) {
                    let parent_id = match fk_value {
                        Value::String(s) => s.to_string(),
                        Value::Reference(r) => r.id.to_string(),
                        _ => continue,
                    };

                    // Create node without the FK field
                    let mut new_fields: SmallVec<[Value; 4]> = SmallVec::new();
                    for (i, field) in row.fields.iter().enumerate() {
                        if i != fk_idx {
                            new_fields.push(field.clone());
                        }
                    }

                    let child_node = Node {
                        type_name: row.type_name.clone(),
                        id: row.id.clone(),
                        fields: new_fields,
                        children: row.children.clone(),
                        child_count: row.child_count,
                    };

                    children_by_parent
                        .entry(parent_id)
                        .or_default()
                        .push(child_node);
                }
            }
        }

        // Add children to parent rows
        if let Some(Item::List(parent_list)) = doc.root.get_mut(&parent_collection) {
            for row in &mut parent_list.rows {
                if let Some(children) = children_by_parent.remove(&row.id) {
                    let child_count = children.len();
                    for child in children {
                        row.add_child(&child_type_name, child);
                    }
                    row.child_count =
                        (row.child_count as usize + child_count).min(u16::MAX as usize) as u16;
                }
            }
        }

        // Add NEST declaration
        doc.nests
            .entry(parent_type_name)
            .or_default()
            .push(child_type_name.clone());

        // Update struct schema to remove FK field
        if let Some(schema) = doc.structs.get_mut(&child_type_name) {
            schema.retain(|s| s != &fk_field);
        }
    }

    // Remove child collections from root
    doc.root.retain(|key, _| !child_collections.contains(key));

    Ok(doc)
}

/// Detect if a field is a foreign key and return the target collection name.
///
/// Checks patterns:
/// - `{collection_singular}_id` → `{collection}` (e.g., `device_id` → `devices`)
/// - `{collection_singular}` → `{collection}` (e.g., `device` → `devices`)
///
/// Also validates that FK values actually exist in the target collection.
fn detect_fk_target(
    field_name: &str,
    collection_ids: &HashMap<String, HashSet<String>>,
    rows: &[Node],
    field_idx: usize,
) -> Option<String> {
    // Pattern 1: field ends with `_id` (e.g., `customer_id`, `device_id`)
    if let Some(base) = field_name.strip_suffix("_id") {
        // Try plural forms
        for suffix in &["s", "es", "ies"] {
            let candidate = if base.ends_with('y') && *suffix == "ies" {
                format!("{}ies", &base[..base.len() - 1])
            } else {
                format!("{base}{suffix}")
            };

            if let Some(target_ids) = collection_ids.get(&candidate) {
                if validate_fk_values(rows, field_idx, target_ids) {
                    return Some(candidate);
                }
            }
        }

        // Try exact match (already plural)
        if let Some(target_ids) = collection_ids.get(base) {
            if validate_fk_values(rows, field_idx, target_ids) {
                return Some(base.to_string());
            }
        }
    }

    // Pattern 2: field name is singular form of a collection
    // e.g., field `customer` might reference `customers` collection
    for suffix in &["s", "es", "ies"] {
        let candidate = if field_name.ends_with('y') && *suffix == "ies" {
            format!("{}ies", &field_name[..field_name.len() - 1])
        } else {
            format!("{field_name}{suffix}")
        };

        if let Some(target_ids) = collection_ids.get(&candidate) {
            if validate_fk_values(rows, field_idx, target_ids) {
                return Some(candidate);
            }
        }
    }

    None
}

/// Validate that FK values in rows actually reference IDs in the target collection.
///
/// Returns true if at least 50% of non-null FK values exist in target_ids.
/// This threshold allows for some data inconsistency while avoiding false positives.
///
/// Optimized with early exit: if we've seen enough mismatches that 50% is impossible,
/// or if field values aren't string-like, we return false immediately.
fn validate_fk_values(rows: &[Node], field_idx: usize, target_ids: &HashSet<String>) -> bool {
    if rows.is_empty() || target_ids.is_empty() {
        return false;
    }

    // Sample first few rows to check if field contains string/reference values
    // Most non-FK fields will fail this check quickly
    let sample_size = rows.len().min(5);
    let mut string_like_count = 0;
    for row in rows.iter().take(sample_size) {
        if let Some(value) = row.fields.get(field_idx) {
            match value {
                Value::String(_) | Value::Reference(_) => string_like_count += 1,
                Value::Null => {}  // Nulls are ok, skip
                _ => return false, // Non-string/ref value found, not an FK field
            }
        }
    }
    // If no string-like values in sample, not an FK field
    if string_like_count == 0 && sample_size > 0 {
        return false;
    }

    let mut valid_count = 0;
    let mut invalid_count = 0;
    let total_rows = rows.len();

    for row in rows {
        if let Some(value) = row.fields.get(field_idx) {
            let fk_str: Option<&str> = match value {
                Value::String(s) => Some(&**s),
                Value::Reference(r) => Some(&*r.id),
                Value::Null => continue, // Skip nulls
                _ => return false,       // Non-string type in FK field
            };

            if let Some(fk) = fk_str {
                if target_ids.contains(fk) {
                    valid_count += 1;
                } else {
                    invalid_count += 1;
                    // Early exit: if invalid > total/2, can't reach 50%
                    if invalid_count > total_rows / 2 {
                        return false;
                    }
                }
            }
        }
    }

    let total_count = valid_count + invalid_count;
    // Require at least 50% match rate with minimum 1 match
    total_count > 0 && valid_count > 0 && (valid_count * 2 >= total_count)
}

/// Topologically sort FK relationships so deepest children are processed first.
fn topological_sort_fks(
    fks: &[(String, String, String, usize)],
) -> Vec<(String, String, String, usize)> {
    // Collections that are parents in some relationship
    let parent_collections: HashSet<&str> = fks.iter().map(|(p, _, _, _)| p.as_str()).collect();

    // Separate leaf rules (child is never a parent) from intermediate rules
    // Pre-allocate to avoid reallocation
    let mut leaf_rules: Vec<(String, String, String, usize)> = Vec::with_capacity(fks.len());
    let mut intermediate_rules: Vec<(String, String, String, usize)> =
        Vec::with_capacity(fks.len());

    for tuple in fks {
        let (parent, child, fk, idx) = tuple;
        if parent_collections.contains(child.as_str()) {
            // Clone necessary: building owned intermediate collection
            intermediate_rules.push((parent.clone(), child.clone(), fk.clone(), *idx));
        } else {
            // Clone necessary: building owned leaf collection
            leaf_rules.push((parent.clone(), child.clone(), fk.clone(), *idx));
        }
    }

    let mut result = leaf_rules;
    let mut processed: HashSet<String> = result.iter().map(|(_, c, _, _)| c.clone()).collect();

    // Iteratively add rules whose children have been processed
    while !intermediate_rules.is_empty() {
        let mut moved = false;
        intermediate_rules.retain(|(parent, child, fk, idx)| {
            let ready = fks
                .iter()
                .filter(|(p, _, _, _)| p == child)
                .all(|(_, c, _, _)| processed.contains(c.as_str()));

            if ready {
                // Clone necessary: moving from intermediate to result
                result.push((parent.clone(), child.clone(), fk.clone(), *idx));
                processed.insert(child.clone());
                moved = true;
                false
            } else {
                true
            }
        });

        if !moved {
            result.append(&mut intermediate_rules);
            break;
        }
    }

    result
}

/// Infer NEST declarations from existing children in the document.
///
/// When JSON is already nested (children embedded in parent objects), the converter
/// creates the proper Node.children structure but doesn't populate doc.nests.
/// This function scans the document to discover all parent-child relationships
/// and adds the corresponding NEST declarations.
pub fn infer_nests_from_children(doc: &mut Document) {
    // Collect all parent→child type relationships found in the document
    fn collect_nests_from_nodes(nodes: &[Node], nests: &mut BTreeMap<String, Vec<String>>) {
        for node in nodes {
            if let Some(children) = node.children() {
                for (child_type_name, child_nodes) in children.iter() {
                    // Add this relationship
                    let parent_type = node.type_name.clone();
                    let entry = nests.entry(parent_type).or_default();
                    if !entry.contains(child_type_name) {
                        entry.push(child_type_name.clone());
                    }

                    // Recurse into grandchildren
                    collect_nests_from_nodes(child_nodes, nests);
                }
            }
        }
    }

    // Scan all root items for nested children
    let mut inferred_nests: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for item in doc.root.values() {
        if let Item::List(list) = item {
            collect_nests_from_nodes(&list.rows, &mut inferred_nests);
        }
    }

    // Merge inferred nests into doc.nests (don't overwrite existing)
    for (parent, children) in inferred_nests {
        let entry = doc.nests.entry(parent).or_default();
        for child in children {
            if !entry.contains(&child) {
                entry.push(child);
            }
        }
    }
}
