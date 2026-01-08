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
//! - **Header**: Header directives (%VERSION, %STRUCT, %ALIAS, %NEST)
//! - **Reference**: Type names after @ symbol
//! - **ReferenceId**: Entity IDs after @Type:
//! - **ListType**: Type names in list declarations
//! - **MatrixCell**: Values in matrix cells (ditto, null, booleans, references)
//! - **Key**: Property keys in object notation
//! - **Value**: Property values (aliases, type references)
//!
//! # Examples
//!
//! ```text
//! %STRUCT U|         → Suggests STRUCT completion
//! users: @U|         → Suggests User type
//! @User:|            → Suggests entity IDs for User type
//! | alice | @U|      → Suggests references in matrix cell
//! ```

use crate::analysis::AnalyzedDocument;
use crate::utils::safe_slice_to;
use tower_lsp::lsp_types::*;

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
    Reference { partial_type: Option<String> },
    /// After @Type: in reference position.
    ReferenceId { type_name: String },
    /// After : in list declaration (type name).
    ListType,
    /// In matrix row (cell values).
    MatrixCell {
        type_name: String,
        column_index: usize,
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
/// Uses cached analysis data including header_end_line for O(1) context detection
/// and reference_index for fast entity lookup.
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
fn determine_context_optimized(
    analysis: &AnalyzedDocument,
    content: &str,
    position: Position,
) -> CompletionContext {
    let lines: Vec<&str> = content.lines().collect();
    let line_num = position.line as usize;

    if line_num >= lines.len() {
        return CompletionContext::Unknown;
    }

    let line = lines[line_num];
    let char_pos = position.character as usize;

    // Security: Use safe slicing to prevent UTF-8 boundary panics
    let prefix = safe_slice_to(line, char_pos);

    // Performance: Use cached header_end_line for O(1) lookup
    let in_header = if let Some(header_end) = analysis.header_end_line {
        line_num < header_end
    } else {
        // Fallback: check if we're before --- (slower but safe)
        lines[..line_num].iter().all(|l| *l != "---")
    };

    if in_header && (prefix.trim().starts_with('%') || prefix.trim().is_empty()) {
        return CompletionContext::Header;
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

    // Check for matrix row
    if line.trim_start().starts_with('|') {
        // Find which cell we're in
        let pipe_count = prefix.matches('|').count();
        return CompletionContext::MatrixCell {
            type_name: find_active_list_type(lines, line_num),
            column_index: pipe_count.saturating_sub(1),
        };
    }

    // Check for key vs value position
    if prefix.contains(':') {
        CompletionContext::Value
    } else {
        CompletionContext::Key
    }
}

/// Legacy determine_context for backwards compatibility (deprecated).
///
/// # Security
///
/// Uses safe string slicing to prevent UTF-8 boundary panics when the cursor
/// position falls in the middle of a multi-byte character.
#[allow(dead_code)]
fn determine_context(content: &str, position: Position) -> CompletionContext {
    let lines: Vec<&str> = content.lines().collect();
    let line_num = position.line as usize;

    if line_num >= lines.len() {
        return CompletionContext::Unknown;
    }

    let line = lines[line_num];
    let char_pos = position.character as usize;

    // Security: Use safe slicing to prevent UTF-8 boundary panics
    let prefix = safe_slice_to(line, char_pos);

    // Check if we're in header (before ---)
    let in_header = lines[..line_num].iter().all(|l| *l != "---");

    if in_header && (prefix.trim().starts_with('%') || prefix.trim().is_empty()) {
        return CompletionContext::Header;
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

    // Check for matrix row
    if line.trim_start().starts_with('|') {
        // Find which cell we're in
        let pipe_count = prefix.matches('|').count();
        return CompletionContext::MatrixCell {
            type_name: find_active_list_type(lines, line_num),
            column_index: pipe_count.saturating_sub(1),
        };
    }

    // Check for key vs value position
    if prefix.contains(':') {
        CompletionContext::Value
    } else {
        CompletionContext::Key
    }
}

fn find_active_list_type(lines: Vec<&str>, current_line: usize) -> String {
    // Look backwards to find the list declaration
    for i in (0..current_line).rev() {
        let line = lines[i].trim();
        if line.contains(": @") {
            // Extract type name
            if let Some(at_pos) = line.find('@') {
                let rest = &line[at_pos + 1..];
                let end = rest
                    .find(|c: char| c == '[' || c.is_whitespace())
                    .unwrap_or(rest.len());
                return rest[..end].to_string();
            }
        }
    }
    String::new()
}

fn header_completions() -> Vec<CompletionItem> {
    vec![
        CompletionItem {
            label: "%VERSION".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("HEDL version declaration".to_string()),
            insert_text: Some("%VERSION 1.0".to_string()),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            documentation: Some(Documentation::String(
                "Declares the HEDL version. Required as first directive.".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%STRUCT".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Schema definition".to_string()),
            insert_text: Some("%STRUCT ${1:TypeName}[${2:id}, ${3:field}]".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Defines a schema for a typed list. First column is always the ID.".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%ALIAS".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Alias definition".to_string()),
            insert_text: Some("%ALIAS ${1:short} = \"${2:long value}\"".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Defines an alias for repeated values.".to_string(),
            )),
            ..Default::default()
        },
        CompletionItem {
            label: "%NEST".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some("Nesting relationship".to_string()),
            insert_text: Some("%NEST ${1:Parent} > ${2:Child}".to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            documentation: Some(Documentation::String(
                "Declares a parent-child nesting relationship for hierarchical data.".to_string(),
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
            .map(|m| m.len())
            .unwrap_or(0);

        items.push(CompletionItem {
            label: type_name.clone(),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some(format!("{} entities", entity_count)),
            insert_text: Some(format!("{}:", type_name)),
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
            detail: Some(format!("@{}:{}", type_name, id)),
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
                insert_text: Some(format!("@{}", type_name)),
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
                                label: format!("@{}:{}", t, id),
                                kind: Some(CompletionItemKind::REFERENCE),
                                detail: Some(format!("Reference to {} entity", t)),
                                ..Default::default()
                            });
                        }
                    }
                    break;
                }
            }
        }
    }

    // Add ditto marker
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
            detail: Some(format!("List of {} entities", type_name)),
            insert_text: Some(format!("{}: @{}", type_name.to_lowercase(), type_name)),
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
            label: format!("${}", alias),
            kind: Some(CompletionItemKind::VARIABLE),
            detail: Some(format!("Alias for \"{}\"", value)),
            insert_text: Some(format!("${}", alias)),
            ..Default::default()
        });
    }

    // Add type references for list declarations
    for type_name in analysis.get_type_names() {
        items.push(CompletionItem {
            label: format!("@{}", type_name),
            kind: Some(CompletionItemKind::CLASS),
            detail: Some("Start a typed list".to_string()),
            insert_text: Some(format!("@{}", type_name)),
            ..Default::default()
        });
    }

    items
}
