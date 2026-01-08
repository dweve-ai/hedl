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

//! Example: Processing large XML files with streaming API
//!
//! This example demonstrates how to use the streaming XML parser to handle
//! large files without loading them entirely into memory.
//!
//! The streaming parser yields items incrementally as they're parsed,
//! making it suitable for multi-gigabyte XML files.

use hedl_xml::streaming::{from_xml_stream, StreamConfig};
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Processing a moderate-sized XML document
    println!("Example 1: Streaming a moderate XML document");
    println!("===========================================\n");

    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <user id="1">
            <name>Alice</name>
            <email>alice@example.com</email>
        </user>
        <user id="2">
            <name>Bob</name>
            <email>bob@example.com</email>
        </user>
        <user id="3">
            <name>Charlie</name>
            <email>charlie@example.com</email>
        </user>
    </hedl>"#;

    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config)?;

    let mut item_count = 0;
    for result in parser {
        match result {
            Ok(item) => {
                println!("Item: key={}, value_type={:?}", item.key, item.value);
                item_count += 1;
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
    println!("\nProcessed {} top-level items\n", item_count);

    // Example 2: Custom buffer configuration
    println!("Example 2: Custom buffer configuration");
    println!("======================================\n");

    let config = StreamConfig {
        buffer_size: 131072, // 128KB instead of 64KB
        max_recursion_depth: 50,
        max_batch_size: 500,
        ..Default::default()
    };

    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config)?;

    for result in parser {
        match result {
            Ok(item) => {
                println!("Streaming with custom buffer: {}", item.key);
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }
    println!();

    // Example 3: Simulating a very large dataset
    println!("Example 3: Processing a large dataset (simulated)");
    println!("=================================================\n");

    let mut large_xml = String::from(r#"<?xml version="1.0"?><hedl>"#);

    // Generate 1000 items
    for i in 0..1000 {
        large_xml.push_str(&format!(
            r#"<record id="{}" index="{}">
                <value>{}</value>
                <timestamp>{}</timestamp>
            </record>"#,
            i,
            i,
            i * 10,
            (i * 1000) as u64
        ));
    }
    large_xml.push_str("</hedl>");

    println!("Generated XML size: {} bytes", large_xml.len());
    println!("This would represent a much larger file in production.\n");

    let config = StreamConfig::default();
    let cursor = Cursor::new(large_xml.as_bytes());
    let parser = from_xml_stream(cursor, &config)?;

    let mut total_records = 0;
    for result in parser {
        match result {
            Ok(item) => {
                // In a real scenario, you would process each item here
                // and potentially write results to a database or file
                // without accumulating them in memory
                if total_records < 5 {
                    println!("Record {}: {}", total_records, item.key);
                } else if total_records == 5 {
                    println!("... processing {} more records ...", 1000 - 5);
                }
                total_records += 1;
            }
            Err(e) => {
                eprintln!("Parse error on record {}: {}", total_records, e);
                break;
            }
        }
    }
    println!("\nTotal records processed: {}\n", total_records);

    // Example 4: Processing with streaming statistics
    println!("Example 4: Streaming with statistics collection");
    println!("===============================================\n");

    let config = StreamConfig::default();
    let cursor = Cursor::new(xml.as_bytes());
    let parser = from_xml_stream(cursor, &config)?;

    let mut stats = StreamStatistics::new();

    for result in parser {
        match result {
            Ok(item) => {
                stats.process_item(&item);
            }
            Err(e) => eprintln!("Parse error: {}", e),
        }
    }

    println!("{}", stats);

    Ok(())
}

/// Simple statistics collector for streaming items
struct StreamStatistics {
    total_items: usize,
    item_keys: Vec<String>,
}

impl StreamStatistics {
    fn new() -> Self {
        StreamStatistics {
            total_items: 0,
            item_keys: Vec::new(),
        }
    }

    fn process_item(&mut self, item: &hedl_xml::StreamItem) {
        self.total_items += 1;
        self.item_keys.push(item.key.clone());
    }
}

impl std::fmt::Display for StreamStatistics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Stream Statistics:")?;
        writeln!(f, "  Total items: {}", self.total_items)?;
        writeln!(f, "  Unique keys: {}", unique_count(&self.item_keys))?;
        writeln!(f, "  Keys seen: {}", self.item_keys.join(", "))?;
        Ok(())
    }
}

fn unique_count(items: &[String]) -> usize {
    items.iter().collect::<std::collections::HashSet<_>>().len()
}
