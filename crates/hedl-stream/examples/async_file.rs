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

//! Async file processing example.
//!
//! Demonstrates reading HEDL files asynchronously from disk.
//!
//! Run with: cargo run --example async_file --features async

#[cfg(feature = "async")]
use hedl_stream::{AsyncStreamingParser, NodeEvent};

#[cfg(feature = "async")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::io::Cursor;

    println!("=== Async File Processing Example ===\n");

    let content = r#"
%VERSION: 1.0
%STRUCT: Employee: [id, name, department, salary]
---
employees: @Employee
  | emp1, Alice Johnson, Engineering, 95000
  | emp2, Bob Smith, Sales, 75000
  | emp3, Carol White, Engineering, 105000
  | emp4, David Brown, Marketing, 80000
  | emp5, Eve Davis, Engineering, 98000
"#;

    println!("Parsing data asynchronously...\n");

    // Create async parser from in-memory data
    // (In real usage, you'd use tokio::fs::File with the "fs" feature)
    let mut parser = AsyncStreamingParser::new(Cursor::new(content)).await?;

    // Process events
    let mut employees = Vec::new();

    while let Some(event) = parser.next_event().await? {
        if let NodeEvent::Node(node) = event {
            let name = node.get_field(1).unwrap();
            let dept = node.get_field(2).unwrap();
            let salary = node.get_field(3).unwrap();

            employees.push((
                node.id.clone(),
                name.clone(),
                dept.clone(),
                salary.clone(),
            ));
        }
    }

    // Display results
    println!("=== Employees (sorted by salary) ===\n");

    employees.sort_by(|a, b| {
        let salary_a = if let hedl_stream::Value::Int(v) = a.3 {
            v
        } else {
            0
        };
        let salary_b = if let hedl_stream::Value::Int(v) = b.3 {
            v
        } else {
            0
        };
        salary_b.cmp(&salary_a)
    });

    for (id, name, dept, salary) in employees {
        println!("{}: {} - {} (${:?})", id, name, dept, salary);
    }

    println!("\n✓ File processed successfully");

    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature to be enabled.");
    eprintln!("Run with: cargo run --example async_file --features async");
}
