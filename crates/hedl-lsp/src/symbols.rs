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

//! Document symbols for HEDL files.
//!
//! This module provides symbol extraction for LSP features like:
//! - Document outline view
//! - Workspace-wide symbol search
//! - Breadcrumb navigation
//!
//! # Error Handling
//!
//! Symbol extraction is designed to be fault-tolerant:
//! - Empty documents return empty symbol lists
//! - Malformed schemas/entities are skipped
//! - Position calculations are bounds-checked

use crate::analysis::AnalyzedDocument;
use crate::constants::{
    HEADER_SELECTION_CHAR, LINE_NUMBER_OFFSET, POSITION_ZERO, SYMBOL_LINE_END_CHAR,
};
use tower_lsp::lsp_types::{
    DocumentSymbol, Location, Position, Range, SymbolInformation, SymbolKind, Url,
};
use tracing::{debug, warn};

/// Construct a `DocumentSymbol` without triggering the deprecated `deprecated` field warning.
///
/// The `deprecated` field on `DocumentSymbol` is deprecated in lsp-types in favor of
/// `tags` (using `SymbolTag::DEPRECATED`). Since we must still set the field in struct
/// literals (no `Default` impl), this helper centralizes the suppression in one place.
fn make_document_symbol(
    name: String,
    detail: Option<String>,
    kind: SymbolKind,
    range: Range,
    selection_range: Range,
    children: Option<Vec<DocumentSymbol>>,
) -> DocumentSymbol {
    // The `deprecated` field on DocumentSymbol is deprecated in lsp-types.
    // The replacement is `tags` with `SymbolTag::DEPRECATED`. Since we never
    // mark symbols as deprecated, both fields are None. We must still set the
    // struct field because DocumentSymbol does not implement Default.
    #[allow(deprecated)]
    DocumentSymbol {
        name,
        detail,
        kind,
        tags: None,
        deprecated: None,
        range,
        selection_range,
        children,
    }
}

/// Construct a `SymbolInformation` without triggering the deprecated `deprecated` field warning.
///
/// Same rationale as [`make_document_symbol`]: the `deprecated` field is deprecated
/// in favor of `tags`.
fn make_symbol_information(
    name: String,
    kind: SymbolKind,
    location: Location,
    container_name: Option<String>,
) -> SymbolInformation {
    #[allow(deprecated)]
    SymbolInformation {
        name,
        kind,
        tags: None,
        deprecated: None,
        location,
        container_name,
    }
}

/// Get document symbols for outline view.
///
/// # Error Handling
///
/// - Empty content: Returns empty symbol list
/// - Missing header delimiter: Treated as header-only document
/// - Invalid line numbers: Clamped to valid ranges
pub fn get_document_symbols(analysis: &AnalyzedDocument, content: &str) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        debug!("Empty document, no symbols to extract");
        return symbols;
    }

    // Find header section
    let header_end = lines.iter().position(|l| *l == "---").unwrap_or_else(|| {
        debug!("No header delimiter found, treating entire document as header");
        lines.len()
    });

    // Add header as a container
    if header_end > 0 {
        let mut header_children = Vec::new();

        // Add schemas
        for (type_name, (cols, line)) in &analysis.schemas {
            debug!(
                "Adding schema symbol: {} with {} columns at line {}",
                type_name,
                cols.len(),
                line
            );
            header_children.push(make_document_symbol(
                type_name.clone(),
                Some(format!("[{}]", cols.join(", "))),
                SymbolKind::STRUCT,
                line_range(*line),
                line_range(*line),
                None,
            ));
        }

        // Add aliases
        for (alias, (value, line)) in &analysis.aliases {
            debug!(
                "Adding alias symbol: {} = '{}' at line {}",
                alias, value, line
            );
            header_children.push(make_document_symbol(
                format!("${alias}"),
                Some(format!("= \"{value}\"")),
                SymbolKind::CONSTANT,
                line_range(*line),
                line_range(*line),
                None,
            ));
        }

        // Add nests
        for (parent, child_list) in &analysis.nests {
            for (child, line) in child_list {
                debug!(
                    "Adding nest symbol: {} > {} at line {}",
                    parent, child, line
                );
                header_children.push(make_document_symbol(
                    format!("{parent} > {child}"),
                    Some("Nesting relationship".to_string()),
                    SymbolKind::INTERFACE,
                    line_range(*line),
                    line_range(*line),
                    None,
                ));
            }
        }

        if !header_children.is_empty() {
            debug!(
                "Adding header container with {} directives",
                header_children.len()
            );
            symbols.push(make_document_symbol(
                "Header".to_string(),
                Some(format!("{} directives", header_children.len())),
                SymbolKind::NAMESPACE,
                Range {
                    start: Position {
                        line: POSITION_ZERO,
                        character: POSITION_ZERO,
                    },
                    end: Position {
                        line: header_end as u32,
                        character: POSITION_ZERO,
                    },
                },
                Range {
                    start: Position {
                        line: POSITION_ZERO,
                        character: POSITION_ZERO,
                    },
                    end: Position {
                        line: POSITION_ZERO,
                        character: HEADER_SELECTION_CHAR,
                    },
                },
                Some(header_children),
            ));
        }
    }

    // Add entities by type
    for (type_name, entities) in &analysis.entities {
        if entities.is_empty() {
            warn!("Type {} has no entities, skipping symbol", type_name);
            continue;
        }

        let mut entity_children = Vec::new();

        for (id, line) in entities {
            entity_children.push(make_document_symbol(
                id.clone(),
                None,
                SymbolKind::OBJECT,
                line_range(*line),
                line_range(*line),
                None,
            ));
        }

        // Sort by line number
        entity_children.sort_by_key(|s| s.range.start.line);

        let first_line = entities.values().min().copied().unwrap_or(header_end + 1);
        let last_line = entities.values().max().copied().unwrap_or(header_end + 1);

        debug!(
            "Adding type symbol: {} with {} entities (lines {}-{})",
            type_name,
            entities.len(),
            first_line,
            last_line
        );

        symbols.push(make_document_symbol(
            type_name.clone(),
            Some(format!("{} entities", entities.len())),
            SymbolKind::CLASS,
            Range {
                start: Position {
                    line: first_line as u32,
                    character: POSITION_ZERO,
                },
                end: Position {
                    line: last_line as u32,
                    character: SYMBOL_LINE_END_CHAR,
                },
            },
            Range {
                start: Position {
                    line: first_line as u32,
                    character: POSITION_ZERO,
                },
                end: Position {
                    line: first_line as u32,
                    character: type_name.len() as u32,
                },
            },
            Some(entity_children),
        ));
    }

    debug!(
        "Document symbols extraction complete: {} top-level symbols",
        symbols.len()
    );

    symbols
}

/// Get workspace symbols matching a query.
///
/// # Error Handling
///
/// - Empty query: Returns all symbols (filtered by caller)
/// - Invalid URIs: Placeholder URI used (replaced by caller)
/// - Case-insensitive matching for better UX
pub fn get_workspace_symbols(analysis: &AnalyzedDocument, query: &str) -> Vec<SymbolInformation> {
    let mut symbols = Vec::new();
    let query_lower = query.to_lowercase();

    debug!(
        "Searching workspace symbols with query: '{}' (case-insensitive)",
        query
    );

    // Search schemas
    for (type_name, (_cols, line)) in &analysis.schemas {
        if type_name.to_lowercase().contains(&query_lower) {
            debug!("Schema '{}' matches query '{}'", type_name, query);
            symbols.push(make_symbol_information(
                type_name.clone(),
                SymbolKind::STRUCT,
                Location {
                    // SAFETY: "file:///" is a valid URL constant
                    uri: Url::parse("file:///").expect("valid file URL"),
                    range: line_range(*line),
                },
                Some("Schema".to_string()),
            ));
        }
    }

    // Search entities
    for (type_name, entities) in &analysis.entities {
        for (id, line) in entities {
            if id.to_lowercase().contains(&query_lower)
                || type_name.to_lowercase().contains(&query_lower)
            {
                symbols.push(make_symbol_information(
                    id.clone(),
                    SymbolKind::OBJECT,
                    Location {
                        // SAFETY: "file:///" is a valid URL constant
                        uri: Url::parse("file:///").expect("valid file URL"),
                        range: line_range(*line),
                    },
                    Some(type_name.clone()),
                ));
            }
        }
    }

    // Search aliases
    for (alias, (_, line)) in &analysis.aliases {
        if alias.to_lowercase().contains(&query_lower) {
            debug!("Alias '{}' matches query '{}'", alias, query);
            symbols.push(make_symbol_information(
                format!("${alias}"),
                SymbolKind::CONSTANT,
                Location {
                    // SAFETY: "file:///" is a valid URL constant
                    uri: Url::parse("file:///").expect("valid file URL"),
                    range: line_range(*line),
                },
                Some("Alias".to_string()),
            ));
        }
    }

    debug!(
        "Workspace symbol search for '{}' found {} matches",
        query,
        symbols.len()
    );

    symbols
}

fn line_range(line: usize) -> Range {
    Range {
        start: Position {
            line: (line.saturating_sub(LINE_NUMBER_OFFSET)) as u32,
            character: POSITION_ZERO,
        },
        end: Position {
            line: (line.saturating_sub(LINE_NUMBER_OFFSET)) as u32,
            character: SYMBOL_LINE_END_CHAR,
        },
    }
}
