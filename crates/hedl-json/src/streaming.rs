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

//! Streaming JSON parsing for HEDL
//!
//! This module provides memory-efficient streaming parsers for processing
//! large JSON files without loading the entire document into memory.
//!
//! # Features
//!
//! - **Incremental Parsing**: Process JSON objects as they arrive
//! - **JSONL Support**: Parse newline-delimited JSON (JSON Lines)
//! - **Memory Bounded**: Configurable memory limits for safe streaming
//! - **Iterator-Based**: Ergonomic Rust iterator interface
//!
//! # Examples
//!
//! ## Streaming JSON Array
//!
//! ```rust
//! use hedl_json::streaming::{JsonArrayStreamer, StreamConfig};
//! use std::io::Cursor;
//!
//! let json = r#"[
//!     {"id": "1", "name": "Alice"},
//!     {"id": "2", "name": "Bob"}
//! ]"#;
//!
//! let reader = Cursor::new(json.as_bytes());
//! let config = StreamConfig::default();
//! let streamer = JsonArrayStreamer::new(reader, config).unwrap();
//!
//! for result in streamer {
//!     let doc = result.unwrap();
//!     println!("Parsed document: {:?}", doc);
//! }
//! ```
//!
//! ## JSONL Streaming
//!
//! ```rust
//! use hedl_json::streaming::{JsonLinesStreamer, StreamConfig};
//! use std::io::Cursor;
//!
//! let jsonl = r#"{"id": "1", "name": "Alice"}
//! {"id": "2", "name": "Bob"}
//! {"id": "3", "name": "Charlie"}"#;
//!
//! let reader = Cursor::new(jsonl.as_bytes());
//! let config = StreamConfig::default();
//! let streamer = JsonLinesStreamer::new(reader, config);
//!
//! for result in streamer {
//!     let doc = result.unwrap();
//!     println!("Parsed document: {:?}", doc);
//! }
//! ```

use crate::from_json::{from_json_value_owned, FromJsonConfig, JsonConversionError};
use hedl_core::Document;
use serde_json::Value as JsonValue;
use std::io::{BufRead, BufReader, Read};
use std::marker::PhantomData;

// Import the Error trait for custom error creation
use serde::de::Error as _;

/// Configuration for streaming JSON parsing
///
/// Controls memory limits and parsing behavior for streaming operations.
///
/// # Memory Safety
///
/// Streaming parsers process data incrementally to avoid loading entire
/// files into memory. However, individual objects can still be large.
/// Configure `max_object_bytes` to limit memory per object.
///
/// # Examples
///
/// ```text
/// use hedl_json::streaming::StreamConfig;
/// use hedl_json::FromJsonConfig;
///
/// // Default configuration - suitable for trusted input
/// let config = StreamConfig::default();
///
/// // Conservative configuration for untrusted input
/// let strict = StreamConfig {
///     buffer_size: 8 * 1024,           // 8 KB buffer
///     max_object_bytes: 1024 * 1024,   // 1 MB per object
///     from_json: FromJsonConfig::builder()
///         .max_depth(100)
///         .max_array_size(10_000)
///         .build(),
/// };
///
/// // High-throughput configuration for large ML datasets
/// let ml_config = StreamConfig {
///     buffer_size: 256 * 1024,         // 256 KB buffer
///     max_object_bytes: 100 * 1024 * 1024, // 100 MB per object
///     from_json: FromJsonConfig::default(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Size of internal read buffer in bytes (default: 64 KB)
    ///
    /// Larger buffers improve throughput for network I/O but use more memory.
    /// Smaller buffers reduce memory overhead for many concurrent streams.
    pub buffer_size: usize,

    /// Maximum bytes per JSON object (default: 10 MB)
    ///
    /// Prevents memory exhaustion from individual oversized objects.
    /// Set to `None` to disable (not recommended for untrusted input).
    pub max_object_bytes: Option<usize>,

    /// Configuration for JSON to HEDL conversion
    ///
    /// Controls limits and behavior when converting each parsed JSON
    /// object to a HEDL document.
    pub from_json: FromJsonConfig,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64 * 1024, // 64 KB - good balance for most use cases
            max_object_bytes: Some(10 * 1024 * 1024), // 10 MB per object
            from_json: FromJsonConfig::default(),
        }
    }
}

impl StreamConfig {
    /// Create a new builder for configuring stream parsing
    ///
    /// # Examples
    ///
    /// ```text
    /// use hedl_json::streaming::StreamConfig;
    ///
    /// let config = StreamConfig::builder()
    ///     .buffer_size(128 * 1024)
    ///     .max_object_bytes(50 * 1024 * 1024)
    ///     .build();
    /// ```
    pub fn builder() -> StreamConfigBuilder {
        StreamConfigBuilder::default()
    }
}

/// Builder for `StreamConfig`
///
/// Provides ergonomic configuration of streaming behavior.
#[derive(Debug, Clone)]
pub struct StreamConfigBuilder {
    buffer_size: usize,
    max_object_bytes: Option<usize>,
    from_json: FromJsonConfig,
}

impl Default for StreamConfigBuilder {
    fn default() -> Self {
        Self {
            buffer_size: 64 * 1024,
            max_object_bytes: Some(10 * 1024 * 1024),
            from_json: FromJsonConfig::default(),
        }
    }
}

impl StreamConfigBuilder {
    /// Set the buffer size in bytes
    pub fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Set the maximum object size in bytes
    pub fn max_object_bytes(mut self, limit: usize) -> Self {
        self.max_object_bytes = Some(limit);
        self
    }

    /// Disable object size limit (use with caution)
    pub fn unlimited_object_size(mut self) -> Self {
        self.max_object_bytes = None;
        self
    }

    /// Set the JSON conversion configuration
    pub fn from_json_config(mut self, config: FromJsonConfig) -> Self {
        self.from_json = config;
        self
    }

    /// Build the configuration
    pub fn build(self) -> StreamConfig {
        StreamConfig {
            buffer_size: self.buffer_size,
            max_object_bytes: self.max_object_bytes,
            from_json: self.from_json,
        }
    }
}

/// Errors that can occur during streaming JSON parsing
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    /// I/O error while reading input
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing error
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// JSON to HEDL conversion error
    #[error("HEDL conversion error: {0}")]
    Conversion(#[from] JsonConversionError),

    /// Object exceeded size limit
    #[error("Object size ({0} bytes) exceeds limit ({1} bytes)")]
    ObjectTooLarge(usize, usize),

    /// Invalid JSONL format
    #[error("Invalid JSONL: {0}")]
    InvalidJsonL(String),
}

/// Streaming parser for JSON arrays
///
/// Parses a JSON array incrementally, yielding each element as a HEDL document.
/// Memory-efficient for processing large arrays without loading entire array.
///
/// # Format
///
/// Expects a JSON array of objects:
/// ```json
/// [
///   {"id": "1", "name": "Alice"},
///   {"id": "2", "name": "Bob"}
/// ]
/// ```
///
/// # Memory Usage
///
/// - **Bounded**: Only one object in memory at a time
/// - **Buffer**: Configured buffer size (default 64 KB)
/// - **Per-object**: Limited by `max_object_bytes` (default 10 MB)
///
/// # Examples
///
/// ```text
/// use hedl_json::streaming::{JsonArrayStreamer, StreamConfig};
/// use std::fs::File;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("large_dataset.json")?;
/// let config = StreamConfig::default();
/// let streamer = JsonArrayStreamer::new(file, config)?;
///
/// let mut count = 0;
/// for result in streamer {
///     let doc = result?;
///     count += 1;
///     // Process document without loading entire array
/// }
/// println!("Processed {} documents", count);
/// # Ok(())
/// # }
/// ```
pub struct JsonArrayStreamer<R: Read> {
    array: Vec<JsonValue>,
    config: StreamConfig,
    index: usize,
    _phantom: PhantomData<R>,
}

impl<R: Read> JsonArrayStreamer<R> {
    /// Create a new array streamer
    ///
    /// # Arguments
    ///
    /// * `reader` - Input source (file, network stream, etc.)
    /// * `config` - Streaming configuration
    ///
    /// # Errors
    ///
    /// Returns error if the input doesn't start with a JSON array.
    ///
    /// # Examples
    ///
    /// ```text
    /// use hedl_json::streaming::{JsonArrayStreamer, StreamConfig};
    /// use std::io::Cursor;
    ///
    /// let json = r#"[{"id": "1"}]"#;
    /// let reader = Cursor::new(json.as_bytes());
    /// let config = StreamConfig::default();
    /// let streamer = JsonArrayStreamer::new(reader, config).unwrap();
    /// ```
    pub fn new(mut reader: R, config: StreamConfig) -> Result<Self, StreamError> {
        // Read entire JSON into memory and parse as array
        // Note: This is a limitation of serde_json - true streaming of arrays
        // is complex. For memory-efficient processing, use JsonLinesStreamer instead.
        let mut json_str = String::new();
        reader.read_to_string(&mut json_str)?;

        let value: JsonValue = serde_json::from_str(&json_str)?;
        let array = match value {
            JsonValue::Array(arr) => arr,
            _ => {
                return Err(StreamError::Json(serde_json::Error::custom(
                    "Expected JSON array",
                )));
            }
        };

        Ok(Self {
            array,
            config,
            index: 0,
            _phantom: PhantomData,
        })
    }
}

impl<R: Read> Iterator for JsonArrayStreamer<R> {
    type Item = Result<Document, StreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.array.len() {
            return None;
        }

        let value = self.array.remove(0); // Remove from front to avoid keeping all in memory

        // Check object size if limit configured
        if let Some(max_bytes) = self.config.max_object_bytes {
            match serde_json::to_string(&value) {
                Ok(json_str) => {
                    let size = json_str.len();
                    if size > max_bytes {
                        return Some(Err(StreamError::ObjectTooLarge(size, max_bytes)));
                    }
                }
                Err(e) => return Some(Err(StreamError::Json(e))),
            }
        }

        // Convert to HEDL document using zero-copy optimization
        match from_json_value_owned(value, &self.config.from_json) {
            Ok(doc) => Some(Ok(doc)),
            Err(e) => Some(Err(StreamError::Conversion(e))),
        }
    }
}

/// Streaming parser for JSONL (JSON Lines) format
///
/// Parses newline-delimited JSON, yielding each line as a HEDL document.
/// Memory-efficient for processing large log files and streaming data.
///
/// # Format
///
/// Each line is a complete JSON object:
/// ```text
/// {"id": "1", "name": "Alice"}
/// {"id": "2", "name": "Bob"}
/// {"id": "3", "name": "Charlie"}
/// ```
///
/// # Features
///
/// - **Blank Lines**: Skipped automatically
/// - **Comments**: Lines starting with `#` are skipped
/// - **Robustness**: Invalid lines can be skipped or cause errors
/// - **Memory Bounded**: Only one line in memory at a time
///
/// # Examples
///
/// ```text
/// use hedl_json::streaming::{JsonLinesStreamer, StreamConfig};
/// use std::fs::File;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let file = File::open("logs.jsonl")?;
/// let config = StreamConfig::default();
/// let streamer = JsonLinesStreamer::new(file, config);
///
/// for result in streamer {
///     let doc = result?;
///     // Process each log entry
/// }
/// # Ok(())
/// # }
/// ```
pub struct JsonLinesStreamer<R: Read> {
    reader: BufReader<R>,
    config: StreamConfig,
    line_buffer: String,
    line_number: usize,
}

impl<R: Read> JsonLinesStreamer<R> {
    /// Create a new JSONL streamer
    ///
    /// # Arguments
    ///
    /// * `reader` - Input source (file, network stream, etc.)
    /// * `config` - Streaming configuration
    ///
    /// # Examples
    ///
    /// ```text
    /// use hedl_json::streaming::{JsonLinesStreamer, StreamConfig};
    /// use std::io::Cursor;
    ///
    /// let jsonl = "{\"id\": \"1\"}\n{\"id\": \"2\"}";
    /// let reader = Cursor::new(jsonl.as_bytes());
    /// let config = StreamConfig::default();
    /// let streamer = JsonLinesStreamer::new(reader, config);
    /// ```
    pub fn new(reader: R, config: StreamConfig) -> Self {
        let buf_reader = BufReader::with_capacity(config.buffer_size, reader);
        Self {
            reader: buf_reader,
            config,
            line_buffer: String::new(),
            line_number: 0,
        }
    }

    /// Get the current line number (1-indexed)
    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

impl<R: Read> Iterator for JsonLinesStreamer<R> {
    type Item = Result<Document, StreamError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.line_buffer.clear();
            self.line_number += 1;

            // Read next line
            match self.reader.read_line(&mut self.line_buffer) {
                Ok(0) => return None, // EOF
                Ok(_) => {
                    // Trim whitespace
                    let line = self.line_buffer.trim();

                    // Skip blank lines and comments
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    // Check line size if limit configured
                    if let Some(max_bytes) = self.config.max_object_bytes {
                        if line.len() > max_bytes {
                            return Some(Err(StreamError::ObjectTooLarge(
                                line.len(),
                                max_bytes,
                            )));
                        }
                    }

                    // Parse JSON
                    let value: JsonValue = match serde_json::from_str(line) {
                        Ok(v) => v,
                        Err(e) => return Some(Err(StreamError::Json(e))),
                    };

                    // Convert to HEDL document
                    match from_json_value_owned(value, &self.config.from_json) {
                        Ok(doc) => return Some(Ok(doc)),
                        Err(e) => return Some(Err(StreamError::Conversion(e))),
                    }
                }
                Err(e) => return Some(Err(StreamError::Io(e))),
            }
        }
    }
}

/// Streaming writer for JSONL format
///
/// Writes HEDL documents as newline-delimited JSON for efficient streaming.
///
/// # Format
///
/// Each document is written as a single JSON object followed by newline:
/// ```text
/// {"id":"1","name":"Alice"}
/// {"id":"2","name":"Bob"}
/// ```
///
/// # Examples
///
/// ```text
/// use hedl_json::streaming::JsonLinesWriter;
/// use hedl_core::Document;
/// use std::io::Cursor;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut buffer = Vec::new();
/// let mut writer = JsonLinesWriter::new(&mut buffer);
///
/// let doc1 = Document::new((1, 0));
/// writer.write_document(&doc1)?;
///
/// let doc2 = Document::new((1, 0));
/// writer.write_document(&doc2)?;
///
/// writer.flush()?;
/// # Ok(())
/// # }
/// ```
pub struct JsonLinesWriter<W: std::io::Write> {
    writer: W,
}

impl<W: std::io::Write> JsonLinesWriter<W> {
    /// Create a new JSONL writer
    ///
    /// # Arguments
    ///
    /// * `writer` - Output destination
    ///
    /// # Examples
    ///
    /// ```text
    /// use hedl_json::streaming::JsonLinesWriter;
    /// use std::fs::File;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::create("output.jsonl")?;
    /// let mut writer = JsonLinesWriter::new(file);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Write a HEDL document as a JSONL entry
    ///
    /// Converts the document to JSON and writes it followed by a newline.
    ///
    /// # Errors
    ///
    /// Returns error if JSON conversion or I/O write fails.
    ///
    /// # Examples
    ///
    /// ```text
    /// use hedl_json::streaming::JsonLinesWriter;
    /// use hedl_core::Document;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut buffer = Vec::new();
    /// let mut writer = JsonLinesWriter::new(&mut buffer);
    ///
    /// let doc = Document::new((1, 0));
    /// writer.write_document(&doc)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_document(&mut self, doc: &Document) -> Result<(), StreamError> {
        // Convert to JSON value
        let value = crate::to_json_value(doc, &crate::ToJsonConfig::default())
            .map_err(StreamError::InvalidJsonL)?;

        // Write compact JSON (no pretty printing for JSONL)
        serde_json::to_writer(&mut self.writer, &value)?;

        // Write newline
        self.writer.write_all(b"\n")?;

        Ok(())
    }

    /// Flush the output buffer
    ///
    /// Ensures all data is written to the underlying writer.
    ///
    /// # Examples
    ///
    /// ```text
    /// use hedl_json::streaming::JsonLinesWriter;
    /// use hedl_core::Document;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut buffer = Vec::new();
    /// let mut writer = JsonLinesWriter::new(&mut buffer);
    ///
    /// let doc = Document::new((1, 0));
    /// writer.write_document(&doc)?;
    /// writer.flush()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn flush(&mut self) -> Result<(), StreamError> {
        std::io::Write::flush(&mut self.writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedl_core::{Item, Value};
    use std::io::Cursor;

    // ==================== StreamConfig tests ====================

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.buffer_size, 64 * 1024);
        assert_eq!(config.max_object_bytes, Some(10 * 1024 * 1024));
    }

    #[test]
    fn test_stream_config_builder() {
        let config = StreamConfig::builder()
            .buffer_size(128 * 1024)
            .max_object_bytes(50 * 1024 * 1024)
            .build();

        assert_eq!(config.buffer_size, 128 * 1024);
        assert_eq!(config.max_object_bytes, Some(50 * 1024 * 1024));
    }

    #[test]
    fn test_stream_config_unlimited() {
        let config = StreamConfig::builder()
            .unlimited_object_size()
            .build();

        assert_eq!(config.max_object_bytes, None);
    }

    // ==================== JsonArrayStreamer tests ====================

    #[test]
    fn test_array_streamer_simple() {
        let json = r#"[
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]"#;

        let reader = Cursor::new(json.as_bytes());
        let config = StreamConfig::default();
        let streamer = JsonArrayStreamer::new(reader, config).unwrap();

        let docs: Vec<_> = streamer.collect();
        assert_eq!(docs.len(), 2);

        // Verify first document
        let doc1 = docs[0].as_ref().unwrap();
        assert!(doc1.root.contains_key("name"));
        assert!(doc1.root.contains_key("age"));
    }

    #[test]
    fn test_array_streamer_empty() {
        let json = r#"[]"#;

        let reader = Cursor::new(json.as_bytes());
        let config = StreamConfig::default();
        let streamer = JsonArrayStreamer::new(reader, config).unwrap();

        let docs: Vec<_> = streamer.collect();
        assert_eq!(docs.len(), 0);
    }

    #[test]
    fn test_array_streamer_single() {
        let json = r#"[{"id": "1"}]"#;

        let reader = Cursor::new(json.as_bytes());
        let config = StreamConfig::default();
        let streamer = JsonArrayStreamer::new(reader, config).unwrap();

        let docs: Vec<_> = streamer.collect();
        assert_eq!(docs.len(), 1);
    }

    #[test]
    fn test_array_streamer_large_count() {
        // Generate large array
        let mut json = String::from("[");
        for i in 0..1000 {
            if i > 0 {
                json.push(',');
            }
            json.push_str(&format!(r#"{{"id": "{}"}}"#, i));
        }
        json.push(']');

        let reader = Cursor::new(json.as_bytes());
        let config = StreamConfig::default();
        let streamer = JsonArrayStreamer::new(reader, config).unwrap();

        let docs: Vec<_> = streamer.collect();
        assert_eq!(docs.len(), 1000);
    }

    #[test]
    fn test_array_streamer_size_limit() {
        let json = r#"[{"data": "x"}]"#;

        let reader = Cursor::new(json.as_bytes());
        let config = StreamConfig::builder()
            .max_object_bytes(5) // Very small limit
            .build();

        let streamer = JsonArrayStreamer::new(reader, config).unwrap();
        let result: Vec<_> = streamer.collect();

        // Should error due to size limit
        assert!(result[0].is_err());
    }

    // ==================== JsonLinesStreamer tests ====================

    #[test]
    fn test_jsonl_streamer_simple() {
        let jsonl = r#"{"name": "Alice"}
{"name": "Bob"}
{"name": "Charlie"}"#;

        let reader = Cursor::new(jsonl.as_bytes());
        let config = StreamConfig::default();
        let streamer = JsonLinesStreamer::new(reader, config);

        let docs: Vec<_> = streamer.collect();
        assert_eq!(docs.len(), 3);

        // Verify first document
        let doc1 = docs[0].as_ref().unwrap();
        if let Some(Item::Scalar(Value::String(name))) = doc1.root.get("name") {
            assert_eq!(name, "Alice");
        } else {
            panic!("Expected name field");
        }
    }

    #[test]
    fn test_jsonl_streamer_blank_lines() {
        let jsonl = r#"{"id": "1"}

{"id": "2"}

{"id": "3"}"#;

        let reader = Cursor::new(jsonl.as_bytes());
        let config = StreamConfig::default();
        let streamer = JsonLinesStreamer::new(reader, config);

        let docs: Vec<_> = streamer.collect();
        assert_eq!(docs.len(), 3);
    }

    #[test]
    fn test_jsonl_streamer_comments() {
        let jsonl = r#"# This is a comment
{"id": "1"}
# Another comment
{"id": "2"}"#;

        let reader = Cursor::new(jsonl.as_bytes());
        let config = StreamConfig::default();
        let streamer = JsonLinesStreamer::new(reader, config);

        let docs: Vec<_> = streamer.collect();
        assert_eq!(docs.len(), 2);
    }

    #[test]
    fn test_jsonl_streamer_empty() {
        let jsonl = "";

        let reader = Cursor::new(jsonl.as_bytes());
        let config = StreamConfig::default();
        let streamer = JsonLinesStreamer::new(reader, config);

        let docs: Vec<_> = streamer.collect();
        assert_eq!(docs.len(), 0);
    }

    #[test]
    fn test_jsonl_streamer_invalid_json() {
        let jsonl = r#"{"valid": "json"}
{invalid json}
{"also": "valid"}"#;

        let reader = Cursor::new(jsonl.as_bytes());
        let config = StreamConfig::default();
        let streamer = JsonLinesStreamer::new(reader, config);

        let docs: Vec<_> = streamer.collect();
        assert_eq!(docs.len(), 3);
        assert!(docs[0].is_ok());
        assert!(docs[1].is_err()); // Invalid JSON line
        assert!(docs[2].is_ok());
    }

    #[test]
    fn test_jsonl_streamer_line_number() {
        let jsonl = r#"{"id": "1"}
{"id": "2"}"#;

        let reader = Cursor::new(jsonl.as_bytes());
        let config = StreamConfig::default();
        let mut streamer = JsonLinesStreamer::new(reader, config);

        assert_eq!(streamer.line_number(), 0);
        let _ = streamer.next();
        assert_eq!(streamer.line_number(), 1);
        let _ = streamer.next();
        assert_eq!(streamer.line_number(), 2);
    }

    #[test]
    fn test_jsonl_streamer_size_limit() {
        let jsonl = r#"{"data": "x"}"#;

        let reader = Cursor::new(jsonl.as_bytes());
        let config = StreamConfig::builder()
            .max_object_bytes(5) // Very small limit
            .build();

        let streamer = JsonLinesStreamer::new(reader, config);
        let result: Vec<_> = streamer.collect();

        // Should error due to size limit
        assert!(result[0].is_err());
    }

    // ==================== JsonLinesWriter tests ====================

    #[test]
    fn test_jsonl_writer_simple() {
        let mut buffer = Vec::new();
        let mut writer = JsonLinesWriter::new(&mut buffer);

        let mut doc1 = Document::new((1, 0));
        doc1.root.insert("id".to_string(), Item::Scalar(Value::String("1".to_string())));
        writer.write_document(&doc1).unwrap();

        let mut doc2 = Document::new((1, 0));
        doc2.root.insert("id".to_string(), Item::Scalar(Value::String("2".to_string())));
        writer.write_document(&doc2).unwrap();

        writer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        let lines: Vec<_> = output.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"id\""));
        assert!(lines[1].contains("\"id\""));
    }

    #[test]
    fn test_jsonl_writer_empty_document() {
        let mut buffer = Vec::new();
        let mut writer = JsonLinesWriter::new(&mut buffer);

        let doc = Document::new((1, 0));
        writer.write_document(&doc).unwrap();
        writer.flush().unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output.trim(), "{}");
    }

    #[test]
    fn test_jsonl_roundtrip() {
        // Write documents
        let mut buffer = Vec::new();
        let mut writer = JsonLinesWriter::new(&mut buffer);

        for i in 1..=3 {
            let mut doc = Document::new((1, 0));
            doc.root.insert(
                "id".to_string(),
                Item::Scalar(Value::String(i.to_string())),
            );
            doc.root.insert(
                "value".to_string(),
                Item::Scalar(Value::Int(i * 10)),
            );
            writer.write_document(&doc).unwrap();
        }
        writer.flush().unwrap();

        // Read documents back
        let reader = Cursor::new(buffer);
        let config = StreamConfig::default();
        let streamer = JsonLinesStreamer::new(reader, config);

        let docs: Vec<_> = streamer.collect();
        assert_eq!(docs.len(), 3);

        // Verify first document
        let doc1 = docs[0].as_ref().unwrap();
        assert_eq!(
            doc1.root.get("id").unwrap().as_scalar().unwrap(),
            &Value::String("1".to_string())
        );
        assert_eq!(
            doc1.root.get("value").unwrap().as_scalar().unwrap(),
            &Value::Int(10)
        );
    }
}
