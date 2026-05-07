// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Docker command filters.

pub fn filter_ps(output: &str, _has_errors: bool, use_hedl: bool) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() < 2 { return output.to_string(); }

    let mut containers = Vec::new();
    for line in &lines[1..] {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let id = &parts[0][..12.min(parts[0].len())];
            let image = parts[1];
            let status = parts.iter().find(|p| p.contains("Up") || p.contains("Exited")).copied().unwrap_or("unknown");
            let name = parts.last().unwrap_or(&"unknown");
            containers.push((id.to_string(), image.to_string(), status.to_string(), name.to_string()));
        }
    }

    if use_hedl {
        return crate::output::docker_ps_to_hedl(&containers);
    }

    containers.iter().map(|(id, image, status, name)| {
        format!("{} {} {} {}", id, image, status, name)
    }).collect::<Vec<_>>().join("\n")
}

pub fn filter_images(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() < 2 { return output.to_string(); }
    let mut result = Vec::new();
    for line in &lines[1..] {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            result.push(format!("{}:{} {}", parts[0], parts[1], parts[parts.len()-1]));
        }
    }
    result.join("\n")
}

pub fn filter_logs(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut last = "";
    let mut dup_count = 0;
    for line in output.lines() {
        if line == last { dup_count += 1; }
        else {
            if dup_count > 0 { result.push(format!("... ({}x)", dup_count + 1)); }
            result.push(line.to_string());
            last = line; dup_count = 0;
        }
    }
    if dup_count > 0 { result.push(format!("... ({}x)", dup_count + 1)); }
    result.join("\n")
}

pub fn filter_compose_ps(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    filter_ps(output, _has_errors, _use_hedl)
}

pub fn filter_compose_logs(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    filter_logs(output, _has_errors, _use_hedl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_ps() {
        let input = "CONTAINER ID   IMAGE    COMMAND   CREATED   STATUS    PORTS   NAMES\nabc123   nginx   \"nginx\"   2h   Up 2h   80/tcp   web\n";
        let out = filter_ps(input, false, false);
        assert!(out.contains("abc123"));
        assert!(!out.contains("CONTAINER ID"));
    }
}
