// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Example: Processing multiple HEDL files concurrently.
//!
//! Demonstrates parallel processing of multiple HEDL streams
//! using tokio's join! and spawn for concurrent execution.

use hedl_stream::{AsyncStreamingParser, NodeEvent};
use std::time::Instant;
use tokio::task::JoinSet;

const FILE1: &str = r#"
%VERSION 1.0
%STRUCT Product[id, name, price]
---
products:@Product
  | p1, Widget, 9.99
  | p2, Gadget, 19.99
  | p3, Gizmo, 29.99
"#;

const FILE2: &str = r#"
%VERSION 1.0
%STRUCT Order[id, product_id, quantity]
---
orders:@Order
  | o1, p1, 5
  | o2, p2, 3
  | o3, p3, 1
"#;

const FILE3: &str = r#"
%VERSION 1.0
%STRUCT Customer[id, name, email]
---
customers:@Customer
  | c1, Alice, alice@example.com
  | c2, Bob, bob@example.com
"#;

async fn process_hedl(
    name: &str,
    content: &str,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let cursor = std::io::Cursor::new(content);
    let mut parser = AsyncStreamingParser::new(cursor).await?;

    let mut count = 0;
    while let Some(event) = parser.next_event().await? {
        if matches!(event, NodeEvent::Node(_)) {
            count += 1;
        }
        // Simulate some processing time
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    println!("  {} processed {} items", name, count);
    Ok(count)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Async Concurrent Files Example\n");

    // Method 1: Using join! for a fixed number of files
    println!("Method 1: Using join! for concurrent processing");
    let start = Instant::now();

    let (r1, r2, r3) = tokio::join!(
        process_hedl("products.hedl", FILE1),
        process_hedl("orders.hedl", FILE2),
        process_hedl("customers.hedl", FILE3)
    );

    let total: usize = r1.unwrap_or(0) + r2.unwrap_or(0) + r3.unwrap_or(0);
    println!("  Total items: {} in {:?}\n", total, start.elapsed());

    // Method 2: Using JoinSet for dynamic number of files
    println!("Method 2: Using JoinSet for dynamic file list");
    let start = Instant::now();

    let files = vec![
        ("products.hedl", FILE1),
        ("orders.hedl", FILE2),
        ("customers.hedl", FILE3),
    ];

    let mut set = JoinSet::new();
    for (name, content) in files {
        let name = name.to_string();
        let content = content.to_string();
        set.spawn(async move { process_hedl(&name, &content).await });
    }

    let mut total = 0;
    while let Some(result) = set.join_next().await {
        match result {
            Ok(Ok(count)) => total += count,
            Ok(Err(e)) => eprintln!("  Error processing file: {}", e),
            Err(e) => eprintln!("  Task failed: {}", e),
        }
    }

    println!("  Total items: {} in {:?}", total, start.elapsed());
    println!("\nConcurrent files example complete.");
    Ok(())
}
