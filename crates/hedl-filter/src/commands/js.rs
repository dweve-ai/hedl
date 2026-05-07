// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! JavaScript/TypeScript command filters.

pub fn filter_npm(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&"> ") || trimmed.starts_with("$ ") || trimmed.is_empty() { continue; }
        if trimmed.contains("ERR!") || trimmed.contains("error") || trimmed.contains("warning") {
            result.push(trimmed.to_string());
        }
    }
    if result.is_empty() { "ok".to_string() } else { result.join("\n") }
}

pub fn filter_npx(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    filter_npm(output, _has_errors, _use_hedl)
}

pub fn filter_pnpm(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    filter_npm(output, _has_errors, _use_hedl)
}

pub fn filter_vitest(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut failures = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("FAIL ") || trimmed.contains("AssertionError") || trimmed.contains("Error:") {
            failures.push(trimmed.to_string());
        } else if trimmed.contains("Test Files") || trimmed.contains("Tests") {
            result.push(trimmed.to_string());
        }
    }
    if failures.is_empty() { result.push("all tests passed".to_string()); }
    else { result.push(format!("\n{} failures:", failures.len())); result.extend(failures); }
    result.join("\n")
}

pub fn filter_jest(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    filter_vitest(output, _has_errors, _use_hedl)
}

pub fn filter_tsc(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut current = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("error TS") {
            if !current.is_empty() { result.push(current.join("\n")); current.clear(); }
            current.push(trimmed.to_string());
        } else if !current.is_empty() {
            current.push(trimmed.to_string());
        }
    }
    if !current.is_empty() { result.push(current.join("\n")); }
    if result.is_empty() { "ok (no type errors)".to_string() } else { result.join("\n\n") }
}

pub fn filter_next(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Route") || trimmed.contains("error") || trimmed.contains("Build error") {
            result.push(trimmed.to_string());
        }
    }
    if result.is_empty() { "ok (build successful)".to_string() } else { result.join("\n") }
}

pub fn filter_eslint(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut current = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("error") || trimmed.contains("warning") {
            if !current.is_empty() { result.push(current.join("\n")); current.clear(); }
            current.push(trimmed.to_string());
        } else if !current.is_empty() {
            current.push(trimmed.to_string());
        }
    }
    if !current.is_empty() { result.push(current.join("\n")); }
    if result.is_empty() { "ok (no lint errors)".to_string() } else { result.join("\n\n") }
}

pub fn filter_prettier(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        if line.contains("[error]") || line.contains("Checking formatting") {
            result.push(line.trim().to_string());
        }
    }
    if result.is_empty() { "ok (all formatted)".to_string() } else { result.join("\n") }
}

pub fn filter_playwright(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut failures = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("Error:") || trimmed.contains("failed") {
            failures.push(trimmed.to_string());
        } else if trimmed.contains("passed") || trimmed.contains("failed") {
            result.push(trimmed.to_string());
        }
    }
    if failures.is_empty() { result.push("all tests passed".to_string()); }
    else { result.push(format!("\n{} failures:", failures.len())); result.extend(failures); }
    result.join("\n")
}

pub fn filter_prisma(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("error") || trimmed.contains("warning") || trimmed.contains("Generated") {
            result.push(trimmed.to_string());
        }
    }
    if result.is_empty() { "ok".to_string() } else { result.join("\n") }
}
