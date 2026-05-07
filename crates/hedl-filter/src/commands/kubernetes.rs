// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Kubernetes command filters.

pub fn filter_pods(output: &str, _has_errors: bool, use_hedl: bool) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() { return output.to_string(); }

    let mut pods = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && !parts[0].starts_with("NAME") {
            let restarts = parts.get(3).unwrap_or(&"0");
            pods.push((parts[0].to_string(), parts[1].to_string(), parts[2].to_string(), restarts.to_string()));
        }
    }

    if use_hedl {
        return crate::output::kubectl_pods_to_hedl(&pods);
    }

    pods.iter().map(|(name, ready, status, restarts)| {
        format!("{} {} {} (restarts: {})", name, ready, status, restarts)
    }).collect::<Vec<_>>().join("\n")
}

pub fn filter_services(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() { return output.to_string(); }
    let mut result = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 && !parts[0].starts_with("NAME") {
            let ports = parts.get(4).unwrap_or(&"");
            result.push(format!("{} {} {} {}", parts[0], parts[1], parts[2], ports));
        }
    }
    result.join("\n")
}

pub fn filter_logs(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    crate::commands::docker::filter_logs(output, _has_errors, _use_hedl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_pods() {
        let out = filter_pods("NAME   READY   STATUS   RESTARTS\nweb-1   1/1   Running   0\n", false, false);
        assert!(out.contains("web-1"));
        assert!(!out.contains("NAME"));
    }
}
