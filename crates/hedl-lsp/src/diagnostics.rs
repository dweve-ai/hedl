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

//! LSP-specific diagnostics for HEDL files.
//!
//! This module provides additional diagnostics beyond the core parser and linter,
//! focusing on LSP-specific concerns like inline child list validation.

use crate::analysis::AnalyzedDocument;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

/// Maximum recommended count for inline children before suggesting expanded format.
/// Per v2.0 spec, this is a "style rule (not a hard syntax limit)".
const MAX_INLINE_CHILDREN: usize = 10;

/// Generate LSP-specific diagnostics for inline child lists.
///
/// Checks for:
/// - Count mismatch between declared #N and actual children
/// - Inline children count exceeding recommended maximum (> 10, style guideline)
/// - Space after |in inline child data
#[must_use]
pub fn check_inline_child_lists(content: &str, analysis: &AnalyzedDocument) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();

        // Check for inline child list pattern:@TypeName#N:|data
        if !trimmed.starts_with('@') {
            continue;
        }

        // Parse the declaration
        if let Some((type_name, count, after_pipe)) = parse_inline_child_declaration(trimmed) {
            // Validate type exists
            if analysis.get_schema(&type_name).is_none() {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String(
                        "inline-child-unknown-type".to_string(),
                    )),
                    source: Some("hedl-lsp".to_string()),
                    message: format!("Unknown type '{type_name}' in inline child list"),
                    ..Default::default()
                });
                continue;
            }

            // Count actual children (pipe-separated values)
            let actual_count = after_pipe
                .split('|')
                .filter(|s| !s.trim().is_empty())
                .count();

            // Check count mismatch
            if actual_count != count {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("inline-child-count-mismatch".to_string())),
                    source: Some("hedl-lsp".to_string()),
                    message: format!(
                        "Inline child count mismatch: declared {count} but found {actual_count} children"
                    ),
                    ..Default::default()
                });
            }

            // Check if count exceeds recommended maximum
            // SPEC v2.0 line 58: "Style rule (not a hard syntax limit): keep inline N <= 10"
            if count > MAX_INLINE_CHILDREN {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String(
                        "inline-child-exceeds-max".to_string(),
                    )),
                    source: Some("hedl-lsp".to_string()),
                    message: format!(
                        "Inline children count ({count}) exceeds recommended maximum ({MAX_INLINE_CHILDREN}). \
                         Consider using expanded format for better readability."
                    ),
                    ..Default::default()
                });
            }

            // Check for space after |
            if after_pipe.starts_with(' ') {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String(
                        "inline-child-space-after-pipe".to_string(),
                    )),
                    source: Some("hedl-lsp".to_string()),
                    message: "Unnecessary space after '|' in inline child list. \
                              Remove for optimal token efficiency."
                        .to_string(),
                    ..Default::default()
                });
            }
        }
    }

    diagnostics
}

/// Parse an inline child declaration line.
///
/// Returns (type_name, count, data_after_pipe) if valid, None otherwise.
fn parse_inline_child_declaration(line: &str) -> Option<(String, usize, &str)> {
    // Pattern:@TypeName#N:|data
    let line = line.strip_prefix('@')?;

    let hash_pos = line.find('#')?;
    let type_name = line[..hash_pos].to_string();

    let after_hash = &line[hash_pos + 1..];
    let colon_pos = after_hash.find(':')?;
    let count_str = &after_hash[..colon_pos];
    let count = count_str.parse::<usize>().ok()?;

    // Must have |after :
    let after_colon = &after_hash[colon_pos + 1..];
    if !after_colon.starts_with('|') {
        return None;
    }

    let after_pipe = &after_colon[1..];

    Some((type_name, count, after_pipe))
}

/// Generate v2.0-specific validation diagnostics.
///
/// Checks for:
/// - Missing required %NULL: directive
/// - Missing required %QUOTE: directive
/// - Use of ditto (^) which is forbidden in v2.0
/// - Use of legacy syntax (%VERSION:, %STRUCT:, %NEST:) in v2.0 documents
#[must_use]
pub fn check_v20_compliance(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Find the separator (---) to know where header ends
    let separator_line = lines.iter().position(|l| l.trim() == "---");

    // Check if this is a v2.0 document
    let is_v20 = lines
        .iter()
        .take(separator_line.unwrap_or(lines.len()))
        .any(|l| {
            let t = l.trim();
            t.starts_with("%V:2.0") || t.starts_with("%V: 2.0")
        });

    if !is_v20 {
        // Not a v2.0 document, skip v2.0-specific checks
        return diagnostics;
    }

    // Check for required headers in v2.0
    let header_lines = separator_line.unwrap_or(lines.len());
    let has_null = lines
        .iter()
        .take(header_lines)
        .any(|l| l.trim().starts_with("%NULL:"));
    let has_quote = lines
        .iter()
        .take(header_lines)
        .any(|l| l.trim().starts_with("%QUOTE:"));

    if !has_null {
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 10,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("v20-missing-null".to_string())),
            source: Some("hedl-lsp".to_string()),
            message: "v2.0 requires %NULL: directive. Add %NULL:~ to define the null character."
                .to_string(),
            ..Default::default()
        });
    }

    if !has_quote {
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 10,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            code: Some(NumberOrString::String("v20-missing-quote".to_string())),
            source: Some("hedl-lsp".to_string()),
            message:
                "v2.0 requires %QUOTE: directive. Add %QUOTE:\" to define the quote character."
                    .to_string(),
            ..Default::default()
        });
    }

    // Check for legacy syntax in v2.0 documents
    for (line_num, line) in lines.iter().take(header_lines).enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("%VERSION") {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: line_num as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: line.len() as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("v20-legacy-version".to_string())),
                source: Some("hedl-lsp".to_string()),
                message: "Legacy %VERSION directive in v2.0 document. Use %V:2.0 instead."
                    .to_string(),
                ..Default::default()
            });
        }

        if trimmed.starts_with("%STRUCT") {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: line_num as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: line.len() as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("v20-legacy-struct".to_string())),
                source: Some("hedl-lsp".to_string()),
                message: "Legacy %STRUCT directive in v2.0 document. Use %S: instead.".to_string(),
                ..Default::default()
            });
        }

        if trimmed.starts_with("%NEST") {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: line_num as u32,
                        character: 0,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: line.len() as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::WARNING),
                code: Some(NumberOrString::String("v20-legacy-nest".to_string())),
                source: Some("hedl-lsp".to_string()),
                message: "Legacy %NEST directive in v2.0 document. Use %N: instead.".to_string(),
                ..Default::default()
            });
        }
    }

    // Check for ditto usage in body (forbidden in v2.0)
    if let Some(body_start) = separator_line {
        for (line_num, line) in lines.iter().enumerate().skip(body_start + 1) {
            let trimmed = line.trim_start();

            // Only check matrix rows (start with |)
            if !trimmed.starts_with('|') {
                continue;
            }

            // Check for ditto operator (^) in row data
            // Need to be careful to only match ^ as a cell value, not inside quotes
            let after_pipe = trimmed.strip_prefix('|').unwrap_or(trimmed);
            if contains_ditto_cell(after_pipe) {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("v20-ditto-forbidden".to_string())),
                    source: Some("hedl-lsp".to_string()),
                    message:
                        "Ditto operator (^) is NOT allowed in v2.0. Use explicit values instead."
                            .to_string(),
                    ..Default::default()
                });
            }
        }
    }

    diagnostics
}

/// Check if a CSV row contains a ditto cell (^) outside of quotes.
fn contains_ditto_cell(row: &str) -> bool {
    let mut in_quotes = false;
    let mut prev_char = None;

    for (i, c) in row.chars().enumerate() {
        match c {
            '"' => {
                // Toggle quote state (handle escaped quotes "")
                if prev_char == Some('"') && in_quotes {
                    prev_char = None;
                    continue;
                }
                in_quotes = !in_quotes;
            }
            '^' if !in_quotes => {
                // Check if this is a standalone ^ cell
                // Should be preceded by comma/|or start and followed by comma/|or end
                let before_ok = i == 0
                    || row.chars().nth(i.saturating_sub(1)) == Some(',')
                    || row.chars().nth(i.saturating_sub(1)) == Some(' ');
                let after_ok = i + 1 >= row.len()
                    || row.chars().nth(i + 1) == Some(',')
                    || row.chars().nth(i + 1) == Some(' ');

                if before_ok && after_ok {
                    return true;
                }
            }
            _ => {}
        }
        prev_char = Some(c);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inline_child_declaration_valid() {
        let line = "@Review#3:|rev-001,5,Great|rev-002,4,Good|rev-003,3,Ok";
        let result = parse_inline_child_declaration(line);
        assert!(result.is_some());
        let (type_name, count, data) = result.unwrap();
        assert_eq!(type_name, "Review");
        assert_eq!(count, 3);
        assert_eq!(data, "rev-001,5,Great|rev-002,4,Good|rev-003,3,Ok");
    }

    #[test]
    fn test_parse_inline_child_declaration_without_space() {
        // v2.0 spec: NO space after pipe character
        let line = "@Review#2:|rev-001,5,Great|rev-002,4,Good";
        let result = parse_inline_child_declaration(line);
        assert!(result.is_some());
        let (type_name, count, data) = result.unwrap();
        assert_eq!(type_name, "Review");
        assert_eq!(count, 2);
        assert_eq!(data, "rev-001,5,Great|rev-002,4,Good");
    }

    #[test]
    fn test_parse_inline_child_declaration_invalid() {
        assert!(parse_inline_child_declaration("@Review:").is_none());
        assert!(parse_inline_child_declaration("@Review#").is_none());
        assert!(parse_inline_child_declaration("@Review#3:").is_none());
        assert!(parse_inline_child_declaration("Review#3:|data").is_none());
    }

    #[test]
    fn test_v20_compliance_missing_headers() {
        let doc = r#"%V:2.0
%S:User:[id,name]
---
users:@User
|u1,Alice
"#;
        let diagnostics = check_v20_compliance(doc);
        assert!(diagnostics
            .iter()
            .any(|d| d.code.as_ref().is_some_and(
                |c| matches!(c, NumberOrString::String(s) if s == "v20-missing-null")
            )));
        assert!(diagnostics
            .iter()
            .any(|d| d.code.as_ref().is_some_and(
                |c| matches!(c, NumberOrString::String(s) if s == "v20-missing-quote")
            )));
    }

    #[test]
    fn test_v20_compliance_complete_headers() {
        let doc = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
|u1,Alice
"#;
        let diagnostics = check_v20_compliance(doc);
        assert!(!diagnostics.iter().any(|d| d.code.as_ref().is_some_and(
            |c| matches!(c, NumberOrString::String(s) if s.starts_with("v20-missing"))
        )));
    }

    #[test]
    fn test_v20_compliance_ditto_forbidden() {
        let doc = r#"%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
|u1,Alice
|u2,^
"#;
        let diagnostics = check_v20_compliance(doc);
        assert!(diagnostics.iter().any(|d| d.code.as_ref().is_some_and(
            |c| matches!(c, NumberOrString::String(s) if s == "v20-ditto-forbidden")
        )));
    }

    #[test]
    fn test_v20_compliance_legacy_syntax() {
        let doc = r#"%V:2.0
%NULL:~
%QUOTE:"
%STRUCT: User: [id, name]
%NEST: Team>User
---
users:@User
|u1,Alice
"#;
        let diagnostics = check_v20_compliance(doc);
        assert!(diagnostics
            .iter()
            .any(|d| d.code.as_ref().is_some_and(
                |c| matches!(c, NumberOrString::String(s) if s == "v20-legacy-struct")
            )));
        assert!(diagnostics
            .iter()
            .any(|d| d.code.as_ref().is_some_and(
                |c| matches!(c, NumberOrString::String(s) if s == "v20-legacy-nest")
            )));
    }

    #[test]
    fn test_v20_compliance_skips_non_v20() {
        let doc = r#"%VERSION: 1.2
%S:User:[id, name]
---
users:@User
|u1,Alice
|u2,^
"#;
        let diagnostics = check_v20_compliance(doc);
        // Should not report any errors for non-v2.0 documents
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn test_contains_ditto_cell() {
        assert!(contains_ditto_cell("^"));
        assert!(contains_ditto_cell("^,other"));
        assert!(contains_ditto_cell("value,^"));
        assert!(contains_ditto_cell("value, ^"));
        assert!(contains_ditto_cell("value,^,other"));

        // Should NOT match ^ inside quotes
        assert!(!contains_ditto_cell("\"has^caret\""));
        assert!(!contains_ditto_cell("\"value with ^ in it\",normal"));
    }
}
