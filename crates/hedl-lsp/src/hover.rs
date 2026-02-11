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
//! - **Directives**: Documentation for %V, %S, %A, %N (and legacy %VERSION, %STRUCT, %ALIAS, %NEST)
//! - **References**: Entity validation and type information for @Type:id
//! - **Aliases**: Expansion of $alias with definition location
//! - **Types**: Schema definition with entity count and nesting info
//! - **Special Tokens**: Explanation of ~ (null) and ^ (ditto, removed in v2.0)
//!
//! # Examples
//!
//! Hovering over `@User:alice` shows:
//! - Whether the entity exists (✓ or ⚠)
//! - The entity ID
//! - The User schema definition
//! - Line number where it's defined
//!
//! Hovering over `^` in pre-v2.0 documents shows documentation about the ditto operator and
//! its role in reducing token usage.

use crate::analysis::AnalyzedDocument;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position, Range};

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
#[must_use]
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
        // Check for inline child list syntax first
        if let Some(hover) = get_inline_child_hover(analysis, &word, line) {
            Some(hover)
        } else {
            // Regular reference
            get_reference_hover(analysis, &word)
        }
    } else if let Some(alias_name) = word.strip_prefix('$') {
        // Alias usage
        get_alias_hover(analysis, alias_name)
    } else if word == "^" {
        // Ditto operator
        Some(create_hover_content(
            "**Ditto Operator** (`^`)",
            "⚠️ **DEPRECATED**: Ditto is NOT allowed in v2.0.\n\n\
             In pre-v2.0 documents, this repeated the value from the same column in the previous row.\n\
             It was a key optimization feature that reduced token usage.\n\n\
             **v2.0 Migration**: Replace all `^` with explicit values.",
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
    } else if (line.contains(":@") || line.contains(": @")) && line.contains(&word) {
        // Could be a list key or type
        get_list_hover(analysis, line, &word)
    } else {
        // Check if this is an entity ID in a definition
        get_entity_id_hover(analysis, &word, line)
    }?;

    // Adjust range if word starts with | (from matrix row entity ID detection)
    // The hover range should only cover the entity ID, not the pipe
    let (adjusted_start, adjusted_end) = if word.starts_with('|') && !word.starts_with("@") {
        (word_start + 1, word_end)
    } else {
        (word_start, word_end)
    };

    Some(Hover {
        contents: HoverContents::Markup(hover_content),
        range: Some(Range {
            start: Position {
                line: position.line,
                character: adjusted_start as u32,
            },
            end: Position {
                line: position.line,
                character: adjusted_end as u32,
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
    // Include # and | for inline child syntax (@Type#N:|)
    let is_word_char = |c: char| {
        c.is_alphanumeric()
            || c == '_'
            || c == '@'
            || c == '$'
            || c == ':'
            || c == '-'
            || c == '#'
            || c == '|'
    };

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

    // v2.0 compact directives (check first due to prefix overlap)
    if trimmed.starts_with("%V:") {
        Some(create_hover_content(
            "**%V: Directive** (v2.0)",
            "Declares the HEDL version for this document.\n\n\
             ```hedl\n%V:2.0\n```\n\n\
             Must be the first directive in the header.\n\n\
             v2.0 features:\n\
             - 1-space indentation\n\
             - Required %NULL: and %QUOTE: directives\n\
             - Ditto (^) is forbidden\n\
             - Inline children:@Type#N:|row1|row2|...",
        ))
    } else if trimmed.starts_with("%NULL:") {
        Some(create_hover_content(
            "**%NULL: Directive** (v2.0 required)",
            "Defines the null literal character.\n\n\
             ```hedl\n%NULL:~\n```\n\n\
             Common choice: `~` (tilde)\n\n\
             This directive is required in v2.0 documents.",
        ))
    } else if trimmed.starts_with("%QUOTE:") {
        Some(create_hover_content(
            "**%QUOTE: Directive** (v2.0 required)",
            "Defines the quote character for strings.\n\n\
             ```hedl\n%QUOTE:\"\n```\n\n\
             Common choice: `\"` (double quote)\n\n\
             This directive is required in v2.0 documents.",
        ))
    } else if trimmed.starts_with("%S:") {
        Some(create_hover_content(
            "**%S: Directive** (v2.0 compact schema)",
            "Defines a schema for a typed matrix list.\n\n\
             ```hedl\n%S:User:[id,name,email]\n```\n\n\
             - First column is always the unique entity ID\n\
             - Columns define the structure for all rows of this type\n\n\
             Equivalent to legacy `%S:User:[id, name, email]`",
        ))
    } else if trimmed.starts_with("%N:") {
        Some(create_hover_content(
            "**%N: Directive** (v2.0 compact nesting)",
            "Declares a parent-child nesting relationship.\n\n\
             ```hedl\n%N:Order>OrderItem\n```\n\n\
             Allows child rows to be indented under parent rows.\n\n\
             Equivalent to legacy `%N:Order>OrderItem`",
        ))
    } else if trimmed.starts_with("%C:") {
        Some(create_hover_content(
            "**%C: Directive** (v2.0 count hint)",
            "Declares authoritative count information for a type.\n\n\
             ```hedl\n%C:User.total=100\n%C:User.status:active=80,inactive=20\n```\n\n\
             Count hints are authoritative for counting questions.\n\
             Use `.total=N` for total count, `.field:value=N` for field breakdowns.",
        ))
    // Legacy verbose directives
    } else if trimmed.starts_with("%VERSION") {
        Some(create_hover_content(
            "**%VERSION Directive** (legacy)",
            "Declares the HEDL version for this document.\n\n\
             ```hedl\n%V:2.0\n%NULL:~\n%QUOTE:\"\n```\n\n\
             Must be the first directive in the header.\n\n\
             For v2.0 documents, use `%V:2.0` instead.",
        ))
    } else if trimmed.starts_with("%STRUCT") {
        Some(create_hover_content(
            "**%STRUCT Directive** (legacy)",
            "Defines a schema for a typed matrix list.\n\n\
             ```hedl\n%S:User:[id, name, email]\n```\n\n\
             - First column is always the unique entity ID\n\
             - Columns define the structure for all rows of this type\n\n\
             For v2.0 documents, use `%S:User:[id,name,email]` instead.",
        ))
    } else if trimmed.starts_with("%ALIAS") || trimmed.starts_with("%A:") {
        Some(create_hover_content(
            "**%ALIAS Directive**",
            "Defines an alias for frequently used values.\n\n\
             ```hedl\n%A:%active:\"Active Status\"\n```\n\n\
             Use with `$alias_name` in the body to reduce repetition.",
        ))
    } else if trimmed.starts_with("%NEST") {
        Some(create_hover_content(
            "**%NEST Directive** (legacy)",
            "Declares a parent-child nesting relationship.\n\n\
             ```hedl\n%N:Order>OrderItem\n```\n\n\
             Allows child rows to be indented under parent rows.\n\n\
             For v2.0 documents, use `%N:Order>OrderItem` instead.",
        ))
    } else {
        None
    }
}

fn get_reference_hover(analysis: &AnalyzedDocument, reference: &str) -> Option<MarkupContent> {
    let ref_content = reference.strip_prefix('@')?;

    let (type_name, id) = if let Some(colon_pos) = ref_content.find(':') {
        let type_part = &ref_content[..colon_pos];
        let id_part = &ref_content[colon_pos + 1..];
        (Some(type_part), id_part)
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
        Some(t) => format!("**Reference** `@{t}:{id}`"),
        None => format!("**Reference** `@{id}`"),
    };

    let mut description = format!("{status}\n\nPoints to entity with ID `{id}`.");

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
        &format!("**Alias** `${alias_name}`"),
        &format!("Expands to: `\"{value}\"`\n\nDefined on line {line}."),
    ))
}

fn get_type_hover(analysis: &AnalyzedDocument, type_name: &str) -> Option<MarkupContent> {
    let (schema, line) = analysis.schemas.get(type_name)?;
    let entity_count = analysis
        .entities
        .get(type_name)
        .map_or(0, std::collections::HashMap::len);

    let mut description = format!(
        "**Schema**: `[{}]`\n\n\
         **Entities**: {} defined\n\n\
         Defined on line {}.",
        schema.join(", "),
        entity_count,
        line
    );

    // Add nest info
    if let Some(children) = analysis.nests.get(type_name) {
        let child_names: Vec<&str> = children.iter().map(|(name, _)| name.as_str()).collect();
        description.push_str(&format!(
            "\n\n**Nests**: `{}` children",
            child_names.join(", ")
        ));
    }

    Some(create_hover_content(
        &format!("**Type** `{type_name}`"),
        &description,
    ))
}

fn get_list_hover(analysis: &AnalyzedDocument, line: &str, word: &str) -> Option<MarkupContent> {
    // Check if this is a list declaration like "users:@User" or "users: @User"
    if line.contains(&format!(":@{word}")) || line.contains(&format!(": @{word}")) {
        return get_type_hover(analysis, word);
    }

    None
}

fn is_type_name(word: &str, analysis: &AnalyzedDocument) -> bool {
    analysis.schemas.contains_key(word)
}

fn get_entity_id_hover(
    analysis: &AnalyzedDocument,
    word: &str,
    line: &str,
) -> Option<MarkupContent> {
    // Only trigger entity ID hover when word is in the first column of a matrix row.
    // This prevents false positives when the same word appears elsewhere (e.g., in values).

    // Must be a matrix row (starts with |)
    let trimmed = line.trim_start();
    if !trimmed.starts_with('|') {
        return None;
    }

    // Extract the first column (entity ID) from the row
    let after_pipe = trimmed.strip_prefix('|')?;

    // Skip row prefix like |5 or |[10]
    let csv_start = if after_pipe.starts_with('[') {
        // |[N] format
        after_pipe.find(']').map_or(0, |i| i + 1)
    } else if after_pipe
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
    {
        // |N format
        after_pipe.chars().take_while(char::is_ascii_digit).count()
    } else {
        0
    };

    let csv_portion = &after_pipe[csv_start..];

    // Parse to get first field (entity ID)
    let first_field = match hedl_core::lex::parse_csv_row(csv_portion.trim()) {
        Ok(fields) if !fields.is_empty() => fields[0].value.clone(),
        _ => {
            // Fallback: take content before first comma
            csv_portion
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        }
    };

    // Strip leading pipe from word if present (from find_word_at including | in word chars)
    let word_to_check = word.strip_prefix('|').unwrap_or(word);

    // Only show hover if word matches the first field (entity ID column)
    if first_field.trim() != word_to_check && first_field.trim().trim_matches('"') != word_to_check
    {
        return None;
    }

    // Check if this word appears as an entity ID in any type
    for (type_name, entities) in &analysis.entities {
        if entities.contains_key(word_to_check) {
            // Found the entity, provide hover info
            let description = if let Some(schema) = analysis.get_schema(type_name) {
                format!(
                    "Entity of type `{}`\n\n**Schema**: `[{}]`",
                    type_name,
                    schema.join(", ")
                )
            } else {
                format!("Entity of type `{type_name}`")
            };

            return Some(create_hover_content(
                &format!("**Entity ID** `{word_to_check}`"),
                &format!("{description}\n\nThis is the entity definition."),
            ));
        }
    }

    None
}

fn create_hover_content(title: &str, description: &str) -> MarkupContent {
    MarkupContent {
        kind: MarkupKind::Markdown,
        value: format!("{title}\n\n---\n\n{description}"),
    }
}

/// Get hover information for inline child list syntax.
///
/// Detects patterns like `@ChildType#N:|data` and provides hover info
/// including validation warnings if count exceeds recommended maximum.
fn get_inline_child_hover(
    analysis: &AnalyzedDocument,
    word: &str,
    line: &str,
) -> Option<MarkupContent> {
    // Check if this is an inline child list declaration
    // Pattern:@TypeName#N:|data
    let word_content = word.strip_prefix('@')?;

    // Must contain # for count hint
    let hash_pos = word_content.find('#')?;
    let type_name = &word_content[..hash_pos];

    // Parse the count hint
    let after_hash = &word_content[hash_pos + 1..];
    let colon_pos = after_hash.find(':')?;
    let count_str = &after_hash[..colon_pos];
    let count = count_str.parse::<usize>().ok()?;

    // Check if the full line matches the inline child pattern
    let trimmed = line.trim_start();
    if !trimmed.starts_with(&format!("@{type_name}#{count}:|")) {
        return None;
    }

    // Get schema for the child type
    let schema = analysis.get_schema(type_name);
    let schema_str = schema.map_or_else(
        || "Unknown type".to_string(),
        |cols| format!("**Schema**: `[{}]`", cols.join(", ")),
    );

    // Count actual children in the line
    let after_decl = trimmed.strip_prefix(&format!("@{type_name}#{count}:|"))?;
    let actual_count = after_decl
        .split('|')
        .filter(|s| !s.trim().is_empty())
        .count();

    // Build description with validation warnings
    let mut description = format!(
        "Inline child list of `{type_name}` with declared count: {count}\n\n\
         {schema_str}\n\n\
         Actual children found: {actual_count}"
    );

    // Validation warnings
    if actual_count != count {
        description.push_str(&format!(
            "\n\n⚠️ **Warning**: Count mismatch! Declared {count} but found {actual_count} children."
        ));
    }

    if count > 10 {
        description.push_str(&format!(
            "\n\n⚠️ **Style**: Inline children > 10 ({count} declared).\n\
             Style guideline recommends N <= 10 for readability. Consider expanded format:\n\n\
             ```hedl\n @{type_name}#N:\n |child1\n |child2\n ...\n```"
        ));
    }

    // Check for space after |
    if after_decl.starts_with(' ') {
        description.push_str(
            "\n\n⚠️ **Warning**: Space after `|` detected. \
             Inline children should have no space after the pipe for optimal token efficiency.",
        );
    }

    Some(create_hover_content(
        &format!("**Inline Child List** `@{type_name}#{count}:`"),
        &description,
    ))
}
