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

//! Core streaming parser implementation

use super::config::StreamingParserConfig;
use super::context::{self, Context, ParserState};
use super::directives;
use super::list_parsing;
use super::value_inference;
use crate::buffer_pool::BufferPool;
use crate::error::{StreamError, StreamResult};
use crate::event::{HeaderInfo, NodeEvent};
use crate::reader::LineReader;
use hedl_core::lex::{calculate_indent, is_valid_key_token, strip_comment};
use std::io::Read;
use std::time::Instant;

/// Streaming HEDL parser.
///
/// Processes HEDL documents incrementally, yielding `NodeEvent` items as they
/// are parsed without loading the entire document into memory.
pub struct StreamingParser<R: Read> {
    reader: LineReader<R>,
    config: StreamingParserConfig,
    header: Option<HeaderInfo>,
    state: ParserState,
    finished: bool,
    errored: bool,              // Track if an error occurred to skip finalize
    sent_end_of_document: bool, // Track if EndOfDocument has been returned
    start_time: Instant,
    operations_count: usize, // Track operations for periodic timeout checks
    // Note: Buffer pool integration deferred - requires refactoring of parse_data_row
    // to support pooled allocations. Current direct allocation pattern is simpler
    // and performs adequately for typical use cases. Pooling would benefit only
    // extremely high-throughput scenarios (>1M rows/sec).
    _buffer_pool: Option<BufferPool>, // Optional buffer pool for high-throughput scenarios (not yet integrated)
}

impl<R: Read> StreamingParser<R> {
    /// Create a new streaming parser with default configuration.
    ///
    /// The parser immediately reads and validates the HEDL header (version and
    /// schema directives). If the header is invalid, this function returns an error.
    ///
    /// # Parameters
    ///
    /// - `reader`: Any type implementing `Read` (files, network streams, buffers, etc.)
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
    /// use hedl_stream::StreamingParser;
    /// use std::fs::File;
    /// use std::io::BufReader;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let file = File::open("data.hedl")?;
    /// let reader = BufReader::new(file);
    /// let parser = StreamingParser::new(reader)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## From a String
    ///
    /// ```rust
    /// use hedl_stream::StreamingParser;
    /// use std::io::Cursor;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let data = r#"
    /// %VERSION: 1.0
    /// %STRUCT: User: [id, name]
    /// ---
    /// users:@User
    ///   | alice, Alice
    /// "#;
    ///
    /// let parser = StreamingParser::new(Cursor::new(data))?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## From Stdin
    ///
    /// ```rust,no_run
    /// use hedl_stream::StreamingParser;
    /// use std::io::stdin;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let parser = StreamingParser::new(stdin().lock())?;
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
    pub fn new(reader: R) -> StreamResult<Self> {
        Self::with_config(reader, StreamingParserConfig::default())
    }

    /// Create a streaming parser with custom configuration.
    ///
    /// Use this when you need to control memory limits, buffer sizes, or enable
    /// timeout protection for untrusted input.
    ///
    /// # Parameters
    ///
    /// - `reader`: Any type implementing `Read`
    /// - `config`: Parser configuration options
    ///
    /// # Returns
    ///
    /// - `Ok(parser)`: Parser ready to yield events
    /// - `Err(e)`: Configuration invalid or header parsing failed
    ///
    /// # Examples
    ///
    /// ## With Timeout Protection
    ///
    /// ```rust
    /// use hedl_stream::{StreamingParser, StreamingParserConfig};
    /// use std::time::Duration;
    /// use std::io::Cursor;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = StreamingParserConfig {
    ///     timeout: Some(Duration::from_secs(30)),
    ///     ..Default::default()
    /// };
    ///
    /// let untrusted_input = "...";
    /// let parser = StreamingParser::with_config(
    ///     Cursor::new(untrusted_input),
    ///     config
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## For Large Files
    ///
    /// ```rust
    /// use hedl_stream::{StreamingParser, StreamingParserConfig};
    /// use std::io::Cursor;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = StreamingParserConfig {
    ///     buffer_size: 256 * 1024,      // 256KB read buffer
    ///     max_line_length: 10_000_000,  // 10MB max line
    ///     max_indent_depth: 1000,       // Deep nesting allowed
    ///     timeout: None,
    ///     ..Default::default()
    /// };
    ///
    /// let parser = StreamingParser::with_config(
    ///     Cursor::new("..."),
    ///     config
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## For Constrained Environments
    ///
    /// ```rust
    /// use hedl_stream::{StreamingParser, StreamingParserConfig};
    /// use std::time::Duration;
    /// use std::io::Cursor;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = StreamingParserConfig {
    ///     buffer_size: 8 * 1024,        // Small 8KB buffer
    ///     max_line_length: 100_000,     // 100KB max line
    ///     max_indent_depth: 50,         // Limited nesting
    ///     timeout: Some(Duration::from_secs(10)),
    ///     ..Default::default()
    /// };
    ///
    /// let parser = StreamingParser::with_config(
    ///     Cursor::new("..."),
    ///     config
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Same as [`new()`](Self::new), plus:
    ///
    /// - `StreamError::Timeout`: Header parsing exceeded configured timeout
    pub fn with_config(reader: R, config: StreamingParserConfig) -> StreamResult<Self> {
        // Initialize buffer pool if enabled
        let buffer_pool = if config.enable_pooling && config.memory_limits.enable_buffer_pooling {
            Some(BufferPool::new(config.memory_limits.max_pool_size))
        } else {
            None
        };

        let mut parser = Self {
            reader: LineReader::with_capacity_and_max_length(
                reader,
                config.buffer_size,
                config.max_line_length,
            ),
            config,
            header: None,
            state: ParserState::default(),
            finished: false,
            errored: false,
            sent_end_of_document: false,
            start_time: Instant::now(),
            operations_count: 0,
            _buffer_pool: buffer_pool,
        };

        // Parse header immediately
        parser.parse_header()?;

        Ok(parser)
    }

    /// Check if timeout has been exceeded.
    /// This is called periodically during parsing to prevent infinite loops.
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

    /// Get the parsed header information.
    ///
    /// Returns header metadata including version, schema definitions, aliases,
    /// and nesting rules. This is available immediately after parser creation.
    ///
    /// # Returns
    ///
    /// - `Some(&HeaderInfo)`: Header was successfully parsed
    /// - `None`: Should never happen after successful parser creation
    ///
    /// # Examples
    ///
    /// ## Inspecting Schema Definitions
    ///
    /// ```rust
    /// use hedl_stream::StreamingParser;
    /// use std::io::Cursor;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let input = r#"
    /// %VERSION: 1.0
    /// %STRUCT: User: [id, name, email]
    /// %STRUCT: Order: [id, user_id, amount]
    /// %ALIAS: active = "Active"
    /// %NEST: User > Order
    /// ---
    /// "#;
    ///
    /// let parser = StreamingParser::new(Cursor::new(input))?;
    /// let header = parser.header().unwrap();
    ///
    /// // Check version
    /// assert_eq!(header.version, (1, 0));
    ///
    /// // Get schema
    /// let user_schema = header.get_schema("User").unwrap();
    /// assert_eq!(user_schema, &vec!["id", "name", "email"]);
    ///
    /// // Check aliases
    /// assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
    ///
    /// // Check nesting rules
    /// assert!(header.get_child_types("User").map_or(false, |v| v.contains(&"Order".to_string())));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Validating Before Processing
    ///
    /// ```rust
    /// use hedl_stream::StreamingParser;
    /// use std::io::Cursor;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let input = r#"
    /// %VERSION: 1.0
    /// %STRUCT: User: [id, name]
    /// ---
    /// users:@User
    ///   | alice, Alice
    /// "#;
    ///
    /// let parser = StreamingParser::new(Cursor::new(input))?;
    ///
    /// // Validate we have the expected schema before processing
    /// if let Some(header) = parser.header() {
    ///     if header.version.0 != 1 {
    ///         eprintln!("Warning: Unexpected major version");
    ///     }
    ///
    ///     if !header.structs.contains_key("User") {
    ///         return Err("Missing User schema".into());
    ///     }
    /// }
    ///
    /// // Proceed with parsing...
    /// # Ok(())
    /// # }
    /// ```
    pub fn header(&self) -> Option<&HeaderInfo> {
        self.header.as_ref()
    }

    /// Parse the header section.
    fn parse_header(&mut self) -> StreamResult<()> {
        let mut header = HeaderInfo::new();
        let mut found_version = false;
        let mut _found_separator = false;

        while let Some((line_num, line)) = self.reader.next_line()? {
            // Check timeout every iteration in header parsing
            self.check_timeout()?;

            let trimmed = line.trim();

            // Skip blank lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for separator
            if trimmed == "---" {
                _found_separator = true;
                break;
            }

            // Parse directives
            if trimmed.starts_with('%') {
                directives::parse_directive(trimmed, line_num, &mut header, &mut found_version)?;
            } else {
                // Not a directive - might be body content without separator
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

    /// Parse the next event from the stream.
    fn next_event(&mut self) -> StreamResult<Option<NodeEvent>> {
        // If errored, stop immediately without finalize
        if self.errored {
            return Ok(None);
        }

        // Drain pending events from inline children first
        if let Some(event) = self.state.pending_events.pop_front() {
            return Ok(Some(event));
        }

        // If finished, continue emitting remaining context ends until stack is empty
        if self.finished {
            return self.finalize();
        }

        loop {
            // Check timeout periodically (every 100 operations to minimize overhead)
            self.operations_count += 1;
            if self.operations_count % 100 == 0 {
                self.check_timeout()?;
            }

            let (line_num, line) = if let Some(l) = self.reader.next_line()? {
                l
            } else {
                self.finished = true;
                // Emit any remaining list ends
                return self.finalize();
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
                    format!("indent depth {indent} exceeds limit"),
                ));
            }

            // Pop contexts as needed based on indentation
            let events = context::pop_contexts(&mut self.state.stack, indent)?;
            if let Some(event) = events {
                // Push back the current line to process after emitting list end
                self.reader.push_back(line_num, line);
                return Ok(Some(event));
            }

            // Parse line content
            return self.parse_line(content, indent, line_num);
        }
    }

    fn parse_line(
        &mut self,
        content: &str,
        indent: usize,
        line_num: usize,
    ) -> StreamResult<Option<NodeEvent>> {
        // Check for child block syntax BEFORE stripping comments
        // because @Type#N: uses # which would otherwise be treated as comment start
        if content.starts_with('@') && content.contains('#') {
            // Check if it looks like child block pattern: @Type#N:
            // We need to pass the original content to preserve the #N: syntax
            return list_parsing::try_parse_child_block(
                content,
                indent,
                line_num,
                &self.state.stack,
                &mut self.state.pending_events,
                &mut self.state.prev_row,
                self.header.as_ref(),
            );
        }

        // Strip inline comment for all other line types
        let content = strip_comment(content);

        if let Some(row_content) = content.strip_prefix('|') {
            // Matrix row
            let event = list_parsing::parse_matrix_row(
                row_content,
                indent,
                line_num,
                &mut self.state.stack,
                &mut self.state.prev_row,
                self.header.as_ref(),
            )?;
            // Update list context after parsing row
            if let NodeEvent::Node(ref node) = event {
                context::update_list_context(&mut self.state.stack, &node.type_name, &node.id);
            }
            return Ok(Some(event));
        } else if content.starts_with('@') {
            // Child block without # - this is an error (requires #N: format)
            return list_parsing::try_parse_child_block(
                content,
                indent,
                line_num,
                &self.state.stack,
                &mut self.state.pending_events,
                &mut self.state.prev_row,
                self.header.as_ref(),
            );
        }

        if let Some(colon_pos) = content.find(':') {
            let key = content[..colon_pos].trim();
            let after_colon = &content[colon_pos + 1..];

            if !is_valid_key_token(key) {
                return Err(StreamError::syntax(line_num, format!("invalid key: {key}")));
            }

            let after_colon_trimmed = after_colon.trim();

            if after_colon_trimmed.is_empty() {
                // Object start: validate indent and context
                context::validate_indent_for_key_value(&self.state.stack, indent, line_num)?;

                self.state.stack.push(Context::Object {
                    key: key.to_string(),
                    indent,
                });
                Ok(Some(NodeEvent::ObjectStart {
                    key: key.to_string(),
                    line: line_num,
                }))
            } else if after_colon_trimmed.starts_with('@')
                && list_parsing::is_list_start(after_colon_trimmed)
            {
                // Matrix list start
                // Accept both "key:@Type" (v2.0 canonical) and "key:@Type" (backward compat)

                // List declarations are allowed in list context (for nested lists)
                // so we don't call validate_indent_for_key_value here

                let (type_name, schema) = list_parsing::parse_list_start(
                    after_colon_trimmed,
                    line_num,
                    self.header.as_ref(),
                )?;

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
                // Key-value pair: require space after colon and validate indent
                if !after_colon.starts_with(' ') {
                    return Err(StreamError::syntax(
                        line_num,
                        "space required after ':' in key-value",
                    ));
                }
                context::validate_indent_for_key_value(&self.state.stack, indent, line_num)?;

                let value = value_inference::infer_value(
                    after_colon.trim(),
                    line_num,
                    self.header.as_ref(),
                )?;
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

    fn finalize(&mut self) -> StreamResult<Option<NodeEvent>> {
        // If we already sent EndOfDocument, return None to signal true end of stream
        if self.sent_end_of_document {
            return Ok(None);
        }

        // Pop remaining contexts
        while self.state.stack.len() > 1 {
            // Safe: loop condition guarantees stack has elements
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
                    // Root context should never be popped
                }
            }
        }

        // Mark that we've sent EndOfDocument, so subsequent calls return None
        self.sent_end_of_document = true;
        Ok(Some(NodeEvent::EndOfDocument))
    }
}

impl<R: Read> Iterator for StreamingParser<R> {
    type Item = StreamResult<NodeEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_event() {
            Ok(Some(NodeEvent::EndOfDocument)) => None,
            Ok(Some(event)) => Some(Ok(event)),
            Ok(None) => None,
            Err(e) => {
                // Stop iteration after an error to prevent inconsistent state
                self.finished = true;
                self.errored = true;
                Some(Err(e))
            }
        }
    }
}

// File opening with compression support
#[cfg(feature = "compression")]
impl StreamingParser<crate::compression::CompressionReader<std::fs::File>> {
    /// Open a file with automatic compression detection.
    ///
    /// Detects compression format from the file extension (`.gz`, `.zst`, `.lz4`)
    /// and automatically decompresses the content.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_stream::StreamingParser;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // Open a GZIP-compressed HEDL file
    /// let parser = StreamingParser::open("data.hedl.gz")?;
    ///
    /// for event in parser {
    ///     println!("{:?}", event?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// - `StreamError::Io`: File not found or cannot be opened
    /// - `StreamError::Compression`: Decompression initialization failed
    /// - `StreamError::MissingVersion`: Invalid HEDL header
    pub fn open<P: AsRef<std::path::Path>>(path: P) -> StreamResult<Self> {
        Self::open_with_config(path, StreamingParserConfig::default())
    }

    /// Open a file with automatic compression detection and custom configuration.
    ///
    /// Combines automatic compression detection with custom parser settings.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_stream::{StreamingParser, StreamingParserConfig};
    /// use std::time::Duration;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = StreamingParserConfig {
    ///     timeout: Some(Duration::from_secs(30)),
    ///     ..Default::default()
    /// };
    ///
    /// let parser = StreamingParser::open_with_config("data.hedl.zst", config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_config<P: AsRef<std::path::Path>>(
        path: P,
        config: StreamingParserConfig,
    ) -> StreamResult<Self> {
        use crate::compression::{CompressionFormat, CompressionReader};

        let path = path.as_ref();
        let format = CompressionFormat::from_path(path);

        let file = std::fs::File::open(path).map_err(StreamError::Io)?;
        let reader = CompressionReader::with_format(file, format).map_err(StreamError::Io)?;

        Self::with_config(reader, config)
    }

    /// Open a file with explicit compression format.
    ///
    /// Use this when the file extension doesn't match the actual compression
    /// format, or when you want to force a specific decompression algorithm.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hedl_stream::StreamingParser;
    /// use hedl_stream::compression::CompressionFormat;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// // File has no extension but is GZIP compressed
    /// let parser = StreamingParser::open_with_compression(
    ///     "data.hedl",
    ///     CompressionFormat::Gzip,
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_compression<P: AsRef<std::path::Path>>(
        path: P,
        format: crate::compression::CompressionFormat,
    ) -> StreamResult<Self> {
        use crate::compression::CompressionReader;

        let file = std::fs::File::open(path).map_err(StreamError::Io)?;
        let reader = CompressionReader::with_format(file, format).map_err(StreamError::Io)?;

        Self::new(reader)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_basic_parsing() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
 | alice, Alice
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_object_parsing() {
        let input = r#"
%VERSION: 1.0
---
config:
 debug: true
 timeout: 30
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_nested_lists() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, title]
%NEST: User > Post
---
users:@User
 | alice, Alice
  | post1, First Post
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_inline_children() {
        let input = r#"
%VERSION: 2.0
%S:User:[id,name]
%S:Post:[id,title]
%N:User>Post
---
users:@User
 | alice, Alice
  @Post#2:|p1,First|p2,Second
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
        assert!(!events.is_empty());
    }
}
