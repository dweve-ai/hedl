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

//! Main parser for HEDL documents.
//!
//! # Security Limits
//!
//! The parser enforces several security limits to prevent denial-of-service attacks:
//!
//! - `max_file_size`: Maximum input file size (default: 1GB)
//! - `max_line_length`: Maximum line length (default: 1MB)
//! - `max_indent_depth`: Maximum nesting depth for objects (default: 50)
//! - `max_nodes`: Maximum number of matrix list nodes (default: 10M)
//! - `max_aliases`: Maximum number of aliases (default: 10k)
//! - `max_columns`: Maximum columns per schema (default: 100)
//! - `max_nest_depth`: Maximum NEST hierarchy depth (default: 100)
//! - `max_block_string_size`: Maximum block string size (default: 10MB)
//! - `max_object_keys`: Maximum keys per object (default: 10k)
//! - **`max_total_keys`**: Maximum total keys across all objects (default: 10M)
//!
//! ## max_total_keys: Defense in Depth
//!
//! The `max_total_keys` limit is a critical security feature that prevents
//! memory exhaustion attacks via cumulative key allocation. Without this limit,
//! an attacker could create many small objects, each under `max_object_keys`,
//! but collectively consuming excessive memory.
//!
//! ### Attack Scenario (Without max_total_keys)
//!
//! ```text
//! # Attacker creates 100,000 objects with 10 keys each
//! # Each object is "valid" (under max_object_keys = 10,000)
//! # But total memory usage is excessive: 1,000,000 keys!
//! object0:
//!   key0: val0
//!   key1: val1
//!   ...
//!   key9: val9
//! object1:
//!   key0: val0
//!   ...
//! # ... 99,998 more objects
//! ```
//!
//! ### Defense (With max_total_keys = 10,000,000)
//!
//! The parser tracks cumulative keys across all objects and rejects documents
//! that exceed the limit, preventing this attack vector while allowing legitimate
//! large documents. The 10M default accommodates most real-world datasets while
//! still providing protection. For extremely large datasets, this limit can be
//! increased via `ParseOptions`.

use crate::block_string::{try_start_block_string, BlockStringResult, BlockStringState};
use crate::document::{Document, Item, MatrixList, Node};
use crate::error::{HedlError, HedlResult};
use crate::header::parse_header;
use crate::inference::{infer_quoted_value, infer_value, InferenceContext};
use crate::limits::Limits;
use crate::preprocess::{is_blank_line, is_comment_line, preprocess};
use crate::reference::{register_node, resolve_references, TypeRegistry};
use crate::value::Value;
use crate::lex::{calculate_indent, is_valid_key_token, is_valid_type_name, strip_comment};
use crate::lex::row::parse_csv_row;
use std::collections::BTreeMap;

/// Parsing options for configuring HEDL document parsing behavior.
///
/// ParseOptions provides both direct field access and a fluent builder API
/// for convenient configuration. All parsing functions accept ParseOptions
/// to customize limits, security settings, and error handling behavior.
///
/// # Creating ParseOptions
///
/// ## Using the builder pattern (recommended)
///
/// ```text
/// use hedl_core::ParseOptions;
///
/// // Typical strict parsing with custom depth limit
/// let opts = ParseOptions::builder()
///     .max_depth(100)
///     .strict(true)
///     .build();
///
/// // Lenient parsing for large datasets
/// let opts = ParseOptions::builder()
///     .max_array_length(50_000)
///     .strict(false)
///     .max_block_string_size(50 * 1024 * 1024)
///     .build();
///
/// // Restrictive parsing for security
/// let opts = ParseOptions::builder()
///     .max_file_size(10 * 1024 * 1024)
///     .max_line_length(64 * 1024)
///     .max_depth(20)
///     .max_array_length(1000)
///     .strict(true)
///     .build();
/// ```
///
/// ## Using defaults
///
/// ```text
/// use hedl_core::{ParseOptions, parse_with_limits};
///
/// // Default options: strict refs, normal limits
/// let opts = ParseOptions::default();
///
/// // Parse with defaults
/// let doc = parse_with_limits(input, opts)?;
/// ```
///
/// ## Direct field access
///
/// ```text
/// use hedl_core::{ParseOptions, Limits};
///
/// let mut opts = ParseOptions::default();
/// opts.strict_refs = false;
/// opts.limits.max_nodes = 5000;
/// ```
///
/// # Security Considerations
///
/// ParseOptions includes multiple security limits to prevent denial-of-service attacks:
///
/// - `max_file_size`: Prevents loading extremely large files
/// - `max_line_length`: Prevents regex DOS via extremely long lines
/// - `max_indent_depth`: Prevents stack overflow via deep nesting
/// - `max_nodes`: Prevents memory exhaustion via large matrix lists
/// - `max_object_keys` and `max_total_keys`: Prevent memory exhaustion via many objects
/// - `max_nest_depth`: Prevents stack overflow via deeply nested NEST hierarchies
/// - `max_block_string_size`: Prevents memory exhaustion via large block strings
///
/// # Fields
///
/// - `limits`: Security limits for parser resources
/// - `strict_refs`: When true, unresolved references cause errors; when false, ignored
#[derive(Debug, Clone)]
pub struct ParseOptions {
    /// Security limits.
    pub limits: Limits,
    /// Strict reference resolution (error on unresolved).
    pub strict_refs: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            limits: Limits::default(),
            strict_refs: true,
        }
    }
}

impl ParseOptions {
    /// Create a new builder for ParseOptions.
    ///
    /// # Examples
    ///
    /// ```text
    /// let opts = ParseOptions::builder()
    ///     .max_depth(100)
    ///     .strict(true)
    ///     .build();
    /// ```
    pub fn builder() -> ParseOptionsBuilder {
        ParseOptionsBuilder::new()
    }
}

/// Builder for ergonomic construction of ParseOptions.
///
/// Provides a fluent API for configuring parser options with sensible defaults.
///
/// # Examples
///
/// ```text
/// // Using builder with custom limits
/// let opts = ParseOptions::builder()
///     .max_depth(200)
///     .max_array_length(5000)
///     .strict(false)
///     .build();
///
/// // Using builder with defaults
/// let opts = ParseOptions::builder().build();
/// ```
#[derive(Debug, Clone)]
pub struct ParseOptionsBuilder {
    limits: Limits,
    strict_refs: bool,
}

impl ParseOptionsBuilder {
    /// Create a new builder with default options.
    pub fn new() -> Self {
        Self {
            limits: Limits::default(),
            strict_refs: true,
        }
    }

    /// Set the maximum nesting depth (indent depth).
    ///
    /// # Parameters
    ///
    /// - `depth`: Maximum nesting level (default: 50)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_depth(100)
    /// ```
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.limits.max_indent_depth = depth;
        self
    }

    /// Set the maximum array length (nodes in matrix lists).
    ///
    /// # Parameters
    ///
    /// - `length`: Maximum number of nodes (default: 10M)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_array_length(5000)
    /// ```
    pub fn max_array_length(mut self, length: usize) -> Self {
        self.limits.max_nodes = length;
        self
    }

    /// Set strict reference resolution mode.
    ///
    /// When `true`, unresolved references cause parsing errors.
    /// When `false`, unresolved references are silently ignored.
    ///
    /// # Parameters
    ///
    /// - `strict`: Whether to enforce strict reference resolution (default: true)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().strict(false)
    /// ```
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict_refs = strict;
        self
    }

    /// Set the maximum file size in bytes.
    ///
    /// # Parameters
    ///
    /// - `size`: Maximum file size in bytes (default: 1GB)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_file_size(500 * 1024 * 1024)
    /// ```
    pub fn max_file_size(mut self, size: usize) -> Self {
        self.limits.max_file_size = size;
        self
    }

    /// Set the maximum line length in bytes.
    ///
    /// # Parameters
    ///
    /// - `length`: Maximum line length in bytes (default: 1MB)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_line_length(512 * 1024)
    /// ```
    pub fn max_line_length(mut self, length: usize) -> Self {
        self.limits.max_line_length = length;
        self
    }

    /// Set the maximum number of aliases.
    ///
    /// # Parameters
    ///
    /// - `count`: Maximum number of aliases (default: 10k)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_aliases(5000)
    /// ```
    pub fn max_aliases(mut self, count: usize) -> Self {
        self.limits.max_aliases = count;
        self
    }

    /// Set the maximum columns per schema.
    ///
    /// # Parameters
    ///
    /// - `count`: Maximum columns (default: 100)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_columns(50)
    /// ```
    pub fn max_columns(mut self, count: usize) -> Self {
        self.limits.max_columns = count;
        self
    }

    /// Set the maximum NEST hierarchy depth.
    ///
    /// # Parameters
    ///
    /// - `depth`: Maximum nesting depth (default: 100)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_nest_depth(50)
    /// ```
    pub fn max_nest_depth(mut self, depth: usize) -> Self {
        self.limits.max_nest_depth = depth;
        self
    }

    /// Set the maximum block string size in bytes.
    ///
    /// # Parameters
    ///
    /// - `size`: Maximum block string size (default: 10MB)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_block_string_size(5 * 1024 * 1024)
    /// ```
    pub fn max_block_string_size(mut self, size: usize) -> Self {
        self.limits.max_block_string_size = size;
        self
    }

    /// Set the maximum keys per object.
    ///
    /// # Parameters
    ///
    /// - `count`: Maximum keys per object (default: 10k)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_object_keys(5000)
    /// ```
    pub fn max_object_keys(mut self, count: usize) -> Self {
        self.limits.max_object_keys = count;
        self
    }

    /// Set the maximum total keys across all objects.
    ///
    /// This provides defense-in-depth against memory exhaustion attacks.
    ///
    /// # Parameters
    ///
    /// - `count`: Maximum total keys (default: 10M)
    ///
    /// # Examples
    ///
    /// ```text
    /// ParseOptions::builder().max_total_keys(5_000_000)
    /// ```
    pub fn max_total_keys(mut self, count: usize) -> Self {
        self.limits.max_total_keys = count;
        self
    }

    /// Build the ParseOptions.
    pub fn build(self) -> ParseOptions {
        ParseOptions {
            limits: self.limits,
            strict_refs: self.strict_refs,
        }
    }
}

impl Default for ParseOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a HEDL document from bytes.
pub fn parse(input: &[u8]) -> HedlResult<Document> {
    parse_with_limits(input, ParseOptions::default())
}

/// Parse a HEDL document with custom options.
pub fn parse_with_limits(input: &[u8], options: ParseOptions) -> HedlResult<Document> {
    // Phase 1: Preprocess (zero-copy line splitting)
    let preprocessed = preprocess(input, &options.limits)?;

    // Collect lines as borrowed slices (no per-line allocation)
    let lines: Vec<(usize, &str)> = preprocessed.lines().collect();

    // Phase 2: Parse header
    let (header, body_start_idx) = parse_header(&lines, &options.limits)?;

    // Phase 3: Parse body
    let body_lines = &lines[body_start_idx..];
    let mut type_registries = TypeRegistry::new();
    let root = parse_body(body_lines, &header, &options.limits, &mut type_registries)?;

    // Build document
    let mut doc = Document::new(header.version);
    doc.aliases = header.aliases;
    doc.structs = header.structs;
    doc.nests = header.nests;
    doc.root = root;

    // Phase 4: Reference resolution
    resolve_references(&doc, options.strict_refs)?;

    Ok(doc)
}

// --- Context Stack ---

#[derive(Debug)]
enum Frame {
    Root {
        object: BTreeMap<String, Item>,
    },
    Object {
        indent: usize,
        key: String,
        object: BTreeMap<String, Item>,
    },
    List {
        #[allow(dead_code)]
        list_start_indent: usize,
        row_indent: usize,
        type_name: String,
        schema: Vec<String>,
        last_row_values: Option<Vec<Value>>,
        list: Vec<Node>,
        key: String,
        count_hint: Option<usize>,
    },
}

// --- Body Parsing ---

fn parse_body(
    lines: &[(usize, &str)],
    header: &crate::header::Header,
    limits: &Limits,
    type_registries: &mut TypeRegistry,
) -> HedlResult<BTreeMap<String, Item>> {
    let mut stack: Vec<Frame> = vec![Frame::Root {
        object: BTreeMap::new(),
    }];
    let mut node_count = 0usize;
    let mut total_keys = 0usize;
    let mut block_string: Option<BlockStringState> = None;

    for &(line_num, line) in lines {
        // Handle block string accumulation mode
        if let Some(ref mut state) = block_string {
            // Process the line and check if block string is complete
            if let Some(full_content) = state.process_line(line, line_num, limits)? {
                // Block string is complete
                let value = Value::String(full_content);
                pop_frames(&mut stack, state.indent);
                insert_into_current(&mut stack, state.key.clone(), Item::Scalar(value));
                block_string = None;
            }
            continue;
        }

        // Skip blank and comment lines
        if is_blank_line(line) || is_comment_line(line) {
            continue;
        }

        // Calculate indentation
        let indent_info =
            calculate_indent(line, line_num as u32).map_err(|e| HedlError::syntax(e.to_string(), line_num))?;

        let indent_info = match indent_info {
            Some(info) => info,
            None => continue, // Blank line
        };

        if indent_info.level > limits.max_indent_depth {
            return Err(HedlError::security(
                format!(
                    "indent depth {} exceeds limit {}",
                    indent_info.level, limits.max_indent_depth
                ),
                line_num,
            ));
        }

        let indent = indent_info.level;
        let content = &line[indent_info.spaces..];

        // Pop frames as needed based on indentation
        pop_frames(&mut stack, indent);

        // Classify and parse line
        if content.starts_with('|') {
            parse_matrix_row(
                &mut stack,
                content,
                indent,
                line_num,
                header,
                limits,
                type_registries,
                &mut node_count,
            )?;
        } else {
            // Check if this starts a block string
            match try_start_block_string(content, indent, line_num)? {
                BlockStringResult::MultiLineStarted(state) => {
                    // Validate indent and check for duplicate key
                    validate_indent_for_child(&stack, indent, line_num)?;
                    check_duplicate_key(&stack, &state.key, line_num, limits, &mut total_keys)?;
                    block_string = Some(state);
                }
                BlockStringResult::NotBlockString => {
                    parse_non_matrix_line(&mut stack, content, indent, line_num, header, limits, &mut total_keys)?;
                }
            }
        }
    }

    // Check for unclosed block string
    if let Some(state) = block_string {
        return Err(HedlError::syntax(
            format!(
                "unclosed block string starting at line {}",
                state.start_line
            ),
            state.start_line,
        ));
    }

    // Finalize: pop all frames and build result
    finalize_stack(stack)
}


fn pop_frames(stack: &mut Vec<Frame>, current_indent: usize) {
    while stack.len() > 1 {
        let should_pop = match stack.last().unwrap() {
            Frame::Root { .. } => false,
            Frame::Object { indent, .. } => current_indent <= *indent,
            Frame::List { row_indent, .. } => current_indent < *row_indent,
        };

        if should_pop {
            let frame = stack.pop().unwrap();
            attach_frame_to_parent(stack, frame);
        } else {
            break;
        }
    }
}

fn attach_frame_to_parent(stack: &mut [Frame], frame: Frame) {
    match frame {
        Frame::Object { key, object, .. } => {
            let item = Item::Object(object);
            insert_into_parent(stack, key, item);
        }
        Frame::List {
            key,
            type_name,
            schema,
            list,
            count_hint,
            ..
        } => {
            let mut matrix_list = if let Some(count) = count_hint {
                MatrixList::with_count_hint(type_name, schema, count)
            } else {
                MatrixList::new(type_name, schema)
            };
            matrix_list.rows = list;
            insert_into_parent(stack, key, Item::List(matrix_list));
        }
        Frame::Root { .. } => {}
    }
}

fn insert_into_parent(stack: &mut [Frame], key: String, item: Item) {
    if let Some(parent) = stack.last_mut() {
        match parent {
            Frame::Root { object } | Frame::Object { object, .. } => {
                // Note: max_object_keys limit check is performed at a higher level
                // during parsing, not here, to provide better error context
                object.insert(key, item);
            }
            Frame::List { list, .. } => {
                // Attach children to the last node in the list
                if let Some(parent_node) = list.last_mut() {
                    if let Item::List(child_list) = item {
                        parent_node
                            .children
                            .entry(child_list.type_name.clone())
                            .or_default()
                            .extend(child_list.rows);
                    }
                }
            }
        }
    }
}

fn parse_non_matrix_line(
    stack: &mut Vec<Frame>,
    content: &str,
    indent: usize,
    line_num: usize,
    header: &crate::header::Header,
    limits: &Limits,
    total_keys: &mut usize,
) -> HedlResult<()> {
    let content = strip_comment(content);

    // Find colon
    let colon_pos = content
        .find(':')
        .ok_or_else(|| HedlError::syntax("expected ':' in line", line_num))?;

    let key_with_hint = content[..colon_pos].trim();
    let after_colon = &content[colon_pos + 1..];

    // Extract count hint from key if present (e.g., "teams(3)")
    let (key, count_hint) = parse_key_with_count_hint(key_with_hint, line_num)?;

    // Validate key
    if !is_valid_key_token(&key) {
        return Err(HedlError::syntax(format!("invalid key: {}", key), line_num));
    }

    // Check for duplicate key
    check_duplicate_key(stack, &key, line_num, limits, total_keys)?;

    // Determine line type
    let after_colon_trimmed = after_colon.trim();

    if after_colon_trimmed.is_empty() {
        // Object start
        if count_hint.is_some() {
            return Err(HedlError::syntax(
                "count hint not allowed on object declarations",
                line_num,
            ));
        }
        validate_indent_for_child(stack, indent, line_num)?;
        stack.push(Frame::Object {
            indent,
            key: key.to_string(),
            object: BTreeMap::new(),
        });
    } else if after_colon_trimmed.starts_with('@') && is_list_start(after_colon_trimmed) {
        // Matrix list start
        if !after_colon.starts_with(' ') {
            return Err(HedlError::syntax(
                "space required after ':' before '@'",
                line_num,
            ));
        }

        // Check if this is a nested list declaration inside a list context
        let parent_list_idx = validate_nested_list_indent(stack, indent, line_num)?;

        let (type_name, schema) = parse_list_start(after_colon_trimmed, line_num, header, limits)?;

        if let Some(_parent_idx) = parent_list_idx {
            // This is a nested list inside a list context (e.g., divisions(3): @Division under a company row)
            // Push the new list frame - it will be attached to parent row when finalized
            stack.push(Frame::List {
                list_start_indent: indent,
                row_indent: indent + 1,
                type_name,
                schema,
                last_row_values: None,
                list: Vec::new(),
                key: key.to_string(),
                count_hint,
            });
        } else {
            // Normal top-level or object-nested list
            stack.push(Frame::List {
                list_start_indent: indent,
                row_indent: indent + 1,
                type_name,
                schema,
                last_row_values: None,
                list: Vec::new(),
                key: key.to_string(),
                count_hint,
            });
        }
    } else {
        // Key-value pair
        if count_hint.is_some() {
            return Err(HedlError::syntax(
                "count hint not allowed on scalar values",
                line_num,
            ));
        }
        if !after_colon.starts_with(' ') {
            return Err(HedlError::syntax(
                "space required after ':' in key-value",
                line_num,
            ));
        }
        validate_indent_for_child(stack, indent, line_num)?;
        let value_str = after_colon.trim();
        let ctx = InferenceContext::for_key_value(&header.aliases);
        let value = if value_str.starts_with('"') {
            // Quoted value
            let inner = parse_quoted_string(value_str, line_num)?;
            infer_quoted_value(&inner)
        } else {
            infer_value(value_str, &ctx, line_num)?
        };
        insert_into_current(stack, key.to_string(), Item::Scalar(value));
    }

    Ok(())
}

/// Parse a key that may have a count hint in parentheses.
/// Examples: "teams" -> ("teams", None), "teams(3)" -> ("teams", Some(3))
///
/// DEPRECATED: The `name(N): @Type` syntax for count hints is being replaced by
/// the new row-level `|N|data` syntax. This function is maintained for backward
/// compatibility but the old syntax is deprecated and may be removed in future versions.
fn parse_key_with_count_hint(key: &str, line_num: usize) -> HedlResult<(String, Option<usize>)> {
    if let Some(paren_pos) = key.find('(') {
        // Extract key and count
        let key_part = &key[..paren_pos];

        // Find closing parenthesis
        if !key.ends_with(')') {
            return Err(HedlError::syntax(
                "unclosed count hint parenthesis",
                line_num,
            ));
        }

        let count_str = &key[paren_pos + 1..key.len() - 1];

        // Parse count
        let count = count_str.parse::<usize>().map_err(|_| {
            HedlError::syntax(
                format!("invalid count hint: '{}'", count_str),
                line_num,
            )
        })?;

        if count == 0 {
            return Err(HedlError::syntax(
                "count hint must be greater than zero",
                line_num,
            ));
        }

        Ok((key_part.to_string(), Some(count)))
    } else {
        Ok((key.to_string(), None))
    }
}

fn is_list_start(s: &str) -> bool {
    // @TypeName or @TypeName[...]
    let s = s.trim();
    if !s.starts_with('@') {
        return false;
    }
    let rest = &s[1..];
    // Find end of type name
    let type_end = rest
        .find(|c: char| c == '[' || c.is_whitespace())
        .unwrap_or(rest.len());
    let type_name = &rest[..type_end];
    is_valid_type_name(type_name)
}

fn parse_list_start(
    s: &str,
    line_num: usize,
    header: &crate::header::Header,
    limits: &Limits,
) -> HedlResult<(String, Vec<String>)> {
    let s = s.trim();
    let rest = &s[1..]; // Skip @

    if let Some(bracket_pos) = rest.find('[') {
        // Inline schema: @TypeName[col1, col2]
        let type_name = &rest[..bracket_pos];
        if !is_valid_type_name(type_name) {
            return Err(HedlError::syntax(
                format!("invalid type name: {}", type_name),
                line_num,
            ));
        }

        let schema_str = &rest[bracket_pos..];
        let schema = parse_inline_schema(schema_str, line_num, limits)?;

        // Check against declared schema if exists
        if let Some(declared) = header.structs.get(type_name) {
            if declared != &schema {
                return Err(HedlError::schema(
                    format!(
                        "inline schema for '{}' doesn't match declared schema",
                        type_name
                    ),
                    line_num,
                ));
            }
        }

        Ok((type_name.to_string(), schema))
    } else {
        // Reference to declared schema: @TypeName
        let type_name = rest.trim();
        if !is_valid_type_name(type_name) {
            return Err(HedlError::syntax(
                format!("invalid type name: {}", type_name),
                line_num,
            ));
        }

        let schema = header
            .structs
            .get(type_name)
            .ok_or_else(|| HedlError::schema(format!("undefined type: {}", type_name), line_num))?;

        Ok((type_name.to_string(), schema.clone()))
    }
}

fn parse_inline_schema(s: &str, line_num: usize, limits: &Limits) -> HedlResult<Vec<String>> {
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(HedlError::syntax("invalid inline schema format", line_num));
    }

    let inner = &s[1..s.len() - 1];
    let mut columns = Vec::new();

    for part in inner.split(',') {
        let col = part.trim();
        if col.is_empty() {
            continue;
        }
        if !is_valid_key_token(col) {
            return Err(HedlError::syntax(
                format!("invalid column name: {}", col),
                line_num,
            ));
        }
        columns.push(col.to_string());
    }

    if columns.is_empty() {
        return Err(HedlError::syntax("empty inline schema", line_num));
    }

    if columns.len() > limits.max_columns {
        return Err(HedlError::security(
            format!("too many columns: {}", columns.len()),
            line_num,
        ));
    }

    Ok(columns)
}

/// Parse the row prefix to extract optional child count.
/// Patterns:
/// - `|[N] data` -> (Some(N), "data")  - parent with N children
/// - `|data`     -> (None, "data")     - leaf node (no count)
fn parse_row_prefix(content: &str, line_num: usize) -> HedlResult<(Option<usize>, &str)> {
    // Content should start with |
    if !content.starts_with('|') {
        return Err(HedlError::syntax(
            "matrix row must start with '|'",
            line_num,
        ));
    }

    let rest = &content[1..]; // Skip first |

    // Check for |[N] pattern
    if rest.starts_with('[') {
        if let Some(bracket_end) = rest.find(']') {
            let count_str = &rest[1..bracket_end];
            if let Ok(count) = count_str.parse::<usize>() {
                // Count 0 is valid - means row has no children (empty parent)
                // Skip |[N] and any following space
                let data = rest[bracket_end + 1..].trim_start();
                return Ok((Some(count), data));
            }
        }
    }

    // No count pattern, treat as |data (leaf node)
    Ok((None, rest))
}

#[allow(clippy::too_many_arguments)]
fn parse_matrix_row(
    stack: &mut Vec<Frame>,
    content: &str,
    indent: usize,
    line_num: usize,
    header: &crate::header::Header,
    limits: &Limits,
    type_registries: &mut TypeRegistry,
    node_count: &mut usize,
) -> HedlResult<()> {
    // Find the active list frame
    let list_frame_idx = find_list_frame(stack, indent, line_num, header, limits)?;

    // Parse the row prefix to extract optional child count and CSV content
    let (child_count, csv_content) = parse_row_prefix(content, line_num)?;
    let csv_content = strip_comment(csv_content).trim();

    // Get list info
    let (type_name, schema, prev_row) = {
        let frame = &stack[list_frame_idx];
        match frame {
            Frame::List {
                type_name,
                schema,
                last_row_values,
                ..
            } => (type_name.clone(), schema.clone(), last_row_values.clone()),
            _ => unreachable!(),
        }
    };

    // Parse CSV
    let fields =
        parse_csv_row(csv_content).map_err(|e| HedlError::syntax(e.to_string(), line_num))?;

    // Validate shape
    if fields.len() != schema.len() {
        return Err(HedlError::shape(
            format!("expected {} columns, got {}", schema.len(), fields.len()),
            line_num,
        ));
    }

    // Infer values
    let mut values = Vec::with_capacity(fields.len());
    for (col_idx, field) in fields.iter().enumerate() {
        let ctx = InferenceContext::for_matrix_cell(
            &header.aliases,
            col_idx,
            prev_row.as_deref(),
            &type_name,
        );

        let value = if field.is_quoted {
            infer_quoted_value(&field.value)
        } else {
            infer_value(&field.value, &ctx, line_num)?
        };

        values.push(value);
    }

    // Get ID from first column
    let id = match &values[0] {
        Value::String(s) => s.clone(),
        _ => {
            return Err(HedlError::semantic("ID column must be a string", line_num));
        }
    };

    // Register node ID
    register_node(type_registries, &type_name, &id, line_num)?;

    // Check node count limit with checked arithmetic to prevent overflow
    *node_count = node_count.checked_add(1).ok_or_else(|| {
        HedlError::security("node count overflow", line_num)
    })?;
    if *node_count > limits.max_nodes {
        return Err(HedlError::security(
            format!("too many nodes: exceeds limit of {}", limits.max_nodes),
            line_num,
        ));
    }

    // Update list frame - avoid clone by storing values first, then creating node
    if let Frame::List {
        last_row_values,
        list,
        ..
    } = &mut stack[list_frame_idx]
    {
        // Store values for ditto support before moving to node
        *last_row_values = Some(values.clone());
        // Create node taking ownership of values - no extra clone needed
        let mut node = Node::new(&type_name, &id, values);

        // Store child count from |N| syntax if present
        if let Some(count) = child_count {
            node.set_child_count(count);
        }

        list.push(node);
    }

    Ok(())
}

/// Finds the appropriate list frame for a matrix row at the given indent level.
///
/// This function performs critical depth checking to prevent stack overflow attacks
/// via deeply nested NEST hierarchies. When a child row is detected (indent = parent + 1),
/// it validates that adding a new NEST level would not exceed `max_nest_depth`.
///
/// # Security
///
/// **DoS Prevention**: Without depth limits, an attacker could craft a HEDL document
/// with thousands of nested NEST levels, causing stack overflow or excessive memory
/// consumption during parsing. The depth check prevents this attack vector.
///
/// # Parameters
///
/// - `stack`: The parsing stack containing current frame hierarchy
/// - `indent`: Indentation level of the current matrix row
/// - `line_num`: Line number for error reporting
/// - `header`: Document header containing NEST rules and schemas
/// - `limits`: Security limits including `max_nest_depth`
///
/// # Returns
///
/// Returns the index of the list frame where this row should be added.
///
/// # Errors
///
/// - `HedlError::Security` if nesting depth exceeds `limits.max_nest_depth`
/// - `HedlError::OrphanRow` if child row has no parent or no NEST rule exists
/// - `HedlError::Schema` if child type is not defined
/// - `HedlError::Syntax` if row is outside list context
///
/// # Examples
///
/// ```text
/// # Valid nested structure within depth limit
/// TYPE Person id name
/// TYPE Address street city
/// NEST Person Address
///
/// Person
/// 1, Alice    # depth 0
///   1, Main St, NYC    # depth 1 - child of Person row
/// ```
fn find_list_frame(
    stack: &mut Vec<Frame>,
    indent: usize,
    line_num: usize,
    header: &crate::header::Header,
    limits: &Limits,
) -> HedlResult<usize> {
    // Look for a list frame where this indent makes sense
    for (idx, frame) in stack.iter().enumerate().rev() {
        if let Frame::List {
            row_indent,
            type_name,
            list,
            ..
        } = frame
        {
            if indent == *row_indent {
                // Peer row
                return Ok(idx);
            } else if indent == *row_indent + 1 {
                // Child row - need NEST rule
                // Check if there's a parent row to attach to
                if list.is_empty() {
                    return Err(HedlError::orphan_row(
                        "child row has no parent row",
                        line_num,
                    ));
                }

                let child_type = header.nests.get(type_name).ok_or_else(|| {
                    HedlError::orphan_row(
                        format!("no NEST rule for parent type '{}'", type_name),
                        line_num,
                    )
                })?;

                // Get child schema
                let child_schema = header.structs.get(child_type).ok_or_else(|| {
                    HedlError::schema(format!("child type '{}' not defined", child_type), line_num)
                })?;

                // SECURITY: Check NEST depth before pushing child frame to prevent DoS
                // Count current depth by counting List frames in the stack
                // Each List frame represents one level in the NEST hierarchy
                let current_depth = stack.iter().filter(|f| matches!(f, Frame::List { .. })).count();

                if current_depth >= limits.max_nest_depth {
                    return Err(HedlError::security(
                        format!(
                            "NEST hierarchy depth {} exceeds maximum allowed depth {}",
                            current_depth + 1, limits.max_nest_depth
                        ),
                        line_num,
                    ));
                }

                // Push a new list frame for the child
                stack.push(Frame::List {
                    list_start_indent: indent - 1,
                    row_indent: indent,
                    type_name: child_type.clone(),
                    schema: child_schema.clone(),
                    last_row_values: None,
                    list: Vec::new(),
                    key: child_type.clone(),
                    count_hint: None, // Child lists from NEST don't have count hints
                });

                return Ok(stack.len() - 1);
            }
        }
    }

    Err(HedlError::syntax(
        "matrix row outside of list context",
        line_num,
    ))
}

fn validate_indent_for_child(stack: &[Frame], indent: usize, line_num: usize) -> HedlResult<()> {
    let expected = match stack.last() {
        Some(Frame::Root { .. }) => 0,
        Some(Frame::Object {
            indent: parent_indent,
            ..
        }) => parent_indent + 1,
        Some(Frame::List { row_indent: _, .. }) => {
            return Err(HedlError::syntax(
                "cannot add key-value inside list context",
                line_num,
            ));
        }
        None => 0,
    };

    if indent != expected {
        return Err(HedlError::syntax(
            format!("expected indent level {}, got {}", expected, indent),
            line_num,
        ));
    }

    Ok(())
}

/// Validate indent for nested list declarations inside a list context.
/// Unlike scalar key-values, nested list declarations ARE allowed inside lists.
/// Returns the parent list frame index if valid, or error if invalid.
fn validate_nested_list_indent(
    stack: &[Frame],
    indent: usize,
    line_num: usize,
) -> HedlResult<Option<usize>> {
    // Check if we're inside a list context
    for (idx, frame) in stack.iter().enumerate().rev() {
        match frame {
            Frame::List { row_indent, list, .. } => {
                // Nested list declaration should be at row_indent + 1 (child level)
                if indent == *row_indent + 1 {
                    // Must have a parent row to attach to
                    if list.is_empty() {
                        return Err(HedlError::orphan_row(
                            "nested list declaration has no parent row",
                            line_num,
                        ));
                    }
                    return Ok(Some(idx));
                }
            }
            Frame::Root { .. } => {
                if indent == 0 {
                    return Ok(None); // Normal top-level list
                }
            }
            Frame::Object { indent: obj_indent, .. } => {
                if indent == obj_indent + 1 {
                    return Ok(None); // Normal list inside object
                }
            }
        }
    }

    Err(HedlError::syntax(
        format!("invalid indent level {} for nested list declaration", indent),
        line_num,
    ))
}

/// Check for duplicate keys and enforce security limits.
///
/// This function validates that:
/// 1. The key is not already present in the current object
/// 2. The object doesn't exceed max_object_keys limit
/// 3. The total number of keys across all objects doesn't exceed max_total_keys limit
///
/// # Security
///
/// The total_keys counter prevents DoS attacks where an attacker creates many small
/// objects, each under the max_object_keys limit, but collectively consuming excessive
/// memory. This provides defense-in-depth against memory exhaustion attacks.
fn check_duplicate_key(
    stack: &[Frame],
    key: &str,
    line_num: usize,
    limits: &Limits,
    total_keys: &mut usize,
) -> HedlResult<()> {
    let object_opt = match stack.last() {
        Some(Frame::Root { object }) | Some(Frame::Object { object, .. }) => Some(object),
        _ => None,
    };

    if let Some(object) = object_opt {
        // Check for duplicate key
        if object.contains_key(key) {
            return Err(HedlError::semantic(
                format!("duplicate key: {}", key),
                line_num,
            ));
        }

        // Security: Enforce max_object_keys limit to prevent memory exhaustion per object
        if object.len() >= limits.max_object_keys {
            return Err(HedlError::security(
                format!(
                    "object has too many keys: {} (max: {})",
                    object.len() + 1,
                    limits.max_object_keys
                ),
                line_num,
            ));
        }

        // Security: Enforce max_total_keys limit to prevent cumulative memory exhaustion
        *total_keys = total_keys.checked_add(1).ok_or_else(|| {
            HedlError::security("total key count overflow", line_num)
        })?;

        if *total_keys > limits.max_total_keys {
            return Err(HedlError::security(
                format!(
                    "too many total keys: {} exceeds limit {}",
                    *total_keys, limits.max_total_keys
                ),
                line_num,
            ));
        }
    }

    Ok(())
}

fn insert_into_current(stack: &mut [Frame], key: String, item: Item) {
    if let Some(Frame::Root { object } | Frame::Object { object, .. }) = stack.last_mut() {
        object.insert(key, item);
    }
}

fn parse_quoted_string(s: &str, line_num: usize) -> HedlResult<String> {
    if !s.starts_with('"') {
        return Err(HedlError::syntax("expected quoted string", line_num));
    }

    let mut result = String::new();
    let mut chars = s[1..].chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '"' {
            if chars.peek() == Some(&'"') {
                // Escaped quote
                chars.next();
                result.push('"');
            } else {
                // End of string
                return Ok(result);
            }
        } else {
            result.push(ch);
        }
    }

    Err(HedlError::syntax("unclosed quoted string", line_num))
}

fn finalize_stack(mut stack: Vec<Frame>) -> HedlResult<BTreeMap<String, Item>> {
    // Per SPEC Section 14.5: Detect truncated input.
    // Check only the DEEPEST (last) non-Root frame for truncation.
    // Intermediate frames will be empty until children are attached during pop.
    // Only if the deepest frame is an empty Object do we have actual truncation.
    // Note: Empty lists declared with @TypeName are allowed.
    if stack.len() > 1 {
        if let Some(Frame::Object { key, object, .. }) = stack.last() {
            if object.is_empty() {
                return Err(HedlError::syntax(
                    format!("truncated input: object '{}' has no children", key),
                    0,
                ));
            }
        }
    }

    // Pop all frames back to root
    while stack.len() > 1 {
        let frame = stack.pop().unwrap();
        attach_frame_to_parent(&mut stack, frame);
    }

    // Extract root object
    match stack.pop() {
        Some(Frame::Root { object }) => Ok(object),
        _ => Ok(BTreeMap::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ParseOptionsBuilder::new() tests ====================

    #[test]
    fn test_builder_new_creates_default_options() {
        let builder = ParseOptionsBuilder::new();
        let opts = builder.build();

        assert_eq!(opts.strict_refs, true);
        assert_eq!(opts.limits.max_indent_depth, 50);
        assert_eq!(opts.limits.max_nodes, 10_000_000);
    }

    #[test]
    fn test_builder_default_trait() {
        let builder1 = ParseOptionsBuilder::new();
        let builder2 = ParseOptionsBuilder::default();
        let opts1 = builder1.build();
        let opts2 = builder2.build();

        assert_eq!(opts1.strict_refs, opts2.strict_refs);
        assert_eq!(opts1.limits.max_indent_depth, opts2.limits.max_indent_depth);
    }

    // ==================== ParseOptions::builder() tests ====================

    #[test]
    fn test_parse_options_builder_method() {
        let opts = ParseOptions::builder().build();
        assert_eq!(opts.strict_refs, true);
    }

    // ==================== Chainable method tests ====================

    #[test]
    fn test_builder_max_depth() {
        let opts = ParseOptions::builder()
            .max_depth(100)
            .build();

        assert_eq!(opts.limits.max_indent_depth, 100);
    }

    #[test]
    fn test_builder_max_array_length() {
        let opts = ParseOptions::builder()
            .max_array_length(5000)
            .build();

        assert_eq!(opts.limits.max_nodes, 5000);
    }

    #[test]
    fn test_builder_strict_true() {
        let opts = ParseOptions::builder()
            .strict(true)
            .build();

        assert_eq!(opts.strict_refs, true);
    }

    #[test]
    fn test_builder_strict_false() {
        let opts = ParseOptions::builder()
            .strict(false)
            .build();

        assert_eq!(opts.strict_refs, false);
    }

    #[test]
    fn test_builder_max_file_size() {
        let size = 500 * 1024 * 1024;
        let opts = ParseOptions::builder()
            .max_file_size(size)
            .build();

        assert_eq!(opts.limits.max_file_size, size);
    }

    #[test]
    fn test_builder_max_line_length() {
        let length = 512 * 1024;
        let opts = ParseOptions::builder()
            .max_line_length(length)
            .build();

        assert_eq!(opts.limits.max_line_length, length);
    }

    #[test]
    fn test_builder_max_aliases() {
        let opts = ParseOptions::builder()
            .max_aliases(5000)
            .build();

        assert_eq!(opts.limits.max_aliases, 5000);
    }

    #[test]
    fn test_builder_max_columns() {
        let opts = ParseOptions::builder()
            .max_columns(50)
            .build();

        assert_eq!(opts.limits.max_columns, 50);
    }

    #[test]
    fn test_builder_max_nest_depth() {
        let opts = ParseOptions::builder()
            .max_nest_depth(50)
            .build();

        assert_eq!(opts.limits.max_nest_depth, 50);
    }

    #[test]
    fn test_builder_max_block_string_size() {
        let size = 5 * 1024 * 1024;
        let opts = ParseOptions::builder()
            .max_block_string_size(size)
            .build();

        assert_eq!(opts.limits.max_block_string_size, size);
    }

    #[test]
    fn test_builder_max_object_keys() {
        let opts = ParseOptions::builder()
            .max_object_keys(5000)
            .build();

        assert_eq!(opts.limits.max_object_keys, 5000);
    }

    #[test]
    fn test_builder_max_total_keys() {
        let opts = ParseOptions::builder()
            .max_total_keys(5_000_000)
            .build();

        assert_eq!(opts.limits.max_total_keys, 5_000_000);
    }

    // ==================== Multiple chained methods tests ====================

    #[test]
    fn test_builder_multiple_chains() {
        let opts = ParseOptions::builder()
            .max_depth(100)
            .max_array_length(5000)
            .strict(false)
            .build();

        assert_eq!(opts.limits.max_indent_depth, 100);
        assert_eq!(opts.limits.max_nodes, 5000);
        assert_eq!(opts.strict_refs, false);
    }

    #[test]
    fn test_builder_all_options_chained() {
        let opts = ParseOptions::builder()
            .max_depth(75)
            .max_array_length(2000)
            .strict(false)
            .max_file_size(100 * 1024 * 1024)
            .max_line_length(256 * 1024)
            .max_aliases(1000)
            .max_columns(25)
            .max_nest_depth(30)
            .max_block_string_size(1024 * 1024)
            .max_object_keys(1000)
            .max_total_keys(1_000_000)
            .build();

        assert_eq!(opts.limits.max_indent_depth, 75);
        assert_eq!(opts.limits.max_nodes, 2000);
        assert_eq!(opts.strict_refs, false);
        assert_eq!(opts.limits.max_file_size, 100 * 1024 * 1024);
        assert_eq!(opts.limits.max_line_length, 256 * 1024);
        assert_eq!(opts.limits.max_aliases, 1000);
        assert_eq!(opts.limits.max_columns, 25);
        assert_eq!(opts.limits.max_nest_depth, 30);
        assert_eq!(opts.limits.max_block_string_size, 1024 * 1024);
        assert_eq!(opts.limits.max_object_keys, 1000);
        assert_eq!(opts.limits.max_total_keys, 1_000_000);
    }

    // ==================== Override tests ====================

    #[test]
    fn test_builder_override_previous_value() {
        let opts = ParseOptions::builder()
            .max_depth(50)
            .max_depth(100)
            .build();

        assert_eq!(opts.limits.max_indent_depth, 100);
    }

    #[test]
    fn test_builder_override_multiple_times() {
        let opts = ParseOptions::builder()
            .max_array_length(1000)
            .max_array_length(2000)
            .max_array_length(3000)
            .build();

        assert_eq!(opts.limits.max_nodes, 3000);
    }

    // ==================== Default behavior tests ====================

    #[test]
    fn test_builder_default_keeps_other_defaults() {
        let opts = ParseOptions::builder()
            .max_depth(100)
            .build();

        assert_eq!(opts.limits.max_indent_depth, 100);
        // Other values should remain at defaults
        assert_eq!(opts.limits.max_file_size, 1024 * 1024 * 1024);
        assert_eq!(opts.limits.max_line_length, 1024 * 1024);
        assert_eq!(opts.limits.max_nodes, 10_000_000);
        assert_eq!(opts.strict_refs, true);
    }

    // ==================== Edge case tests ====================

    #[test]
    fn test_builder_zero_values() {
        let opts = ParseOptions::builder()
            .max_depth(0)
            .max_array_length(0)
            .max_aliases(0)
            .build();

        assert_eq!(opts.limits.max_indent_depth, 0);
        assert_eq!(opts.limits.max_nodes, 0);
        assert_eq!(opts.limits.max_aliases, 0);
    }

    #[test]
    fn test_builder_max_values() {
        let opts = ParseOptions::builder()
            .max_depth(usize::MAX)
            .max_array_length(usize::MAX)
            .max_file_size(usize::MAX)
            .build();

        assert_eq!(opts.limits.max_indent_depth, usize::MAX);
        assert_eq!(opts.limits.max_nodes, usize::MAX);
        assert_eq!(opts.limits.max_file_size, usize::MAX);
    }

    // ==================== Equivalence tests ====================

    #[test]
    fn test_builder_build_equivalent_to_default() {
        let builder_opts = ParseOptions::builder().build();
        let default_opts = ParseOptions::default();

        assert_eq!(builder_opts.strict_refs, default_opts.strict_refs);
        assert_eq!(builder_opts.limits.max_indent_depth, default_opts.limits.max_indent_depth);
        assert_eq!(builder_opts.limits.max_nodes, default_opts.limits.max_nodes);
        assert_eq!(builder_opts.limits.max_file_size, default_opts.limits.max_file_size);
    }

    #[test]
    fn test_builder_clone_independent() {
        let builder1 = ParseOptions::builder().max_depth(100);
        let builder2 = builder1.clone().max_depth(200);

        let opts1 = builder1.build();
        let opts2 = builder2.build();

        assert_eq!(opts1.limits.max_indent_depth, 100);
        assert_eq!(opts2.limits.max_indent_depth, 200);
    }

    // ==================== Usage pattern tests ====================

    #[test]
    fn test_builder_typical_usage_pattern() {
        // Typical use case: strict parsing with moderate limits
        let opts = ParseOptions::builder()
            .max_depth(100)
            .strict(true)
            .build();

        assert!(opts.strict_refs);
        assert_eq!(opts.limits.max_indent_depth, 100);
    }

    #[test]
    fn test_builder_lenient_parsing_pattern() {
        // Lenient parsing with higher limits
        let opts = ParseOptions::builder()
            .max_array_length(50_000)
            .strict(false)
            .max_block_string_size(50 * 1024 * 1024)
            .build();

        assert!(!opts.strict_refs);
        assert_eq!(opts.limits.max_nodes, 50_000);
        assert_eq!(opts.limits.max_block_string_size, 50 * 1024 * 1024);
    }

    #[test]
    fn test_builder_restricted_parsing_pattern() {
        // Restricted parsing for security
        let opts = ParseOptions::builder()
            .max_file_size(10 * 1024 * 1024)
            .max_line_length(64 * 1024)
            .max_depth(20)
            .max_array_length(1000)
            .strict(true)
            .build();

        assert_eq!(opts.limits.max_file_size, 10 * 1024 * 1024);
        assert_eq!(opts.limits.max_line_length, 64 * 1024);
        assert_eq!(opts.limits.max_indent_depth, 20);
        assert_eq!(opts.limits.max_nodes, 1000);
        assert!(opts.strict_refs);
    }
}
