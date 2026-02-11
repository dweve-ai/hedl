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

//! Line parsing functions for different HEDL syntax elements.

use super::context::Frame;
use super::utils::{
    check_duplicate_key, insert_into_current, parse_quoted_string, validate_indent_for_child,
    validate_nested_list_indent,
};
use crate::document::{Item, Node};
use crate::error::{HedlError, HedlResult};
use crate::inference::{infer_quoted_value, infer_value, InferenceContext};
use crate::lex::row::parse_csv_row;
use crate::lex::{is_valid_key_token, is_valid_type_name, strip_comment};
use crate::limits::Limits;
use crate::reference::{register_node, TypeRegistry};
use crate::value::Value;
use std::collections::BTreeMap;

/// Bundled parameters for matrix row and inline child list parsing.
///
/// Groups the immutable context needed by `parse_matrix_row` and
/// `parse_inline_child_list` to keep function signatures concise.
pub(super) struct MatrixParseParams<'a> {
    /// The content of the line being parsed (after indent stripping).
    pub content: &'a str,
    /// The indentation level of the current line.
    pub indent: usize,
    /// The 1-based line number for error reporting.
    pub line_num: usize,
    /// The parsed HEDL header with schema definitions.
    pub header: &'a crate::header::Header,
    /// Resource limits for security enforcement.
    pub limits: &'a Limits,
}

/// Parse a non-matrix line (key-value, object declaration, or list declaration).
pub(super) fn parse_non_matrix_line(
    stack: &mut Vec<Frame>,
    content: &str,
    indent: usize,
    line_num: usize,
    header: &crate::header::Header,
    limits: &Limits,
    total_keys: &mut usize,
) -> HedlResult<()> {
    let content = strip_comment(content);

    // Find colon
    let colon_pos = content
        .find(':')
        .ok_or_else(|| HedlError::syntax("expected ':' in line", line_num))?;

    let key_with_hint = content[..colon_pos].trim();
    let after_colon = &content[colon_pos + 1..];

    // Extract count hint from key if present (e.g., "teams(3)")
    let (key, count_hint) = parse_key_with_count_hint(key_with_hint, line_num)?;

    // Validate key
    if !is_valid_key_token(&key) {
        return Err(HedlError::syntax(format!("invalid key: {}", key), line_num));
    }

    // Check for duplicate key
    check_duplicate_key(stack, &key, line_num, limits, total_keys)?;

    // Determine line type
    let after_colon_trimmed = after_colon.trim();

    if after_colon_trimmed.is_empty() {
        // Object start
        if count_hint.is_some() {
            return Err(HedlError::syntax(
                "count hint not allowed on object declarations",
                line_num,
            ));
        }
        validate_indent_for_child(stack, indent, line_num)?;
        stack.push(Frame::Object {
            indent,
            key: key.to_string(),
            object: BTreeMap::new(),
        });
    } else if after_colon_trimmed.starts_with('@') && is_list_start(after_colon_trimmed) {
        // Matrix list start
        // Accept both "key:@Type" (v2.0 canonical) and "key:@Type" (backward compat)

        // Check if this is a nested list declaration inside a list context
        let parent_list_idx = validate_nested_list_indent(stack, indent, line_num)?;

        let (type_name, schema) = parse_list_start(after_colon_trimmed, line_num, header, limits)?;

        if let Some(_parent_idx) = parent_list_idx {
            // This is a nested list inside a list context (e.g., divisions(3):@Division under a company row)
            // Push the new list frame - it will be attached to parent row when finalized
            // Pre-allocate list based on count_hint (bounded to prevent DoS)
            let capacity = count_hint.unwrap_or(0).min(limits.max_nodes);
            stack.push(Frame::List {
                row_indent: indent + 1,
                type_name,
                schema,
                last_row_values: None,
                list: Vec::with_capacity(capacity),
                key: key.to_string(),
                count_hint,
            });
        } else {
            // Normal top-level or object-nested list
            // Pre-allocate list based on count_hint (bounded to prevent DoS)
            let capacity = count_hint.unwrap_or(0).min(limits.max_nodes);
            stack.push(Frame::List {
                row_indent: indent + 1,
                type_name,
                schema,
                last_row_values: None,
                list: Vec::with_capacity(capacity),
                key: key.to_string(),
                count_hint,
            });
        }
    } else {
        // Key-value pair
        if count_hint.is_some() {
            return Err(HedlError::syntax(
                "count hint not allowed on scalar values",
                line_num,
            ));
        }
        if !after_colon.starts_with(' ') {
            return Err(HedlError::syntax(
                "space required after ':' in key-value",
                line_num,
            ));
        }
        validate_indent_for_child(stack, indent, line_num)?;
        let value_str = after_colon.trim();
        let ctx = InferenceContext::for_key_value(&header.aliases)
            .with_version(header.version)
            .with_null_char(header.null_char);
        let value = if value_str.starts_with('"') {
            // Quoted value
            let inner = parse_quoted_string(value_str, line_num)?;
            infer_quoted_value(&inner)
        } else {
            infer_value(value_str, &ctx, line_num)?
        };
        insert_into_current(stack, key.to_string(), Item::Scalar(value));
    }

    Ok(())
}

/// Parse a key that may have a count hint in parentheses.
/// Examples: "teams" -> ("teams", None), "teams(3)" -> ("teams", Some(3))
///
/// DEPRECATED: The `name(N):@Type` syntax for count hints is being replaced by
/// the new row-level `|N|data` syntax. This function is maintained for backward
/// compatibility but the old syntax is deprecated and may be removed in future versions.
pub(super) fn parse_key_with_count_hint(
    key: &str,
    line_num: usize,
) -> HedlResult<(String, Option<usize>)> {
    if let Some(paren_pos) = key.find('(') {
        // Extract key and count
        let key_part = &key[..paren_pos];

        // Find closing parenthesis
        if !key.ends_with(')') {
            return Err(HedlError::syntax(
                "unclosed count hint parenthesis",
                line_num,
            ));
        }

        let count_str = &key[paren_pos + 1..key.len() - 1];

        // Parse count
        let count = count_str.parse::<usize>().map_err(|_| {
            HedlError::syntax(format!("invalid count hint: '{}'", count_str), line_num)
        })?;

        if count == 0 {
            return Err(HedlError::syntax(
                "count hint must be greater than zero",
                line_num,
            ));
        }

        Ok((key_part.to_string(), Some(count)))
    } else {
        Ok((key.to_string(), None))
    }
}

/// Check if a string represents the start of a matrix list (@TypeName or @TypeName[...]).
pub(super) fn is_list_start(s: &str) -> bool {
    // @TypeName or @TypeName[...]
    let s = s.trim();
    if !s.starts_with('@') {
        return false;
    }
    let rest = &s[1..];
    // Find end of type name
    let type_end = rest
        .find(|c: char| c == '[' || c.is_whitespace())
        .unwrap_or(rest.len());
    let type_name = &rest[..type_end];
    is_valid_type_name(type_name)
}

/// Parse the start of a list declaration (@TypeName or @TypeName[cols]).
pub(super) fn parse_list_start(
    s: &str,
    line_num: usize,
    header: &crate::header::Header,
    limits: &Limits,
) -> HedlResult<(String, Vec<String>)> {
    let s = s.trim();
    let rest = &s[1..]; // Skip @

    if let Some(bracket_pos) = rest.find('[') {
        // Inline schema:@TypeName[col1, col2]
        let type_name = &rest[..bracket_pos];
        if !is_valid_type_name(type_name) {
            return Err(HedlError::syntax(
                format!("invalid type name: {}", type_name),
                line_num,
            ));
        }

        let schema_str = &rest[bracket_pos..];
        let schema = parse_inline_schema(schema_str, line_num, limits)?;

        // Check against declared schema if exists
        if let Some(declared) = header.structs.get(type_name) {
            if declared != &schema {
                return Err(HedlError::schema(
                    format!(
                        "inline schema for '{}' doesn't match declared schema",
                        type_name
                    ),
                    line_num,
                ));
            }
        }

        Ok((type_name.to_string(), schema))
    } else {
        // Reference to declared schema:@TypeName
        let type_name = rest.trim();
        if !is_valid_type_name(type_name) {
            return Err(HedlError::syntax(
                format!("invalid type name: {}", type_name),
                line_num,
            ));
        }

        let schema = header
            .structs
            .get(type_name)
            .ok_or_else(|| HedlError::schema(format!("undefined type: {}", type_name), line_num))?;

        Ok((type_name.to_string(), schema.clone()))
    }
}

/// Parse an inline schema [col1, col2, ...].
pub(super) fn parse_inline_schema(
    s: &str,
    line_num: usize,
    limits: &Limits,
) -> HedlResult<Vec<String>> {
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(HedlError::syntax("invalid inline schema format", line_num));
    }

    let inner = &s[1..s.len() - 1];
    // Pre-allocate based on comma count (bounded by max_columns to prevent DoS)
    let estimated_count = inner.matches(',').count() + 1;
    let mut columns = Vec::with_capacity(estimated_count.min(limits.max_columns));

    for part in inner.split(',') {
        let col = part.trim();
        if col.is_empty() {
            continue;
        }
        if !is_valid_key_token(col) {
            return Err(HedlError::syntax(
                format!("invalid column name: {}", col),
                line_num,
            ));
        }
        columns.push(col.to_string());
    }

    if columns.is_empty() {
        return Err(HedlError::syntax("empty inline schema", line_num));
    }

    if columns.len() > limits.max_columns {
        return Err(HedlError::security(
            format!("too many columns: {}", columns.len()),
            line_num,
        ));
    }

    Ok(columns)
}

/// Parse the row prefix to extract optional child count.
/// Patterns:
/// - `|[N] data` -> (Some(N), "data")  - parent with N children
/// - `|data`     -> (None, "data")     - leaf node (no count)
///
/// Note: Parser accepts `|[N]` in pre-v2.0 documents for backward compatibility.
/// v2.0+ REJECTS `|[N]` syntax per spec (use %C: header directives instead).
/// v2.0+ also REJECTS space after the pipe character.
pub(super) fn parse_row_prefix(
    content: &str,
    line_num: usize,
    version: (u32, u32),
) -> HedlResult<(Option<usize>, &str)> {
    // Content should start with |
    if !content.starts_with('|') {
        return Err(HedlError::syntax(
            "matrix row must start with '|'",
            line_num,
        ));
    }

    let rest = &content[1..]; // Skip first |

    // v2.0+: NO space after the pipe character (SPEC line 43)
    if version >= (2, 0) && rest.starts_with(' ') {
        return Err(HedlError::syntax(
            "v2.0 does not allow space after '|' in matrix rows",
            line_num,
        ));
    }

    // Check for |[N] pattern
    if rest.starts_with('[') {
        if let Some(bracket_end) = rest.find(']') {
            let count_str = &rest[1..bracket_end];
            if let Ok(count) = count_str.parse::<usize>() {
                // v2.0+: |[N] inline count hints are NOT allowed (SPEC line 46)
                if version >= (2, 0) {
                    return Err(HedlError::syntax(
                        "v2.0 does not allow inline count hints |[N], use %C: header directives instead",
                        line_num,
                    ));
                }
                // Count 0 is valid - means row has no children (empty parent)
                // Skip |[N] and any following space (pre-v2.0 only)
                let data = rest[bracket_end + 1..].trim_start();
                return Ok((Some(count), data));
            }
        }
    }

    // No count pattern, treat as |data (leaf node)
    Ok((None, rest))
}

/// Parse a matrix row line (|data or |[N]data).
pub(super) fn parse_matrix_row(
    stack: &mut Vec<Frame>,
    params: &MatrixParseParams<'_>,
    type_registries: &mut TypeRegistry,
    node_count: &mut usize,
) -> HedlResult<()> {
    let MatrixParseParams {
        content,
        indent,
        line_num,
        header,
        limits,
    } = params;
    let (indent, line_num) = (*indent, *line_num);
    // Parse the row prefix to extract optional child count and CSV content
    // Note: v2.0+ rejects |[N] and space after pipe
    let (child_count, csv_content) = parse_row_prefix(content, line_num, header.version)?;
    let csv_content = strip_comment(csv_content).trim();

    // Parse CSV early to get field count for child type detection
    let fields =
        parse_csv_row(csv_content).map_err(|e| HedlError::syntax(e.to_string(), line_num))?;

    // Find the active list frame, using field count for child type disambiguation
    let list_frame_idx =
        find_list_frame(stack, indent, line_num, header, limits, Some(fields.len()))?;

    // Get list info - optimize by avoiding unnecessary clones
    // We only need type_name as &str for inference and registration
    // Schema length is validated, prev_row is borrowed for inference
    let (schema_len, type_name_owned, prev_row_clone) = {
        let frame = &stack[list_frame_idx];
        match frame {
            Frame::List {
                type_name,
                schema,
                last_row_values,
                ..
            } => (schema.len(), type_name.clone(), last_row_values.clone()),
            _ => unreachable!(),
        }
    };

    // Validate shape
    if fields.len() != schema_len {
        return Err(HedlError::shape(
            format!("expected {} columns, got {}", schema_len, fields.len()),
            line_num,
        ));
    }

    // Infer values
    let mut values = Vec::with_capacity(fields.len());
    for (col_idx, field) in fields.iter().enumerate() {
        let ctx = InferenceContext::for_matrix_cell(
            &header.aliases,
            col_idx,
            prev_row_clone.as_deref(),
            &type_name_owned,
        )
        .with_version(header.version)
        .with_null_char(header.null_char);

        let value = if field.is_quoted {
            infer_quoted_value(&field.value)
        } else {
            infer_value(&field.value, &ctx, line_num)?
        };

        values.push(value);
    }

    // Get ID from first column
    let id = match &values[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(HedlError::semantic("ID column must be a string", line_num));
        }
    };

    // Register node ID
    register_node(type_registries, &type_name_owned, &id, line_num, limits)?;

    // Check node count limit with checked arithmetic to prevent overflow
    *node_count = node_count
        .checked_add(1)
        .ok_or_else(|| HedlError::security("node count overflow", line_num))?;
    if *node_count > limits.max_nodes {
        return Err(HedlError::security(
            format!("too many nodes: exceeds limit of {}", limits.max_nodes),
            line_num,
        ));
    }

    // Update list frame - avoid clone by storing values first, then creating node
    if let Frame::List {
        last_row_values,
        list,
        ..
    } = &mut stack[list_frame_idx]
    {
        // Store values for ditto support before moving to node
        *last_row_values = Some(values.clone());
        // Create node taking ownership of values - no extra clone needed
        let mut node = Node::new(&type_name_owned, &*id, values);

        // Store child count from |N| syntax if present
        if let Some(count) = child_count {
            node.set_child_count(count);
        }

        list.push(node);
    }

    Ok(())
}

/// Check if content matches inline child list pattern:@Type#N:|...
pub(super) fn is_inline_child_list(content: &str) -> bool {
    // Pattern:@TypeName#N:|child1|child2|...
    // Must start with @, have #, have :|
    if !content.starts_with('@') {
        return false;
    }

    // Find # and :| sequence
    if let Some(hash_pos) = content.find('#') {
        if let Some(colon_pos) = content[hash_pos..].find(":|") {
            // Validate that between # and :| is a number
            let count_str = &content[hash_pos + 1..hash_pos + colon_pos];
            return count_str.chars().all(|c| c.is_ascii_digit()) && !count_str.is_empty();
        }
    }

    false
}

/// Check if content matches expanded child list pattern:@Type#N: (no inline data)
pub(super) fn is_expanded_child_list(content: &str) -> bool {
    // Pattern:@TypeName#N: (ends with colon, no | after)
    // Must start with @, have #N, end with :
    if !content.starts_with('@') {
        return false;
    }

    let trimmed = content.trim();

    // Must end with : but not :|
    if !trimmed.ends_with(':') || trimmed.ends_with(":|") {
        return false;
    }

    // Find # and validate count
    if let Some(hash_pos) = trimmed.find('#') {
        // Content between # and : should be a number
        let colon_pos = trimmed.len() - 1;
        if hash_pos < colon_pos {
            let count_str = &trimmed[hash_pos + 1..colon_pos];
            return count_str.chars().all(|c| c.is_ascii_digit()) && !count_str.is_empty();
        }
    }

    false
}

/// Parse expanded child list declaration:@Type#N:
///
/// This creates a nested list frame for children that follow on subsequent lines.
pub(super) fn parse_expanded_child_list(
    stack: &mut Vec<Frame>,
    content: &str,
    indent: usize,
    line_num: usize,
    header: &crate::header::Header,
    limits: &Limits,
) -> HedlResult<()> {
    let trimmed = content.trim();

    // Extract type name (between @ and #)
    let hash_pos = trimmed
        .find('#')
        .ok_or_else(|| HedlError::syntax("expected '#' in expanded child list", line_num))?;
    let type_name = &trimmed[1..hash_pos];

    if !is_valid_type_name(type_name) {
        return Err(HedlError::syntax(
            format!("invalid type name in expanded child list: {}", type_name),
            line_num,
        ));
    }

    // Extract count (between # and :)
    let colon_pos = trimmed.len() - 1;
    let count_str = &trimmed[hash_pos + 1..colon_pos];
    let declared_count: usize = count_str.parse().map_err(|_| {
        HedlError::syntax(
            format!("invalid count in expanded child list: {}", count_str),
            line_num,
        )
    })?;

    // Get child schema
    let child_schema = header.structs.get(type_name).ok_or_else(|| {
        HedlError::schema(
            format!("type '{}' not defined in expanded child list", type_name),
            line_num,
        )
    })?;

    // Find the parent list frame
    let parent_list_idx = find_parent_list_for_inline_children(stack, indent, line_num)?;

    // Get parent type name for NEST validation
    let parent_type_name = {
        let frame = &stack[parent_list_idx];
        match frame {
            Frame::List { type_name, .. } => type_name.clone(),
            _ => unreachable!(),
        }
    };

    // Validate NEST relationship
    let child_types = header.nests.get(&parent_type_name).ok_or_else(|| {
        HedlError::schema(
            format!(
                "no NEST rule for parent type '{}' to child type '{}'",
                parent_type_name, type_name
            ),
            line_num,
        )
    })?;

    // Use iter().any() to avoid allocating a String for the contains check
    if !child_types.iter().any(|s| s == type_name) {
        return Err(HedlError::schema(
            format!(
                "type '{}' is not a declared child of '{}' (allowed: {:?})",
                type_name, parent_type_name, child_types
            ),
            line_num,
        ));
    }

    // SECURITY: Check NEST depth
    let current_depth = stack
        .iter()
        .filter(|f| matches!(f, Frame::List { .. }))
        .count();

    if current_depth >= limits.max_nest_depth {
        return Err(HedlError::security(
            format!(
                "NEST hierarchy depth {} exceeds maximum allowed depth {}",
                current_depth + 1,
                limits.max_nest_depth
            ),
            line_num,
        ));
    }

    // Push a new list frame for the expanded child list
    // Children will be parsed as regular matrix rows at indent + 1
    // Pre-allocate based on declared count (bounded to prevent DoS)
    let capacity = declared_count.min(limits.max_nodes);
    stack.push(Frame::List {
        row_indent: indent + 1,
        type_name: type_name.to_string(),
        schema: child_schema.clone(),
        last_row_values: None,
        list: Vec::with_capacity(capacity),
        key: type_name.to_string(),
        count_hint: Some(declared_count),
    });

    Ok(())
}

/// Parse inline child list:@Type#N:|child1|child2|...|childN
///
/// This parses the compact inline child list syntax where multiple child rows
/// are specified on a single line, separated by pipes.
pub(super) fn parse_inline_child_list(
    stack: &mut [Frame],
    params: &MatrixParseParams<'_>,
    type_registries: &mut TypeRegistry,
    node_count: &mut usize,
) -> HedlResult<()> {
    let MatrixParseParams {
        content,
        indent,
        line_num,
        header,
        limits,
    } = params;
    let (indent, line_num) = (*indent, *line_num);
    // Parse:@TypeName#N:|child1|child2|...|childN
    // Note: We don't call strip_comment here because # is part of the syntax (#N count).
    // Comments at the end of inline children are handled in the children data portion.

    // Extract type name (between @ and #)
    let hash_pos = content
        .find('#')
        .ok_or_else(|| HedlError::syntax("expected '#' in inline child list", line_num))?;
    let type_name = &content[1..hash_pos];

    if !is_valid_type_name(type_name) {
        return Err(HedlError::syntax(
            format!("invalid type name in inline child list: {}", type_name),
            line_num,
        ));
    }

    // Extract count (between # and :)
    let colon_pipe_pos = content
        .find(":|")
        .ok_or_else(|| HedlError::syntax("expected ':|' in inline child list", line_num))?;
    let count_str = &content[hash_pos + 1..colon_pipe_pos];
    let declared_count: usize = count_str.parse().map_err(|_| {
        HedlError::syntax(
            format!("invalid count in inline child list: {}", count_str),
            line_num,
        )
    })?;

    // NOTE: Per SPEC v2.0 line 58, the N <= 10 limit is a "style rule (not a hard syntax limit)"
    // The parser does NOT enforce this limit; use hedl-lint for style warnings.

    // Extract children data (after :|)
    // Apply strip_comment here to handle trailing comments like:@Type#2:|a|b # comment
    let children_data = strip_comment(&content[colon_pipe_pos + 2..]);

    // Split by | to get individual child rows
    // Use proper top-level splitting that respects quoted strings,
    // tensor literals, and list literals
    let child_rows: Vec<&str> = if children_data.is_empty() {
        Vec::new()
    } else {
        crate::lex::csv::split_inline_children(children_data, header.quote_char)
            .map_err(|e| HedlError::syntax(e.to_string(), line_num))?
    };

    // Validate count matches
    if child_rows.len() != declared_count {
        return Err(HedlError::syntax(
            format!(
                "inline child count mismatch: declared {} but found {}",
                declared_count,
                child_rows.len()
            ),
            line_num,
        ));
    }

    // Get child schema
    let child_schema = header.structs.get(type_name).ok_or_else(|| {
        HedlError::schema(
            format!("type '{}' not defined in inline child list", type_name),
            line_num,
        )
    })?;

    // Find the parent list frame - we need a parent row to attach children to
    let parent_list_idx = find_parent_list_for_inline_children(stack, indent, line_num)?;

    // Get parent list info for validation
    let parent_type_name = {
        let frame = &stack[parent_list_idx];
        match frame {
            Frame::List { type_name, .. } => type_name.clone(),
            _ => unreachable!(),
        }
    };

    // Validate NEST relationship exists
    let child_types = header.nests.get(&parent_type_name).ok_or_else(|| {
        HedlError::schema(
            format!(
                "no NEST rule for parent type '{}' to child type '{}'",
                parent_type_name, type_name
            ),
            line_num,
        )
    })?;

    // Use iter().any() to avoid allocating a String for the contains check
    if !child_types.iter().any(|s| s == type_name) {
        return Err(HedlError::schema(
            format!(
                "type '{}' is not a declared child of '{}' (allowed: {:?})",
                type_name, parent_type_name, child_types
            ),
            line_num,
        ));
    }

    // SECURITY: Check NEST depth
    let current_depth = stack
        .iter()
        .filter(|f| matches!(f, Frame::List { .. }))
        .count();

    if current_depth >= limits.max_nest_depth {
        return Err(HedlError::security(
            format!(
                "NEST hierarchy depth {} exceeds maximum allowed depth {}",
                current_depth + 1,
                limits.max_nest_depth
            ),
            line_num,
        ));
    }

    // Parse each child row and collect nodes
    let mut child_nodes = Vec::with_capacity(declared_count);
    let mut last_row_values: Option<Vec<Value>> = None;

    for child_csv in &child_rows {
        let child_csv = child_csv.trim();
        if child_csv.is_empty() {
            continue;
        }

        // Parse CSV
        let fields =
            parse_csv_row(child_csv).map_err(|e| HedlError::syntax(e.to_string(), line_num))?;

        // Validate shape
        if fields.len() != child_schema.len() {
            return Err(HedlError::shape(
                format!(
                    "expected {} columns for type '{}', got {}",
                    child_schema.len(),
                    type_name,
                    fields.len()
                ),
                line_num,
            ));
        }

        // Infer values
        let mut values = Vec::with_capacity(fields.len());
        for (col_idx, field) in fields.iter().enumerate() {
            let ctx = InferenceContext::for_matrix_cell(
                &header.aliases,
                col_idx,
                last_row_values.as_deref(),
                type_name,
            )
            .with_version(header.version)
            .with_null_char(header.null_char);

            let value = if field.is_quoted {
                infer_quoted_value(&field.value)
            } else {
                infer_value(&field.value, &ctx, line_num)?
            };

            values.push(value);
        }

        // Get ID from first column
        let id = match &values[0] {
            Value::String(s) => s.clone(),
            _ => {
                return Err(HedlError::semantic(
                    "ID column must be a string in inline child",
                    line_num,
                ));
            }
        };

        // Register node ID
        register_node(type_registries, type_name, &id, line_num, limits)?;

        // Check node count limit
        *node_count = node_count
            .checked_add(1)
            .ok_or_else(|| HedlError::security("node count overflow", line_num))?;
        if *node_count > limits.max_nodes {
            return Err(HedlError::security(
                format!("too many nodes: exceeds limit of {}", limits.max_nodes),
                line_num,
            ));
        }

        // Create node
        let node = Node::new(type_name, &*id, values.clone());
        child_nodes.push(node);

        // Update last_row_values for ditto support within inline children
        last_row_values = Some(values);
    }

    // Attach children to the most recent parent row
    if let Frame::List { list, .. } = &mut stack[parent_list_idx] {
        if let Some(parent_node) = list.last_mut() {
            let children = parent_node
                .children
                .get_or_insert_with(|| Box::new(BTreeMap::new()));
            children
                .entry(type_name.to_string())
                .or_default()
                .extend(child_nodes);
        } else {
            return Err(HedlError::orphan_row(
                "inline child list has no parent row to attach to",
                line_num,
            ));
        }
    }

    Ok(())
}

/// Find the parent list frame for inline children.
/// Inline children are indented ONE LEVEL DEEPER than parent rows.
pub(super) fn find_parent_list_for_inline_children(
    stack: &[Frame],
    indent: usize,
    line_num: usize,
) -> HedlResult<usize> {
    // Look for a list frame where indent = row_indent + 1 (one level deeper than parent rows)
    for (idx, frame) in stack.iter().enumerate().rev() {
        if let Frame::List {
            row_indent, list, ..
        } = frame
        {
            // Inline children are one level deeper than parent rows
            if indent == *row_indent + 1 {
                // Must have a parent row to attach to
                if list.is_empty() {
                    return Err(HedlError::orphan_row(
                        "inline child list has no parent row",
                        line_num,
                    ));
                }
                return Ok(idx);
            }
        }
    }

    Err(HedlError::syntax(
        "inline child list outside of list context",
        line_num,
    ))
}

/// Finds the appropriate list frame for a matrix row at the given indent level.
///
/// This function performs critical depth checking to prevent stack overflow attacks
/// via deeply nested NEST hierarchies. When a child row is detected (indent = parent + 1),
/// it validates that adding a new NEST level would not exceed `max_nest_depth`.
///
/// # Security
///
/// **DoS Prevention**: Without depth limits, an attacker could craft a HEDL document
/// with thousands of nested NEST levels, causing stack overflow or excessive memory
/// consumption during parsing. The depth check prevents this attack vector.
///
/// # Parameters
///
/// - `stack`: The parsing stack containing current frame hierarchy
/// - `indent`: Indentation level of the current matrix row
/// - `line_num`: Line number for error reporting
/// - `header`: Document header containing NEST rules and schemas
/// - `limits`: Security limits including `max_nest_depth`
///
/// # Returns
///
/// Returns the index of the list frame where this row should be added.
///
/// # Errors
///
/// - `HedlError::Security` if nesting depth exceeds `limits.max_nest_depth`
/// - `HedlError::OrphanRow` if child row has no parent or no NEST rule exists
/// - `HedlError::Schema` if child type is not defined
/// - `HedlError::Syntax` if row is outside list context
///
/// # Examples
///
/// ```text
/// # Valid nested structure within depth limit
/// TYPE Person id name
/// TYPE Address street city
/// NEST Person Address
///
/// Person
/// 1, Alice    # depth 0
///   1, Main St, NYC    # depth 1 - child of Person row
/// ```
pub(super) fn find_list_frame(
    stack: &mut Vec<Frame>,
    indent: usize,
    line_num: usize,
    header: &crate::header::Header,
    limits: &Limits,
    field_count: Option<usize>,
) -> HedlResult<usize> {
    // Look for a list frame where this indent makes sense
    for (idx, frame) in stack.iter().enumerate().rev() {
        if let Frame::List {
            row_indent,
            type_name,
            list,
            ..
        } = frame
        {
            if indent == *row_indent {
                // Peer row - check if field count matches current schema
                if let Some(fc) = field_count {
                    let schema = header.structs.get(type_name);
                    if schema.is_some_and(|s| s.len() == fc) {
                        // Field count matches, use this frame
                        return Ok(idx);
                    }
                    // Field count doesn't match - look for a sibling type that matches
                    // Find the actual parent frame (the one with row_indent = indent - 1)
                    let parent_row_indent = indent - 1;
                    let parent_frame = stack.iter().rev().find(|f| {
                        matches!(f, Frame::List { row_indent: ri, .. } if *ri == parent_row_indent)
                    });
                    if let Some(Frame::List {
                        type_name: parent_type,
                        ..
                    }) = parent_frame
                    {
                        if let Some(child_types) = header.nests.get(parent_type) {
                            // Find a child type whose schema matches the field count
                            let matching_type = child_types
                                .iter()
                                .find(|ct| header.structs.get(*ct).is_some_and(|s| s.len() == fc));
                            if let Some(new_type) = matching_type {
                                // Found a matching type - create a new sibling list frame
                                // SAFETY: find() predicate verified new_type exists in header.structs
                                let new_schema = header
                                    .structs
                                    .get(new_type)
                                    .expect("matching_type exists in header.structs")
                                    .clone();
                                stack.push(Frame::List {
                                    row_indent: indent,
                                    type_name: new_type.clone(),
                                    schema: new_schema,
                                    last_row_values: None,
                                    list: Vec::new(),
                                    key: new_type.clone(),
                                    count_hint: None,
                                });
                                return Ok(stack.len() - 1);
                            }
                        }
                    }
                }
                // Fallback to current frame (field count will be validated later)
                return Ok(idx);
            } else if indent == *row_indent + 1 {
                // Child row - need NEST rule
                // Check if there's a parent row to attach to
                if list.is_empty() {
                    return Err(HedlError::orphan_row(
                        "child row has no parent row",
                        line_num,
                    ));
                }

                let child_types = header.nests.get(type_name).ok_or_else(|| {
                    HedlError::orphan_row(
                        format!("no NEST rule for parent type '{}'", type_name),
                        line_num,
                    )
                })?;

                // With multiple possible child types, we need context to disambiguate.
                // For inline lists (@Type#N:), the type is explicit and handled elsewhere.
                // For block rows (| data), we use field count to match the schema.
                let child_type = if child_types.len() == 1 {
                    &child_types[0]
                } else if let Some(fc) = field_count {
                    // Multiple child types: find one whose schema column count matches
                    let matching_type = child_types.iter().find(|ct| {
                        header
                            .structs
                            .get(*ct)
                            .is_some_and(|schema| schema.len() == fc)
                    });
                    matching_type.unwrap_or(&child_types[0])
                } else {
                    // No field count available, use first type
                    &child_types[0]
                };

                // Get child schema
                let child_schema = header.structs.get(child_type).ok_or_else(|| {
                    HedlError::schema(format!("child type '{}' not defined", child_type), line_num)
                })?;

                // SECURITY: Check NEST depth before pushing child frame to prevent DoS
                // Count current depth by counting List frames in the stack
                // Each List frame represents one level in the NEST hierarchy
                let current_depth = stack
                    .iter()
                    .filter(|f| matches!(f, Frame::List { .. }))
                    .count();

                if current_depth >= limits.max_nest_depth {
                    return Err(HedlError::security(
                        format!(
                            "NEST hierarchy depth {} exceeds maximum allowed depth {}",
                            current_depth + 1,
                            limits.max_nest_depth
                        ),
                        line_num,
                    ));
                }

                // Push a new list frame for the child
                stack.push(Frame::List {
                    row_indent: indent,
                    type_name: child_type.clone(),
                    schema: child_schema.clone(),
                    last_row_values: None,
                    list: Vec::new(),
                    key: child_type.clone(),
                    count_hint: None, // Child lists from NEST don't have count hints
                });

                return Ok(stack.len() - 1);
            }
        }
    }

    Err(HedlError::syntax(
        "matrix row outside of list context",
        line_num,
    ))
}
