#![cfg(feature = "async-io")]

use hedl_core::{Document, Item};
use hedl_parquet::async_io::{
    from_parquet_async, from_parquet_bytes_async, to_parquet_async, to_parquet_bytes_async,
};
use tempfile::TempDir;

/// Generate test dataset
fn generate_dataset(num_rows: usize) -> hedl_core::Document {
    let mut matrix_list = hedl_core::MatrixList::new(
        "TestData",
        vec!["id".to_string(), "name".to_string(), "value".to_string()],
    );

    for i in 0..num_rows {
        matrix_list.add_row(hedl_core::Node::new(
            "TestData",
            format!("row{}", i),
            vec![
                hedl_core::Value::String(format!("row{}", i).into()),
                hedl_core::Value::String(format!("name_{}", i).into()),
                hedl_core::Value::Int(i as i64),
            ],
        ));
    }

    let mut doc = hedl_core::Document::new((1, 0));
    doc.root
        .insert("data".to_string(), hedl_core::Item::List(matrix_list));
    doc
}

#[tokio::test]
async fn test_async_round_trip_file() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_async.parquet");

    // Create document
    let doc = generate_dataset(100);

    // Write asynchronously
    to_parquet_async(&doc, &file_path).await.unwrap();

    // Read asynchronously
    let restored = from_parquet_async(&file_path).await.unwrap();

    // Verify
    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 100);
    } else {
        panic!("Expected list");
    }
}

#[tokio::test]
async fn test_async_round_trip_bytes() {
    let doc = generate_dataset(50);

    // Convert to bytes asynchronously
    let bytes = to_parquet_bytes_async(&doc).await.unwrap();

    // Read from bytes asynchronously
    let restored = from_parquet_bytes_async(&bytes).await.unwrap();

    // Verify
    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 50);
    } else {
        panic!("Expected list");
    }
}

#[tokio::test]
async fn test_async_concurrent_reads() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple test files
    let mut paths = Vec::new();
    for i in 0..10 {
        let path = temp_dir.path().join(format!("test_{}.parquet", i));
        let doc = generate_dataset(100);
        to_parquet_async(&doc, &path).await.unwrap();
        paths.push(path);
    }

    // Read all files concurrently
    let mut tasks = Vec::new();
    for path in paths {
        let task = tokio::spawn(async move { from_parquet_async(&path).await.unwrap() });
        tasks.push(task);
    }

    // Wait for all reads to complete
    let results = futures::future::join_all(tasks).await;

    // Verify all reads succeeded
    assert_eq!(results.len(), 10);
    for result in results {
        let doc = result.unwrap();
        if let Some(Item::List(list)) = doc.root.get("data") {
            assert_eq!(list.rows.len(), 100);
        } else {
            panic!("Expected list");
        }
    }
}

#[tokio::test]
async fn test_async_concurrent_writes() {
    let temp_dir = TempDir::new().unwrap();

    // Create multiple documents
    let docs: Vec<_> = (0..10).map(|_| generate_dataset(100)).collect();

    // Write all documents concurrently
    let mut tasks = Vec::new();
    for (i, doc) in docs.iter().enumerate() {
        let path = temp_dir.path().join(format!("test_{}.parquet", i));
        let doc_clone = doc.clone(); // Clone for move into async block
        let task = tokio::spawn(async move {
            to_parquet_async(&doc_clone, &path).await.unwrap();
            path
        });
        tasks.push(task);
    }

    // Wait for all writes to complete
    let paths = futures::future::join_all(tasks).await;

    // Verify all files were written
    assert_eq!(paths.len(), 10);
    for result in paths {
        let path = result.unwrap();
        assert!(path.exists());
    }
}

#[tokio::test]
async fn test_async_large_dataset() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("large_async.parquet");

    // Create large dataset (1M rows)
    let doc = generate_dataset(1_000_000);

    // Write asynchronously
    to_parquet_async(&doc, &file_path).await.unwrap();

    // Read asynchronously
    let restored = from_parquet_async(&file_path).await.unwrap();

    // Verify
    if let Some(Item::List(list)) = restored.root.get("data") {
        assert_eq!(list.rows.len(), 1_000_000);
    } else {
        panic!("Expected list");
    }
}

#[tokio::test]
async fn test_async_mixed_operations() {
    let temp_dir = TempDir::new().unwrap();

    // Create test files
    let mut write_tasks = Vec::new();
    for i in 0..5 {
        let path = temp_dir.path().join(format!("test_{}.parquet", i));
        let doc = generate_dataset(100);
        let task = tokio::spawn(async move {
            to_parquet_async(&doc, &path).await.unwrap();
            path
        });
        write_tasks.push(task);
    }

    // Wait for writes
    let paths = futures::future::join_all(write_tasks).await;
    let paths: Vec<_> = paths.into_iter().map(|r| r.unwrap()).collect();

    // Concurrent reads
    let mut read_tasks = Vec::new();
    for path in paths {
        let task = tokio::spawn(async move { from_parquet_async(&path).await.unwrap() });
        read_tasks.push(task);
    }

    // Wait for reads
    let results = futures::future::join_all(read_tasks).await;

    // Verify
    assert_eq!(results.len(), 5);
    for result in results {
        let doc = result.unwrap();
        if let Some(Item::List(list)) = doc.root.get("data") {
            assert_eq!(list.rows.len(), 100);
        } else {
            panic!("Expected list");
        }
    }
}

#[tokio::test]
async fn test_async_error_handling() {
    use std::path::PathBuf;

    // Test reading non-existent file
    let result = from_parquet_async(PathBuf::from("/nonexistent/file.parquet").as_path()).await;
    assert!(result.is_err());

    // Test writing to invalid path
    let doc = generate_dataset(10);
    let result =
        to_parquet_async(&doc, PathBuf::from("/invalid/path/file.parquet").as_path()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_async_empty_document() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty_async.parquet");

    let doc = Document::new((1, 0));

    // Empty documents should succeed
    let result = to_parquet_async(&doc, &file_path).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_concurrent_read_same_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("shared.parquet");

    // Create test file
    let doc = generate_dataset(1000);
    to_parquet_async(&doc, &file_path).await.unwrap();

    // Read same file concurrently from multiple tasks
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let path = file_path.clone();
        let task = tokio::spawn(async move { from_parquet_async(&path).await.unwrap() });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    // Verify all reads succeeded
    assert_eq!(results.len(), 10);
    for result in results {
        let doc = result.unwrap();
        if let Some(Item::List(list)) = doc.root.get("data") {
            assert_eq!(list.rows.len(), 1000);
        } else {
            panic!("Expected list");
        }
    }
}
