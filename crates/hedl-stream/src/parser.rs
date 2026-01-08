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

//! Streaming parser implementation.
//!
//! This module provides the core streaming parser for HEDL documents. The parser
//! processes HEDL files incrementally, yielding events as they are encountered,
//! making it suitable for processing files that are too large to fit in memory.
//!
//! # Design Philosophy
//!
//! - **Memory Efficiency**: Only the current line and parsing state are kept in memory
//! - **Iterator-Based**: Standard Rust iterator interface for easy composition
//! - **Error Recovery**: Clear error messages with line numbers for debugging
//! - **Safety**: Built-in timeout protection against malicious/untrusted input
//!
//! # Basic Usage
//!
//! ```rust
//! use hedl_stream::{StreamingParser, NodeEvent};
//! use std::io::Cursor;
//!
//! let input = r#"
//! %VERSION: 1.0
//! %STRUCT: User: [id, name, email]
//! ---
//! users: @User
//!   | alice, Alice Smith, alice@example.com
//!   | bob, Bob Jones, bob@example.com
//! "#;
//!
//! let parser = StreamingParser::new(Cursor::new(input)).unwrap();
//!
//! for event in parser {
//!     match event {
//!         Ok(NodeEvent::Node(node)) => {
//!             println!("Found {}: {}", node.type_name, node.id);
//!         }
//!         Err(e) => eprintln!("Parse error: {}", e),
//!         _ => {}
//!     }
//! }
//! ```

use crate::error::{StreamError, StreamResult};
use crate::event::{HeaderInfo, NodeEvent, NodeInfo};
use crate::reader::LineReader;
use hedl_core::Value;
use hedl_core::lex::{calculate_indent, is_valid_key_token, is_valid_type_name};
use std::io::Read;
use std::time::{Duration, Instant};

/// Type alias for list context lookup result: (type_name, schema, optional last_node info)
type ListContextResult = (String, Vec<String>, Option<(String, String)>);

/// Configuration options for the streaming parser.
///
/// Controls memory limits, buffer sizes, and timeout behavior.
///
/// # Examples
///
/// ## Default Configuration
///
/// ```rust
/// use hedl_stream::StreamingParserConfig;
///
/// let config = StreamingParserConfig::default();
/// assert_eq!(config.max_line_length, 1_000_000);
/// assert_eq!(config.max_indent_depth, 100);
/// assert_eq!(config.buffer_size, 64 * 1024);
/// assert_eq!(config.timeout, None);
/// ```
///
/// ## Custom Configuration for Large Files
///
/// ```rust
/// use hedl_stream::StreamingParserConfig;
///
/// let config = StreamingParserConfig {
///     max_line_length: 10_000_000,  // 10MB lines
///     max_indent_depth: 1000,        // Deep nesting
///     buffer_size: 256 * 1024,       // 256KB buffer
///     timeout: None,                 // No timeout
/// };
/// ```
///
/// ## Configuration for Untrusted Input
///
/// ```rust
/// use hedl_stream::StreamingParserConfig;
/// use std::time::Duration;
///
/// let config = StreamingParserConfig {
///     max_line_length: 100_000,           // Limit line length
///     max_indent_depth: 50,                // Limit nesting
///     buffer_size: 32 * 1024,              // Smaller buffer
///     timeout: Some(Duration::from_secs(10)), // 10 second timeout
/// };
/// ```
#[derive(Debug, Clone)]
pub struct StreamingParserConfig {
    /// Maximum line length in bytes.
    ///
    /// Lines exceeding this length will cause a parsing error. This protects against
    /// malformed input with extremely long lines that could exhaust memory.
    ///
    /// Default: 1,000,000 bytes (1MB)
    pub max_line_length: usize,

    /// Maximum indentation depth.
    ///
    /// Indentation levels exceeding this depth will cause a parsing error. This
    /// protects against deeply nested structures that could cause stack overflow
    /// or performance issues.
    ///
    /// Default: 100 levels
    pub max_indent_depth: usize,

    /// Buffer size for reading input.
    ///
    /// Larger buffers can improve performance for large files by reducing the
    /// number of system calls, but use more memory.
    ///
    /// Default: 64KB
    pub buffer_size: usize,

    /// Timeout for parsing operations.
    ///
    /// If set, the parser will return a `StreamError::Timeout` if parsing takes
    /// longer than the specified duration. This protects against infinite loops
    /// from malicious or malformed input.
    ///
    /// Set to `None` to disable timeout checking (default for trusted input).
    ///
    /// Default: None (no timeout)
    ///
    /// # Performance Note
    ///
    /// Timeout checking is performed periodically (every 100 operations) to minimize
    /// overhead. For very fast parsing, the actual timeout may slightly exceed the
    /// configured limit.
    pub timeout: Option<Duration>,
}

impl Default for StreamingParserConfig {
    fn default() -> Self {
        Self {
            max_line_length: 1_000_000,
            max_indent_depth: 100,
            buffer_size: 64 * 1024,
            timeout: None, // No timeout by default for backward compatibility
        }
    }
}

/// Streaming HEDL parser.
///
/// Processes HEDL documents incrementally, yielding `NodeEvent` items as they
/// are parsed without loading the entire document into memory. This makes it
/// suitable for processing multi-gigabyte files on systems with limited RAM.
///
/// # Memory Characteristics
///
/// - **Header**: Parsed once at initialization and kept in memory
/// - **Per-Line**: Only current line and parsing context (stack depth proportional to nesting)
/// - **No Buffering**: Nodes are yielded immediately after parsing
///
/// # When to Use
///
/// - **Large Files**: Files too large to fit comfortably in memory
/// - **Streaming Workflows**: Processing data as it arrives (pipes, network streams)
/// - **Memory-Constrained**: Embedded systems or containers with memory limits
/// - **ETL Pipelines**: Extract-transform-load workflows with HEDL data
///
/// # Iterator Interface
///
/// `StreamingParser` implements `Iterator<Item = StreamResult<NodeEvent>>`, allowing
/// use with standard iterator methods like `filter`, `map`, `collect`, etc.
///
/// # Examples
///
/// ## Basic Streaming Parse
///
/// ```rust
/// use hedl_stream::{StreamingParser, NodeEvent};
/// use std::fs::File;
/// use std::io::BufReader;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// # use std::io::Cursor;
/// # let file = Cursor::new(r#"
/// # %VERSION: 1.0
/// # %STRUCT: User: [id, name]
/// # ---
/// # users: @User
/// #   | alice, Alice
/// # "#);
/// let reader = BufReader::new(file);
/// let parser = StreamingParser::new(reader)?;
///
/// for event in parser {
///     match event? {
///         NodeEvent::Node(node) => {
///             println!("Processing {}: {}", node.type_name, node.id);
///             // Process node immediately, no buffering
///         }
///         NodeEvent::ListStart { type_name, .. } => {
///             println!("Starting list of {}", type_name);
///         }
///         NodeEvent::ListEnd { count, .. } => {
///             println!("Finished list with {} items", count);
///         }
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Filtering During Parse
///
/// ```rust
/// use hedl_stream::{StreamingParser, NodeEvent};
/// use std::io::Cursor;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let input = r#"
/// %VERSION: 1.0
/// %STRUCT: User: [id, name, active]
/// ---
/// users: @User
///   | alice, Alice, true
///   | bob, Bob, false
///   | carol, Carol, true
/// "#;
///
/// let parser = StreamingParser::new(Cursor::new(input))?;
///
/// // Only collect active users
/// let active_users: Vec<_> = parser
///     .filter_map(|event| event.ok())
///     .filter_map(|event| {
///         if let NodeEvent::Node(node) = event {
///             Some(node)
///         } else {
///             None
///         }
///     })
///     .filter(|node| {
///         // Check if 'active' field (index 2) is true
///         matches!(node.get_field(2), Some(hedl_core::Value::Bool(true)))
///     })
///     .collect();
///
/// assert_eq!(active_users.len(), 2); // alice and carol
/// # Ok(())
/// # }
/// ```
///
/// ## With Timeout for Untrusted Input
///
/// ```rust
/// use hedl_stream::{StreamingParser, StreamingParserConfig, StreamError};
/// use std::time::Duration;
/// use std::io::Cursor;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let untrusted_input = "..."; // Input from external source
///
/// let config = StreamingParserConfig {
///     timeout: Some(Duration::from_secs(5)),
///     ..Default::default()
/// };
///
/// let parser = StreamingParser::with_config(
///     Cursor::new(untrusted_input),
///     config
/// )?;
///
/// for event in parser {
///     match event {
///         Ok(event) => {
///             // Process event
///         }
///         Err(StreamError::Timeout { elapsed, limit }) => {
///             eprintln!("Parsing timed out after {:?}", elapsed);
///             break;
///         }
///         Err(e) => {
///             eprintln!("Parse error: {}", e);
///             break;
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// ## Processing Nested Structures
///
/// ```rust
/// use hedl_stream::{StreamingParser, NodeEvent};
/// use std::io::Cursor;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let input = r#"
/// %VERSION: 1.0
/// %STRUCT: User: [id, name]
/// %STRUCT: Order: [id, amount]
/// %NEST: User > Order
/// ---
/// users: @User
///   | alice, Alice
///     | order1, 100.00
///     | order2, 50.00
///   | bob, Bob
///     | order3, 75.00
/// "#;
///
/// let parser = StreamingParser::new(Cursor::new(input))?;
///
/// for event in parser.filter_map(|e| e.ok()) {
///     if let NodeEvent::Node(node) = event {
///         if node.is_nested() {
///             println!("  Child: {} belongs to {:?}",
///                 node.id, node.parent_id);
///         } else {
///             println!("Parent: {}", node.id);
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
///
/// # Error Handling
///
/// Parsing errors include line numbers for easy debugging:
///
/// ```rust
/// use hedl_stream::{StreamingParser, StreamError};
/// use std::io::Cursor;
///
/// let bad_input = r#"
/// %VERSION: 1.0
/// ---
/// invalid line without colon
/// "#;
///
/// let parser = StreamingParser::new(Cursor::new(bad_input)).unwrap();
///
/// for event in parser {
///     if let Err(e) = event {
///         if let Some(line) = e.line() {
///             eprintln!("Error at line {}: {}", line, e);
///         }
///     }
/// }
/// ```
pub struct StreamingParser<R: Read> {
    reader: LineReader<R>,
    config: StreamingParserConfig,
    header: Option<HeaderInfo>,
    state: ParserState,
    finished: bool,
    start_time: Instant,
    operations_count: usize, // Track operations for periodic timeout checks
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
    /// users: @User
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
        let mut parser = Self {
            reader: LineReader::with_capacity(reader, config.buffer_size),
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
    /// assert_eq!(header.get_child_type("User"), Some(&"Order".to_string()));
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
    /// users: @User
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
                self.parse_directive(trimmed, line_num, &mut header, &mut found_version)?;
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
        // Safe: starts_with check guarantees prefix exists
        let rest = line.strip_prefix("%VERSION").expect("prefix exists").trim();
        // Handle both "%VERSION: 1.0" and "%VERSION: 1.0" formats
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
        // Safe: starts_with check guarantees prefix exists
        let rest = line.strip_prefix("%STRUCT").expect("prefix exists").trim();
        // Handle both "%STRUCT TypeName: [cols]" and "%STRUCT: TypeName: [cols]" formats
        let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

        let bracket_start = rest
            .find('[')
            .ok_or_else(|| StreamError::syntax(line_num, "missing '[' in %STRUCT"))?;
        let bracket_end = rest
            .find(']')
            .ok_or_else(|| StreamError::syntax(line_num, "missing ']' in %STRUCT"))?;

        // Type name may have trailing colon and optional count, strip them
        // Format: TypeName: or TypeName (N):
        let type_part = rest[..bracket_start].trim().trim_end_matches(':').trim();
        // Handle optional count: "TypeName (N)" -> extract just "TypeName"
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
        // Safe: starts_with check guarantees prefix exists
        let rest = line.strip_prefix("%ALIAS").expect("prefix exists").trim();
        // Handle both "%ALIAS: %short: = ..." and "%ALIAS: %short: ..." formats
        let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

        // Support both '=' and ':' as separators
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
        // Safe: starts_with check guarantees prefix exists
        let rest = line.strip_prefix("%NEST").expect("prefix exists").trim();
        // Handle both "%NEST: Parent > Child" and "%NEST: Parent > Child" formats
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

    /// Parse the next event from the stream.
    fn next_event(&mut self) -> StreamResult<Option<NodeEvent>> {
        if self.finished {
            return Ok(None);
        }

        loop {
            // Check timeout periodically (every 100 operations to minimize overhead)
            self.operations_count += 1;
            if self.operations_count.is_multiple_of(100) {
                self.check_timeout()?;
            }

            let (line_num, line) = match self.reader.next_line()? {
                Some(l) => l,
                None => {
                    self.finished = true;
                    // Emit any remaining list ends
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

    fn pop_contexts(&mut self, current_indent: usize) -> StreamResult<Option<NodeEvent>> {
        while self.state.stack.len() > 1 {
            // Safe: loop condition guarantees stack has elements
            let should_pop = match self.state.stack.last().expect("stack has elements") {
                Context::Root => false,
                Context::Object { indent, .. } => current_indent <= *indent,
                Context::List { row_indent, .. } => current_indent < *row_indent,
            };

            if should_pop {
                // Safe: loop condition guarantees stack has elements
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
        // Strip inline comment
        let content = strip_comment(content);

        if let Some(row_content) = content.strip_prefix('|') {
            // Matrix row
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
                // Object start
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
                // List start
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
                // Key-value pair
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
        let rest = &s[1..]; // Skip @

        if let Some(bracket_pos) = rest.find('[') {
            // Inline schema: @TypeName[col1, col2]
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
            // Reference to declared schema: @TypeName
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

        // Find active list context
        let (type_name, schema, parent_info) = self.find_list_context(indent, line_num)?;

        // Parse HEDL matrix row (comma-separated values after the |)
        // Use hedl_row parser for proper CSV-like parsing
        let fields = hedl_core::lex::parse_csv_row(content)
            .map_err(|e| StreamError::syntax(line_num, format!("row parse error: {}", e)))?;

        // Validate shape
        if fields.len() != schema.len() {
            return Err(StreamError::ShapeMismatch {
                line: line_num,
                expected: schema.len(),
                got: fields.len(),
            });
        }

        // Infer values with ditto handling
        let mut values = Vec::with_capacity(fields.len());
        for (col_idx, field) in fields.iter().enumerate() {
            let value = if field.value == "^" {
                // Ditto - use previous row's value
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

        // Get ID from first column
        let id = match &values[0] {
            Value::String(s) => s.clone(),
            _ => return Err(StreamError::syntax(line_num, "ID column must be a string")),
        };

        // Update context
        self.update_list_context(&type_name, &id);
        self.state.prev_row = Some(values.clone());

        // Build node info
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
                    // Peer row
                    return Ok((type_name.clone(), schema.clone(), None));
                } else if indent == *row_indent + 1 {
                    // Child row
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

                    // Push child list context
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

        // Reference
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

        // Alias
        if let Some(alias) = s.strip_prefix('$') {
            if let Some(header) = &self.header {
                if let Some(value) = header.aliases.get(alias) {
                    return Ok(Value::String(value.clone()));
                }
            }
            return Ok(Value::String(s.to_string()));
        }

        // Number
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Value::Int(i));
        }
        if let Ok(f) = s.parse::<f64>() {
            return Ok(Value::Float(f));
        }

        // Default to string
        Ok(Value::String(s.to_string()))
    }

    fn finalize(&mut self) -> StreamResult<Option<NodeEvent>> {
        // Pop remaining contexts
        while self.state.stack.len() > 1 {
            // Safe: loop condition guarantees stack has elements
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

impl<R: Read> Iterator for StreamingParser<R> {
    type Item = StreamResult<NodeEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_event() {
            Ok(Some(NodeEvent::EndOfDocument)) => None,
            Ok(Some(event)) => Some(Ok(event)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

/// SIMD-optimized comment scanning module.
///
/// This module provides AVX2-accelerated scanning for the '#' character
/// when detecting comment boundaries. Falls back to scalar implementation
/// on non-AVX2 platforms or when the feature is disabled.
mod simd_comment {
    #[cfg(all(target_arch = "x86_64", feature = "avx2"))]
    use std::arch::x86_64::*;

    /// Find the first occurrence of '#' in the input string using SIMD.
    ///
    /// This function scans 32 bytes at a time using AVX2 instructions
    /// on x86_64 platforms when the `avx2` feature is enabled.
    ///
    /// # Safety
    ///
    /// On AVX2-enabled platforms, this uses `unsafe` SIMD intrinsics
    /// but ensures all memory accesses are valid.
    #[inline]
    pub fn find_hash_simd(s: &[u8]) -> Option<usize> {
        #[cfg(all(target_arch = "x86_64", feature = "avx2"))]
        {
            // Check if AVX2 is available at runtime
            if is_x86_feature_detected!("avx2") {
                return unsafe { find_hash_avx2(s) };
            }
        }

        // Fallback to scalar search
        find_hash_scalar(s)
    }

    /// AVX2 implementation for finding '#' character.
    ///
    /// Scans 32 bytes at a time using SIMD instructions.
    #[cfg(all(target_arch = "x86_64", feature = "avx2"))]
    #[target_feature(enable = "avx2")]
    unsafe fn find_hash_avx2(s: &[u8]) -> Option<usize> {
        const CHUNK_SIZE: usize = 32;
        let len = s.len();

        if len == 0 {
            return None;
        }

        let hash_vec = _mm256_set1_epi8(b'#' as i8);
        let mut offset = 0;

        // Process 32-byte chunks
        while offset + CHUNK_SIZE <= len {
            // Load 32 bytes from input
            let chunk = _mm256_loadu_si256(s.as_ptr().add(offset) as *const __m256i);

            // Compare with '#' character
            let matches = _mm256_cmpeq_epi8(chunk, hash_vec);

            // Convert to bitmask
            let mask = _mm256_movemask_epi8(matches);

            if mask != 0 {
                // Found at least one match, find the first one
                let bit_pos = mask.trailing_zeros() as usize;
                return Some(offset + bit_pos);
            }

            offset += CHUNK_SIZE;
        }

        // Handle remaining bytes with scalar search
        find_hash_scalar(&s[offset..]).map(|pos| offset + pos)
    }

    /// Scalar fallback implementation for finding '#' character.
    #[inline]
    fn find_hash_scalar(s: &[u8]) -> Option<usize> {
        s.iter().position(|&b| b == b'#')
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_find_hash_basic() {
            assert_eq!(find_hash_simd(b"hello # world"), Some(6));
            assert_eq!(find_hash_simd(b"no comment"), None);
            assert_eq!(find_hash_simd(b"#start"), Some(0));
            assert_eq!(find_hash_simd(b"end#"), Some(3));
        }

        #[test]
        fn test_find_hash_long() {
            // Test with string longer than SIMD chunk size
            let long = b"a".repeat(100);
            assert_eq!(find_hash_simd(&long), None);

            let mut with_hash = b"a".repeat(50);
            with_hash.push(b'#');
            with_hash.extend_from_slice(&b"a".repeat(50));
            assert_eq!(find_hash_simd(&with_hash), Some(50));
        }

        #[test]
        fn test_find_hash_edge_cases() {
            assert_eq!(find_hash_simd(b""), None);
            assert_eq!(find_hash_simd(b"#"), Some(0));
            assert_eq!(find_hash_simd(b"##"), Some(0));
        }

        #[test]
        fn test_find_hash_alignment() {
            // Test various alignment scenarios
            for offset in 0..32 {
                let mut data = vec![b'a'; offset];
                data.push(b'#');
                data.extend_from_slice(&vec![b'b'; 32]);
                assert_eq!(find_hash_simd(&data), Some(offset));
            }
        }

        #[test]
        fn test_find_hash_multiple() {
            assert_eq!(find_hash_simd(b"# # #"), Some(0));
            assert_eq!(find_hash_simd(b"a # # #"), Some(2));
        }
    }
}

/// Strip inline comments from a line, respecting quoted strings and escapes.
///
/// Finds the first unquoted, unescaped '#' character and returns the string
/// up to that point. Uses SIMD acceleration when available.
///
/// # Examples
///
/// ```text
/// use hedl_stream::parser::strip_comment;
/// assert_eq!(strip_comment("hello # comment"), "hello");
/// assert_eq!(strip_comment(r#""hello # not comment""#), r#""hello # not comment""#);
/// assert_eq!(strip_comment("no comment"), "no comment");
/// ```
#[inline]
pub(crate) fn strip_comment(s: &str) -> &str {
    // Find # not inside quotes using SIMD-accelerated scanning
    let bytes = s.as_bytes();
    let mut in_quotes = false;
    let mut escape = false;
    let mut search_start = 0;

    loop {
        // Use SIMD to find the next potential comment start
        let hash_pos = match simd_comment::find_hash_simd(&bytes[search_start..]) {
            Some(pos) => search_start + pos,
            None => return s, // No '#' found
        };

        // Verify this '#' is not inside quotes or escaped
        // Scan from last search position to hash position
        for i in search_start..hash_pos {
            let c = bytes[i];

            if escape {
                escape = false;
                continue;
            }

            match c {
                b'\\' => escape = true,
                b'"' => in_quotes = !in_quotes,
                _ => {}
            }
        }

        // Check the '#' itself
        if escape {
            // '#' is escaped, continue searching
            escape = false;
            search_start = hash_pos + 1;
            continue;
        }

        if !in_quotes {
            // Found unquoted, unescaped '#' - this is the comment start
            return s[..hash_pos].trim_end();
        }

        // '#' is inside quotes, continue searching
        search_start = hash_pos + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // ============ HEADER PARSING TESTS ============

    #[test]
    fn test_parse_header() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
%ALIAS active = "Active"
%NEST: User > Order
---
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();

        assert_eq!(header.version, (1, 0));
        assert_eq!(
            header.structs.get("User"),
            Some(&vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string()
            ])
        );
        assert_eq!(header.aliases.get("active"), Some(&"Active".to_string()));
        assert_eq!(header.nests.get("User"), Some(&"Order".to_string()));
    }

    #[test]
    fn test_header_missing_version() {
        let input = r#"
%STRUCT: User: [id, name]
---
"#;
        let result = StreamingParser::new(Cursor::new(input));
        assert!(matches!(result, Err(StreamError::MissingVersion)));
    }

    #[test]
    fn test_header_invalid_version_format() {
        let input = r#"
%VERSION abc
---
"#;
        let result = StreamingParser::new(Cursor::new(input));
        assert!(matches!(result, Err(StreamError::InvalidVersion(_))));
    }

    #[test]
    fn test_header_version_single_number() {
        let input = r#"
%VERSION 1
---
"#;
        let result = StreamingParser::new(Cursor::new(input));
        assert!(matches!(result, Err(StreamError::InvalidVersion(_))));
    }

    #[test]
    fn test_header_multiple_schemas() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Product: [id, title, price]
%STRUCT: Order: [id, user_id, product_id]
---
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();

        assert_eq!(header.structs.len(), 3);
        assert!(header.structs.contains_key("User"));
        assert!(header.structs.contains_key("Product"));
        assert!(header.structs.contains_key("Order"));
    }

    #[test]
    fn test_header_struct_missing_bracket() {
        let input = r#"
%VERSION: 1.0
%STRUCT User id, name
---
"#;
        let result = StreamingParser::new(Cursor::new(input));
        assert!(matches!(result, Err(StreamError::Syntax { .. })));
    }

    #[test]
    fn test_header_empty_struct() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: []
---
"#;
        let result = StreamingParser::new(Cursor::new(input));
        assert!(matches!(result, Err(StreamError::Syntax { .. })));
    }

    #[test]
    fn test_header_alias_missing_equals() {
        let input = r#"
%VERSION: 1.0
%ALIAS foo "bar"
---
"#;
        let result = StreamingParser::new(Cursor::new(input));
        assert!(matches!(result, Err(StreamError::Syntax { .. })));
    }

    #[test]
    fn test_header_nest_missing_arrow() {
        let input = r#"
%VERSION: 1.0
%NEST Parent Child
---
"#;
        let result = StreamingParser::new(Cursor::new(input));
        assert!(matches!(result, Err(StreamError::Syntax { .. })));
    }

    #[test]
    fn test_header_with_comments() {
        let input = r#"
%VERSION: 1.0
# This is a comment
%STRUCT: User: [id, name] # inline comment
---
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();
        assert!(header.structs.contains_key("User"));
    }

    #[test]
    fn test_header_blank_lines() {
        let input = r#"
%VERSION: 1.0

%STRUCT: User: [id, name]

---
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let header = parser.header().unwrap();
        assert!(header.structs.contains_key("User"));
    }

    // ============ BASIC STREAMING TESTS ============

    #[test]
    fn test_streaming_nodes() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice Smith
  | bob, Bob Jones
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();

        let events: Vec<_> = parser.collect();
        for event in &events {
            if let Err(e) = event {
                eprintln!("Error: {:?}", e);
            }
        }
        assert!(events.iter().all(|e| e.is_ok()));

        let nodes: Vec<_> = events
            .iter()
            .filter_map(|e| e.as_ref().ok())
            .filter_map(|e| e.as_node())
            .collect();

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, "alice");
        assert_eq!(nodes[1].id, "bob");
    }

    #[test]
    fn test_streaming_empty_body() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect();
        assert!(events.is_empty());
    }

    #[test]
    fn test_streaming_list_start_end_events() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();

        let list_starts: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, NodeEvent::ListStart { .. }))
            .collect();
        let list_ends: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, NodeEvent::ListEnd { .. }))
            .collect();

        assert_eq!(list_starts.len(), 1);
        assert_eq!(list_ends.len(), 1);

        if let NodeEvent::ListStart { key, type_name, .. } = &list_starts[0] {
            assert_eq!(key, "users");
            assert_eq!(type_name, "User");
        }

        if let NodeEvent::ListEnd {
            type_name, count, ..
        } = &list_ends[0]
        {
            assert_eq!(type_name, "User");
            assert_eq!(*count, 1);
        }
    }

    // ============ MATRIX ROW EDGE CASES ============

    #[test]
    fn test_matrix_row_empty_fields() {
        // Note: Empty fields are NOT preserved in the current pipe-splitting logic
        // When the field is truly empty (just whitespace between pipes), it's filtered
        // Use ~ (tilde) for explicit null values
        let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id, optional, required]
---
data: @Data
  | row1, ~, value
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "row1");
        assert_eq!(nodes[0].fields[1], Value::Null);
    }

    #[test]
    fn test_matrix_row_quoted_fields() {
        let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id, description]
---
data: @Data
  | row1, "Hello, World"
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 1);
        assert_eq!(
            nodes[0].fields[1],
            Value::String("Hello, World".to_string())
        );
    }

    #[test]
    fn test_matrix_row_shape_mismatch() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect();
        let errors: Vec<_> = events.iter().filter(|e| e.is_err()).collect();

        assert!(!errors.is_empty());
        if let Err(StreamError::ShapeMismatch { expected, got, .. }) = &errors[0] {
            assert_eq!(*expected, 3);
            assert_eq!(*got, 2);
        }
    }

    #[test]
    fn test_matrix_row_references() {
        let input = r#"
%VERSION: 1.0
%STRUCT: Order: [id, user]
---
orders: @Order
  | order1, @User:alice
  | order2, @bob
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 2);

        if let Value::Reference(r) = &nodes[0].fields[1] {
            assert_eq!(r.type_name, Some("User".to_string()));
            assert_eq!(r.id, "alice");
        } else {
            panic!("Expected reference");
        }

        if let Value::Reference(r) = &nodes[1].fields[1] {
            assert_eq!(r.type_name, None);
            assert_eq!(r.id, "bob");
        } else {
            panic!("Expected reference");
        }
    }

    #[test]
    fn test_matrix_row_booleans() {
        let input = r#"
%VERSION: 1.0
%STRUCT: Flag: [id, active, verified]
---
flags: @Flag
  | flag1, true, false
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].fields[1], Value::Bool(true));
        assert_eq!(nodes[0].fields[2], Value::Bool(false));
    }

    #[test]
    fn test_matrix_row_numbers() {
        let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id, int_val, float_val]
---
data: @Data
  | row1, 42, 3.5
  | row2, -100, -2.5
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].fields[1], Value::Int(42));
        assert_eq!(nodes[0].fields[2], Value::Float(3.5));
        assert_eq!(nodes[1].fields[1], Value::Int(-100));
        assert_eq!(nodes[1].fields[2], Value::Float(-2.5));
    }

    #[test]
    fn test_matrix_row_null() {
        let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id, nullable]
---
data: @Data
  | row1, ~
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].fields[1], Value::Null);
    }

    #[test]
    fn test_matrix_row_ditto() {
        let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id, category]
---
data: @Data
  | row1, CategoryA
  | row2, ^
  | row3, ^
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].fields[1], Value::String("CategoryA".to_string()));
        assert_eq!(nodes[1].fields[1], Value::String("CategoryA".to_string()));
        assert_eq!(nodes[2].fields[1], Value::String("CategoryA".to_string()));
    }

    #[test]
    fn test_matrix_row_alias_substitution() {
        let input = r#"
%VERSION: 1.0
%ALIAS status = "Active"
%STRUCT: User: [id, status]
---
users: @User
  | alice, $status
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].fields[1], Value::String("Active".to_string()));
    }

    // ============ INLINE SCHEMA TESTS ============

    #[test]
    fn test_inline_schema() {
        let input = r#"
%VERSION: 1.0
---
items: @Item[id, name]
  | item1, First
  | item2, Second
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].type_name, "Item");
    }

    #[test]
    fn test_inline_schema_overrides_header() {
        let input = r#"
%VERSION: 1.0
%STRUCT: Item: [id, name, extra]
---
items: @Item[id, name]
  | item1, First
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        // Should use inline schema with 2 fields, not header schema with 3
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].fields.len(), 2);
    }

    // ============ OBJECT CONTEXT TESTS ============

    #[test]
    fn test_object_context() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
db:
  users: @User
    | alice, Alice
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();

        let obj_starts: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, NodeEvent::ObjectStart { .. }))
            .collect();
        assert_eq!(obj_starts.len(), 1);

        if let NodeEvent::ObjectStart { key, .. } = obj_starts[0] {
            assert_eq!(key, "db");
        }
    }

    #[test]
    fn test_scalar_value() {
        let input = r#"
%VERSION: 1.0
---
config:
  timeout: 30
  enabled: true
  name: "Test Config"
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();

        let scalars: Vec<_> = events
            .iter()
            .filter(|e| matches!(e, NodeEvent::Scalar { .. }))
            .collect();
        assert_eq!(scalars.len(), 3);
    }

    // ============ UNICODE TESTS ============

    #[test]
    fn test_unicode_ids() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | 用户1, 张三
  | пользователь, Иван
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].id, "用户1");
        assert_eq!(nodes[1].id, "пользователь");
    }

    #[test]
    fn test_unicode_in_values() {
        let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id, emoji]
---
data: @Data
  | row1, 🎉✨🚀
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].fields[1], Value::String("🎉✨🚀".to_string()));
    }

    // ============ COMMENT HANDLING TESTS ============

    #[test]
    fn test_inline_comments() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User  # list of users
  | alice, Alice Smith  # first user
  | bob, Bob Jones
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 2);
        // Comments should be stripped
        assert_eq!(nodes[0].id, "alice");
    }

    #[test]
    fn test_full_line_comments() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
# This is a comment
users: @User
  # Comment between rows
  | alice, Alice
  | bob, Bob
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 2);
    }

    #[test]
    fn test_hash_in_quoted_string() {
        let input =
            "%VERSION: 1.0\n%STRUCT: Data: [id, tag]\n---\ndata: @Data\n  | row1, \"#hashtag\"\n";
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].fields[1], Value::String("#hashtag".to_string()));
    }

    // ============ INDENT AND CONTEXT TESTS ============

    #[test]
    fn test_multiple_lists() {
        let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Product: [id, title]
---
users: @User
  | alice, Alice
products: @Product
  | prod1, Widget
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].type_name, "User");
        assert_eq!(nodes[1].type_name, "Product");
    }

    #[test]
    fn test_excessive_indent_error() {
        let config = StreamingParserConfig {
            max_indent_depth: 2,
            ..Default::default()
        };
        let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id]
---
level1:
  level2:
    level3:
      data: @Data
        | row1
"#;
        let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();

        // Should get an error for excessive indent
        let mut found_indent_error = false;
        for result in parser {
            if let Err(StreamError::Syntax { message, .. }) = result {
                if message.contains("indent depth") {
                    found_indent_error = true;
                    break;
                }
            }
        }
        assert!(found_indent_error);
    }

    // ============ ERROR HANDLING TESTS ============

    #[test]
    fn test_undefined_schema() {
        let input = r#"
%VERSION: 1.0
---
users: @User
  | alice, Alice
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect();
        let errors: Vec<_> = events.iter().filter(|e| e.is_err()).collect();

        // Should get an error because User schema is not defined
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_orphan_row_without_context() {
        let input = r#"
%VERSION: 1.0
%STRUCT: Data: [id]
---
| orphan_row
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect();
        let errors: Vec<_> = events.iter().filter(|e| e.is_err()).collect();

        assert!(!errors.is_empty());
    }

    #[test]
    fn test_missing_colon_error() {
        let input = r#"
%VERSION: 1.0
---
invalid line without colon
"#;
        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.collect();
        let errors: Vec<_> = events.iter().filter(|e| e.is_err()).collect();

        assert!(!errors.is_empty());
        if let Err(StreamError::Syntax { message, .. }) = &errors[0] {
            assert!(message.contains(":"));
        }
    }

    // ============ LARGE FILE SIMULATION ============

    #[test]
    fn test_many_rows() {
        let mut input = String::from(
            r#"
%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
"#,
        );
        for i in 0..1000 {
            input.push_str(&format!("  | row{}, value{}\n", i, i));
        }

        let parser = StreamingParser::new(Cursor::new(input)).unwrap();
        let events: Vec<_> = parser.filter_map(|e| e.ok()).collect();
        let nodes: Vec<_> = events.iter().filter_map(|e| e.as_node()).collect();

        assert_eq!(nodes.len(), 1000);
    }

    // ============ STRIP COMMENT HELPER TESTS ============

    #[test]
    fn test_strip_comment_basic() {
        assert_eq!(strip_comment("hello # comment"), "hello");
    }

    #[test]
    fn test_strip_comment_quoted() {
        assert_eq!(
            strip_comment(r#""hello # not comment""#),
            r#""hello # not comment""#
        );
    }

    #[test]
    fn test_strip_comment_escaped() {
        // Backslash escapes the hash, so \# is not treated as comment start
        assert_eq!(
            strip_comment(r#"hello\# not a comment"#),
            r#"hello\# not a comment"#
        );
        // But a later unescaped hash still starts a comment
        assert_eq!(
            strip_comment(r#"hello\# still here # comment"#),
            r#"hello\# still here"#
        );
    }

    #[test]
    fn test_strip_comment_escaped_in_quotes() {
        // Inside quotes, backslash-hash is preserved
        assert_eq!(
            strip_comment(r#""hello\#world" more"#),
            r#""hello\#world" more"#
        );
    }

    #[test]
    fn test_strip_comment_no_comment() {
        assert_eq!(strip_comment("hello world"), "hello world");
    }
}
