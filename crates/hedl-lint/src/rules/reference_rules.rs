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

//! Reference lint rules

use super::common::{LintRule, MAX_RECURSION_DEPTH};
use crate::diagnostic::Diagnostic;
use hedl_core::{Document, Item};
use std::collections::BTreeMap;

/// Rule: Unqualified references in Key-Value context
pub struct UnqualifiedKvReferenceRule;

impl LintRule for UnqualifiedKvReferenceRule {
    fn id(&self) -> &'static str {
        "unqualified-kv-ref"
    }
    fn description(&self) -> &'static str {
        "Warn about unqualified references in Key-Value context"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        check_kv_references(&doc.root, &mut diagnostics);
        diagnostics
    }
}

/// Check for unqualified references in Key-Value context with depth protection.
///
/// # Security
///
/// Implements recursion depth limiting to prevent stack overflow from
/// deeply nested document structures during reference checking.
fn check_kv_references(items: &BTreeMap<String, Item>, diagnostics: &mut Vec<Diagnostic>) {
    check_kv_references_bounded(items, diagnostics, 0);
}

fn check_kv_references_bounded(
    items: &BTreeMap<String, Item>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    use crate::diagnostic::DiagnosticKind;
    use hedl_core::Value;

    if depth > MAX_RECURSION_DEPTH {
        diagnostics.push(Diagnostic::warning(
            DiagnosticKind::Custom("max-depth-exceeded".to_string()),
            format!(
                "Maximum nesting depth of {MAX_RECURSION_DEPTH} exceeded during reference checking. \
                 Further nested items will not be checked."
            ),
            "unqualified-kv-ref",
        ));
        return;
    }

    for item in items.values() {
        match item {
            Item::Scalar(Value::Reference(r)) => {
                if r.type_name.is_none() {
                    diagnostics.push(
                        Diagnostic::warning(
                            DiagnosticKind::UnqualifiedKvReference,
                            format!("Unqualified reference '@{}' in Key-Value context, consider using qualified form '@Type:{}'", r.id, r.id),
                            "unqualified-kv-ref"
                        ).with_suggestion(format!("Use @Type:{}", r.id))
                    );
                }
            }
            Item::Object(child) => {
                check_kv_references_bounded(child, diagnostics, depth + 1);
            }
            _ => {}
        }
    }
}
