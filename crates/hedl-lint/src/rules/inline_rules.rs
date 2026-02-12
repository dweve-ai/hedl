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

//! Inline child list lint rules

use super::common::{LintRule, MAX_RECURSION_DEPTH};
use crate::diagnostic::Diagnostic;
use hedl_core::{Document, Item, Node};
use std::collections::BTreeMap;

/// Rule: Inline child list exceeds maximum
pub struct InlineChildExceedsMaxRule;

impl LintRule for InlineChildExceedsMaxRule {
    fn id(&self) -> &'static str {
        "inline-child-exceeds-max"
    }

    fn description(&self) -> &'static str {
        "Check that inline child lists do not exceed recommended maximum of 10 entries (SPEC v2.0 style rule)"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_inline_exceeds_max(&doc.root, &mut diagnostics, 0);
        diagnostics
    }
}

/// Check inline child counts exceed maximum with depth protection.
fn check_inline_exceeds_max(
    items: &BTreeMap<String, Item>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        return;
    }

    for item in items.values() {
        match item {
            Item::List(list) => {
                for row in &list.rows {
                    check_node_exceeds_max(row, diagnostics, depth + 1);
                }
            }
            Item::Object(child) => {
                check_inline_exceeds_max(child, diagnostics, depth + 1);
            }
            _ => {}
        }
    }
}

fn check_node_exceeds_max(node: &Node, diagnostics: &mut Vec<Diagnostic>, depth: usize) {
    use crate::diagnostic::DiagnosticKind;

    if depth > MAX_RECURSION_DEPTH {
        return;
    }

    // Only check nodes with count hints
    if let Some(hint_count) = node.get_child_count() {
        let actual_count = node
            .children()
            .map(|c| c.values().map(|v| v.len()).sum())
            .unwrap_or(0);

        // Only report exceeds-max if count actually matches (avoids duplicate with mismatch rule)
        // SPEC v2.0 line 58: "Style rule (not a hard syntax limit): keep inline N <= 10"
        if actual_count == hint_count && hint_count > 10 {
            diagnostics.push(Diagnostic::warning(
                DiagnosticKind::InlineChildExceedsMax,
                format!(
                    "inline child list for '{}' has {} entries, recommended maximum is 10 (use expanded format for readability)",
                    node.id, hint_count
                ),
                "inline-child-exceeds-max",
            ));
        }
    }

    // Recursively check nested children
    if let Some(children) = node.children() {
        for child_nodes in children.values() {
            for child_node in child_nodes {
                check_node_exceeds_max(child_node, diagnostics, depth + 1);
            }
        }
    }
}

/// Rule: Inline count mismatch
pub struct InlineCountMismatchRule;

impl LintRule for InlineCountMismatchRule {
    fn id(&self) -> &'static str {
        "inline-count-mismatch"
    }

    fn description(&self) -> &'static str {
        "Check that inline child count hints match actual child count"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_inline_count_mismatch(&doc.root, &mut diagnostics, 0);
        diagnostics
    }
}

/// Check inline child count mismatches with depth protection.
fn check_inline_count_mismatch(
    items: &BTreeMap<String, Item>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        return;
    }

    for item in items.values() {
        match item {
            Item::List(list) => {
                for row in &list.rows {
                    check_node_count_mismatch(row, diagnostics, depth + 1);
                }
            }
            Item::Object(child) => {
                check_inline_count_mismatch(child, diagnostics, depth + 1);
            }
            _ => {}
        }
    }
}

fn check_node_count_mismatch(node: &Node, diagnostics: &mut Vec<Diagnostic>, depth: usize) {
    use crate::diagnostic::DiagnosticKind;

    if depth > MAX_RECURSION_DEPTH {
        return;
    }

    // Only check nodes with count hints
    if let Some(hint_count) = node.get_child_count() {
        let actual_count = node
            .children()
            .map(|c| c.values().map(|v| v.len()).sum())
            .unwrap_or(0);

        if actual_count != hint_count {
            diagnostics.push(Diagnostic::error(
                DiagnosticKind::InlineCountMismatch,
                format!(
                    "inline child count for '{}' declares {} but has {} children",
                    node.id, hint_count, actual_count
                ),
                "inline-count-mismatch",
            ));
        }
    }

    // Recursively check nested children
    if let Some(children) = node.children() {
        for child_nodes in children.values() {
            for child_node in child_nodes {
                check_node_count_mismatch(child_node, diagnostics, depth + 1);
            }
        }
    }
}

/// Rule: Missing count hint on inline-style children
pub struct MissingCountHintRule;

impl LintRule for MissingCountHintRule {
    fn id(&self) -> &'static str {
        "missing-count-hint"
    }

    fn description(&self) -> &'static str {
        "Suggest count hints for children that could use inline syntax"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_missing_count_hint(&doc.root, &mut diagnostics, 0);
        diagnostics
    }
}

/// Check for missing count hints with depth protection.
fn check_missing_count_hint(
    items: &BTreeMap<String, Item>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        return;
    }

    for item in items.values() {
        match item {
            Item::List(list) => {
                for row in &list.rows {
                    check_node_missing_hint(row, diagnostics, depth + 1);
                }
            }
            Item::Object(child) => {
                check_missing_count_hint(child, diagnostics, depth + 1);
            }
            _ => {}
        }
    }
}

fn check_node_missing_hint(node: &Node, diagnostics: &mut Vec<Diagnostic>, depth: usize) {
    use crate::diagnostic::DiagnosticKind;

    if depth > MAX_RECURSION_DEPTH {
        return;
    }

    // Only suggest hints for nodes WITHOUT count hints
    if node.get_child_count().is_none() {
        if let Some(children) = node.children() {
            // Suggest count hint for children that could use inline syntax
            // Only suggest for multiple children of the same type, within the 2-5 range
            for (child_type, child_nodes) in children {
                if child_nodes.len() >= 2 && child_nodes.len() <= 5 {
                    diagnostics.push(Diagnostic::hint(
                        DiagnosticKind::MissingCountHint,
                        format!(
                            "node '{}' has {} '{}' children but no count hint, consider using inline syntax with #{}",
                            node.id, child_nodes.len(), child_type, child_nodes.len()
                        ),
                        "missing-count-hint",
                    ));
                }
            }
        }
    }

    // Recursively check nested children
    if let Some(children) = node.children() {
        for child_nodes in children.values() {
            for child_node in child_nodes {
                check_node_missing_hint(child_node, diagnostics, depth + 1);
            }
        }
    }
}
