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


// Example: JSON Schema Generation from HEDL
//
// This example demonstrates how to generate JSON Schema (Draft 7)
// from HEDL documents with various configurations.

use hedl_core::parse;
use hedl_json::schema_gen::{generate_schema, SchemaConfig};

/// Helper to parse HEDL from string
fn parse_hedl(input: &str) -> hedl_core::Document {
    // Prepend HEDL header if not present, or separate header from body if needed
    let hedl = if input.contains("%VERSION") || input.starts_with("%HEDL") {
        input.to_string()
    } else if input.contains("%STRUCT") || input.contains("%NEST") {
        // Has directives but no VERSION - add VERSION and ensure separator
        let (header, body) = if input.contains("---") {
            let parts: Vec<&str> = input.splitn(2, "---").collect();
            (parts[0].trim().to_string(), parts.get(1).map(|s| s.trim().to_string()).unwrap_or_default())
        } else {
            // Extract directives to header
            let mut header_lines = Vec::new();
            let mut body_lines = Vec::new();
            for line in input.lines() {
                if line.trim().starts_with('%') {
                    header_lines.push(line.to_string());
                } else {
                    body_lines.push(line.to_string());
                }
            }
            (header_lines.join("\n"), body_lines.join("\n"))
        };
        format!("%VERSION: 1.0\n{}\n---\n{}", header, body)
    } else {
        format!("%VERSION: 1.0\n---\n{}", input)
    };
    parse(hedl.as_bytes()).unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Basic Schema Generation
    println!("=== Example 1: Basic Schema ===\n");
    let hedl = r#"
name: Alice
age: 30
active: true
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema(&doc, &SchemaConfig::default())?;
    println!("{}\n", schema);

    // Example 2: Schema with %STRUCT:definitions
    println!("=== Example 2: Schema with %STRUCT:===\n");
    let hedl = r#"
%STRUCT: User: [id, name, email]
users: @User
  |u1, Alice, alice@example.com
  |u2, Bob, bob@example.com
"#;

    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder()
        .title("User API Schema")
        .description("Schema for user management API")
        .build();
    let schema = generate_schema(&doc, &config)?;
    println!("{}\n", schema);

    // Example 3: Schema with Nested Types (NEST)
    println!("=== Example 3: Schema with %NEST:===\n");
    let hedl = r#"
%STRUCT: Team: [id, name]
%STRUCT: Member: [id, name, role]
%NEST: Team > Member

teams: @Team
  |t1, Engineering
  |t2, Design
"#;

    let doc = parse_hedl(hedl);
    let schema = generate_schema(&doc, &SchemaConfig::default())?;
    println!("{}\n", schema);

    // Example 4: Strict Schema with Examples
    println!("=== Example 4: Strict Schema with Examples ===\n");
    let hedl = r#"
%STRUCT: Product: [id, name, price, in_stock]
products: @Product
  |p1, Widget, 19.99, true
  |p2, Gadget, 29.99, false
"#;

    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder()
        .title("Product Catalog Schema")
        .schema_id("https://api.example.com/product-schema.json")
        .strict(true)
        .include_examples(true)
        .build();
    let schema = generate_schema(&doc, &config)?;
    println!("{}\n", schema);

    // Example 5: Complex Nested Structure
    println!("=== Example 5: Complex Nested Structure ===\n");
    let hedl = r#"
%STRUCT: Organization: [id, name, founded]
%STRUCT: Department: [id, name, budget]
%STRUCT: Employee: [id, name, email, hire_date]

%NEST: Organization > Department
%NEST: Department > Employee

organizations: @Organization
  |org1, Acme Corp, 1990
"#;

    let doc = parse_hedl(hedl);
    let config = SchemaConfig::builder()
        .title("Organization Schema")
        .description("Hierarchical organization structure")
        .strict(true)
        .build();
    let schema = generate_schema(&doc, &config)?;
    println!("{}\n", schema);

    Ok(())
}
