// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Cloud command filters.

pub fn filter_aws(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("{") || trimmed.starts_with("[") {
            // JSON output - just truncate if too long
            result.push(trimmed.to_string());
        } else if !trimmed.is_empty() {
            result.push(trimmed.to_string());
        }
    }
    if result.len() > 50 {
        let head = result[..50].join("\n");
        format!("{}\n... ({} more lines)", head, result.len() - 50)
    } else {
        result.join("\n")
    }
}

pub fn filter_psql(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("+") || trimmed.starts_with("|") && trimmed.ends_with("|") {
            // Table border - skip
            if trimmed.contains("---") { continue; }
        }
        if !trimmed.is_empty() {
            result.push(trimmed.to_string());
        }
    }
    result.join("\n")
}
