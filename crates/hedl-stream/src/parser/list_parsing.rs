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

//! List and matrix row parsing
//!
//! Handles parsing of:
//! - List start declarations (@Type or @Type[schema])
//! - Matrix rows (| csv, values)
//! - Child blocks (@Type#N: or @Type#N:|inline|rows)
//! - Inline children

use super::context::Context;
use super::helpers::split_inline_rows;
use super::value_inference::infer_value;
use crate::error::{StreamError, StreamResult};
use crate::event::{HeaderInfo, NodeEvent, NodeInfo};
use hedl_core::lex::{is_valid_key_token, is_valid_type_name, strip_comment};
use hedl_core::Value;
use std::collections::VecDeque;

/// Type alias for list context lookup result: (`type_name`, schema, optional `last_node` info)
type ListContextResult = (String, Vec<String>, Option<(String, String)>);

/// Check if a string looks like a list start declaration.
#[inline]
pub(crate) fn is_list_start(s: &str) -> bool {
    let s = s.trim();
    if !s.starts_with('@') {
        return false;
    }
    let rest = &s[1..];
    let type_end = rest
        .find(|c: char| c == '[' || c.is_whitespace())
        .unwrap_or(rest.len());
    let type_name = &rest[..type_end];
    is_valid_type_name(type_name)
}

/// Parse a list start declaration (@Type or @Type[schema]).
pub(crate) fn parse_list_start(
    s: &str,
    line_num: usize,
    header: Option<&HeaderInfo>,
) -> StreamResult<(String, Vec<String>)> {
    let s = s.trim();
    let rest = &s[1..]; // Skip @

    if let Some(bracket_pos) = rest.find('[') {
        // Inline schema:@TypeName[col1, col2]
        let type_name = &rest[..bracket_pos];
        if !is_valid_type_name(type_name) {
            return Err(StreamError::syntax(
                line_num,
                format!("invalid type name: {type_name}"),
            ));
        }

        let bracket_end = rest
            .find(']')
            .ok_or_else(|| StreamError::syntax(line_num, "missing ']'"))?;

        let cols_str = &rest[bracket_pos + 1..bracket_end];
        let mut columns = Vec::new();

        for part in cols_str.split(',') {
            let col = part.trim();
            if col.is_empty() {
                continue;
            }
            // Validate column name
            if !is_valid_key_token(col) {
                return Err(StreamError::syntax(
                    line_num,
                    format!("invalid column name: {col}"),
                ));
            }
            columns.push(col.to_string());
        }

        // Check for empty schema
        if columns.is_empty() {
            return Err(StreamError::syntax(line_num, "empty inline schema"));
        }

        // Check against declared schema if type exists in header
        if let Some(header) = header {
            if let Some(declared) = header.structs.get(type_name) {
                if declared != &columns {
                    return Err(StreamError::schema(
                        line_num,
                        format!("inline schema for '{type_name}' doesn't match declared schema"),
                    ));
                }
            }
        }

        Ok((type_name.to_string(), columns))
    } else {
        // Reference to declared schema:@TypeName
        let type_name = rest.trim();
        if !is_valid_type_name(type_name) {
            return Err(StreamError::syntax(
                line_num,
                format!("invalid type name: {type_name}"),
            ));
        }

        let header = header.ok_or_else(|| StreamError::Header("header not parsed".to_string()))?;

        let schema = header
            .structs
            .get(type_name)
            .ok_or_else(|| StreamError::schema(line_num, format!("undefined type: {type_name}")))?;

        Ok((type_name.to_string(), schema.clone()))
    }
}

/// Parse row prefix to extract optional child count and CSV content.
///
/// Handles `|[N]` syntax where N is the expected child count.
/// Returns (Option<`child_count`>, `csv_content`).
pub(crate) fn parse_row_prefix(content: &str) -> (Option<usize>, &str) {
    // Content is already after the leading |
    // Check for [N] pattern at start
    if content.starts_with('[') {
        if let Some(bracket_end) = content.find(']') {
            let count_str = &content[1..bracket_end];
            if let Ok(count) = count_str.parse::<usize>() {
                // Count 0 is valid - means row has no children (empty parent)
                // Skip [N] and any following space
                let data = content[bracket_end + 1..].trim_start();
                return (Some(count), data);
            }
            // Invalid count format - fall through and treat as regular content
        }
    }

    // No child count prefix
    (None, content)
}

/// Parse a matrix row (| csv, values).
pub(crate) fn parse_matrix_row(
    content: &str,
    indent: usize,
    line_num: usize,
    stack: &mut Vec<Context>,
    prev_row: &mut Option<Vec<Value>>,
    header: Option<&HeaderInfo>,
) -> StreamResult<NodeEvent> {
    // Parse row prefix to extract optional child count and CSV content
    let (child_count, csv_content) = parse_row_prefix(content);
    let content = strip_comment(csv_content).trim();

    // Find active list context
    let (type_name, schema, parent_info) = find_list_context(stack, indent, line_num, header)?;

    // Parse HEDL matrix row (comma-separated values after the |)
    // Use hedl_row parser for proper CSV-like parsing
    let fields = hedl_core::lex::parse_csv_row(content)
        .map_err(|e| StreamError::syntax(line_num, format!("row parse error: {e}")))?;

    // Validate shape
    if fields.len() != schema.len() {
        return Err(StreamError::ShapeMismatch {
            line: line_num,
            expected: schema.len(),
            got: fields.len(),
        });
    }

    // Infer values with ditto handling
    let mut values = Vec::with_capacity(fields.len());
    for (col_idx, field) in fields.iter().enumerate() {
        let value = if field.value == "^" {
            // Check version: ditto operator is forbidden in HEDL v2.0+
            let header =
                header.ok_or_else(|| StreamError::Header("header not parsed".to_string()))?;
            if header.version >= (2, 0) {
                return Err(StreamError::syntax(
                    line_num,
                    "Ditto operator (^) is forbidden in HEDL v2.0",
                ));
            }
            // Ditto - use previous row's value (pre-v2.0 only)
            prev_row
                .as_ref()
                .and_then(|prev| prev.get(col_idx).cloned())
                .unwrap_or(Value::Null)
        } else if field.is_quoted {
            Value::String(field.value.clone().into())
        } else {
            infer_value(&field.value, line_num, header)?
        };
        values.push(value);
    }

    // Get ID from first column
    let id = match &values[0] {
        Value::String(s) => s.to_string(),
        _ => return Err(StreamError::syntax(line_num, "ID column must be a string")),
    };

    // Update prev_row for ditto handling
    *prev_row = Some(values.clone());

    // Calculate depth as number of list contexts minus 1 (0-indexed nesting level)
    let depth = stack
        .iter()
        .filter(|ctx| matches!(ctx, Context::List { .. }))
        .count()
        .saturating_sub(1);

    // Build node info
    let mut node = NodeInfo::new(type_name.clone(), id, values, depth, line_num);

    if let Some((parent_type, parent_id)) = parent_info {
        node = node.with_parent(parent_type, parent_id);
    }

    if let Some(count) = child_count {
        node = node.with_child_count(count);
    }

    Ok(NodeEvent::Node(node))
}

/// Find the active list context for the given indent level.
fn find_list_context(
    stack: &mut Vec<Context>,
    indent: usize,
    line_num: usize,
    header: Option<&HeaderInfo>,
) -> StreamResult<ListContextResult> {
    let header = header.ok_or_else(|| StreamError::Header("header not parsed".to_string()))?;

    for ctx in stack.iter().rev() {
        if let Context::List {
            type_name,
            schema,
            row_indent,
            last_node,
            ..
        } = ctx
        {
            if indent == *row_indent {
                // Peer row
                return Ok((type_name.clone(), schema.clone(), None));
            } else if indent == *row_indent + 1 {
                // Child row
                let parent_info = last_node
                    .clone()
                    .ok_or_else(|| StreamError::orphan_row(line_num, "child row has no parent"))?;

                let child_types = header.nests.get(type_name).ok_or_else(|| {
                    StreamError::orphan_row(
                        line_num,
                        format!("no NEST rule for parent type '{type_name}'"),
                    )
                })?;

                // Find the first child type that has a defined schema
                let (child_type, child_schema) = child_types
                    .iter()
                    .find_map(|ct| header.structs.get(ct).map(|schema| (ct, schema)))
                    .ok_or_else(|| {
                        StreamError::schema(
                            line_num,
                            format!(
                                "no schema defined for child types {:?} of parent '{}'",
                                child_types, type_name
                            ),
                        )
                    })?;

                // Push child list context
                stack.push(Context::List {
                    key: child_type.clone(),
                    type_name: child_type.clone(),
                    schema: child_schema.clone(),
                    row_indent: indent,
                    count: 0,
                    last_node: None,
                });

                return Ok((child_type.clone(), child_schema.clone(), Some(parent_info)));
            }
        }
    }

    Err(StreamError::syntax(
        line_num,
        "matrix row outside of list context",
    ))
}

/// Try to parse a child block line: `@Type#N:` or `@Type#N:|row1|row2|...|rowN`
pub(crate) fn try_parse_child_block(
    content: &str,
    indent: usize,
    line_num: usize,
    stack: &[Context],
    pending_events: &mut VecDeque<NodeEvent>,
    prev_row: &mut Option<Vec<Value>>,
    header: Option<&HeaderInfo>,
) -> StreamResult<Option<NodeEvent>> {
    // Pattern:@TypeName#N: or @TypeName#N:|...
    let content = content.trim();

    // Must start with @
    if !content.starts_with('@') {
        return Err(StreamError::syntax(
            line_num,
            format!("line starting with '@' must be a child block: {content}"),
        ));
    }

    let rest = &content[1..]; // Skip @

    // Find # separator - required for child block syntax
    let hash_pos = match rest.find('#') {
        Some(pos) => pos,
        None => {
            return Err(StreamError::syntax(
                line_num,
                format!("child block syntax requires @Type#N: format, got: {content}"),
            ))
        }
    };

    let type_name = &rest[..hash_pos];
    if !is_valid_type_name(type_name) {
        return Err(StreamError::syntax(
            line_num,
            format!("invalid type name in child block: {type_name}"),
        ));
    }

    let after_hash = &rest[hash_pos + 1..];

    // Find : separator - required for child block syntax
    let colon_pos = match after_hash.find(':') {
        Some(pos) => pos,
        None => {
            return Err(StreamError::syntax(
                line_num,
                format!("child block syntax requires @Type#N: format, got: {content}"),
            ))
        }
    };

    // Parse the count N
    let count_str = &after_hash[..colon_pos];
    let expected_count: usize = count_str.parse().map_err(|_| {
        StreamError::syntax(
            line_num,
            format!("invalid child count in child block: {count_str}"),
        )
    })?;

    let after_colon = &after_hash[colon_pos + 1..];

    // Get header for schema lookup and nesting validation
    let header = header
        .ok_or_else(|| StreamError::Header("header not parsed".to_string()))?
        .clone();

    // Get schema for child type
    let schema = header.structs.get(type_name).ok_or_else(|| {
        StreamError::schema(
            line_num,
            format!("no schema defined for child type '{type_name}'"),
        )
    })?;

    // Find parent info from list context
    // Child block is one indent deeper than parent row
    let parent_info = find_parent_for_child_block(stack, indent, type_name, line_num, &header)?;

    // Calculate depth for inline children
    let depth = stack
        .iter()
        .filter(|ctx| matches!(ctx, Context::List { .. }))
        .count();

    // Check if inline children are present (starts with |)
    if after_colon.starts_with('|') {
        // Inline children:@Type#N:|row1|row2|...|rowN
        let params = InlineChildParams {
            content: after_colon,
            type_name,
            schema,
            expected_count,
            parent_info,
            line_num,
            header: &header,
            depth,
        };
        parse_inline_children(&params, pending_events, prev_row)
    } else if after_colon.trim().is_empty() {
        // Block form: expect next N lines to be child rows
        // Block form tracking requires stateful row collection across multiple lines
        Err(StreamError::syntax(
            line_num,
            "block form child lists (@Type#N: without inline rows) not yet supported in streaming parser",
        ))
    } else {
        Err(StreamError::syntax(
            line_num,
            format!("invalid content after child block colon: {after_colon}"),
        ))
    }
}

/// Find parent info for a child block line.
///
/// A child block line is indented one level deeper than its parent row.
fn find_parent_for_child_block(
    stack: &[Context],
    indent: usize,
    child_type: &str,
    line_num: usize,
    header: &HeaderInfo,
) -> StreamResult<(String, String)> {
    // Child block is at indent L, parent row is at indent L-1
    if indent == 0 {
        return Err(StreamError::syntax(
            line_num,
            "child block cannot be at root level",
        ));
    }

    let parent_indent = indent - 1;

    // Find the list context that has rows at parent_indent
    for ctx in stack.iter().rev() {
        if let Context::List {
            type_name: parent_type,
            row_indent,
            last_node,
            ..
        } = ctx
        {
            if *row_indent == parent_indent {
                // Found the parent list context
                let (parent_type_name, parent_id) = last_node.as_ref().ok_or_else(|| {
                    StreamError::orphan_row(line_num, "child block has no parent row")
                })?;

                // Validate nesting rule
                let allowed_children = header.nests.get(parent_type).ok_or_else(|| {
                    StreamError::syntax(
                        line_num,
                        format!("no NEST rule for parent type '{parent_type}'"),
                    )
                })?;

                if !allowed_children.contains(&child_type.to_string()) {
                    return Err(StreamError::syntax(
                        line_num,
                        format!("'{child_type}' is not a declared child type of '{parent_type}'"),
                    ));
                }

                return Ok((parent_type_name.clone(), parent_id.clone()));
            }
        }
    }

    Err(StreamError::syntax(
        line_num,
        "child block outside of list context",
    ))
}

/// Bundled parameters for inline child parsing.
struct InlineChildParams<'a> {
    content: &'a str,
    type_name: &'a str,
    schema: &'a [String],
    expected_count: usize,
    parent_info: (String, String),
    line_num: usize,
    header: &'a HeaderInfo,
    depth: usize,
}

/// Parse inline children from `|row1|row2|...|rowN` format.
fn parse_inline_children(
    params: &InlineChildParams<'_>,
    pending_events: &mut VecDeque<NodeEvent>,
    prev_row: &mut Option<Vec<Value>>,
) -> StreamResult<Option<NodeEvent>> {
    // Split by | at top level (respecting quotes and brackets)
    let rows = split_inline_rows(params.content)?;

    // Validate count
    if rows.len() != params.expected_count {
        return Err(StreamError::syntax(
            params.line_num,
            format!(
                "inline child count mismatch: expected {}, got {}",
                params.expected_count,
                rows.len()
            ),
        ));
    }

    // Handle zero count case
    if params.expected_count == 0 {
        return Ok(None);
    }

    let depth = params.depth;

    let (parent_type, parent_id) = &params.parent_info;

    // Parse each row and create NodeInfo
    let mut nodes = Vec::with_capacity(rows.len());
    for row_content in rows {
        let row_content = row_content.trim();
        if row_content.is_empty() {
            continue;
        }

        // Parse CSV row
        let fields = hedl_core::lex::parse_csv_row(row_content)
            .map_err(|e| StreamError::syntax(params.line_num, format!("row parse error: {e}")))?;

        // Validate shape
        if fields.len() != params.schema.len() {
            return Err(StreamError::ShapeMismatch {
                line: params.line_num,
                expected: params.schema.len(),
                got: fields.len(),
            });
        }

        // Infer values (no ditto in inline children for v2.0)
        let mut values = Vec::with_capacity(fields.len());
        for field in &fields {
            let value = if field.value == "^" {
                // Check version: ditto forbidden in v2.0+
                if params.header.version >= (2, 0) {
                    return Err(StreamError::syntax(
                        params.line_num,
                        "Ditto operator (^) is forbidden in HEDL v2.0",
                    ));
                }
                // For older versions, ditto in inline children uses previous inline child
                prev_row
                    .as_ref()
                    .and_then(|prev| prev.get(values.len()).cloned())
                    .unwrap_or(Value::Null)
            } else if field.is_quoted {
                Value::String(field.value.clone().into())
            } else {
                infer_value(&field.value, params.line_num, Some(params.header))?
            };
            values.push(value);
        }

        // Get ID from first column
        let id = match &values[0] {
            Value::String(s) => s.to_string(),
            _ => {
                return Err(StreamError::syntax(
                    params.line_num,
                    "ID column must be a string in inline child",
                ))
            }
        };

        // Update prev_row for ditto handling in older versions
        *prev_row = Some(values.clone());

        let node = NodeInfo::new(
            params.type_name.to_string(),
            id,
            values,
            depth,
            params.line_num,
        )
        .with_parent(parent_type.clone(), parent_id.clone());

        nodes.push(NodeEvent::Node(node));
    }

    // Return first node, queue the rest
    if nodes.is_empty() {
        return Ok(None);
    }

    let first = nodes.remove(0);
    for node in nodes {
        pending_events.push_back(node);
    }

    Ok(Some(first))
}
