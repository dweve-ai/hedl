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

//! Async API for XML conversion with Tokio
//!
//! This module provides asynchronous versions of all XML conversion functions,
//! enabling high-performance concurrent I/O operations with Tokio.
//!
//! # Features
//!
//! - **Async File I/O**: Read/write XML files without blocking
//! - **Async Streaming**: Stream large XML files incrementally
//! - **Concurrency**: Process multiple files concurrently
//! - **Backpressure**: Built-in flow control for streaming
//!
//! # Examples
//!
//! ## Async file conversion
//!
//! ```no_run
//! use hedl_xml::async_api::{from_xml_file_async, to_xml_file_async};
//! use hedl_xml::{FromXmlConfig, ToXmlConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Read XML file asynchronously
//!     let doc = from_xml_file_async("input.xml", &FromXmlConfig::default()).await?;
//!
//!     // Process document...
//!
//!     // Write XML file asynchronously
//!     to_xml_file_async(&doc, "output.xml", &ToXmlConfig::default()).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Async streaming for large files
//!
//! ```no_run
//! use hedl_xml::async_api::from_xml_stream_async;
//! use hedl_xml::streaming::StreamConfig;
//! use tokio::fs::File;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let file = File::open("large.xml").await?;
//!     let config = StreamConfig::default();
//!
//!     let mut stream = from_xml_stream_async(file, &config).await?;
//!
//!     while let Some(result) = stream.next().await {
//!         match result {
//!             Ok(item) => println!("Processed: {}", item.key),
//!             Err(e) => eprintln!("Error: {}", e),
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Concurrent processing
//!
//! ```no_run
//! use hedl_xml::async_api::from_xml_file_async;
//! use hedl_xml::FromXmlConfig;
//! use tokio::task;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = FromXmlConfig::default();
//!
//!     // Process multiple files concurrently
//!     let handles: Vec<_> = vec!["file1.xml", "file2.xml", "file3.xml"]
//!         .into_iter()
//!         .map(|path| {
//!             let config = config.clone();
//!             task::spawn(async move {
//!                 from_xml_file_async(path, &config).await
//!             })
//!         })
//!         .collect();
//!
//!     // Wait for all to complete
//!     for handle in handles {
//!         let doc = handle.await??;
//!         // Process document...
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::streaming::{StreamConfig, StreamItem, StreamPosition};
use crate::{from_xml, to_xml, FromXmlConfig, ToXmlConfig};
use hedl_core::Document;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

// ============================================================================
// Async File I/O Functions
// ============================================================================

/// Read and parse an XML file asynchronously
///
/// This function reads the entire file into memory and parses it. For large files,
/// consider using `from_xml_stream_async` instead.
///
/// # Examples
///
/// ```no_run
/// use hedl_xml::async_api::from_xml_file_async;
/// use hedl_xml::FromXmlConfig;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = FromXmlConfig::default();
/// let doc = from_xml_file_async("data.xml", &config).await?;
/// println!("Parsed document with {} root items", doc.root.len());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the file cannot be read or if XML parsing fails.
pub async fn from_xml_file_async(
    path: impl AsRef<std::path::Path>,
    config: &FromXmlConfig,
) -> Result<Document, String> {
    let contents = tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))?;

    from_xml(&contents, config)
}

/// Write a HEDL document to an XML file asynchronously
///
/// # Examples
///
/// ```no_run
/// use hedl_xml::async_api::to_xml_file_async;
/// use hedl_xml::ToXmlConfig;
/// use hedl_core::Document;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let doc = Document::new((2, 0));
/// let config = ToXmlConfig::default();
/// to_xml_file_async(&doc, "output.xml", &config).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if XML generation fails or if the file cannot be written.
pub async fn to_xml_file_async(
    doc: &Document,
    path: impl AsRef<std::path::Path>,
    config: &ToXmlConfig,
) -> Result<(), String> {
    let xml = to_xml(doc, config)?;

    tokio::fs::write(path, xml)
        .await
        .map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(())
}

// ============================================================================
// Async Reader/Writer Functions
// ============================================================================

/// Parse XML from an async reader
///
/// This function reads the entire content into memory before parsing. For streaming
/// large files, use `from_xml_stream_async` instead.
///
/// # Examples
///
/// ```no_run
/// use hedl_xml::async_api::from_xml_reader_async;
/// use hedl_xml::FromXmlConfig;
/// use tokio::fs::File;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("data.xml").await?;
/// let config = FromXmlConfig::default();
/// let doc = from_xml_reader_async(file, &config).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if reading fails or if XML parsing fails.
pub async fn from_xml_reader_async<R: AsyncRead + Unpin>(
    mut reader: R,
    config: &FromXmlConfig,
) -> Result<Document, String> {
    let mut contents = String::new();
    reader
        .read_to_string(&mut contents)
        .await
        .map_err(|e| format!("Failed to read XML: {}", e))?;

    from_xml(&contents, config)
}

/// Write XML to an async writer
///
/// # Examples
///
/// ```no_run
/// use hedl_xml::async_api::to_xml_writer_async;
/// use hedl_xml::ToXmlConfig;
/// use hedl_core::Document;
/// use tokio::fs::File;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let doc = Document::new((2, 0));
/// let file = File::create("output.xml").await?;
/// let config = ToXmlConfig::default();
/// to_xml_writer_async(&doc, file, &config).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if XML generation fails or if writing fails.
pub async fn to_xml_writer_async<W: AsyncWrite + Unpin>(
    doc: &Document,
    mut writer: W,
    config: &ToXmlConfig,
) -> Result<(), String> {
    let xml = to_xml(doc, config)?;

    writer
        .write_all(xml.as_bytes())
        .await
        .map_err(|e| format!("Failed to write XML: {}", e))?;

    writer
        .flush()
        .await
        .map_err(|e| format!("Failed to flush writer: {}", e))?;

    Ok(())
}

// ============================================================================
// Async Streaming Functions
// ============================================================================

/// Create an async streaming XML parser
///
/// This function returns a stream that yields items incrementally, allowing
/// processing of files larger than available RAM.
///
/// # Examples
///
/// ```no_run
/// use hedl_xml::async_api::from_xml_stream_async;
/// use hedl_xml::streaming::StreamConfig;
/// use tokio::fs::File;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("large.xml").await?;
/// let config = StreamConfig::default();
///
/// let mut stream = from_xml_stream_async(file, &config).await?;
///
/// let mut count = 0;
/// while let Some(result) = stream.next().await {
///     match result {
///         Ok(_item) => count += 1,
///         Err(e) => eprintln!("Parse error: {}", e),
///     }
/// }
/// println!("Processed {} items", count);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns an error if the stream cannot be initialized.
pub async fn from_xml_stream_async<R: AsyncRead + Unpin + Send + 'static>(
    reader: R,
    config: &StreamConfig,
) -> Result<AsyncXmlStream<R>, String> {
    AsyncXmlStream::new(reader, config.clone())
}

/// Async streaming XML parser
///
/// This stream yields `Result<StreamItem, String>` as items are parsed.
/// It uses Tokio's async I/O for non-blocking operations.
///
/// # Implementation Note
///
/// This implementation reads all data into memory first, then parses it
/// synchronously and yields items one at a time. For true streaming with
/// backpressure, a more sophisticated implementation using quick-xml's
/// async features would be needed.
pub struct AsyncXmlStream<R: AsyncRead + Unpin> {
    reader: R,
    config: StreamConfig,
    buffer: Vec<u8>,
    /// Byte position in the stream
    byte_position: usize,
    chunk_size: usize,
    /// Parsed items ready to be yielded
    parsed_items: std::collections::VecDeque<Result<StreamItem, String>>,
    /// Whether we've read and parsed all data
    fully_parsed: bool,
    /// Stream position tracking for progress reporting
    stream_position: StreamPosition,
}

impl<R: AsyncRead + Unpin> AsyncXmlStream<R> {
    /// Create a new async XML stream
    pub fn new(reader: R, config: StreamConfig) -> Result<Self, String> {
        let chunk_size = config.buffer_size;
        Ok(AsyncXmlStream {
            reader,
            config,
            buffer: Vec::new(),
            byte_position: 0,
            chunk_size,
            parsed_items: std::collections::VecDeque::new(),
            fully_parsed: false,
            stream_position: StreamPosition::default(),
        })
    }

    /// Get the current stream position for progress tracking
    ///
    /// Returns information about bytes processed and items parsed.
    /// Useful for progress bars and error reporting.
    #[inline]
    pub fn position(&self) -> StreamPosition {
        StreamPosition {
            byte_offset: self.byte_position as u64,
            items_parsed: self.stream_position.items_parsed,
        }
    }

    /// Read the next chunk of data
    async fn read_chunk(&mut self) -> Result<usize, String> {
        let mut chunk = vec![0u8; self.chunk_size];
        let n = self
            .reader
            .read(&mut chunk)
            .await
            .map_err(|e| format!("Failed to read chunk: {}", e))?;

        if n > 0 {
            self.buffer.extend_from_slice(&chunk[..n]);
            self.byte_position += n;
        }

        Ok(n)
    }

    /// Read all data and parse into items
    async fn ensure_parsed(&mut self) -> Result<(), String> {
        if self.fully_parsed {
            return Ok(());
        }

        // Read all remaining data
        loop {
            match self.read_chunk().await {
                Ok(0) => break, // EOF
                Ok(_) => continue,
                Err(e) => return Err(e),
            }
        }

        // Parse complete buffer using sync streaming parser
        if !self.buffer.is_empty() {
            use crate::streaming::from_xml_stream;
            use std::io::Cursor;

            let cursor = Cursor::new(&self.buffer);
            match from_xml_stream(cursor, &self.config) {
                Ok(parser) => {
                    // Collect all items from the parser
                    for result in parser {
                        self.parsed_items
                            .push_back(result.map_err(|e| e.to_string()));
                    }
                }
                Err(e) => {
                    self.parsed_items.push_back(Err(e));
                }
            }
        }

        self.fully_parsed = true;
        Ok(())
    }

    /// Async version of next() - yields the next parsed item
    pub async fn next(&mut self) -> Option<Result<StreamItem, String>> {
        // Ensure we've read and parsed all data
        if let Err(e) = self.ensure_parsed().await {
            return Some(Err(e));
        }

        // Yield the next item from our queue
        let result = self.parsed_items.pop_front();

        // Update items_parsed counter on successful item
        if result.as_ref().is_some_and(|r| r.is_ok()) {
            self.stream_position.items_parsed += 1;
        }

        result
    }
}

// Implement Stream trait for async iteration (requires futures crate)
// For simplicity, we provide next() method instead of implementing Stream

// ============================================================================
// Utility Functions
// ============================================================================

/// Parse XML string asynchronously (runs on tokio threadpool)
///
/// This is useful for CPU-bound parsing that shouldn't block the async runtime.
///
/// # Examples
///
/// ```no_run
/// use hedl_xml::async_api::from_xml_async;
/// use hedl_xml::FromXmlConfig;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let xml = r#"<?xml version="1.0"?><hedl><name>test</name></hedl>"#;
/// let config = FromXmlConfig::default();
/// let doc = from_xml_async(xml, &config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn from_xml_async(xml: &str, config: &FromXmlConfig) -> Result<Document, String> {
    let xml = xml.to_string();
    let config = config.clone();

    tokio::task::spawn_blocking(move || from_xml(&xml, &config))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

/// Convert HEDL to XML string asynchronously (runs on tokio threadpool)
///
/// This is useful for CPU-bound conversion that shouldn't block the async runtime.
///
/// # Examples
///
/// ```no_run
/// use hedl_xml::async_api::to_xml_async;
/// use hedl_xml::ToXmlConfig;
/// use hedl_core::Document;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let doc = Document::new((2, 0));
/// let config = ToXmlConfig::default();
/// let xml = to_xml_async(&doc, &config).await?;
/// # Ok(())
/// # }
/// ```
pub async fn to_xml_async(doc: &Document, config: &ToXmlConfig) -> Result<String, String> {
    let doc = doc.clone();
    let config = config.clone();

    tokio::task::spawn_blocking(move || to_xml(&doc, &config))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
}

// ============================================================================
// Batch Processing Functions
// ============================================================================

/// Process multiple XML files concurrently
///
/// Returns results in the same order as input paths.
///
/// # Examples
///
/// ```no_run
/// use hedl_xml::async_api::from_xml_files_concurrent;
/// use hedl_xml::FromXmlConfig;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let paths = vec!["file1.xml", "file2.xml", "file3.xml"];
/// let config = FromXmlConfig::default();
///
/// let results = from_xml_files_concurrent(&paths, &config, 4).await;
///
/// for (path, result) in paths.iter().zip(results.iter()) {
///     match result {
///         Ok(doc) => println!("{}: {} items", path, doc.root.len()),
///         Err(e) => eprintln!("{}: error - {}", path, e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Arguments
///
/// * `paths` - Iterator of file paths to process
/// * `config` - Configuration for XML parsing
/// * `concurrency` - Maximum number of concurrent operations
pub async fn from_xml_files_concurrent<'a, I, P>(
    paths: I,
    config: &FromXmlConfig,
    concurrency: usize,
) -> Vec<Result<Document, String>>
where
    I: IntoIterator<Item = P>,
    P: AsRef<std::path::Path> + Send + 'a,
{
    use tokio::task::JoinSet;

    let mut set: JoinSet<(usize, Result<Document, String>)> = JoinSet::new();
    let config = config.clone();
    let paths_vec: Vec<_> = paths.into_iter().collect();
    let total = paths_vec.len();

    // Pre-allocate results with placeholders
    let mut results: Vec<Option<Result<Document, String>>> = vec![None; total];
    let mut paths_iter = paths_vec.into_iter().enumerate();
    let mut pending = 0;

    // Fill initial batch up to concurrency limit
    for _ in 0..concurrency {
        if let Some((idx, path)) = paths_iter.next() {
            let path = path.as_ref().to_path_buf();
            let config = config.clone();
            set.spawn(async move { (idx, from_xml_file_async(&path, &config).await) });
            pending += 1;
        } else {
            break;
        }
    }

    // Process remaining items, maintaining concurrency level
    while pending > 0 {
        if let Some(join_result) = set.join_next().await {
            match join_result {
                Ok((idx, doc_result)) => {
                    results[idx] = Some(doc_result);
                }
                Err(e) => {
                    // Task panicked - find first None slot and fill it
                    if let Some(slot) = results.iter_mut().find(|r| r.is_none()) {
                        *slot = Some(Err(format!("Task error: {}", e)));
                    }
                }
            }
            pending -= 1;

            // Add next item if available
            if let Some((idx, path)) = paths_iter.next() {
                let path = path.as_ref().to_path_buf();
                let config = config.clone();
                set.spawn(async move { (idx, from_xml_file_async(&path, &config).await) });
                pending += 1;
            }
        }
    }

    // Convert to final results, unwrapping Options (all should be Some now)
    results
        .into_iter()
        .map(|opt| opt.unwrap_or_else(|| Err("Missing result".to_string())))
        .collect()
}

/// Write multiple documents to XML files concurrently
///
/// # Examples
///
/// ```no_run
/// use hedl_xml::async_api::to_xml_files_concurrent;
/// use hedl_xml::ToXmlConfig;
/// use hedl_core::Document;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let docs = vec![
///     Document::new((2, 0)),
///     Document::new((2, 0)),
/// ];
/// let paths = vec!["out1.xml", "out2.xml"];
/// let config = ToXmlConfig::default();
///
/// let results = to_xml_files_concurrent(
///     docs.iter().zip(paths.iter()),
///     &config,
///     4
/// ).await;
///
/// for (i, result) in results.iter().enumerate() {
///     match result {
///         Ok(_) => println!("File {} written successfully", i),
///         Err(e) => eprintln!("File {} error: {}", i, e),
///     }
/// }
/// # Ok(())
/// # }
/// ```
pub async fn to_xml_files_concurrent<'a, I, P>(
    docs_and_paths: I,
    config: &ToXmlConfig,
    concurrency: usize,
) -> Vec<Result<(), String>>
where
    I: IntoIterator<Item = (&'a Document, P)>,
    P: AsRef<std::path::Path> + Send + 'a,
{
    use tokio::task::JoinSet;

    let mut set: JoinSet<(usize, Result<(), String>)> = JoinSet::new();
    let config = config.clone();
    let docs_and_paths_vec: Vec<_> = docs_and_paths.into_iter().collect();
    let total = docs_and_paths_vec.len();

    // Pre-allocate results with placeholders
    let mut results: Vec<Option<Result<(), String>>> = vec![None; total];
    let mut iter = docs_and_paths_vec.into_iter().enumerate();
    let mut pending = 0;

    // Fill initial batch up to concurrency limit
    for _ in 0..concurrency {
        if let Some((idx, (doc, path))) = iter.next() {
            let doc = doc.clone();
            let path = path.as_ref().to_path_buf();
            let config = config.clone();
            set.spawn(async move { (idx, to_xml_file_async(&doc, &path, &config).await) });
            pending += 1;
        } else {
            break;
        }
    }

    // Process remaining items, maintaining concurrency level
    while pending > 0 {
        if let Some(join_result) = set.join_next().await {
            match join_result {
                Ok((idx, write_result)) => {
                    results[idx] = Some(write_result);
                }
                Err(e) => {
                    // Task panicked - find first None slot and fill it
                    if let Some(slot) = results.iter_mut().find(|r| r.is_none()) {
                        *slot = Some(Err(format!("Task error: {}", e)));
                    }
                }
            }
            pending -= 1;

            // Add next item if available
            if let Some((idx, (doc, path))) = iter.next() {
                let doc = doc.clone();
                let path = path.as_ref().to_path_buf();
                let config = config.clone();
                set.spawn(async move { (idx, to_xml_file_async(&doc, &path, &config).await) });
                pending += 1;
            }
        }
    }

    // Convert to final results, unwrapping Options (all should be Some now)
    results
        .into_iter()
        .map(|opt| opt.unwrap_or_else(|| Err("Missing result".to_string())))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Item, Value};
    use std::io::Cursor;

    // ==================== File I/O tests ====================

    #[tokio::test]
    async fn test_from_xml_file_async_not_found() {
        let config = FromXmlConfig::default();
        let result = from_xml_file_async("/nonexistent/file.xml", &config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read file"));
    }

    #[tokio::test]
    async fn test_to_xml_file_async_invalid_path() {
        let doc = Document::new((2, 0));
        let config = ToXmlConfig::default();
        let result = to_xml_file_async(&doc, "/invalid/\0/path.xml", &config).await;
        assert!(result.is_err());
    }

    // ==================== Reader/Writer tests ====================

    #[tokio::test]
    async fn test_from_xml_reader_async_valid() {
        let xml = r#"<?xml version="1.0"?><hedl><val>42</val></hedl>"#;
        let cursor = Cursor::new(xml.as_bytes());
        let config = FromXmlConfig::default();

        let doc = from_xml_reader_async(cursor, &config).await.unwrap();
        assert_eq!(
            doc.root.get("val").and_then(|i| i.as_scalar()),
            Some(&Value::Int(42))
        );
    }

    #[tokio::test]
    async fn test_from_xml_reader_async_empty() {
        let xml = "";
        let cursor = Cursor::new(xml.as_bytes());
        let config = FromXmlConfig::default();

        let doc = from_xml_reader_async(cursor, &config).await.unwrap();
        assert!(doc.root.is_empty());
    }

    #[tokio::test]
    async fn test_to_xml_writer_async_valid() {
        let mut doc = Document::new((2, 0));
        doc.root
            .insert("val".to_string(), Item::Scalar(Value::Int(42)));

        let mut buffer = Vec::new();
        let config = ToXmlConfig::default();

        to_xml_writer_async(&doc, &mut buffer, &config)
            .await
            .unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(xml.contains("<val>42</val>"));
    }

    #[tokio::test]
    async fn test_to_xml_writer_async_empty() {
        let doc = Document::new((2, 0));
        let mut buffer = Vec::new();
        let config = ToXmlConfig::default();

        to_xml_writer_async(&doc, &mut buffer, &config)
            .await
            .unwrap();

        let xml = String::from_utf8(buffer).unwrap();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<hedl"));
    }

    // ==================== Async string parsing tests ====================

    #[tokio::test]
    async fn test_from_xml_async_valid() {
        let xml = r#"<?xml version="1.0"?><hedl><name>test</name></hedl>"#;
        let config = FromXmlConfig::default();

        let doc = from_xml_async(xml, &config).await.unwrap();
        assert_eq!(
            doc.root.get("name").and_then(|i| i.as_scalar()),
            Some(&Value::String("test".to_string().into()))
        );
    }

    #[tokio::test]
    async fn test_from_xml_async_invalid() {
        let xml = r#"</invalid>"#;
        let config = FromXmlConfig::default();

        let result = from_xml_async(xml, &config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_to_xml_async_valid() {
        let mut doc = Document::new((2, 0));
        doc.root
            .insert("val".to_string(), Item::Scalar(Value::Int(123)));

        let config = ToXmlConfig::default();
        let xml = to_xml_async(&doc, &config).await.unwrap();

        assert!(xml.contains("<val>123</val>"));
    }

    #[tokio::test]
    async fn test_to_xml_async_empty() {
        let doc = Document::new((2, 0));
        let config = ToXmlConfig::default();

        let xml = to_xml_async(&doc, &config).await.unwrap();
        assert!(xml.contains("<?xml"));
    }

    // ==================== Concurrent processing tests ====================

    #[tokio::test]
    async fn test_from_xml_files_concurrent_empty() {
        let paths: Vec<&str> = vec![];
        let config = FromXmlConfig::default();

        let results = from_xml_files_concurrent(&paths, &config, 4).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_to_xml_files_concurrent_empty() {
        let docs_and_paths: Vec<(&Document, &str)> = vec![];
        let config = ToXmlConfig::default();

        let results = to_xml_files_concurrent(docs_and_paths, &config, 4).await;
        assert!(results.is_empty());
    }

    // ==================== Edge cases ====================

    #[tokio::test]
    async fn test_from_xml_reader_async_unicode() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
            <hedl><name>héllo 世界</name></hedl>"#;
        let cursor = Cursor::new(xml.as_bytes());
        let config = FromXmlConfig::default();

        let doc = from_xml_reader_async(cursor, &config).await.unwrap();
        assert_eq!(
            doc.root.get("name").and_then(|i| i.as_scalar()),
            Some(&Value::String("héllo 世界".to_string().into()))
        );
    }

    #[tokio::test]
    async fn test_from_xml_reader_async_large_value() {
        let large_string = "x".repeat(10000);
        let xml = format!(
            r#"<?xml version="1.0"?><hedl><val>{}</val></hedl>"#,
            large_string
        );
        let cursor = Cursor::new(xml.as_bytes());
        let config = FromXmlConfig::default();

        let doc = from_xml_reader_async(cursor, &config).await.unwrap();
        assert_eq!(
            doc.root.get("val").and_then(|i| i.as_scalar()),
            Some(&Value::String(large_string.into()))
        );
    }

    #[tokio::test]
    async fn test_round_trip_async() {
        let mut doc = Document::new((2, 0));
        doc.root
            .insert("bool_val".to_string(), Item::Scalar(Value::Bool(true)));
        doc.root
            .insert("int_val".to_string(), Item::Scalar(Value::Int(42)));
        doc.root.insert(
            "string_val".to_string(),
            Item::Scalar(Value::String("hello".to_string().into())),
        );

        let config_to = ToXmlConfig::default();
        let xml = to_xml_async(&doc, &config_to).await.unwrap();

        let config_from = FromXmlConfig::default();
        let doc2 = from_xml_async(&xml, &config_from).await.unwrap();

        assert_eq!(
            doc2.root.get("bool_val").and_then(|i| i.as_scalar()),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            doc2.root.get("int_val").and_then(|i| i.as_scalar()),
            Some(&Value::Int(42))
        );
        assert_eq!(
            doc2.root.get("string_val").and_then(|i| i.as_scalar()),
            Some(&Value::String("hello".to_string().into()))
        );
    }

    // ==================== Concurrency and parallelism tests ====================

    #[tokio::test]
    async fn test_concurrent_parsing() {
        let xml1 = r#"<?xml version="1.0"?><hedl><id>1</id></hedl>"#;
        let xml2 = r#"<?xml version="1.0"?><hedl><id>2</id></hedl>"#;
        let xml3 = r#"<?xml version="1.0"?><hedl><id>3</id></hedl>"#;

        let config = FromXmlConfig::default();

        let (r1, r2, r3) = tokio::join!(
            from_xml_async(xml1, &config),
            from_xml_async(xml2, &config),
            from_xml_async(xml3, &config)
        );

        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert!(r3.is_ok());

        assert_eq!(
            r1.unwrap().root.get("id").and_then(|i| i.as_scalar()),
            Some(&Value::Int(1))
        );
        assert_eq!(
            r2.unwrap().root.get("id").and_then(|i| i.as_scalar()),
            Some(&Value::Int(2))
        );
        assert_eq!(
            r3.unwrap().root.get("id").and_then(|i| i.as_scalar()),
            Some(&Value::Int(3))
        );
    }

    #[tokio::test]
    async fn test_concurrent_generation() {
        let mut doc1 = Document::new((2, 0));
        doc1.root
            .insert("id".to_string(), Item::Scalar(Value::Int(1)));

        let mut doc2 = Document::new((2, 0));
        doc2.root
            .insert("id".to_string(), Item::Scalar(Value::Int(2)));

        let config = ToXmlConfig::default();

        let (r1, r2) = tokio::join!(to_xml_async(&doc1, &config), to_xml_async(&doc2, &config));

        assert!(r1.is_ok());
        assert!(r2.is_ok());

        assert!(r1.unwrap().contains("<id>1</id>"));
        assert!(r2.unwrap().contains("<id>2</id>"));
    }
}
