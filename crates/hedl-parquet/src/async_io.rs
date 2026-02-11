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

//! Async I/O support for hedl-parquet.
//!
//! This module provides async variants of read and write operations
//! using the tokio async runtime. File I/O is performed asynchronously,
//! while CPU-bound Parquet parsing/writing is offloaded to blocking threads.
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(feature = "async-io")]
//! # async fn example() {
//! use hedl_parquet::async_io::from_parquet_async;
//! use std::path::Path;
//!
//! let doc = from_parquet_async(Path::new("input.parquet")).await.unwrap();
//! println!("Read {} entries", doc.root.len());
//! # }
//! ```

use std::path::Path;

use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use hedl_core::{Document, HedlError};

use crate::config::FromParquetConfig;
use crate::from_parquet::from_parquet_bytes_with_config;
use crate::to_parquet::{to_parquet_bytes_with_config, ToParquetConfig};

/// Read a HEDL document from a Parquet file asynchronously.
///
/// This function uses tokio for non-blocking file I/O, with CPU-bound
/// Parquet parsing offloaded to a blocking thread pool.
///
/// # Arguments
///
/// * `path` - Path to the Parquet file to read
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The Parquet format is invalid
/// - The data cannot be converted to HEDL
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "async-io")]
/// # async fn example() {
/// use hedl_parquet::async_io::from_parquet_async;
/// use std::path::Path;
///
/// let doc = from_parquet_async(Path::new("input.parquet")).await.unwrap();
/// # }
/// ```
pub async fn from_parquet_async(path: &Path) -> Result<Document, HedlError> {
    from_parquet_with_config_async(path, &FromParquetConfig::default()).await
}

/// Read a HEDL document from a Parquet file with custom configuration asynchronously.
///
/// # Arguments
///
/// * `path` - Path to the Parquet file to read
/// * `config` - Configuration for handling edge cases like null IDs
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "async-io")]
/// # async fn example() {
/// use hedl_parquet::async_io::from_parquet_with_config_async;
/// use hedl_parquet::FromParquetConfig;
/// use std::path::Path;
///
/// let config = FromParquetConfig::lenient();
/// let doc = from_parquet_with_config_async(Path::new("input.parquet"), &config).await.unwrap();
/// # }
/// ```
pub async fn from_parquet_with_config_async(
    path: &Path,
    config: &FromParquetConfig,
) -> Result<Document, HedlError> {
    // Open file asynchronously
    let mut file = File::open(path)
        .await
        .map_err(|e| HedlError::io(format!("Failed to open Parquet file: {e}")))?;

    // Read entire file into memory asynchronously
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .await
        .map_err(|e| HedlError::io(format!("Failed to read Parquet file: {e}")))?;

    // Parse in blocking thread pool (Parquet parsing is CPU-bound)
    from_parquet_bytes_with_config_async(&buffer, config).await
}

/// Read a HEDL document from Parquet bytes asynchronously.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "async-io")]
/// # async fn example() {
/// use hedl_parquet::async_io::from_parquet_bytes_async;
///
/// let bytes = vec![]; // Some Parquet bytes
/// if !bytes.is_empty() {
///     let doc = from_parquet_bytes_async(&bytes).await.unwrap();
/// }
/// # }
/// ```
pub async fn from_parquet_bytes_async(bytes: &[u8]) -> Result<Document, HedlError> {
    from_parquet_bytes_with_config_async(bytes, &FromParquetConfig::default()).await
}

/// Read a HEDL document from Parquet bytes with custom configuration asynchronously.
///
/// Uses `spawn_blocking` to offload CPU-bound Parquet parsing to the
/// blocking thread pool, avoiding blocking the async runtime.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "async-io")]
/// # async fn example() {
/// use hedl_parquet::async_io::from_parquet_bytes_with_config_async;
/// use hedl_parquet::FromParquetConfig;
///
/// let bytes = vec![]; // Some Parquet bytes
/// let config = FromParquetConfig::lenient();
/// if !bytes.is_empty() {
///     let doc = from_parquet_bytes_with_config_async(&bytes, &config).await.unwrap();
/// }
/// # }
/// ```
pub async fn from_parquet_bytes_with_config_async(
    bytes: &[u8],
    config: &FromParquetConfig,
) -> Result<Document, HedlError> {
    // Clone data for move into blocking task
    let bytes_owned = bytes.to_vec();
    let config_owned = config.clone();

    // Offload CPU-bound parsing to blocking thread pool
    tokio::task::spawn_blocking(move || from_parquet_bytes_with_config(&bytes_owned, &config_owned))
        .await
        .map_err(|e| HedlError::io(format!("Parquet parsing task failed: {e}")))?
}

/// Write a HEDL document to a Parquet file asynchronously.
///
/// # Arguments
///
/// * `doc` - The HEDL document to convert
/// * `path` - Path to the output Parquet file
///
/// # Errors
///
/// Returns an error if:
/// - The document contains unsupported structures
/// - Parquet writing fails
/// - I/O operations fail
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "async-io")]
/// # async fn example() {
/// use hedl_core::Document;
/// use hedl_parquet::async_io::to_parquet_async;
/// use std::path::Path;
///
/// let doc = Document::new((2, 0));
/// to_parquet_async(&doc, Path::new("output.parquet")).await.unwrap();
/// # }
/// ```
pub async fn to_parquet_async(doc: &Document, path: &Path) -> Result<(), HedlError> {
    to_parquet_with_config_async(doc, path, &ToParquetConfig::default()).await
}

/// Write a HEDL document to a Parquet file with custom configuration asynchronously.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "async-io")]
/// # async fn example() {
/// use hedl_core::Document;
/// use hedl_parquet::async_io::to_parquet_with_config_async;
/// use hedl_parquet::ToParquetConfig;
/// use std::path::Path;
///
/// let doc = Document::new((2, 0));
/// let config = ToParquetConfig::default();
/// to_parquet_with_config_async(&doc, Path::new("output.parquet"), &config).await.unwrap();
/// # }
/// ```
pub async fn to_parquet_with_config_async(
    doc: &Document,
    path: &Path,
    config: &ToParquetConfig,
) -> Result<(), HedlError> {
    // Generate bytes in blocking thread pool
    let bytes = to_parquet_bytes_with_config_async(doc, config).await?;

    // Write bytes asynchronously
    let mut file = File::create(path)
        .await
        .map_err(|e| HedlError::io(format!("Failed to create Parquet file: {e}")))?;

    file.write_all(&bytes)
        .await
        .map_err(|e| HedlError::io(format!("Failed to write Parquet file: {e}")))?;

    file.flush()
        .await
        .map_err(|e| HedlError::io(format!("Failed to flush Parquet file: {e}")))?;

    Ok(())
}

/// Convert a HEDL document to Parquet bytes asynchronously.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "async-io")]
/// # async fn example() {
/// use hedl_core::Document;
/// use hedl_parquet::async_io::to_parquet_bytes_async;
///
/// let doc = Document::new((2, 0));
/// let bytes = to_parquet_bytes_async(&doc).await.unwrap();
/// # }
/// ```
pub async fn to_parquet_bytes_async(doc: &Document) -> Result<Vec<u8>, HedlError> {
    to_parquet_bytes_with_config_async(doc, &ToParquetConfig::default()).await
}

/// Convert a HEDL document to Parquet bytes with custom configuration asynchronously.
///
/// Uses `spawn_blocking` to offload CPU-bound Parquet writing to the
/// blocking thread pool, avoiding blocking the async runtime.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "async-io")]
/// # async fn example() {
/// use hedl_core::Document;
/// use hedl_parquet::async_io::to_parquet_bytes_with_config_async;
/// use hedl_parquet::ToParquetConfig;
///
/// let doc = Document::new((2, 0));
/// let config = ToParquetConfig::default();
/// let bytes = to_parquet_bytes_with_config_async(&doc, &config).await.unwrap();
/// # }
/// ```
pub async fn to_parquet_bytes_with_config_async(
    doc: &Document,
    config: &ToParquetConfig,
) -> Result<Vec<u8>, HedlError> {
    // Clone data for move into blocking task
    let doc_owned = doc.clone();
    let config_owned = config.clone();

    // Offload CPU-bound writing to blocking thread pool
    tokio::task::spawn_blocking(move || to_parquet_bytes_with_config(&doc_owned, &config_owned))
        .await
        .map_err(|e| HedlError::io(format!("Parquet writing task failed: {e}")))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Item, MatrixList, Node, Value};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_roundtrip_async() {
        let mut doc = Document::new((2, 0));
        let mut list = MatrixList::new("TestEntity", vec!["id".to_string(), "name".to_string()]);
        list.rows.push(Node::new(
            "TestEntity",
            "e1",
            vec![
                Value::String("e1".to_string().into()),
                Value::String("Entity One".to_string().into()),
            ],
        ));
        doc.root.insert("entities".to_string(), Item::List(list));

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.parquet");

        // Write async
        to_parquet_async(&doc, &path).await.unwrap();

        // Read async
        let loaded = from_parquet_async(&path).await.unwrap();
        assert!(loaded.root.contains_key("entities"));
    }

    #[tokio::test]
    async fn test_bytes_roundtrip_async() {
        let mut doc = Document::new((2, 0));
        let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
        list.rows.push(Node::new(
            "Item",
            "i1",
            vec![Value::String("i1".to_string().into()), Value::Int(42)],
        ));
        doc.root.insert("items".to_string(), Item::List(list));

        // Write to bytes async
        let bytes = to_parquet_bytes_async(&doc).await.unwrap();
        assert!(!bytes.is_empty());

        // Read from bytes async
        let loaded = from_parquet_bytes_async(&bytes).await.unwrap();
        assert!(loaded.root.contains_key("items"));
    }
}
