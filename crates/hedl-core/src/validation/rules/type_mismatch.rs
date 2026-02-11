// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Rule to detect type mismatches in fields.

use crate::validation::traverse::visit_all_nodes;
use crate::validation::{Diagnostic, Rule, RuleCategory, Severity, ValidationContext};
use crate::{Document, HedlError, Value};
use std::collections::HashMap;

/// Detects basic type mismatches in matrix list fields.
pub struct TypeMismatchRule;

impl Rule for TypeMismatchRule {
    fn id(&self) -> &str {
        "type-mismatch"
    }

    fn description(&self) -> &str {
        "Detect basic type mismatches in fields"
    }

    fn category(&self) -> RuleCategory {
        RuleCategory::TypeSafety
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn cost_estimate(&self) -> u8 {
        35
    }

    fn check(
        &self,
        doc: &Document,
        _context: &mut ValidationContext,
    ) -> Result<Vec<Diagnostic>, HedlError> {
        use crate::validation::DiagnosticKind;

        let mut diagnostics = Vec::new();

        // Track field types: type_name -> field_index -> value_type
        let mut field_types: HashMap<String, HashMap<usize, String>> = HashMap::new();

        // Helper to get a simple type name for a value
        let get_value_type = |value: &Value| -> String {
            match value {
                Value::Null => "null".to_string(),
                Value::Bool(_) => "bool".to_string(),
                Value::Int(_) => "int".to_string(),
                Value::Float(_) => "float".to_string(),
                Value::String(_) => "string".to_string(),
                Value::Tensor(_) => "tensor".to_string(),
                Value::Reference(_) => "reference".to_string(),
                Value::Expression(_) => "expression".to_string(),
                Value::List(_) => "list".to_string(),
            }
        };

        // Check all nodes using recursive traversal (handles any nesting depth)
        visit_all_nodes(doc, |node, ctx| {
            let type_name = ctx.type_name;
            let type_map = field_types.entry(type_name.to_string()).or_default();

            // Check each field
            for (field_idx, field_value) in node.fields.iter().enumerate() {
                let value_type = get_value_type(field_value);

                if let Some(expected_type) = type_map.get(&field_idx) {
                    // Check if type matches
                    if &value_type != expected_type {
                        let diag = Diagnostic::warning(
                            DiagnosticKind::TypeMismatch,
                            format!(
                                "Field {} in type '{}' has inconsistent type: expected '{}', found '{}'",
                                field_idx, type_name, expected_type, value_type
                            ),
                            self.id(),
                        );
                        diagnostics.push(diag);
                    }
                } else {
                    // First occurrence - record the type
                    type_map.insert(field_idx, value_type);
                }
            }
        });

        Ok(diagnostics)
    }
}
