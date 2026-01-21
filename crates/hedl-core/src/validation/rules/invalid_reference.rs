// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Rule to detect invalid references (dangling pointers).

use crate::validation::traverse::{visit_all_nodes, visit_all_references};
use crate::validation::{Diagnostic, Rule, RuleCategory, Severity, ValidationContext};
use crate::{Document, HedlError, Reference};

/// Detects references that point to non-existent nodes.
pub struct InvalidReferenceRule;

impl Rule for InvalidReferenceRule {
    fn id(&self) -> &str {
        "invalid-reference"
    }

    fn description(&self) -> &str {
        "Detect references to non-existent nodes"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::References
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn cost_estimate(&self) -> u8 {
        40
    }

    fn before_document(&self, _context: &mut ValidationContext) -> Result<(), HedlError> {
        // Symbol table will be populated during check
        Ok(())
    }

    fn check(
        &self,
        doc: &Document,
        context: &mut ValidationContext,
    ) -> Result<Vec<Diagnostic>, HedlError> {
        use crate::validation::DiagnosticKind;

        let mut diagnostics = Vec::new();

        // First pass: build symbol table using recursive traversal
        visit_all_nodes(doc, |node, ctx| {
            context.register_node(ctx.type_name, node);
        });

        // Second pass: collect all references using recursive traversal
        let mut all_refs: Vec<Reference> = Vec::new();
        visit_all_references(doc, |r, _ctx| {
            all_refs.push(r.clone());
        });

        // Validate all references
        for reference in &all_refs {
            if context.resolve_reference(reference).is_none() {
                let message = if let Some(type_name) = &reference.type_name {
                    format!(
                        "Reference '{}' points to non-existent '{}' with ID '{}'",
                        reference.to_ref_string(),
                        type_name,
                        reference.id
                    )
                } else {
                    format!(
                        "Reference '{}' points to non-existent ID",
                        reference.to_ref_string()
                    )
                };

                let diag = Diagnostic::error(DiagnosticKind::InvalidReference, message, self.id());
                diagnostics.push(diag);
            }
        }

        Ok(diagnostics)
    }
}
