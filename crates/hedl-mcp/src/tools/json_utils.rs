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

//! JSON serialization utilities for HEDL data structures.

use hedl_core::{Document, Item, Node, Value};
use serde_json::{json, Value as JsonValue};
use std::collections::BTreeMap;

/// Count entities in a document by type.
pub fn count_entities(doc: &Document) -> JsonValue {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();

    for item in doc.root.values() {
        count_item_entities(item, &mut counts);
    }

    json!(counts)
}

fn count_item_entities(item: &Item, counts: &mut BTreeMap<String, usize>) {
    match item {
        Item::List(list) => {
            *counts.entry(list.type_name.clone()).or_default() += list.rows.len();
            for node in &list.rows {
                count_node_entities(node, counts);
            }
        }
        Item::Object(obj) => {
            for child in obj.values() {
                count_item_entities(child, counts);
            }
        }
        Item::Scalar(_) => {}
    }
}

fn count_node_entities(node: &Node, counts: &mut BTreeMap<String, usize>) {
    for children in node.children.values() {
        for child in children {
            *counts.entry(child.type_name.clone()).or_default() += 1;
            count_node_entities(child, counts);
        }
    }
}

/// Convert a Node to JSON representation.
pub fn node_to_json(node: &Node, schema: &[String], include_children: bool) -> JsonValue {
    let mut obj = json!({
        "type": node.type_name,
        "id": node.id,
    });

    // Add fields
    let mut fields = serde_json::Map::new();
    for (i, value) in node.fields.iter().enumerate() {
        if i < schema.len() {
            fields.insert(schema[i].clone(), value_to_json(value));
        }
    }
    obj["fields"] = JsonValue::Object(fields);

    if include_children && !node.children.is_empty() {
        let mut children = serde_json::Map::new();
        for (child_type, child_nodes) in &node.children {
            children.insert(
                child_type.clone(),
                JsonValue::Array(
                    child_nodes
                        .iter()
                        .map(|n| {
                            json!({
                                "id": n.id,
                                "type": n.type_name
                            })
                        })
                        .collect(),
                ),
            );
        }
        obj["children"] = JsonValue::Object(children);
    }

    obj
}

/// Convert a HEDL Value to JSON.
pub fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Int(i) => json!(i),
        Value::Float(f) => json!(f),
        Value::String(s) => JsonValue::String(s.clone()),
        Value::Reference(r) => {
            if let Some(ref t) = r.type_name {
                json!(format!("@{}:{}", t, r.id))
            } else {
                json!(format!("@{}", r.id))
            }
        }
        Value::Tensor(t) => json!({
            "tensor": {
                "shape": t.shape(),
                "data": t.flatten()
            }
        }),
        Value::Expression(e) => json!(format!("$({})", e)),
    }
}

/// Get a placeholder schema for a type (in a real implementation, this would
/// look up the schema from the document).
pub fn doc_schema_for_type(_type_name: &str) -> Vec<String> {
    // In a real implementation, we'd look this up from the document
    vec!["id".to_string()]
}
