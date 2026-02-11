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

//! Parser context and frame stack management.

use crate::document::{Item, MatrixList};
use std::collections::BTreeMap;

/// Parser frame for tracking the parsing context stack.
#[derive(Debug)]
pub(super) enum Frame {
    Root {
        object: BTreeMap<String, Item>,
    },
    Object {
        indent: usize,
        key: String,
        object: BTreeMap<String, Item>,
    },
    List {
        row_indent: usize,
        type_name: String,
        schema: Vec<String>,
        last_row_values: Option<Vec<crate::value::Value>>,
        list: Vec<crate::document::Node>,
        key: String,
        count_hint: Option<usize>,
    },
}

/// Pop frames from the stack based on indentation.
pub(super) fn pop_frames(stack: &mut Vec<Frame>, current_indent: usize) {
    while stack.len() > 1 {
        // SAFETY: stack.len() > 1 guarantees at least one element exists
        let should_pop = match stack.last().expect("stack has at least 2 elements") {
            Frame::Root { .. } => false,
            Frame::Object { indent, .. } => current_indent <= *indent,
            Frame::List { row_indent, .. } => current_indent < *row_indent,
        };

        if should_pop {
            // SAFETY: stack.len() > 1 guarantees pop succeeds
            let frame = stack.pop().expect("stack has at least 2 elements");
            attach_frame_to_parent(stack, frame);
        } else {
            break;
        }
    }
}

/// Attach a completed frame to its parent in the stack.
pub(super) fn attach_frame_to_parent(stack: &mut [Frame], frame: Frame) {
    match frame {
        Frame::Object { key, object, .. } => {
            let item = Item::Object(object);
            insert_into_parent(stack, key, item);
        }
        Frame::List {
            key,
            type_name,
            schema,
            list,
            count_hint,
            ..
        } => {
            let mut matrix_list = if let Some(count) = count_hint {
                MatrixList::with_count_hint(type_name, schema, count)
            } else {
                MatrixList::new(type_name, schema)
            };
            matrix_list.rows = list;
            insert_into_parent(stack, key, Item::List(matrix_list));
        }
        Frame::Root { .. } => {}
    }
}

/// Insert an item into the parent frame.
pub(super) fn insert_into_parent(stack: &mut [Frame], key: String, item: Item) {
    if let Some(parent) = stack.last_mut() {
        match parent {
            Frame::Root { object } | Frame::Object { object, .. } => {
                // Note: max_object_keys limit check is performed at a higher level
                // during parsing, not here, to provide better error context
                object.insert(key, item);
            }
            Frame::List { list, .. } => {
                // Attach children to the last node in the list
                if let Some(parent_node) = list.last_mut() {
                    if let Item::List(child_list) = item {
                        let children = parent_node
                            .children
                            .get_or_insert_with(|| Box::new(BTreeMap::new()));
                        children
                            .entry(child_list.type_name.clone())
                            .or_default()
                            .extend(child_list.rows);
                    }
                }
            }
        }
    }
}
