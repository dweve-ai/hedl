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

//! Standard and streaming batch operations.

use super::traits::{BatchOperation, StreamingBatchOperation};
use crate::error::CliError;
use std::collections::HashSet;
use std::path::Path;

// ============================================================================
// Standard Operations
// ============================================================================

/// Batch validation operation.
///
/// Validates multiple HEDL files in parallel, checking syntax and optionally
/// enforcing strict reference resolution.
#[derive(Debug, Clone)]
pub struct ValidationOperation {
    /// Enable strict reference validation
    pub strict: bool,
}

impl BatchOperation for ValidationOperation {
    type Output = ValidationStats;

    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
        use hedl_core::{parse_with_limits, Item, Node, ParseOptions, ReferenceMode};

        let content = std::fs::read_to_string(path).map_err(|e| CliError::io_error(path, e))?;

        let options = ParseOptions {
            reference_mode: if self.strict {
                ReferenceMode::Strict
            } else {
                ReferenceMode::Lenient
            },
            ..ParseOptions::default()
        };

        let doc = parse_with_limits(content.as_bytes(), options)
            .map_err(|e| CliError::parse(e.to_string()))?;

        // Collect statistics from the parsed document
        let mut stats = ValidationStats::new();

        // Get version from document metadata
        stats.version = format!("{}.{}", doc.version.0, doc.version.1);

        // Recursive helper to count nodes
        fn count_node(node: &Node, stats: &mut ValidationStats) {
            stats.node_count += 1;
            stats.field_count += node.fields.len();
            let full_id = format!("{}:{}", node.type_name, node.id);
            stats.seen_ids.insert(full_id);

            // Count children recursively
            if let Some(ref children) = node.children {
                for child_nodes in children.values() {
                    for child in child_nodes {
                        count_node(child, stats);
                    }
                }
            }
        }

        // Recursive helper to traverse items
        fn traverse_item(item: &Item, stats: &mut ValidationStats) {
            match item {
                Item::List(list) => {
                    stats.list_count += 1;
                    for node in &list.rows {
                        count_node(node, stats);
                    }
                }
                Item::Object(obj) => {
                    for child_item in obj.values() {
                        traverse_item(child_item, stats);
                    }
                }
                Item::Scalar(_) => {
                    // Scalars don't contribute to node counts
                }
            }
        }

        // Traverse all items in the document root
        for item in doc.root.values() {
            traverse_item(item, &mut stats);
        }

        Ok(stats)
    }

    fn name(&self) -> &'static str {
        "validate"
    }
}

/// Batch format operation.
///
/// Formats multiple HEDL files to canonical form, optionally checking if files
/// are already canonical.
#[derive(Debug, Clone)]
pub struct FormatOperation {
    /// Only check if files are canonical (don't write)
    pub check: bool,
    /// Use ditto optimization
    pub ditto: bool,
    /// Add count hints to matrix lists
    pub with_counts: bool,
}

impl BatchOperation for FormatOperation {
    type Output = String;

    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
        use hedl_c14n::{canonicalize_with_config, CanonicalConfig};
        use hedl_core::parse;

        let content = std::fs::read_to_string(path).map_err(|e| CliError::io_error(path, e))?;

        let mut doc = parse(content.as_bytes()).map_err(|e| CliError::parse(e.to_string()))?;

        // Add count hints if requested
        if self.with_counts {
            add_count_hints(&mut doc);
        }

        let config = CanonicalConfig::new().with_ditto(self.ditto);

        let canonical = canonicalize_with_config(&doc, &config)
            .map_err(|e| CliError::canonicalization(e.to_string()))?;

        if self.check && canonical != content {
            return Err(CliError::NotCanonical);
        }

        Ok(canonical)
    }

    fn name(&self) -> &str {
        if self.check {
            "format-check"
        } else {
            "format"
        }
    }
}

/// Batch lint operation.
///
/// Lints multiple HEDL files for best practices and common issues.
#[derive(Debug, Clone)]
pub struct LintOperation {
    /// Treat warnings as errors
    pub warn_error: bool,
}

impl BatchOperation for LintOperation {
    type Output = Vec<String>;

    fn process_file(&self, path: &Path) -> Result<Self::Output, CliError> {
        use hedl_core::parse;
        use hedl_lint::lint;

        let content = std::fs::read_to_string(path).map_err(|e| CliError::io_error(path, e))?;

        let doc = parse(content.as_bytes()).map_err(|e| CliError::parse(e.to_string()))?;

        let diagnostics = lint(&doc);

        if self.warn_error && !diagnostics.is_empty() {
            return Err(CliError::LintErrors);
        }

        Ok(diagnostics
            .iter()
            .map(std::string::ToString::to_string)
            .collect())
    }

    fn name(&self) -> &'static str {
        "lint"
    }
}

// ============================================================================
// Streaming Operations
// ============================================================================

/// Statistics collected during streaming validation.
///
/// Provides detailed statistics about the parsed document including
/// entity counts, field counts, and ID tracking for reference validation.
#[derive(Debug, Clone, Default)]
pub struct ValidationStats {
    /// HEDL version string
    pub version: String,
    /// Number of lists encountered
    pub list_count: usize,
    /// Total number of nodes processed
    pub node_count: usize,
    /// Total number of fields across all nodes
    pub field_count: usize,
    /// Set of seen IDs for strict reference validation (type:id format)
    pub seen_ids: HashSet<String>,
}

impl ValidationStats {
    /// Create new empty validation statistics
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Streaming validation operation for memory-efficient validation of large files.
///
/// Uses the streaming parser from `hedl-stream` to validate files with O(1) memory
/// usage regardless of file size. Ideal for:
/// - Files larger than 100MB
/// - Validating thousands of files with limited RAM
/// - Container environments with memory limits
///
/// # Memory Profile
///
/// - **Input**: O(1) - buffer size only (~8KB)
/// - **Working**: `O(n_ids)` - seen ID set for strict validation
/// - **Output**: O(1) - small statistics struct
/// - **Peak**: ~8KB + ID set size (vs. full file size in standard mode)
///
/// # Examples
///
/// ```rust,no_run
/// use hedl_cli::batch::{BatchExecutor, StreamingValidationOperation};
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let processor = BatchExecutor::default_config();
/// let files = vec![PathBuf::from("large-file.hedl")];
///
/// let operation = StreamingValidationOperation { strict: false };
/// let results = processor.process_streaming(&files, operation, true)?;
///
/// println!("Validated {} files with constant memory", results.success_count());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct StreamingValidationOperation {
    /// Enable strict reference validation
    pub strict: bool,
}

impl StreamingBatchOperation for StreamingValidationOperation {
    type Output = ValidationStats;

    fn process_file_streaming(&self, path: &Path) -> Result<Self::Output, CliError> {
        use hedl_stream::{NodeEvent, StreamError, StreamingParser};
        use std::fs::File;
        use std::io::BufReader;

        let file = File::open(path).map_err(|e| CliError::io_error(path, e))?;
        let reader = BufReader::with_capacity(8192, file);

        let parser = StreamingParser::new(reader)
            .map_err(|e: StreamError| CliError::parse(e.to_string()))?;

        let mut stats = ValidationStats::new();
        let mut _current_type = String::new();

        // Process events incrementally
        for event in parser {
            let event = event.map_err(|e: StreamError| CliError::parse(e.to_string()))?;

            match event {
                NodeEvent::Header(info) => {
                    // Validate version exists
                    let version_str = format!("{}.{}", info.version.0, info.version.1);
                    if version_str.is_empty() {
                        return Err(CliError::parse("Missing VERSION".to_string()));
                    }
                    stats.version = version_str;
                }
                NodeEvent::ListStart { type_name, .. } => {
                    stats.list_count += 1;
                    _current_type = type_name;
                }
                NodeEvent::Node(node) => {
                    stats.node_count += 1;
                    stats.field_count += node.fields.len();

                    // Track IDs for strict mode validation
                    let full_id = format!("{}:{}", node.type_name, node.id);

                    if self.strict {
                        // In strict mode, validate references
                        // For now, just track IDs - full reference validation
                        // would require accumulating references and validating at end
                        stats.seen_ids.insert(full_id);
                    } else {
                        stats.seen_ids.insert(full_id);
                    }
                }
                NodeEvent::ListEnd { .. } => {
                    // List validation complete
                }
                NodeEvent::Scalar { .. } => {
                    // Scalar validation - no action needed
                }
                NodeEvent::ObjectStart { .. } => {
                    // Object start - no action needed
                }
                NodeEvent::ObjectEnd { .. } => {
                    // Object end - no action needed
                }
                NodeEvent::EndOfDocument => {
                    // Document complete
                    break;
                }
            }
        }

        Ok(stats)
    }

    fn name(&self) -> &'static str {
        "validate-streaming"
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

// ============================================================================
// Helper Functions for Count Hints
// ============================================================================

/// Recursively add count hints to all matrix lists in the document
fn add_count_hints(doc: &mut hedl_core::Document) {
    for item in doc.root.values_mut() {
        add_count_hints_to_item(item);
    }
}

/// Recursively add count hints to an item
fn add_count_hints_to_item(item: &mut hedl_core::Item) {
    use hedl_core::Item;

    match item {
        Item::List(list) => {
            // Set count hint based on actual row count
            list.count_hint = Some(list.rows.len());

            // Recursively add child counts to each node
            for node in &mut list.rows {
                add_child_count_to_node(node);
            }
        }
        Item::Object(map) => {
            // Recursively process nested objects
            for nested_item in map.values_mut() {
                add_count_hints_to_item(nested_item);
            }
        }
        Item::Scalar(_) => {
            // Scalars don't have matrix lists
        }
    }
}

/// Recursively set `child_count` on nodes that have children
fn add_child_count_to_node(node: &mut hedl_core::Node) {
    // Calculate total number of direct children across all child types
    let total_children: usize = node
        .children()
        .map_or(0, |c| c.values().map(std::vec::Vec::len).sum());

    if total_children > 0 {
        node.child_count = total_children.min(u16::MAX as usize) as u16;

        // Recursively process all child nodes
        if let Some(children) = node.children_mut() {
            for child_list in children.values_mut() {
                for child_node in child_list {
                    add_child_count_to_node(child_node);
                }
            }
        }
    }
}
