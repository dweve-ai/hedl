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

//! Async streaming parser implementation.
//!
//! This module provides an asynchronous streaming parser for HEDL documents that mirrors
//! the synchronous [`StreamingParser`](crate::StreamingParser) but uses tokio's async I/O.
//!
//! # When to Use Async
//!
//! **Choose Async (`AsyncStreamingParser`) when:**
//! - Parsing network streams or remote data sources
//! - High-concurrency scenarios (thousands of concurrent parsers)
//! - Integration with async web frameworks (axum, actix-web, etc.)
//! - Need to parse multiple streams concurrently
//! - Working in an async runtime context
//!
//! **Choose Sync (`StreamingParser`) when:**
//! - Parsing local files
//! - Single-threaded batch processing
//! - Simpler synchronous code is preferred
//! - Performance is critical and no I/O waiting occurs
//!
//! # Performance Characteristics
//!
//! - **Non-blocking I/O**: Yields to runtime when waiting for data
//! - **Same Memory Profile**: Identical to sync parser (~constant memory)
//! - **Concurrent Processing**: Can process many streams simultaneously
//! - **Zero-Copy**: Minimal allocations, same as sync version
//!
//! # Examples
//!
//! ## Basic Async Streaming
//!
//! ```rust,no_run
//! # #[cfg(feature = "async")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use hedl_stream::{AsyncStreamingParser, NodeEvent};
//! use tokio::fs::File;
//!
//! let file = File::open("large-dataset.hedl").await?;
//! let mut parser = AsyncStreamingParser::new(file).await?;
//!
//! while let Some(event) = parser.next_event().await? {
//!     match event {
//!         NodeEvent::Node(node) => {
//!             println!("{}:{}", node.type_name, node.id);
//!         }
//!         NodeEvent::ListStart { type_name, .. } => {
//!             println!("List started: {}", type_name);
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Concurrent Processing
//!
//! ```rust,no_run
//! # #[cfg(feature = "async")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use hedl_stream::{AsyncStreamingParser, NodeEvent};
//! use tokio::fs::File;
//!
//! async fn process_file(path: &str) -> Result<usize, Box<dyn std::error::Error>> {
//!     let file = File::open(path).await?;
//!     let mut parser = AsyncStreamingParser::new(file).await?;
//!
//!     let mut count = 0;
//!     while let Some(event) = parser.next_event().await? {
//!         if let NodeEvent::Node(_) = event {
//!             count += 1;
//!         }
//!     }
//!     Ok(count)
//! }
//!
//! // Process multiple files concurrently
//! let results = tokio::join!(
//!     process_file("file1.hedl"),
//!     process_file("file2.hedl"),
//!     process_file("file3.hedl"),
//! );
//! # Ok(())
//! # }
//! ```

mod header_parsing;
mod line_parsing;

use crate::async_reader::AsyncLineReader;
use crate::error::{StreamError, StreamResult};
use crate::event::{HeaderInfo, NodeEvent, NodeInfo};
use crate::parser::StreamingParserConfig;
use hedl_core::Value;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use std::time::Instant;
use tokio::io::AsyncRead;

/// Type alias for list context lookup result: (`type_name`, schema, optional `last_node` info)
type ListContextResult = (String, Vec<String>, Option<(String, String)>);

/// Async streaming HEDL parser.
///
/// Processes HEDL documents asynchronously, yielding `NodeEvent` items as they
/// are parsed without loading the entire document into memory. Uses tokio's
/// async I/O for non-blocking operation.
///
/// # Memory Characteristics
///
/// - **Header**: Parsed once at initialization and kept in memory
/// - **Per-Line**: Only current line and parsing context (stack depth proportional to nesting)
/// - **No Buffering**: Nodes are yielded immediately after parsing
/// - **Identical to Sync**: Same memory profile as synchronous parser
///
/// # Examples
///
/// ## Parse from Async File
///
/// ```rust,no_run
/// # #[cfg(feature = "async")]
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use hedl_stream::{AsyncStreamingParser, NodeEvent};
/// use tokio::fs::File;
///
/// let file = File::open("data.hedl").await?;
/// let mut parser = AsyncStreamingParser::new(file).await?;
///
/// while let Some(event) = parser.next_event().await? {
///     if let NodeEvent::Node(node) = event {
///         println!("Processing {}: {}", node.type_name, node.id);
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## With Timeout Protection
///
/// ```rust,no_run
/// # #[cfg(feature = "async")]
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use hedl_stream::{AsyncStreamingParser, StreamingParserConfig, StreamError};
/// use std::time::Duration;
/// use std::io::Cursor;
///
/// let config = StreamingParserConfig {
///     timeout: Some(Duration::from_secs(10)),
///     ..Default::default()
/// };
///
/// let mut parser = AsyncStreamingParser::with_config(
///     Cursor::new("untrusted input"),
///     config
/// ).await?;
///
/// while let Some(event) = parser.next_event().await? {
///     // Process event
/// }
/// # Ok(())
/// # }
/// ```
pub struct AsyncStreamingParser<R: AsyncRead + Unpin> {
    reader: AsyncLineReader<R>,
    config: StreamingParserConfig,
    header: Option<HeaderInfo>,
    state: ParserState,
    finished: bool,
    errored: bool,              // Track if an error occurred to skip finalize
    sent_end_of_document: bool, // Track if EndOfDocument has been returned
    start_time: Instant,
    operations_count: usize,
}

#[derive(Debug)]
struct ParserState {
    /// Stack of active contexts.
    stack: Vec<Context>,
    /// Previous row values for ditto handling (deprecated in v2.0+).
    prev_row: Option<Vec<Value>>,
    /// Pending events from inline children parsing.
    pending_events: Vec<NodeEvent>,
}

#[derive(Debug, Clone)]
enum Context {
    Root,
    Object {
        key: String,
        indent: usize,
    },
    List {
        key: String,
        type_name: String,
        schema: Vec<String>,
        row_indent: usize,
        count: usize,
        last_node: Option<(String, String)>, // (type, id)
    },
}

impl<R: AsyncRead + Unpin> AsyncStreamingParser<R> {
    /// Create a new async streaming parser with default configuration.
    ///
    /// The parser immediately reads and validates the HEDL header (version and
    /// schema directives). If the header is invalid, this function returns an error.
    ///
    /// # Parameters
    ///
    /// - `reader`: Any type implementing `AsyncRead + Unpin`
    ///
    /// # Returns
    ///
    /// - `Ok(parser)`: Parser ready to yield events
    /// - `Err(e)`: Header parsing failed (missing version, invalid schema, etc.)
    ///
    /// # Examples
    ///
    /// ## From a File
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::AsyncStreamingParser;
    /// use tokio::fs::File;
    ///
    /// let file = File::open("data.hedl").await?;
    /// let parser = AsyncStreamingParser::new(file).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## From a String
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::AsyncStreamingParser;
    /// use std::io::Cursor;
    ///
    /// let data = r#"
    /// %VERSION: 1.0
    /// %STRUCT: User: [id, name]
    /// ---
    /// users:@User
    ///  | alice, Alice
    /// "#;
    ///
    /// let parser = AsyncStreamingParser::new(Cursor::new(data)).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - `StreamError::MissingVersion`: No `%VERSION` directive found
    /// - `StreamError::InvalidVersion`: Invalid version format
    /// - `StreamError::Syntax`: Malformed header directive
    /// - `StreamError::Io`: I/O error reading input
    pub async fn new(reader: R) -> StreamResult<Self> {
        Self::with_config(reader, StreamingParserConfig::default()).await
    }

    /// Create an async streaming parser with custom configuration.
    ///
    /// Use this when you need to control memory limits, buffer sizes, or enable
    /// timeout protection for untrusted input.
    ///
    /// # Examples
    ///
    /// ## With Timeout
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::{AsyncStreamingParser, StreamingParserConfig};
    /// use std::time::Duration;
    /// use std::io::Cursor;
    ///
    /// let config = StreamingParserConfig {
    ///     timeout: Some(Duration::from_secs(30)),
    ///     ..Default::default()
    /// };
    ///
    /// let parser = AsyncStreamingParser::with_config(
    ///     Cursor::new("untrusted input"),
    ///     config
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_config(reader: R, config: StreamingParserConfig) -> StreamResult<Self> {
        let mut parser = Self {
            reader: AsyncLineReader::with_capacity(reader, config.buffer_size),
            config,
            header: None,
            state: ParserState {
                stack: vec![Context::Root],
                prev_row: None,
                pending_events: Vec::new(),
            },
            finished: false,
            errored: false,
            sent_end_of_document: false,
            start_time: Instant::now(),
            operations_count: 0,
        };

        // Parse header immediately
        parser.parse_header().await?;

        Ok(parser)
    }

    /// Check if timeout has been exceeded.
    #[inline]
    fn check_timeout(&self) -> StreamResult<()> {
        if let Some(timeout) = self.config.timeout {
            let elapsed = self.start_time.elapsed();
            if elapsed > timeout {
                return Err(StreamError::Timeout {
                    elapsed,
                    limit: timeout,
                });
            }
        }
        Ok(())
    }

    /// Set the errored flag and return an error.
    ///
    /// This helper ensures that after any error is returned, subsequent calls
    /// to `next_event` will return `Ok(None)` without attempting further parsing.
    #[inline]
    fn return_error<T>(&mut self, e: StreamError) -> StreamResult<T> {
        self.finished = true;
        self.errored = true;
        Err(e)
    }

    /// Get the parsed header information.
    ///
    /// Returns header metadata including version, schema definitions, aliases,
    /// and nesting rules. This is available immediately after parser creation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::AsyncStreamingParser;
    /// use std::io::Cursor;
    ///
    /// let input = r#"
    /// %VERSION: 1.0
    /// %STRUCT: User: [id, name, email]
    /// ---
    /// "#;
    ///
    /// let parser = AsyncStreamingParser::new(Cursor::new(input)).await?;
    /// let header = parser.header().unwrap();
    ///
    /// assert_eq!(header.version, (1, 0));
    /// let user_schema = header.get_schema("User").unwrap();
    /// assert_eq!(user_schema, &vec!["id", "name", "email"]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn header(&self) -> Option<&HeaderInfo> {
        self.header.as_ref()
    }

    /// Parse the next event from the stream asynchronously.
    ///
    /// Returns `Ok(Some(event))` if an event was parsed, `Ok(None)` at end of document,
    /// or `Err` on parsing errors.
    ///
    /// # Performance
    ///
    /// This method is async and will yield to the tokio runtime when waiting for I/O,
    /// allowing other tasks to run. It does not block the thread.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::{AsyncStreamingParser, NodeEvent};
    /// use std::io::Cursor;
    ///
    /// let input = r#"
    /// %VERSION: 1.0
    /// %STRUCT: User: [id, name]
    /// ---
    /// users:@User
    ///  | alice, Alice
    /// "#;
    ///
    /// let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await?;
    ///
    /// while let Some(event) = parser.next_event().await? {
    ///     match event {
    ///         NodeEvent::Node(node) => println!("Node: {}", node.id),
    ///         NodeEvent::ListStart { type_name, .. } => println!("List: {}", type_name),
    ///         _ => {}
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn next_event(&mut self) -> StreamResult<Option<NodeEvent>> {
        // If errored, stop immediately without finalize
        if self.errored {
            return Ok(None);
        }

        // Drain pending events from inline children first
        if !self.state.pending_events.is_empty() {
            return Ok(Some(self.state.pending_events.remove(0)));
        }

        // If finished, continue emitting remaining context ends until stack is empty
        if self.finished {
            return self.finalize();
        }

        loop {
            // Check timeout periodically (every 100 operations to minimize overhead)
            self.operations_count += 1;
            if self.operations_count % 100 == 0 {
                if let Err(e) = self.check_timeout() {
                    return self.return_error(e);
                }
            }

            let (line_num, line) = match self.reader.next_line().await {
                Ok(Some(l)) => l,
                Ok(None) => {
                    self.finished = true;
                    return self.finalize();
                }
                Err(e) => return self.return_error(e),
            };

            let trimmed = line.trim();

            // Skip blank lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Calculate indentation
            let indent_info = match hedl_core::lex::calculate_indent(&line, line_num as u32) {
                Ok(info) => info,
                Err(e) => return self.return_error(StreamError::syntax(line_num, e.to_string())),
            };

            let (indent, content) = match indent_info {
                Some(info) => (info.level, &line[info.spaces..]),
                None => continue,
            };

            if indent > self.config.max_indent_depth {
                return self.return_error(StreamError::syntax(
                    line_num,
                    format!("indent depth {indent} exceeds limit"),
                ));
            }

            // Pop contexts as needed based on indentation
            let events = match self.pop_contexts(indent) {
                Ok(e) => e,
                Err(e) => return self.return_error(e),
            };
            if let Some(event) = events {
                // Push back the current line to process after emitting list end
                self.reader.push_back(line_num, line);
                return Ok(Some(event));
            }

            // Parse line content
            return match self.parse_line(content, indent, line_num) {
                Ok(result) => Ok(result),
                Err(e) => self.return_error(e),
            };
        }
    }

    fn finalize(&mut self) -> StreamResult<Option<NodeEvent>> {
        // If we already sent EndOfDocument, return None to signal true end of stream
        if self.sent_end_of_document {
            return Ok(None);
        }

        while self.state.stack.len() > 1 {
            let ctx = self.state.stack.pop().expect("stack has elements");
            match ctx {
                Context::List {
                    key,
                    type_name,
                    count,
                    ..
                } => {
                    return Ok(Some(NodeEvent::ListEnd {
                        key,
                        type_name,
                        count,
                    }));
                }
                Context::Object { key, .. } => {
                    return Ok(Some(NodeEvent::ObjectEnd { key }));
                }
                Context::Root => {
                    // Root context handled by the while condition
                }
            }
        }

        // Mark that we've sent EndOfDocument, so subsequent calls return None
        self.sent_end_of_document = true;
        Ok(Some(NodeEvent::EndOfDocument))
    }

    /// Read up to `n` events in a single async operation.
    ///
    /// Reduces await overhead for high-throughput scenarios by batching event reads.
    /// This can improve performance when processing many small events.
    ///
    /// # Parameters
    ///
    /// - `n`: Maximum number of events to read
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<NodeEvent>)`: Vector of events (may be fewer than `n` if EOF reached)
    /// - `Err(e)`: Parsing error encountered
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::AsyncStreamingParser;
    /// use tokio::fs::File;
    ///
    /// let file = File::open("data.hedl").await?;
    /// let mut parser = AsyncStreamingParser::new(file).await?;
    ///
    /// // Read events in batches of 100
    /// loop {
    ///     let batch = parser.next_batch(100).await?;
    ///     if batch.is_empty() {
    ///         break;
    ///     }
    ///
    ///     // Process batch
    ///     for event in batch {
    ///         // ...
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn next_batch(&mut self, n: usize) -> StreamResult<Vec<NodeEvent>> {
        let mut batch = Vec::with_capacity(n.min(100)); // Cap initial allocation
        for _ in 0..n {
            match self.next_event().await? {
                Some(NodeEvent::EndOfDocument) => break,
                Some(event) => batch.push(event),
                None => break,
            }
        }
        Ok(batch)
    }

    /// Read events with cancellation support via tokio watch channel.
    ///
    /// Returns `Ok(None)` if cancelled, otherwise behaves like `next_event()`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[cfg(feature = "async")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use hedl_stream::AsyncStreamingParser;
    /// use tokio::sync::watch;
    /// use std::io::Cursor;
    ///
    /// let input = r#"
    /// %VERSION: 1.0
    /// %STRUCT: User: [id, name]
    /// ---
    /// users:@User
    ///  | alice, Alice
    /// "#;
    ///
    /// let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await?;
    /// let (cancel_tx, mut cancel_rx) = watch::channel(false);
    ///
    /// // Can cancel from another task
    /// tokio::spawn(async move {
    ///     tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    ///     let _ = cancel_tx.send(true);
    /// });
    ///
    /// while let Some(event) = parser.next_event_cancellable(&mut cancel_rx).await? {
    ///     // Process event
    ///     # break;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "async")]
    pub async fn next_event_cancellable(
        &mut self,
        cancel_rx: &mut tokio::sync::watch::Receiver<bool>,
    ) -> StreamResult<Option<NodeEvent>> {
        // Check if cancelled
        if *cancel_rx.borrow() {
            return Ok(None);
        }

        tokio::select! {
            result = self.next_event() => result,
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() {
                    Ok(None)
                } else {
                    // False alarm, continue
                    self.next_event().await
                }
            }
        }
    }
}

// Stream trait implementation for futures ecosystem integration
#[cfg(feature = "async")]
impl<R: AsyncRead + Unpin> futures_core::Stream for AsyncStreamingParser<R> {
    type Item = StreamResult<NodeEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        // Create a future from next_event and poll it
        let fut = self.next_event();
        tokio::pin!(fut);

        match fut.poll(cx) {
            Poll::Ready(Ok(Some(NodeEvent::EndOfDocument))) => Poll::Ready(None),
            Poll::Ready(Ok(Some(event))) => Poll::Ready(Some(Ok(event))),
            Poll::Ready(Ok(None)) => Poll::Ready(None),
            // Note: errored flag is set inside next_event before returning errors
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(all(test, feature = "async"))]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::time::Duration;

    #[tokio::test]
    async fn test_parse_header() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
%ALIAS active = "Active"
%NEST: User > Order
---
"#;
        let parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();
        let header = parser.header().unwrap();

        assert_eq!(header.version, (1, 0));
        assert!(header.structs.contains_key("User"));
        assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
        assert_eq!(header.nests.get("User"), Some(&vec!["Order".to_string()]));
    }

    #[tokio::test]
    async fn test_streaming_nodes() {
        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice Smith
 | bob, Bob Jones
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let mut events = Vec::new();
        while let Some(event) = parser.next_event().await.unwrap() {
            events.push(event);
        }

        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, "alice");
        assert_eq!(nodes[1].id, "bob");
    }

    #[tokio::test]
    async fn test_timeout() {
        // Test that parsing completes successfully with a reasonable timeout.
        // Using 100ms to avoid flakiness on slow systems while still testing timeout config.
        let config = StreamingParserConfig {
            timeout: Some(Duration::from_millis(100)),
            ..Default::default()
        };

        let input = r"
%VERSION: 1.0
---
";
        let parser = AsyncStreamingParser::with_config(Cursor::new(input), config).await;
        assert!(parser.is_ok()); // Header should parse within timeout
    }

    #[tokio::test]
    async fn test_inline_schema() {
        let input = r"
%VERSION: 1.0
---
items:@Item[id, name]
 | item1, First
 | item2, Second
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let mut nodes = Vec::new();
        while let Some(event) = parser.next_event().await.unwrap() {
            if let NodeEvent::Node(node) = event {
                nodes.push(node);
            }
        }

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].type_name, "Item");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let input = r"
%VERSION: 1.0
---
invalid line without colon
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let result = parser.next_event().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StreamError::Syntax { .. }));
    }

    #[tokio::test]
    async fn test_unicode() {
        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | 用户1, 张三
 | пользователь, Иван
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let mut nodes = Vec::new();
        while let Some(event) = parser.next_event().await.unwrap() {
            if let NodeEvent::Node(node) = event {
                nodes.push(node);
            }
        }

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, "用户1");
        assert_eq!(nodes[1].id, "пользователь");
    }

    // ============ STREAM TRAIT TESTS ============

    #[tokio::test]
    async fn test_stream_trait_basic() {
        use futures::StreamExt;

        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
 | bob, Bob
";
        let parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let events: Vec<_> = parser.collect().await;
        assert!(events.iter().all(std::result::Result::is_ok));

        let nodes: Vec<_> = events
            .iter()
            .filter_map(|e| e.as_ref().ok())
            .filter_map(|e| e.as_node())
            .collect();

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, "alice");
        assert_eq!(nodes[1].id, "bob");
    }

    #[tokio::test]
    async fn test_stream_trait_filter_map() {
        use futures::StreamExt;

        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name, active]
---
users:@User
 | alice, Alice, true
 | bob, Bob, false
 | carol, Carol, true
";
        let parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        // Only collect active users using stream combinators
        let active_nodes: Vec<_> = parser
            .filter_map(|result| async move {
                result.ok().and_then(|event| {
                    if let NodeEvent::Node(node) = event {
                        Some(node)
                    } else {
                        None
                    }
                })
            })
            .filter(|node| {
                let is_active = matches!(node.get_field(2), Some(Value::Bool(true)));
                async move { is_active }
            })
            .collect()
            .await;

        assert_eq!(active_nodes.len(), 2);
        assert_eq!(active_nodes[0].id, "alice");
        assert_eq!(active_nodes[1].id, "carol");
    }

    #[tokio::test]
    async fn test_stream_trait_take() {
        use futures::StreamExt;

        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
 | bob, Bob
 | carol, Carol
 | dave, Dave
";
        let parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        // Take only first 2 node events
        let nodes: Vec<_> = parser
            .filter_map(|result| async move {
                result.ok().and_then(|event| {
                    if let NodeEvent::Node(node) = event {
                        Some(node)
                    } else {
                        None
                    }
                })
            })
            .take(2)
            .collect()
            .await;

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, "alice");
        assert_eq!(nodes[1].id, "bob");
    }

    #[tokio::test]
    async fn test_stream_trait_count() {
        use futures::StreamExt;

        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
 | bob, Bob
 | carol, Carol
";
        let parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let total = parser.count().await;
        // Should count all events: ListStart, 3 Nodes, ListEnd = 5 events
        assert_eq!(total, 5);
    }

    // ============ BATCH READING TESTS ============

    #[tokio::test]
    async fn test_next_batch_basic() {
        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
 | bob, Bob
 | carol, Carol
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        // Read all events in one batch
        let batch = parser.next_batch(10).await.unwrap();
        assert_eq!(batch.len(), 5); // ListStart, 3 Nodes, ListEnd

        // Next batch should be empty (EOF)
        let batch = parser.next_batch(10).await.unwrap();
        assert!(batch.is_empty());
    }

    #[tokio::test]
    async fn test_next_batch_incremental() {
        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
 | bob, Bob
 | carol, Carol
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        // Read in small batches
        let batch1 = parser.next_batch(2).await.unwrap();
        assert_eq!(batch1.len(), 2); // ListStart, Node

        let batch2 = parser.next_batch(2).await.unwrap();
        assert_eq!(batch2.len(), 2); // Node, Node

        let batch3 = parser.next_batch(2).await.unwrap();
        assert_eq!(batch3.len(), 1); // ListEnd

        let batch4 = parser.next_batch(2).await.unwrap();
        assert!(batch4.is_empty());
    }

    #[tokio::test]
    async fn test_next_batch_empty_file() {
        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let batch = parser.next_batch(10).await.unwrap();
        assert!(batch.is_empty());
    }

    #[tokio::test]
    async fn test_next_batch_large() {
        let mut input = String::from(
            r"
%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data:@Data
",
        );
        for i in 0..500 {
            input.push_str(&format!(" | row{i}, value{i}\n"));
        }

        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        // Read in large batches
        let batch1 = parser.next_batch(100).await.unwrap();
        assert_eq!(batch1.len(), 100); // ListStart + 99 Nodes

        let batch2 = parser.next_batch(100).await.unwrap();
        assert_eq!(batch2.len(), 100); // 100 Nodes

        // Continue until we get all events
        let mut total = batch1.len() + batch2.len();
        loop {
            let batch = parser.next_batch(100).await.unwrap();
            if batch.is_empty() {
                break;
            }
            total += batch.len();
        }

        // Total: ListStart + 500 Nodes + ListEnd = 502
        assert_eq!(total, 502);
    }

    // ============ CANCELLATION TESTS ============

    #[tokio::test]
    async fn test_cancellation_basic() {
        use tokio::sync::watch;

        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
 | bob, Bob
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let (cancel_tx, mut cancel_rx) = watch::channel(false);

        // Read first event normally
        let event1 = parser.next_event_cancellable(&mut cancel_rx).await.unwrap();
        assert!(event1.is_some());

        // Cancel
        cancel_tx.send(true).unwrap();

        // Next read should return None (cancelled)
        let event2 = parser.next_event_cancellable(&mut cancel_rx).await.unwrap();
        assert!(event2.is_none());
    }

    #[tokio::test]
    async fn test_cancellation_not_cancelled() {
        use tokio::sync::watch;

        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let (_cancel_tx, mut cancel_rx) = watch::channel(false);

        // Read all events without cancellation
        let mut count = 0;
        while let Some(_event) = parser.next_event_cancellable(&mut cancel_rx).await.unwrap() {
            count += 1;
        }

        // Should have read all events: ListStart, Node, ListEnd, EndOfDocument
        assert_eq!(count, 4);
    }

    #[tokio::test]
    async fn test_cancellation_during_processing() {
        use tokio::sync::watch;

        let mut input = String::from(
            r"
%VERSION: 1.0
%STRUCT: Data: [id]
---
data:@Data
",
        );
        for i in 0..1000 {
            input.push_str(&format!(" | row{i}\n"));
        }

        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let (cancel_tx, mut cancel_rx) = watch::channel(false);

        // Spawn a task that cancels after reading some events
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            cancel_tx.send(true).unwrap();
        });

        let mut count = 0;
        while let Some(_event) = parser.next_event_cancellable(&mut cancel_rx).await.unwrap() {
            count += 1;
            // Small delay to allow cancellation to trigger
            tokio::time::sleep(tokio::time::Duration::from_micros(10)).await;
        }

        // Should have read some events but not all 1002 (ListStart + 1000 Nodes + ListEnd)
        assert!(count < 1002);
        assert!(count > 0);
    }

    // ============ CONCURRENT PROCESSING TESTS ============

    #[tokio::test]
    async fn test_concurrent_file_processing() {
        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
 | bob, Bob
";

        // Process multiple identical streams concurrently
        let tasks: Vec<_> = (0..5)
            .map(|_| {
                let input_clone = input.to_string();
                tokio::spawn(async move {
                    let mut parser = AsyncStreamingParser::new(Cursor::new(input_clone))
                        .await
                        .unwrap();

                    let mut count = 0;
                    while let Some(_event) = parser.next_event().await.unwrap() {
                        count += 1;
                    }
                    count
                })
            })
            .collect();

        let results = futures::future::join_all(tasks).await;

        // All tasks should succeed and count the same number of events
        for result in results {
            assert_eq!(result.unwrap(), 5); // ListStart, 2 Nodes, ListEnd, EndOfDocument
        }
    }

    #[tokio::test]
    async fn test_concurrent_with_stream_trait() {
        use futures::StreamExt;

        let input = r"
%VERSION: 1.0
%STRUCT: Data: [id]
---
data:@Data
 | row1
 | row2
 | row3
";

        // Process multiple streams concurrently using manual await
        // Note: futures::Stream combinators create !Send futures, so we can't use tokio::spawn
        let mut counts = Vec::new();

        for _ in 0..10 {
            let parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

            // Count nodes using stream combinators
            let count = parser
                .filter_map(|result| async move {
                    result.ok().and_then(|event| {
                        if let NodeEvent::Node(_) = event {
                            Some(())
                        } else {
                            None
                        }
                    })
                })
                .count()
                .await;

            counts.push(count);
        }

        // All should count 3 nodes
        for count in counts {
            assert_eq!(count, 3);
        }
    }

    // ============ EDGE CASE AND INTEGRATION TESTS ============

    #[tokio::test]
    async fn test_stream_trait_with_errors() {
        use futures::StreamExt;

        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
 | bob
 | carol, Carol
";
        let parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let results: Vec<_> = parser.collect().await;

        // Should have error for malformed row (bob with only 1 field)
        let errors: Vec<_> = results.iter().filter(|r| r.is_err()).collect();
        assert!(!errors.is_empty());
    }

    #[tokio::test]
    async fn test_batch_with_mixed_events() {
        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Product: [id, title]
---
users:@User
 | alice, Alice
products:@Product
 | prod1, Widget
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let batch = parser.next_batch(10).await.unwrap();

        // Should contain: ListStart(User), Node(alice), ListEnd(User), ListStart(Product), Node(prod1), ListEnd(Product)
        assert_eq!(batch.len(), 6);

        let list_starts: Vec<_> = batch
            .iter()
            .filter(|e| matches!(e, NodeEvent::ListStart { .. }))
            .collect();
        assert_eq!(list_starts.len(), 2);
    }

    #[tokio::test]
    async fn test_stream_empty_after_cancellation() {
        use tokio::sync::watch;

        let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
 | bob, Bob
";
        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let (cancel_tx, mut cancel_rx) = watch::channel(false);

        // Read one event
        let _event = parser.next_event_cancellable(&mut cancel_rx).await.unwrap();

        // Cancel
        cancel_tx.send(true).unwrap();

        // Subsequent reads should return None
        assert!(parser
            .next_event_cancellable(&mut cancel_rx)
            .await
            .unwrap()
            .is_none());
        assert!(parser
            .next_event_cancellable(&mut cancel_rx)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn test_batch_reading_performance() {
        // Create a large dataset
        let mut input = String::from(
            r"
%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data:@Data
",
        );
        for i in 0..1000 {
            input.push_str(&format!(" | row{i}, value{i}\n"));
        }

        let mut parser = AsyncStreamingParser::new(Cursor::new(input)).await.unwrap();

        let start = std::time::Instant::now();

        // Read in batches
        let mut total = 0;
        loop {
            let batch = parser.next_batch(100).await.unwrap();
            if batch.is_empty() {
                break;
            }
            total += batch.len();
        }

        let elapsed = start.elapsed();

        // Should have read all events
        assert_eq!(total, 1002); // ListStart + 1000 Nodes + ListEnd

        // Should complete reasonably quickly (< 100ms for 1000 rows)
        assert!(elapsed.as_millis() < 100);
    }
}
