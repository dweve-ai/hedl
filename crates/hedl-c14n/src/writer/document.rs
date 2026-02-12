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

//! Document and matrix list writing implementation for canonical writer.

use std::collections::BTreeMap;
use std::fmt::Write;

use super::constants::*;
use super::CanonicalWriter;
use crate::ditto::can_use_ditto;
use hedl_core::{Document, HedlError, Item, MatrixList, Node, Value};

impl CanonicalWriter {
    /// Writes a HEDL document to canonical string format.
    ///
    /// Returns the canonicalized document as a string, or an error if writing fails.
    pub fn write_document(&mut self, doc: &Document) -> Result<String, HedlError> {
        // Store document version for version-specific output formatting
        self.version = doc.version;

        // P3 OPTIMIZATION: Pre-size output buffer based on document structure
        let estimated_size = Self::estimate_output_size(doc);
        if estimated_size > self.output.capacity() {
            self.output.reserve(estimated_size - self.output.capacity());
        }

        // Check if using compact format
        let compact = Self::use_compact_format(doc);

        // Header: VERSION
        if compact {
            writeln!(self.output, "%V:{}.{}", doc.version.0, doc.version.1)
                .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;
            // v2.0 required headers: null marker and quote character
            writeln!(self.output, "%NULL:~")
                .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;
            writeln!(self.output, "%QUOTE:\"")
                .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;
        } else {
            writeln!(self.output, "%VERSION: {}.{}", doc.version.0, doc.version.1)
                .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;
        }

        // Aliases (sorted)
        let mut aliases: Vec<_> = doc.aliases.iter().collect();
        aliases.sort_by_key(|(k, _)| *k);
        for (key, value) in aliases {
            if compact {
                writeln!(
                    self.output,
                    "%A:%{}:\"{}\"",
                    key,
                    Self::escape_quoted(value)
                )
                .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;
            } else {
                writeln!(
                    self.output,
                    "%ALIAS: %{}: \"{}\"",
                    key,
                    Self::escape_quoted(value)
                )
                .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;
            }
        }

        // Structs (sorted) - only if not using inline schemas
        // When not using inline schemas, we must include ALL types used in the body
        // (both header-declared and inline schema types)
        if !self.config.inline_schemas {
            // Start with header-declared structs
            let mut all_structs: BTreeMap<String, Vec<String>> = doc.structs.clone();

            // Extract types from all matrix lists in the body and collect counts
            let mut struct_counts: BTreeMap<String, usize> = BTreeMap::new();
            Self::collect_matrix_list_types_and_counts(
                &doc.root,
                &mut all_structs,
                &mut struct_counts,
            );

            let mut structs: Vec<_> = all_structs.iter().collect();
            structs.sort_by_key(|(k, _)| *k);
            for (type_name, columns) in structs {
                if compact {
                    // v2.0 format: %S:Type:[fields] (NO count in parentheses)
                    writeln!(self.output, "%S:{}:[{}]", type_name, columns.join(",")).map_err(
                        |e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN),
                    )?;
                } else {
                    // Verbose format
                    writeln!(
                        self.output,
                        "%STRUCT: {}: [{}]",
                        type_name,
                        columns.join(",")
                    )
                    .map_err(|e| {
                        HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                    })?;
                }
            }

            // v2.0: Write count directives AFTER struct declarations
            // %C:Type.total=N for each type that has instances
            if self.is_v20_or_later() {
                let mut count_entries: Vec<_> = struct_counts.iter().collect();
                count_entries.sort_by_key(|(type_name, _)| *type_name);
                for (type_name, count) in count_entries {
                    writeln!(self.output, "%C:{}.total={}", type_name, count).map_err(|e| {
                        HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                    })?;
                }
            }
        }

        // Nests (sorted by parent then child)
        // Each parent can have multiple children
        let mut nest_pairs: Vec<(&String, &String)> = Vec::new();
        for (parent, children) in &doc.nests {
            for child in children {
                nest_pairs.push((parent, child));
            }
        }
        nest_pairs.sort_by_key(|(p, c)| (*p, *c));
        for (parent, child) in nest_pairs {
            if compact {
                writeln!(self.output, "%N:{parent}>{child}").map_err(|e| {
                    HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                })?;
            } else {
                writeln!(self.output, "%NEST: {parent} > {child}").map_err(|e| {
                    HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                })?;
            }
        }

        // Separator
        writeln!(self.output, "---")
            .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;

        // Body (sorted keys if configured)
        self.write_items(&doc.root, ROOT_INDENT_LEVEL)?;

        Ok(std::mem::take(&mut self.output))
    }

    /// Recursively collect all `MatrixList` types and their counts from the document body.
    /// This ensures inline schema types are included in STRUCT declarations with counts.
    pub(super) fn collect_matrix_list_types_and_counts(
        items: &BTreeMap<String, Item>,
        structs: &mut BTreeMap<String, Vec<String>>,
        counts: &mut BTreeMap<String, usize>,
    ) {
        for item in items.values() {
            match item {
                Item::List(matrix_list) => {
                    // Add this type if not already present
                    structs
                        .entry(matrix_list.type_name.clone())
                        .or_insert_with(|| matrix_list.schema.clone());

                    // Sum counts across all lists of the same type
                    *counts
                        .entry(matrix_list.type_name.clone())
                        .or_insert(INITIAL_STRUCT_COUNT) += matrix_list.rows.len();
                }
                Item::Object(child_items) => {
                    // Recurse into nested objects
                    Self::collect_matrix_list_types_and_counts(child_items, structs, counts);
                }
                Item::Scalar(_) => {}
            }
        }
    }

    /// Write all items in a key-value map to output.
    ///
    /// Recursively handles nested objects, matrix lists, and scalar values.
    /// Enforces maximum nesting depth limit to prevent stack overflow.
    ///
    /// # Arguments
    ///
    /// * `items` - Map of keys to items to serialize
    /// * `indent` - Current indentation level (0 = root level)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Writing to output buffer fails
    /// - Nesting depth exceeds `MAX_NESTING_DEPTH`
    pub(super) fn write_items(
        &mut self,
        items: &BTreeMap<String, Item>,
        indent: usize,
    ) -> Result<(), HedlError> {
        // SECURITY: Prevent stack overflow DoS attacks from deeply nested documents
        if indent > MAX_NESTING_DEPTH {
            return Err(HedlError::syntax(
                format!(
                    "Maximum nesting depth of {MAX_NESTING_DEPTH} exceeded (current depth: {indent})"
                ),
                ERROR_LINE_UNKNOWN,
            ));
        }

        // P2 OPTIMIZATION: Use cached indentation string
        // Note: Clone the string to avoid holding a borrow across recursive calls
        let indent_str = self.indent_cache[indent].clone();

        // P0 OPTIMIZATION: Eliminate key cloning (1.15x speedup, 10-15% fewer allocations)
        // BTreeMap is already sorted, iterate directly without collecting/cloning
        // Note: sort_keys config is redundant for BTreeMap (always sorted)
        for (key, item) in items {
            match item {
                Item::Scalar(value) => {
                    let (formatted, needs_block) = self.format_value_for_kv(value);
                    if needs_block {
                        // Write block string
                        writeln!(self.output, "{indent_str}{key}: \"\"\"").map_err(|e| {
                            HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                        })?;
                        for line in formatted.lines() {
                            writeln!(self.output, "{line}").map_err(|e| {
                                HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                            })?;
                        }
                        writeln!(self.output, "\"\"\"").map_err(|e| {
                            HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                        })?;
                    } else {
                        writeln!(self.output, "{indent_str}{key}: {formatted}").map_err(|e| {
                            HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                        })?;
                    }
                }
                Item::Object(child_items) => {
                    writeln!(self.output, "{indent_str}{key}:").map_err(|e| {
                        HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                    })?;
                    self.write_items(child_items, indent + INDENT_INCREMENT)?;
                }
                Item::List(matrix_list) => {
                    self.write_matrix_list(key, matrix_list, indent)?;
                }
            }
        }

        Ok(())
    }

    /// Format a row's cells with ditto optimization.
    ///
    /// This is a common helper used by both `write_matrix_list` and `write_child_rows`
    /// to format row cells with optional ditto markers for repeated values.
    ///
    /// # Arguments
    ///
    /// * `values` - The row's field values
    /// * `last_values` - Previous row's values for ditto comparison (None for first row)
    /// * `cells` - Reusable buffer to populate with formatted cells
    ///
    /// # Performance
    ///
    /// - Reuses the provided `cells` buffer to minimize allocations (P1 optimization)
    /// - P2 OPTIMIZATION: Uses references instead of cloning for ditto comparison
    pub(super) fn format_row_cells(
        &self,
        values: &[Value],
        last_values: Option<&[Value]>,
        cells: &mut Vec<String>,
    ) {
        let num_cols = values.len();
        cells.clear();

        for (i, value) in values.iter().enumerate() {
            let is_last_col = i == num_cols - LAST_COLUMN_OFFSET;

            // Never use ditto for ID column (first column), and ditto is NOT allowed in v2.0+
            let cell = if i == ID_COLUMN_INDEX || !self.config.use_ditto || self.is_v20_or_later() {
                self.format_cell_value_with_position(value, is_last_col)
            } else if let Some(prev) = last_values {
                if can_use_ditto(value, &prev[i]) {
                    "^".to_string()
                } else {
                    self.format_cell_value_with_position(value, is_last_col)
                }
            } else {
                self.format_cell_value_with_position(value, is_last_col)
            };
            cells.push(cell);
        }
    }

    pub(super) fn write_matrix_list(
        &mut self,
        key: &str,
        list: &MatrixList,
        indent: usize,
    ) -> Result<(), HedlError> {
        // P2 OPTIMIZATION: Use cached indentation strings
        // Note: Clone to avoid holding borrows across mutable calls
        let indent_str = self.indent_cache[indent].clone();
        let row_indent = self.indent_cache[indent + MATRIX_ROW_INDENT_OFFSET].clone();

        // List declaration (counts go in %STRUCT header, not here)
        if self.config.inline_schemas {
            // P5 OPTIMIZATION: Use schema cache
            let schema_str = self.get_schema_string(&list.type_name, &list.schema);
            writeln!(
                self.output,
                "{}{}:@{}[{}]",
                indent_str, key, list.type_name, schema_str
            )
            .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;
        } else {
            // P5 OPTIMIZATION: Use type reference cache
            // v2.0 canonical format: key:@Type (no space between key and @Type)
            let type_ref = self.get_type_ref(&list.type_name);
            writeln!(self.output, "{indent_str}{key}:{type_ref}")
                .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;
        }

        // Rows with ditto optimization
        let mut last_values: Option<&[Value]> = None;

        for row_node in &list.rows {
            // P2 OPTIMIZATION: Use references instead of cloning
            let values = &row_node.fields;

            // Scope the cell buffer borrow so it's released before recursion
            {
                // P2 OPTIMIZATION: Use pooled cell buffer
                let mut cells = self.cell_buffer.borrow_mut();
                cells.clear();
                cells.reserve(list.schema.len());

                // Format row cells with ditto optimization
                self.format_row_cells(values, last_values, &mut cells);

                // P3 OPTIMIZATION: Write cells directly to output without intermediate join
                write!(self.output, "{row_indent}|").map_err(|e| {
                    HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                })?;

                // For pre-v2.0: use |[N] prefix if node has child_count
                // For v2.0+: no |[N] count hints, use %C: header directives instead
                if !self.is_v20_or_later() && row_node.child_count > 0 {
                    write!(self.output, "[{}]", row_node.child_count).map_err(|e| {
                        HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                    })?;
                }

                // Write cells directly without intermediate string allocation
                for (i, cell) in cells.iter().enumerate() {
                    if i > 0 {
                        write!(self.output, ",").map_err(|e| {
                            HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                        })?;
                    }
                    write!(self.output, "{cell}").map_err(|e| {
                        HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                    })?;
                }
                writeln!(self.output).map_err(|e| {
                    HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                })?;
            } // cells is dropped here

            // Write children if any (borrow is released)
            if let Some(children) = row_node.children() {
                for (child_type, child_nodes) in children.iter() {
                    // Must use expanded format if:
                    // 1. More than INLINE_CHILD_THRESHOLD children, OR
                    // 2. Any child has its own children (grandchildren)
                    // Reason: inline format `@Type#N:|a|b|c` can't have nested children
                    // because the parser can't associate grandchildren with inline parents
                    let any_has_grandchildren = child_nodes.iter().any(|n| n.children().is_some());
                    if child_nodes.len() <= INLINE_CHILD_THRESHOLD && !any_has_grandchildren {
                        // Use inline format for <= 5 leaf children
                        self.write_inline_child_rows(
                            child_type,
                            child_nodes,
                            indent + MATRIX_CHILD_INDENT_OFFSET,
                        )?;
                    } else {
                        // Use expanded format when > 5 children or when there are grandchildren
                        self.write_expanded_child_rows(
                            child_type,
                            child_nodes,
                            indent + MATRIX_CHILD_INDENT_OFFSET,
                        )?;
                    }
                }
            }

            last_values = Some(values);
        }

        Ok(())
    }

    /// Write child rows in inline format.
    ///
    /// Inline format: `@ChildType#N:|child1|child2|...|childN`
    /// Used when child count <= INLINE_CHILD_THRESHOLD (5).
    ///
    /// # Arguments
    ///
    /// * `child_type` - Type name of the children
    /// * `nodes` - Child nodes to serialize (must be <= INLINE_CHILD_THRESHOLD)
    /// * `indent` - Current indentation level
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Writing to output buffer fails
    /// - Nesting depth exceeds `MAX_NESTING_DEPTH`
    pub(super) fn write_inline_child_rows(
        &mut self,
        child_type: &str,
        nodes: &[Node],
        indent: usize,
    ) -> Result<(), HedlError> {
        // SECURITY: Prevent stack overflow DoS attacks
        if indent > MAX_NESTING_DEPTH {
            return Err(HedlError::syntax(
                format!(
                    "Maximum nesting depth of {MAX_NESTING_DEPTH} exceeded in matrix list (current depth: {indent})"
                ),
                ERROR_LINE_UNKNOWN,
            ));
        }

        let row_indent = self.indent_cache[indent].clone();

        // Write inline child declaration:@ChildType#N:
        write!(
            self.output,
            "{}@{}#{}:",
            row_indent,
            child_type,
            nodes.len()
        )
        .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;

        // Write inline children separated by |
        let mut last_values: Option<&[Value]> = None;

        for row_node in nodes.iter() {
            let values = &row_node.fields;

            // Use pooled cell buffer
            let mut cells = self.cell_buffer.borrow_mut();
            cells.clear();

            // Format row cells with ditto optimization
            self.format_row_cells(values, last_values, &mut cells);

            // Write pipe separator (before each child)
            write!(self.output, "|")
                .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;

            // Write cells directly without intermediate string allocation
            for (j, cell) in cells.iter().enumerate() {
                if j > 0 {
                    write!(self.output, ",").map_err(|e| {
                        HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                    })?;
                }
                write!(self.output, "{cell}").map_err(|e| {
                    HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                })?;
            }

            // Release the cells borrow before checking grandchildren
            drop(cells);

            last_values = Some(values);

            // Note: Inline format does not support nested grandchildren
            // If row_node has children, they would be written on separate lines after this inline row
            // This is handled by the parent logic
        }

        // End the inline children line
        writeln!(self.output)
            .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;

        // Write grandchildren if any (on separate expanded lines)
        for row_node in nodes {
            if let Some(children) = row_node.children() {
                for (grandchild_type, grandchild_nodes) in children.iter() {
                    if grandchild_nodes.len() <= INLINE_CHILD_THRESHOLD {
                        self.write_inline_child_rows(
                            grandchild_type,
                            grandchild_nodes,
                            indent + INDENT_INCREMENT,
                        )?;
                    } else {
                        self.write_expanded_child_rows(
                            grandchild_type,
                            grandchild_nodes,
                            indent + INDENT_INCREMENT,
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Write child rows in expanded format (one per line).
    ///
    /// Expanded format used when child count > INLINE_CHILD_THRESHOLD (5).
    /// Also writes the type declaration line: `@ChildType#N:`
    ///
    /// # Arguments
    ///
    /// * `child_type` - Type name of the children
    /// * `nodes` - Child nodes to serialize
    /// * `indent` - Current indentation level
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Writing to output buffer fails
    /// - Nesting depth exceeds `MAX_NESTING_DEPTH`
    pub(super) fn write_expanded_child_rows(
        &mut self,
        child_type: &str,
        nodes: &[Node],
        indent: usize,
    ) -> Result<(), HedlError> {
        // SECURITY: Prevent stack overflow DoS attacks from deeply nested matrix lists
        if indent > MAX_NESTING_DEPTH {
            return Err(HedlError::syntax(
                format!(
                    "Maximum nesting depth of {MAX_NESTING_DEPTH} exceeded in matrix list (current depth: {indent})"
                ),
                ERROR_LINE_UNKNOWN,
            ));
        }

        // P2 OPTIMIZATION: Use cached indentation string
        // Note: Clone to avoid holding borrow across recursive calls
        // @Type declaration and its |rows are at the same indent level
        let indent_str = self.indent_cache[indent].clone();

        // Write type declaration line: @ChildType#N:
        writeln!(
            self.output,
            "{}@{}#{}:",
            indent_str,
            child_type,
            nodes.len()
        )
        .map_err(|e| HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN))?;

        let mut last_values: Option<&[Value]> = None;

        for row_node in nodes {
            // P2 OPTIMIZATION: Use references instead of cloning
            let values = &row_node.fields;

            // Scope the cell buffer borrow so it's released before recursion
            {
                // P2 OPTIMIZATION: Use pooled cell buffer
                let mut cells = self.cell_buffer.borrow_mut();
                cells.clear();

                // Format row cells with ditto optimization
                self.format_row_cells(values, last_values, &mut cells);

                // P3 OPTIMIZATION: Write cells directly to output without intermediate join
                write!(self.output, "{indent_str}|").map_err(|e| {
                    HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                })?;

                // For pre-v2.0: use |[N] prefix if node has child_count
                // For v2.0+: no |[N] count hints, use %C: header directives instead
                if !self.is_v20_or_later() && row_node.child_count > 0 {
                    write!(self.output, "[{}]", row_node.child_count).map_err(|e| {
                        HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                    })?;
                }

                // Write cells directly without intermediate string allocation
                for (i, cell) in cells.iter().enumerate() {
                    if i > 0 {
                        write!(self.output, ",").map_err(|e| {
                            HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                        })?;
                    }
                    write!(self.output, "{cell}").map_err(|e| {
                        HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                    })?;
                }
                writeln!(self.output).map_err(|e| {
                    HedlError::syntax(format!("Write error: {e}"), ERROR_LINE_UNKNOWN)
                })?;
            } // cells is dropped here

            // Write children if any (recursive, borrow is released)
            if let Some(children) = row_node.children() {
                for (child_type, child_nodes) in children.iter() {
                    // Must use expanded format if:
                    // 1. More than INLINE_CHILD_THRESHOLD children, OR
                    // 2. Any child has its own children (grandchildren)
                    let any_has_grandchildren = child_nodes.iter().any(|n| n.children().is_some());
                    if child_nodes.len() <= INLINE_CHILD_THRESHOLD && !any_has_grandchildren {
                        // Use inline format for <= 5 leaf children
                        self.write_inline_child_rows(
                            child_type,
                            child_nodes,
                            indent + INDENT_INCREMENT,
                        )?;
                    } else {
                        // Use expanded format when > 5 children or when there are grandchildren
                        self.write_expanded_child_rows(
                            child_type,
                            child_nodes,
                            indent + INDENT_INCREMENT,
                        )?;
                    }
                }
            }

            last_values = Some(values);
        }

        Ok(())
    }
}
