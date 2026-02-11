// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Rule to detect unused nodes (declared but never referenced).

use crate::validation::traverse::{visit_all_nodes, visit_all_references};
use crate::validation::{Diagnostic, Rule, RuleCategory, Severity, ValidationContext};
use crate::{Document, HedlError};
use std::collections::{HashMap, HashSet};

/// Detects nodes that are declared but never referenced.
pub struct UnusedReferenceRule;

impl Rule for UnusedReferenceRule {
    fn id(&self) -> &str {
        "unused-reference"
    }

    fn description(&self) -> &str {
        "Detect nodes that are declared but never referenced"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::Style
    }

    fn default_severity(&self) -> Severity {
        Severity::Hint
    }

    fn cost_estimate(&self) -> u8 {
        45
    }

    fn check(
        &self,
        doc: &Document,
        _context: &mut ValidationContext,
    ) -> Result<Vec<Diagnostic>, HedlError> {
        use crate::validation::DiagnosticKind;

        let mut diagnostics = Vec::new();

        // Track all declared node IDs: (type_name, id) pairs
        let mut declared_ids: HashMap<String, HashSet<String>> = HashMap::new();

        // Track all referenced (type_name, id) pairs for qualified refs
        // and unqualified IDs separately
        let mut qualified_refs: HashSet<(String, String)> = HashSet::new();
        let mut unqualified_refs: HashSet<String> = HashSet::new();

        // First pass: collect all declared nodes using recursive traversal
        visit_all_nodes(doc, |node, ctx| {
            declared_ids
                .entry(ctx.type_name.to_string())
                .or_default()
                .insert(node.id.clone());
        });

        // Second pass: collect all referenced IDs using recursive traversal
        visit_all_references(doc, |r, _ctx| {
            if let Some(type_name) = &r.type_name {
                // Qualified reference:@Type:id
                qualified_refs.insert((type_name.to_string(), r.id.to_string()));
            } else {
                // Unqualified reference:@id
                unqualified_refs.insert(r.id.to_string());
            }
        });

        // Find unused nodes (declared but never referenced)
        for (type_name, ids) in &declared_ids {
            for id in ids {
                // Check if referenced by qualified or unqualified ref
                let is_qualified_ref = qualified_refs.contains(&(type_name.clone(), id.clone()));
                let is_unqualified_ref = unqualified_refs.contains(id);

                if !is_qualified_ref && !is_unqualified_ref {
                    let diag = Diagnostic::hint(
                        DiagnosticKind::UnusedReference,
                        format!("Node '{}' of type '{}' is never referenced", id, type_name),
                        self.id(),
                    );
                    diagnostics.push(diag);
                }
            }
        }

        Ok(diagnostics)
    }
}
