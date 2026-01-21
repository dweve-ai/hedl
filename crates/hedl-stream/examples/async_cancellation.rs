// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Example: Async cancellation of HEDL stream processing.
//!
//! Demonstrates how to cancel long-running stream operations
//! using tokio's select! macro and cancellation tokens.

use hedl_stream::{AsyncStreamingParser, NodeEvent};
use std::time::Duration;
use tokio::select;
use tokio::sync::oneshot;
use tokio::time::timeout;

const LARGE_HEDL: &str = r#"
%VERSION 1.0
%STRUCT DataPoint[id, value, timestamp]
---
data: @DataPoint
  | p1, 100, 2024-01-01T00:00:00Z
  | p2, 200, 2024-01-01T01:00:00Z
  | p3, 300, 2024-01-01T02:00:00Z
  | p4, 400, 2024-01-01T03:00:00Z
  | p5, 500, 2024-01-01T04:00:00Z
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Async Cancellation Example\n");

    // Example 1: Timeout-based cancellation
    println!("1. Timeout-based cancellation:");
    let cursor = std::io::Cursor::new(LARGE_HEDL);
    let mut parser = AsyncStreamingParser::new(cursor).await?;

    let result = timeout(Duration::from_millis(500), async {
        let mut count = 0;
        while let Some(event) = parser.next_event().await? {
            if matches!(event, NodeEvent::Node(_)) {
                count += 1;
            }
            // Simulate slow processing
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        Ok::<_, Box<dyn std::error::Error>>(count)
    })
    .await;

    match result {
        Ok(Ok(count)) => println!("   Completed: processed {} items", count),
        Ok(Err(e)) => println!("   Error: {}", e),
        Err(_) => println!("   Timed out (expected for this demo)"),
    }

    // Example 2: Manual cancellation with oneshot channel
    println!("\n2. Manual cancellation:");
    let (cancel_tx, cancel_rx) = oneshot::channel::<()>();

    let cursor = std::io::Cursor::new(LARGE_HEDL);
    let mut parser = AsyncStreamingParser::new(cursor).await?;

    // Spawn cancellation trigger
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(150)).await;
        let _ = cancel_tx.send(());
        println!("   Cancellation signal sent");
    });

    let mut count = 0;
    let mut cancel_rx = cancel_rx;

    loop {
        select! {
            _ = &mut cancel_rx => {
                println!("   Received cancellation, stopping...");
                break;
            }
            result = parser.next_event() => {
                match result {
                    Ok(Some(event)) => {
                        if matches!(event, NodeEvent::Node(_)) {
                            count += 1;
                        }
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                    Ok(None) => {
                        println!("   Stream completed normally");
                        break;
                    }
                    Err(e) => {
                        println!("   Error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    println!("   Processed {} items before cancellation", count);
    println!("\nCancellation example complete.");
    Ok(())
}
