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

//! Basic async streaming parser example.
//!
//! Demonstrates async I/O with tokio for processing HEDL documents.
//!
//! Run with: cargo run --example async_basic --features async

#[cfg(feature = "async")]
use hedl_stream::{AsyncStreamingParser, NodeEvent};
#[cfg(feature = "async")]
use std::io::Cursor;

#[cfg(feature = "async")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email, active]
%STRUCT: Order: [id, amount, status]
%NEST: User > Order
---
users: @User
  | alice, Alice Smith, alice@example.com, true
    | order1, 100.00, shipped
    | order2, 50.00, pending
  | bob, Bob Jones, bob@example.com, true
    | order3, 75.00, delivered
  | carol, Carol White, carol@example.com, false
"#;

    println!("=== Async Streaming HEDL Parser Example ===\n");

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await?;

    // Display header information
    if let Some(header) = parser.header() {
        println!("HEDL Version: {}.{}", header.version.0, header.version.1);
        println!("Schemas defined: {}", header.structs.len());
        println!();
    }

    println!("Processing events:\n");

    let mut user_count = 0;
    let mut order_count = 0;

    while let Some(event) = parser.next_event().await? {
        match event {
            NodeEvent::ListStart { key, type_name, .. } => {
                println!("📋 List '{}' of type {} started", key, type_name);
            }
            NodeEvent::Node(node) => {
                if node.type_name == "User" {
                    user_count += 1;
                    let name = node.get_field(1).unwrap();
                    let email = node.get_field(2).unwrap();
                    let active = node.get_field(3).unwrap();
                    println!(
                        "  👤 User {}: {} ({}) [{}]",
                        node.id,
                        name,
                        email,
                        if matches!(active, hedl_stream::Value::Bool(true)) {
                            "active"
                        } else {
                            "inactive"
                        }
                    );
                } else if node.type_name == "Order" {
                    order_count += 1;
                    let amount = node.get_field(1).unwrap();
                    let status = node.get_field(2).unwrap();
                    println!(
                        "    📦 Order {}: {} - {}",
                        node.id,
                        amount,
                        status
                    );
                }
            }
            NodeEvent::ListEnd { type_name, count, .. } => {
                println!("✓ List of {} ended ({} items)\n", type_name, count);
            }
            _ => {}
        }
    }

    println!("\n=== Summary ===");
    println!("Total users: {}", user_count);
    println!("Total orders: {}", order_count);

    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature to be enabled.");
    eprintln!("Run with: cargo run --example async_basic --features async");
}
