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

//! Helper functions for HedlDocument entity operations.

use hedl_core::{Item, Node};

#[cfg(feature = "query-api")]
use hedl_core::Value;

#[cfg(feature = "query-api")]
use serde::Serialize;

/// Count entities in an item and its children.
pub(crate) fn count_item_entities(
    item: &Item,
    counts: &mut std::collections::BTreeMap<String, usize>,
) {
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

/// Count entities in a node and its children.
pub(crate) fn count_node_entities(
    node: &Node,
    counts: &mut std::collections::BTreeMap<String, usize>,
) {
    if let Some(children_map) = node.children() {
        for children in children_map.values() {
            for child in children {
                *counts.entry(child.type_name.clone()).or_default() += 1;
                count_node_entities(child, counts);
            }
        }
    }
}

#[cfg(feature = "query-api")]
#[derive(Serialize)]
pub(crate) struct EntityResult {
    #[serde(rename = "type")]
    pub(crate) type_name: String,
    pub(crate) id: String,
    pub(crate) fields: serde_json::Value,
}

#[cfg(feature = "query-api")]
pub(crate) fn find_entities(
    item: &Item,
    type_filter: &Option<String>,
    id_filter: &Option<String>,
    results: &mut Vec<EntityResult>,
) {
    match item {
        Item::List(list) => {
            let type_matches = type_filter.as_ref().map_or(true, |t| &list.type_name == t);

            for node in &list.rows {
                let id_matches = id_filter.as_ref().map_or(true, |i| &node.id == i);

                if type_matches && id_matches {
                    results.push(EntityResult {
                        type_name: node.type_name.clone(),
                        id: node.id.clone(),
                        fields: node_fields_to_json(&node.fields, &list.schema),
                    });
                }

                if let Some(children_map) = node.children() {
                    for children in children_map.values() {
                        for child in children {
                            find_node_entities(child, type_filter, id_filter, results);
                        }
                    }
                }
            }
        }
        Item::Object(obj) => {
            for child in obj.values() {
                find_entities(child, type_filter, id_filter, results);
            }
        }
        Item::Scalar(_) => {}
    }
}

#[cfg(feature = "query-api")]
pub(crate) fn find_node_entities(
    node: &Node,
    type_filter: &Option<String>,
    id_filter: &Option<String>,
    results: &mut Vec<EntityResult>,
) {
    let type_matches = type_filter.as_ref().map_or(true, |t| &node.type_name == t);
    let id_matches = id_filter.as_ref().map_or(true, |i| &node.id == i);

    if type_matches && id_matches {
        results.push(EntityResult {
            type_name: node.type_name.clone(),
            id: node.id.clone(),
            fields: node_fields_to_json(&node.fields, &[]),
        });
    }

    if let Some(children_map) = node.children() {
        for children in children_map.values() {
            for child in children {
                find_node_entities(child, type_filter, id_filter, results);
            }
        }
    }
}

#[cfg(feature = "query-api")]
pub(crate) fn node_fields_to_json(fields: &[Value], schema: &[String]) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    for (i, value) in fields.iter().enumerate() {
        let key = if i < schema.len() {
            schema[i].clone()
        } else {
            format!("field_{i}")
        };
        obj.insert(key, value_to_json(value));
    }
    serde_json::Value::Object(obj)
}

#[cfg(feature = "query-api")]
pub(crate) fn value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::json!(i),
        Value::Float(f) => serde_json::json!(f),
        Value::String(s) => serde_json::Value::String(s.to_string()),
        Value::Reference(r) => {
            if let Some(ref t) = r.type_name {
                serde_json::json!(format!("@{}:{}", t, r.id))
            } else {
                serde_json::json!(format!("@{}", r.id))
            }
        }
        Value::Tensor(t) => serde_json::json!({
            "shape": t.shape(),
            "data": t.flatten()
        }),
        Value::Expression(e) => serde_json::json!(format!("$({})", e)),
        Value::List(items) => {
            let arr: Vec<serde_json::Value> = items.iter().map(value_to_json).collect();
            serde_json::Value::Array(arr)
        }
    }
}
