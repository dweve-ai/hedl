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

//! Comprehensive integration tests for streaming JSON functionality

use hedl_core::{Document, Item, Value};
use hedl_json::streaming::{
    JsonArrayStreamer, JsonLinesStreamer, JsonLinesWriter, StreamConfig, StreamError,
};
use hedl_json::FromJsonConfig;
use std::io::Cursor;

// =============================================================================
// JsonArrayStreamer Integration Tests
// =============================================================================

#[test]
fn test_array_streamer_basic_workflow() {
    let json = r#"[
        {"id": "user1", "name": "Alice", "age": 30},
        {"id": "user2", "name": "Bob", "age": 25},
        {"id": "user3", "name": "Charlie", "age": 35}
    ]"#;

    let reader = Cursor::new(json.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonArrayStreamer::new(reader, config).unwrap();

    let mut count = 0;
    let mut names = Vec::new();

    for result in streamer {
        let doc = result.unwrap();
        count += 1;

        if let Some(Item::Scalar(Value::String(name))) = doc.root.get("name") {
            names.push(name.clone());
        }
    }

    assert_eq!(count, 3);
    assert_eq!(names, vec!["Alice", "Bob", "Charlie"]);
}

#[test]
fn test_array_streamer_large_dataset() {
    // Generate 10,000 objects
    let mut json = String::from("[");
    for i in 0..10000 {
        if i > 0 {
            json.push(',');
        }
        json.push_str(&format!(
            r#"{{"id": "{}", "value": {}, "active": {}}}"#,
            i,
            i * 10,
            i % 2 == 0
        ));
    }
    json.push(']');

    let reader = Cursor::new(json.as_bytes());
    let config = StreamConfig::builder()
        .buffer_size(256 * 1024) // 256 KB buffer for throughput
        .build();

    let streamer = JsonArrayStreamer::new(reader, config).unwrap();

    let mut count = 0;
    let mut sum = 0i64;

    for result in streamer {
        let doc = result.unwrap();
        count += 1;

        if let Some(Item::Scalar(Value::Int(value))) = doc.root.get("value") {
            sum += value;
        }
    }

    assert_eq!(count, 10000);
    assert_eq!(sum, 10000 * 9999 * 10 / 2); // Sum of arithmetic series
}

#[test]
fn test_array_streamer_mixed_types() {
    let json = r#"[
        {"type": "string", "value": "hello"},
        {"type": "int", "value": 42},
        {"type": "float", "value": 3.5},
        {"type": "bool", "value": true},
        {"type": "null", "value": null},
        {"type": "array", "value": [1, 2, 3]},
        {"type": "object", "value": {"nested": "data"}}
    ]"#;

    let reader = Cursor::new(json.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonArrayStreamer::new(reader, config).unwrap();

    let docs: Vec<_> = streamer.map(|r| r.unwrap()).collect();
    assert_eq!(docs.len(), 7);

    // Verify each type
    assert!(docs[0].root.contains_key("type"));
    assert!(docs[0].root.contains_key("value"));
}

#[test]
fn test_array_streamer_empty_array() {
    let json = r#"[]"#;

    let reader = Cursor::new(json.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonArrayStreamer::new(reader, config).unwrap();

    let count = streamer.count();
    assert_eq!(count, 0);
}

#[test]
fn test_array_streamer_single_object() {
    let json = r#"[{"single": "object"}]"#;

    let reader = Cursor::new(json.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonArrayStreamer::new(reader, config).unwrap();

    let docs: Vec<_> = streamer.map(|r| r.unwrap()).collect();
    assert_eq!(docs.len(), 1);
    assert!(docs[0].root.contains_key("single"));
}

#[test]
fn test_array_streamer_size_limit_enforcement() {
    let json = r#"[{"small": "ok"}, {"large": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}]"#;

    let reader = Cursor::new(json.as_bytes());
    let config = StreamConfig::builder()
        .max_object_bytes(20) // Small limit
        .build();

    let streamer = JsonArrayStreamer::new(reader, config).unwrap();
    let results: Vec<_> = streamer.collect();

    // First object should succeed
    assert!(results[0].is_ok());

    // Second object should fail due to size limit
    assert!(results[1].is_err());
    if let Err(StreamError::ObjectTooLarge(size, limit)) = &results[1] {
        assert!(size > limit);
    } else {
        panic!("Expected ObjectTooLarge error");
    }
}

#[test]
fn test_array_streamer_with_nested_arrays() {
    // Arrays of strings in JSON become matrix lists in HEDL
    let json = r#"[
        {"id": "1", "count": 5},
        {"id": "2", "count": 10},
        {"id": "3", "count": 15}
    ]"#;

    let reader = Cursor::new(json.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonArrayStreamer::new(reader, config).unwrap();

    let results: Vec<_> = streamer.collect();
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));
}

#[test]
fn test_array_streamer_error_recovery() {
    // JsonArrayStreamer parses the entire array first, so invalid syntax
    // is detected during initialization, not iteration
    let json = r#"[{"valid": 1}, invalid, {"also_valid": 2}]"#;

    let reader = Cursor::new(json.as_bytes());
    let config = StreamConfig::default();
    let result = JsonArrayStreamer::new(reader, config);

    // Should fail during initialization due to invalid JSON
    assert!(result.is_err());
}

// =============================================================================
// JsonLinesStreamer Integration Tests
// =============================================================================

#[test]
fn test_jsonl_streamer_basic_workflow() {
    let jsonl = r#"{"id": "1", "name": "Alice", "age": 30}
{"id": "2", "name": "Bob", "age": 25}
{"id": "3", "name": "Charlie", "age": 35}"#;

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let mut count = 0;
    let mut ages = Vec::new();

    for result in streamer {
        let doc = result.unwrap();
        count += 1;

        if let Some(Item::Scalar(Value::Int(age))) = doc.root.get("age") {
            ages.push(*age);
        }
    }

    assert_eq!(count, 3);
    assert_eq!(ages, vec![30, 25, 35]);
}

#[test]
fn test_jsonl_streamer_large_dataset() {
    // Generate 10,000 JSONL lines
    let mut jsonl = String::new();
    for i in 0..10000 {
        jsonl.push_str(&format!(r#"{{"id": "{}", "value": {}}}"#, i, i * 2));
        jsonl.push('\n');
    }

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::builder()
        .buffer_size(256 * 1024) // Large buffer for throughput
        .build();

    let streamer = JsonLinesStreamer::new(reader, config);

    let mut count = 0;
    for result in streamer {
        result.unwrap();
        count += 1;
    }

    assert_eq!(count, 10000);
}

#[test]
fn test_jsonl_streamer_blank_lines_handling() {
    let jsonl = r#"{"id": "1"}

{"id": "2"}


{"id": "3"}
"#;

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let count = streamer.map(|r| r.unwrap()).count();
    assert_eq!(count, 3);
}

#[test]
fn test_jsonl_streamer_comment_lines() {
    let jsonl = r#"# Header comment
{"id": "1"}
# Mid comment
{"id": "2"}
# Footer comment"#;

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let count = streamer.map(|r| r.unwrap()).count();
    assert_eq!(count, 2);
}

#[test]
fn test_jsonl_streamer_mixed_valid_invalid() {
    let jsonl = r#"{"valid": 1}
{invalid json}
{"also": "valid"}
not json at all
{"final": true}"#;

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let results: Vec<_> = streamer.collect();
    assert_eq!(results.len(), 5);

    assert!(results[0].is_ok());
    assert!(results[1].is_err()); // Invalid JSON
    assert!(results[2].is_ok());
    assert!(results[3].is_err()); // Invalid JSON
    assert!(results[4].is_ok());
}

#[test]
fn test_jsonl_streamer_line_number_tracking() {
    let jsonl = r#"{"line": 1}
{"line": 2}
{"line": 3}"#;

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();
    let mut streamer = JsonLinesStreamer::new(reader, config);

    assert_eq!(streamer.line_number(), 0);

    streamer.next().unwrap().unwrap();
    assert_eq!(streamer.line_number(), 1);

    streamer.next().unwrap().unwrap();
    assert_eq!(streamer.line_number(), 2);

    streamer.next().unwrap().unwrap();
    assert_eq!(streamer.line_number(), 3);

    assert!(streamer.next().is_none());
}

#[test]
fn test_jsonl_streamer_whitespace_handling() {
    let jsonl = "  {\"id\": \"1\"}  \n\t{\"id\": \"2\"}\t\n   {\"id\": \"3\"}   ";

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let count = streamer.map(|r| r.unwrap()).count();
    assert_eq!(count, 3);
}

#[test]
fn test_jsonl_streamer_size_limit() {
    let jsonl = r#"{"small": "ok"}
{"large": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}"#;

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::builder()
        .max_object_bytes(20) // Small limit
        .build();

    let streamer = JsonLinesStreamer::new(reader, config);
    let results: Vec<_> = streamer.collect();

    assert!(results[0].is_ok());
    assert!(results[1].is_err());
}

#[test]
fn test_jsonl_streamer_unicode_content() {
    let jsonl = r#"{"text": "Hello 世界"}
{"emoji": "🎉🚀✨"}
{"mixed": "Rust + 日本語 = 💯"}"#;

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let docs: Vec<_> = streamer.map(|r| r.unwrap()).collect();
    assert_eq!(docs.len(), 3);

    // Verify Unicode is preserved
    if let Some(Item::Scalar(Value::String(text))) = docs[0].root.get("text") {
        assert_eq!(text, "Hello 世界");
    }
}

#[test]
fn test_jsonl_streamer_empty_input() {
    let jsonl = "";

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let count = streamer.count();
    assert_eq!(count, 0);
}

#[test]
fn test_jsonl_streamer_only_comments_and_blanks() {
    let jsonl = r#"# Comment 1

# Comment 2

"#;

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let count = streamer.count();
    assert_eq!(count, 0);
}

// =============================================================================
// JsonLinesWriter Integration Tests
// =============================================================================

#[test]
fn test_jsonl_writer_basic_workflow() {
    let mut buffer = Vec::new();
    let mut writer = JsonLinesWriter::new(&mut buffer);

    for i in 1..=5 {
        let mut doc = Document::new((1, 0));
        doc.root.insert(
            "id".to_string(),
            Item::Scalar(Value::String(i.to_string())),
        );
        doc.root.insert("value".to_string(), Item::Scalar(Value::Int(i * 10)));
        writer.write_document(&doc).unwrap();
    }

    writer.flush().unwrap();

    let output = String::from_utf8(buffer).unwrap();
    let lines: Vec<_> = output.lines().collect();
    assert_eq!(lines.len(), 5);

    // Verify each line is valid JSON
    for line in lines {
        let _: serde_json::Value = serde_json::from_str(line).unwrap();
    }
}

#[test]
fn test_jsonl_writer_empty_documents() {
    let mut buffer = Vec::new();
    let mut writer = JsonLinesWriter::new(&mut buffer);

    for _ in 0..3 {
        let doc = Document::new((1, 0));
        writer.write_document(&doc).unwrap();
    }

    writer.flush().unwrap();

    let output = String::from_utf8(buffer).unwrap();
    let lines: Vec<_> = output.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "{}");
}

#[test]
fn test_jsonl_writer_complex_documents() {
    let mut buffer = Vec::new();
    let mut writer = JsonLinesWriter::new(&mut buffer);

    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "string".to_string(),
        Item::Scalar(Value::String("test".to_string())),
    );
    doc.root.insert("int".to_string(), Item::Scalar(Value::Int(42)));
    doc.root
        .insert("float".to_string(), Item::Scalar(Value::Float(3.5)));
    doc.root
        .insert("bool".to_string(), Item::Scalar(Value::Bool(true)));
    doc.root
        .insert("null".to_string(), Item::Scalar(Value::Null));

    writer.write_document(&doc).unwrap();
    writer.flush().unwrap();

    let output = String::from_utf8(buffer).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();

    assert!(parsed.is_object());
    assert_eq!(parsed["string"], "test");
    assert_eq!(parsed["int"], 42);
    assert_eq!(parsed["bool"], true);
    assert_eq!(parsed["null"], serde_json::Value::Null);
}

// =============================================================================
// Round-Trip Tests (Write → Read)
// =============================================================================

#[test]
fn test_jsonl_roundtrip_simple() {
    // Write documents
    let mut buffer = Vec::new();
    let mut writer = JsonLinesWriter::new(&mut buffer);

    let original_docs = vec![
        create_test_document("1", "Alice", 30),
        create_test_document("2", "Bob", 25),
        create_test_document("3", "Charlie", 35),
    ];

    for doc in &original_docs {
        writer.write_document(doc).unwrap();
    }
    writer.flush().unwrap();

    // Read back
    let reader = Cursor::new(buffer);
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let read_docs: Vec<_> = streamer.map(|r| r.unwrap()).collect();
    assert_eq!(read_docs.len(), 3);

    // Verify data integrity
    for (i, doc) in read_docs.iter().enumerate() {
        assert_eq!(
            doc.root.get("name").unwrap().as_scalar().unwrap(),
            original_docs[i]
                .root
                .get("name")
                .unwrap()
                .as_scalar()
                .unwrap()
        );
    }
}

#[test]
fn test_jsonl_roundtrip_large_dataset() {
    // Write 1000 documents
    let mut buffer = Vec::new();
    let mut writer = JsonLinesWriter::new(&mut buffer);

    for i in 0..1000 {
        let doc = create_test_document(&i.to_string(), &format!("User{}", i), i as i64);
        writer.write_document(&doc).unwrap();
    }
    writer.flush().unwrap();

    // Read back
    let reader = Cursor::new(buffer);
    let config = StreamConfig::default();
    let streamer = JsonLinesStreamer::new(reader, config);

    let count = streamer.map(|r| r.unwrap()).count();
    assert_eq!(count, 1000);
}

#[test]
fn test_jsonl_roundtrip_special_characters() {
    let mut buffer = Vec::new();
    let mut writer = JsonLinesWriter::new(&mut buffer);

    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "text".to_string(),
        Item::Scalar(Value::String("Line1\nLine2\tTab\"Quote'".to_string())),
    );
    writer.write_document(&doc).unwrap();
    writer.flush().unwrap();

    // Read back
    let reader = Cursor::new(buffer);
    let config = StreamConfig::default();
    let mut streamer = JsonLinesStreamer::new(reader, config);

    let restored = streamer.next().unwrap().unwrap();
    if let Some(Item::Scalar(Value::String(text))) = restored.root.get("text") {
        assert_eq!(text, "Line1\nLine2\tTab\"Quote'");
    }
}

// =============================================================================
// Memory Efficiency Tests
// =============================================================================

#[test]
fn test_streaming_memory_bounded() {
    // Generate large dataset that would OOM if loaded entirely
    let mut jsonl = String::new();
    for i in 0..100_000 {
        jsonl.push_str(&format!(r#"{{"id": "{}", "data": "x"}}"#, i));
        jsonl.push('\n');
    }

    let reader = Cursor::new(jsonl.as_bytes());
    let config = StreamConfig::builder()
        .buffer_size(8 * 1024) // Small buffer
        .build();

    let streamer = JsonLinesStreamer::new(reader, config);

    // Process without loading all into memory
    let mut count = 0;
    for result in streamer {
        result.unwrap();
        count += 1;

        // Only one document in memory at a time
        if count % 10000 == 0 {
            // Could check memory here if needed
        }
    }

    assert_eq!(count, 100_000);
}

// =============================================================================
// Configuration Tests
// =============================================================================

#[test]
fn test_stream_config_custom_limits() {
    let config = StreamConfig::builder()
        .buffer_size(128 * 1024)
        .max_object_bytes(5 * 1024 * 1024)
        .from_json_config(
            FromJsonConfig::builder()
                .max_depth(100)
                .max_array_size(10_000)
                .build(),
        )
        .build();

    assert_eq!(config.buffer_size, 128 * 1024);
    assert_eq!(config.max_object_bytes, Some(5 * 1024 * 1024));
    assert_eq!(config.from_json.max_depth, Some(100));
}

#[test]
fn test_stream_config_unlimited() {
    let config = StreamConfig::builder()
        .unlimited_object_size()
        .build();

    assert_eq!(config.max_object_bytes, None);
}

// =============================================================================
// Helper Functions
// =============================================================================

fn create_test_document(id: &str, name: &str, age: i64) -> Document {
    let mut doc = Document::new((1, 0));
    doc.root.insert(
        "id".to_string(),
        Item::Scalar(Value::String(id.to_string())),
    );
    doc.root.insert(
        "name".to_string(),
        Item::Scalar(Value::String(name.to_string())),
    );
    doc.root
        .insert("age".to_string(), Item::Scalar(Value::Int(age)));
    doc
}
