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

//! Naming convention lint rules

use super::common::{LintRule, MAX_RECURSION_DEPTH};
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use hedl_core::{Document, Item, Node};
use std::collections::BTreeMap;

/// Rule: ID naming conventions
pub struct IdNamingRule;

impl LintRule for IdNamingRule {
    fn id(&self) -> &'static str {
        "id-naming"
    }
    fn description(&self) -> &'static str {
        "Check ID naming conventions (lowercase, descriptive)"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_item_ids(&doc.root, &mut diagnostics);
        diagnostics
    }
}

/// Check all item IDs in a document tree with depth protection.
///
/// # Security
///
/// This function implements recursive depth limiting to prevent stack overflow
/// from maliciously crafted deeply nested documents. If the depth limit is
/// exceeded, further traversal is halted and a warning diagnostic is generated.
fn check_item_ids(items: &BTreeMap<String, Item>, diagnostics: &mut Vec<Diagnostic>) {
    check_item_ids_bounded(items, diagnostics, 0);
}

fn check_item_ids_bounded(
    items: &BTreeMap<String, Item>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        diagnostics.push(Diagnostic::warning(
            DiagnosticKind::Custom("max-depth-exceeded".to_string()),
            format!(
                "Maximum nesting depth of {MAX_RECURSION_DEPTH} exceeded during ID checking. \
                 Further nested items will not be checked."
            ),
            "id-naming",
        ));
        return;
    }

    for item in items.values() {
        match item {
            Item::List(list) => {
                for row in &list.rows {
                    check_node_id(&row.id, diagnostics);
                    if let Some(children) = row.children() {
                        check_node_children_bounded(children, diagnostics, depth + 1);
                    }
                }
            }
            Item::Object(child) => {
                check_item_ids_bounded(child, diagnostics, depth + 1);
            }
            _ => {}
        }
    }
}

fn check_node_id(id: &str, diagnostics: &mut Vec<Diagnostic>) {
    // Check for non-descriptive IDs
    if id.len() == 1 {
        diagnostics.push(Diagnostic::hint(
            DiagnosticKind::IdNaming,
            format!("ID '{id}' is very short, consider a more descriptive name"),
            "id-naming",
        ));
    }
    // Check for numeric-only IDs (must have at least one digit, not just underscores)
    let has_digit = id.chars().any(|c| c.is_ascii_digit());
    let all_numeric_or_underscore = id.chars().all(|c| c.is_ascii_digit() || c == '_');
    if has_digit && all_numeric_or_underscore {
        diagnostics.push(Diagnostic::hint(
            DiagnosticKind::IdNaming,
            format!("ID '{id}' contains only numbers, consider adding descriptive prefix"),
            "id-naming",
        ));
    }
}

/// Check node children with recursion depth protection.
///
/// # Security
///
/// This function enforces a maximum recursion depth to prevent stack overflow
/// vulnerabilities. Deeply nested structures are common attack vectors for
/// causing denial-of-service through stack exhaustion.
fn check_node_children_bounded(
    children: &BTreeMap<String, Vec<Node>>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        diagnostics.push(Diagnostic::warning(
            DiagnosticKind::Custom("max-depth-exceeded".to_string()),
            format!(
                "Maximum nesting depth of {MAX_RECURSION_DEPTH} exceeded. \
                 Further nested nodes will not be checked."
            ),
            "id-naming",
        ));
        return;
    }

    for nodes in children.values() {
        for node in nodes {
            check_node_id(&node.id, diagnostics);
            if let Some(node_children) = node.children() {
                check_node_children_bounded(node_children, diagnostics, depth + 1);
            }
        }
    }
}
