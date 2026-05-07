// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! .NET command filters.

pub fn filter_dotnet(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut current = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("error") || trimmed.contains("warning") {
            if !current.is_empty() { result.push(current.join("\n")); current.clear(); }
            current.push(trimmed.to_string());
        } else if !current.is_empty() {
            current.push(trimmed.to_string());
        } else if trimmed.contains("Build succeeded") || trimmed.contains("Build FAILED") {
            result.push(trimmed.to_string());
        }
    }
    if !current.is_empty() { result.push(current.join("\n")); }
    if result.is_empty() { "ok".to_string() } else { result.join("\n\n") }
}
