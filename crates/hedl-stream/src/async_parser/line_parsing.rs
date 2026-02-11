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

//! Line parsing and context management for async streaming parser.

use super::{
    AsyncStreamingParser, Context, HeaderInfo, ListContextResult, NodeEvent, NodeInfo, StreamError,
    StreamResult,
};
use crate::parser::helpers::split_inline_rows;
use hedl_core::lex::{is_valid_key_token, is_valid_type_name, strip_comment};
use hedl_core::Value;
use tokio::io::AsyncRead;

impl<R: AsyncRead + Unpin> AsyncStreamingParser<R> {
    pub(super) fn pop_contexts(
        &mut self,
        current_indent: usize,
    ) -> StreamResult<Option<NodeEvent>> {
        while self.state.stack.len() > 1 {
            let should_pop = match self.state.stack.last().expect("stack has elements") {
                Context::Root => false,
                Context::Object { indent, .. } => current_indent <= *indent,
                Context::List { row_indent, .. } => current_indent < *row_indent,
            };

            if should_pop {
                let ctx = self.state.stack.pop().expect("stack has elements");
                match ctx {
                    Context::List {
                        key,
                        type_name,
                        count,
                        ..
                    } => {
                        return Ok(Some(NodeEvent::ListEnd {
                            key,
                            type_name,
                            count,
                        }));
                    }
                    Context::Object { key, .. } => {
                        return Ok(Some(NodeEvent::ObjectEnd { key }));
                    }
                    Context::Root => {
                        // Root context should never be popped
                    }
                }
            } else {
                break;
            }
        }

        Ok(None)
    }

    pub(super) fn parse_line(
        &mut self,
        content: &str,
        indent: usize,
        line_num: usize,
    ) -> StreamResult<Option<NodeEvent>> {
        // Check for child block syntax BEFORE stripping comments
        // because @Type#N: uses # which would otherwise be treated as comment start
        if content.starts_with('@') && content.contains('#') {
            // Check if it looks like child block pattern: @Type#N:
            // We need to pass the original content to preserve the #N: syntax
            return self.try_parse_child_block(content, indent, line_num);
        }

        // Strip inline comment for all other line types
        let content = strip_comment(content);

        if let Some(row_content) = content.strip_prefix('|') {
            self.parse_matrix_row(row_content, indent, line_num)
        } else if content.starts_with('@') {
            // Child block without # - this is an error (requires #N: format)
            self.try_parse_child_block(content, indent, line_num)
        } else if let Some(colon_pos) = content.find(':') {
            let key = content[..colon_pos].trim();
            let after_colon = &content[colon_pos + 1..];

            if !is_valid_key_token(key) {
                return Err(StreamError::syntax(line_num, format!("invalid key: {key}")));
            }

            let after_colon_trimmed = after_colon.trim();

            if after_colon_trimmed.is_empty() {
                self.state.stack.push(Context::Object {
                    key: key.to_string(),
                    indent,
                });
                Ok(Some(NodeEvent::ObjectStart {
                    key: key.to_string(),
                    line: line_num,
                }))
            } else if after_colon_trimmed.starts_with('@')
                && self.is_list_start(after_colon_trimmed)
            {
                let (type_name, schema) = self.parse_list_start(after_colon_trimmed, line_num)?;

                self.state.stack.push(Context::List {
                    key: key.to_string(),
                    type_name: type_name.clone(),
                    schema: schema.clone(),
                    row_indent: indent + 1,
                    count: 0,
                    last_node: None,
                });

                self.state.prev_row = None;

                Ok(Some(NodeEvent::ListStart {
                    key: key.to_string(),
                    type_name,
                    schema,
                    line: line_num,
                }))
            } else {
                let value = self.infer_value(after_colon.trim(), line_num)?;
                Ok(Some(NodeEvent::Scalar {
                    key: key.to_string(),
                    value,
                    line: line_num,
                }))
            }
        } else {
            Err(StreamError::syntax(line_num, "expected ':' in line"))
        }
    }

    #[inline]
    pub(super) fn is_list_start(&self, s: &str) -> bool {
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

    pub(super) fn parse_list_start(
        &self,
        s: &str,
        line_num: usize,
    ) -> StreamResult<(String, Vec<String>)> {
        let s = s.trim();
        let rest = &s[1..];

        if let Some(bracket_pos) = rest.find('[') {
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
            let columns: Vec<String> = cols_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Ok((type_name.to_string(), columns))
        } else {
            let type_name = rest.trim();
            if !is_valid_type_name(type_name) {
                return Err(StreamError::syntax(
                    line_num,
                    format!("invalid type name: {type_name}"),
                ));
            }

            let header = self
                .header
                .as_ref()
                .ok_or_else(|| StreamError::Header("header not parsed".to_string()))?;

            let schema = header.structs.get(type_name).ok_or_else(|| {
                StreamError::schema(line_num, format!("undefined type: {type_name}"))
            })?;

            Ok((type_name.to_string(), schema.clone()))
        }
    }

    pub(super) fn parse_matrix_row(
        &mut self,
        content: &str,
        indent: usize,
        line_num: usize,
    ) -> StreamResult<Option<NodeEvent>> {
        let content = strip_comment(content).trim();

        let (type_name, schema, parent_info) = self.find_list_context(indent, line_num)?;

        let fields = hedl_core::lex::parse_csv_row(content)
            .map_err(|e| StreamError::syntax(line_num, format!("row parse error: {e}")))?;

        if fields.len() != schema.len() {
            return Err(StreamError::ShapeMismatch {
                line: line_num,
                expected: schema.len(),
                got: fields.len(),
            });
        }

        let mut values = Vec::with_capacity(fields.len());
        for (col_idx, field) in fields.iter().enumerate() {
            let value = if field.value == "^" {
                // Check version: ditto operator is forbidden in HEDL v2.0+
                let header = self
                    .header
                    .as_ref()
                    .ok_or_else(|| StreamError::Header("header not parsed".to_string()))?;
                if header.version >= (2, 0) {
                    return Err(StreamError::syntax(
                        line_num,
                        "Ditto operator (^) is forbidden in HEDL v2.0",
                    ));
                }
                // Ditto - use previous row's value (pre-v2.0 only)
                self.state
                    .prev_row
                    .as_ref()
                    .and_then(|prev| prev.get(col_idx).cloned())
                    .unwrap_or(Value::Null)
            } else if field.is_quoted {
                Value::String(field.value.clone().into())
            } else {
                self.infer_value(&field.value, line_num)?
            };
            values.push(value);
        }

        let id = match &values[0] {
            Value::String(s) => s.clone(),
            _ => return Err(StreamError::syntax(line_num, "ID column must be a string")),
        };

        self.update_list_context(&type_name, &id);
        self.state.prev_row = Some(values.clone());

        // Calculate depth as number of list contexts minus 1 (0-indexed nesting level)
        let depth = self
            .state
            .stack
            .iter()
            .filter(|ctx| matches!(ctx, Context::List { .. }))
            .count()
            .saturating_sub(1);

        let mut node = NodeInfo::new(type_name.clone(), id.to_string(), values, depth, line_num);

        if let Some((parent_type, parent_id)) = parent_info {
            node = node.with_parent(parent_type, parent_id);
        }

        Ok(Some(NodeEvent::Node(node)))
    }

    pub(super) fn find_list_context(
        &mut self,
        indent: usize,
        line_num: usize,
    ) -> StreamResult<ListContextResult> {
        let header = self
            .header
            .as_ref()
            .ok_or_else(|| StreamError::Header("header not parsed".to_string()))?;

        for ctx in self.state.stack.iter().rev() {
            if let Context::List {
                type_name,
                schema,
                row_indent,
                last_node,
                ..
            } = ctx
            {
                if indent == *row_indent {
                    return Ok((type_name.clone(), schema.clone(), None));
                } else if indent == *row_indent + 1 {
                    let parent_info = last_node.clone().ok_or_else(|| {
                        StreamError::orphan_row(line_num, "child row has no parent")
                    })?;

                    let child_types = header.nests.get(type_name).ok_or_else(|| {
                        StreamError::orphan_row(
                            line_num,
                            format!("no NEST rule for parent type '{type_name}'"),
                        )
                    })?;

                    // Use the first child type (for simple cases)
                    let child_type = child_types.first().ok_or_else(|| {
                        StreamError::schema(line_num, format!("empty NEST rule for '{type_name}'"))
                    })?;

                    let child_schema = header.structs.get(child_type).ok_or_else(|| {
                        StreamError::schema(
                            line_num,
                            format!("child type '{child_type}' not defined"),
                        )
                    })?;

                    self.state.stack.push(Context::List {
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

    pub(super) fn update_list_context(&mut self, type_name: &str, id: &str) {
        for ctx in self.state.stack.iter_mut().rev() {
            if let Context::List {
                type_name: ctx_type,
                last_node,
                count,
                ..
            } = ctx
            {
                if ctx_type == type_name {
                    *last_node = Some((type_name.to_string(), id.to_string()));
                    *count += 1;
                    break;
                }
            }
        }
    }

    /// Try to parse a child block line:@Type#N: or @Type#N:|...
    pub(super) fn try_parse_child_block(
        &mut self,
        content: &str,
        indent: usize,
        line_num: usize,
    ) -> StreamResult<Option<NodeEvent>> {
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

        // Validate count limit (max 5 inline children)
        if expected_count > 5 {
            return Err(StreamError::syntax(
                line_num,
                format!("inline child count {} exceeds maximum 5", expected_count),
            ));
        }

        let after_colon = &after_hash[colon_pos + 1..];

        // Get header for schema lookup and nesting validation
        let header = self
            .header
            .as_ref()
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
        let parent_info = self.find_parent_for_child_block(indent, type_name, line_num, &header)?;

        // Check if inline children are present (starts with |)
        if after_colon.starts_with('|') {
            self.parse_inline_children(
                after_colon,
                type_name,
                schema,
                expected_count,
                parent_info,
                line_num,
            )
        } else if after_colon.trim().is_empty() {
            // Block form not yet supported in async parser
            Err(StreamError::syntax(
                line_num,
                "block form child lists (@Type#N: without inline rows) not yet supported in async streaming parser",
            ))
        } else {
            Err(StreamError::syntax(
                line_num,
                format!("invalid content after child block colon: {after_colon}"),
            ))
        }
    }

    /// Find parent info for a child block line.
    pub(super) fn find_parent_for_child_block(
        &self,
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
        for ctx in self.state.stack.iter().rev() {
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
                            format!(
                                "'{child_type}' is not a declared child type of '{parent_type}'"
                            ),
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

    /// Parse inline children from `|row1|row2|...|rowN` format.
    pub(super) fn parse_inline_children(
        &mut self,
        content: &str,
        type_name: &str,
        schema: &[String],
        expected_count: usize,
        parent_info: (String, String),
        line_num: usize,
    ) -> StreamResult<Option<NodeEvent>> {
        // Split by | at top level (respecting quotes and brackets)
        let rows = split_inline_rows(content)?;

        // Validate count
        if rows.len() != expected_count {
            return Err(StreamError::syntax(
                line_num,
                format!(
                    "inline child count mismatch: expected {expected_count}, got {}",
                    rows.len()
                ),
            ));
        }

        // Handle zero count case
        if expected_count == 0 {
            return Ok(None);
        }

        // Calculate depth
        let depth = self
            .state
            .stack
            .iter()
            .filter(|ctx| matches!(ctx, Context::List { .. }))
            .count();

        let (parent_type, parent_id) = parent_info;

        // Parse each row and create NodeInfo
        let mut nodes = Vec::with_capacity(rows.len());
        for row_content in rows {
            let row_content = row_content.trim();
            if row_content.is_empty() {
                continue;
            }

            // Parse CSV row
            let fields = hedl_core::lex::parse_csv_row(row_content)
                .map_err(|e| StreamError::syntax(line_num, format!("row parse error: {e}")))?;

            // Validate shape
            if fields.len() != schema.len() {
                return Err(StreamError::ShapeMismatch {
                    line: line_num,
                    expected: schema.len(),
                    got: fields.len(),
                });
            }

            // Infer values (no ditto in inline children for v2.0)
            let header = self
                .header
                .as_ref()
                .ok_or_else(|| StreamError::Header("header not parsed".to_string()))?;

            let mut values = Vec::with_capacity(fields.len());
            for field in &fields {
                let value = if field.value == "^" {
                    // Check version: ditto forbidden in v2.0+
                    if header.version >= (2, 0) {
                        return Err(StreamError::syntax(
                            line_num,
                            "Ditto operator (^) is forbidden in HEDL v2.0",
                        ));
                    }
                    // For older versions, ditto in inline children uses previous inline child
                    self.state
                        .prev_row
                        .as_ref()
                        .and_then(|prev| prev.get(values.len()).cloned())
                        .unwrap_or(Value::Null)
                } else if field.is_quoted {
                    Value::String(field.value.clone().into())
                } else {
                    self.infer_value(&field.value, line_num)?
                };
                values.push(value);
            }

            // Get ID from first column
            let id = match &values[0] {
                Value::String(s) => s.to_string(),
                _ => {
                    return Err(StreamError::syntax(
                        line_num,
                        "ID column must be a string in inline child",
                    ))
                }
            };

            // Update prev_row for ditto handling in older versions
            self.state.prev_row = Some(values.clone());

            let node = NodeInfo::new(type_name.to_string(), id, values, depth, line_num)
                .with_parent(parent_type.clone(), parent_id.clone());

            nodes.push(NodeEvent::Node(node));
        }

        // Return first node, queue the rest
        if nodes.is_empty() {
            return Ok(None);
        }

        let first = nodes.remove(0);
        // Queue remaining nodes for subsequent next_event calls
        self.state.pending_events.extend(nodes);

        Ok(Some(first))
    }

    #[inline]
    pub(super) fn infer_value(&self, s: &str, line_num: usize) -> StreamResult<Value> {
        let s = s.trim();

        // Handle null values: empty, ~, or the keyword "null"
        if s.is_empty() || s == "~" || s == "null" {
            return Ok(Value::Null);
        }

        if s == "true" {
            return Ok(Value::Bool(true));
        }
        if s == "false" {
            return Ok(Value::Bool(false));
        }

        // List literal: (...)
        if s.starts_with('(') && s.ends_with(')') {
            match hedl_core::lex::parse_list_literal(s, 0) {
                Ok((lex_value, _)) => {
                    // Convert lex::Value::List to Value::List
                    if let hedl_core::lex::Value::List(items) = lex_value {
                        let converted_items: Result<Vec<Value>, StreamError> = items
                            .into_iter()
                            .map(|item| self.convert_lex_value(item, line_num))
                            .collect();
                        return Ok(Value::List(Box::new(converted_items?)));
                    }
                }
                Err(_) => {
                    // If list parsing fails, fall through to other inference
                }
            }
        }

        // Reference
        if let Some(ref_part) = s.strip_prefix('@') {
            if let Some(colon_pos) = ref_part.find(':') {
                let type_name = &ref_part[..colon_pos];
                let id = &ref_part[colon_pos + 1..];
                return Ok(Value::Reference(hedl_core::Reference {
                    type_name: Some(type_name.to_string().into()),
                    id: id.to_string().into(),
                }));
            }
            return Ok(Value::Reference(hedl_core::Reference {
                type_name: None,
                id: ref_part.to_string().into(),
            }));
        }

        // Alias
        if let Some(alias) = s.strip_prefix('$') {
            if let Some(header) = &self.header {
                if let Some(value) = header.aliases.get(alias) {
                    return Ok(Value::String(value.clone().into()));
                }
            }
            return Ok(Value::String(s.to_string().into()));
        }

        // Number
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Value::Int(i));
        }
        if let Ok(f) = s.parse::<f64>() {
            return Ok(Value::Float(f));
        }

        // Default to string
        Ok(Value::String(s.to_string().into()))
    }

    /// Convert a lex::Value to the main Value type.
    ///
    /// This handles the conversion from the lexer-level value representation
    /// to the document-level value representation used in streaming events.
    pub(super) fn convert_lex_value(
        &self,
        lex_val: hedl_core::lex::Value,
        line_num: usize,
    ) -> StreamResult<Value> {
        match lex_val {
            hedl_core::lex::Value::Null => Ok(Value::Null),
            hedl_core::lex::Value::Bool(b) => Ok(Value::Bool(b)),
            hedl_core::lex::Value::Int(i) => Ok(Value::Int(i)),
            hedl_core::lex::Value::Float(f) => Ok(Value::Float(f)),
            hedl_core::lex::Value::String(s) => Ok(Value::String(s.into_boxed_str())),
            hedl_core::lex::Value::Reference(r) => Ok(Value::Reference(hedl_core::Reference {
                type_name: r.type_name.map(|t| t.into_boxed_str()),
                id: r.id.into_boxed_str(),
            })),
            hedl_core::lex::Value::Expression(e) => Ok(Value::Expression(Box::new(e))),
            hedl_core::lex::Value::Tensor(_) => Err(StreamError::syntax(
                line_num,
                "tensors not supported in list literals",
            )),
            hedl_core::lex::Value::List(items) => {
                let converted_items: Result<Vec<Value>, StreamError> = items
                    .into_iter()
                    .map(|item| self.convert_lex_value(item, line_num))
                    .collect();
                Ok(Value::List(Box::new(converted_items?)))
            }
        }
    }
}
