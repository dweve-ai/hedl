// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Fixture counting utilities.
//!
//! Functions for counting nodes and references in test fixtures for verification.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use std::collections::BTreeMap;


/// Count nodes in a document.
pub fn count_nodes(doc: &Document) -> usize {
    let mut count = 0;
    for item in doc.root.values() {
        if let Item::List(list) = item {
            count += count_nodes_in_list(list);
        }
    }
    count
}

fn count_nodes_in_list(list: &MatrixList) -> usize {
    let mut count = list.rows.len();
    for row in &list.rows {
        count += count_children(&row.children);
    }
    count
}

fn count_children(children: &BTreeMap<String, Vec<Node>>) -> usize {
    let mut count = 0;
    for nodes in children.values() {
        count += nodes.len();
        for node in nodes {
            count += count_children(&node.children);
        }
    }
    count
}

/// Count references in a document.
pub fn count_references(doc: &Document) -> usize {
    let mut count = 0;
    for item in doc.root.values() {
        match item {
            Item::Scalar(Value::Reference(_)) => count += 1,
            Item::List(list) => {
                for row in &list.rows {
                    count += count_refs_in_node(row);
                }
            }
            _ => {}
        }
    }
    count
}

fn count_refs_in_node(node: &Node) -> usize {
    let mut count = 0;
    for field in &node.fields {
        if matches!(field, Value::Reference(_)) {
            count += 1;
        }
    }
    for children in node.children.values() {
        for child in children {
            count += count_refs_in_node(child);
        }
    }
    count
}
