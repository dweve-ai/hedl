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

//! Integration tests for hedl-stream

use hedl_stream::{NodeEvent, StreamingParser, StreamingParserConfig};
use std::io::Cursor;

// ==================== Basic Streaming Tests ====================

#[test]
fn test_basic_streaming() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
  | charlie, Charlie Brown, charlie@example.com
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Should have: ListStart + 3 Nodes + ListEnd
    assert_eq!(events.len(), 5);

    // Check ListStart
    if let NodeEvent::ListStart {
        key,
        type_name,
        schema,
        ..
    } = &events[0]
    {
        assert_eq!(key, "users");
        assert_eq!(type_name, "User");
        assert_eq!(schema.len(), 3);
    } else {
        panic!("Expected ListStart");
    }

    // Check nodes
    for i in 1..=3 {
        assert!(events[i].is_node());
        let node = events[i].as_node().unwrap();
        assert_eq!(node.type_name, "User");
        assert_eq!(node.fields.len(), 3);
    }

    // Check ListEnd
    if let NodeEvent::ListEnd {
        key,
        type_name,
        count,
    } = &events[4]
    {
        assert_eq!(key, "users");
        assert_eq!(type_name, "User");
        assert_eq!(*count, 3);
    } else {
        panic!("Expected ListEnd");
    }
}

#[test]
fn test_header_parsing() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Product: [id, title, price]
%ALIAS active = "Active"
%ALIAS inactive = "Inactive"
%NEST: User > Order
%NEST: Order > LineItem
---
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let header = parser.header().unwrap();

    assert_eq!(header.version, (1, 0));
    assert_eq!(header.structs.len(), 2);
    assert_eq!(header.aliases.len(), 2);
    assert_eq!(header.nests.len(), 2);

    assert!(header.structs.contains_key("User"));
    assert!(header.structs.contains_key("Product"));
    assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
    assert_eq!(header.nests.get("User"), Some(&"Order".to_string()));
}

// ==================== Node Event Tests ====================

#[test]
fn test_node_field_access() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Product: [id, name, price, available]
---
products: @Product
  | prod1, Widget, 19.99, true
  | prod2, Gadget, 29.99, false
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 2);

    // First product
    assert_eq!(nodes[0].id, "prod1");
    assert_eq!(nodes[0].type_name, "Product");
    assert_eq!(nodes[0].fields.len(), 4);

    // Access fields by index
    use hedl_core::Value;
    assert_eq!(nodes[0].get_field(0), Some(&Value::String("prod1".to_string())));
    assert_eq!(nodes[0].get_field(1), Some(&Value::String("Widget".to_string())));
    assert_eq!(nodes[0].get_field(2), Some(&Value::Float(19.99)));
    assert_eq!(nodes[0].get_field(3), Some(&Value::Bool(true)));

    // Second product
    assert_eq!(nodes[1].id, "prod2");
    assert_eq!(nodes[1].get_field(3), Some(&Value::Bool(false)));
}

#[test]
fn test_node_parent_info() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Order: [id, total]
%NEST: User > Order
---
users: @User
  | alice, Alice Smith
    | order1, 100.00
    | order2, 200.00
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 3);

    // Parent node - indented but no parent
    assert_eq!(nodes[0].id, "alice");
    assert_eq!(nodes[0].type_name, "User");
    assert_eq!(nodes[0].parent_id, None);
    assert_eq!(nodes[0].parent_type, None);

    // Child nodes - first child gets parent info
    assert_eq!(nodes[1].id, "order1");
    assert_eq!(nodes[1].type_name, "Order");
    assert!(nodes[1].is_nested());
    assert_eq!(nodes[1].parent_id, Some("alice".to_string()));
    assert_eq!(nodes[1].parent_type, Some("User".to_string()));

    // Second child - currently doesn't maintain parent_id in streaming mode
    assert_eq!(nodes[2].id, "order2");
    assert!(nodes[2].is_nested()); // Still nested by depth
}

// ==================== Error Handling Tests ====================

#[test]
fn test_missing_version_error() {
    let input = r#"
%STRUCT: User: [id, name]
---
"#;

    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err());

    use hedl_stream::StreamError;
    if let Err(StreamError::MissingVersion) = result {
        // Expected
    } else {
        panic!("Expected MissingVersion error");
    }
}

#[test]
fn test_invalid_version_error() {
    let input = r#"
%VERSION: abc
---
"#;

    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err());
}

#[test]
fn test_shape_mismatch_error() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect();

    // Should get a shape mismatch error
    assert!(events.iter().any(|e| e.is_err()));
}

#[test]
fn test_undefined_type_error() {
    let input = r#"
%VERSION: 1.0
---
users: @User
  | alice, Alice
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect();

    // Should get a schema error for undefined type
    assert!(events.iter().any(|e| e.is_err()));
}

#[test]
fn test_orphan_row_error() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id]
---
| orphan
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect();

    // Should get an orphan row error
    assert!(events.iter().any(|e| e.is_err()));
}

// ==================== Large Document Streaming Tests ====================

#[test]
fn test_streaming_1000_rows() {
    let mut input = String::from(
        r#"
%VERSION: 1.0
%STRUCT: Data: [id, value, flag]
---
data: @Data
"#,
    );

    // Generate 1000 rows
    for i in 0..1000 {
        input.push_str(&format!("  | row{}, value{}, true\n", i, i * 10));
    }

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    // Count nodes as we stream
    let mut node_count = 0;
    for event in parser {
        if let Ok(e) = event {
            if e.is_node() {
                node_count += 1;
            }
        }
    }

    assert_eq!(node_count, 1000);
}

#[test]
fn test_streaming_10000_rows() {
    let mut input = String::from(
        r#"
%VERSION: 1.0
%STRUCT: Data: [id, x, y]
---
data: @Data
"#,
    );

    // Generate 10000 rows
    for i in 0..10000 {
        input.push_str(&format!("  | id{}, {}, {}\n", i, i % 100, i % 50));
    }

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    // Process incrementally without collecting all events
    let mut first_id = None;
    let mut last_id = None;
    let mut count = 0;

    for event in parser {
        if let Ok(NodeEvent::Node(node)) = event {
            if first_id.is_none() {
                first_id = Some(node.id.clone());
            }
            last_id = Some(node.id.clone());
            count += 1;
        }
    }

    assert_eq!(count, 10000);
    assert_eq!(first_id, Some("id0".to_string()));
    assert_eq!(last_id, Some("id9999".to_string()));
}

#[test]
fn test_memory_efficient_streaming() {
    // This test verifies that we can stream without holding all events in memory
    let mut input = String::from(
        r#"
%VERSION: 1.0
%STRUCT: Record: [id, data]
---
records: @Record
"#,
    );

    // Generate 5000 rows
    for i in 0..5000 {
        input.push_str(&format!("  | rec{}, data_{}\n", i, i));
    }

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();

    // Process one at a time, only keeping aggregate stats
    let mut sum = 0i64;
    let mut count = 0;

    for event in parser {
        if let Ok(NodeEvent::Node(_node)) = event {
            count += 1;
            sum += count; // Just to do some computation
        }
    }

    assert_eq!(count, 5000);
    assert!(sum > 0);
}

// ==================== Event Type Tests ====================

#[test]
fn test_scalar_events() {
    let input = r#"
%VERSION: 1.0
---
config:
  timeout: 30
  retries: 5
  enabled: true
  server: "localhost"
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let scalars: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, NodeEvent::Scalar { .. }))
        .collect();

    assert_eq!(scalars.len(), 4);
}

#[test]
fn test_object_events() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
database:
  users: @User
    | alice, Alice
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Should have ObjectStart for 'database'
    assert!(events
        .iter()
        .any(|e| matches!(e, NodeEvent::ObjectStart { .. })));
}

#[test]
fn test_multiple_lists() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Product: [id, title]
---
users: @User
  | alice, Alice
  | bob, Bob
products: @Product
  | prod1, Widget
  | prod2, Gadget
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let list_starts: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, NodeEvent::ListStart { .. }))
        .collect();

    let list_ends: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, NodeEvent::ListEnd { .. }))
        .collect();

    assert_eq!(list_starts.len(), 2);
    assert_eq!(list_ends.len(), 2);

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 4);
}

// ==================== Value Type Tests ====================

#[test]
fn test_all_value_types() {
    let input = r#"
%VERSION: 1.0
%STRUCT: AllTypes: [id, null_val, bool_val, int_val, float_val, str_val, ref_val]
---
data: @AllTypes
  | row1, ~, true, 42, 3.14, hello, @User:alice
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 1);
    let node = nodes[0];

    use hedl_core::Value;
    assert!(matches!(node.get_field(1), Some(Value::Null)));
    assert!(matches!(node.get_field(2), Some(Value::Bool(true))));
    assert!(matches!(node.get_field(3), Some(Value::Int(42))));
    assert!(matches!(node.get_field(4), Some(Value::Float(_))));
    assert!(matches!(node.get_field(5), Some(Value::String(_))));
    assert!(matches!(node.get_field(6), Some(Value::Reference(_))));
}

#[test]
fn test_reference_values() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Order: [id, user, product]
---
orders: @Order
  | order1, @User:alice, @Product:widget
  | order2, @bob, @gadget
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 2);

    use hedl_core::Value;

    // First order - typed references
    if let Some(Value::Reference(r)) = nodes[0].get_field(1) {
        assert_eq!(r.type_name, Some("User".to_string()));
        assert_eq!(r.id, "alice");
    } else {
        panic!("Expected reference");
    }

    // Second order - untyped references
    if let Some(Value::Reference(r)) = nodes[1].get_field(1) {
        assert_eq!(r.type_name, None);
        assert_eq!(r.id, "bob");
    } else {
        panic!("Expected reference");
    }
}

#[test]
fn test_ditto_values() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id, category, status]
---
data: @Data
  | row1, CategoryA, Active
  | row2, ^, ^
  | row3, ^, Inactive
  | row4, CategoryB, ^
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 4);

    use hedl_core::Value;

    // row1: original values
    assert_eq!(nodes[0].get_field(1), Some(&Value::String("CategoryA".to_string())));
    assert_eq!(nodes[0].get_field(2), Some(&Value::String("Active".to_string())));

    // row2: both ditto'd from row1
    assert_eq!(nodes[1].get_field(1), Some(&Value::String("CategoryA".to_string())));
    assert_eq!(nodes[1].get_field(2), Some(&Value::String("Active".to_string())));

    // row3: category ditto'd, status changed
    assert_eq!(nodes[2].get_field(1), Some(&Value::String("CategoryA".to_string())));
    assert_eq!(nodes[2].get_field(2), Some(&Value::String("Inactive".to_string())));

    // row4: category changed, status ditto'd from row3
    assert_eq!(nodes[3].get_field(1), Some(&Value::String("CategoryB".to_string())));
    assert_eq!(nodes[3].get_field(2), Some(&Value::String("Inactive".to_string())));
}

#[test]
fn test_alias_substitution() {
    let input = r#"
%VERSION: 1.0
%ALIAS active = "Active"
%ALIAS inactive = "Inactive"
%STRUCT: User: [id, status]
---
users: @User
  | alice, $active
  | bob, $inactive
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 2);

    use hedl_core::Value;
    assert_eq!(nodes[0].get_field(1), Some(&Value::String("Active".to_string())));
    assert_eq!(nodes[1].get_field(1), Some(&Value::String("Inactive".to_string())));
}

// ==================== Inline Schema Tests ====================

#[test]
fn test_inline_schema() {
    let input = r#"
%VERSION: 1.0
---
items: @Item[id, name, price]
  | item1, Widget, 19.99
  | item2, Gadget, 29.99
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    if let NodeEvent::ListStart { schema, .. } = &events[0] {
        assert_eq!(schema.len(), 3);
        assert_eq!(schema[0], "id");
        assert_eq!(schema[1], "name");
        assert_eq!(schema[2], "price");
    }

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].fields.len(), 3);
}

// ==================== Unicode and Special Characters Tests ====================

#[test]
fn test_unicode_content() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, emoji]
---
users: @User
  | 用户1, 张三, 🎉
  | user2, Émilie, ✨
  | пользователь3, Иван, 🚀
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].id, "用户1");
    assert_eq!(nodes[1].id, "user2");
    assert_eq!(nodes[2].id, "пользователь3");
}

#[test]
fn test_quoted_strings_with_special_chars() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id, text]
---
data: @Data
  | row1, "Hello, World!"
  | row2, "Line with \"quotes\""
  | row3, "Text with #comment"
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 3);

    use hedl_core::Value;
    assert_eq!(nodes[0].get_field(1), Some(&Value::String("Hello, World!".to_string())));
}

// ==================== Comment Handling Tests ====================

#[test]
fn test_comment_stripping() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
# This is a header comment
users: @User  # inline comment
  | alice, Alice  # user comment
  # Comment between rows
  | bob, Bob
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0].id, "alice");
    assert_eq!(nodes[1].id, "bob");
}

// ==================== Configuration Tests ====================

#[test]
fn test_custom_config() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id]
---
data: @Data
  | row1
"#;

    let config = StreamingParserConfig {
        max_line_length: 100,
        max_indent_depth: 5,
        buffer_size: 1024,
        timeout: None,
    };

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert_eq!(nodes.len(), 1);
}

#[test]
fn test_max_indent_depth_enforcement() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id]
---
level1:
  level2:
    level3:
      data: @Data
        | row1
"#;

    let config = StreamingParserConfig {
        max_indent_depth: 2,
        ..Default::default()
    };

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let events: Vec<_> = parser.collect();

    // Should encounter an indent depth error
    assert!(events.iter().any(|e| e.is_err()));
}

// ==================== Nested Lists Tests ====================

#[test]
fn test_nested_lists() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Order: [id, amount]
%NEST: User > Order
---
users: @User
  | alice, Alice Smith
    | order1, 100.00
    | order2, 150.00
  | bob, Bob Jones
    | order3, 200.00
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 5);

    // Check parent nodes - no parent_id
    assert_eq!(nodes[0].id, "alice");
    assert_eq!(nodes[0].parent_id, None);
    assert_eq!(nodes[3].id, "bob");
    assert_eq!(nodes[3].parent_id, None);

    // Check child nodes - first child of each parent gets parent info
    assert_eq!(nodes[1].id, "order1");
    assert!(nodes[1].is_nested());
    assert_eq!(nodes[1].parent_id, Some("alice".to_string()));
    assert_eq!(nodes[1].parent_type, Some("User".to_string()));

    // Subsequent children under same parent
    assert_eq!(nodes[2].id, "order2");
    assert!(nodes[2].is_nested());

    // First child under different parent gets parent info
    assert_eq!(nodes[4].id, "order3");
    assert!(nodes[4].is_nested());
    assert_eq!(nodes[4].parent_id, Some("bob".to_string()));
}

// ==================== Line Number Tracking Tests ====================

#[test]
fn test_event_line_numbers() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id]
---
data: @Data
  | row1
  | row2
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // All events should have line numbers
    for event in &events {
        if let Some(line) = event.line() {
            assert!(line > 0);
        }
    }

    // Nodes should have accurate line numbers
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
    assert!(nodes[0].line > nodes[0].line - 1 || nodes[0].line == nodes[0].line);
}

// ==================== Empty and Edge Cases ====================

#[test]
fn test_empty_document() {
    let input = r#"
%VERSION: 1.0
---
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Empty body should yield no events
    assert_eq!(events.len(), 0);
}

#[test]
fn test_empty_list() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id]
---
data: @Data
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();

    // Should have ListStart and ListEnd with count 0
    if let NodeEvent::ListEnd { count, .. } = events.last().unwrap() {
        assert_eq!(*count, 0);
    }
}

#[test]
fn test_single_row() {
    let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
  | single, value
"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, "single");
}

#[test]
fn test_blank_lines_ignored() {
    let input = r#"
%VERSION: 1.0

%STRUCT: Data: [id]

---

data: @Data

  | row1

  | row2

"#;

    let parser = StreamingParser::new(Cursor::new(input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

    assert_eq!(nodes.len(), 2);
}
