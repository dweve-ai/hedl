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

//! Demonstration of English pluralization in TOON conversion
//!
//! This example shows how irregular English plurals are correctly handled
//! when converting HEDL documents to TOON format.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_toon::hedl_to_toon;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HEDL to TOON Pluralization Demo ===\n");

    // Example 1: Children
    println!("Example 1: Child → Children");
    println!("{}", "-".repeat(50));
    let doc1 = create_parent_child_doc();
    let toon1 = hedl_to_toon(&doc1)?;
    println!("{}\n", toon1);

    // Example 2: People
    println!("Example 2: Person → People");
    println!("{}", "-".repeat(50));
    let doc2 = create_team_person_doc();
    let toon2 = hedl_to_toon(&doc2)?;
    println!("{}\n", toon2);

    // Example 3: Mice
    println!("Example 3: Mouse → Mice");
    println!("{}", "-".repeat(50));
    let doc3 = create_lab_mouse_doc();
    let toon3 = hedl_to_toon(&doc3)?;
    println!("{}\n", toon3);

    // Example 4: Teeth
    println!("Example 4: Tooth → Teeth");
    println!("{}", "-".repeat(50));
    let doc4 = create_mouth_tooth_doc();
    let toon4 = hedl_to_toon(&doc4)?;
    println!("{}\n", toon4);

    // Example 5: Cacti
    println!("Example 5: Cactus → Cacti");
    println!("{}", "-".repeat(50));
    let doc5 = create_garden_cactus_doc();
    let toon5 = hedl_to_toon(&doc5)?;
    println!("{}\n", toon5);

    // Example 6: Regular plural (for comparison)
    println!("Example 6: Item → Items (Regular Plural)");
    println!("{}", "-".repeat(50));
    let doc6 = create_order_item_doc();
    let toon6 = hedl_to_toon(&doc6)?;
    println!("{}\n", toon6);

    Ok(())
}

fn create_parent_child_doc() -> Document {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Child".to_string(), vec!["id".to_string(), "name".to_string()]);

    let mut parent_list = MatrixList::new("Parent", vec!["id".to_string(), "name".to_string()]);
    let mut parent_node = Node::new("Parent", "p1", vec![
        Value::String("p1".to_string()),
        Value::String("John".to_string()),
    ]);

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
    doc
}

fn create_team_person_doc() -> Document {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Person".to_string(), vec!["id".to_string(), "name".to_string()]);

    let mut team_list = MatrixList::new("Team", vec!["id".to_string(), "name".to_string()]);
    let mut team_node = Node::new("Team", "t1", vec![
        Value::String("t1".to_string()),
        Value::String("Engineering".to_string()),
    ]);

    team_node.children.insert("Person".to_string(), vec![
        Node::new("Person", "p1", vec![
            Value::String("p1".to_string()),
            Value::String("Alice".to_string()),
        ]),
        Node::new("Person", "p2", vec![
            Value::String("p2".to_string()),
            Value::String("Bob".to_string()),
        ]),
        Node::new("Person", "p3", vec![
            Value::String("p3".to_string()),
            Value::String("Carol".to_string()),
        ]),
    ]);

    team_list.add_row(team_node);
    doc.root.insert("teams".to_string(), Item::List(team_list));
    doc
}

fn create_lab_mouse_doc() -> Document {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Mouse".to_string(), vec!["id".to_string(), "strain".to_string()]);

    let mut lab_list = MatrixList::new("Lab", vec!["id".to_string(), "name".to_string()]);
    let mut lab_node = Node::new("Lab", "lab1", vec![
        Value::String("lab1".to_string()),
        Value::String("Genetics Lab".to_string()),
    ]);

    lab_node.children.insert("Mouse".to_string(), vec![
        Node::new("Mouse", "m1", vec![
            Value::String("m1".to_string()),
            Value::String("C57BL/6".to_string()),
        ]),
        Node::new("Mouse", "m2", vec![
            Value::String("m2".to_string()),
            Value::String("BALB/c".to_string()),
        ]),
    ]);

    lab_list.add_row(lab_node);
    doc.root.insert("labs".to_string(), Item::List(lab_list));
    doc
}

fn create_mouth_tooth_doc() -> Document {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Tooth".to_string(), vec!["id".to_string(), "position".to_string()]);

    let mut patient_list = MatrixList::new("Patient", vec!["id".to_string(), "name".to_string()]);
    let mut patient_node = Node::new("Patient", "pt1", vec![
        Value::String("pt1".to_string()),
        Value::String("John Doe".to_string()),
    ]);

    patient_node.children.insert("Tooth".to_string(), vec![
        Node::new("Tooth", "t1", vec![
            Value::String("t1".to_string()),
            Value::String("upper-left-1".to_string()),
        ]),
        Node::new("Tooth", "t2", vec![
            Value::String("t2".to_string()),
            Value::String("upper-right-1".to_string()),
        ]),
    ]);

    patient_list.add_row(patient_node);
    doc.root.insert("patients".to_string(), Item::List(patient_list));
    doc
}

fn create_garden_cactus_doc() -> Document {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Cactus".to_string(), vec!["id".to_string(), "species".to_string()]);

    let mut garden_list = MatrixList::new("Garden", vec!["id".to_string(), "name".to_string()]);
    let mut garden_node = Node::new("Garden", "g1", vec![
        Value::String("g1".to_string()),
        Value::String("Desert Garden".to_string()),
    ]);

    garden_node.children.insert("Cactus".to_string(), vec![
        Node::new("Cactus", "c1", vec![
            Value::String("c1".to_string()),
            Value::String("Saguaro".to_string()),
        ]),
        Node::new("Cactus", "c2", vec![
            Value::String("c2".to_string()),
            Value::String("Barrel".to_string()),
        ]),
        Node::new("Cactus", "c3", vec![
            Value::String("c3".to_string()),
            Value::String("Prickly Pear".to_string()),
        ]),
    ]);

    garden_list.add_row(garden_node);
    doc.root.insert("gardens".to_string(), Item::List(garden_list));
    doc
}

fn create_order_item_doc() -> Document {
    let mut doc = Document::new((1, 0));
    doc.structs.insert("Item".to_string(), vec!["id".to_string(), "product".to_string()]);

    let mut order_list = MatrixList::new("Order", vec!["id".to_string(), "customer".to_string()]);
    let mut order_node = Node::new("Order", "o1", vec![
        Value::String("o1".to_string()),
        Value::String("ACME Corp".to_string()),
    ]);

    order_node.children.insert("Item".to_string(), vec![
        Node::new("Item", "i1", vec![
            Value::String("i1".to_string()),
            Value::String("Widget A".to_string()),
        ]),
        Node::new("Item", "i2", vec![
            Value::String("i2".to_string()),
            Value::String("Gadget B".to_string()),
        ]),
    ]);

    order_list.add_row(order_node);
    doc.root.insert("orders".to_string(), Item::List(order_list));
    doc
}
