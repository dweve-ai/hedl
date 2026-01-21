// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Rule to detect duplicate keys within the same scope.

use crate::validation::traverse::{visit_all_nodes, NodeVisitContext};
use crate::validation::{Diagnostic, Rule, RuleCategory, Severity, ValidationContext};
use crate::{Document, HedlError, Node};
use std::collections::HashMap;

/// Detects duplicate IDs within the same type scope.
pub struct DuplicateKeyRule;

impl Rule for DuplicateKeyRule {
    fn id(&self) -> &str {
        "duplicate-key"
    }

    fn description(&self) -> &str {
        "Detect duplicate IDs within the same type scope"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Structure
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn cost_estimate(&self) -> u8 {
        30
    }

    fn check(
        &self,
        doc: &Document,
        _context: &mut ValidationContext,
    ) -> Result<Vec<Diagnostic>, HedlError> {
        use crate::validation::{DiagnosticKind, RelatedDiagnostic, SourceLocation};

        let mut diagnostics = Vec::new();

        // Track IDs per type: type_name -> map of (id -> first occurrence info)
        let mut seen_ids: HashMap<String, HashMap<String, Option<usize>>> = HashMap::new();

        // Use recursive traversal to check all nodes at any nesting depth
        visit_all_nodes(doc, |node: &Node, ctx: &NodeVisitContext<'_>| {
            let type_name = ctx.type_name;
            let id = &node.id;
            let line: Option<usize> = None; // TODO: Track line numbers

            let type_map = seen_ids.entry(type_name.to_string()).or_default();

            if let Some(first_line) = type_map.get(id.as_str()) {
                // Duplicate found
                let mut diag = Diagnostic::error(
                    DiagnosticKind::DuplicateKey,
                    format!("Duplicate ID '{}' found in type '{}'", id, type_name),
                    self.id(),
                );

                if let Some(l) = line {
                    diag = diag.with_location(SourceLocation::from_line(l));
                }

                // Add related diagnostic showing first occurrence
                if let Some(first_l) = *first_line {
                    diag = diag.with_related(RelatedDiagnostic::new(
                        "First defined here",
                        SourceLocation::from_line(first_l),
                    ));
                }

                diagnostics.push(diag);
            } else {
                // First occurrence - store it
                type_map.insert(id.to_string(), line);
            }
        });

        Ok(diagnostics)
    }
}
