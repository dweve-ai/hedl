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

//! Integration tests for async API
//!
//! These tests verify the async API works correctly with real file I/O
//! and concurrent operations.

#![cfg(feature = "async")]

use hedl_core::{Document, Item, Value};
use hedl_xml::async_api::*;
use hedl_xml::{FromXmlConfig, ToXmlConfig};
use std::collections::BTreeMap;
use tempfile::TempDir;
use tokio::fs;

// ============================================================================
// Helper Functions
// ============================================================================

async fn create_test_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).await.unwrap();
    path
}

fn create_test_document() -> Document {
    let mut doc = Document::new((1, 0));
    doc.root
        .insert("name".to_string(), Item::Scalar(Value::String("test".to_string())));
    doc.root.insert("value".to_string(), Item::Scalar(Value::Int(42)));
    doc.root
        .insert("active".to_string(), Item::Scalar(Value::Bool(true)));
    doc
}

// ============================================================================
// File I/O Tests
// ============================================================================

#[tokio::test]
async fn test_async_file_round_trip() {
    let dir = TempDir::new().unwrap();
    let doc = create_test_document();

    let write_path = dir.path().join("test.xml");
    let config_to = ToXmlConfig::default();

    // Write async
    to_xml_file_async(&doc, &write_path, &config_to)
        .await
        .unwrap();

    // Read async
    let config_from = FromXmlConfig::default();
    let doc2 = from_xml_file_async(&write_path, &config_from)
        .await
        .unwrap();

    // Verify
    assert_eq!(
        doc2.root.get("name").and_then(|i| i.as_scalar()),
        Some(&Value::String("test".to_string()))
    );
    assert_eq!(
        doc2.root.get("value").and_then(|i| i.as_scalar()),
        Some(&Value::Int(42))
    );
    assert_eq!(
        doc2.root.get("active").and_then(|i| i.as_scalar()),
        Some(&Value::Bool(true))
    );
}

#[tokio::test]
async fn test_async_reader_writer() {
    let doc = create_test_document();

    // Write to buffer
    let mut buffer = Vec::new();
    let config_to = ToXmlConfig::default();
    to_xml_writer_async(&doc, &mut buffer, &config_to)
        .await
        .unwrap();

    // Read from buffer
    let cursor = tokio::io::Cursor::new(&buffer);
    let config_from = FromXmlConfig::default();
    let doc2 = from_xml_reader_async(cursor, &config_from).await.unwrap();

    // Verify
    assert_eq!(
        doc2.root.get("name").and_then(|i| i.as_scalar()),
        Some(&Value::String("test".to_string()))
    );
}

#[tokio::test]
async fn test_async_string_parsing() {
    let xml = r#"<?xml version="1.0"?><hedl><id>123</id><name>Alice</name></hedl>"#;
    let config = FromXmlConfig::default();

    let doc = from_xml_async(xml, &config).await.unwrap();

    assert_eq!(
        doc.root.get("id").and_then(|i| i.as_scalar()),
        Some(&Value::Int(123))
    );
    assert_eq!(
        doc.root.get("name").and_then(|i| i.as_scalar()),
        Some(&Value::String("Alice".to_string()))
    );
}

#[tokio::test]
async fn test_async_string_generation() {
    let mut doc = Document::new((1, 0));
    doc.root.insert("test".to_string(), Item::Scalar(Value::Int(999)));

    let config = ToXmlConfig::default();
    let xml = to_xml_async(&doc, &config).await.unwrap();

    assert!(xml.contains("<test>999</test>"));
}

// ============================================================================
// Concurrent Processing Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_file_reads() {
    let dir = TempDir::new().unwrap();

    // Create test files
    let xml1 = r#"<?xml version="1.0"?><hedl><id>1</id></hedl>"#;
    let xml2 = r#"<?xml version="1.0"?><hedl><id>2</id></hedl>"#;
    let xml3 = r#"<?xml version="1.0"?><hedl><id>3</id></hedl>"#;

    let path1 = create_test_file(&dir, "file1.xml", xml1).await;
    let path2 = create_test_file(&dir, "file2.xml", xml2).await;
    let path3 = create_test_file(&dir, "file3.xml", xml3).await;

    // Process concurrently
    let paths = vec![path1, path2, path3];
    let config = FromXmlConfig::default();
    let results = from_xml_files_concurrent(&paths, &config, 3).await;

    // Verify all succeeded
    assert_eq!(results.len(), 3);
    for result in &results {
        assert!(result.is_ok());
    }

    // Verify IDs
    assert_eq!(
        results[0]
            .as_ref()
            .unwrap()
            .root
            .get("id")
            .and_then(|i| i.as_scalar()),
        Some(&Value::Int(1))
    );
    assert_eq!(
        results[1]
            .as_ref()
            .unwrap()
            .root
            .get("id")
            .and_then(|i| i.as_scalar()),
        Some(&Value::Int(2))
    );
    assert_eq!(
        results[2]
            .as_ref()
            .unwrap()
            .root
            .get("id")
            .and_then(|i| i.as_scalar()),
        Some(&Value::Int(3))
    );
}

#[tokio::test]
async fn test_concurrent_file_writes() {
    let dir = TempDir::new().unwrap();

    // Create documents
    let mut doc1 = Document::new((1, 0));
    doc1.root.insert("id".to_string(), Item::Scalar(Value::Int(1)));

    let mut doc2 = Document::new((1, 0));
    doc2.root.insert("id".to_string(), Item::Scalar(Value::Int(2)));

    // Write concurrently
    let path1 = dir.path().join("out1.xml");
    let path2 = dir.path().join("out2.xml");

    let docs_and_paths = vec![(&doc1, &path1), (&doc2, &path2)];
    let config = ToXmlConfig::default();
    let results = to_xml_files_concurrent(docs_and_paths, &config, 2).await;

    // Verify all succeeded
    assert_eq!(results.len(), 2);
    for result in &results {
        assert!(result.is_ok());
    }

    // Verify files exist and contain correct data
    let content1 = fs::read_to_string(path1).await.unwrap();
    let content2 = fs::read_to_string(path2).await.unwrap();

    assert!(content1.contains("<id>1</id>"));
    assert!(content2.contains("<id>2</id>"));
}

#[tokio::test]
async fn test_concurrent_parsing() {
    let xml1 = r#"<?xml version="1.0"?><hedl><val>a</val></hedl>"#;
    let xml2 = r#"<?xml version="1.0"?><hedl><val>b</val></hedl>"#;
    let xml3 = r#"<?xml version="1.0"?><hedl><val>c</val></hedl>"#;

    let config = FromXmlConfig::default();

    // Parse concurrently
    let (r1, r2, r3) = tokio::join!(
        from_xml_async(xml1, &config),
        from_xml_async(xml2, &config),
        from_xml_async(xml3, &config)
    );

    // Verify all succeeded
    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(r3.is_ok());

    // Verify values
    assert_eq!(
        r1.unwrap().root.get("val").and_then(|i| i.as_scalar()),
        Some(&Value::String("a".to_string()))
    );
    assert_eq!(
        r2.unwrap().root.get("val").and_then(|i| i.as_scalar()),
        Some(&Value::String("b".to_string()))
    );
    assert_eq!(
        r3.unwrap().root.get("val").and_then(|i| i.as_scalar()),
        Some(&Value::String("c".to_string()))
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_async_file_not_found() {
    let config = FromXmlConfig::default();
    let result = from_xml_file_async("/nonexistent/file.xml", &config).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read file"));
}

#[tokio::test]
async fn test_async_invalid_xml() {
    let xml = "<invalid>xml<";
    let config = FromXmlConfig::default();

    let result = from_xml_async(xml, &config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_write_invalid_path() {
    let doc = Document::new((1, 0));
    let config = ToXmlConfig::default();

    let result = to_xml_file_async(&doc, "/invalid/\0/path.xml", &config).await;
    assert!(result.is_err());
}

// ============================================================================
// Edge Cases and Large Data Tests
// ============================================================================

#[tokio::test]
async fn test_async_large_string() {
    let large_content = "x".repeat(100_000);
    let xml = format!(
        r#"<?xml version="1.0"?><hedl><data>{}</data></hedl>"#,
        large_content
    );

    let config = FromXmlConfig::default();
    let doc = from_xml_async(&xml, &config).await.unwrap();

    assert_eq!(
        doc.root.get("data").and_then(|i| i.as_scalar()),
        Some(&Value::String(large_content))
    );
}

#[tokio::test]
async fn test_async_unicode() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <hedl><text>Hello 世界 🌍 héllo</text></hedl>"#;

    let config = FromXmlConfig::default();
    let doc = from_xml_async(xml, &config).await.unwrap();

    assert_eq!(
        doc.root.get("text").and_then(|i| i.as_scalar()),
        Some(&Value::String("Hello 世界 🌍 héllo".to_string()))
    );
}

#[tokio::test]
async fn test_async_empty_document() {
    let doc = Document::new((1, 0));
    let config = ToXmlConfig::default();

    let xml = to_xml_async(&doc, &config).await.unwrap();
    assert!(xml.contains("<?xml"));
    assert!(xml.contains("<hedl"));

    // Parse it back
    let config_from = FromXmlConfig::default();
    let doc2 = from_xml_async(&xml, &config_from).await.unwrap();
    assert!(doc2.root.is_empty());
}

// ============================================================================
// Performance and Stress Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_large_batch() {
    let dir = TempDir::new().unwrap();

    // Create 20 small files
    let mut paths = Vec::new();
    for i in 0..20 {
        let xml = format!(r#"<?xml version="1.0"?><hedl><id>{}</id></hedl>"#, i);
        let path = create_test_file(&dir, &format!("file{}.xml", i), &xml).await;
        paths.push(path);
    }

    // Process with limited concurrency
    let config = FromXmlConfig::default();
    let results = from_xml_files_concurrent(&paths, &config, 4).await;

    // Verify all succeeded
    assert_eq!(results.len(), 20);
    for result in &results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_concurrent_writes_batch() {
    let dir = TempDir::new().unwrap();

    // Create 15 documents
    let mut docs = Vec::new();
    let mut paths = Vec::new();

    for i in 0..15 {
        let mut doc = Document::new((1, 0));
        doc.root.insert("id".to_string(), Item::Scalar(Value::Int(i)));
        docs.push(doc);
        paths.push(dir.path().join(format!("out{}.xml", i)));
    }

    // Write concurrently with limited concurrency
    let docs_and_paths: Vec<_> = docs.iter().zip(paths.iter()).collect();
    let config = ToXmlConfig::default();
    let results = to_xml_files_concurrent(docs_and_paths, &config, 5).await;

    // Verify all succeeded
    assert_eq!(results.len(), 15);
    for result in &results {
        assert!(result.is_ok());
    }

    // Verify all files exist
    for path in paths {
        assert!(fs::metadata(path).await.is_ok());
    }
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[tokio::test]
async fn test_async_custom_config() {
    let mut doc = Document::new((2, 5));
    doc.root.insert("test".to_string(), Item::Scalar(Value::Int(123)));

    let config = ToXmlConfig {
        pretty: true,
        indent: "  ".to_string(),
        root_element: "custom".to_string(),
        include_metadata: true,
        use_attributes: false,
    };

    let xml = to_xml_async(&doc, &config).await.unwrap();

    assert!(xml.contains("<custom"));
    assert!(xml.contains("version=\"2.5\""));
    assert!(xml.contains("<test>123</test>"));
}

#[tokio::test]
async fn test_async_compact_output() {
    let mut doc = Document::new((1, 0));
    doc.root.insert("val".to_string(), Item::Scalar(Value::Int(42)));

    let config_pretty = ToXmlConfig {
        pretty: true,
        ..Default::default()
    };
    let config_compact = ToXmlConfig {
        pretty: false,
        ..Default::default()
    };

    let xml_pretty = to_xml_async(&doc, &config_pretty).await.unwrap();
    let xml_compact = to_xml_async(&doc, &config_compact).await.unwrap();

    // Compact should be smaller
    assert!(xml_compact.len() < xml_pretty.len());
}
