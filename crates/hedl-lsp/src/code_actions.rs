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

//! Code actions for HEDL LSP.
//!
//! This module provides quick fixes and refactoring actions for HEDL documents.

use std::collections::HashMap;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Range, TextEdit, WorkspaceEdit,
};

/// Generate code actions for inline child list issues.
///
/// Provides quick fixes for:
/// - Converting inline children to expanded format when count > 10 (v2.0 style guideline)
/// - Removing spaces after | in inline child data
#[must_use]
pub fn get_inline_child_code_actions(
    uri: &tower_lsp::lsp_types::Url,
    content: &str,
    range: Range,
) -> Vec<CodeActionOrCommand> {
    let mut actions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let line_num = range.start.line as usize;

    if line_num >= lines.len() {
        return actions;
    }

    let line = lines[line_num];
    let trimmed = line.trim_start();
    let indent = " ".repeat(line.len() - trimmed.len());

    // Check for inline child list
    if let Some((type_name, count, after_pipe)) = parse_inline_child_declaration(trimmed) {
        // Action 1: Convert to expanded format if > 10 children (v2.0 style guideline)
        if count > 10 {
            if let Some(expanded) = convert_to_expanded_format(&indent, &type_name, after_pipe) {
                let mut changes = HashMap::new();
                changes.insert(
                    uri.clone(),
                    vec![TextEdit {
                        range: Range {
                            start: range.start,
                            end: tower_lsp::lsp_types::Position {
                                line: range.start.line,
                                character: line.len() as u32,
                            },
                        },
                        new_text: expanded,
                    }],
                );

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Convert to expanded format ({} children)", count),
                    kind: Some(CodeActionKind::REFACTOR_REWRITE),
                    diagnostics: None,
                    edit: Some(WorkspaceEdit {
                        changes: Some(changes),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }
        }

        // Action 2: Remove space after |
        if after_pipe.starts_with(' ') {
            let fixed = format!(
                "{}@{}#{}:|{}",
                indent,
                type_name,
                count,
                after_pipe.trim_start()
            );

            let mut changes = HashMap::new();
            changes.insert(
                uri.clone(),
                vec![TextEdit {
                    range: Range {
                        start: range.start,
                        end: tower_lsp::lsp_types::Position {
                            line: range.start.line,
                            character: line.len() as u32,
                        },
                    },
                    new_text: fixed,
                }],
            );

            actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                title: "Remove space after |".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: None,
                edit: Some(WorkspaceEdit {
                    changes: Some(changes),
                    ..Default::default()
                }),
                ..Default::default()
            }));
        }
    }

    actions
}

/// Parse inline child declaration.
fn parse_inline_child_declaration(line: &str) -> Option<(String, usize, &str)> {
    let line = line.strip_prefix('@')?;
    let hash_pos = line.find('#')?;
    let type_name = line[..hash_pos].to_string();
    let after_hash = &line[hash_pos + 1..];
    let colon_pos = after_hash.find(':')?;
    let count_str = &after_hash[..colon_pos];
    let count = count_str.parse::<usize>().ok()?;
    let after_colon = &after_hash[colon_pos + 1..];
    if !after_colon.starts_with('|') {
        return None;
    }
    let after_pipe = &after_colon[1..];
    Some((type_name, count, after_pipe))
}

/// Convert inline child list to expanded format.
///
/// Takes inline data like `rev-001,5,Great|rev-002,4,Good` and converts to:
/// ```hedl
/// @Review:
/// |rev-001,5,Great
/// |rev-002,4,Good
/// ```
fn convert_to_expanded_format(indent: &str, type_name: &str, data: &str) -> Option<String> {
    let children: Vec<&str> = data.split('|').filter(|s| !s.trim().is_empty()).collect();

    if children.is_empty() {
        return None;
    }

    let mut result = format!("{}@{}:\n", indent, type_name);

    for child in children {
        result.push_str(&format!("{}|{}\n", indent, child.trim()));
    }

    // Remove trailing newline
    result.pop();

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_expanded_format() {
        let data = "rev-001,5,Great|rev-002,4,Good|rev-003,3,Ok";
        let result = convert_to_expanded_format("  ", "Review", data);
        assert!(result.is_some());

        let expanded = result.unwrap();
        assert!(expanded.contains("@Review:"));
        assert!(expanded.contains("|rev-001,5,Great"));
        assert!(expanded.contains("|rev-002,4,Good"));
        assert!(expanded.contains("|rev-003,3,Ok"));
    }

    #[test]
    fn test_convert_to_expanded_format_with_spaces() {
        let data = " rev-001,5,Great | rev-002,4,Good ";
        let result = convert_to_expanded_format("    ", "Item", data);
        assert!(result.is_some());

        let expanded = result.unwrap();
        // Should trim spaces from individual children
        assert!(expanded.contains("|rev-001,5,Great"));
        assert!(expanded.contains("|rev-002,4,Good"));
    }

    #[test]
    fn test_parse_inline_child_declaration() {
        let line = "@Review#3:|rev-001,5|rev-002,4|rev-003,3";
        let result = parse_inline_child_declaration(line);
        assert!(result.is_some());

        let (type_name, count, data) = result.unwrap();
        assert_eq!(type_name, "Review");
        assert_eq!(count, 3);
        assert_eq!(data, "rev-001,5|rev-002,4|rev-003,3");
    }
}
