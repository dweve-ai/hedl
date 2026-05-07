// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! System command filters.

use regex::Regex;

pub fn filter_ls(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("total ") || trimmed.is_empty() { continue; }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 9 {
            let size = parts[4];
            let name = parts[8..].join(" ");
            result.push(format!("{} {}", size, name));
        } else if !trimmed.is_empty() {
            result.push(trimmed.to_string());
        }
    }
    result.join("\n")
}

pub fn filter_find(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("./") { result.push(trimmed[2..].to_string()); }
        else if !trimmed.is_empty() { result.push(trimmed.to_string()); }
    }
    result.join("\n")
}

pub fn filter_grep(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let re = Regex::new(r"^(.+):(\d+):(.+)$").unwrap();
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(caps) = re.captures(trimmed) {
            result.push(format!("{}:{} {}", &caps[1], &caps[2], &caps[3].trim()));
        } else if !trimmed.is_empty() {
            result.push(trimmed.to_string());
        }
    }
    result.join("\n")
}

pub fn filter_env(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let sensitive = ["PASSWORD", "SECRET", "TOKEN", "KEY", "AUTH", "CREDENTIAL", "PRIVATE", "API_KEY", "ACCESS_KEY", "AWS_SECRET"];
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(eq_pos) = trimmed.find('=') {
            let name = &trimmed[..eq_pos];
            let upper = name.to_uppercase();
            let is_sensitive = sensitive.iter().any(|s| upper.contains(s));
            if is_sensitive { result.push(format!("{}=***", name)); }
            else { result.push(trimmed.to_string()); }
        }
    }
    result.join("\n")
}

pub fn filter_ps(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() < 2 { return output.to_string(); }
    let mut result = Vec::new();
    for line in &lines[1..] {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            result.push(format!("{} {}", parts[0], parts[parts.len() - 1]));
        }
    }
    result.join("\n")
}

pub fn filter_df(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Filesystem") || trimmed.is_empty() { continue; }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 6 {
            result.push(format!("{} {} used {} avail {} {} ({})", parts[0], parts[4], parts[2], parts[3], parts[1], parts[5]));
        }
    }
    result.join("\n")
}

pub fn filter_du(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            result.push(format!("{} {}", parts[0], parts[1..].join(" ")));
        }
    }
    result.join("\n")
}

pub fn filter_ping(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut transmitted = 0;
    let mut received = 0;
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.contains("packets transmitted") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 4 {
                transmitted = parts[0].parse().unwrap_or(0);
                received = parts[3].parse().unwrap_or(0);
            }
            result.push(trimmed.to_string());
        } else if trimmed.contains("time=") && result.len() < 5 {
            result.push(trimmed.to_string());
        } else if trimmed.contains("rtt min/avg/max") || trimmed.contains("round-trip") {
            result.push(trimmed.to_string());
        }
    }
    if result.len() > 5 {
        let summary = format!("{} packets transmitted, {} received", transmitted, received);
        result.insert(0, summary);
    }
    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_masks() {
        let out = filter_env("API_KEY=secret\nPATH=/usr/bin\nAWS_SECRET=xyz\n", false, false);
        assert!(out.contains("API_KEY=***"));
        assert!(out.contains("PATH=/usr/bin"));
        assert!(out.contains("AWS_SECRET=***"));
    }
}
