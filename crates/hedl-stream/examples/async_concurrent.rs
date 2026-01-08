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

//! Concurrent async parsing example.
//!
//! Demonstrates processing multiple HEDL streams concurrently using tokio.
//!
//! Run with: cargo run --example async_concurrent --features async

#[cfg(feature = "async")]
use hedl_stream::{AsyncStreamingParser, NodeEvent};
#[cfg(feature = "async")]
use std::io::Cursor;

#[cfg(feature = "async")]
async fn process_stream(name: &str, data: &str) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    println!("[{}] Starting processing...", name);

    let mut parser = AsyncStreamingParser::new(Cursor::new(data)).await?;

    let mut count = 0;
    while let Some(event) = parser.next_event().await? {
        if let NodeEvent::Node(_) = event {
            count += 1;
        }
    }

    println!("[{}] Completed: {} nodes processed", name, count);
    Ok(count)
}

#[cfg(feature = "async")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("=== Concurrent Async Parsing Example ===\n");

    let data1 = r#"
%VERSION: 1.0
%STRUCT: Product: [id, name, price]
---
products: @Product
  | p1, Widget, 9.99
  | p2, Gadget, 19.99
  | p3, Doohickey, 29.99
"#;

    let data2 = r#"
%VERSION: 1.0
%STRUCT: Customer: [id, name, tier]
---
customers: @Customer
  | c1, Alice, Gold
  | c2, Bob, Silver
  | c3, Carol, Bronze
  | c4, David, Gold
"#;

    let data3 = r#"
%VERSION: 1.0
%STRUCT: Transaction: [id, amount, type]
---
transactions: @Transaction
  | t1, 100.00, debit
  | t2, 50.00, credit
  | t3, 75.00, debit
  | t4, 125.00, credit
  | t5, 200.00, debit
"#;

    // Process all streams concurrently
    let (result1, result2, result3) = tokio::join!(
        process_stream("Products", data1),
        process_stream("Customers", data2),
        process_stream("Transactions", data3),
    );

    println!("\n=== Results ===");
    println!("Products: {} nodes", result1?);
    println!("Customers: {} nodes", result2?);
    println!("Transactions: {} nodes", result3?);

    // Demonstrate parallel processing with multiple tasks
    println!("\n=== Spawning Independent Tasks ===\n");

    let handle1 = tokio::spawn(process_stream("Stream 1", data1));
    let handle2 = tokio::spawn(process_stream("Stream 2", data2));
    let handle3 = tokio::spawn(process_stream("Stream 3", data3));

    // Wait for all tasks to complete
    let _ = tokio::try_join!(handle1, handle2, handle3)?;

    println!("\nAll tasks completed successfully!");

    Ok(())
}

#[cfg(not(feature = "async"))]
fn main() {
    eprintln!("This example requires the 'async' feature to be enabled.");
    eprintln!("Run with: cargo run --example async_concurrent --features async");
}
