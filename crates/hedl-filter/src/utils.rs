// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Utility functions for text processing and command execution.

use lazy_static::lazy_static;
use regex::Regex;
use std::process::Command;

pub fn strip_ansi(text: &str) -> String {
    lazy_static! {
        static ref ANSI_RE: Regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").unwrap();
    }
    ANSI_RE.replace_all(text, "").to_string()
}

pub fn truncate(s: &str, max_len: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_len { s.to_string() }
    else if max_len < 3 { "...".to_string() }
    else { format!("{}...", s.chars().take(max_len - 3).collect::<String>()) }
}

pub fn count_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}

pub fn format_tokens(n: usize) -> String {
    if n >= 1_000_000 { format!("{:.1}M", n as f64 / 1_000_000.0) }
    else if n >= 1_000 { format!("{:.1}K", n as f64 / 1_000.0) }
    else { format!("{}", n) }
}

pub fn exit_code_from_output(output: &std::process::Output) -> i32 {
    match output.status.code() {
        Some(code) => code,
        None => {
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                if let Some(sig) = output.status.signal() { return 128 + sig; }
            }
            1
        }
    }
}

pub fn resolved_command(name: &str) -> Command {
    match which::which(name) {
        Ok(path) => Command::new(path),
        Err(_) => Command::new(name),
    }
}

pub fn tool_exists(name: &str) -> bool {
    which::which(name).is_ok()
}

pub fn human_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB { format!("{:.1} TB", bytes as f64 / TB as f64) }
    else if bytes >= GB { format!("{:.1} GB", bytes as f64 / GB as f64) }
    else if bytes >= MB { format!("{:.1} MB", bytes as f64 / MB as f64) }
    else if bytes >= KB { format!("{:.1} KB", bytes as f64 / KB as f64) }
    else { format!("{} B", bytes) }
}

pub fn join_with_overflow(items: &[ String ], total: usize, max: usize, label: &str) -> String {
    let mut out = items.join("\n");
    if total > max {
        out.push_str(&format!("\n... +{} more {}", total - max, label));
    }
    out
}

pub fn shorten_arn(arn: &str) -> &str {
    let slash = arn.rsplit('/').next().unwrap_or(arn);
    if slash == arn { arn.rsplit(':').next().unwrap_or(arn) } else { slash }
}

pub fn detect_package_manager() -> &'static str {
    if std::path::Path::new("pnpm-lock.yaml").exists() { "pnpm" }
    else if std::path::Path::new("yarn.lock").exists() { "yarn" }
    else if std::path::Path::new("package-lock.json").exists() { "npm" }
    else if std::path::Path::new("bun.lockb").exists() || std::path::Path::new("bun.lock").exists() { "bun" }
    else { "npm" }
}

pub fn shell_split(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }
        match ch {
            '\\' if in_double => { escape = true; }
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            ' ' | '\t' | '\n' if !in_single && !in_double => {
                if !current.is_empty() { tokens.push(current.clone()); current.clear(); }
            }
            _ => { current.push(ch); }
        }
    }
    if !current.is_empty() { tokens.push(current); }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[31mError\x1b[0m"), "Error");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 10), "hi");
    }

    #[test]
    fn test_shell_split() {
        assert_eq!(shell_split("git log --format='%H %s'"), vec!["git", "log", "--format=%H %s"]);
    }
}
