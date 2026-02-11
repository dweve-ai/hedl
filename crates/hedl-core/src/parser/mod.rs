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

mod context;
mod line_parsing;
mod options;
mod utils;

// Re-export public types
pub use options::{ParseOptions, ParseOptionsBuilder};

use crate::block_string::{try_start_block_string, BlockStringResult, BlockStringState};
use crate::document::{Document, Item};
use crate::error::{HedlError, HedlResult};
use crate::header::parse_header;
use crate::lex::calculate_indent;
use crate::limits::{Limits, TimeoutCheckExt, TimeoutContext};
use crate::preprocess::{is_blank_line, is_comment_line, preprocess};
use crate::reference::{resolve_references, TypeRegistry};
use crate::value::Value;
use context::{pop_frames, Frame};
use line_parsing::{
    is_expanded_child_list, is_inline_child_list, parse_expanded_child_list,
    parse_inline_child_list, parse_matrix_row, parse_non_matrix_line, MatrixParseParams,
};
use std::collections::BTreeMap;
use utils::{check_duplicate_key, finalize_stack, insert_into_current, validate_indent_for_child};

/// Recommended maximum inline children in `@Type#N:|...` syntax.
/// Per SPEC v2.0 line 58: "Style rule (not a hard syntax limit): keep inline N <= 10"
/// This is NOT enforced by the parser; use hedl-lint for style warnings.
const _STYLE_INLINE_CHILDREN_LIMIT: usize = 10;

/// Parse a HEDL document from bytes.
pub fn parse(input: &[u8]) -> HedlResult<Document> {
    parse_with_limits(input, ParseOptions::default())
}

/// Parse a HEDL document with custom options.
pub fn parse_with_limits(input: &[u8], options: ParseOptions) -> HedlResult<Document> {
    // Create timeout context for parsing
    let timeout_ctx = TimeoutContext::new(options.limits.timeout);

    // Phase 1: Preprocess (zero-copy line splitting)
    let preprocessed = preprocess(input, &options.limits)?;

    // Collect lines as borrowed slices (no per-line allocation)
    let lines: Vec<(usize, &str)> = preprocessed.lines().collect();

    // Phase 2: Parse header
    let (header, body_start_idx) = parse_header(&lines, &options.limits, &timeout_ctx)?;

    // Phase 3: Parse body
    let body_lines = &lines[body_start_idx..];
    let mut type_registries = TypeRegistry::new();
    let root = parse_body(
        body_lines,
        &header,
        &options.limits,
        &mut type_registries,
        &timeout_ctx,
    )?;

    // Build document
    let mut doc = Document::new(header.version);
    doc.aliases = header.aliases;
    doc.structs = header.structs;
    doc.nests = header.nests;
    doc.root = root;

    // Phase 4: Reference resolution (with timeout check)
    timeout_ctx.check_timeout(0)?;
    resolve_references(&doc, options.reference_mode)?;

    Ok(doc)
}

/// Context for body parsing, holding references to shared state.
struct ParseContext<'a> {
    header: &'a crate::header::Header,
    limits: &'a Limits,
    type_registries: &'a mut TypeRegistry,
    node_count: &'a mut usize,
}

fn parse_body(
    lines: &[(usize, &str)],
    header: &crate::header::Header,
    limits: &Limits,
    type_registries: &mut TypeRegistry,
    timeout_ctx: &TimeoutContext,
) -> HedlResult<BTreeMap<String, Item>> {
    let mut stack: Vec<Frame> = vec![Frame::Root {
        object: BTreeMap::new(),
    }];
    let mut node_count = 0usize;
    let mut total_keys = 0usize;
    let mut block_string: Option<BlockStringState> = None;

    // Create parsing context once for reuse throughout the loop
    let ctx = ParseContext {
        header,
        limits,
        type_registries,
        node_count: &mut node_count,
    };

    // Automatic timeout checking every 10,000 iterations
    for result in lines.iter().copied().with_timeout_check(timeout_ctx) {
        let (line_num, line) = result?;
        // Handle block string accumulation mode
        if let Some(ref mut state) = block_string {
            // Process the line and check if block string is complete
            if let Some(full_content) = state.process_line(line, line_num, limits)? {
                // Block string is complete
                let value = Value::String(full_content.into());
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
        let indent_info = calculate_indent(line, line_num as u32)
            .map_err(|e| HedlError::syntax(e.to_string(), line_num))?;

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
            let params = MatrixParseParams {
                content,
                indent,
                line_num,
                header: ctx.header,
                limits: ctx.limits,
            };
            parse_matrix_row(&mut stack, &params, ctx.type_registries, ctx.node_count)?;
        } else if content.starts_with('@') && is_inline_child_list(content) {
            // Inline child list:@Type#N:|child1|child2|...
            let params = MatrixParseParams {
                content,
                indent,
                line_num,
                header: ctx.header,
                limits: ctx.limits,
            };
            parse_inline_child_list(&mut stack, &params, ctx.type_registries, ctx.node_count)?;
        } else if content.starts_with('@') && is_expanded_child_list(content) {
            // Expanded child list:@Type#N: (children on following lines)
            parse_expanded_child_list(
                &mut stack, content, indent, line_num, ctx.header, ctx.limits,
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
                    parse_non_matrix_line(
                        &mut stack,
                        content,
                        indent,
                        line_num,
                        header,
                        limits,
                        &mut total_keys,
                    )?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference::ReferenceMode;

    // ==================== ParseOptionsBuilder::new() tests ====================

    #[test]
    fn test_builder_new_creates_default_options() {
        let builder = ParseOptionsBuilder::new();
        let opts = builder.build();

        assert_eq!(opts.reference_mode, ReferenceMode::Strict);
        assert_eq!(opts.limits.max_indent_depth, 50);
        assert_eq!(opts.limits.max_nodes, 10_000_000);
    }

    #[test]
    fn test_builder_default_trait() {
        let builder1 = ParseOptionsBuilder::new();
        let builder2 = ParseOptionsBuilder::default();
        let opts1 = builder1.build();
        let opts2 = builder2.build();

        assert_eq!(opts1.reference_mode, opts2.reference_mode);
        assert_eq!(opts1.limits.max_indent_depth, opts2.limits.max_indent_depth);
    }

    // ==================== ParseOptions::builder() tests ====================

    #[test]
    fn test_parse_options_builder_method() {
        let opts = ParseOptions::builder().build();
        assert_eq!(opts.reference_mode, ReferenceMode::Strict);
    }

    // ==================== Chainable method tests ====================

    #[test]
    fn test_builder_max_depth() {
        let opts = ParseOptions::builder().max_depth(100).build();

        assert_eq!(opts.limits.max_indent_depth, 100);
    }

    #[test]
    fn test_builder_max_array_length() {
        let opts = ParseOptions::builder().max_array_length(5000).build();

        assert_eq!(opts.limits.max_nodes, 5000);
    }

    #[test]
    fn test_builder_strict_true() {
        let opts = ParseOptions::builder().strict(true).build();

        assert_eq!(opts.reference_mode, ReferenceMode::Strict);
    }

    #[test]
    fn test_builder_strict_false() {
        let opts = ParseOptions::builder().strict(false).build();

        assert_eq!(opts.reference_mode, ReferenceMode::Lenient);
    }

    #[test]
    fn test_builder_max_file_size() {
        let size = 500 * 1024 * 1024;
        let opts = ParseOptions::builder().max_file_size(size).build();

        assert_eq!(opts.limits.max_file_size, size);
    }

    #[test]
    fn test_builder_max_line_length() {
        let length = 512 * 1024;
        let opts = ParseOptions::builder().max_line_length(length).build();

        assert_eq!(opts.limits.max_line_length, length);
    }

    #[test]
    fn test_builder_max_aliases() {
        let opts = ParseOptions::builder().max_aliases(5000).build();

        assert_eq!(opts.limits.max_aliases, 5000);
    }

    #[test]
    fn test_builder_max_columns() {
        let opts = ParseOptions::builder().max_columns(50).build();

        assert_eq!(opts.limits.max_columns, 50);
    }

    #[test]
    fn test_builder_max_nest_depth() {
        let opts = ParseOptions::builder().max_nest_depth(50).build();

        assert_eq!(opts.limits.max_nest_depth, 50);
    }

    #[test]
    fn test_builder_max_block_string_size() {
        let size = 5 * 1024 * 1024;
        let opts = ParseOptions::builder().max_block_string_size(size).build();

        assert_eq!(opts.limits.max_block_string_size, size);
    }

    #[test]
    fn test_builder_max_object_keys() {
        let opts = ParseOptions::builder().max_object_keys(5000).build();

        assert_eq!(opts.limits.max_object_keys, 5000);
    }

    #[test]
    fn test_builder_max_total_keys() {
        let opts = ParseOptions::builder().max_total_keys(5_000_000).build();

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
        assert_eq!(opts.reference_mode, ReferenceMode::Lenient);
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
        assert_eq!(opts.reference_mode, ReferenceMode::Lenient);
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
        let opts = ParseOptions::builder().max_depth(50).max_depth(100).build();

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
        let opts = ParseOptions::builder().max_depth(100).build();

        assert_eq!(opts.limits.max_indent_depth, 100);
        // Other values should remain at defaults
        assert_eq!(opts.limits.max_file_size, 1024 * 1024 * 1024);
        assert_eq!(opts.limits.max_line_length, 1024 * 1024);
        assert_eq!(opts.limits.max_nodes, 10_000_000);
        assert_eq!(opts.reference_mode, ReferenceMode::Strict);
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

        assert_eq!(builder_opts.reference_mode, default_opts.reference_mode);
        assert_eq!(
            builder_opts.limits.max_indent_depth,
            default_opts.limits.max_indent_depth
        );
        assert_eq!(builder_opts.limits.max_nodes, default_opts.limits.max_nodes);
        assert_eq!(
            builder_opts.limits.max_file_size,
            default_opts.limits.max_file_size
        );
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
        let opts = ParseOptions::builder().max_depth(100).strict(true).build();

        assert_eq!(opts.reference_mode, ReferenceMode::Strict);
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

        assert_eq!(opts.reference_mode, ReferenceMode::Lenient);
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
        assert_eq!(opts.reference_mode, ReferenceMode::Strict);
    }

    // ==================== Timeout integration tests ====================

    #[test]
    fn test_parse_with_generous_timeout_succeeds() {
        let doc = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\n";
        let mut opts = ParseOptions::default();
        opts.limits.timeout = Some(std::time::Duration::from_secs(10));
        let result = parse_with_limits(doc, opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_no_timeout_succeeds() {
        let doc = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\n";
        let mut opts = ParseOptions::default();
        opts.limits.timeout = None;
        let result = parse_with_limits(doc, opts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_with_very_short_timeout_fails() {
        // Create a document large enough to take some time
        let mut doc = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ndata:\n");
        for i in 0..100_000 {
            doc.push_str(&format!("  key{}: value{}\n", i, i));
        }

        let mut opts = ParseOptions::default();
        // Set an impossibly short timeout (1 microsecond)
        opts.limits.timeout = Some(std::time::Duration::from_micros(1));

        let result = parse_with_limits(doc.as_bytes(), opts);
        assert!(result.is_err());

        if let Err(e) = result {
            let msg = e.to_string();
            assert!(msg.contains("timeout") || msg.contains("Timeout"));
        }
    }

    #[test]
    fn test_default_timeout_is_reasonable() {
        let opts = ParseOptions::default();
        assert_eq!(
            opts.limits.timeout,
            Some(std::time::Duration::from_secs(30))
        );
    }

    #[test]
    fn test_unlimited_has_no_timeout() {
        let limits = Limits::unlimited();
        assert_eq!(limits.timeout, None);
    }
}
