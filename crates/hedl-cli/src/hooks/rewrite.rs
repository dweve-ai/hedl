// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Command rewrite engine for AI agent hooks.
//!
//! Rewrites arbitrary shell commands to use HEDL for optimal token efficiency.
//! Covers filtering, data conversion, validation, and all HEDL operations.

use lazy_static::lazy_static;
use regex::{Regex, RegexSet};

/// Check if a command should be rewritten and return the HEDL equivalent.
pub fn get_rewritten(cmd: &str) -> Option<String> {
    // Skip if explicitly disabled
    if cmd.starts_with("HEDL_DISABLED=1") || cmd.starts_with("RTK_DISABLED=1") {
        return None;
    }

    // Skip if already using hedl
    if cmd.starts_with("hedl ") {
        return None;
    }

    // Skip compound commands with heredocs (too complex to rewrite safely)
    if has_heredoc(cmd) {
        return None;
    }

    // Handle compound commands (&&, ||, ;, |, &)
    if is_compound_command(cmd) {
        return rewrite_compound(cmd);
    }

    // Single command - try direct rewrite
    rewrite_single(cmd)
}

fn has_heredoc(cmd: &str) -> bool {
    cmd.contains("<<-") || cmd.contains("<<")
}

fn is_compound_command(cmd: &str) -> bool {
    cmd.contains(" && ") || cmd.contains(" || ") || cmd.contains("; ") || cmd.contains(" | ") || cmd.ends_with(" &")
}

fn rewrite_compound(cmd: &str) -> Option<String> {
    // Tokenize by compound operators
    let operators = [" && ", " || ", "; ", " | "];
    let mut parts = vec![cmd];
    for op in &operators {
        let mut new_parts = Vec::new();
        for part in parts {
            for segment in part.split(op) {
                if !segment.is_empty() {
                    new_parts.push(segment);
                }
            }
        }
        parts = new_parts;
    }

    let mut rewritten_parts = Vec::new();
    let mut changed = false;

    for part in parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        match rewrite_single(trimmed) {
            Some(r) => {
                rewritten_parts.push(r);
                changed = true;
            }
            None => {
                rewritten_parts.push(trimmed.to_string());
            }
        }
    }

    if !changed {
        return None;
    }

    // Simple case: just join with && (most common)
    Some(rewritten_parts.join(" && "))
}

fn rewrite_single(cmd: &str) -> Option<String> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check env var prefix
    let (env_prefix, rest) = split_env_prefix(trimmed);

    // Try each rewrite category
    if let Some(r) = rewrite_data_command(rest) {
        return Some(format!("{}{}", env_prefix, r));
    }
    if let Some(r) = rewrite_filter_command(rest) {
        return Some(format!("{}{}", env_prefix, r));
    }
    if let Some(r) = rewrite_system_command(rest) {
        return Some(format!("{}{}", env_prefix, r));
    }

    None
}

fn split_env_prefix(cmd: &str) -> (String, &str) {
    let mut env_vars = Vec::new();
    let mut rest = cmd;

    while let Some(pos) = rest.find('=') {
        let before = &rest[..pos];
        if before.chars().all(|c| c.is_alphanumeric() || c == '_') {
            // Find the space after the value
            if let Some(space_pos) = rest[pos..].find(' ') {
                let var = &rest[..pos + space_pos];
                env_vars.push(var.to_string());
                rest = rest[pos + space_pos..].trim_start();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if env_vars.is_empty() {
        (String::new(), cmd)
    } else {
        let prefix = env_vars.join(" ") + " ";
        (prefix, rest)
    }
}

// ── Data Format Commands ──────────────────────────────────────

fn rewrite_data_command(cmd: &str) -> Option<String> {
    // JSON files - convert to HEDL for LLM consumption
    if cmd.starts_with("cat ") && cmd.ends_with(".json") {
        let file = &cmd[4..].trim();
        return Some(format!("hedl from-json {} --compact", file));
    }

    // XML files
    if cmd.starts_with("cat ") && cmd.ends_with(".xml") {
        let file = &cmd[4..].trim();
        return Some(format!("hedl from-xml {} --compact", file));
    }

    // YAML files
    if cmd.starts_with("cat ") && (cmd.ends_with(".yaml") || cmd.ends_with(".yml")) {
        let file = &cmd[4..].trim();
        return Some(format!("hedl from-yaml {} --compact", file));
    }

    // CSV files
    if cmd.starts_with("cat ") && cmd.ends_with(".csv") {
        let file = &cmd[4..].trim();
        return Some(format!("hedl from-csv {} --compact", file));
    }

    // TOML files
    if cmd.starts_with("cat ") && cmd.ends_with(".toml") {
        let file = &cmd[4..].trim();
        return Some(format!("hedl from-toml {} --compact", file));
    }

    // Parquet files
    if cmd.starts_with("cat ") && cmd.ends_with(".parquet") {
        let file = &cmd[4..].trim();
        return Some(format!("hedl from-parquet {} --compact", file));
    }

    // jq - pipe through hedl for better formatting
    if cmd.starts_with("jq ") {
        return Some(format!("hedl run -- {}", cmd));
    }

    // curl with JSON output
    if cmd.starts_with("curl ") && (cmd.contains("application/json") || cmd.contains("-H \"Accept: application/json\"")) {
        return Some(format!("hedl run -- {}", cmd));
    }

    None
}

// ── Filter Commands ───────────────────────────────────────────

lazy_static! {
    static ref FILTER_PATTERNS: RegexSet = RegexSet::new([
        // Git
        r"^git\s+status",
        r"^git\s+log",
        r"^git\s+diff",
        r"^git\s+show",
        r"^git\s+branch",
        r"^git\s+add",
        r"^git\s+commit",
        r"^git\s+push",
        r"^git\s+pull",
        r"^git\s+fetch",
        r"^git\s+stash",
        r"^git\s+tag",
        r"^git\s+remote",
        // Cargo
        r"^cargo\s+build",
        r"^cargo\s+test",
        r"^cargo\s+run",
        r"^cargo\s+check",
        r"^cargo\s+clippy",
        r"^cargo\s+fmt",
        // Docker
        r"^docker\s+ps",
        r"^docker\s+images",
        r"^docker\s+logs",
        r"^docker\s+inspect",
        r"^docker\s+network",
        r"^docker\s+volume",
        // Kubernetes
        r"^kubectl\s+get",
        r"^kubectl\s+describe",
        r"^kubectl\s+logs",
        r"^kubectl\s+top",
        // System
        r"^ls\s",
        r"^ls$",
        r"^ll$",
        r"^la$",
        r"^find\s",
        r"^grep\s",
        r"^rg\s",
        r"^ps\s",
        r"^top$",
        r"^htop$",
        r"^df\s",
        r"^du\s",
        r"^env$",
        r"^env\s",
        r"^printenv",
        r"^cat\s",
        r"^head\s",
        r"^tail\s",
        r"^less\s",
        r"^more\s",
        r"^wc\s",
        r"^sort\s",
        r"^uniq\s",
        r"^date$",
        r"^uptime$",
        r"^whoami$",
        r"^uname",
        r"^ping\s",
        // Package managers
        r"^npm\s",
        r"^yarn\s",
        r"^pnpm\s",
        r"^pip\s",
        r"^poetry\s",
        r"^bundle\s",
        r"^gem\s",
        r"^composer\s",
        // Build tools
        r"^make\s",
        r"^make$",
        r"^cmake\s",
        r"^meson\s",
        r"^ninja\s",
        r"^gradle\s",
        r"^mvn\s",
        // Node.js
        r"^node\s",
        r"^deno\s",
        r"^bun\s",
        // Python
        r"^python\s",
        r"^python3\s",
        r"^pytest\s",
        r"^mypy\s",
        r"^black\s",
        r"^ruff\s",
        r"^flake8\s",
        r"^pylint\s",
        // Go
        r"^go\s+build",
        r"^go\s+test",
        r"^go\s+run",
        // Rust
        r"^rustc\s",
        r"^rustup\s",
        // Cloud
        r"^aws\s",
        r"^gcloud\s",
        r"^az\s",
        r"^terraform\s",
        r"^pulumi\s",
        // Version control
        r"^gh\s",
        r"^glab\s",
        r"^svn\s",
        // Misc
        r"^tree\s",
        r"^tree$",
        r"^tldr\s",
        r"^man\s",
    ]).unwrap();
}

fn rewrite_filter_command(cmd: &str) -> Option<String> {
    if FILTER_PATTERNS.is_match(cmd) {
        Some(format!("hedl run -- {}", cmd))
    } else {
        None
    }
}

// ── System / Validation Commands ──────────────────────────────

fn rewrite_system_command(cmd: &str) -> Option<String> {
    // HEDL file validation
    if cmd.ends_with(".hedl") && (cmd.starts_with("cat ") || cmd.starts_with("less ") || cmd.starts_with("more ")) {
        let file = cmd.split_whitespace().last()?;
        return Some(format!("hedl inspect {}", file));
    }

    // Check if file is HEDL
    if cmd.starts_with("file ") && cmd.ends_with(".hedl") {
        let file = &cmd[5..].trim();
        return Some(format!("hedl validate {}", file));
    }

    // Directory listing with HEDL files
    if cmd == "ls" || cmd == "ll" || cmd == "la" {
        return Some("hedl run -- ls".to_string());
    }

    None
}
