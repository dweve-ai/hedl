// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for buffer pool with `StreamingParser`

use hedl_stream::{BufferSizeHint, MemoryLimits, StreamingParser, StreamingParserConfig};
use std::io::Cursor;

// ==================== Buffer Pool Configuration Tests ====================

#[test]
fn test_parser_with_pooling_enabled() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
  | c
";

    let config = StreamingParserConfig::default().with_buffer_pooling(true);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Should work the same with pooling enabled
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 3);
}

#[test]
fn test_parser_with_pooling_disabled() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
";

    let config = StreamingParserConfig::default().with_buffer_pooling(false);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 2);
}

#[test]
fn test_parser_with_custom_pool_size() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
";

    let config = StreamingParserConfig::default()
        .with_buffer_pooling(true)
        .with_pool_size(50);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    assert!(!events.is_empty());
}

#[test]
fn test_parser_with_large_pool() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | item1
  | item2
  | item3
  | item4
  | item5
";

    let config = StreamingParserConfig::default()
        .with_buffer_pooling(true)
        .with_pool_size(100);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 5);
}

#[test]
fn test_parser_with_zero_pool_size() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
";

    let config = StreamingParserConfig::default()
        .with_buffer_pooling(true)
        .with_pool_size(0);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Should still work, just won't pool anything
    assert!(!events.is_empty());
}

// ==================== Memory Limits Integration Tests ====================

#[test]
fn test_parser_with_embedded_limits() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
";

    let config = StreamingParserConfig::default().with_memory_limits(MemoryLimits::embedded());

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    assert!(!events.is_empty());
}

#[test]
fn test_parser_with_high_throughput_limits() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id, value]
---
items: @Item
  | a, value_a
  | b, value_b
";

    let config =
        StreamingParserConfig::default().with_memory_limits(MemoryLimits::high_throughput());

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 2);
}

#[test]
fn test_parser_with_untrusted_limits() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
";

    let config = StreamingParserConfig::default().with_memory_limits(MemoryLimits::untrusted());

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    assert!(!events.is_empty());
}

#[test]
fn test_untrusted_limits_enforce_line_length() {
    let limits = MemoryLimits::untrusted();
    assert_eq!(limits.max_line_length, 100_000);

    let long_line = "x".repeat(150_000);
    let input = format!("%VERSION: 1.0\n---\nkey: {long_line}\n");

    let config = StreamingParserConfig::default().with_memory_limits(limits);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect();

    // Should get LineTooLong error
    assert!(events.iter().any(std::result::Result::is_err));
}

// ==================== Buffer Size Hint Tests ====================

#[test]
fn test_parser_with_small_buffer() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
";

    let config = StreamingParserConfig::default().with_buffer_hint(BufferSizeHint::Small);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 2);
}

#[test]
fn test_parser_with_medium_buffer() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
  | c
";

    let config = StreamingParserConfig::default().with_buffer_hint(BufferSizeHint::Medium);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 3);
}

#[test]
fn test_parser_with_large_buffer() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
  | c
  | d
";

    let config = StreamingParserConfig::default().with_buffer_hint(BufferSizeHint::Large);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 4);
}

#[test]
fn test_parser_with_huge_buffer() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
  | c
  | d
  | e
";

    let config = StreamingParserConfig::default().with_buffer_hint(BufferSizeHint::Huge);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 5);
}

// ==================== Configuration Combinations ====================

#[test]
fn test_pooling_with_small_buffer() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
";

    let config = StreamingParserConfig::default()
        .with_buffer_hint(BufferSizeHint::Small)
        .with_buffer_pooling(true);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    assert!(!events.is_empty());
}

#[test]
fn test_pooling_with_huge_buffer() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
";

    let config = StreamingParserConfig::default()
        .with_buffer_hint(BufferSizeHint::Huge)
        .with_buffer_pooling(true)
        .with_pool_size(25);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 2);
}

#[test]
fn test_embedded_with_pooling_disabled() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
";

    let config = StreamingParserConfig::default()
        .with_memory_limits(MemoryLimits::embedded())
        .with_buffer_pooling(false);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    assert!(!events.is_empty());
}

#[test]
fn test_high_throughput_with_large_buffer() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
  | c
";

    let config = StreamingParserConfig::default()
        .with_memory_limits(MemoryLimits::high_throughput())
        .with_buffer_hint(BufferSizeHint::Large);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 3);
}

// ==================== Stress Tests with Different Configs ====================

#[test]
fn test_many_rows_with_pooling() {
    let mut input = String::from("%VERSION: 1.0\n%STRUCT: Item: [id]\n---\nitems: @Item\n");
    for i in 0..500 {
        input.push_str(&format!("  | item{i}\n"));
    }

    let config = StreamingParserConfig::default()
        .with_buffer_pooling(true)
        .with_pool_size(20);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 500);
}

#[test]
fn test_many_rows_with_small_buffer() {
    let mut input = String::from("%VERSION: 1.0\n%STRUCT: Item: [id]\n---\nitems: @Item\n");
    for i in 0..200 {
        input.push_str(&format!("  | item{i}\n"));
    }

    let config = StreamingParserConfig::default().with_buffer_hint(BufferSizeHint::Small);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 200);
}

#[test]
fn test_wide_rows_with_pooling() {
    let mut header = String::from("%VERSION: 1.0\n%STRUCT: Wide: [");
    for i in 0..50 {
        if i > 0 {
            header.push_str(", ");
        }
        header.push_str(&format!("f{i}"));
    }
    header.push_str("]\n---\ndata: @Wide\n  | ");

    for i in 0..50 {
        if i > 0 {
            header.push_str(", ");
        }
        header.push_str(&format!("val{i}"));
    }
    header.push('\n');

    let config = StreamingParserConfig::default()
        .with_buffer_pooling(true)
        .with_pool_size(30);

    let parser = StreamingParser::with_config(Cursor::new(header), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].fields.len(), 50);
}

// ==================== Memory Efficiency Tests ====================

#[test]
fn test_embedded_rejects_large_lines() {
    let limits = MemoryLimits::embedded();
    assert_eq!(limits.max_line_length, 10_000);

    let long_line = "x".repeat(15_000);
    let input = format!("%VERSION: 1.0\n---\nkey: {long_line}\n");

    let config = StreamingParserConfig::default().with_memory_limits(limits);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect();

    // Should error on line too long
    assert!(events.iter().any(std::result::Result::is_err));
}

#[test]
fn test_high_throughput_allows_large_lines() {
    let limits = MemoryLimits::high_throughput();
    assert_eq!(limits.max_line_length, 10_000_000);

    let long_line = "x".repeat(100_000);
    let input = format!("%VERSION: 1.0\n---\nkey: {long_line}\n");

    let config = StreamingParserConfig::default().with_memory_limits(limits);

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Should succeed
    assert!(!events.is_empty());
}

// ==================== Config Clone and Debug Tests ====================

#[test]
fn test_config_clone() {
    let config1 = StreamingParserConfig::default()
        .with_buffer_pooling(true)
        .with_pool_size(15);

    let config2 = config1.clone();

    assert_eq!(config2.enable_pooling, config1.enable_pooling);
    assert_eq!(
        config2.memory_limits.max_pool_size,
        config1.memory_limits.max_pool_size
    );
}

#[test]
fn test_config_debug() {
    let config = StreamingParserConfig::default();
    let debug = format!("{config:?}");
    assert!(debug.contains("StreamingParserConfig"));
}
