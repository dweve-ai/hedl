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

//! Example demonstrating the streaming API for large documents.
//!
//! This example shows how to use `to_cypher_stream()` to generate Cypher
//! queries from large HEDL documents with constant memory usage.
//!
//! Run with: cargo run --example streaming_example

use hedl_core::{Document, Item, MatrixList, Node, Value};
use hedl_neo4j::{to_cypher, to_cypher_stream, ToCypherConfig};
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};

fn create_large_document(num_nodes: usize) -> Document {
    println!("Creating document with {} nodes...", num_nodes);

    let mut rows = Vec::new();
    for i in 0..num_nodes {
        rows.push(Node {
            type_name: "User".to_string(),
            id: format!("user{}", i),
            fields: vec![
                Value::String(format!("user{}", i)),
                Value::String(format!("User {}", i)),
                Value::String(format!("user{}@example.com", i)),
            ],
            children: BTreeMap::new(),
            child_count: None,
        });
    }

    let mut root = BTreeMap::new();
    root.insert(
        "users".to_string(),
        Item::List(MatrixList {
            type_name: "User".to_string(),
            schema: vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string(),
            ],
            rows,
            count_hint: None,
        }),
    );

    Document {
        version: (1, 0),
        aliases: BTreeMap::new(),
        structs: BTreeMap::new(),
        nests: BTreeMap::new(),
        root,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HEDL Neo4j Streaming API Example ===\n");

    // Configuration
    let config = ToCypherConfig::new()
        .with_batch_size(1000)
        .without_comments(); // Disable comments for cleaner output

    // Small document - both APIs work fine
    println!("1. Small document (100 nodes):");
    let small_doc = create_large_document(100);

    // Regular API
    let start = std::time::Instant::now();
    let regular_output = to_cypher(&small_doc, &config)?;
    let regular_time = start.elapsed();
    println!("   Regular API: {} µs, {} bytes", regular_time.as_micros(), regular_output.len());

    // Streaming API
    let start = std::time::Instant::now();
    let mut streaming_output = Vec::new();
    to_cypher_stream(&small_doc, &config, &mut streaming_output)?;
    let streaming_time = start.elapsed();
    println!("   Streaming API: {} µs, {} bytes", streaming_time.as_micros(), streaming_output.len());

    // Verify identical output
    assert_eq!(regular_output, String::from_utf8(streaming_output)?);
    println!("   ✓ Output identical\n");

    // Large document - streaming API shines
    println!("2. Large document (10,000 nodes):");
    let large_doc = create_large_document(10_000);

    // Regular API
    let start = std::time::Instant::now();
    let regular_output = to_cypher(&large_doc, &config)?;
    let regular_time = start.elapsed();
    println!(
        "   Regular API: {} ms, {:.2} MB",
        regular_time.as_millis(),
        regular_output.len() as f64 / 1_000_000.0
    );

    // Streaming API to memory
    let start = std::time::Instant::now();
    let mut streaming_output = Vec::new();
    to_cypher_stream(&large_doc, &config, &mut streaming_output)?;
    let streaming_time = start.elapsed();
    println!(
        "   Streaming API (to memory): {} ms, {:.2} MB",
        streaming_time.as_millis(),
        streaming_output.len() as f64 / 1_000_000.0
    );

    // Verify identical output
    assert_eq!(regular_output, String::from_utf8(streaming_output.clone())?);
    println!("   ✓ Output identical\n");

    // Streaming to file
    println!("3. Streaming to file:");
    let file = std::fs::File::create("output.cypher")?;
    let mut writer = BufWriter::new(file);

    let start = std::time::Instant::now();
    to_cypher_stream(&large_doc, &config, &mut writer)?;
    writer.flush()?;
    let streaming_time = start.elapsed();

    let file_size = std::fs::metadata("output.cypher")?.len();
    println!(
        "   Streamed to output.cypher: {} ms, {:.2} MB",
        streaming_time.as_millis(),
        file_size as f64 / 1_000_000.0
    );
    println!("   ✓ File written successfully\n");

    // Streaming to stdout (first 5 statements only)
    println!("4. Streaming to stdout (first 5 statements):");
    let small_sample = create_large_document(3); // Just 3 nodes for demo
    let config_with_comments = ToCypherConfig::new()
        .with_batch_size(1000)
        .with_create(); // Use CREATE for variety

    let stdout = std::io::stdout();
    let mut writer = BufWriter::new(stdout.lock());
    to_cypher_stream(&small_sample, &config_with_comments, &mut writer)?;
    writer.flush()?;
    println!("\n");

    // Demonstrate custom sink (counting writer)
    println!("5. Custom writer (byte counter):");
    struct CountingWriter {
        count: usize,
    }

    impl Write for CountingWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.count += buf.len();
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let mut counter = CountingWriter { count: 0 };
    to_cypher_stream(&large_doc, &config, &mut counter)?;
    println!("   Total bytes written: {}", counter.count);
    println!("   Average bytes per node: {}", counter.count / 10_000);

    println!("\n=== Summary ===");
    println!("✓ Streaming API produces identical output to regular API");
    println!("✓ Streaming API has constant memory overhead");
    println!("✓ Streaming API works with any Write implementation");
    println!("✓ Perfect for large documents (>10MB) and memory-constrained environments");

    // Clean up
    std::fs::remove_file("output.cypher")?;

    Ok(())
}
