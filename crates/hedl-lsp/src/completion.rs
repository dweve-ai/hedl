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

//! Autocompletion for HEDL files.
//!
//! This module provides context-aware autocompletion for HEDL documents,
//! suggesting appropriate completions based on cursor position and
//! surrounding syntax.
//!
//! # Completion Contexts
//!
//! The system recognizes several distinct contexts:
//!
//! - **Header**: Header directives (%VERSION, %STRUCT, %ALIAS, %NEST, %MODE, %PROMPT)
//! - **Reference**: Type names after @ symbol
//! - **`ReferenceId`**: Entity IDs after @Type:
//! - **`ListType`**: Type names in list declarations
//! - **`MatrixCell`**: Values in matrix cells (null, booleans, references, enum codes, ditto in pre-v2.0 only)
//! - **Key**: Property keys in object notation
//! - **Value**: Property values (aliases, type references)
//!
//! # Examples
//!
//! ```text
//! %STRUCT U|         → Suggests STRUCT completion
//! users:@U|         → Suggests User type
//! @User:|            → Suggests entity IDs for User type
//! | alice | @U|      → Suggests references in matrix cell
//! ```

use crate::analysis::AnalyzedDocument;
use crate::utf_encoding::safe_slice_to;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, Documentation, InsertTextFormat, Position,
};

/// Completion context for determining what to suggest.
///
/// This enum represents the different syntactic contexts in a HEDL document
/// where completions can be provided. Each variant contains the information
/// needed to generate appropriate completion items.
#[derive(Debug, Clone, PartialEq)]
pub enum CompletionContext {
    /// In header section (directives).
    Header,
    /// After @ in reference position.
    Reference {
        /// Partially typed type name, if any.
        partial_type: Option<String>,
    },
    /// After @Type: in reference position.
    ReferenceId {
        /// The type name for entity ID completion.
        type_name: String,
    },
    /// After : in list declaration (type name).
    ListType,
    /// In matrix row (cell values).
    MatrixCell {
        /// The type name of the matrix.
        type_name: String,
        /// The column index (0-based).
        column_index: usize,
    },
    /// After @ at start of indented line (inline child list context).
    InlineChildType {
        /// Parent type name (from previous row).
        parent_type: Option<String>,
        /// Partially typed child type name, if any.
        partial_type: Option<String>,
    },
    /// Key position in body.
    Key,
    /// Value position in body.
    Value,
    /// Unknown context.
    Unknown,
}

/// Get completions for a position in the document.
///
/// # Performance
///
/// Uses cached analysis data including `header_end_line` for O(1) context detection
/// and `reference_index` for fast entity lookup.
#[must_use]
pub fn get_completions(
    analysis: &AnalyzedDocument,
    content: &str,
    position: Position,
) -> Vec<CompletionItem> {
    let context = determine_context_optimized(analysis, content, position);
    let mut items = Vec::new();

    match context {
        CompletionContext::Header => {
            items.extend(header_completions());
        }
        CompletionContext::Reference { partial_type } => {
            items.extend(reference_type_completions(
                analysis,
                partial_type.as_deref(),
            ));
        }
        CompletionContext::ReferenceId { type_name } => {
            items.extend(reference_id_completions(analysis, &type_name));
        }
        CompletionContext::ListType => {
            items.extend(list_type_completions(analysis));
        }
        CompletionContext::MatrixCell {
            type_name,
            column_index,
        } => {
            items.extend(matrix_cell_completions(analysis, &type_name, column_index));
        }
        CompletionContext::InlineChildType {
            parent_type,
            partial_type,
        } => {
            items.extend(inline_child_type_completions(
                analysis,
                parent_type.as_deref(),
                partial_type.as_deref(),
            ));
        }
        CompletionContext::Key => {
            items.extend(key_completions(analysis));
        }
        CompletionContext::Value => {
            items.extend(value_completions(analysis));
        }
        CompletionContext::Unknown => {}
    }

    items
}

/// Determine completion context from position with cached optimization.
///
/// # Security
///
/// Uses safe string slicing to prevent UTF-8 boundary panics when the cursor
/// position falls in the middle of a multi-byte character.
///
/// # Performance Optimization
///
/// Uses cached `header_end_line` from analysis for O(1) header detection instead
/// of O(n) iteration through all lines.
///
/// # Position Handling
///
/// LSP positions use UTF-16 code units, so we must convert to byte offsets
/// for proper handling of multi-byte UTF-8 characters.
#[must_use]
pub fn determine_context_optimized(
    analysis: &AnalyzedDocument,
    content: &str,
    position: Position,
) -> CompletionContext {
    use crate::utf_encoding::utf16_col_to_byte_offset;

    let lines: Vec<&str> = content.lines().collect();
    let line_num = position.line as usize;

    if line_num >= lines.len() {
        return CompletionContext::Unknown;
    }

    let line = lines[line_num];

    // Convert UTF-16 position to byte offset
    let byte_offset = utf16_col_to_byte_offset(line, position.character);

    // Security: Use safe slicing to prevent UTF-8 boundary panics
    let prefix = safe_slice_to(line, byte_offset);

    // Performance: Use cached header_end_line for O(1) lookup
    let in_header = if let Some(header_end) = analysis.header_end_line {
        line_num < header_end
    } else {
        // Fallback: check if we're before --- (slower but safe)
        lines[..line_num].iter().all(|l| *l != "---")
    };

    if in_header {
        let trimmed_prefix = prefix.trim();

        if trimmed_prefix.starts_with('%') || trimmed_prefix.is_empty() {
            return CompletionContext::Header;
        }
    }

    // Check for inline child list context (indented @ at start of line)
    // Pattern: "  @ChildType#N:|data" (inline child list syntax)
    let trimmed_line = line.trim_start();
    if trimmed_line.starts_with('@') {
        let indent_level = line.len() - trimmed_line.len();

        // Must be indented (inline children are nested under parent rows)
        if indent_level > 0 {
            // Check if cursor is before # or : - this is where we suggest child types
            // Need to check the portion of the line up to cursor position
            let prefix_from_at_start = if byte_offset > indent_level {
                safe_slice_to(trimmed_line, byte_offset - indent_level)
            } else {
                ""
            };

            // If we're after @ but before # or :, suggest child types
            if let Some(after_at_to_cursor) = prefix_from_at_start.strip_prefix('@') {
                // Check if we haven't typed # or : yet
                if !after_at_to_cursor.contains('#') && !after_at_to_cursor.contains(':') {
                    // Extract parent type from previous less-indented line
                    let parent_type = find_parent_type_for_inline_child(lines.clone(), line_num);

                    let partial = if !after_at_to_cursor.is_empty() {
                        Some(after_at_to_cursor.to_string())
                    } else {
                        None
                    };

                    return CompletionContext::InlineChildType {
                        parent_type,
                        partial_type: partial,
                    };
                }
            }
        }
    }

    // Check for matrix row FIRST (before reference check)
    // This prevents @ symbols in email addresses or other data from being
    // misinterpreted as reference syntax
    if line.trim_start().starts_with('|') {
        // Find which cell we're in by parsing the CSV up to the cursor

        // Get the portion of the line after the pipe
        let pipe_pos = line.find('|').unwrap_or(0);
        let after_pipe = &line[pipe_pos + 1..];

        // Skip row prefix (|N or |[N]) to find where CSV starts
        let csv_start_offset = if after_pipe.trim_start().starts_with('[') {
            // |[N] pattern - skip the bracket notation
            if let Some(bracket_end) = after_pipe.find(']') {
                bracket_end + 1
            } else {
                0
            }
        } else if after_pipe
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
        {
            // |N pattern - skip the number
            after_pipe.chars().take_while(char::is_ascii_digit).count()
        } else {
            0
        };

        let csv_start = pipe_pos + 1 + csv_start_offset;
        let csv_portion = if byte_offset > csv_start {
            safe_slice_to(&line[csv_start..], byte_offset - csv_start)
        } else {
            ""
        };

        // Parse CSV up to cursor to count columns (respecting quotes)
        let column_index = if csv_portion.is_empty() {
            0
        } else {
            // Parse the CSV portion to count completed fields
            match hedl_core::lex::parse_csv_row(csv_portion) {
                Ok(fields) => {
                    // Check if we're in the middle of a field or past it
                    // If the portion doesn't end with a comma, we're in the last parsed field
                    if csv_portion.trim_end().ends_with(',') {
                        fields.len()
                    } else {
                        fields.len().saturating_sub(1)
                    }
                }
                Err(_) => {
                    // Fallback: quote-aware comma counting for incomplete fields
                    // This handles the case where cursor is inside an unclosed quote
                    count_columns_quote_aware(csv_portion)
                }
            }
        };

        return CompletionContext::MatrixCell {
            type_name: find_active_list_type(lines, line_num),
            column_index,
        };
    }

    // Check for reference context
    if let Some(at_pos) = prefix.rfind('@') {
        let after_at = &prefix[at_pos + 1..];

        if let Some(colon_pos) = after_at.rfind(':') {
            // After @Type:
            let type_name = after_at[..colon_pos].to_string();
            return CompletionContext::ReferenceId { type_name };
        } else {
            // After @ but before :
            let partial = if after_at.is_empty() {
                None
            } else {
                Some(after_at.to_string())
            };
            return CompletionContext::Reference {
                partial_type: partial,
            };
        }
    }

    // Check for list type context
    if prefix.contains(':') && prefix.trim_end().ends_with('@') {
        return CompletionContext::ListType;
    }

    // Check for key vs value position
    if prefix.contains(':') {
        CompletionContext::Value
    } else {
        CompletionContext::Key
    }
}

/// Find the parent type for an inline child list by looking backward
/// for the most recent less-indented matrix row.
fn find_parent_type_for_inline_child(lines: Vec<&str>, current_line: usize) -> Option<String> {
    let current_line_content = lines.get(current_line)?;
    let current_indent = current_line_content.len() - current_line_content.trim_start().len();

    // Search backwards for a matrix row at less indentation (parent row)
    for i in (0..current_line).rev() {
        let line = lines[i];
        let trimmed = line.trim_start();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let line_indent = line.len() - line.trim_start().len();

        // Must be at less indentation to be a parent
        if line_indent >= current_indent {
            continue;
        }

        // Check if it's a matrix row (starts with |)
        if trimmed.starts_with('|') {
            // This is the parent row - find its list declaration to get the type
            let parent_type = find_active_list_type(lines.clone(), i);
            if !parent_type.is_empty() {
                return Some(parent_type);
            }
        }
    }

    None
}

fn find_active_list_type(lines: Vec<&str>, current_line: usize) -> String {
    // Look backwards to find the list declaration for the current row.
    // Key insight: List declarations (e.g., "users:@User") are at a LOWER
    // indentation level than their rows (e.g., "  | alice, Alice") OR at the
    // same level for top-level lists (e.g., "products:@Product" followed by "|p01,...").

    let current_line_content = match lines.get(current_line) {
        Some(line) => *line,
        None => return String::new(),
    };

    let current_indent = current_line_content.len() - current_line_content.trim_start().len();

    // Search backwards for a list declaration at less or equal indentation
    for i in (0..current_line).rev() {
        let line = lines[i];
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let line_indent = line.len() - line.trim_start().len();

        // Declaration must be at LESS or EQUAL indentation than the row
        if line_indent > current_indent {
            // More indentation: this is nested content, skip
            continue;
        }

        // Found a line at less or equal indentation - check if it's a list declaration
        if trimmed.contains(":@") || trimmed.contains(": @") {
            // Extract the type name after @
            if let Some(at_pos) = trimmed.find('@') {
                let rest = &trimmed[at_pos + 1..];
                let end = rest
                    .find(|c: char| c == '[' || c.is_whitespace())
                    .unwrap_or(rest.len());
                return rest[..end].to_string();
            }
        }

        // If we found a line at less indentation that's not a list declaration,
        // we've gone too far up the hierarchy
        if line_indent < current_indent {
            break;
        }
    }

    String::new()
}

/// Count columns in a CSV portion using quote-aware parsing.
/// This handles incomplete quoted fields where `parse_csv_row` would fail.
/// Counts commas that are outside of quoted regions.
fn count_columns_quote_aware(s: &str) -> usize {
    let mut column_count = 0;
    let mut in_quotes = false;
    let mut prev_char = None;

    for c in s.chars() {
        match c {
            '"' => {
                // Toggle quote state (handle escaped quotes "")
                if prev_char == Some('"') && in_quotes {
                    // This is an escaped quote, don't toggle
                    prev_char = None;
                    continue;
                }
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                column_count += 1;
            }
            _ => {}
        }
        prev_char = Some(c);
    }

    column_count
}

fn header_completions() -> Vec<CompletionItem> {
    vec![
        // v2.0 compact directives (recommended)
        CompletionItem {
            label: "%V:2.0".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("v2.0 version (compact, recommended)".to_string()),
            insert_text: Some("%V:2.0".to_string()),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            documentation: Some(Documentation::String(
                "HEDL v2.0 version directive. Required as first line.\n\n\
                 v2.0 uses 1-space indentation and requires %NULL: and %QUOTE: directives."
                    .to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%NULL:~".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Null character (v2.0 required)".to_string()),
            insert_text: Some("%NULL:~".to_string()),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            documentation: Some(Documentation::String(
                "Defines the null literal character. Required in v2.0.\n\n\
                 Common choice: ~ (tilde)"
                    .to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%QUOTE:\"".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Quote character (v2.0 required)".to_string()),
            insert_text: Some("%QUOTE:\"".to_string()),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            documentation: Some(Documentation::String(
                "Defines the quote character for strings. Required in v2.0.\n\n\
                 Common choice: \" (double quote)"
                    .to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%S:".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Schema definition (v2.0 compact)".to_string()),
            insert_text: Some("%S:${1:TypeName}:[${2:id},${3:field}]".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Compact schema definition (v2.0).\n\n\
                 Example: %S:User:[id,name,email]\n\n\
                 First column is always the ID."
                    .to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%N:".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Nesting relationship (v2.0 compact)".to_string()),
            insert_text: Some("%N:${1:Parent}>${2:Child}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Compact nesting declaration (v2.0).\n\n\
                 Example: %N:Task>Comment\n\n\
                 Declares a parent-child relationship for hierarchical data."
                    .to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%C:".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Count hint (v2.0)".to_string()),
            insert_text: Some("%C:${1:Type}.total=${2:N}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Declared count for a type (v2.0). Authoritative for counting questions.\n\n\
                 Examples:\n\
                 - %C:User.total=100\n\
                 - %C:User.status:active=80,inactive=20"
                    .to_string(),
            )),
            ..Default::default()
        },
        // Legacy verbose directives
        CompletionItem {
            label: "%VERSION".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Version declaration (legacy)".to_string()),
            insert_text: Some("%VERSION: 1.2".to_string()),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            documentation: Some(Documentation::String(
                "Legacy verbose version directive.\n\n\
                 Consider using %V:2.0 for new documents."
                    .to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%STRUCT".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Schema definition (legacy)".to_string()),
            insert_text: Some("%STRUCT: ${1:TypeName}: [${2:id}, ${3:field}]".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Legacy verbose schema definition.\n\n\
                 Consider using %S: for v2.0 documents."
                    .to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%ALIAS".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Alias definition".to_string()),
            insert_text: Some("%ALIAS: ${1:short} = \"${2:long value}\"".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Defines an alias for repeated values.".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%NEST".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Nesting relationship (legacy)".to_string()),
            insert_text: Some("%NEST: ${1:Parent} > ${2:Child}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Legacy verbose nesting declaration.\n\n\
                 Consider using %N: for v2.0 documents."
                    .to_string(),
            )),
            ..Default::default()
        },
    ]
}

fn reference_type_completions(
    analysis: &AnalyzedDocument,
    partial: Option<&str>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    for type_name in analysis.get_type_names() {
        if let Some(p) = partial {
            if !type_name.to_lowercase().starts_with(&p.to_lowercase()) {
                continue;
            }
        }

        let entity_count = analysis
            .entities
            .get(&type_name)
            .map_or(0, std::collections::HashMap::len);

        items.push(CompletionItem {
            label: type_name.clone(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some(format!("{entity_count} entities")),
            insert_text: Some(format!("{type_name}:")),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            documentation: analysis
                .get_schema(&type_name)
                .map(|cols| Documentation::String(format!("Schema: [{}]", cols.join(", ")))),
            ..Default::default()
        });
    }

    items
}

fn reference_id_completions(analysis: &AnalyzedDocument, type_name: &str) -> Vec<CompletionItem> {
    analysis
        .get_entity_ids(type_name)
        .into_iter()
        .map(|id| CompletionItem {
            label: id.clone(),
            kind: Some(CompletionItemKind::REFERENCE),
            detail: Some(format!("@{type_name}:{id}")),
            insert_text: Some(id),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..Default::default()
        })
        .collect()
}

fn list_type_completions(analysis: &AnalyzedDocument) -> Vec<CompletionItem> {
    analysis
        .get_type_names()
        .into_iter()
        .map(|type_name| {
            let schema = analysis.get_schema(&type_name);
            CompletionItem {
                label: type_name.clone(),
                kind: Some(CompletionItemKind::CLASS),
                detail: schema.map(|cols| format!("[{}]", cols.join(", "))),
                insert_text: Some(format!("@{type_name}")),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                documentation: Some(Documentation::String(
                    "Use this type for the list".to_string(),
                )),
                ..Default::default()
            }
        })
        .collect()
}

fn matrix_cell_completions(
    analysis: &AnalyzedDocument,
    type_name: &str,
    column_index: usize,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Get schema to understand the column
    if let Some(schema) = analysis.get_schema(type_name) {
        if column_index < schema.len() {
            let column_name = &schema[column_index];

            // If column looks like a reference (e.g., "owner", "user_id"), suggest entity IDs
            let reference_patterns = ["_id", "owner", "user", "author", "creator", "parent"];
            for pattern in reference_patterns {
                if column_name.to_lowercase().contains(pattern) {
                    // Suggest all entity IDs
                    for (t, entities) in &analysis.entities {
                        for id in entities.keys() {
                            items.push(CompletionItem {
                                label: format!("@{t}:{id}"),
                                kind: Some(CompletionItemKind::REFERENCE),
                                detail: Some(format!("Reference to {t} entity")),
                                ..Default::default()
                            });
                        }
                    }
                    break;
                }
            }
        }
    }

    // Add ditto marker (only for pre-v2.0 documents)
    if analysis.ditto_allowed() {
        items.push(CompletionItem {
            label: "^".to_string(),
            kind: Some(CompletionItemKind::OPERATOR),
            detail: Some("Ditto - repeat previous row's value".to_string()),
            documentation: Some(Documentation::String(
                "The ditto operator (^) repeats the value from the same column in the previous row."
                    .to_string(),
            )),
            ..Default::default()
        });
    }

    // Add common scalar values
    items.push(CompletionItem {
        label: "~".to_string(),
        kind: Some(CompletionItemKind::CONSTANT),
        detail: Some("Null value".to_string()),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "true".to_string(),
        kind: Some(CompletionItemKind::CONSTANT),
        detail: Some("Boolean true".to_string()),
        ..Default::default()
    });

    items.push(CompletionItem {
        label: "false".to_string(),
        kind: Some(CompletionItemKind::CONSTANT),
        detail: Some("Boolean false".to_string()),
        ..Default::default()
    });

    items
}

fn key_completions(analysis: &AnalyzedDocument) -> Vec<CompletionItem> {
    // Suggest common key patterns
    let mut items = vec![
        CompletionItem {
            label: "id".to_string(),
            kind: Some(CompletionItemKind::FIELD),
            detail: Some("Entity identifier".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "name".to_string(),
            kind: Some(CompletionItemKind::FIELD),
            detail: Some("Display name".to_string()),
            ..Default::default()
        },
        CompletionItem {
            label: "description".to_string(),
            kind: Some(CompletionItemKind::FIELD),
            detail: Some("Description field".to_string()),
            ..Default::default()
        },
    ];

    // Add defined type names as potential list keys
    for type_name in analysis.get_type_names() {
        items.push(CompletionItem {
            label: type_name.to_lowercase(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(format!("List of {type_name} entities")),
            insert_text: Some(format!("{}:@{}", type_name.to_lowercase(), type_name)),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..Default::default()
        });
    }

    items
}

fn value_completions(analysis: &AnalyzedDocument) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Add aliases
    for (alias, (value, _)) in &analysis.aliases {
        items.push(CompletionItem {
            label: format!("${alias}"),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some(format!("Alias for \"{value}\"")),
            insert_text: Some(format!("${alias}")),
            ..Default::default()
        });
    }

    // Add type references for list declarations
    for type_name in analysis.get_type_names() {
        items.push(CompletionItem {
            label: format!("@{type_name}"),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some("Start a typed list".to_string()),
            insert_text: Some(format!("@{type_name}")),
            ..Default::default()
        });
    }

    items
}

fn inline_child_type_completions(
    analysis: &AnalyzedDocument,
    parent_type: Option<&str>,
    partial: Option<&str>,
) -> Vec<CompletionItem> {
    let mut items = Vec::new();

    // Get valid child types for the parent from NEST declarations
    let valid_child_types: Vec<String> = if let Some(parent) = parent_type {
        // Get all children for this parent
        if let Some(children) = analysis.nests.get(parent) {
            children.iter().map(|(child, _)| child.clone()).collect()
        } else {
            Vec::new()
        }
    } else {
        // No parent type found, suggest all types
        analysis.get_type_names()
    };

    for child_type in valid_child_types {
        // Apply partial filter
        if let Some(p) = partial {
            if !child_type.to_lowercase().starts_with(&p.to_lowercase()) {
                continue;
            }
        }

        let schema = analysis.get_schema(&child_type);
        let detail = schema.map(|cols| format!("Schema: [{}]", cols.join(", ")));

        items.push(CompletionItem {
            label: child_type.clone(),
            kind: Some(CompletionItemKind::CLASS),
            detail: detail.clone(),
            insert_text: Some(format!("{child_type}#${{1:N}}:|${{2:data}}")),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(format!(
                "Inline child list syntax (v2.0)\n\n\
                 Style guideline: keep N <= 10 for readability.\n\n\
                 Use expanded format for more:\n  @{child_type}#N:\n  |row1\n  |row2"
            ))),
            ..Default::default()
        });
    }

    items
}
