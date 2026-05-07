// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Ruby command filters.

pub fn filter_rake(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut failures = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Failure:") || trimmed.contains("Error:") {
            failures.push(trimmed.to_string());
        } else if trimmed.contains("runs,") || trimmed.contains("assertions,") {
            result.push(trimmed.to_string());
        }
    }
    if failures.is_empty() { result.push("all tests passed".to_string()); }
    else { result.push(format!("\n{} failures:", failures.len())); result.extend(failures); }
    result.join("\n")
}

pub fn filter_rspec(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut failures = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Failure/Error:") || trimmed.starts_with("  ") && trimmed.contains("expected") {
            failures.push(trimmed.to_string());
        } else if trimmed.contains("examples,") || trimmed.contains("failures,") {
            result.push(trimmed.to_string());
        }
    }
    if failures.is_empty() { result.push("all tests passed".to_string()); }
    else { result.push(format!("\n{} failures:", failures.len())); result.extend(failures); }
    result.join("\n")
}

pub fn filter_rubocop(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("offense") || trimmed.contains("error") {
            result.push(trimmed.to_string());
        }
    }
    if result.is_empty() { "ok (no offenses)".to_string() } else { result.join("\n") }
}
