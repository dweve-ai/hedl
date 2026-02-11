// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Example: Using the Stream trait with HEDL async parser.
//!
//! Demonstrates using futures::Stream combinators for
//! functional-style processing of HEDL streams.

use futures::StreamExt;
use hedl_stream::AsyncStreamingParser;

const SAMPLE_HEDL: &str = r#"
%VERSION 1.0
%STRUCT Metric[id, name, value, unit]
---
metrics:@Metric
  | cpu_usage, "CPU Usage", 45.5, percent
  | mem_usage, "Memory Usage", 72.3, percent
  | disk_io, "Disk I/O", 1024, MB/s
  | net_in, "Network In", 500, Mbps
  | net_out, "Network Out", 250, Mbps
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Async Stream Trait Example\n");

    // Example 1: Count items
    println!("1. Counting items:");
    let cursor = std::io::Cursor::new(SAMPLE_HEDL);
    let parser = AsyncStreamingParser::new(cursor).await?;
    let count = parser.count().await;
    println!("   Total items: {}\n", count);

    // Example 2: Enumerate items
    println!("2. Enumerating items:");
    let cursor = std::io::Cursor::new(SAMPLE_HEDL);
    let parser = AsyncStreamingParser::new(cursor).await?;
    let mut stream = parser.enumerate();

    while let Some((index, result)) = stream.next().await {
        match result {
            Ok(event) => println!("   Event {}: {:?}", index, event),
            Err(e) => eprintln!("   Error at {}: {}", index, e),
        }
    }

    // Example 3: Take first N items
    println!("\n3. Taking first 3 items:");
    let cursor = std::io::Cursor::new(SAMPLE_HEDL);
    let parser = AsyncStreamingParser::new(cursor).await?;
    let mut stream = parser.take(3);

    while let Some(result) = stream.next().await {
        match result {
            Ok(event) => println!("   {:?}", event),
            Err(e) => eprintln!("   Error: {}", e),
        }
    }

    // Example 4: Collect into Vec (with error handling)
    println!("\n4. Collecting all items:");
    let cursor = std::io::Cursor::new(SAMPLE_HEDL);
    let parser = AsyncStreamingParser::new(cursor).await?;
    let events: Vec<_> = parser.filter_map(|r| async { r.ok() }).collect().await;
    println!("   Collected {} events", events.len());

    println!("\nStream trait example complete.");
    Ok(())
}
