//! Comprehensive async API tests for hedl-xml
//!
//! Tests async file I/O, reader/writer operations, and streaming.

#![cfg(feature = "async")]

use hedl_core::{Document, Item, Value};
use hedl_xml::async_api::{
    from_xml_file_async, from_xml_reader_async, from_xml_stream_async, to_xml_file_async,
    to_xml_writer_async,
};
use hedl_xml::streaming::StreamConfig;
use hedl_xml::{FromXmlConfig, ToXmlConfig};
use tempfile::TempDir;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufReader};

// ==================== Async file I/O tests ====================

#[tokio::test]
async fn test_async_file_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.xml");
    let output_path = temp_dir.path().join("output.xml");

    // Create test document
    let mut doc = Document::new((2, 0));
    doc.root
        .insert("test".to_string(), Item::Scalar(Value::Int(42)));

    // Write async
    to_xml_file_async(&doc, &input_path, &ToXmlConfig::default())
        .await
        .unwrap();

    // Read async
    let doc2 = from_xml_file_async(&input_path, &FromXmlConfig::default())
        .await
        .unwrap();

    assert_eq!(
        doc2.root.get("test").and_then(|i| i.as_scalar()),
        Some(&Value::Int(42))
    );

    // Write again
    to_xml_file_async(&doc2, &output_path, &ToXmlConfig::default())
        .await
        .unwrap();

    // Verify output exists
    assert!(output_path.exists());
}

#[tokio::test]
async fn test_async_file_read_error() {
    let result = from_xml_file_async("/nonexistent/file.xml", &FromXmlConfig::default()).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Failed to read file"));
}

#[tokio::test]
async fn test_async_file_write_error() {
    let doc = Document::new((2, 0));
    let result = to_xml_file_async(&doc, "/invalid/path/output.xml", &ToXmlConfig::default()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_file_large_document() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("large.xml");

    // Create larger document
    let mut doc = Document::new((2, 0));
    for i in 0..1000 {
        doc.root.insert(
            format!("item_{}", i),
            Item::Scalar(Value::String(format!("value_{}", i).into())),
        );
    }

    // Write and read back
    to_xml_file_async(&doc, &path, &ToXmlConfig::default())
        .await
        .unwrap();
    let doc2 = from_xml_file_async(&path, &FromXmlConfig::default())
        .await
        .unwrap();

    assert_eq!(doc2.root.len(), 1000);
    assert!(doc2.root.contains_key("item_0"));
    assert!(doc2.root.contains_key("item_999"));
}

// ==================== Async reader/writer tests ====================

#[tokio::test]
async fn test_async_reader_from_bytes() {
    let xml = r#"<?xml version="1.0"?><hedl><value>42</value></hedl>"#;
    let bytes = xml.as_bytes();
    let reader = BufReader::new(bytes);

    let doc = from_xml_reader_async(reader, &FromXmlConfig::default())
        .await
        .unwrap();

    assert_eq!(
        doc.root.get("value").and_then(|i| i.as_scalar()),
        Some(&Value::Int(42))
    );
}

#[tokio::test]
async fn test_async_writer_to_vec() {
    let mut doc = Document::new((2, 0));
    doc.root.insert(
        "test".to_string(),
        Item::Scalar(Value::String("hello".to_string().into())),
    );

    let mut buffer = Vec::new();
    to_xml_writer_async(&doc, &mut buffer, &ToXmlConfig::default())
        .await
        .unwrap();

    let xml = String::from_utf8(buffer).unwrap();
    assert!(xml.contains("<?xml"));
    assert!(xml.contains("<test>hello</test>"));
}

#[tokio::test]
async fn test_async_reader_empty_input() {
    let xml = "";
    let reader = BufReader::new(xml.as_bytes());

    let doc = from_xml_reader_async(reader, &FromXmlConfig::default())
        .await
        .unwrap();

    assert!(doc.root.is_empty());
}

#[tokio::test]
async fn test_async_reader_invalid_xml() {
    let xml = r#"<unclosed><tag>value</unclosed>"#;
    let reader = BufReader::new(xml.as_bytes());

    let result = from_xml_reader_async(reader, &FromXmlConfig::default()).await;
    assert!(result.is_err());
}

// ==================== Async streaming tests ====================

#[tokio::test]
async fn test_async_stream_basic() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item1>value1</item1>
        <item2>value2</item2>
        <item3>value3</item3>
    </hedl>"#;

    let reader = BufReader::new(xml.as_bytes());
    let config = StreamConfig::default();

    let mut stream = from_xml_stream_async(reader, &config).await.unwrap();
    let mut count = 0;

    while let Some(result) = stream.next().await {
        let item = result.unwrap();
        assert!(item.key.starts_with("item"));
        count += 1;
    }

    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_async_stream_empty_document() {
    let xml = r#"<?xml version="1.0"?><hedl></hedl>"#;
    let reader = BufReader::new(xml.as_bytes());
    let config = StreamConfig::default();

    let mut stream = from_xml_stream_async(reader, &config).await.unwrap();
    let mut count = 0;

    while stream.next().await.is_some() {
        count += 1;
    }

    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_async_stream_with_large_items() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("stream.xml");

    // Create file with many items
    let mut file = tokio::fs::File::create(&path).await.unwrap();
    file.write_all(b"<?xml version=\"1.0\"?><hedl>")
        .await
        .unwrap();
    for i in 0..100 {
        let item = format!("<item{}>value{}</item{}>", i, i, i);
        file.write_all(item.as_bytes()).await.unwrap();
    }
    file.write_all(b"</hedl>").await.unwrap();
    file.flush().await.unwrap();

    // Stream it
    let file = File::open(&path).await.unwrap();
    let reader = BufReader::new(file);
    let config = StreamConfig::default();

    let mut stream = from_xml_stream_async(reader, &config).await.unwrap();
    let mut count = 0;

    while let Some(result) = stream.next().await {
        result.unwrap();
        count += 1;
    }

    assert_eq!(count, 100);
}

#[tokio::test]
async fn test_async_stream_error_in_middle() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item1>value1</item1>
        <item2>unclosed
        <item3>value3</item3>
    </hedl>"#;

    let reader = BufReader::new(xml.as_bytes());
    let config = StreamConfig::default();

    let mut stream = from_xml_stream_async(reader, &config).await.unwrap();

    while let Some(result) = stream.next().await {
        if result.is_err() {
            // Error found, but we just want to ensure no panic
            break;
        }
    }

    // May or may not error depending on how parser handles it
    // At minimum, we shouldn't panic
}

#[tokio::test]
async fn test_async_stream_custom_config() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item>value</item>
    </hedl>"#;

    let reader = BufReader::new(xml.as_bytes());
    let config = StreamConfig {
        buffer_size: 128,
        max_recursion_depth: 50,
        max_batch_size: 10,
        ..Default::default()
    };

    let mut stream = from_xml_stream_async(reader, &config).await.unwrap();

    if let Some(result) = stream.next().await {
        let item = result.unwrap();
        assert_eq!(item.key, "item");
    }
}

#[tokio::test]
async fn test_async_stream_position_tracking() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item1>value1</item1>
        <item2>value2</item2>
    </hedl>"#;

    let reader = BufReader::new(xml.as_bytes());
    let config = StreamConfig::default();

    let mut stream = from_xml_stream_async(reader, &config).await.unwrap();

    let pos_start = stream.position();
    assert_eq!(pos_start.items_parsed, 0);

    while let Some(result) = stream.next().await {
        result.unwrap();
    }

    let pos_end = stream.position();
    assert_eq!(pos_end.items_parsed, 2);
    assert!(pos_end.byte_offset > 0);
}

// ==================== Concurrent async operations ====================

#[tokio::test]
async fn test_concurrent_file_reads() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test files
    let mut paths = Vec::new();
    for i in 0..5 {
        let path = temp_dir.path().join(format!("file_{}.xml", i));
        let mut doc = Document::new((2, 0));
        doc.root
            .insert("value".to_string(), Item::Scalar(Value::Int(i as i64)));
        to_xml_file_async(&doc, &path, &ToXmlConfig::default())
            .await
            .unwrap();
        paths.push(path);
    }

    // Read concurrently
    let mut handles = Vec::new();
    for (i, path) in paths.into_iter().enumerate() {
        let handle = tokio::spawn(async move {
            let doc = from_xml_file_async(&path, &FromXmlConfig::default())
                .await
                .unwrap();
            let value = doc
                .root
                .get("value")
                .and_then(|item| item.as_scalar())
                .cloned()
                .unwrap();
            (i, value)
        });
        handles.push(handle);
    }

    // Collect results
    for handle in handles {
        let (i, value) = handle.await.unwrap();
        assert_eq!(value, Value::Int(i as i64));
    }
}

#[tokio::test]
async fn test_concurrent_file_writes() {
    let temp_dir = TempDir::new().unwrap();

    let mut handles = Vec::new();
    for i in 0..5 {
        let path = temp_dir.path().join(format!("output_{}.xml", i));
        let handle = tokio::spawn(async move {
            let mut doc = Document::new((2, 0));
            doc.root
                .insert("id".to_string(), Item::Scalar(Value::Int(i)));
            to_xml_file_async(&doc, &path, &ToXmlConfig::default())
                .await
                .unwrap();
            path
        });
        handles.push(handle);
    }

    // Wait for all writes
    for handle in handles {
        let path = handle.await.unwrap();
        assert!(path.exists());
    }
}

// ==================== Edge cases ====================

#[tokio::test]
async fn test_async_very_long_string() {
    let long_value = "x".repeat(100000);
    let xml = format!(
        r#"<?xml version="1.0"?><hedl><data>{}</data></hedl>"#,
        long_value
    );
    let reader = BufReader::new(xml.as_bytes());

    let doc = from_xml_reader_async(reader, &FromXmlConfig::default())
        .await
        .unwrap();

    if let Some(Item::Scalar(Value::String(s))) = doc.root.get("data") {
        assert_eq!(s.len(), 100000);
    } else {
        panic!("Expected long string");
    }
}

#[tokio::test]
async fn test_async_unicode_in_filenames() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("测试_тест_🚀.xml");

    let mut doc = Document::new((2, 0));
    doc.root
        .insert("test".to_string(), Item::Scalar(Value::Int(42)));

    to_xml_file_async(&doc, &path, &ToXmlConfig::default())
        .await
        .unwrap();

    let doc2 = from_xml_file_async(&path, &FromXmlConfig::default())
        .await
        .unwrap();

    assert_eq!(
        doc2.root.get("test").and_then(|i| i.as_scalar()),
        Some(&Value::Int(42))
    );
}

#[tokio::test]
async fn test_async_reader_with_timeout() {
    let xml = r#"<?xml version="1.0"?><hedl><value>42</value></hedl>"#;
    let reader = BufReader::new(xml.as_bytes());

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        from_xml_reader_async(reader, &FromXmlConfig::default()),
    )
    .await;

    assert!(result.is_ok());
    let doc = result.unwrap().unwrap();
    assert_eq!(
        doc.root.get("value").and_then(|i| i.as_scalar()),
        Some(&Value::Int(42))
    );
}

#[tokio::test]
async fn test_async_stream_with_backpressure() {
    let xml = r#"<?xml version="1.0"?>
    <hedl>
        <item1>value1</item1>
        <item2>value2</item2>
        <item3>value3</item3>
    </hedl>"#;

    let reader = BufReader::new(xml.as_bytes());
    let config = StreamConfig {
        max_batch_size: 1,
        ..Default::default()
    };

    let mut stream = from_xml_stream_async(reader, &config).await.unwrap();
    let mut items = Vec::new();

    while let Some(result) = stream.next().await {
        let item = result.unwrap();
        // Simulate slow processing
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        items.push(item.key);
    }

    assert_eq!(items.len(), 3);
}
