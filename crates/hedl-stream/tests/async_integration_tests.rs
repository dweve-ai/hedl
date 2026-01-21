// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Async integration tests for hedl-stream
//!
//! These tests verify async functionality when the "async" feature is enabled.

#![cfg(feature = "async")]

use hedl_stream::{AsyncStreamingParser, NodeEvent, StreamingParserConfig};
use std::io::Cursor;
use std::time::Duration;

// ==================== Basic Async Tests ====================

#[tokio::test]
async fn test_async_basic_parsing() {
    let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
  | bob, Bob
";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut node_count = 0;
    while let Some(event) = parser.next_event().await.unwrap() {
        if let NodeEvent::Node(_) = event {
            node_count += 1;
        }
    }

    assert_eq!(node_count, 2);
}

#[tokio::test]
async fn test_async_header_access() {
    let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
";

    let parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let header = parser.header().unwrap();
    assert_eq!(header.version, (1, 0));
    assert!(header.structs.contains_key("User"));
}

#[tokio::test]
async fn test_async_empty_input() {
    let input = "%VERSION: 1.0\n---\n";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut event_count = 0;
    while let Some(_event) = parser.next_event().await.unwrap() {
        event_count += 1;
    }

    // Should have at least EndOfDocument
    assert!(event_count > 0);
}

// ==================== Error Handling Tests ====================

#[tokio::test]
async fn test_async_missing_version() {
    let input = "---\ndata\n";

    let result = AsyncStreamingParser::new(Cursor::new(input)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_shape_mismatch() {
    let input = r"
%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
  | id1, val1
  | id2
";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut found_error = false;
    loop {
        match parser.next_event().await {
            Ok(Some(_)) => continue,
            Ok(None) => break,
            Err(_) => {
                found_error = true;
                break;
            }
        }
    }

    assert!(found_error);
}

// ==================== Configuration Tests ====================

#[tokio::test]
async fn test_async_with_config() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
";

    let config = StreamingParserConfig::default();

    let mut parser = AsyncStreamingParser::with_config(Cursor::new(input), config)
        .await
        .unwrap();

    let mut node_count = 0;
    while let Some(event) = parser.next_event().await.unwrap() {
        if event.is_node() {
            node_count += 1;
        }
    }

    assert_eq!(node_count, 2);
}

#[tokio::test]
async fn test_async_with_timeout_config() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
";

    let config = StreamingParserConfig {
        timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    };

    let mut parser = AsyncStreamingParser::with_config(Cursor::new(input), config)
        .await
        .unwrap();

    let mut event_count = 0;
    while let Some(_event) = parser.next_event().await.unwrap() {
        event_count += 1;
    }

    assert!(event_count > 0);
}

// ==================== Streaming Tests ====================

#[tokio::test]
async fn test_async_list_events() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
  | c
";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut list_start_found = false;
    let mut list_end_found = false;
    let mut node_count = 0;

    while let Some(event) = parser.next_event().await.unwrap() {
        match event {
            NodeEvent::ListStart { .. } => list_start_found = true,
            NodeEvent::ListEnd { count, .. } => {
                list_end_found = true;
                assert_eq!(count, 3);
            }
            NodeEvent::Node(_) => node_count += 1,
            _ => {}
        }
    }

    assert!(list_start_found);
    assert!(list_end_found);
    assert_eq!(node_count, 3);
}

#[tokio::test]
async fn test_async_object_events() {
    let input = r"
%VERSION: 1.0
---
config:
  timeout: 30
";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut object_start_found = false;
    let mut object_end_found = false;

    while let Some(event) = parser.next_event().await.unwrap() {
        match event {
            NodeEvent::ObjectStart { .. } => object_start_found = true,
            NodeEvent::ObjectEnd { .. } => object_end_found = true,
            _ => {}
        }
    }

    assert!(object_start_found);
    assert!(object_end_found);
}

// ==================== Nesting Tests ====================

#[tokio::test]
async fn test_async_nested_data() {
    let input = r"
%VERSION: 1.0
%STRUCT: Parent: [id]
%STRUCT: Child: [id]
%NEST: Parent > Child
---
data: @Parent
  | parent1
    | child1
    | child2
";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut nodes = Vec::new();
    while let Some(event) = parser.next_event().await.unwrap() {
        if let NodeEvent::Node(node) = event {
            nodes.push(node);
        }
    }

    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].id, "parent1");
    assert_eq!(nodes[1].id, "child1");
    assert_eq!(nodes[2].id, "child2");
}

// ==================== Concurrent Processing Tests ====================

#[tokio::test]
async fn test_async_concurrent_parsers() {
    let input1 = r"
%VERSION: 1.0
%STRUCT: A: [id]
---
a: @A
  | a1
";

    let input2 = r"
%VERSION: 1.0
%STRUCT: B: [id]
---
b: @B
  | b1
  | b2
";

    let input3 = r"
%VERSION: 1.0
%STRUCT: C: [id]
---
c: @C
  | c1
  | c2
  | c3
";

    async fn count_nodes(input: &str) -> usize {
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();
        let mut count = 0;
        while let Some(event) = parser.next_event().await.unwrap() {
            if event.is_node() {
                count += 1;
            }
        }
        count
    }

    let (count1, count2, count3) = tokio::join!(
        count_nodes(input1),
        count_nodes(input2),
        count_nodes(input3),
    );

    assert_eq!(count1, 1);
    assert_eq!(count2, 2);
    assert_eq!(count3, 3);
}

// ==================== Large Data Tests ====================

#[tokio::test]
async fn test_async_many_rows() {
    let mut input = String::from("%VERSION: 1.0\n%STRUCT: Item: [id]\n---\nitems: @Item\n");
    for i in 0..100 {
        input.push_str(&format!("  | item{i}\n"));
    }

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut node_count = 0;
    while let Some(event) = parser.next_event().await.unwrap() {
        if event.is_node() {
            node_count += 1;
        }
    }

    assert_eq!(node_count, 100);
}

// ==================== Scalar Value Tests ====================

#[tokio::test]
async fn test_async_scalar_values() {
    let input = r"
%VERSION: 1.0
---
name: Alice
age: 30
active: true
score: 95.5
optional: null
";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut scalar_count = 0;
    while let Some(event) = parser.next_event().await.unwrap() {
        if matches!(event, NodeEvent::Scalar { .. }) {
            scalar_count += 1;
        }
    }

    assert_eq!(scalar_count, 5);
}

// ==================== Unicode Tests ====================

#[tokio::test]
async fn test_async_unicode_data() {
    let input = r"
%VERSION: 1.0
%STRUCT: Text: [id, content]
---
texts: @Text
  | text1, Hello 世界 🌍
  | text2, Привет мир
";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut node_count = 0;
    while let Some(event) = parser.next_event().await.unwrap() {
        if event.is_node() {
            node_count += 1;
        }
    }

    assert_eq!(node_count, 2);
}

// ==================== Edge Case Tests ====================

#[tokio::test]
async fn test_async_empty_list() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut found_list_end = false;
    while let Some(event) = parser.next_event().await.unwrap() {
        if let NodeEvent::ListEnd { count, .. } = event {
            assert_eq!(count, 0);
            found_list_end = true;
        }
    }

    assert!(found_list_end);
}

#[tokio::test]
async fn test_async_comments() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
# This is a comment
items: @Item
  | a
  # Another comment
  | b
";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut node_count = 0;
    while let Some(event) = parser.next_event().await.unwrap() {
        if event.is_node() {
            node_count += 1;
        }
    }

    assert_eq!(node_count, 2);
}

// ==================== Multiple Structures Tests ====================

#[tokio::test]
async fn test_async_multiple_lists() {
    let input = r"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
---
a: @A
  | a1
  | a2
b: @B
  | b1
";

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut a_count = 0;
    let mut b_count = 0;

    while let Some(event) = parser.next_event().await.unwrap() {
        if let NodeEvent::Node(node) = event {
            if node.type_name == "A" {
                a_count += 1;
            } else if node.type_name == "B" {
                b_count += 1;
            }
        }
    }

    assert_eq!(a_count, 2);
    assert_eq!(b_count, 1);
}

// ==================== Cancel Safety Tests ====================

#[tokio::test]
async fn test_async_early_termination() {
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

    let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    let mut node_count = 0;
    while let Some(event) = parser.next_event().await.unwrap() {
        if event.is_node() {
            node_count += 1;
            if node_count == 3 {
                break; // Early termination
            }
        }
    }

    assert_eq!(node_count, 3);
}

// ==================== State Consistency Tests ====================

#[tokio::test]
async fn test_async_header_available_before_iteration() {
    let input = r"
%VERSION: 1.0
%STRUCT: User: [id]
---
users: @User
  | alice
";

    let parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

    // Header should be available before calling next_event
    let header = parser.header();
    assert!(header.is_some());
    assert_eq!(header.unwrap().version, (1, 0));
}
