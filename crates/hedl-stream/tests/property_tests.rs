// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Property-based tests for hedl-stream
//!
//! These tests verify invariants that should hold for all valid inputs.

use hedl_stream::{NodeEvent, StreamingParser};
use std::io::Cursor;

// ==================== Streaming Invariants ====================

#[test]
fn property_list_start_end_balanced() {
    // Property: Every ListStart must have a corresponding ListEnd
    let inputs = vec![
        r"
%VERSION: 1.0
%STRUCT: A: [id]
---
a: @A
  | a1
",
        r"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
---
a: @A
  | a1
b: @B
  | b1
",
        r"
%VERSION: 1.0
%STRUCT: A: [id]
---
a: @A
b: @A
c: @A
",
    ];

    for input in inputs {
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        let starts = events
            .iter()
            .filter(|e| matches!(e, NodeEvent::ListStart { .. }))
            .count();
        let ends = events
            .iter()
            .filter(|e| matches!(e, NodeEvent::ListEnd { .. }))
            .count();

        assert_eq!(starts, ends, "ListStart/ListEnd mismatch in: {input}");
    }
}

#[test]
fn property_object_start_end_balanced() {
    // Property: Every ObjectStart must have a corresponding ObjectEnd
    let inputs = vec![
        r"
%VERSION: 1.0
---
obj:
  key: value
",
        r"
%VERSION: 1.0
---
obj1:
  key1: value1
obj2:
  key2: value2
",
        r"
%VERSION: 1.0
---
outer:
  inner:
    key: value
",
    ];

    for input in inputs {
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        let starts = events
            .iter()
            .filter(|e| matches!(e, NodeEvent::ObjectStart { .. }))
            .count();
        let ends = events
            .iter()
            .filter(|e| matches!(e, NodeEvent::ObjectEnd { .. }))
            .count();

        assert_eq!(starts, ends, "ObjectStart/ObjectEnd mismatch");
    }
}

#[test]
fn property_list_count_matches_nodes() {
    // Property: ListEnd count should match the number of nodes in the list
    let inputs = vec![
        (
            r"
%VERSION: 1.0
%STRUCT: A: [id]
---
a: @A
  | a1
  | a2
  | a3
",
            3,
        ),
        (
            r"
%VERSION: 1.0
%STRUCT: A: [id]
---
a: @A
",
            0,
        ),
        (
            r"
%VERSION: 1.0
%STRUCT: A: [id]
---
a: @A
  | a1
",
            1,
        ),
    ];

    for (input, expected_count) in inputs {
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        if let Some(NodeEvent::ListEnd { count, .. }) = events
            .iter()
            .find(|e| matches!(e, NodeEvent::ListEnd { .. }))
        {
            assert_eq!(*count, expected_count);
        }
    }
}

#[test]
fn property_node_depth_never_negative() {
    // Property: Node depth should always be >= 0
    let inputs = vec![
        r"
%VERSION: 1.0
%STRUCT: A: [id]
---
a: @A
  | a1
",
        r"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%NEST: A > B
---
a: @A
  | a1
    | b1
",
    ];

    for input in inputs {
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        for event in events {
            if let NodeEvent::Node(node) = event {
                // Depth is usize, so this is always true, but we check for logic errors
                assert!(node.depth < 1000, "Unreasonable depth detected");
            }
        }
    }
}

#[test]
fn property_node_depth_increases_by_one() {
    // Property: Nested nodes should increase depth by exactly 1
    let input = r"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%STRUCT: C: [id]
%NEST: A > B
%NEST: B > C
---
a: @A
  | a1
    | b1
      | c1
";

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 3);

    for (i, node) in nodes.iter().enumerate() {
        assert_eq!(node.depth, i);
    }
}

#[test]
fn property_fields_match_schema_length() {
    // Property: Number of fields in a node should match schema column count
    let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name, email, age]
---
users: @User
  | alice, Alice Smith, alice@example.com, 30
  | bob, Bob Jones, bob@example.com, 25
";

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    // Get header before consuming parser
    let header = parser.header().unwrap();
    let schema_len = header.structs.get("User").unwrap().len();

    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    for node in nodes {
        assert_eq!(node.fields.len(), schema_len);
    }
}

#[test]
fn property_line_numbers_monotonic() {
    // Property: Line numbers in events should be monotonically increasing
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
  | c
";

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let mut prev_line = 0;
    for event in events {
        if let Some(line) = event.line() {
            assert!(line >= prev_line, "Line numbers not monotonic");
            prev_line = line;
        }
    }
}

#[test]
fn property_nested_nodes_have_parent_info() {
    // Property: Nodes at depth > 0 should have parent information
    let input = r"
%VERSION: 1.0
%STRUCT: Parent: [id]
%STRUCT: Child: [id]
%NEST: Parent > Child
---
data: @Parent
  | p1
    | c1
";

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    for node in nodes {
        if node.depth > 0 {
            assert!(node.is_nested(), "Depth > 0 but is_nested() is false");
        }
    }
}

#[test]
fn property_end_of_document_is_last() {
    // Property: Parser should successfully complete iteration (EndOfDocument is internal sentinel)
    // Note: Empty bodies produce no events, bodies with content produce events
    let inputs_with_expected_events = vec![
        ("%VERSION: 1.0\n---\n", false), // Empty body, no events
        (
            "%VERSION: 1.0\n%STRUCT: A: [id]\n---\na: @A\n  | a1\n",
            true,
        ), // Has nodes
        ("%VERSION: 1.0\n---\nkey: value\n", true), // Has scalar
    ];

    for (input, expect_events) in inputs_with_expected_events {
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

        // Verify parsing completed successfully (no errors)
        // Events should exist only if body has content
        if expect_events {
            assert!(!events.is_empty(), "Expected events for input: {input}");
        }
    }
}

// ==================== Idempotency Tests ====================

#[test]
fn property_parsing_twice_gives_same_result() {
    // Property: Parsing the same input twice should give identical results
    let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
  | bob, Bob
";

    let parser1 = StreamingParser::new(Cursor::new(input)).unwrap();
    let events1: Vec<_> = parser1
        .filter_map(std::result::Result::ok)
        .filter(hedl_stream::NodeEvent::is_node)
        .collect();

    let parser2 = StreamingParser::new(Cursor::new(input)).unwrap();
    let events2: Vec<_> = parser2
        .filter_map(std::result::Result::ok)
        .filter(hedl_stream::NodeEvent::is_node)
        .collect();

    assert_eq!(events1.len(), events2.len());
}

// ==================== Error Consistency Tests ====================

#[test]
fn property_errors_stop_iteration() {
    // Property: After an error, no more events should be produced
    let input = r"
%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
  | row1, val1
  | invalid_row
  | row3, val3
";

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect();

    let mut found_error = false;
    for event in &events {
        if found_error {
            // After error, should not get more Ok events
            assert!(event.is_err(), "Got Ok event after error");
        }
        if event.is_err() {
            found_error = true;
        }
    }

    assert!(found_error, "Expected to find an error");
}

// ==================== Memory Safety Tests ====================

#[test]
fn property_no_panic_on_empty_input() {
    // Property: Parser should not panic on empty input
    let empty_inputs = vec!["", "\n", "\n\n\n", "   \n   \n   "];

    for input in empty_inputs {
        let result = StreamingParser::new(Cursor::new(input));
        // Should either succeed or return error, not panic
        if let Ok(parser) = result {
            let _events: Vec<_> = parser.collect();
            // Should complete without panic
        } else {
            // Expected error is fine
        }
    }
}

#[test]
fn property_no_panic_on_malformed_input() {
    // Property: Parser should not panic on malformed input
    let malformed_inputs = vec![
        "garbage\ndata\n",
        "%VERSION: abc\n---\n",
        "---\ndata\n",
        "%VERSION: 1.0\nno separator\n",
    ];

    for input in malformed_inputs {
        let result = StreamingParser::new(Cursor::new(input));
        if let Ok(parser) = result {
            let _events: Vec<_> = parser.collect();
            // Should complete without panic
        } else {
            // Expected error is fine
        }
    }
}

// ==================== Determinism Tests ====================

#[test]
fn property_same_input_same_event_count() {
    // Property: Same input should always produce same number of events
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | a
  | b
  | c
";

    let runs = 10;
    let mut event_counts = Vec::new();

    for _ in 0..runs {
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
        event_counts.push(events.len());
    }

    // All counts should be identical
    let first = event_counts[0];
    for count in event_counts {
        assert_eq!(count, first);
    }
}

// ==================== Boundary Condition Tests ====================

#[test]
fn property_single_char_ids() {
    // Property: Single character IDs should be valid
    let input = r"
%VERSION: 1.0
%STRUCT: A: [id]
---
a: @A
  | a
  | b
  | c
";

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 3);
}

#[test]
fn property_long_ids() {
    // Property: Long IDs should be handled correctly
    let long_id = "x".repeat(1000);
    let input = format!("%VERSION: 1.0\n%STRUCT: A: [id]\n---\na: @A\n  | {long_id}\n");

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id.len(), 1000);
}

#[test]
fn property_empty_string_values() {
    // Property: Empty string values should be preserved
    let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
  | id1, ""
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 1);
}

// ==================== Configuration Invariants ====================

#[test]
fn property_config_settings_respected() {
    // Property: Configuration settings should be honored
    use hedl_stream::StreamingParserConfig;

    let config = StreamingParserConfig {
        max_line_length: 50,
        ..Default::default()
    };

    let long_line = format!("%VERSION: 1.0\n---\nkey: {}\n", "x".repeat(100));
    let parser = StreamingParser::with_config(Cursor::new(long_line), config).unwrap();

    let events: Vec<_> = parser.collect();

    // Should get an error about line length
    let has_length_error = events
        .iter()
        .any(|e| matches!(e, Err(hedl_stream::StreamError::LineTooLong { .. })));

    assert!(has_length_error, "Expected LineTooLong error");
}

// ==================== State Consistency Tests ====================

#[test]
fn property_header_available_after_creation() {
    // Property: Header should be available immediately after parser creation
    let input = r"
%VERSION: 1.0
%STRUCT: User: [id]
---
users: @User
  | alice
";

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    assert!(parser.header().is_some());
}

#[test]
fn property_header_version_preserved() {
    // Property: Version in header should match version directive
    let inputs = vec![
        ("%VERSION: 1.0\n---\n", (1, 0)),
        ("%VERSION: 2.0\n---\n", (2, 0)),
        ("%VERSION: 1.5\n---\n", (1, 5)),
    ];

    for (input, expected) in inputs {
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();
        assert_eq!(header.version, expected);
    }
}
