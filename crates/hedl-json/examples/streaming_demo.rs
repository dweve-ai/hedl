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

//! Streaming JSON examples
//!
//! Demonstrates streaming JSON parsing for memory-efficient processing of large files.

use hedl_json::streaming::{JsonArrayStreamer, JsonLinesStreamer, JsonLinesWriter, StreamConfig};
use std::io::Cursor;

fn main() {
    println!("=== HEDL JSON Streaming Demo ===\n");

    // Example 1: JSONL Streaming
    println!("1. JSONL Streaming (Memory-Efficient)");
    println!("--------------------------------------");

    let jsonl = r#"{"id": "1", "name": "Alice", "age": 30}
{"id": "2", "name": "Bob", "age": 25}
{"id": "3", "name": "Charlie", "age": 35}"#;

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let mut count = 0;
    for result in streamer {
        let doc = result.unwrap();
        count += 1;
        println!("  Document {}: {} fields", count, doc.root.len());
    }
    println!("  Total: {} documents processed\n", count);

    // Example 2: JSON Array Streaming
    println!("2. JSON Array Streaming");
    println!("-----------------------");

    let json = r#"[
        {"id": "1", "type": "user"},
        {"id": "2", "type": "admin"},
        {"id": "3", "type": "guest"}
    ]"#;

    let reader = Cursor::new(json.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonArrayStreamer::new(reader, config).unwrap();

    for (i, result) in streamer.enumerate() {
        let doc = result.unwrap();
        println!("  Array element {}: {} fields", i + 1, doc.root.len());
    }
    println!();

    // Example 3: Writing JSONL
    println!("3. Writing JSONL");
    println!("----------------");

    let mut buffer = Vec::new();
    let mut writer = JsonLinesWriter::new(&mut buffer);

    for i in 1..=5 {
        let mut doc = hedl_core::Document::new((1, 0));
        doc.root.insert(
            "id".to_string(),
            hedl_core::Item::Scalar(hedl_core::Value::String(i.to_string())),
        );
        doc.root.insert(
            "value".to_string(),
            hedl_core::Item::Scalar(hedl_core::Value::Int(i * 10)),
        );
        writer.write_document(&doc).unwrap();
    }
    writer.flush().unwrap();

    let output = String::from_utf8(buffer).unwrap();
    println!("  Generated JSONL:");
    for line in output.lines() {
        println!("    {}", line);
    }
    println!();

    // Example 4: Custom Configuration
    println!("4. Custom Stream Configuration");
    println!("------------------------------");

    let config = StreamConfig::builder()
        .buffer_size(128 * 1024)  // 128 KB buffer
        .max_object_bytes(5 * 1024 * 1024)  // 5 MB max object
        .build();

    println!("  Buffer size: {} KB", config.buffer_size / 1024);
    println!("  Max object size: {} MB", config.max_object_bytes.unwrap() / 1024 / 1024);
    println!();

    // Example 5: Large Dataset Processing
    println!("5. Large Dataset Processing");
    println!("---------------------------");

    // Generate large JSONL dataset
    let mut large_jsonl = String::new();
    for i in 0..1000 {
        large_jsonl.push_str(&format!(r#"{{"id": "{}", "value": {}}}"#, i, i * 2));
        large_jsonl.push('\n');
    }

    let reader = Cursor::new(large_jsonl.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let total = streamer.map(|r| r.unwrap()).count();
    println!("  Processed {} documents from large dataset", total);
    println!("  Memory used: Only one document at a time!");
    println!();

    println!("=== Demo Complete ===");
}
