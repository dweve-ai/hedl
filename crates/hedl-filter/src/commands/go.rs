// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Go command filters.

pub fn filter_go(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut failures = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("--- FAIL:") || trimmed.starts_with("FAIL\t") {
            failures.push(trimmed.to_string());
        } else if trimmed.contains("PASS") || trimmed.contains("ok\t") {
            result.push(trimmed.to_string());
        } else if trimmed.starts_with("# ") {
            result.push(trimmed.to_string());
        }
    }
    if failures.is_empty() { result.push("all tests passed".to_string()); }
    else { result.push(format!("\n{} failures:", failures.len())); result.extend(failures); }
    result.join("\n")
}

pub fn filter_golangci(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut current = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains(": ") && (trimmed.contains("error") || trimmed.contains("warning")) {
            if !current.is_empty() { result.push(current.join("\n")); current.clear(); }
            current.push(trimmed.to_string());
        } else if !current.is_empty() {
            current.push(trimmed.to_string());
        }
    }
    if !current.is_empty() { result.push(current.join("\n")); }
    if result.is_empty() { "ok (no issues)".to_string() } else { result.join("\n\n") }
}
