// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Example: Async batch processing of HEDL streams.
//!
//! Demonstrates processing HEDL items in batches asynchronously,
//! which is useful for rate-limited APIs or batch database operations.

use hedl_stream::{AsyncStreamingParser, NodeEvent};
use std::time::Duration;

const SAMPLE_HEDL: &str = r"
%VERSION 1.0
%STRUCT User[id, name, email]
---
users:@User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
  | carol, Carol White, carol@example.com
  | dave, Dave Brown, dave@example.com
  | eve, Eve Black, eve@example.com
";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Async Batch Processing Example\n");
    println!("Processing HEDL data in batches of 2...\n");

    let cursor = std::io::Cursor::new(SAMPLE_HEDL);
    let mut parser = AsyncStreamingParser::new(cursor).await?;

    let mut batch = Vec::new();
    let batch_size = 2;
    let mut batch_num = 0;

    while let Some(event) = parser.next_event().await? {
        // Only batch Node events
        if let NodeEvent::Node(node) = event {
            batch.push(node);

            if batch.len() >= batch_size {
                batch_num += 1;
                println!("Processing batch {batch_num}:");
                for node in &batch {
                    println!("  - {}:{}", node.type_name, node.id);
                }

                // Simulate async processing (e.g., API call, database insert)
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("  Batch {batch_num} processed.\n");

                batch.clear();
            }
        }
    }

    // Process remaining items
    if !batch.is_empty() {
        batch_num += 1;
        println!("Processing final batch {batch_num}:");
        for node in &batch {
            println!("  - {}:{}", node.type_name, node.id);
        }
        println!("  Batch {batch_num} processed.\n");
    }

    println!("Total batches processed: {batch_num}");
    Ok(())
}
