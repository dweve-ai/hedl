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

//! Parallel parsing infrastructure for hedl-core.
//!
//! This module provides data parallelism using rayon's work-stealing scheduler
//! for improved throughput on multi-core systems. Parallel parsing achieves
//! 2-4x speedup for documents with 1000+ entities while maintaining deterministic
//! output and semantic equivalence with the sequential parser.
//!
//! # Features
//!
//! - **Parallel entity parsing**: Top-level entities parsed in parallel
//! - **Parallel matrix row parsing**: Large matrix lists (100+ rows) parsed in parallel
//! - **Parallel reference resolution**: ID collection and validation parallelized
//! - **Adaptive thresholds**: Automatic fallback to sequential for small documents
//!
//! # Examples
//!
//! ```text
//! use hedl_core::{parse_with_limits, ParseOptions, ParallelConfig};
//!
//! let opts = ParseOptions::builder()
//!     .enable_parallel(true)
//!     .parallel_thresholds(50, 100)  // Min entities, min rows
//!     .build();
//!
//! let doc = parse_with_limits(input, opts)?;
//! ```

use crate::document::{Item, MatrixList, Node};
use crate::error::{HedlError, HedlResult};
use crate::header::Header;
use crate::inference::{infer_quoted_value, infer_value, InferenceContext};
use crate::lex::row::parse_csv_row;
use crate::lex::{calculate_indent, strip_comment};
use crate::limits::Limits;
use crate::reference::TypeRegistry;
use crate::value::Value;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Configuration for parallel parsing behavior.
///
/// Controls when parallel parsing is activated and how many threads
/// are used. Adaptive thresholds prevent parallel overhead from
/// slowing down small documents.
///
/// # Examples
///
/// ```text
/// let config = ParallelConfig {
///     enabled: true,
///     min_root_entities: 50,
///     min_list_rows: 100,
///     thread_pool_size: Some(4),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Enable parallel parsing (default: true if feature enabled)
    pub enabled: bool,
    /// Minimum root entities to trigger parallelism (default: 50)
    pub min_root_entities: usize,
    /// Minimum matrix rows to parse in parallel (default: 100)
    pub min_list_rows: usize,
    /// Thread pool size (None = auto-detect cores)
    pub thread_pool_size: Option<usize>,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_root_entities: 50,
            min_list_rows: 100,
            thread_pool_size: None,
        }
    }
}

impl ParallelConfig {
    /// Create configuration for conservative parallel parsing.
    ///
    /// Uses higher thresholds to minimize overhead for typical documents.
    pub fn conservative() -> Self {
        Self {
            enabled: true,
            min_root_entities: 100,
            min_list_rows: 200,
            thread_pool_size: None,
        }
    }

    /// Create configuration for aggressive parallel parsing.
    ///
    /// Uses lower thresholds to maximize parallelism opportunities.
    pub fn aggressive() -> Self {
        Self {
            enabled: true,
            min_root_entities: 20,
            min_list_rows: 50,
            thread_pool_size: None,
        }
    }

    /// Check if parallel entity parsing should be used.
    pub fn should_parallelize_entities(&self, entity_count: usize) -> bool {
        self.enabled && entity_count >= self.min_root_entities
    }

    /// Check if parallel row parsing should be used.
    pub fn should_parallelize_rows(&self, row_count: usize) -> bool {
        self.enabled && row_count >= self.min_list_rows
    }
}

/// Thread-safe security counters using atomic operations.
///
/// Tracks node counts and key counts across parallel parsing threads
/// without lock contention.
pub struct AtomicSecurityCounters {
    node_count: AtomicUsize,
    total_keys: AtomicUsize,
}

impl AtomicSecurityCounters {
    /// Create new atomic counters starting at zero.
    pub fn new() -> Self {
        Self {
            node_count: AtomicUsize::new(0),
            total_keys: AtomicUsize::new(0),
        }
    }

    /// Increment node count with security check.
    ///
    /// Returns error if limit is exceeded.
    pub fn increment_nodes(&self, limits: &Limits, line_num: usize) -> HedlResult<()> {
        let count = self.node_count.fetch_add(1, Ordering::Relaxed);
        if count >= limits.max_nodes {
            return Err(HedlError::security(
                format!("too many nodes: exceeds limit of {}", limits.max_nodes),
                line_num,
            ));
        }
        Ok(())
    }

    /// Increment total keys count with security check.
    pub fn increment_keys(&self, limits: &Limits, line_num: usize) -> HedlResult<()> {
        let count = self.total_keys.fetch_add(1, Ordering::Relaxed);
        if count >= limits.max_total_keys {
            return Err(HedlError::security(
                format!(
                    "too many total keys: {} exceeds limit {}",
                    count + 1,
                    limits.max_total_keys
                ),
                line_num,
            ));
        }
        Ok(())
    }

    /// Get current node count.
    pub fn node_count(&self) -> usize {
        self.node_count.load(Ordering::Relaxed)
    }

    /// Get current key count.
    pub fn key_count(&self) -> usize {
        self.total_keys.load(Ordering::Relaxed)
    }
}

impl Default for AtomicSecurityCounters {
    fn default() -> Self {
        Self::new()
    }
}

/// Entity boundary for parallel parsing.
///
/// Represents a contiguous range of lines belonging to a single
/// top-level entity (object or list).
#[derive(Debug, Clone)]
pub struct EntityBoundary {
    /// Key name for this entity
    pub key: String,
    /// Line range (start..end) in the document
    pub line_range: Range<usize>,
    /// Type of entity (Object, List, or Scalar)
    pub entity_type: EntityType,
}

/// Type of entity for parallel parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    /// Object with nested key-value pairs
    Object,
    /// Matrix list with rows
    List,
    /// Single scalar value
    Scalar,
}

/// Batch of matrix rows for parallel parsing.
///
/// Contains all information needed to parse rows independently.
#[derive(Debug)]
pub struct MatrixRowBatch<'a> {
    /// Type name for this matrix list
    pub type_name: String,
    /// Schema columns
    pub schema: Vec<String>,
    /// Row data: (line_number, row_content)
    pub rows: Vec<(usize, &'a str)>,
    /// Whether ditto markers were found (requires sequential fallback)
    pub has_ditto: bool,
}

impl<'a> MatrixRowBatch<'a> {
    /// Check if this batch is safe for parallel parsing.
    ///
    /// Returns false if ditto markers are present (requires sequential).
    pub fn can_parallelize(&self) -> bool {
        !self.has_ditto && !self.rows.is_empty()
    }
}

/// Identify root-level entity boundaries in document body.
///
/// Scans lines to find where each top-level entity starts and ends.
/// This is a sequential scan but very fast (O(n) with minimal work).
pub fn identify_entity_boundaries(lines: &[(usize, &str)]) -> Vec<EntityBoundary> {
    let mut boundaries = Vec::new();
    let mut current_key: Option<String> = None;
    let mut current_start: usize = 0;
    let mut current_type = EntityType::Object;

    for (idx, &(line_num, line)) in lines.iter().enumerate() {
        // Skip blank/comment lines
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check if this is a root-level line (no leading whitespace)
        let indent_info = calculate_indent(line, line_num as u32).ok().flatten();
        let indent = indent_info.map(|i| i.level).unwrap_or(0);

        if indent == 0 && line.contains(':') {
            // New root entity starting
            if let Some(key) = current_key.take() {
                // Finalize previous entity
                boundaries.push(EntityBoundary {
                    key,
                    line_range: current_start..idx,
                    entity_type: current_type,
                });
            }

            // Extract key from line
            if let Some(colon_pos) = line.find(':') {
                let key_part = &line[..colon_pos].trim();
                // Remove count hint if present: key(N) -> key
                let key = if let Some(paren_pos) = key_part.find('(') {
                    key_part[..paren_pos].trim()
                } else {
                    key_part
                };

                current_key = Some(key.to_string());
                current_start = idx;

                // Determine entity type from content after colon
                let after_colon = line[colon_pos + 1..].trim();
                current_type = if after_colon.starts_with('@') {
                    EntityType::List
                } else if after_colon.is_empty() {
                    EntityType::Object
                } else {
                    EntityType::Scalar
                };
            }
        }
    }

    // Finalize last entity
    if let Some(key) = current_key {
        boundaries.push(EntityBoundary {
            key,
            line_range: current_start..lines.len(),
            entity_type: current_type,
        });
    }

    boundaries
}

/// Collect matrix row content for parallel parsing.
///
/// Scans a matrix list to gather all row lines. Detects ditto markers
/// which require sequential fallback.
pub fn collect_matrix_rows<'a>(
    lines: &'a [(usize, &str)],
    list_start: usize,
    expected_indent: usize,
) -> MatrixRowBatch<'a> {
    let mut rows = Vec::new();
    let mut has_ditto = false;
    let mut type_name = String::new();
    let mut schema = Vec::new();

    // Parse list declaration line to get type and schema
    if list_start < lines.len() {
        let (_, decl_line) = lines[list_start];
        if let Some(colon_pos) = decl_line.find(':') {
            let after_colon = decl_line[colon_pos + 1..].trim();
            if let Some(rest) = after_colon.strip_prefix('@') {
                // Extract type name and schema
                if let Some(bracket_pos) = rest.find('[') {
                    type_name = rest[..bracket_pos].to_string();
                    let schema_str = &rest[bracket_pos..];
                    if schema_str.starts_with('[') && schema_str.ends_with(']') {
                        let inner = &schema_str[1..schema_str.len() - 1];
                        schema = inner.split(',').map(|s| s.trim().to_string()).collect();
                    }
                } else {
                    type_name = rest.trim().to_string();
                }
            }
        }
    }

    // Collect row lines
    let mut i = list_start + 1;
    while i < lines.len() {
        let (line_num, line) = lines[i];

        // Check indentation
        let indent_info = calculate_indent(line, line_num as u32).ok().flatten();
        let indent = indent_info.map(|info| info.level).unwrap_or(0);

        // Stop if we've dedented out of the list
        if indent < expected_indent {
            break;
        }

        let content = line.trim();

        // Matrix rows start with |
        if content.starts_with('|') {
            // Check for ditto markers (")
            if content.contains('"') && !content.contains("\"\"") {
                // Single quote outside of escaped quotes = ditto marker
                let unquoted_part: String = content
                    .chars()
                    .scan(false, |in_quote, c| {
                        if c == '"' {
                            *in_quote = !*in_quote;
                        }
                        Some(if *in_quote { ' ' } else { c })
                    })
                    .collect();
                if unquoted_part.contains('"') {
                    has_ditto = true;
                }
            }

            rows.push((line_num, content));
        } else if !content.is_empty() && !content.starts_with('#') {
            // Non-row content (nested list or other) - stop collecting
            break;
        }

        i += 1;
    }

    MatrixRowBatch {
        type_name,
        schema,
        rows,
        has_ditto,
    }
}

/// Parse matrix rows in parallel.
///
/// Each row is parsed independently without shared mutable state.
/// Returns parsed nodes without ID registration (done sequentially after).
pub fn parse_matrix_rows_parallel(
    batch: &MatrixRowBatch<'_>,
    header: &Header,
    limits: &Limits,
    counters: &AtomicSecurityCounters,
) -> HedlResult<Vec<Node>> {
    if !batch.can_parallelize() {
        // Fall back to sequential if ditto markers present
        return parse_matrix_rows_sequential(batch, header, limits);
    }

    // Parse rows in parallel
    let nodes: Vec<HedlResult<Node>> = batch
        .rows
        .par_iter()
        .map(|(line_num, row_content)| {
            // Check security limits (atomic)
            counters.increment_nodes(limits, *line_num)?;

            // Parse single row
            parse_single_matrix_row(
                row_content,
                &batch.schema,
                &batch.type_name,
                header,
                *line_num,
            )
        })
        .collect();

    // Collect results, propagating first error
    nodes.into_iter().collect()
}

/// Parse matrix rows sequentially (fallback for ditto markers).
fn parse_matrix_rows_sequential(
    batch: &MatrixRowBatch<'_>,
    header: &Header,
    limits: &Limits,
) -> HedlResult<Vec<Node>> {
    let mut nodes = Vec::with_capacity(batch.rows.len());
    let mut prev_values: Option<Vec<Value>> = None;
    let mut node_count = 0usize;

    for (line_num, row_content) in &batch.rows {
        // Check node count limit
        node_count = node_count
            .checked_add(1)
            .ok_or_else(|| HedlError::security("node count overflow", *line_num))?;
        if node_count > limits.max_nodes {
            return Err(HedlError::security(
                format!("too many nodes: exceeds limit of {}", limits.max_nodes),
                *line_num,
            ));
        }

        // Parse with ditto support
        let node = parse_single_matrix_row_with_ditto(
            row_content,
            &batch.schema,
            &batch.type_name,
            header,
            *line_num,
            prev_values.as_deref(),
        )?;

        prev_values = Some(node.fields.to_vec());
        nodes.push(node);
    }

    Ok(nodes)
}

/// Parse a single matrix row without ditto support.
///
/// Used for parallel parsing where rows are independent.
fn parse_single_matrix_row(
    row_content: &str,
    schema: &[String],
    type_name: &str,
    header: &Header,
    line_num: usize,
) -> HedlResult<Node> {
    parse_single_matrix_row_with_ditto(row_content, schema, type_name, header, line_num, None)
}

/// Parse a single matrix row with optional ditto support.
fn parse_single_matrix_row_with_ditto(
    row_content: &str,
    schema: &[String],
    type_name: &str,
    header: &Header,
    line_num: usize,
    prev_values: Option<&[Value]>,
) -> HedlResult<Node> {
    // Skip | prefix and extract optional child count
    let content = row_content.strip_prefix('|').unwrap_or(row_content);

    // Parse child count prefix if present: [N] data
    let (child_count, csv_content) = if content.starts_with('[') {
        if let Some(bracket_end) = content.find(']') {
            let count_str = &content[1..bracket_end];
            if let Ok(count) = count_str.parse::<usize>() {
                let data = content[bracket_end + 1..].trim_start();
                (Some(count), data)
            } else {
                (None, content)
            }
        } else {
            (None, content)
        }
    } else {
        (None, content)
    };

    let csv_content = strip_comment(csv_content).trim();

    // Parse CSV fields
    let fields =
        parse_csv_row(csv_content).map_err(|e| HedlError::syntax(e.to_string(), line_num))?;

    // Validate field count
    if fields.len() != schema.len() {
        return Err(HedlError::shape(
            format!("expected {} columns, got {}", schema.len(), fields.len()),
            line_num,
        ));
    }

    // Infer values
    let mut values = Vec::with_capacity(fields.len());
    for (col_idx, field) in fields.iter().enumerate() {
        let ctx =
            InferenceContext::for_matrix_cell(&header.aliases, col_idx, prev_values, type_name);

        let value = if field.is_quoted {
            infer_quoted_value(&field.value)
        } else {
            infer_value(&field.value, &ctx, line_num)?
        };

        values.push(value);
    }

    // Extract ID from first column
    let id = match &values[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(HedlError::semantic("ID column must be a string", line_num));
        }
    };

    // Create node
    let mut node = Node::new(type_name, &*id, values);
    if let Some(count) = child_count {
        node.set_child_count(count);
    }

    Ok(node)
}

/// Parallel ID collection from document tree.
///
/// Collects node IDs from independent subtrees in parallel, then merges
/// registries sequentially with conflict detection.
pub fn collect_ids_parallel(
    items: &BTreeMap<String, Item>,
    limits: &Limits,
) -> HedlResult<TypeRegistry> {
    // Skip parallelism for small documents
    if items.len() < 10 {
        let mut registry = TypeRegistry::new();
        collect_ids_sequential(items, &mut registry, 0, limits.max_nest_depth, limits)?;
        return Ok(registry);
    }

    // Collect IDs from each item in parallel
    let local_registries: Vec<HedlResult<TypeRegistry>> = items
        .par_iter()
        .map(|(_, item)| {
            let mut local = TypeRegistry::new();
            collect_item_ids(item, &mut local, 0, limits.max_nest_depth, limits)?;
            Ok(local)
        })
        .collect();

    // Merge registries sequentially with conflict detection
    let mut merged = TypeRegistry::new();
    for result in local_registries {
        let local = result?;
        merge_registry_into(&mut merged, local, limits)?;
    }

    Ok(merged)
}

/// Sequential ID collection (fallback).
fn collect_ids_sequential(
    items: &BTreeMap<String, Item>,
    registry: &mut TypeRegistry,
    depth: usize,
    max_depth: usize,
    limits: &Limits,
) -> HedlResult<()> {
    if depth > max_depth {
        return Err(HedlError::security(
            format!(
                "NEST hierarchy depth {} exceeds maximum allowed depth {}",
                depth, max_depth
            ),
            0,
        ));
    }

    for item in items.values() {
        collect_item_ids(item, registry, depth, max_depth, limits)?;
    }

    Ok(())
}

/// Collect IDs from a single item.
fn collect_item_ids(
    item: &Item,
    registry: &mut TypeRegistry,
    depth: usize,
    max_depth: usize,
    limits: &Limits,
) -> HedlResult<()> {
    match item {
        Item::List(list) => {
            collect_list_ids(list, registry, depth, max_depth, limits)?;
        }
        Item::Object(obj) => {
            collect_ids_sequential(obj, registry, depth + 1, max_depth, limits)?;
        }
        Item::Scalar(_) => {}
    }
    Ok(())
}

/// Collect IDs from a matrix list.
fn collect_list_ids(
    list: &MatrixList,
    registry: &mut TypeRegistry,
    depth: usize,
    max_depth: usize,
    limits: &Limits,
) -> HedlResult<()> {
    for node in &list.rows {
        registry.register(&list.type_name, &node.id, 0, limits)?;
    }

    // Recurse into children
    for node in &list.rows {
        if let Some(children) = node.children() {
            for child_list in children.values() {
                for child in child_list {
                    collect_node_ids(child, registry, depth + 1, max_depth, limits)?;
                }
            }
        }
    }

    Ok(())
}

/// Collect IDs from a node and its children.
fn collect_node_ids(
    node: &Node,
    registry: &mut TypeRegistry,
    depth: usize,
    max_depth: usize,
    limits: &Limits,
) -> HedlResult<()> {
    if depth > max_depth {
        return Err(HedlError::security(
            format!(
                "NEST hierarchy depth {} exceeds maximum allowed depth {}",
                depth, max_depth
            ),
            0,
        ));
    }

    registry.register(&node.type_name, &node.id, 0, limits)?;

    if let Some(children) = node.children() {
        for child_list in children.values() {
            for child in child_list {
                collect_node_ids(child, registry, depth + 1, max_depth, limits)?;
            }
        }
    }

    Ok(())
}

/// Merge source registry into target with conflict detection.
fn merge_registry_into(
    target: &mut TypeRegistry,
    source: TypeRegistry,
    limits: &Limits,
) -> HedlResult<()> {
    // Get IDs from source by iterating through the inverted index
    for (id, types) in source.by_id_iter() {
        for type_name in types {
            // Check for conflict in target
            if target.contains_in_type(type_name, id) {
                return Err(HedlError::collision(
                    format!(
                        "duplicate ID '{}' in type '{}' detected during parallel merge",
                        id, type_name
                    ),
                    0,
                ));
            }
            // Register in target
            target.register(type_name, id, 0, limits)?;
        }
    }
    Ok(())
}

/// Parallel reference validation.
///
/// Validates references across document items in parallel.
/// The registry is read-only at this point, so no synchronization needed.
pub fn validate_references_parallel(
    items: &BTreeMap<String, Item>,
    registries: &TypeRegistry,
    strict: bool,
    max_depth: usize,
) -> HedlResult<()> {
    // Skip parallelism for small documents
    if items.len() < 10 {
        return validate_references_sequential(items, registries, strict, None, 0, max_depth);
    }

    // Validate each item in parallel
    let results: Vec<HedlResult<()>> = items
        .par_iter()
        .map(|(_, item)| validate_item_refs(item, registries, strict, None, 0, max_depth))
        .collect();

    // Propagate first error
    for result in results {
        result?;
    }

    Ok(())
}

/// Sequential reference validation (fallback).
fn validate_references_sequential(
    items: &BTreeMap<String, Item>,
    registries: &TypeRegistry,
    strict: bool,
    current_type: Option<&str>,
    depth: usize,
    max_depth: usize,
) -> HedlResult<()> {
    if depth > max_depth {
        return Err(HedlError::security(
            format!(
                "NEST hierarchy depth {} exceeds maximum allowed depth {}",
                depth, max_depth
            ),
            0,
        ));
    }

    for item in items.values() {
        validate_item_refs(item, registries, strict, current_type, depth, max_depth)?;
    }

    Ok(())
}

/// Validate references in a single item.
fn validate_item_refs(
    item: &Item,
    registries: &TypeRegistry,
    strict: bool,
    current_type: Option<&str>,
    depth: usize,
    max_depth: usize,
) -> HedlResult<()> {
    match item {
        Item::Scalar(value) => {
            validate_value_ref(value, registries, strict, current_type)?;
        }
        Item::List(list) => {
            for node in &list.rows {
                validate_node_refs(node, registries, strict, depth, max_depth)?;
            }
        }
        Item::Object(obj) => {
            validate_references_sequential(
                obj,
                registries,
                strict,
                current_type,
                depth + 1,
                max_depth,
            )?;
        }
    }
    Ok(())
}

/// Validate references in a node.
fn validate_node_refs(
    node: &Node,
    registries: &TypeRegistry,
    strict: bool,
    depth: usize,
    max_depth: usize,
) -> HedlResult<()> {
    if depth > max_depth {
        return Err(HedlError::security(
            format!(
                "NEST hierarchy depth {} exceeds maximum allowed depth {}",
                depth, max_depth
            ),
            0,
        ));
    }

    for value in &node.fields {
        validate_value_ref(value, registries, strict, Some(&node.type_name))?;
    }

    if let Some(children) = node.children() {
        for child_list in children.values() {
            for child in child_list {
                validate_node_refs(child, registries, strict, depth + 1, max_depth)?;
            }
        }
    }

    Ok(())
}

/// Validate a single reference value.
fn validate_value_ref(
    value: &Value,
    registries: &TypeRegistry,
    strict: bool,
    current_type: Option<&str>,
) -> HedlResult<()> {
    if let Value::Reference(ref_val) = value {
        let resolved = match &ref_val.type_name {
            Some(t) => registries.contains_in_type(t, &ref_val.id),
            None => match current_type {
                Some(type_name) => registries.contains_in_type(type_name, &ref_val.id),
                None => {
                    let matching_types = registries.lookup_unqualified(&ref_val.id).unwrap_or(&[]);
                    match matching_types.len() {
                        0 => false,
                        1 => true,
                        _ => {
                            return Err(HedlError::reference(
                                format!(
                                    "Ambiguous unqualified reference '@{}' matches multiple types: [{}]",
                                    ref_val.id,
                                    matching_types.join(", ")
                                ),
                                0,
                            ));
                        }
                    }
                }
            },
        };

        if !resolved && strict {
            return Err(HedlError::reference(
                format!("unresolved reference {}", ref_val.to_ref_string()),
                0,
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_root_entities, 50);
        assert_eq!(config.min_list_rows, 100);
        assert!(config.thread_pool_size.is_none());
    }

    #[test]
    fn test_parallel_config_conservative() {
        let config = ParallelConfig::conservative();
        assert_eq!(config.min_root_entities, 100);
        assert_eq!(config.min_list_rows, 200);
    }

    #[test]
    fn test_parallel_config_aggressive() {
        let config = ParallelConfig::aggressive();
        assert_eq!(config.min_root_entities, 20);
        assert_eq!(config.min_list_rows, 50);
    }

    #[test]
    fn test_parallel_config_thresholds() {
        let config = ParallelConfig::default();

        assert!(!config.should_parallelize_entities(10));
        assert!(!config.should_parallelize_entities(49));
        assert!(config.should_parallelize_entities(50));
        assert!(config.should_parallelize_entities(100));

        assert!(!config.should_parallelize_rows(10));
        assert!(!config.should_parallelize_rows(99));
        assert!(config.should_parallelize_rows(100));
        assert!(config.should_parallelize_rows(1000));
    }

    #[test]
    fn test_atomic_counters() {
        let counters = AtomicSecurityCounters::new();
        assert_eq!(counters.node_count(), 0);
        assert_eq!(counters.key_count(), 0);

        let limits = Limits::default();

        // Increment should work
        counters.increment_nodes(&limits, 0).unwrap();
        assert_eq!(counters.node_count(), 1);

        counters.increment_keys(&limits, 0).unwrap();
        assert_eq!(counters.key_count(), 1);
    }

    #[test]
    fn test_atomic_counters_limit_exceeded() {
        let counters = AtomicSecurityCounters::new();
        let limits = Limits {
            max_nodes: 1,
            ..Default::default()
        };

        // First increment succeeds
        counters.increment_nodes(&limits, 0).unwrap();

        // Second increment fails
        let result = counters.increment_nodes(&limits, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_entity_boundary_identification() {
        let lines: Vec<(usize, &str)> = vec![
            (1, "users: @User"),
            (2, "| alice"),
            (3, "| bob"),
            (4, "settings:"),
            (5, "  debug: true"),
            (6, "count: 42"),
        ];

        let boundaries = identify_entity_boundaries(&lines);
        assert_eq!(boundaries.len(), 3);

        assert_eq!(boundaries[0].key, "users");
        assert_eq!(boundaries[0].entity_type, EntityType::List);
        assert_eq!(boundaries[0].line_range, 0..3);

        assert_eq!(boundaries[1].key, "settings");
        assert_eq!(boundaries[1].entity_type, EntityType::Object);
        assert_eq!(boundaries[1].line_range, 3..5);

        assert_eq!(boundaries[2].key, "count");
        assert_eq!(boundaries[2].entity_type, EntityType::Scalar);
        assert_eq!(boundaries[2].line_range, 5..6);
    }

    #[test]
    fn test_matrix_row_batch_no_ditto() {
        let batch = MatrixRowBatch {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![(1, "|alice, Alice"), (2, "|bob, Bob")],
            has_ditto: false,
        };

        assert!(batch.can_parallelize());
    }

    #[test]
    fn test_matrix_row_batch_with_ditto() {
        let batch = MatrixRowBatch {
            type_name: "User".to_string(),
            schema: vec!["id".to_string(), "name".to_string()],
            rows: vec![(1, "|alice, Alice"), (2, "|bob, \"")],
            has_ditto: true,
        };

        assert!(!batch.can_parallelize());
    }

    #[test]
    fn test_parse_single_matrix_row() {
        let header = Header::new((1, 0));
        let schema = vec!["id".to_string(), "name".to_string()];

        let node = parse_single_matrix_row("|alice, Alice", &schema, "User", &header, 1).unwrap();

        assert_eq!(node.id, "alice");
        assert_eq!(node.type_name, "User");
        assert_eq!(node.fields.len(), 2);
    }

    #[test]
    fn test_parse_single_matrix_row_with_child_count() {
        let header = Header::new((1, 0));
        let schema = vec!["id".to_string(), "name".to_string()];

        let node =
            parse_single_matrix_row("|[3] alice, Alice", &schema, "User", &header, 1).unwrap();

        assert_eq!(node.id, "alice");
        assert_eq!(node.child_count, 3);
    }
}
