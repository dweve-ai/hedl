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

//! HEDL v2.0 specific lint rules

use super::common::LintRule;
use crate::diagnostic::Diagnostic;
use hedl_core::Document;
use std::any::Any;

/// Rule that forbids ditto operator (^) in v2.0+ files.
///
/// Note: This rule checks source text for ditto operators. The parser already
/// rejects ditto operators in v2.0+ documents, but this rule provides early
/// detection and clear diagnostic messages when source text is available.
pub struct ForbidDittoRule;

impl LintRule for ForbidDittoRule {
    fn id(&self) -> &'static str {
        "forbid-ditto"
    }

    fn description(&self) -> &'static str {
        "Ditto operator (^) is forbidden in HEDL v2.0"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        // Without source context, we cannot check for ditto operators in the raw text.
        // The parser already enforces this at parse time for v2.0+.
        if doc.version >= (2, 0) {
            // Parser enforcement is sufficient when no source context is available
        }
        Vec::new()
    }

    fn check_with_context(&self, doc: &Document, context: &dyn Any) -> Vec<Diagnostic> {
        use crate::diagnostic::DiagnosticKind;

        // Only check v2.0+ documents
        if doc.version < (2, 0) {
            return Vec::new();
        }

        // Try to cast context to LintContext
        let Some(lint_context) = context.downcast_ref::<crate::runner::LintContext>() else {
            return Vec::new();
        };

        let mut diagnostics = Vec::new();
        let source_text = &lint_context.source_text;

        // Check each line for ditto operator (^) in the body section
        let mut in_body = false;
        for (line_num, line) in source_text.lines().enumerate() {
            // Skip until we reach the separator
            if line.starts_with("---") {
                in_body = true;
                continue;
            }

            if !in_body {
                continue;
            }

            // Look for ditto operator in matrix list rows
            // Ditto can appear in various positions:
            // - After pipe: |^
            // - After comma in values: |id,^
            // - As a standalone field: ,^,
            if line.contains('|') {
                // Split by comma to check each field
                if let Some(pipe_pos) = line.find('|') {
                    let after_pipe = &line[pipe_pos + 1..];

                    // Check if any field is exactly "^" (with optional whitespace)
                    for field in after_pipe.split(',') {
                        if field.trim() == "^" {
                            diagnostics.push(
                                Diagnostic::error(
                                    DiagnosticKind::ForbidDitto,
                                    "Ditto operator (^) is not allowed in HEDL v2.0. Repeat the actual value instead.".to_string(),
                                    "forbid-ditto",
                                )
                                .with_line(line_num + 1), // 1-indexed line numbers
                            );
                            break; // Only report once per line
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

/// Rule that validates required headers are present in v2.0+ files.
///
/// HEDL v2.0 requires three headers:
/// - %V:2.0 (version)
/// - %NULL:~ (null sentinel, can be customized)
/// - %QUOTE:" (quote character, can be customized)
pub struct RequiredHeadersRule;

impl LintRule for RequiredHeadersRule {
    fn id(&self) -> &'static str {
        "required-headers"
    }

    fn description(&self) -> &'static str {
        "v2.0 requires %V, %NULL, and %QUOTE headers"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        // Without source context, we cannot check for missing headers in the raw text.
        // The parser ensures these are present (with defaults if missing) by the time
        // we have a Document object.
        if doc.version >= (2, 0) {
            // Parser provides defaults when headers are missing, so we can't detect
            // omissions without source text.
        }
        Vec::new()
    }

    fn check_with_context(&self, doc: &Document, context: &dyn Any) -> Vec<Diagnostic> {
        use crate::diagnostic::DiagnosticKind;

        // Only check v2.0+ documents
        if doc.version < (2, 0) {
            return Vec::new();
        }

        // Try to cast context to LintContext
        let Some(lint_context) = context.downcast_ref::<crate::runner::LintContext>() else {
            return Vec::new();
        };

        let mut diagnostics = Vec::new();
        let source_text = &lint_context.source_text;

        // Skip check if source text is empty (no source to analyze)
        if source_text.is_empty() {
            return Vec::new();
        }

        // Track which required headers are present
        let mut has_null = false;
        let mut has_quote = false;

        // Check each line in the header section
        for line in source_text.lines() {
            // Stop at separator
            if line.starts_with("---") {
                break;
            }

            let trimmed = line.trim();

            // Check for %NULL directive
            if trimmed.starts_with("%NULL:") || trimmed.starts_with("%NULL :") {
                has_null = true;
            }

            // Check for %QUOTE directive
            if trimmed.starts_with("%QUOTE:") || trimmed.starts_with("%QUOTE :") {
                has_quote = true;
            }
        }

        // Report missing headers
        if !has_null {
            diagnostics.push(
                Diagnostic::error(
                    DiagnosticKind::RequiredHeaders,
                    "Missing required header %NULL in HEDL v2.0 document. Add: %NULL:~".to_string(),
                    "required-headers",
                )
                .with_suggestion("Add %NULL:~ header before ---"),
            );
        }

        if !has_quote {
            diagnostics.push(
                Diagnostic::error(
                    DiagnosticKind::RequiredHeaders,
                    "Missing required header %QUOTE in HEDL v2.0 document. Add: %QUOTE:\""
                        .to_string(),
                    "required-headers",
                )
                .with_suggestion("Add %QUOTE:\" header before ---"),
            );
        }

        diagnostics
    }
}

/// Rule that detects unnecessary spaces after pipe characters in inline child lists.
///
/// Inline children use a compact format: `@TypeName#N:|data1|data2|data3`.
/// A space after the pipe (`@TypeName#N:|data1`) wastes tokens and should be removed.
///
/// This rule requires source text context to detect formatting issues.
pub struct SpaceAfterPipeRule;

impl LintRule for SpaceAfterPipeRule {
    fn id(&self) -> &'static str {
        "space-after-pipe"
    }

    fn description(&self) -> &'static str {
        "Unnecessary space after pipe character in matrix row"
    }

    fn check(&self, _doc: &Document) -> Vec<Diagnostic> {
        // This rule requires source text context to detect formatting issues.
        // Without context, we cannot detect spaces after pipes.
        Vec::new()
    }

    fn check_with_context(&self, _doc: &Document, context: &dyn Any) -> Vec<Diagnostic> {
        use crate::diagnostic::DiagnosticKind;

        // Try to cast context to LintContext
        let Some(lint_context) = context.downcast_ref::<crate::runner::LintContext>() else {
            return Vec::new();
        };

        let mut diagnostics = Vec::new();
        let source_text = &lint_context.source_text;

        // Parse each line looking for inline child declarations
        for (line_num, line) in source_text.lines().enumerate() {
            let trimmed = line.trim_start();

            // Check for inline child list pattern:@TypeName#N:|data
            if !trimmed.starts_with('@') {
                continue;
            }

            // Look for the pattern @TypeName#N:|with space after pipe
            if let Some(colon_pos) = trimmed.find(':') {
                let after_colon = &trimmed[colon_pos + 1..];

                // Check if it starts with pipe followed by space
                if after_colon.starts_with("| ") {
                    diagnostics.push(
                        Diagnostic::warning(
                            DiagnosticKind::SpaceAfterPipe,
                            "Unnecessary space after '|' in inline child list. Remove for optimal token efficiency.".to_string(),
                            "space-after-pipe",
                        )
                        .with_line(line_num + 1), // 1-indexed line numbers
                    );
                }
            }
        }

        diagnostics
    }
}

/// Rule that validates HEDL v2.0 indentation (1 space per level, no tabs).
///
/// This rule enforces the HEDL v2.0 specification requirement that each nesting
/// level uses exactly 1 space for indentation, and that tabs are not allowed.
pub struct IndentationRule;

impl LintRule for IndentationRule {
    fn id(&self) -> &'static str {
        "indentation"
    }

    fn description(&self) -> &'static str {
        "Validates HEDL v2.0 indentation (1 space per level, no tabs)"
    }

    fn check(&self, doc: &Document) -> Vec<Diagnostic> {
        let diagnostics = Vec::new();

        // Only check v2.0+ documents
        if doc.version < (2, 0) {
            return diagnostics;
        }

        // Without source text, we cannot check indentation
        diagnostics
    }

    fn check_with_context(&self, doc: &Document, context: &dyn Any) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Only check v2.0+ documents
        if doc.version < (2, 0) {
            return diagnostics;
        }

        // Try to downcast context to LintContext
        let Some(ctx) = context.downcast_ref::<crate::runner::LintContext>() else {
            return diagnostics;
        };

        // Check each line of the source text
        let mut nesting_level = 0;
        let mut in_header = true;

        for (line_idx, line) in ctx.source_text.lines().enumerate() {
            let line_number = line_idx + 1;

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            // Check for separator
            if line.starts_with("---") {
                in_header = false;
                nesting_level = 0;
                continue;
            }

            // Skip header lines
            if in_header {
                continue;
            }

            // Get leading whitespace
            let leading_ws = line.len() - line.trim_start().len();
            let leading_text = &line[..leading_ws];

            // Check for tabs
            if leading_text.contains('\t') {
                diagnostics.push(
                    Diagnostic::error(
                        crate::diagnostic::DiagnosticKind::InvalidIndentation,
                        "Tabs are not allowed for indentation in HEDL v2.0 (use spaces instead)"
                            .to_string(),
                        "indentation",
                    )
                    .with_line(line_number)
                    .with_suggestion("Replace tabs with spaces"),
                );
                continue;
            }

            // Determine expected nesting level based on line content
            let trimmed = line.trim_start();

            // Lines starting with |are matrix list rows (inherit parent indent + 1)
            if trimmed.starts_with('|') {
                let expected_indent = nesting_level + 1;
                if leading_ws != expected_indent {
                    diagnostics.push(
                        Diagnostic::error(
                            crate::diagnostic::DiagnosticKind::InvalidIndentation,
                            format!(
                                "Matrix row has {} space(s) but expected {} (1 space per nesting level)",
                                leading_ws, expected_indent
                            ),
                            "indentation",
                        )
                        .with_line(line_number)
                        .with_suggestion(format!("Use {} space(s)", expected_indent)),
                    );
                }
            } else {
                // Regular key-value or nested object lines
                // Check if line has appropriate indentation
                if leading_ws != nesting_level {
                    diagnostics.push(
                        Diagnostic::error(
                            crate::diagnostic::DiagnosticKind::InvalidIndentation,
                            format!(
                                "Line has {} space(s) but expected {} (1 space per nesting level)",
                                leading_ws, nesting_level
                            ),
                            "indentation",
                        )
                        .with_line(line_number)
                        .with_suggestion(format!("Use {} space(s)", nesting_level)),
                    );
                }

                // Update nesting level for next line if this is a key with potential children
                if trimmed.contains(':') && !trimmed.starts_with('%') {
                    let value_part = trimmed.split(':').nth(1).map(|s| s.trim()).unwrap_or("");

                    // If value is empty, check next line for nesting
                    if value_part.is_empty() {
                        // Peek at next line to determine if we should increase nesting
                        if let Some(next_line) = ctx.source_text.lines().nth(line_idx + 1) {
                            if !next_line.trim().is_empty() && !next_line.starts_with("---") {
                                let next_indent = next_line.len() - next_line.trim_start().len();
                                if next_indent > leading_ws {
                                    nesting_level = leading_ws + 1;
                                } else if next_indent < leading_ws {
                                    nesting_level = next_indent;
                                }
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}
