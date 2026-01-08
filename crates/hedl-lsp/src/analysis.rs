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

//! Document analysis for HEDL files.
//!
//! This module provides comprehensive analysis of HEDL documents, extracting:
//! - Parse errors and lint diagnostics
//! - Entity definitions with line numbers
//! - Schema (type) definitions
//! - Alias definitions
//! - Reference usages for go-to-definition and find-references
//! - Nesting relationships
//!
//! The analysis results are cached and used by other LSP features like
//! completion, hover, and symbols.

use crate::constants::{DIAGNOSTIC_LINE_END_CHAR, LINE_NUMBER_OFFSET, POSITION_ZERO};
use crate::reference_index::{RefLocation, ReferenceIndex};
use hedl_core::{parse, Document, HedlError, Item, Node, Value};
use hedl_lint::{lint, Diagnostic, Severity};
use std::collections::HashMap;
use tower_lsp::lsp_types::*;
use tracing::{debug, warn};

/// Analyzed document with parsed content and diagnostics.
///
/// This structure contains the complete analysis of a HEDL document, including
/// both the parsed AST and extracted metadata for LSP features.
///
/// # Structure
///
/// - **document**: The parsed AST (if parsing succeeded)
/// - **errors**: Any parse errors encountered
/// - **lint_diagnostics**: Warnings and suggestions from the linter
/// - **entities**: Index of all entity definitions (type → id → line number)
/// - **schemas**: Schema definitions (type → (columns, line number))
/// - **aliases**: Alias definitions (name → (value, line number))
/// - **references**: All reference usages for find-references
/// - **reference_index_v2**: Enhanced O(1) reference index with precise location tracking
/// - **reference_index**: Legacy line-based index (deprecated, kept for compatibility)
/// - **nests**: Nesting relationships (parent → (child, line number))
/// - **header_end_line**: Cached line number where header ends (--- delimiter)
///
/// # Performance
///
/// The analysis is performed once and cached. Line number estimates are used
/// since the parser doesn't preserve exact line information. This is acceptable
/// for LSP features where approximate positioning is sufficient.
///
/// ## Optimizations
///
/// 1. **Cached Header Boundary**: `header_end_line` eliminates O(n) scan on every completion
/// 2. **Reference Index V2**: `reference_index_v2` provides O(1) lookup with precise locations
/// 3. **Reference Index**: Legacy `reference_index` provides O(1) line-based lookup
/// 4. **Arc-wrapped**: Shared via Arc to avoid unnecessary clones in concurrent access
#[derive(Debug, Clone)]
pub struct AnalyzedDocument {
    /// The parsed document (if successful).
    pub document: Option<Document>,
    /// Parse errors.
    pub errors: Vec<HedlError>,
    /// Lint diagnostics.
    pub lint_diagnostics: Vec<Diagnostic>,
    /// Entity registry: type -> id -> line number.
    pub entities: HashMap<String, HashMap<String, usize>>,
    /// Schema definitions: type -> (columns, line number).
    pub schemas: HashMap<String, (Vec<String>, usize)>,
    /// Alias definitions: alias -> (value, line number).
    pub aliases: HashMap<String, (String, usize)>,
    /// Reference usages: (type, id) -> line numbers.
    pub references: Vec<(Option<String>, String, usize)>,
    /// Enhanced reference index with precise location tracking.
    /// Provides O(1) lookups for both definitions and references with character-level precision.
    pub reference_index_v2: ReferenceIndex,
    /// Legacy fast lookup index for references: reference_string -> vec of line_numbers.
    /// Deprecated: Use reference_index_v2 for new code.
    /// This eliminates O(n) linear search bottleneck for find-references operations.
    pub reference_index: HashMap<String, Vec<usize>>,
    /// Nest relationships: parent -> (child, line number).
    pub nests: HashMap<String, (String, usize)>,
    /// Cached line number where header ends (--- delimiter).
    /// This eliminates O(n) scan on every completion context determination.
    pub header_end_line: Option<usize>,
}

impl AnalyzedDocument {
    /// Analyze a HEDL document and extract all metadata.
    ///
    /// This is the main entry point for document analysis. It performs:
    /// 1. Header parsing to extract schemas, aliases, and nests with line numbers
    /// 2. Full document parsing via hedl-core
    /// 3. Linting via hedl-lint
    /// 4. Entity and reference extraction from the AST
    ///
    /// # Arguments
    ///
    /// * `content` - The full document content as a string
    ///
    /// # Returns
    ///
    /// An `AnalyzedDocument` containing all extracted metadata. Even if parsing
    /// fails, the returned structure will contain parse errors and any metadata
    /// extracted from the header.
    ///
    /// # Performance
    ///
    /// This operation is O(n) where n is the document size. The result should be
    /// cached and reused for multiple LSP queries.
    ///
    /// # Error Handling
    ///
    /// All errors during analysis are captured and stored in the `errors` field.
    /// Parse errors, linting errors, and entity extraction errors are all handled
    /// gracefully to ensure partial analysis results are still available for LSP features.
    pub fn analyze(content: &str) -> Self {
        let mut result = Self {
            document: None,
            errors: Vec::new(),
            lint_diagnostics: Vec::new(),
            entities: HashMap::new(),
            schemas: HashMap::new(),
            aliases: HashMap::new(),
            references: Vec::new(),
            reference_index_v2: ReferenceIndex::new(),
            reference_index: HashMap::new(),
            nests: HashMap::new(),
            header_end_line: None,
        };

        debug!(
            "Starting document analysis: {} bytes, {} lines",
            content.len(),
            content.lines().count()
        );

        // Parse header for schemas, aliases, nests
        result.parse_header(content);

        // Parse document
        match parse(content.as_bytes()) {
            Ok(doc) => {
                debug!("Document parsed successfully");

                // Run linting
                result.lint_diagnostics = lint(&doc);
                debug!(
                    "Linting complete: {} diagnostics",
                    result.lint_diagnostics.len()
                );

                // Extract entities and references
                result.extract_entities(&doc);
                debug!(
                    "Entity extraction complete: {} types, {} total entities, {} references",
                    result.entities.len(),
                    result.entities.values().map(|m| m.len()).sum::<usize>(),
                    result.references.len()
                );

                result.document = Some(doc);
            }
            Err(e) => {
                warn!(
                    "Document parse failed at line {}: {}",
                    e.line, e.message
                );
                result.errors.push(e);
            }
        }

        // Build reference index after all references are collected
        result.build_reference_index();
        result.build_reference_index_v2(content);

        debug!(
            "Analysis complete: {} schemas, {} aliases, {} nests, {} entities, {} references",
            result.schemas.len(),
            result.aliases.len(),
            result.nests.len(),
            result.entities.values().map(|m| m.len()).sum::<usize>(),
            result.references.len()
        );

        result
    }

    /// Parse header directives to extract line numbers.
    ///
    /// # Performance Optimization
    ///
    /// This method now caches the header end line number to eliminate O(n) scans
    /// during completion context determination.
    ///
    /// # Error Handling
    ///
    /// Malformed directives are silently skipped to allow partial header analysis.
    /// Individual directive parsing errors don't prevent the entire header from being processed.
    fn parse_header(&mut self, content: &str) {
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            if line == "---" {
                // Cache the header end line for completion optimization
                self.header_end_line = Some(line_num);
                debug!("Header ends at line {}", line_num);
                break; // End of header
            }

            if line.starts_with("%STRUCT") {
                match parse_struct_directive(line) {
                    Some(def) => {
                        // Convert 0-based line_num to 1-based line number for storage
                        let type_name = def.0.clone();
                        let cols_count = def.1.len();
                        self.schemas.insert(def.0, (def.1, line_num + LINE_NUMBER_OFFSET));
                        debug!(
                            "Parsed STRUCT directive at line {}: {} with {} columns",
                            line_num + LINE_NUMBER_OFFSET,
                            type_name,
                            cols_count
                        );
                    }
                    None => {
                        warn!(
                            "Malformed STRUCT directive at line {}: '{}'",
                            line_num + LINE_NUMBER_OFFSET,
                            line
                        );
                    }
                }
            } else if line.starts_with("%ALIAS") {
                match parse_alias_directive(line) {
                    Some((alias, value)) => {
                        self.aliases.insert(
                            alias.clone(),
                            (value.clone(), line_num + LINE_NUMBER_OFFSET),
                        );
                        debug!(
                            "Parsed ALIAS directive at line {}: {} = '{}'",
                            line_num + LINE_NUMBER_OFFSET,
                            alias,
                            value
                        );
                    }
                    None => {
                        warn!(
                            "Malformed ALIAS directive at line {}: '{}'",
                            line_num + LINE_NUMBER_OFFSET,
                            line
                        );
                    }
                }
            } else if line.starts_with("%NEST") {
                match parse_nest_directive(line) {
                    Some((parent, child)) => {
                        self.nests.insert(
                            parent.clone(),
                            (child.clone(), line_num + LINE_NUMBER_OFFSET),
                        );
                        debug!(
                            "Parsed NEST directive at line {}: {} > {}",
                            line_num + LINE_NUMBER_OFFSET,
                            parent,
                            child
                        );
                    }
                    None => {
                        warn!(
                            "Malformed NEST directive at line {}: '{}'",
                            line_num + LINE_NUMBER_OFFSET,
                            line
                        );
                    }
                }
            }
        }
    }

    /// Extract entities from parsed document.
    fn extract_entities(&mut self, doc: &Document) {
        for item in doc.root.values() {
            self.extract_from_item(item, 0);
        }
    }

    fn extract_from_item(&mut self, item: &Item, line_estimate: usize) {
        match item {
            Item::List(list) => {
                for (i, node) in list.rows.iter().enumerate() {
                    self.entities
                        .entry(list.type_name.clone())
                        .or_default()
                        .insert(node.id.clone(), line_estimate + i);

                    // Extract references from fields
                    for value in &node.fields {
                        if let Value::Reference(r) = value {
                            self.references.push((
                                r.type_name.clone(),
                                r.id.clone(),
                                line_estimate + i,
                            ));
                        }
                    }

                    // Recurse into children
                    for children in node.children.values() {
                        for child in children {
                            self.extract_from_node(child, line_estimate + i);
                        }
                    }
                }
            }
            Item::Object(obj) => {
                for child in obj.values() {
                    self.extract_from_item(child, line_estimate);
                }
            }
            Item::Scalar(v) => {
                if let Value::Reference(r) = v {
                    self.references
                        .push((r.type_name.clone(), r.id.clone(), line_estimate));
                }
            }
        }
    }

    fn extract_from_node(&mut self, node: &Node, line: usize) {
        self.entities
            .entry(node.type_name.clone())
            .or_default()
            .insert(node.id.clone(), line);

        for value in &node.fields {
            if let Value::Reference(r) = value {
                self.references
                    .push((r.type_name.clone(), r.id.clone(), line));
            }
        }

        for children in node.children.values() {
            for child in children {
                self.extract_from_node(child, line);
            }
        }
    }

    /// Build reference index for fast lookup.
    ///
    /// # Performance Optimization
    ///
    /// This method creates a HashMap index of all references for O(1) lookup
    /// during find-references operations, eliminating the O(n) linear search
    /// bottleneck.
    ///
    /// The index maps reference strings (e.g., "@User:alice", "@bob") to vectors
    /// of line numbers where they appear.
    fn build_reference_index(&mut self) {
        for (type_name, id, line) in &self.references {
            // Index both qualified (@Type:id) and unqualified (@id) forms
            let ref_str = match type_name {
                Some(t) => format!("@{}:{}", t, id),
                None => format!("@{}", id),
            };

            self.reference_index
                .entry(ref_str)
                .or_default()
                .push(*line);

            // Also index by just the ID for flexible lookup
            self.reference_index
                .entry(format!("@{}", id))
                .or_default()
                .push(*line);
        }
    }

    /// Build enhanced reference index v2 with precise character positions.
    ///
    /// # Performance Optimization
    ///
    /// This method scans the document content to find exact character positions
    /// for all references and definitions, enabling precise editor navigation
    /// and highlighting.
    ///
    /// # Implementation
    ///
    /// Uses a single-pass scan through the document content to find all '@' tokens
    /// and extract their precise locations. This is more accurate than relying on
    /// estimated line numbers from the parser.
    fn build_reference_index_v2(&mut self, content: &str) {
        // First, index all entity definitions with precise positions
        // Note: We scan the entire document since line numbers from the parser are estimates
        let lines: Vec<&str> = content.lines().collect();

        for (type_name, entities) in &self.entities {
            for id in entities.keys() {
                // Search for the definition across all lines since estimates may be off
                for (line_num, line_content) in lines.iter().enumerate() {
                    // Look for the ID at the start of a matrix row (after |)
                    if let Some(pipe_pos) = line_content.find('|') {
                        let after_pipe = &line_content[pipe_pos + 1..];
                        let trimmed = after_pipe.trim_start();
                        // Check if this line starts with the ID we're looking for
                        if trimmed.starts_with(id.as_str()) {
                            // Verify it's actually the ID (followed by comma or whitespace)
                            let after_id = &trimmed[id.len()..];
                            if after_id.starts_with(',') || after_id.starts_with(char::is_whitespace) || after_id.is_empty() {
                                let start_char =
                                    (pipe_pos + 1 + (after_pipe.len() - trimmed.len())) as u32;
                                let end_char = start_char + id.len() as u32;

                                let location = RefLocation::new(line_num as u32, start_char, end_char);
                                self.reference_index_v2.add_definition(
                                    type_name.clone(),
                                    id.clone(),
                                    location,
                                );
                                break; // Found the definition, move to next entity
                            }
                        }
                    }
                }
            }
        }

        // Now scan for all reference usages (@Type:id or @id)
        for (line_num, line) in content.lines().enumerate() {
            let mut char_pos = 0;
            let mut chars = line.chars().peekable();

            while let Some(ch) = chars.next() {
                if ch == '@' {
                    let start_char = char_pos as u32;
                    let mut ref_str = String::from("@");
                    let mut end_char = start_char + 1;

                    // Read the reference (alphanumeric, underscore, colon)
                    while let Some(&next_ch) = chars.peek() {
                        if next_ch.is_alphanumeric()
                            || next_ch == '_'
                            || next_ch == ':'
                            || next_ch == '-'
                        {
                            ref_str.push(next_ch);
                            chars.next();
                            end_char += next_ch.len_utf8() as u32;
                        } else {
                            break;
                        }
                    }

                    // Parse the reference
                    if ref_str.len() > 1 {
                        let ref_content = &ref_str[1..]; // Remove '@'

                        if let Some(colon_pos) = ref_content.find(':') {
                            // Qualified reference: @Type:id
                            let type_name = ref_content[..colon_pos].to_string();
                            let id = ref_content[colon_pos + 1..].to_string();

                            let location = RefLocation::new(line_num as u32, start_char, end_char);
                            self.reference_index_v2
                                .add_reference(Some(type_name), id, location);
                        } else {
                            // Unqualified reference: @id
                            let id = ref_content.to_string();
                            let location = RefLocation::new(line_num as u32, start_char, end_char);
                            self.reference_index_v2.add_reference(None, id, location);
                        }
                    }
                }

                char_pos += ch.len_utf8();
            }
        }
    }

    /// Convert to LSP diagnostics.
    pub fn to_lsp_diagnostics(&self) -> Vec<tower_lsp::lsp_types::Diagnostic> {
        let mut result = Vec::new();

        // Convert parse errors
        for error in &self.errors {
            result.push(tower_lsp::lsp_types::Diagnostic {
                range: Range {
                    start: Position {
                        line: (error.line.saturating_sub(LINE_NUMBER_OFFSET)) as u32,
                        character: POSITION_ZERO,
                    },
                    end: Position {
                        line: (error.line.saturating_sub(LINE_NUMBER_OFFSET)) as u32,
                        character: DIAGNOSTIC_LINE_END_CHAR,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String(format!("{:?}", error.kind))),
                source: Some("hedl".to_string()),
                message: error.message.clone(),
                ..Default::default()
            });
        }

        // Convert lint diagnostics
        for diag in &self.lint_diagnostics {
            let severity = match diag.severity() {
                Severity::Hint => DiagnosticSeverity::HINT,
                Severity::Warning => DiagnosticSeverity::WARNING,
                Severity::Error => DiagnosticSeverity::ERROR,
            };

            let line_num = diag.line().unwrap_or(LINE_NUMBER_OFFSET);
            result.push(tower_lsp::lsp_types::Diagnostic {
                range: Range {
                    start: Position {
                        line: (line_num.saturating_sub(LINE_NUMBER_OFFSET)) as u32,
                        character: POSITION_ZERO,
                    },
                    end: Position {
                        line: (line_num.saturating_sub(LINE_NUMBER_OFFSET)) as u32,
                        character: DIAGNOSTIC_LINE_END_CHAR,
                    },
                },
                severity: Some(severity),
                code: Some(NumberOrString::String(diag.rule_id().to_string())),
                source: Some("hedl-lint".to_string()),
                message: diag.message().to_string(),
                ..Default::default()
            });
        }

        result
    }

    /// Get all entity IDs for a type.
    pub fn get_entity_ids(&self, type_name: &str) -> Vec<String> {
        self.entities
            .get(type_name)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all type names.
    pub fn get_type_names(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }

    /// Get schema for a type.
    pub fn get_schema(&self, type_name: &str) -> Option<&Vec<String>> {
        self.schemas.get(type_name).map(|(cols, _)| cols)
    }

    /// Check if an entity exists.
    pub fn entity_exists(&self, type_name: Option<&str>, id: &str) -> bool {
        match type_name {
            Some(t) => self.entities.get(t).is_some_and(|m| m.contains_key(id)),
            None => self.entities.values().any(|m| m.contains_key(id)),
        }
    }
}

// --- Helper Functions ---

fn parse_struct_directive(line: &str) -> Option<(String, Vec<String>)> {
    // %STRUCT: TypeName: [col1, col2, col3]
    let rest = line.strip_prefix("%STRUCT")?.trim();
    // Strip leading colon if present (HEDL format uses colons)
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();
    let bracket_start = rest.find('[')?;
    let bracket_end = rest.find(']')?;

    // Type name may have trailing colon, strip it
    let type_name = rest[..bracket_start]
        .trim()
        .trim_end_matches(':')
        .trim()
        .to_string();
    let cols_str = &rest[bracket_start + 1..bracket_end];
    let columns: Vec<String> = cols_str.split(',').map(|s| s.trim().to_string()).collect();

    Some((type_name, columns))
}

fn parse_alias_directive(line: &str) -> Option<(String, String)> {
    // %ALIAS: %short: "long value" or %ALIAS: short = "long value"
    let rest = line.strip_prefix("%ALIAS")?.trim();
    // Strip leading colon if present
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

    // Try the `short = "value"` format first
    if let Some(eq_pos) = rest.find('=') {
        let alias = rest[..eq_pos].trim().trim_start_matches('%').to_string();
        let value = rest[eq_pos + 1..].trim().trim_matches('"').to_string();
        return Some((alias, value));
    }

    // Try the `%short: "value"` format
    if let Some(colon_pos) = rest.rfind(':') {
        let alias = rest[..colon_pos].trim().trim_start_matches('%').to_string();
        let value = rest[colon_pos + 1..].trim().trim_matches('"').to_string();
        return Some((alias, value));
    }

    None
}

fn parse_nest_directive(line: &str) -> Option<(String, String)> {
    // %NEST: Parent > Child
    let rest = line.strip_prefix("%NEST")?.trim();
    // Strip leading colon if present
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();
    let arrow_pos = rest.find('>')?;

    let parent = rest[..arrow_pos].trim().to_string();
    let child = rest[arrow_pos + 1..].trim().to_string();

    Some((parent, child))
}
