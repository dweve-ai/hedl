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

//! Parser utility functions.

use super::context::{attach_frame_to_parent, Frame};
use crate::document::Item;
use crate::error::{HedlError, HedlResult};
use crate::limits::Limits;
use std::collections::BTreeMap;

/// Validate indent for adding a child element to the current frame.
pub(super) fn validate_indent_for_child(
    stack: &[Frame],
    indent: usize,
    line_num: usize,
) -> HedlResult<()> {
    let expected = match stack.last() {
        Some(Frame::Root { .. }) => 0,
        Some(Frame::Object {
            indent: parent_indent,
            ..
        }) => parent_indent + 1,
        Some(Frame::List { row_indent: _, .. }) => {
            return Err(HedlError::syntax(
                "cannot add key-value inside list context",
                line_num,
            ));
        }
        None => 0,
    };

    if indent != expected {
        return Err(HedlError::syntax(
            format!("expected indent level {}, got {}", expected, indent),
            line_num,
        ));
    }

    Ok(())
}

/// Validate indent for nested list declarations inside a list context.
/// Unlike scalar key-values, nested list declarations ARE allowed inside lists.
/// Returns the parent list frame index if valid, or error if invalid.
pub(super) fn validate_nested_list_indent(
    stack: &[Frame],
    indent: usize,
    line_num: usize,
) -> HedlResult<Option<usize>> {
    // Check if we're inside a list context
    for (idx, frame) in stack.iter().enumerate().rev() {
        match frame {
            Frame::List {
                row_indent, list, ..
            } => {
                // Nested list declaration should be at row_indent + 1 (child level)
                if indent == *row_indent + 1 {
                    // Must have a parent row to attach to
                    if list.is_empty() {
                        return Err(HedlError::orphan_row(
                            "nested list declaration has no parent row",
                            line_num,
                        ));
                    }
                    return Ok(Some(idx));
                }
            }
            Frame::Root { .. } => {
                if indent == 0 {
                    return Ok(None); // Normal top-level list
                }
            }
            Frame::Object {
                indent: obj_indent, ..
            } => {
                if indent == obj_indent + 1 {
                    return Ok(None); // Normal list inside object
                }
            }
        }
    }

    Err(HedlError::syntax(
        format!(
            "invalid indent level {} for nested list declaration",
            indent
        ),
        line_num,
    ))
}

/// Check for duplicate keys and enforce security limits.
///
/// This function validates that:
/// 1. The key is not already present in the current object
/// 2. The object doesn't exceed max_object_keys limit
/// 3. The total number of keys across all objects doesn't exceed max_total_keys limit
///
/// # Security
///
/// The total_keys counter prevents DoS attacks where an attacker creates many small
/// objects, each under the max_object_keys limit, but collectively consuming excessive
/// memory. This provides defense-in-depth against memory exhaustion attacks.
pub(super) fn check_duplicate_key(
    stack: &[Frame],
    key: &str,
    line_num: usize,
    limits: &Limits,
    total_keys: &mut usize,
) -> HedlResult<()> {
    let object_opt = match stack.last() {
        Some(Frame::Root { object }) | Some(Frame::Object { object, .. }) => Some(object),
        _ => None,
    };

    if let Some(object) = object_opt {
        // Check for duplicate key
        if object.contains_key(key) {
            return Err(HedlError::semantic(
                format!("duplicate key: {}", key),
                line_num,
            ));
        }

        // Security: Enforce max_object_keys limit to prevent memory exhaustion per object
        if object.len() >= limits.max_object_keys {
            return Err(HedlError::security(
                format!(
                    "object has too many keys: {} (max: {})",
                    object.len() + 1,
                    limits.max_object_keys
                ),
                line_num,
            ));
        }

        // Security: Enforce max_total_keys limit to prevent cumulative memory exhaustion
        *total_keys = total_keys
            .checked_add(1)
            .ok_or_else(|| HedlError::security("total key count overflow", line_num))?;

        if *total_keys > limits.max_total_keys {
            return Err(HedlError::security(
                format!(
                    "too many total keys: {} exceeds limit {}",
                    *total_keys, limits.max_total_keys
                ),
                line_num,
            ));
        }
    }

    Ok(())
}

/// Insert an item into the current frame (root or object).
pub(super) fn insert_into_current(stack: &mut [Frame], key: String, item: Item) {
    if let Some(Frame::Root { object } | Frame::Object { object, .. }) = stack.last_mut() {
        object.insert(key, item);
    }
}

/// Parse a quoted string value, handling escaped quotes.
pub(super) fn parse_quoted_string(s: &str, line_num: usize) -> HedlResult<String> {
    if !s.starts_with('"') {
        return Err(HedlError::syntax("expected quoted string", line_num));
    }

    let mut result = String::new();
    let mut chars = s[1..].chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            if chars.peek() == Some(&'"') {
                // Escaped quote
                chars.next();
                result.push('"');
            } else {
                // End of string
                return Ok(result);
            }
        } else {
            result.push(ch);
        }
    }

    Err(HedlError::syntax("unclosed quoted string", line_num))
}

/// Finalize the parsing stack and extract the root object.
pub(super) fn finalize_stack(mut stack: Vec<Frame>) -> HedlResult<BTreeMap<String, Item>> {
    // Per SPEC Section 14.5: Detect truncated input.
    // Check only the DEEPEST (last) non-Root frame for truncation.
    // Intermediate frames will be empty until children are attached during pop.
    // Only if the deepest frame is an empty Object do we have actual truncation.
    // Note: Empty lists declared with @TypeName are allowed.
    if stack.len() > 1 {
        if let Some(Frame::Object { key, object, .. }) = stack.last() {
            if object.is_empty() {
                return Err(HedlError::syntax(
                    format!("truncated input: object '{}' has no children", key),
                    0,
                ));
            }
        }
    }

    // Pop all frames back to root
    while stack.len() > 1 {
        // SAFETY: stack.len() > 1 guarantees pop succeeds
        let frame = stack.pop().expect("stack has at least 2 elements");
        attach_frame_to_parent(&mut stack, frame);
    }

    // Extract root object
    match stack.pop() {
        Some(Frame::Root { object }) => Ok(object),
        _ => Ok(BTreeMap::new()),
    }
}
