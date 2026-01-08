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

//! Basic usage example of hedl-csv for bidirectional CSV ↔ HEDL conversion.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_csv::{from_csv, to_csv, FromCsvConfig, ToCsvConfig};

fn main() {
    println!("=== HEDL to CSV Conversion ===\n");
    hedl_to_csv_example();

    println!("\n=== CSV to HEDL Conversion ===\n");
    csv_to_hedl_example();

    println!("\n=== Round-trip Conversion ===\n");
    round_trip_example();

    println!("\n=== Custom Configuration ===\n");
    custom_config_example();
}

/// Example: Convert HEDL document to CSV
fn hedl_to_csv_example() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Person",
        vec![
            "name".to_string(),
            "age".to_string(),
            "email".to_string(),
            "active".to_string(),
        ],
    );

    list.add_row(Node::new(
        "Person",
        "1",
        vec![
            Value::String("Alice Johnson".to_string()),
            Value::Int(30),
            Value::String("alice@example.com".to_string()),
            Value::Bool(true),
        ],
    ));

    list.add_row(Node::new(
        "Person",
        "2",
        vec![
            Value::String("Bob Smith".to_string()),
            Value::Int(25),
            Value::String("bob@example.com".to_string()),
            Value::Bool(false),
        ],
    ));

    list.add_row(Node::new(
        "Person",
        "3",
        vec![
            Value::String("Charlie Brown".to_string()),
            Value::Int(35),
            Value::Null, // No email
            Value::Bool(true),
        ],
    ));

    doc.root.insert("people".to_string(), Item::List(list));

    let csv_output = to_csv(&doc).expect("Failed to convert to CSV");
    println!("HEDL document converted to CSV:\n{}", csv_output);
}

/// Example: Convert CSV to HEDL document
fn csv_to_hedl_example() {
    let csv_data = r#"id,product,price,quantity,in_stock
1,Laptop,999.99,10,true
2,Mouse,19.99,50,true
3,Keyboard,49.99,0,false
4,Monitor,299.99,15,true
"#;

    println!("Input CSV:\n{}", csv_data);

    let doc = from_csv(
        csv_data,
        "Product",
        &["product", "price", "quantity", "in_stock"],
    )
    .expect("Failed to parse CSV");

    println!(
        "Parsed document version: {}.{}",
        doc.version.0, doc.version.1
    );

    if let Some(item) = doc.get("products") {
        if let Some(list) = item.as_list() {
            println!("Found {} products:", list.rows.len());
            for row in &list.rows {
                println!(
                    "  - ID: {}, Product: {}, Price: {}, Quantity: {}, In Stock: {}",
                    row.id, row.fields[0], row.fields[1], row.fields[2], row.fields[3]
                );
            }
        }
    }
}

/// Example: Round-trip conversion (HEDL → CSV → HEDL)
fn round_trip_example() {
    // Create original document
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new(
        "Task",
        vec![
            "title".to_string(),
            "priority".to_string(),
            "completed".to_string(),
        ],
    );

    list.add_row(Node::new(
        "Task",
        "t1",
        vec![
            Value::String("Write documentation".to_string()),
            Value::Int(1),
            Value::Bool(true),
        ],
    ));

    list.add_row(Node::new(
        "Task",
        "t2",
        vec![
            Value::String("Fix bugs".to_string()),
            Value::Int(2),
            Value::Bool(false),
        ],
    ));

    doc.root.insert("tasks".to_string(), Item::List(list));

    // Convert to CSV
    let csv = to_csv(&doc).expect("Failed to convert to CSV");
    println!("Step 1 - HEDL to CSV:\n{}", csv);

    // Convert back to HEDL
    let doc2 =
        from_csv(&csv, "Task", &["title", "priority", "completed"]).expect("Failed to parse CSV");

    println!("Step 2 - CSV back to HEDL:");
    if let Some(item) = doc2.get("tasks") {
        if let Some(list) = item.as_list() {
            println!("  Recovered {} tasks", list.rows.len());
            for (i, row) in list.rows.iter().enumerate() {
                println!(
                    "  Task {}: ID={}, Title={}, Priority={}, Completed={}",
                    i + 1,
                    row.id,
                    row.fields[0],
                    row.fields[1],
                    row.fields[2]
                );
            }
        }
    }
}

/// Example: Custom configuration
fn custom_config_example() {
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["value".to_string()]);

    list.add_row(Node::new("Item", "1", vec![Value::Int(100)]));
    list.add_row(Node::new("Item", "2", vec![Value::Int(200)]));

    doc.root.insert("items".to_string(), Item::List(list));

    // Use tab delimiter
    let config = ToCsvConfig {
        delimiter: b'\t',
        ..Default::default()
    };
    let tsv = hedl_csv::to_csv_with_config(&doc, config).expect("Failed to convert to TSV");
    println!("Tab-separated output:\n{}", tsv);

    // Without headers
    let config = ToCsvConfig {
        include_headers: false,
        ..Default::default()
    };
    let csv_no_headers = hedl_csv::to_csv_with_config(&doc, config)
        .expect("Failed to convert to CSV without headers");
    println!("CSV without headers:\n{}", csv_no_headers);

    // Parse TSV with custom config
    let tsv_data = "id\tvalue\n1\t100\n2\t200\n";
    let config = FromCsvConfig {
        delimiter: b'\t',
        ..Default::default()
    };
    let doc2 = hedl_csv::from_csv_with_config(tsv_data, "Item", &["value"], config)
        .expect("Failed to parse TSV");

    println!("Parsed TSV document with {} items", doc2.root.len());
}
