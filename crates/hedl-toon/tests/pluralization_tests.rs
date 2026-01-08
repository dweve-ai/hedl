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

//! Integration tests for pluralization in TOON conversion
//!
//! Tests that child node arrays are correctly pluralized using
//! English irregular plural forms.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_toon::hedl_to_toon;

#[test]
fn test_child_pluralization() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Child".to_string(), vec!["id".to_string(), "name".to_string()]);

    // Create parent with children
    let mut parent_list = MatrixList::new("Parent", vec!["id".to_string(), "name".to_string()]);
    let mut parent_node = Node::new("Parent", "p1", vec![
        Value::String("p1".to_string()),
        Value::String("John".to_string()),
    ]);

    // Add children to parent
    parent_node.children.insert("Child".to_string(), vec![
        Node::new("Child", "c1", vec![
            Value::String("c1".to_string()),
            Value::String("Alice".to_string()),
        ]),
        Node::new("Child", "c2", vec![
            Value::String("c2".to_string()),
            Value::String("Bob".to_string()),
        ]),
    ]);

    parent_list.add_row(parent_node);
    doc.root.insert("parents".to_string(), Item::List(parent_list));

    let result = hedl_to_toon(&doc).unwrap();

    // Should use "children" not "childs"
    assert!(result.contains("children[2]{id,name}:"), "Expected 'children' but got:\n{}", result);
    assert!(!result.contains("childs"), "Should not contain 'childs':\n{}", result);
}

#[test]
fn test_person_pluralization() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Person".to_string(), vec!["id".to_string(), "name".to_string()]);

    let mut team_list = MatrixList::new("Team", vec!["id".to_string(), "name".to_string()]);
    let mut team_node = Node::new("Team", "t1", vec![
        Value::String("t1".to_string()),
        Value::String("Alpha Team".to_string()),
    ]);

    // Add people to team
    team_node.children.insert("Person".to_string(), vec![
        Node::new("Person", "p1", vec![
            Value::String("p1".to_string()),
            Value::String("Alice".to_string()),
        ]),
        Node::new("Person", "p2", vec![
            Value::String("p2".to_string()),
            Value::String("Bob".to_string()),
        ]),
    ]);

    team_list.add_row(team_node);
    doc.root.insert("teams".to_string(), Item::List(team_list));

    let result = hedl_to_toon(&doc).unwrap();

    // Should use "people" not "persons"
    assert!(result.contains("people[2]{id,name}:"), "Expected 'people' but got:\n{}", result);
    assert!(!result.contains("persons"), "Should not contain 'persons':\n{}", result);
}

#[test]
fn test_mouse_pluralization() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Mouse".to_string(), vec!["id".to_string(), "name".to_string()]);

    let mut lab_list = MatrixList::new("Lab", vec!["id".to_string(), "name".to_string()]);
    let mut lab_node = Node::new("Lab", "lab1", vec![
        Value::String("lab1".to_string()),
        Value::String("Research Lab".to_string()),
    ]);

    // Add mice to lab
    lab_node.children.insert("Mouse".to_string(), vec![
        Node::new("Mouse", "m1", vec![
            Value::String("m1".to_string()),
            Value::String("Mickey".to_string()),
        ]),
        Node::new("Mouse", "m2", vec![
            Value::String("m2".to_string()),
            Value::String("Minnie".to_string()),
        ]),
    ]);

    lab_list.add_row(lab_node);
    doc.root.insert("labs".to_string(), Item::List(lab_list));

    let result = hedl_to_toon(&doc).unwrap();

    // Should use "mice" not "mouses"
    assert!(result.contains("mice[2]{id,name}:"), "Expected 'mice' but got:\n{}", result);
    assert!(!result.contains("mouses"), "Should not contain 'mouses':\n{}", result);
}

#[test]
fn test_regular_pluralization_still_works() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Item".to_string(), vec!["id".to_string(), "name".to_string()]);

    let mut order_list = MatrixList::new("Order", vec!["id".to_string(), "name".to_string()]);
    let mut order_node = Node::new("Order", "o1", vec![
        Value::String("o1".to_string()),
        Value::String("Order 1".to_string()),
    ]);

    // Add items to order
    order_node.children.insert("Item".to_string(), vec![
        Node::new("Item", "i1", vec![
            Value::String("i1".to_string()),
            Value::String("Product A".to_string()),
        ]),
        Node::new("Item", "i2", vec![
            Value::String("i2".to_string()),
            Value::String("Product B".to_string()),
        ]),
    ]);

    order_list.add_row(order_node);
    doc.root.insert("orders".to_string(), Item::List(order_list));

    let result = hedl_to_toon(&doc).unwrap();

    // Should use regular plural "items"
    assert!(result.contains("items[2]{id,name}:"), "Expected 'items' but got:\n{}", result);
}

#[test]
fn test_tooth_pluralization() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Tooth".to_string(), vec!["id".to_string(), "position".to_string()]);

    let mut mouth_list = MatrixList::new("Mouth", vec!["id".to_string(), "name".to_string()]);
    let mut mouth_node = Node::new("Mouth", "m1", vec![
        Value::String("m1".to_string()),
        Value::String("Patient 1".to_string()),
    ]);

    // Add teeth to mouth
    mouth_node.children.insert("Tooth".to_string(), vec![
        Node::new("Tooth", "t1", vec![
            Value::String("t1".to_string()),
            Value::String("upper-left".to_string()),
        ]),
        Node::new("Tooth", "t2", vec![
            Value::String("t2".to_string()),
            Value::String("upper-right".to_string()),
        ]),
    ]);

    mouth_list.add_row(mouth_node);
    doc.root.insert("mouths".to_string(), Item::List(mouth_list));

    let result = hedl_to_toon(&doc).unwrap();

    // Should use "teeth" not "tooths"
    assert!(result.contains("teeth[2]{id,position}:"), "Expected 'teeth' but got:\n{}", result);
    assert!(!result.contains("tooths"), "Should not contain 'tooths':\n{}", result);
}

#[test]
fn test_ox_pluralization() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Ox".to_string(), vec!["id".to_string(), "name".to_string()]);

    let mut farm_list = MatrixList::new("Farm", vec!["id".to_string(), "name".to_string()]);
    let mut farm_node = Node::new("Farm", "f1", vec![
        Value::String("f1".to_string()),
        Value::String("Green Acres".to_string()),
    ]);

    // Add oxen to farm
    farm_node.children.insert("Ox".to_string(), vec![
        Node::new("Ox", "ox1", vec![
            Value::String("ox1".to_string()),
            Value::String("Babe".to_string()),
        ]),
        Node::new("Ox", "ox2", vec![
            Value::String("ox2".to_string()),
            Value::String("Paul".to_string()),
        ]),
    ]);

    farm_list.add_row(farm_node);
    doc.root.insert("farms".to_string(), Item::List(farm_list));

    let result = hedl_to_toon(&doc).unwrap();

    // Should use "oxen" not "oxs"
    assert!(result.contains("oxen[2]{id,name}:"), "Expected 'oxen' but got:\n{}", result);
    assert!(!result.contains("oxs"), "Should not contain 'oxs':\n{}", result);
}

#[test]
fn test_cactus_pluralization() {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Cactus".to_string(), vec!["id".to_string(), "species".to_string()]);

    let mut garden_list = MatrixList::new("Garden", vec!["id".to_string(), "name".to_string()]);
    let mut garden_node = Node::new("Garden", "g1", vec![
        Value::String("g1".to_string()),
        Value::String("Desert Garden".to_string()),
    ]);

    // Add cacti to garden
    garden_node.children.insert("Cactus".to_string(), vec![
        Node::new("Cactus", "c1", vec![
            Value::String("c1".to_string()),
            Value::String("Saguaro".to_string()),
        ]),
        Node::new("Cactus", "c2", vec![
            Value::String("c2".to_string()),
            Value::String("Barrel".to_string()),
        ]),
    ]);

    garden_list.add_row(garden_node);
    doc.root.insert("gardens".to_string(), Item::List(garden_list));

    let result = hedl_to_toon(&doc).unwrap();

    // Should use "cacti" not "cactuss" or "cactuses"
    assert!(result.contains("cacti[2]{id,species}:"), "Expected 'cacti' but got:\n{}", result);
    assert!(!result.contains("cactuss"), "Should not contain 'cactuss':\n{}", result);
    assert!(!result.contains("cactuses"), "Should not contain 'cactuses':\n{}", result);
}
