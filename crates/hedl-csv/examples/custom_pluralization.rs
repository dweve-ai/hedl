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

//! Example demonstrating custom pluralization using the list_key parameter.
//!
//! This example shows how to handle irregular plural forms and custom list naming
//! when importing CSV data into HEDL documents.

use hedl_csv::{from_csv_with_config, to_csv_list, FromCsvConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Pluralization Examples ===\n");

    // Example 1: Irregular Plural - Person -> people
    println!("Example 1: Irregular Plural (Person -> people)");
    let csv_people = r#"id,name,age,occupation
1,Alice Johnson,30,Engineer
2,Bob Smith,25,Designer
3,Carol Williams,35,Manager"#;

    let config = FromCsvConfig {
        list_key: Some("people".to_string()),
        ..Default::default()
    };

    let doc = from_csv_with_config(csv_people, "Person", &["name", "age", "occupation"], config)?;

    // Access using custom plural
    let people = doc.get("people").expect("'people' list should exist");
    let list = people.as_list().expect("Should be a matrix list");
    println!("  Type name: {}", list.type_name);
    println!("  List key: people");
    println!("  Rows: {}", list.rows.len());
    println!("  First person: {}", list.rows[0].fields[1]);
    println!();

    // Example 2: Multiple Irregular Plurals
    println!("Example 2: Multiple Irregular Plurals");

    let csv_children = "id,name\n1,Emma\n2,Liam\n";
    let config = FromCsvConfig {
        list_key: Some("children".to_string()),
        ..Default::default()
    };
    let doc_children = from_csv_with_config(csv_children, "Child", &["name"], config)?;

    let csv_mice = "id,species\n1,House Mouse\n2,Field Mouse\n";
    let config = FromCsvConfig {
        list_key: Some("mice".to_string()),
        ..Default::default()
    };
    let doc_mice = from_csv_with_config(csv_mice, "Mouse", &["species"], config)?;

    let csv_teeth = "id,type\n1,Molar\n2,Incisor\n";
    let config = FromCsvConfig {
        list_key: Some("teeth".to_string()),
        ..Default::default()
    };
    let doc_teeth = from_csv_with_config(csv_teeth, "Tooth", &["type"], config)?;

    println!("  Child -> children: {} items", doc_children.get("children").unwrap().as_list().unwrap().rows.len());
    println!("  Mouse -> mice: {} items", doc_mice.get("mice").unwrap().as_list().unwrap().rows.len());
    println!("  Tooth -> teeth: {} items", doc_teeth.get("teeth").unwrap().as_list().unwrap().rows.len());
    println!();

    // Example 3: Collective Nouns
    println!("Example 3: Collective Nouns");

    let csv_data = "id,value,timestamp\n1,42,2024-01-01\n2,43,2024-01-02\n";
    let config = FromCsvConfig {
        list_key: Some("dataset".to_string()),
        ..Default::default()
    };
    let doc_dataset = from_csv_with_config(csv_data, "Data", &["value", "timestamp"], config)?;

    println!("  Data -> dataset: {} items", doc_dataset.get("dataset").unwrap().as_list().unwrap().rows.len());
    println!();

    // Example 4: Case-Sensitive Custom Names
    println!("Example 4: Case-Sensitive Custom Names");

    let csv_items = "id,sku,price\n1,ITEM-001,29.99\n2,ITEM-002,39.99\n";
    let config = FromCsvConfig {
        list_key: Some("ProductCatalog".to_string()),
        ..Default::default()
    };
    let doc_products = from_csv_with_config(csv_items, "Product", &["sku", "price"], config)?;

    println!("  Product -> ProductCatalog (case preserved)");
    println!("  Items: {}", doc_products.get("ProductCatalog").unwrap().as_list().unwrap().rows.len());
    println!();

    // Example 5: Round-Trip with Custom Plurals
    println!("Example 5: Round-Trip Compatibility");

    let original_csv = "id,name,email\n1,Alice,alice@example.com\n2,Bob,bob@example.com\n";
    let config = FromCsvConfig {
        list_key: Some("people".to_string()),
        ..Default::default()
    };
    let doc = from_csv_with_config(original_csv, "Person", &["name", "email"], config)?;

    // Export using the custom key
    let exported_csv = to_csv_list(&doc, "people")?;
    println!("  Original CSV:");
    println!("{}", original_csv.trim());
    println!("\n  Exported CSV:");
    println!("{}", exported_csv.trim());
    println!();

    // Example 6: Complex Naming Conventions
    println!("Example 6: Complex Naming Conventions");

    let csv_events = "id,event_type,timestamp\n1,login,2024-01-01\n2,logout,2024-01-02\n";

    // Use snake_case with version suffix
    let config = FromCsvConfig {
        list_key: Some("event_log_v2".to_string()),
        ..Default::default()
    };
    let doc_events = from_csv_with_config(csv_events, "Event", &["event_type", "timestamp"], config)?;

    println!("  Event -> event_log_v2 (custom naming convention)");
    println!("  Items: {}", doc_events.get("event_log_v2").unwrap().as_list().unwrap().rows.len());
    println!();

    // Example 7: Demonstrating Default Behavior
    println!("Example 7: Default Behavior (No Custom Key)");

    let csv_users = "id,username\n1,alice\n2,bob\n";
    let doc_users = hedl_csv::from_csv(csv_users, "User", &["username"])?;

    println!("  User -> users (default simple pluralization)");
    println!("  Items: {}", doc_users.get("users").unwrap().as_list().unwrap().rows.len());
    println!();

    println!("=== Summary ===");
    println!("The list_key parameter allows you to:");
    println!("  • Handle irregular plural forms (person/people, child/children)");
    println!("  • Use collective nouns (data/dataset)");
    println!("  • Implement custom naming conventions");
    println!("  • Preserve specific casing requirements");
    println!("  • Maintain compatibility with existing data models");

    Ok(())
}
