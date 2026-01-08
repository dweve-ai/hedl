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

//! Hover information for HEDL files.
//!
//! This module provides rich hover information when the user hovers over
//! different elements in a HEDL document.
//!
//! # Supported Elements
//!
//! - **Directives**: Documentation for %VERSION, %STRUCT, %ALIAS, %NEST
//! - **References**: Entity validation and type information for @Type:id
//! - **Aliases**: Expansion of $alias with definition location
//! - **Types**: Schema definition with entity count and nesting info
//! - **Special Tokens**: Explanation of ^ (ditto) and ~ (null)
//!
//! # Examples
//!
//! Hovering over `@User:alice` shows:
//! - Whether the entity exists (✓ or ⚠)
//! - The entity ID
//! - The User schema definition
//! - Line number where it's defined
//!
//! Hovering over `^` shows documentation about the ditto operator and
//! its role in reducing token usage.

use crate::analysis::AnalyzedDocument;
use tower_lsp::lsp_types::*;

/// Get hover information for a position.
///
/// # Arguments
///
/// * `analysis` - The analyzed document with entity and type information
/// * `content` - The full document content
/// * `position` - The cursor position where hover was triggered
///
/// # Returns
///
/// An optional `Hover` with markdown-formatted content and the range of
/// the hovered element. Returns `None` if no hover information is available
/// for the position.
pub fn get_hover(analysis: &AnalyzedDocument, content: &str, position: Position) -> Option<Hover> {
    let lines: Vec<&str> = content.lines().collect();
    let line_num = position.line as usize;

    if line_num >= lines.len() {
        return None;
    }

    let line = lines[line_num];
    let char_pos = position.character as usize;

    // Find word at position
    let (word, word_start, word_end) = find_word_at(line, char_pos)?;

    // Determine what kind of element this is
    let hover_content = if line.trim().starts_with('%') {
        // Header directive
        get_directive_hover(line)
    } else if word.starts_with('@') {
        // Reference
        get_reference_hover(analysis, &word)
    } else if let Some(alias_name) = word.strip_prefix('$') {
        // Alias usage
        get_alias_hover(analysis, alias_name)
    } else if word == "^" {
        // Ditto operator
        Some(create_hover_content(
            "**Ditto Operator** (`^`)",
            "Repeats the value from the same column in the previous row.\n\n\
             This is a key optimization feature in HEDL that reduces token usage \
             when consecutive rows share the same values in a column.",
        ))
    } else if word == "~" {
        // Null
        Some(create_hover_content(
            "**Null Value** (`~`)",
            "Represents an absent or null value in HEDL.",
        ))
    } else if is_type_name(&word, analysis) {
        // Type name
        get_type_hover(analysis, &word)
    } else if line.contains(": @") && line.contains(&word) {
        // Could be a list key or type
        get_list_hover(analysis, line, &word)
    } else {
        None
    }?;

    Some(Hover {
        contents: HoverContents::Markup(hover_content),
        range: Some(Range {
            start: Position {
                line: position.line,
                character: word_start as u32,
            },
            end: Position {
                line: position.line,
                character: word_end as u32,
            },
        }),
    })
}

fn find_word_at(line: &str, pos: usize) -> Option<(String, usize, usize)> {
    let chars: Vec<char> = line.chars().collect();

    // Check against char count, not byte count
    if pos >= chars.len() {
        return None;
    }

    // Handle special single-char tokens
    if let Some(&ch) = chars.get(pos) {
        if ch == '^' || ch == '~' {
            return Some((ch.to_string(), pos, pos + 1));
        }
    }

    // Find word boundaries
    let is_word_char =
        |c: char| c.is_alphanumeric() || c == '_' || c == '@' || c == '$' || c == ':';

    let mut start = pos;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    let mut end = pos;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    if start == end {
        return None;
    }

    let word: String = chars[start..end].iter().collect();
    Some((word, start, end))
}

fn get_directive_hover(line: &str) -> Option<MarkupContent> {
    let trimmed = line.trim();

    if trimmed.starts_with("%VERSION") {
        Some(create_hover_content(
            "**%VERSION Directive**",
            "Declares the HEDL version for this document.\n\n\
             ```hedl\n%VERSION 1.0\n```\n\n\
             Must be the first directive in the header.",
        ))
    } else if trimmed.starts_with("%STRUCT") {
        Some(create_hover_content(
            "**%STRUCT Directive**",
            "Defines a schema for a typed matrix list.\n\n\
             ```hedl\n%STRUCT User[id, name, email]\n```\n\n\
             - First column is always the unique entity ID\n\
             - Columns define the structure for all rows of this type",
        ))
    } else if trimmed.starts_with("%ALIAS") {
        Some(create_hover_content(
            "**%ALIAS Directive**",
            "Defines an alias for frequently used values.\n\n\
             ```hedl\n%ALIAS active = \"Active Status\"\n```\n\n\
             Use with `$alias_name` in the body to reduce repetition.",
        ))
    } else if trimmed.starts_with("%NEST") {
        Some(create_hover_content(
            "**%NEST Directive**",
            "Declares a parent-child nesting relationship.\n\n\
             ```hedl\n%NEST Order > OrderItem\n```\n\n\
             Allows child rows to be indented under parent rows.",
        ))
    } else {
        None
    }
}

fn get_reference_hover(analysis: &AnalyzedDocument, reference: &str) -> Option<MarkupContent> {
    let ref_content = reference.strip_prefix('@')?;

    let (type_name, id) = if let Some(colon_pos) = ref_content.find(':') {
        let t = &ref_content[..colon_pos];
        let i = &ref_content[colon_pos + 1..];
        (Some(t), i)
    } else {
        (None, ref_content)
    };

    // Check if entity exists
    let exists = analysis.entity_exists(type_name, id);

    let status = if exists {
        "✓ Entity found"
    } else {
        "⚠ Entity not found"
    };

    let title = match type_name {
        Some(t) => format!("**Reference** `@{}:{}`", t, id),
        None => format!("**Reference** `@{}`", id),
    };

    let mut description = format!("{}\n\nPoints to entity with ID `{}`.", status, id);

    if let Some(t) = type_name {
        if let Some(schema) = analysis.get_schema(t) {
            description.push_str(&format!("\n\n**Schema**: `[{}]`", schema.join(", ")));
        }
    }

    Some(create_hover_content(&title, &description))
}

fn get_alias_hover(analysis: &AnalyzedDocument, alias_name: &str) -> Option<MarkupContent> {
    let (value, line) = analysis.aliases.get(alias_name)?;

    Some(create_hover_content(
        &format!("**Alias** `${}`", alias_name),
        &format!("Expands to: `\"{}\"`\n\nDefined on line {}.", value, line),
    ))
}

fn get_type_hover(analysis: &AnalyzedDocument, type_name: &str) -> Option<MarkupContent> {
    let (schema, line) = analysis.schemas.get(type_name)?;
    let entity_count = analysis
        .entities
        .get(type_name)
        .map(|m| m.len())
        .unwrap_or(0);

    let mut description = format!(
        "**Schema**: `[{}]`\n\n\
         **Entities**: {} defined\n\n\
         Defined on line {}.",
        schema.join(", "),
        entity_count,
        line
    );

    // Add nest info
    if let Some((child, _)) = analysis.nests.get(type_name) {
        description.push_str(&format!("\n\n**Nests**: `{}` children", child));
    }

    Some(create_hover_content(
        &format!("**Type** `{}`", type_name),
        &description,
    ))
}

fn get_list_hover(analysis: &AnalyzedDocument, line: &str, word: &str) -> Option<MarkupContent> {
    // Check if this is a list declaration like "users: @User"
    if line.contains(&format!(": @{}", word)) {
        return get_type_hover(analysis, word);
    }

    None
}

fn is_type_name(word: &str, analysis: &AnalyzedDocument) -> bool {
    analysis.schemas.contains_key(word)
}

fn create_hover_content(title: &str, description: &str) -> MarkupContent {
    MarkupContent {
        kind: MarkupKind::Markdown,
        value: format!("{}\n\n---\n\n{}", title, description),
    }
}
