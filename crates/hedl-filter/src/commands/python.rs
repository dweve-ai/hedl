// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Python command filters.

pub fn filter_pytest(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut failures = Vec::new();
    let mut in_failure = false;
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("FAILED") || trimmed.contains("ERROR") {
            failures.push(trimmed.to_string());
            in_failure = true;
        } else if in_failure {
            if trimmed.is_empty() || trimmed.starts_with("=") { in_failure = false; }
            else { failures.push(trimmed.to_string()); }
        } else if trimmed.contains("passed") || trimmed.contains("failed") {
            result.push(trimmed.to_string());
        }
    }
    if failures.is_empty() { result.push("all tests passed".to_string()); }
    else { result.push(format!("\n{} failures:", failures.len())); result.extend(failures); }
    result.join("\n")
}

pub fn filter_ruff(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Found") || trimmed.contains("error[") {
            result.push(trimmed.to_string());
        }
    }
    if result.is_empty() { "ok (no issues)".to_string() } else { result.join("\n") }
}

pub fn filter_mypy(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut current_file = String::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("error:") || trimmed.contains("warning:") {
            if !current_file.is_empty() { result.push(current_file.clone()); current_file.clear(); }
            result.push(trimmed.to_string());
        }
    }
    if result.is_empty() { "ok (no type errors)".to_string() } else { result.join("\n") }
}

pub fn filter_pip(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Successfully") || trimmed.starts_with("ERROR") || trimmed.starts_with("WARNING") {
            result.push(trimmed.to_string());
        }
    }
    if result.is_empty() { "ok".to_string() } else { result.join("\n") }
}
