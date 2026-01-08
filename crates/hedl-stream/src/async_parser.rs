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
//! let parser = AsyncStreamingParser::new(file).await?;
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
//!     let parser = AsyncStreamingParser::new(file).await?;
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

use crate::async_reader::AsyncLineReader;
use crate::error::{StreamError, StreamResult};
use crate::event::{HeaderInfo, NodeEvent, NodeInfo};
use crate::parser::{strip_comment, StreamingParserConfig};
use hedl_core::Value;
use hedl_core::lex::{calculate_indent, is_valid_key_token, is_valid_type_name};
use tokio::io::AsyncRead;
use std::time::Instant;

/// Type alias for list context lookup result: (type_name, schema, optional last_node info)
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
/// let parser = AsyncStreamingParser::new(file).await?;
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
/// let parser = AsyncStreamingParser::with_config(
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
    start_time: Instant,
    operations_count: usize,
}

#[derive(Debug)]
struct ParserState {
    /// Stack of active contexts.
    stack: Vec<Context>,
    /// Previous row values for ditto handling.
    prev_row: Option<Vec<Value>>,
}

#[derive(Debug, Clone)]
enum Context {
    Root,
    Object {
        #[allow(dead_code)]
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
    /// users: @User
    ///   | alice, Alice
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
            },
            finished: false,
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
                return Err(StreamError::Timeout { elapsed, limit: timeout });
            }
        }
        Ok(())
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
    /// users: @User
    ///   | alice, Alice
    /// "#;
    ///
    /// let parser = AsyncStreamingParser::new(Cursor::new(input)).await?;
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
        if self.finished {
            return Ok(None);
        }

        loop {
            // Check timeout periodically (every 100 operations to minimize overhead)
            self.operations_count += 1;
            if self.operations_count % 100 == 0 {
                self.check_timeout()?;
            }

            let (line_num, line) = match self.reader.next_line().await? {
                Some(l) => l,
                None => {
                    self.finished = true;
                    return self.finalize();
                }
            };

            let trimmed = line.trim();

            // Skip blank lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Calculate indentation
            let indent_info = calculate_indent(&line, line_num as u32)
                .map_err(|e| StreamError::syntax(line_num, e.to_string()))?;

            let (indent, content) = match indent_info {
                Some(info) => (info.level, &line[info.spaces..]),
                None => continue,
            };

            if indent > self.config.max_indent_depth {
                return Err(StreamError::syntax(
                    line_num,
                    format!("indent depth {} exceeds limit", indent),
                ));
            }

            // Pop contexts as needed based on indentation
            let events = self.pop_contexts(indent)?;
            if let Some(event) = events {
                // Push back the current line to process after emitting list end
                self.reader.push_back(line_num, line);
                return Ok(Some(event));
            }

            // Parse line content
            return self.parse_line(content, indent, line_num);
        }
    }

    async fn parse_header(&mut self) -> StreamResult<()> {
        let mut header = HeaderInfo::new();
        let mut found_version = false;
        let mut _found_separator = false;

        while let Some((line_num, line)) = self.reader.next_line().await? {
            self.check_timeout()?;

            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed == "---" {
                _found_separator = true;
                break;
            }

            if trimmed.starts_with('%') {
                self.parse_directive(trimmed, line_num, &mut header, &mut found_version)?;
            } else {
                self.reader.push_back(line_num, line);
                break;
            }
        }

        if !found_version {
            return Err(StreamError::MissingVersion);
        }

        self.header = Some(header);
        Ok(())
    }

    fn parse_directive(
        &self,
        line: &str,
        line_num: usize,
        header: &mut HeaderInfo,
        found_version: &mut bool,
    ) -> StreamResult<()> {
        if line.starts_with("%VERSION") {
            self.parse_version_directive(line, header, found_version)
        } else if line.starts_with("%STRUCT") {
            self.parse_struct_directive(line, line_num, header)
        } else if line.starts_with("%ALIAS") {
            self.parse_alias_directive(line, line_num, header)
        } else if line.starts_with("%NEST") {
            self.parse_nest_directive(line, line_num, header)
        } else {
            Ok(())
        }
    }

    fn parse_version_directive(
        &self,
        line: &str,
        header: &mut HeaderInfo,
        found_version: &mut bool,
    ) -> StreamResult<()> {
        let rest = line.strip_prefix("%VERSION").expect("prefix exists").trim();
        let rest = rest.strip_prefix(':').unwrap_or(rest).trim();
        let parts: Vec<&str> = rest.split('.').collect();

        if parts.len() != 2 {
            return Err(StreamError::InvalidVersion(rest.to_string()));
        }

        let major: u32 = parts[0]
            .parse()
            .map_err(|_| StreamError::InvalidVersion(rest.to_string()))?;
        let minor: u32 = parts[1]
            .parse()
            .map_err(|_| StreamError::InvalidVersion(rest.to_string()))?;

        header.version = (major, minor);
        *found_version = true;
        Ok(())
    }

    fn parse_struct_directive(
        &self,
        line: &str,
        line_num: usize,
        header: &mut HeaderInfo,
    ) -> StreamResult<()> {
        let rest = line.strip_prefix("%STRUCT").expect("prefix exists").trim();
        let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

        let bracket_start = rest
            .find('[')
            .ok_or_else(|| StreamError::syntax(line_num, "missing '[' in %STRUCT"))?;
        let bracket_end = rest
            .find(']')
            .ok_or_else(|| StreamError::syntax(line_num, "missing ']' in %STRUCT"))?;

        let type_part = rest[..bracket_start].trim().trim_end_matches(':').trim();
        let type_name = if let Some(paren_pos) = type_part.find('(') {
            type_part[..paren_pos].trim()
        } else {
            type_part
        };
        if !is_valid_type_name(type_name) {
            return Err(StreamError::syntax(
                line_num,
                format!("invalid type name: {}", type_name),
            ));
        }

        let cols_str = &rest[bracket_start + 1..bracket_end];
        let columns: Vec<String> = cols_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if columns.is_empty() {
            return Err(StreamError::syntax(line_num, "empty schema"));
        }

        header.structs.insert(type_name.to_string(), columns);
        Ok(())
    }

    fn parse_alias_directive(
        &self,
        line: &str,
        line_num: usize,
        header: &mut HeaderInfo,
    ) -> StreamResult<()> {
        let rest = line.strip_prefix("%ALIAS").expect("prefix exists").trim();
        let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

        let sep_pos = rest
            .find('=')
            .or_else(|| rest.find(':'))
            .ok_or_else(|| StreamError::syntax(line_num, "missing '=' or ':' in %ALIAS"))?;

        let alias = rest[..sep_pos].trim();
        let value = rest[sep_pos + 1..].trim().trim_matches('"');

        header.aliases.insert(alias.to_string(), value.to_string());
        Ok(())
    }

    fn parse_nest_directive(
        &self,
        line: &str,
        line_num: usize,
        header: &mut HeaderInfo,
    ) -> StreamResult<()> {
        let rest = line.strip_prefix("%NEST").expect("prefix exists").trim();
        let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

        let arrow_pos = rest
            .find('>')
            .ok_or_else(|| StreamError::syntax(line_num, "missing '>' in %NEST"))?;

        let parent = rest[..arrow_pos].trim();
        let child = rest[arrow_pos + 1..].trim();

        if !is_valid_type_name(parent) || !is_valid_type_name(child) {
            return Err(StreamError::syntax(line_num, "invalid type name in %NEST"));
        }

        header.nests.insert(parent.to_string(), child.to_string());
        Ok(())
    }

    fn pop_contexts(&mut self, current_indent: usize) -> StreamResult<Option<NodeEvent>> {
        while self.state.stack.len() > 1 {
            let should_pop = match self.state.stack.last().expect("stack has elements") {
                Context::Root => false,
                Context::Object { indent, .. } => current_indent <= *indent,
                Context::List { row_indent, .. } => current_indent < *row_indent,
            };

            if should_pop {
                let ctx = self.state.stack.pop().expect("stack has elements");
                if let Context::List {
                    key,
                    type_name,
                    count,
                    ..
                } = ctx
                {
                    return Ok(Some(NodeEvent::ListEnd {
                        key,
                        type_name,
                        count,
                    }));
                }
            } else {
                break;
            }
        }

        Ok(None)
    }

    fn parse_line(
        &mut self,
        content: &str,
        indent: usize,
        line_num: usize,
    ) -> StreamResult<Option<NodeEvent>> {
        let content = strip_comment(content);

        if let Some(row_content) = content.strip_prefix('|') {
            self.parse_matrix_row(row_content, indent, line_num)
        } else if let Some(colon_pos) = content.find(':') {
            let key = content[..colon_pos].trim();
            let after_colon = &content[colon_pos + 1..];

            if !is_valid_key_token(key) {
                return Err(StreamError::syntax(
                    line_num,
                    format!("invalid key: {}", key),
                ));
            }

            let after_colon_trimmed = after_colon.trim();

            if after_colon_trimmed.is_empty() {
                self.state.stack.push(Context::Object {
                    key: key.to_string(),
                    indent,
                });
                Ok(Some(NodeEvent::ObjectStart {
                    key: key.to_string(),
                    line: line_num,
                }))
            } else if after_colon_trimmed.starts_with('@')
                && self.is_list_start(after_colon_trimmed)
            {
                let (type_name, schema) = self.parse_list_start(after_colon_trimmed, line_num)?;

                self.state.stack.push(Context::List {
                    key: key.to_string(),
                    type_name: type_name.clone(),
                    schema: schema.clone(),
                    row_indent: indent + 1,
                    count: 0,
                    last_node: None,
                });

                self.state.prev_row = None;

                Ok(Some(NodeEvent::ListStart {
                    key: key.to_string(),
                    type_name,
                    schema,
                    line: line_num,
                }))
            } else {
                let value = self.infer_value(after_colon.trim(), line_num)?;
                Ok(Some(NodeEvent::Scalar {
                    key: key.to_string(),
                    value,
                    line: line_num,
                }))
            }
        } else {
            Err(StreamError::syntax(line_num, "expected ':' in line"))
        }
    }

    #[inline]
    fn is_list_start(&self, s: &str) -> bool {
        let s = s.trim();
        if !s.starts_with('@') {
            return false;
        }
        let rest = &s[1..];
        let type_end = rest
            .find(|c: char| c == '[' || c.is_whitespace())
            .unwrap_or(rest.len());
        let type_name = &rest[..type_end];
        is_valid_type_name(type_name)
    }

    fn parse_list_start(&self, s: &str, line_num: usize) -> StreamResult<(String, Vec<String>)> {
        let s = s.trim();
        let rest = &s[1..];

        if let Some(bracket_pos) = rest.find('[') {
            let type_name = &rest[..bracket_pos];
            if !is_valid_type_name(type_name) {
                return Err(StreamError::syntax(
                    line_num,
                    format!("invalid type name: {}", type_name),
                ));
            }

            let bracket_end = rest
                .find(']')
                .ok_or_else(|| StreamError::syntax(line_num, "missing ']'"))?;

            let cols_str = &rest[bracket_pos + 1..bracket_end];
            let columns: Vec<String> = cols_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            Ok((type_name.to_string(), columns))
        } else {
            let type_name = rest.trim();
            if !is_valid_type_name(type_name) {
                return Err(StreamError::syntax(
                    line_num,
                    format!("invalid type name: {}", type_name),
                ));
            }

            let header = self
                .header
                .as_ref()
                .ok_or_else(|| StreamError::Header("header not parsed".to_string()))?;

            let schema = header.structs.get(type_name).ok_or_else(|| {
                StreamError::schema(line_num, format!("undefined type: {}", type_name))
            })?;

            Ok((type_name.to_string(), schema.clone()))
        }
    }

    fn parse_matrix_row(
        &mut self,
        content: &str,
        indent: usize,
        line_num: usize,
    ) -> StreamResult<Option<NodeEvent>> {
        let content = strip_comment(content).trim();

        let (type_name, schema, parent_info) = self.find_list_context(indent, line_num)?;

        let fields = hedl_core::lex::parse_csv_row(content)
            .map_err(|e| StreamError::syntax(line_num, format!("row parse error: {}", e)))?;

        if fields.len() != schema.len() {
            return Err(StreamError::ShapeMismatch {
                line: line_num,
                expected: schema.len(),
                got: fields.len(),
            });
        }

        let mut values = Vec::with_capacity(fields.len());
        for (col_idx, field) in fields.iter().enumerate() {
            let value = if field.value == "^" {
                self.state
                    .prev_row
                    .as_ref()
                    .and_then(|prev| prev.get(col_idx).cloned())
                    .unwrap_or(Value::Null)
            } else if field.is_quoted {
                Value::String(field.value.clone())
            } else {
                self.infer_value(&field.value, line_num)?
            };
            values.push(value);
        }

        let id = match &values[0] {
            Value::String(s) => s.clone(),
            _ => return Err(StreamError::syntax(line_num, "ID column must be a string")),
        };

        self.update_list_context(&type_name, &id);
        self.state.prev_row = Some(values.clone());

        let mut node = NodeInfo::new(type_name.clone(), id, values, indent, line_num);

        if let Some((parent_type, parent_id)) = parent_info {
            node = node.with_parent(parent_type, parent_id);
        }

        Ok(Some(NodeEvent::Node(node)))
    }

    fn find_list_context(
        &mut self,
        indent: usize,
        line_num: usize,
    ) -> StreamResult<ListContextResult> {
        let header = self
            .header
            .as_ref()
            .ok_or_else(|| StreamError::Header("header not parsed".to_string()))?;

        for ctx in self.state.stack.iter().rev() {
            if let Context::List {
                type_name,
                schema,
                row_indent,
                last_node,
                ..
            } = ctx
            {
                if indent == *row_indent {
                    return Ok((type_name.clone(), schema.clone(), None));
                } else if indent == *row_indent + 1 {
                    let parent_info = last_node.clone().ok_or_else(|| {
                        StreamError::orphan_row(line_num, "child row has no parent")
                    })?;

                    let child_type = header.nests.get(type_name).ok_or_else(|| {
                        StreamError::orphan_row(
                            line_num,
                            format!("no NEST rule for parent type '{}'", type_name),
                        )
                    })?;

                    let child_schema = header.structs.get(child_type).ok_or_else(|| {
                        StreamError::schema(
                            line_num,
                            format!("child type '{}' not defined", child_type),
                        )
                    })?;

                    self.state.stack.push(Context::List {
                        key: child_type.clone(),
                        type_name: child_type.clone(),
                        schema: child_schema.clone(),
                        row_indent: indent,
                        count: 0,
                        last_node: None,
                    });

                    return Ok((child_type.clone(), child_schema.clone(), Some(parent_info)));
                }
            }
        }

        Err(StreamError::syntax(
            line_num,
            "matrix row outside of list context",
        ))
    }

    fn update_list_context(&mut self, type_name: &str, id: &str) {
        for ctx in self.state.stack.iter_mut().rev() {
            if let Context::List {
                type_name: ctx_type,
                last_node,
                count,
                ..
            } = ctx
            {
                if ctx_type == type_name {
                    *last_node = Some((type_name.to_string(), id.to_string()));
                    *count += 1;
                    break;
                }
            }
        }
    }

    #[inline]
    fn infer_value(&self, s: &str, _line_num: usize) -> StreamResult<Value> {
        let s = s.trim();

        if s.is_empty() || s == "~" {
            return Ok(Value::Null);
        }

        if s == "true" {
            return Ok(Value::Bool(true));
        }
        if s == "false" {
            return Ok(Value::Bool(false));
        }

        if let Some(ref_part) = s.strip_prefix('@') {
            if let Some(colon_pos) = ref_part.find(':') {
                let type_name = &ref_part[..colon_pos];
                let id = &ref_part[colon_pos + 1..];
                return Ok(Value::Reference(hedl_core::Reference {
                    type_name: Some(type_name.to_string()),
                    id: id.to_string(),
                }));
            } else {
                return Ok(Value::Reference(hedl_core::Reference {
                    type_name: None,
                    id: ref_part.to_string(),
                }));
            }
        }

        if let Some(alias) = s.strip_prefix('$') {
            if let Some(header) = &self.header {
                if let Some(value) = header.aliases.get(alias) {
                    return Ok(Value::String(value.clone()));
                }
            }
            return Ok(Value::String(s.to_string()));
        }

        if let Ok(i) = s.parse::<i64>() {
            return Ok(Value::Int(i));
        }
        if let Ok(f) = s.parse::<f64>() {
            return Ok(Value::Float(f));
        }

        Ok(Value::String(s.to_string()))
    }

    fn finalize(&mut self) -> StreamResult<Option<NodeEvent>> {
        while self.state.stack.len() > 1 {
            let ctx = self.state.stack.pop().expect("stack has elements");
            if let Context::List {
                key,
                type_name,
                count,
                ..
            } = ctx
            {
                return Ok(Some(NodeEvent::ListEnd {
                    key,
                    type_name,
                    count,
                }));
            }
        }

        Ok(Some(NodeEvent::EndOfDocument))
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
        let parser = AsyncStreamingParser::new(Cursor::new(input))
            .await
            .unwrap();
        let header = parser.header().unwrap();

        assert_eq!(header.version, (1, 0));
        assert!(header.structs.contains_key("User"));
        assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
        assert_eq!(header.nests.get("User"), Some(&"Order".to_string()));
    }

    #[tokio::test]
    async fn test_streaming_nodes() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice Smith
  | bob, Bob Jones
"#;
        let mut parser = AsyncStreamingParser::new(Cursor::new(input))
            .await
            .unwrap();

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
        // This test would need a custom reader that delays, simplified for now
        let config = StreamingParserConfig {
            timeout: Some(Duration::from_millis(1)),
            ..Default::default()
        };

        let input = r#"
%VERSION: 1.0
---
"#;
        let parser = AsyncStreamingParser::with_config(Cursor::new(input), config).await;
        assert!(parser.is_ok()); // Header should parse within timeout
    }

    #[tokio::test]
    async fn test_inline_schema() {
        let input = r#"
%VERSION: 1.0
---
items: @Item[id, name]
  | item1, First
  | item2, Second
"#;
        let mut parser = AsyncStreamingParser::new(Cursor::new(input))
            .await
            .unwrap();

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
        let input = r#"
%VERSION: 1.0
---
invalid line without colon
"#;
        let mut parser = AsyncStreamingParser::new(Cursor::new(input))
            .await
            .unwrap();

        let result = parser.next_event().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StreamError::Syntax { .. }));
    }

    #[tokio::test]
    async fn test_unicode() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | 用户1, 张三
  | пользователь, Иван
"#;
        let mut parser = AsyncStreamingParser::new(Cursor::new(input))
            .await
            .unwrap();

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
}
