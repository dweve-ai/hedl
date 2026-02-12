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

//! Parser context and state management
//!
//! Handles the parser's context stack for tracking nested structures
//! (root, objects, lists) and managing indentation-based scope.

use crate::error::{StreamError, StreamResult};
use crate::event::NodeEvent;
use hedl_core::Value;
use std::collections::VecDeque;

/// Parser state tracking
#[derive(Debug)]
pub(crate) struct ParserState {
    /// Stack of active contexts.
    pub stack: Vec<Context>,
    /// Previous row values for ditto handling (deprecated in v2.0+).
    pub prev_row: Option<Vec<Value>>,
    /// Pending events from inline children parsing.
    pub pending_events: VecDeque<NodeEvent>,
}

impl Default for ParserState {
    fn default() -> Self {
        Self {
            stack: vec![Context::Root],
            prev_row: None,
            pending_events: VecDeque::new(),
        }
    }
}

/// Context type for the parser stack
#[derive(Debug, Clone)]
pub(crate) enum Context {
    Root,
    Object {
        key: String,
        indent: usize,
    },
    List {
        key: String,
        type_name: String,
        schema: Vec<String>,
        row_indent: usize,
        count: usize,
        last_node: Option<(String, String)>, // (type, id)
    },
}

/// Pop contexts from the stack based on indentation changes.
///
/// Returns a `ListEnd` or `ObjectEnd` event if a context was popped.
pub(crate) fn pop_contexts(
    stack: &mut Vec<Context>,
    current_indent: usize,
) -> StreamResult<Option<NodeEvent>> {
    while stack.len() > 1 {
        // Safe: loop condition guarantees stack has elements
        let should_pop = match stack.last().expect("stack has elements") {
            Context::Root => false,
            Context::Object { indent, .. } => current_indent <= *indent,
            Context::List { row_indent, .. } => current_indent < *row_indent,
        };

        if should_pop {
            // Safe: loop condition guarantees stack has elements
            let ctx = stack.pop().expect("stack has elements");
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

/// Validate that the indent is correct for a key-value or object start.
///
/// Mirrors `validate_indent_for_child` from hedl-core:
/// - Root context: expects indent 0
/// - Object context: expects `parent_indent` + 1
/// - List context: key-value not allowed (only list declarations)
pub(crate) fn validate_indent_for_key_value(
    stack: &[Context],
    indent: usize,
    line_num: usize,
) -> StreamResult<()> {
    let expected = match stack.last() {
        Some(Context::Root) | None => 0,
        Some(Context::Object {
            indent: parent_indent,
            ..
        }) => parent_indent + 1,
        Some(Context::List { .. }) => {
            return Err(StreamError::syntax(
                line_num,
                "cannot add key-value inside list context",
            ));
        }
    };

    if indent != expected {
        return Err(StreamError::syntax(
            line_num,
            format!("expected indent level {expected}, got {indent}"),
        ));
    }

    Ok(())
}

/// Update the last_node info in the matching list context.
pub(crate) fn update_list_context(stack: &mut [Context], type_name: &str, id: &str) {
    for ctx in stack.iter_mut().rev() {
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
