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

//! Rename refactoring for HEDL symbols.
//!
//! This module implements safe, validated rename refactoring for HEDL symbols:
//! entity IDs, type names, aliases, and field names.
//!
//! # Supported Symbol Types
//!
//! - **Entity IDs**: Individual entity identifiers (e.g., `alice`, `bob`)
//! - **Type Names**: Schema/struct names (e.g., `User`, `Post`)
//! - **Alias Names**: Variable aliases (e.g., `active`, `draft`)
//! - **Field Names**: Column names in schemas (e.g., `email`, `status`)
//!
//! # Features
//!
//! - **Precise Location Tracking**: Character-level position accuracy
//! - **Conflict Detection**: Prevents duplicate names in scope
//! - **Cross-File Support**: Rename across all open documents
//! - **Validation**: Syntax and semantic correctness checks
//! - **LSP Integration**: Full prepare rename and rename support
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use hedl_lsp::rename::{identify_symbol_at_position, find_all_occurrences};
//!
//! let content = load_document();
//! let analysis = AnalyzedDocument::analyze(&content);
//! let position = Position { line: 10, character: 5 };
//!
//! if let Some(symbol) = identify_symbol_at_position(&analysis, &content, position) {
//!     let occurrences = find_all_occurrences(&symbol, &analysis, &content, &uri);
//!     println!("Found {} occurrences", occurrences.len());
//! }
//! ```
//!
//! # Architecture
//!
//! The rename implementation leverages the existing reference index for O(1)
//! lookups and provides additional symbol-specific logic for each kind of
//! renameable symbol.

use crate::analysis::AnalyzedDocument;
use crate::document_manager::DocumentManager;
use crate::reference_index::RefLocation;
use crate::utils::{safe_slice_from, safe_slice_to};
use std::collections::HashMap;
use tower_lsp::lsp_types::{Position, Range, TextEdit, Url, WorkspaceEdit};

/// Represents a symbol that can be renamed in HEDL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    /// Entity ID (e.g., alice, bob)
    EntityId {
        /// The type name the entity belongs to.
        type_name: String,
        /// The entity identifier.
        id: String,
    },
    /// Type name / schema name (e.g., User, Post)
    TypeName(
        /// The type name.
        String,
    ),
    /// Alias name (e.g., active, draft)
    AliasName(
        /// The alias name.
        String,
    ),
    /// Field/column name in a schema
    FieldName {
        /// The type name containing this field.
        type_name: String,
        /// The field name.
        field_name: String,
    },
}

/// Location of a symbol occurrence in a document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolLocation {
    /// The URI of the document
    pub uri: Url,
    /// Precise location with line and character range
    pub location: RefLocation,
    /// Whether this is a definition or reference
    pub is_definition: bool,
}

/// Result of rename validation.
#[derive(Debug, Clone)]
pub struct RenameValidation {
    /// Whether the rename is valid
    pub valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
    /// Warning messages (non-blocking)
    pub warnings: Vec<String>,
}

/// Rename operation representing all changes.
#[derive(Debug, Clone)]
pub struct RenameOperation {
    /// The symbol being renamed
    pub symbol: SymbolKind,
    /// Old name
    pub old_name: String,
    /// New name
    pub new_name: String,
    /// All locations to be changed
    pub locations: Vec<SymbolLocation>,
    /// Validation result
    pub validation: RenameValidation,
}

/// Identify what symbol is at the given position.
#[must_use]
pub fn identify_symbol_at_position(
    analysis: &AnalyzedDocument,
    content: &str,
    position: Position,
) -> Option<SymbolKind> {
    // 1. Check for entity reference using reference_index_v2
    if let Some((ref_str, _loc)) = analysis.reference_index_v2.find_reference_at(position) {
        return identify_reference_symbol(ref_str, analysis);
    }

    // 2. Check for entity definition (in matrix list first column)
    if let Some((type_name, id)) =
        identify_entity_definition_at_position(content, position, analysis)
    {
        return Some(SymbolKind::EntityId { type_name, id });
    }

    // 3. Check for alias reference ($alias pattern)
    if let Some(alias) = identify_alias_at_position(content, position) {
        if analysis.aliases.contains_key(&alias) {
            return Some(SymbolKind::AliasName(alias));
        }
    }

    // 4. Check for type name in matrix declaration or directives
    if let Some(type_name) = identify_type_at_position(content, position, analysis) {
        return Some(SymbolKind::TypeName(type_name));
    }

    // 5. Check for field name in schema directive
    if let Some((type_name, field)) = identify_field_at_position(content, position, analysis) {
        return Some(SymbolKind::FieldName {
            type_name,
            field_name: field,
        });
    }

    None
}

/// Parse a reference string to determine symbol kind.
fn identify_reference_symbol(ref_str: &str, analysis: &AnalyzedDocument) -> Option<SymbolKind> {
    let ref_content = ref_str.strip_prefix('@').unwrap_or(ref_str);

    if let Some(colon_pos) = ref_content.find(':') {
        // Qualified reference: @Type:id
        let type_name = ref_content[..colon_pos].to_string();
        let id = ref_content[colon_pos + 1..].to_string();
        Some(SymbolKind::EntityId { type_name, id })
    } else {
        // Unqualified reference: @id or @Type
        let id = ref_content.to_string();

        // First, check if it's a type name (for matrix declarations like "users: @User")
        if analysis.schemas.contains_key(&id) {
            return Some(SymbolKind::TypeName(id));
        }

        // Otherwise, try to find it as an entity ID
        for (type_name, entities) in &analysis.entities {
            if entities.contains_key(&id) {
                return Some(SymbolKind::EntityId {
                    type_name: type_name.clone(),
                    id,
                });
            }
        }
        None
    }
}

/// Identify entity definition at position (in matrix list first column).
fn identify_entity_definition_at_position(
    content: &str,
    position: Position,
    analysis: &AnalyzedDocument,
) -> Option<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];

    // Check if this line is a matrix row (starts with | after trimming whitespace)
    let trimmed = line.trim_start();
    if !trimmed.starts_with('|') {
        return None;
    }

    // Parse the CSV row using the core parser
    let after_pipe = trimmed.strip_prefix('|')?.trim_start();

    // Use hedl_core's CSV parser to handle quoted IDs and single-column rows
    let fields = hedl_core::lex::parse_csv_row(after_pipe).ok()?;

    if fields.is_empty() {
        return None;
    }

    // Get the first field (the ID)
    let first_field = &fields[0];
    let first_col = first_field.value.as_str();

    // Check if cursor is on this first column
    let char_pos = position.character as usize;

    // Find the position of the first column in the original line
    let pipe_pos = line.find('|')?;
    let after_pipe_start = pipe_pos + 1 + (after_pipe.len() - after_pipe.trim_start().len());

    // Manually find the start and end of the first field value in the CSV
    // We need to handle both quoted and unquoted values
    let (value_start, value_end) = if first_field.is_quoted {
        // Find the opening quote
        let quote_pos = after_pipe.find('"')?;
        let start = after_pipe_start + quote_pos;
        let end = start + first_col.len() + 2; // +2 for quotes
        (start, end)
    } else {
        // Unquoted field starts at the beginning (after whitespace)
        let start = after_pipe_start;
        let end = start + first_col.len();
        (start, end)
    };

    if char_pos < value_start || char_pos > value_end {
        return None;
    }

    // Find which type this entity belongs to
    // The line number is 1-indexed in analysis.entities
    let line_num = position.line as usize + 1;

    for (type_name, entities) in &analysis.entities {
        if let Some(&entity_line) = entities.get(first_col) {
            if entity_line == line_num {
                return Some((type_name.clone(), first_col.to_string()));
            }
        }
    }

    None
}

/// Identify alias at position (%alias pattern).
fn identify_alias_at_position(content: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;

    // Check if we're on or near a % character
    let search_start = char_pos.saturating_sub(20);
    let search_end = (char_pos + 20).min(line.len());
    let search_region = safe_slice_to(line, search_end);
    let search_region = safe_slice_from(search_region, search_start);

    // Find % characters and extract alias names
    for (i, ch) in search_region.char_indices() {
        if ch == '%' {
            let abs_pos = search_start + i;
            if abs_pos > char_pos + 20 {
                break;
            }

            let after_percent = safe_slice_from(line, abs_pos + 1);
            let alias_name: String = after_percent
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .collect();

            if !alias_name.is_empty() {
                let alias_start = abs_pos;
                let alias_end = alias_start + 1 + alias_name.len();

                if char_pos >= alias_start && char_pos <= alias_end {
                    return Some(alias_name);
                }
            }
        }
    }

    None
}

/// Identify type name at position.
fn identify_type_at_position(
    content: &str,
    position: Position,
    analysis: &AnalyzedDocument,
) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;

    // Check if we're in a %STRUCT: directive
    if line.trim().starts_with("%STRUCT") {
        return extract_type_from_struct_directive(line, char_pos);
    }

    // Check if we're in a %NEST: directive
    if line.trim().starts_with("%NEST") {
        return extract_type_from_nest_directive(line, char_pos);
    }

    // Check if we're in a matrix list declaration (key: @Type)
    if line.contains(": @") {
        return extract_type_from_matrix_declaration(line, char_pos, analysis);
    }

    // Check if we're in a qualified reference (@Type:id)
    if line.contains('@') {
        return extract_type_from_reference(line, char_pos);
    }

    None
}

/// Extract type name from %STRUCT: directive.
fn extract_type_from_struct_directive(line: &str, char_pos: usize) -> Option<String> {
    // %STRUCT: TypeName: [col1, col2]
    let rest = line.strip_prefix("%STRUCT")?.trim();
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

    let bracket_start = rest.find('[')?;
    let type_part = &rest[..bracket_start];
    let type_name = type_part.trim().trim_end_matches(':').trim();

    // Calculate position of type name in original line
    let type_start = line.find(type_name)?;
    let type_end = type_start + type_name.len();

    if char_pos >= type_start && char_pos <= type_end {
        Some(type_name.to_string())
    } else {
        None
    }
}

/// Extract type name from %NEST: directive.
fn extract_type_from_nest_directive(line: &str, char_pos: usize) -> Option<String> {
    // %NEST: Parent > Child
    let rest = line.strip_prefix("%NEST")?.trim();
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

    let arrow_pos = rest.find('>')?;
    let parent = rest[..arrow_pos].trim();
    let child = rest[arrow_pos + 1..].trim();

    // Check if cursor is on parent
    if let Some(parent_start) = line.find(parent) {
        let parent_end = parent_start + parent.len();
        if char_pos >= parent_start && char_pos <= parent_end {
            return Some(parent.to_string());
        }
    }

    // Check if cursor is on child
    if let Some(child_start) = line.find(child) {
        let child_end = child_start + child.len();
        if char_pos >= child_start && char_pos <= child_end {
            return Some(child.to_string());
        }
    }

    None
}

/// Extract type name from matrix declaration.
fn extract_type_from_matrix_declaration(
    line: &str,
    char_pos: usize,
    analysis: &AnalyzedDocument,
) -> Option<String> {
    // Format: key: @TypeName
    // Find all occurrences of ": @" to handle multiple on same line
    for (idx, _) in line.match_indices(": @") {
        let at_pos = idx;
        let after_at = &line[at_pos + 3..];
        let type_name: String = after_at
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect();

        if !type_name.is_empty() && analysis.schemas.contains_key(&type_name) {
            let type_start = at_pos + 3;
            let type_end = type_start + type_name.len();

            if char_pos >= type_start && char_pos <= type_end {
                return Some(type_name);
            }
        }
    }

    None
}

/// Extract type name from qualified reference.
fn extract_type_from_reference(line: &str, char_pos: usize) -> Option<String> {
    // Find @ characters and check for Type:id pattern
    for (i, ch) in line.char_indices() {
        if ch == '@' {
            let after_at = &line[i + 1..];
            if let Some(colon_pos) = after_at.find(':') {
                let type_name = &after_at[..colon_pos];
                let type_start = i + 1;
                let type_end = type_start + type_name.len();

                if char_pos >= type_start && char_pos <= type_end {
                    return Some(type_name.to_string());
                }
            }
        }
    }

    None
}

/// Identify field name at position.
fn identify_field_at_position(
    content: &str,
    position: Position,
    _analysis: &AnalyzedDocument,
) -> Option<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];

    // Only check %STRUCT: directives
    if !line.trim().starts_with("%STRUCT") {
        return None;
    }

    // Parse the directive
    let rest = line.strip_prefix("%STRUCT")?.trim();
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

    let bracket_start = rest.find('[')?;
    let bracket_end = rest.find(']')?;

    let type_name = rest[..bracket_start]
        .trim()
        .trim_end_matches(':')
        .trim()
        .to_string();

    let cols_str = &rest[bracket_start + 1..bracket_end];
    let columns: Vec<&str> = cols_str.split(',').map(str::trim).collect();

    // Find which column the cursor is on
    let char_pos = position.character as usize;
    let bracket_start_abs = line.find('[')? + 1;

    let mut current_pos = bracket_start_abs;
    for (i, col) in columns.iter().enumerate() {
        // Skip whitespace
        while current_pos < line.len() && line.chars().nth(current_pos) == Some(' ') {
            current_pos += 1;
        }

        let col_start = current_pos;
        let col_end = col_start + col.len();

        if char_pos >= col_start && char_pos <= col_end {
            return Some((type_name, (*col).to_string()));
        }

        current_pos = col_end;
        // Skip comma and whitespace
        if current_pos < line.len() && line.chars().nth(current_pos) == Some(',') {
            current_pos += 1;
        }
        if i < columns.len() - 1 {
            // Not the last column, advance
        }
    }

    None
}

/// Find all occurrences of a symbol in the document.
#[must_use]
pub fn find_all_occurrences(
    symbol: &SymbolKind,
    analysis: &AnalyzedDocument,
    content: &str,
    uri: &Url,
) -> Vec<SymbolLocation> {
    match symbol {
        SymbolKind::EntityId { type_name, id } => {
            find_entity_occurrences(type_name, id, analysis, content, uri)
        }
        SymbolKind::TypeName(type_name) => find_type_occurrences(type_name, analysis, content, uri),
        SymbolKind::AliasName(alias) => find_alias_occurrences(alias, analysis, content, uri),
        SymbolKind::FieldName {
            type_name,
            field_name,
        } => find_field_occurrences(type_name, field_name, analysis, content, uri),
    }
}

/// Find all entity ID occurrences.
fn find_entity_occurrences(
    type_name: &str,
    id: &str,
    analysis: &AnalyzedDocument,
    _content: &str,
    uri: &Url,
) -> Vec<SymbolLocation> {
    let mut locations = Vec::new();

    // 1. Find definition from reference_index_v2
    if let Some(def_loc) = analysis.reference_index_v2.find_definition(type_name, id) {
        locations.push(SymbolLocation {
            uri: uri.clone(),
            location: def_loc.clone(),
            is_definition: true,
        });
    }

    // 2. Find all references using reference_index_v2
    let qualified_ref = format!("@{type_name}:{id}");
    let unqualified_ref = format!("@{id}");

    for ref_str in &[qualified_ref.as_str(), unqualified_ref.as_str()] {
        for ref_loc in analysis.reference_index_v2.find_references(ref_str) {
            locations.push(SymbolLocation {
                uri: uri.clone(),
                location: ref_loc.clone(),
                is_definition: false,
            });
        }
    }

    locations
}

/// Find all type name occurrences.
fn find_type_occurrences(
    type_name: &str,
    analysis: &AnalyzedDocument,
    content: &str,
    uri: &Url,
) -> Vec<SymbolLocation> {
    let mut locations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // 1. Schema definition in %STRUCT: directive
    if let Some((_, line_num)) = analysis.schemas.get(type_name) {
        if let Some(line) = lines.get(line_num.saturating_sub(1)) {
            if let Some(loc) = find_type_in_struct_directive(line, *line_num, type_name) {
                locations.push(SymbolLocation {
                    uri: uri.clone(),
                    location: loc,
                    is_definition: true,
                });
            }
        }
    }

    // 2. Matrix list declarations (key: @Type)
    for (line_num, line) in lines.iter().enumerate() {
        // Find all occurrences on this line (not just the first one)
        for loc in find_all_types_in_matrix_declaration(line, line_num, type_name) {
            locations.push(SymbolLocation {
                uri: uri.clone(),
                location: loc,
                is_definition: false,
            });
        }
    }

    // 3. Qualified references (@Type:id)
    for (line_num, line) in lines.iter().enumerate() {
        let mut char_pos = 0;
        for ch in line.chars() {
            if ch == '@' {
                let after_at = safe_slice_from(line, char_pos + 1);
                if let Some(colon_pos) = after_at.find(':') {
                    let ref_type = &after_at[..colon_pos];
                    if ref_type == type_name {
                        let start_char = (char_pos + 1) as u32;
                        let end_char = start_char + type_name.len() as u32;
                        locations.push(SymbolLocation {
                            uri: uri.clone(),
                            location: RefLocation::new(line_num as u32, start_char, end_char),
                            is_definition: false,
                        });
                    }
                }
            }
            char_pos += ch.len_utf8();
        }
    }

    // 4. NEST directives
    for (parent, (child, line_num)) in &analysis.nests {
        if parent == type_name || child == type_name {
            if let Some(line) = lines.get(line_num.saturating_sub(1)) {
                if let Some(loc) = find_type_in_nest_directive(line, *line_num, type_name) {
                    locations.push(SymbolLocation {
                        uri: uri.clone(),
                        location: loc,
                        is_definition: false,
                    });
                }
            }
        }
    }

    locations
}

/// Find type name in %STRUCT: directive.
fn find_type_in_struct_directive(
    line: &str,
    line_num: usize,
    type_name: &str,
) -> Option<RefLocation> {
    // Find all occurrences and return the first valid one
    // This fixes the issue where only the first match was found regardless of word boundaries
    let mut byte_pos = 0;
    for ch in line.chars() {
        // Check if this could be the start of our type name
        if line[byte_pos..].starts_with(type_name) {
            // Verify it's actually the type name, not part of another word
            let before = if byte_pos > 0 {
                line[..byte_pos].chars().last()
            } else {
                None
            };

            let after_pos = byte_pos + type_name.len();
            let after = if after_pos < line.len() {
                line[after_pos..].chars().next()
            } else {
                None
            };

            let is_word_boundary = |c: Option<char>| {
                c.is_none() || c.unwrap().is_whitespace() || c.unwrap() == ':' || c.unwrap() == '['
            };

            if is_word_boundary(before) && is_word_boundary(after) {
                return Some(RefLocation::new(
                    (line_num.saturating_sub(1)) as u32,
                    byte_pos as u32,
                    (byte_pos + type_name.len()) as u32,
                ));
            }
        }
        byte_pos += ch.len_utf8();
    }
    None
}

/// Find type name in matrix declaration (deprecated, use `find_all_types_in_matrix_declaration`).
#[allow(dead_code)]
fn find_type_in_matrix_declaration(
    line: &str,
    line_num: usize,
    type_name: &str,
) -> Option<RefLocation> {
    // Format: key: @TypeName
    if let Some(at_pos) = line.find(": @") {
        let after_at = &line[at_pos + 3..];
        if after_at.starts_with(type_name) {
            // Verify word boundary
            let after = after_at.chars().nth(type_name.len());
            if after.is_none() || !after.unwrap().is_alphanumeric() {
                return Some(RefLocation::new(
                    line_num as u32,
                    (at_pos + 3) as u32,
                    (at_pos + 3 + type_name.len()) as u32,
                ));
            }
        }
    }
    None
}

/// Find all type name occurrences in matrix declarations on a line.
/// Handles multiple declarations on the same line like "users: @User, posts: @Post".
fn find_all_types_in_matrix_declaration(
    line: &str,
    line_num: usize,
    type_name: &str,
) -> Vec<RefLocation> {
    let mut locations = Vec::new();

    // Format: key: @TypeName
    // Find all occurrences of ": @" to handle multiple on same line
    for (idx, _) in line.match_indices(": @") {
        let at_pos = idx;
        let after_at = &line[at_pos + 3..];

        if after_at.starts_with(type_name) {
            // Verify word boundary
            let after = after_at.chars().nth(type_name.len());
            if after.is_none() || !after.unwrap().is_alphanumeric() {
                locations.push(RefLocation::new(
                    line_num as u32,
                    (at_pos + 3) as u32,
                    (at_pos + 3 + type_name.len()) as u32,
                ));
            }
        }
    }

    locations
}

/// Find type name in %NEST: directive.
fn find_type_in_nest_directive(
    line: &str,
    line_num: usize,
    type_name: &str,
) -> Option<RefLocation> {
    if let Some(pos) = line.find(type_name) {
        // Verify it's actually the type name
        let before = if pos > 0 {
            line.chars().nth(pos - 1)
        } else {
            None
        };
        let after = line.chars().nth(pos + type_name.len());

        let is_word_boundary = |c: Option<char>| {
            c.is_none() || c.unwrap().is_whitespace() || c.unwrap() == ':' || c.unwrap() == '>'
        };

        if is_word_boundary(before) && is_word_boundary(after) {
            return Some(RefLocation::new(
                (line_num.saturating_sub(1)) as u32,
                pos as u32,
                (pos + type_name.len()) as u32,
            ));
        }
    }
    None
}

/// Find all alias occurrences.
fn find_alias_occurrences(
    alias: &str,
    analysis: &AnalyzedDocument,
    content: &str,
    uri: &Url,
) -> Vec<SymbolLocation> {
    let mut locations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // 1. Alias definition in %ALIAS: directive
    if let Some((_, line_num)) = analysis.aliases.get(alias) {
        if let Some(line) = lines.get(line_num.saturating_sub(1)) {
            if let Some(loc) = find_alias_in_directive(line, *line_num, alias) {
                locations.push(SymbolLocation {
                    uri: uri.clone(),
                    location: loc,
                    is_definition: true,
                });
            }
        }
    }

    // 2. All %alias references in content
    for (line_num, line) in lines.iter().enumerate() {
        let mut char_pos = 0;
        for ch in line.chars() {
            if ch == '%' {
                let after_percent = safe_slice_from(line, char_pos + 1);
                if after_percent.starts_with(alias) {
                    // Verify word boundary
                    let after = after_percent.chars().nth(alias.len());
                    if after.is_none()
                        || (!after.unwrap().is_alphanumeric()
                            && after.unwrap() != '_'
                            && after.unwrap() != '-')
                    {
                        locations.push(SymbolLocation {
                            uri: uri.clone(),
                            location: RefLocation::new(
                                line_num as u32,
                                char_pos as u32,
                                (char_pos + 1 + alias.len()) as u32,
                            ),
                            is_definition: false,
                        });
                    }
                }
            }
            char_pos += ch.len_utf8();
        }
    }

    locations
}

/// Find alias in %ALIAS: directive.
fn find_alias_in_directive(line: &str, line_num: usize, alias: &str) -> Option<RefLocation> {
    // %ALIAS: alias = "value" or %ALIAS: %alias: "value"
    let rest = line.strip_prefix("%ALIAS")?.trim();
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

    // Find the alias name (may have % prefix)
    let alias_with_prefix = format!("%{alias}");

    // Try with % prefix first
    if rest.contains(&alias_with_prefix) {
        let line_pos = line.find(&alias_with_prefix)?;
        return Some(RefLocation::new(
            (line_num.saturating_sub(1)) as u32,
            line_pos as u32,
            (line_pos + alias_with_prefix.len()) as u32,
        ));
    }

    // Try without % prefix
    if let Some(pos) = rest.find(alias) {
        // Verify it's the alias name, not part of value
        let before = if pos > 0 {
            rest.chars().nth(pos - 1)
        } else {
            None
        };
        if before.is_none() || before.unwrap().is_whitespace() || before.unwrap() == ':' {
            let line_pos = line.find(alias)?;
            return Some(RefLocation::new(
                (line_num.saturating_sub(1)) as u32,
                line_pos as u32,
                (line_pos + alias.len()) as u32,
            ));
        }
    }

    None
}

/// Find all field name occurrences.
fn find_field_occurrences(
    type_name: &str,
    field_name: &str,
    analysis: &AnalyzedDocument,
    content: &str,
    uri: &Url,
) -> Vec<SymbolLocation> {
    let mut locations = Vec::new();

    // Field names only appear in %STRUCT: directives
    if let Some((columns, line_num)) = analysis.schemas.get(type_name) {
        if let Some(field_idx) = columns.iter().position(|f| f == field_name) {
            let lines: Vec<&str> = content.lines().collect();
            if let Some(line) = lines.get(line_num.saturating_sub(1)) {
                if let Some(loc) = find_field_in_schema(line, *line_num, field_idx, field_name) {
                    locations.push(SymbolLocation {
                        uri: uri.clone(),
                        location: loc,
                        is_definition: true,
                    });
                }
            }
        }
    }

    locations
}

/// Find field name in %STRUCT: directive.
fn find_field_in_schema(
    line: &str,
    line_num: usize,
    field_idx: usize,
    field_name: &str,
) -> Option<RefLocation> {
    // Parse the directive to find the field position
    let bracket_start = line.find('[')?;
    let bracket_end = line.find(']')?;
    let cols_str = &line[bracket_start + 1..bracket_end];
    let columns: Vec<&str> = cols_str.split(',').map(str::trim).collect();

    if field_idx < columns.len() {
        // Find position of this field in the line
        let target_field = columns[field_idx];
        if target_field == field_name {
            // Find the absolute position in the line
            let before_bracket = &line[..=bracket_start];
            let mut search_str = String::from(before_bracket);

            for (i, col) in columns.iter().enumerate() {
                if i == field_idx {
                    let field_start = search_str.len();
                    return Some(RefLocation::new(
                        (line_num.saturating_sub(1)) as u32,
                        field_start as u32,
                        (field_start + field_name.len()) as u32,
                    ));
                }
                search_str.push_str(col);
                if i < columns.len() - 1 {
                    search_str.push_str(", ");
                }
            }
        }
    }

    None
}

/// Validate a rename operation for safety and correctness.
#[must_use]
pub fn validate_rename(
    symbol: &SymbolKind,
    new_name: &str,
    analysis: &AnalyzedDocument,
) -> RenameValidation {
    let mut validation = RenameValidation {
        valid: true,
        error: None,
        warnings: Vec::new(),
    };

    // 1. Name syntax validation
    if !is_valid_identifier(new_name, symbol) {
        validation.valid = false;
        validation.error = Some(format!(
            "Invalid identifier '{new_name}': must match HEDL naming rules"
        ));
        return validation;
    }

    // 2. Conflict detection
    match symbol {
        SymbolKind::EntityId { type_name, id } => {
            if let Some(entities) = analysis.entities.get(type_name) {
                if entities.contains_key(new_name) && new_name != id.as_str() {
                    validation.valid = false;
                    validation.error = Some(format!(
                        "Conflict: Entity '{new_name}' of type '{type_name}' already exists"
                    ));
                }
            }
        }
        SymbolKind::TypeName(old_name) => {
            if analysis.schemas.contains_key(new_name) && old_name != new_name {
                validation.valid = false;
                validation.error = Some(format!("Conflict: Type '{new_name}' already exists"));
            }

            // Warn if type has many references
            let ref_count = count_type_references(old_name, analysis);
            if ref_count > 50 {
                validation.warnings.push(format!(
                    "Type '{old_name}' has {ref_count} references. This is a large rename operation."
                ));
            }
        }
        SymbolKind::AliasName(old_name) => {
            if analysis.aliases.contains_key(new_name) && old_name != new_name {
                validation.valid = false;
                validation.error = Some(format!("Conflict: Alias '{new_name}' already exists"));
            }
        }
        SymbolKind::FieldName {
            type_name,
            field_name,
        } => {
            if let Some((columns, _)) = analysis.schemas.get(type_name) {
                if columns.contains(&new_name.to_string()) && new_name != field_name {
                    validation.valid = false;
                    validation.error = Some(format!(
                        "Conflict: Field '{new_name}' already exists in type '{type_name}'"
                    ));
                }
            }
        }
    }

    // 3. Case sensitivity warnings
    if let Some(warning) = check_case_similarity(symbol, new_name, analysis) {
        validation.warnings.push(warning);
    }

    validation
}

/// Check if identifier is valid according to HEDL syntax rules.
fn is_valid_identifier(name: &str, _symbol: &SymbolKind) -> bool {
    if name.is_empty() {
        return false;
    }

    // Pattern: [a-zA-Z][a-zA-Z0-9_-]*
    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphabetic() {
        return false;
    }

    for ch in name.chars().skip(1) {
        if !ch.is_alphanumeric() && ch != '_' && ch != '-' {
            return false;
        }
    }

    // Reserved keywords check
    const RESERVED: &[&str] = &["true", "false", "null"];
    if RESERVED.contains(&name.to_lowercase().as_str()) {
        return false;
    }

    true
}

/// Count type references in analysis.
fn count_type_references(type_name: &str, analysis: &AnalyzedDocument) -> usize {
    let mut count = 0;

    // Count entities of this type
    if let Some(entities) = analysis.entities.get(type_name) {
        count += entities.len();
    }

    // Count references that use this type
    for (ref_type, _, _) in &analysis.references {
        if ref_type.as_deref() == Some(type_name) {
            count += 1;
        }
    }

    count
}

/// Check for case similarity warnings.
fn check_case_similarity(
    symbol: &SymbolKind,
    new_name: &str,
    analysis: &AnalyzedDocument,
) -> Option<String> {
    match symbol {
        SymbolKind::TypeName(_) => {
            for existing_type in analysis.schemas.keys() {
                if existing_type.to_lowercase() == new_name.to_lowercase()
                    && existing_type != new_name
                {
                    return Some(format!(
                        "Warning: New name '{new_name}' is similar to existing type '{existing_type}' (differs only in case)"
                    ));
                }
            }
        }
        SymbolKind::EntityId { type_name, .. } => {
            if let Some(entities) = analysis.entities.get(type_name) {
                for existing_id in entities.keys() {
                    if existing_id.to_lowercase() == new_name.to_lowercase()
                        && existing_id != new_name
                    {
                        return Some(format!(
                            "Warning: New name '{new_name}' is similar to existing entity '{existing_id}' (differs only in case)"
                        ));
                    }
                }
            }
        }
        _ => {}
    }
    None
}

/// Generate workspace edit for rename operation.
pub fn generate_workspace_edit(
    operation: &RenameOperation,
    document_manager: &DocumentManager,
) -> Result<WorkspaceEdit, String> {
    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();

    for sym_loc in &operation.locations {
        // Retrieve the content to inspect the original text
        let content = {
            if let Some((content, _analysis)) = document_manager.get(&sym_loc.uri) {
                content
            } else {
                return Err(format!("Document not found: {}", sym_loc.uri));
            }
        };

        let edit = TextEdit {
            range: sym_loc.location.to_range(),
            new_text: generate_replacement_text(
                &operation.symbol,
                &operation.new_name,
                sym_loc,
                &content,
            ),
        };

        changes.entry(sym_loc.uri.clone()).or_default().push(edit);
    }

    // Sort edits by position (reverse order for correct application)
    for edits in changes.values_mut() {
        edits.sort_by(|a, b| {
            b.range
                .start
                .line
                .cmp(&a.range.start.line)
                .then_with(|| b.range.start.character.cmp(&a.range.start.character))
        });
    }

    Ok(WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    })
}

/// Check if an identifier requires quoting in HEDL.
fn requires_quoting(id: &str) -> bool {
    // IDs with hyphens, spaces, or other special characters need quoting
    // Pattern: [a-zA-Z][a-zA-Z0-9_]* doesn't require quoting
    if id.is_empty() {
        return true;
    }

    let first_char = id.chars().next().unwrap();
    if !first_char.is_alphabetic() {
        return true;
    }

    for ch in id.chars().skip(1) {
        if !ch.is_alphanumeric() && ch != '_' {
            return true; // Contains hyphen or other special character
        }
    }

    false
}

/// Generate the replacement text for a specific location.
fn generate_replacement_text(
    symbol: &SymbolKind,
    new_name: &str,
    location: &SymbolLocation,
    content: &str,
) -> String {
    match symbol {
        SymbolKind::EntityId { type_name, .. } => {
            if location.is_definition {
                new_name.to_string()
            } else {
                // References: inspect original text to preserve @Type: or @ prefix and quotes
                let lines: Vec<&str> = content.lines().collect();
                if let Some(line) = lines.get(location.location.line as usize) {
                    let start = location.location.start_char as usize;
                    let end = location.location.end_char as usize;

                    if start < line.len() && end <= line.len() {
                        let original_text = &line[start..end];

                        // Determine if the ID needs quoting based on the original
                        let needs_quoting = requires_quoting(new_name);

                        // Check if it's a qualified reference (@Type:id or @Type:"id")
                        if original_text.starts_with('@') && original_text.contains(':') {
                            if needs_quoting {
                                return format!("@{type_name}:\"{new_name}\"");
                            }
                            return format!("@{type_name}:{new_name}");
                        }
                        // Check if it's an unqualified reference (@id or @"id")
                        else if original_text.starts_with('@') {
                            if needs_quoting {
                                return format!("@\"{new_name}\"");
                            }
                            return format!("@{new_name}");
                        }
                    }
                }
                // Fallback: just the new name (with quotes if needed)
                if requires_quoting(new_name) {
                    format!("\"{new_name}\"")
                } else {
                    new_name.to_string()
                }
            }
        }
        SymbolKind::TypeName(_) => {
            // Type names appear bare in most contexts, but can have @ prefix
            let lines: Vec<&str> = content.lines().collect();
            if let Some(line) = lines.get(location.location.line as usize) {
                let start = location.location.start_char as usize;

                // Check if there's an @ before this position
                if start > 0 && line.chars().nth(start - 1) == Some('@') {
                    // The location doesn't include the @, so just return the name
                    return new_name.to_string();
                }
            }
            new_name.to_string()
        }
        SymbolKind::AliasName(_) => {
            if location.is_definition {
                new_name.to_string()
            } else {
                // Alias references already include % in the location
                // So we just need the name
                new_name.to_string()
            }
        }
        SymbolKind::FieldName { .. } => {
            // Field names are always bare
            new_name.to_string()
        }
    }
}

/// Get the symbol name from a `SymbolKind`.
#[must_use]
pub fn get_symbol_name(symbol: &SymbolKind) -> String {
    match symbol {
        SymbolKind::EntityId { id, .. } => id.clone(),
        SymbolKind::TypeName(name) => name.clone(),
        SymbolKind::AliasName(name) => name.clone(),
        SymbolKind::FieldName { field_name, .. } => field_name.clone(),
    }
}

/// Get the symbol range at a position.
#[must_use]
pub fn get_symbol_range_at_position(
    analysis: &AnalyzedDocument,
    content: &str,
    position: Position,
    symbol: &SymbolKind,
) -> Option<Range> {
    // Use the reference index for entity references
    if matches!(symbol, SymbolKind::EntityId { .. }) {
        if let Some((_, loc)) = analysis.reference_index_v2.find_reference_at(position) {
            return Some(loc.to_range());
        }
    }

    // For other symbol types, find the range manually
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }

    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    let symbol_name = get_symbol_name(symbol);

    // Find the symbol name around the cursor position
    if let Some(start) = line[..char_pos.min(line.len())].rfind(&symbol_name) {
        if char_pos >= start && char_pos <= start + symbol_name.len() {
            return Some(Range {
                start: Position {
                    line: position.line,
                    character: start as u32,
                },
                end: Position {
                    line: position.line,
                    character: (start + symbol_name.len()) as u32,
                },
            });
        }
    }

    // Try searching forward
    if let Some(start) = line[char_pos..].find(&symbol_name) {
        let abs_start = char_pos + start;
        return Some(Range {
            start: Position {
                line: position.line,
                character: abs_start as u32,
            },
            end: Position {
                line: position.line,
                character: (abs_start + symbol_name.len()) as u32,
            },
        });
    }

    None
}

/// Find all occurrences across the entire workspace.
#[must_use]
pub fn find_all_occurrences_workspace(
    symbol: &SymbolKind,
    document_manager: &DocumentManager,
) -> Vec<SymbolLocation> {
    let mut all_locations = Vec::new();

    document_manager.for_each(|uri, state_arc| {
        let (content, analysis) = {
            let state = state_arc.lock();
            (state.rope.to_string(), state.analysis.clone())
        };

        let locations = find_all_occurrences(symbol, analysis.as_ref(), &content, uri);

        all_locations.extend(locations);
    });

    all_locations
}

/// Validate rename across workspace.
#[must_use]
pub fn validate_rename_workspace(
    symbol: &SymbolKind,
    new_name: &str,
    document_manager: &DocumentManager,
) -> RenameValidation {
    let mut validation = RenameValidation {
        valid: true,
        error: None,
        warnings: Vec::new(),
    };

    // Check for conflicts in all documents
    document_manager.for_each(|_uri, state_arc| {
        let state = state_arc.lock();
        let analysis = &state.analysis;

        let doc_validation = validate_rename(symbol, new_name, analysis.as_ref());

        if !doc_validation.valid {
            validation.valid = false;
            validation.error = doc_validation.error;
        }

        validation.warnings.extend(doc_validation.warnings);
    });

    // Deduplicate warnings
    validation.warnings.sort();
    validation.warnings.dedup();

    validation
}
