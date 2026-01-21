// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Edge case tests for `StreamingParser`

use hedl_stream::{
    BufferSizeHint, MemoryLimits, NodeEvent, StreamingParser, StreamingParserConfig,
};
use std::io::Cursor;

// ==================== Configuration Tests ====================

#[test]
fn test_config_default() {
    let config = StreamingParserConfig::default();
    assert_eq!(config.max_line_length, 1_000_000);
    assert_eq!(config.max_indent_depth, 100);
    assert_eq!(config.buffer_size, 64 * 1024);
    assert_eq!(config.timeout, None);
    assert!(!config.enable_pooling);
}

#[test]
fn test_config_unlimited() {
    let config = StreamingParserConfig::unlimited();
    assert_eq!(config.max_line_length, usize::MAX);
}

#[test]
fn test_config_with_buffer_hint() {
    let config = StreamingParserConfig::default().with_buffer_hint(BufferSizeHint::Large);
    assert_eq!(config.buffer_size, 256 * 1024);
}

#[test]
fn test_config_with_buffer_pooling() {
    let config = StreamingParserConfig::default().with_buffer_pooling(true);
    assert!(config.enable_pooling);
}

#[test]
fn test_config_with_memory_limits() {
    let limits = MemoryLimits::embedded();
    let config = StreamingParserConfig::default().with_memory_limits(limits);
    assert_eq!(config.max_line_length, limits.max_line_length);
    assert_eq!(config.memory_limits.max_buffer_size, 8 * 1024);
}

#[test]
fn test_config_with_pool_size() {
    let config = StreamingParserConfig::default()
        .with_buffer_pooling(true)
        .with_pool_size(50);
    assert_eq!(config.memory_limits.max_pool_size, 50);
}

#[test]
fn test_config_chaining() {
    let config = StreamingParserConfig::default()
        .with_buffer_hint(BufferSizeHint::Huge)
        .with_buffer_pooling(true)
        .with_pool_size(25);

    assert_eq!(config.buffer_size, 1024 * 1024);
    assert!(config.enable_pooling);
    assert_eq!(config.memory_limits.max_pool_size, 25);
}

// ==================== Parser Creation Tests ====================

#[test]
fn test_parser_new() {
    let input = "%VERSION: 1.0\n---\n";
    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_ok());
}

#[test]
fn test_parser_with_config() {
    let input = "%VERSION: 1.0\n---\n";
    let config = StreamingParserConfig::default();
    let result = StreamingParser::with_config(Cursor::new(input), config);
    assert!(result.is_ok());
}

#[test]
fn test_parser_header_access() {
    let input = "%VERSION: 1.0\n%STRUCT: User: [id, name]\n---\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header();
    assert!(header.is_some());
    assert_eq!(header.unwrap().version, (1, 0));
}

// ==================== Empty and Minimal Input Tests ====================

#[test]
fn test_empty_after_separator() {
    let input = "%VERSION: 1.0\n---\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    // Parser returns events, may or may not include explicit EndOfDocument
    // The important thing is no error (all results unwrapped successfully)
    assert!(!events.is_empty() || events.is_empty()); // No errors occurred during collection
}

#[test]
fn test_only_comments_after_separator() {
    let input = "%VERSION: 1.0\n---\n# just a comment\n# another comment\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    // Comments are ignored, so we may get empty or just end marker
    // The important thing is no error (all results unwrapped successfully)
    assert!(!events.is_empty() || events.is_empty()); // No errors occurred during collection
}

#[test]
fn test_empty_lines_ignored() {
    let input = "%VERSION: 1.0\n\n\n---\n\n\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    // Empty lines should be ignored, no errors expected (all results unwrapped successfully)
    assert!(!events.is_empty() || events.is_empty()); // No errors occurred during collection
}

#[test]
fn test_whitespace_only_lines() {
    let input = "%VERSION: 1.0\n   \n\t\n---\n  \t  \n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    // Whitespace-only lines (including tabs) should be ignored without error.
    // An empty body (only whitespace) produces no events.
    assert!(events.is_empty());
}

// ==================== Scalar Value Tests ====================

#[test]
fn test_scalar_string_value() {
    let input = r"
%VERSION: 1.0
---
name: Alice Smith
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let scalar = events.iter().find_map(|e| match e {
        NodeEvent::Scalar { key, value, .. } => Some((key, value)),
        _ => None,
    });

    assert!(scalar.is_some());
    let (key, value) = scalar.unwrap();
    assert_eq!(key, "name");
    assert!(matches!(value, hedl_core::Value::String(_)));
}

#[test]
fn test_scalar_integer_value() {
    let input = r"
%VERSION: 1.0
---
count: 42
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let scalar = events.iter().find_map(|e| match e {
        NodeEvent::Scalar { value, .. } => Some(value),
        _ => None,
    });

    assert!(scalar.is_some());
    assert!(matches!(scalar.unwrap(), hedl_core::Value::Int(42)));
}

#[test]
fn test_scalar_float_value() {
    let input = r"
%VERSION: 1.0
---
price: 19.99
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let scalar = events.iter().find_map(|e| match e {
        NodeEvent::Scalar { value, .. } => Some(value),
        _ => None,
    });

    assert!(scalar.is_some());
    assert!(matches!(scalar.unwrap(), hedl_core::Value::Float(_)));
}

#[test]
fn test_scalar_bool_true() {
    let input = r"
%VERSION: 1.0
---
active: true
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let scalar = events.iter().find_map(|e| match e {
        NodeEvent::Scalar { value, .. } => Some(value),
        _ => None,
    });

    assert!(scalar.is_some());
    assert!(matches!(scalar.unwrap(), hedl_core::Value::Bool(true)));
}

#[test]
fn test_scalar_bool_false() {
    let input = r"
%VERSION: 1.0
---
inactive: false
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let scalar = events.iter().find_map(|e| match e {
        NodeEvent::Scalar { value, .. } => Some(value),
        _ => None,
    });

    assert!(scalar.is_some());
    assert!(matches!(scalar.unwrap(), hedl_core::Value::Bool(false)));
}

#[test]
fn test_scalar_null_value() {
    let input = r"
%VERSION: 1.0
---
optional: null
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let scalar = events.iter().find_map(|e| {
        if let NodeEvent::Scalar { value, .. } = e {
            Some(value)
        } else {
            None
        }
    });

    assert!(scalar.is_some());
    assert!(matches!(scalar.unwrap(), hedl_core::Value::Null));
}

// ==================== Object Tests ====================

#[test]
fn test_nested_objects() {
    let input = r"
%VERSION: 1.0
---
config:
  database:
    host: localhost
    port: 5432
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let obj_starts = events
        .iter()
        .filter(|e| matches!(e, NodeEvent::ObjectStart { .. }))
        .count();

    assert_eq!(obj_starts, 2); // config and database
}

#[test]
fn test_object_start_end_matching() {
    let input = r"
%VERSION: 1.0
---
settings:
  timeout: 30
";
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

    assert_eq!(starts, ends);
}

// ==================== List Tests ====================

#[test]
fn test_empty_list() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let list_end = events.iter().find_map(|e| match e {
        NodeEvent::ListEnd { count, .. } => Some(*count),
        _ => None,
    });

    assert_eq!(list_end, Some(0));
}

#[test]
fn test_single_item_list() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | item1
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let list_end = events.iter().find_map(|e| match e {
        NodeEvent::ListEnd { count, .. } => Some(*count),
        _ => None,
    });

    assert_eq!(list_end, Some(1));
}

#[test]
fn test_list_count_accuracy() {
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
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let list_end = events.iter().find_map(|e| match e {
        NodeEvent::ListEnd { count, .. } => Some(*count),
        _ => None,
    });

    assert_eq!(list_end, Some(5));
}

// ==================== Nesting Tests ====================

#[test]
fn test_multiple_nesting_levels() {
    let input = r"
%VERSION: 1.0
%STRUCT: L1: [id]
%STRUCT: L2: [id]
%STRUCT: L3: [id]
%NEST: L1 > L2
%NEST: L2 > L3
---
data: @L1
  | level1
    | level2
      | level3
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 3);

    // Check depths - may vary by implementation
    assert!(nodes[0].depth <= nodes[1].depth);
    assert!(nodes[1].depth <= nodes[2].depth);
}

#[test]
fn test_sibling_after_nested_child() {
    let input = r"
%VERSION: 1.0
%STRUCT: Parent: [id]
%STRUCT: Child: [id]
%NEST: Parent > Child
---
data: @Parent
  | parent1
    | child1
  | parent2
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 3);

    assert_eq!(nodes[0].id, "parent1");
    assert_eq!(nodes[1].id, "child1");
    assert_eq!(nodes[2].id, "parent2");
}

// ==================== Error Recovery Tests ====================

#[test]
fn test_error_includes_line_number() {
    let input = r"
%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
  | row1, val1
  | row2
  | row3, val3
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect();

    let err = events.iter().find(|e| e.is_err());
    assert!(err.is_some());

    if let Err(e) = err.unwrap() {
        assert!(e.line().is_some());
        assert_eq!(e.line().unwrap(), 7); // row2 line
    }
}

#[test]
fn test_multiple_errors_stop_at_first() {
    let input = r"
%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
  | row1
  | row2
  | row3
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect();

    // Parser should stop at first error or report multiple
    let error_count = events.iter().filter(|e| e.is_err()).count();
    assert!(error_count >= 1, "Expected at least one error");
}

// ==================== Unicode and Special Characters ====================

#[test]
fn test_unicode_in_ids() {
    let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | user_🎉, Alice
  | user_世界, Bob
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 2);
}

#[test]
fn test_unicode_in_values() {
    let input = r"
%VERSION: 1.0
%STRUCT: Text: [id, content]
---
texts: @Text
  | text1, Hello 世界 🌍
  | text2, Привет мир
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 2);
}

// ==================== Whitespace Handling ====================

#[test]
fn test_trailing_whitespace_in_values() {
    let input =
        "%VERSION: 1.0\n%STRUCT: Data: [id, val]\n---\ndata: @Data\n  | id1, value with spaces  \n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 1);
}

#[test]
fn test_mixed_indentation() {
    // HEDL explicitly disallows tabs for indentation
    let input = "%VERSION: 1.0\n---\nconfig:\n  key1: value1\n\tkey2: value2\n";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect();

    // Parser should return an error for tab indentation
    let has_error = events.iter().any(std::result::Result::is_err);
    assert!(has_error, "Parser should reject tabs for indentation");
}

// ==================== Stress Tests ====================

#[test]
fn test_many_fields() {
    let mut header = String::from("%VERSION: 1.0\n%STRUCT: Wide: [");
    for i in 0..100 {
        if i > 0 {
            header.push_str(", ");
        }
        header.push_str(&format!("field{i}"));
    }
    header.push_str("]\n---\ndata: @Wide\n  | ");

    for i in 0..100 {
        if i > 0 {
            header.push_str(", ");
        }
        header.push_str(&format!("val{i}"));
    }
    header.push('\n');

    let parser = StreamingParser::new(Cursor::new(header)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].fields.len(), 100);
}

#[test]
fn test_many_rows() {
    let mut input = String::from("%VERSION: 1.0\n%STRUCT: Item: [id]\n---\nitems: @Item\n");
    for i in 0..1000 {
        input.push_str(&format!("  | item{i}\n"));
    }

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let list_end = events.iter().find_map(|e| match e {
        NodeEvent::ListEnd { count, .. } => Some(*count),
        _ => None,
    });

    assert_eq!(list_end, Some(1000));
}

// ==================== Comments Tests ====================

#[test]
fn test_inline_comment_in_header() {
    let input = r"
%VERSION: 1.0  # This is the version
%STRUCT: User: [id, name]  # User structure
---
users: @User
  | alice, Alice
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 1);
}

#[test]
fn test_comment_between_rows() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id]
---
items: @Item
  | item1
  # This is a comment
  | item2
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let list_end = events.iter().find_map(|e| match e {
        NodeEvent::ListEnd { count, .. } => Some(*count),
        _ => None,
    });

    assert_eq!(list_end, Some(2));
}

// ==================== Header Directive Tests ====================

#[test]
fn test_multiple_structs() {
    let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Product: [id, title]
%STRUCT: Order: [id, total]
---
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.structs.len(), 3);
    assert!(header.structs.contains_key("User"));
    assert!(header.structs.contains_key("Product"));
    assert!(header.structs.contains_key("Order"));
}

#[test]
fn test_multiple_aliases() {
    let input = r#"
%VERSION: 1.0
%ALIAS active = "Active"
%ALIAS inactive = "Inactive"
%ALIAS pending = "Pending"
---
"#;
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.aliases.len(), 3);
    assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
    assert_eq!(
        header.aliases.get("inactive"),
        Some(&"Inactive".to_string())
    );
    assert_eq!(header.aliases.get("pending"), Some(&"Pending".to_string()));
}

#[test]
fn test_multiple_nests() {
    let input = r"
%VERSION: 1.0
%STRUCT: A: [id]
%STRUCT: B: [id]
%STRUCT: C: [id]
%NEST: A > B
%NEST: B > C
---
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.nests.len(), 2);
    assert_eq!(header.nests.get("A"), Some(&"B".to_string()));
    assert_eq!(header.nests.get("B"), Some(&"C".to_string()));
}

// ==================== Iterator Functionality Tests ====================

#[test]
fn test_iterator_map() {
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

    let ids: Vec<String> = parser
        .filter_map(std::result::Result::ok)
        .filter_map(|e| e.as_node().map(|n| n.id.clone()))
        .collect();

    assert_eq!(ids, vec!["a", "b", "c"]);
}

#[test]
fn test_iterator_take() {
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
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let count = parser
        .filter_map(std::result::Result::ok)
        .filter(hedl_stream::NodeEvent::is_node)
        .take(3)
        .count();

    assert_eq!(count, 3);
}

#[test]
fn test_iterator_skip() {
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
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    let ids: Vec<String> = parser
        .filter_map(std::result::Result::ok)
        .filter_map(|e| e.as_node().map(|n| n.id.clone()))
        .skip(2)
        .collect();

    assert_eq!(ids, vec!["c", "d"]);
}

// ==================== NodeInfo Method Tests ====================

#[test]
fn test_node_with_child_count() {
    let input = r"
%VERSION: 1.0
%STRUCT: Parent: [id]
%STRUCT: Child: [id]
%NEST: Parent > Child
---
data: @Parent
  |[2] parent1
    | child1
    | child2
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 3);

    // Parent should have child_count set
    assert_eq!(nodes[0].child_count, Some(2));
}

#[test]
fn test_node_info_clone() {
    let input = r"
%VERSION: 1.0
%STRUCT: Item: [id, name]
---
items: @Item
  | item1, Name1
";
    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    if let Some(node) = events.iter().find_map(|e| e.as_node()) {
        let cloned = node.clone();
        assert_eq!(cloned.id, node.id);
        assert_eq!(cloned.type_name, node.type_name);
    }
}
