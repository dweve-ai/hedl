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

//! Schema and structure lint rules

use super::common::{LintRule, MAX_RECURSION_DEPTH};
use crate::diagnostic::Diagnostic;
use hedl_core::{Document, Item, Node};
use std::collections::BTreeMap;

/// Rule: Unused schemas
pub struct UnusedSchemaRule;

impl LintRule for UnusedSchemaRule {
    fn id(&self) -> &'static str {
        "unused-schema"
    }
    fn description(&self) -> &'static str {
        "Check for unused %STRUCT definitions"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        use crate::diagnostic::DiagnosticKind;
        use std::collections::HashSet;

        let mut used_types = HashSet::new();
        collect_used_types(&doc.root, &mut used_types);

        let mut diagnostics = Vec::new();
        for type_name in doc.structs.keys() {
            if !used_types.contains(type_name.as_str()) {
                diagnostics.push(Diagnostic::warning(
                    DiagnosticKind::UnusedSchema,
                    format!("Schema '{type_name}' is defined but never used"),
                    "unused-schema",
                ));
            }
        }

        diagnostics
    }
}

/// Collect used types with depth protection.
///
/// # Security
///
/// Implements recursion depth limiting to prevent stack overflow from
/// deeply nested document structures during type collection.
fn collect_used_types<'a>(
    items: &'a BTreeMap<String, Item>,
    used: &mut std::collections::HashSet<&'a str>,
) {
    collect_used_types_bounded(items, used, 0);
}

fn collect_used_types_bounded<'a>(
    items: &'a BTreeMap<String, Item>,
    used: &mut std::collections::HashSet<&'a str>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        // Silently stop traversal at max depth for type collection
        // This prevents stack overflow while still collecting types from
        // non-malicious portions of the document
        return;
    }

    for item in items.values() {
        match item {
            Item::List(list) => {
                used.insert(&list.type_name);
                for row in &list.rows {
                    collect_node_types(row, used, depth + 1);
                }
            }
            Item::Object(child) => {
                collect_used_types_bounded(child, used, depth + 1);
            }
            _ => {}
        }
    }
}

/// Recursively collect all type names from a node and its children.
///
/// Collects types from:
/// 1. Child node type names (from nested relationships)
/// 2. Reference values in node fields (qualified references like @Type:id)
///
/// # Security
///
/// Implements recursion depth limiting to prevent stack overflow from
/// deeply nested document structures during type collection.
fn collect_node_types<'a>(
    node: &'a Node,
    used: &mut std::collections::HashSet<&'a str>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        // Silently stop traversal at max depth for type collection
        return;
    }

    // Collect type names from reference values in fields
    for field in &node.fields {
        if let hedl_core::Value::Reference(ref r) = field {
            if let Some(ref type_name) = r.type_name {
                used.insert(type_name);
            }
        }
    }

    // Collect type names from nested children
    if let Some(children) = node.children() {
        for (child_type, child_nodes) in children {
            used.insert(child_type);
            // Recursively traverse nested children
            for child_node in child_nodes {
                collect_node_types(child_node, used, depth + 1);
            }
        }
    }
}

/// Rule: Empty matrix lists
pub struct EmptyListRule;

impl LintRule for EmptyListRule {
    fn id(&self) -> &'static str {
        "empty-list"
    }
    fn description(&self) -> &'static str {
        "Warn about empty matrix lists"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_empty_lists(&doc.root, &mut diagnostics);
        diagnostics
    }
}

/// Check for empty lists with depth protection.
///
/// # Security
///
/// Implements recursion depth limiting to prevent stack overflow from
/// deeply nested document structures during empty list detection.
fn check_empty_lists(items: &BTreeMap<String, Item>, diagnostics: &mut Vec<Diagnostic>) {
    check_empty_lists_bounded(items, diagnostics, 0);
}

fn check_empty_lists_bounded(
    items: &BTreeMap<String, Item>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    use crate::diagnostic::DiagnosticKind;

    if depth > MAX_RECURSION_DEPTH {
        diagnostics.push(Diagnostic::warning(
            DiagnosticKind::Custom("max-depth-exceeded".to_string()),
            format!(
                "Maximum nesting depth of {MAX_RECURSION_DEPTH} exceeded during empty list checking. \
                 Further nested items will not be checked."
            ),
            "empty-list",
        ));
        return;
    }

    for (key, item) in items {
        match item {
            Item::List(list) => {
                if list.rows.is_empty() {
                    diagnostics.push(Diagnostic::hint(
                        DiagnosticKind::EmptyList,
                        format!("Matrix list '{key}' is empty"),
                        "empty-list",
                    ));
                }
            }
            Item::Object(child) => {
                check_empty_lists_bounded(child, diagnostics, depth + 1);
            }
            _ => {}
        }
    }
}
