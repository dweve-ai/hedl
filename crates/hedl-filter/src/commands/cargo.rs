// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Cargo (Rust) command filters.

pub fn filter_test(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut failures = Vec::new();
    let mut in_failure = false;

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("test result:") { result.push(trimmed.to_string()); }
        else if trimmed.starts_with("running ") { result.push(trimmed.to_string()); }
        else if trimmed.starts_with("test ") && trimmed.contains("... FAILED") {
            failures.push(trimmed.to_string());
            in_failure = true;
        } else if in_failure {
            if trimmed.starts_with("test ") || trimmed.is_empty() { in_failure = false; }
            else { failures.push(trimmed.to_string()); }
        } else if trimmed.starts_with("error[") || trimmed.starts_with("warning[") {
            failures.push(trimmed.to_string());
        }
    }

    if failures.is_empty() { result.push("all tests passed".to_string()); }
    else { result.push(format!("\n{} failures:", failures.len())); result.extend(failures); }
    result.join("\n")
}

pub fn filter_build(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Compiling ") || trimmed.starts_with("Checking ") ||
           trimmed.starts_with("Finished ") || trimmed.starts_with("Building ") ||
           trimmed.starts_with("Documenting ") { continue; }
        if !trimmed.is_empty() { result.push(trimmed.to_string()); }
    }
    if result.is_empty() { "ok (build successful)".to_string() }
    else { result.join("\n") }
}

pub fn filter_clippy(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut current = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("error[") || trimmed.starts_with("warning[") {
            if !current.is_empty() { result.push(current.join("\n")); current.clear(); }
            current.push(trimmed.to_string());
        } else if !current.is_empty() { current.push(trimmed.to_string()); }
    }
    if !current.is_empty() { result.push(current.join("\n")); }
    if result.is_empty() { "ok (no clippy warnings)".to_string() }
    else { result.join("\n\n") }
}

pub fn filter_install(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Installed") || trimmed.starts_with("error") || trimmed.starts_with("warning") {
            result.push(trimmed.to_string());
        }
    }
    if result.is_empty() { "ok".to_string() } else { result.join("\n") }
}

pub fn filter_nextest(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    filter_test(output, _has_errors, _use_hedl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_test_pass() {
        let out = filter_test("running 5 tests\ntest foo ... ok\ntest result: ok. 5 passed\n", false, false);
        assert!(out.contains("all tests passed"));
    }

    #[test]
    fn test_filter_build() {
        let out = filter_build("Compiling foo\nerror: something\n", false, false);
        assert!(!out.contains("Compiling"));
        assert!(out.contains("error:"));
    }
}
