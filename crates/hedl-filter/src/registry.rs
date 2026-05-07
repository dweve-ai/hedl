// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Command classification and rewrite registry.
//!
//! Matches shell commands against known rewrite rules to translate them
//! into their HEDL-filter equivalents.

use lazy_static::lazy_static;
use regex::{Regex, RegexSet};

/// Classification result for a command.
#[derive(Debug, PartialEq, Clone)]
pub enum Classification {
    /// Command is supported with rewrite info
    Supported {
        hedl_equivalent: String,
        category: String,
        estimated_savings_pct: f64,
    },
    /// Command is not supported
    Unsupported {
        base_command: String,
    },
    /// Command should be ignored
    Ignored,
}

/// A rewrite rule.
struct Rule {
    pattern: &'static str,
    replacement: &'static str,
    category: &'static str,
    savings: f64,
}

const RULES: &[Rule] = &[
    // Git
    Rule { pattern: r"^git\s+status", replacement: "hedl git status", category: "Git", savings: 70.0 },
    Rule { pattern: r"^git\s+log", replacement: "hedl git log", category: "Git", savings: 65.0 },
    Rule { pattern: r"^git\s+diff", replacement: "hedl git diff", category: "Git", savings: 80.0 },
    Rule { pattern: r"^git\s+show", replacement: "hedl git show", category: "Git", savings: 75.0 },
    Rule { pattern: r"^git\s+branch", replacement: "hedl git branch", category: "Git", savings: 60.0 },
    Rule { pattern: r"^git\s+add", replacement: "hedl git add", category: "Git", savings: 50.0 },
    Rule { pattern: r"^git\s+commit", replacement: "hedl git commit", category: "Git", savings: 50.0 },
    Rule { pattern: r"^git\s+push", replacement: "hedl git push", category: "Git", savings: 50.0 },
    Rule { pattern: r"^git\s+pull", replacement: "hedl git pull", category: "Git", savings: 50.0 },
    Rule { pattern: r"^git\s+fetch", replacement: "hedl git fetch", category: "Git", savings: 50.0 },
    Rule { pattern: r"^git\s+stash", replacement: "hedl git stash", category: "Git", savings: 60.0 },
    Rule { pattern: r"^git\s+worktree", replacement: "hedl git worktree", category: "Git", savings: 60.0 },
    // Cargo
    Rule { pattern: r"^cargo\s+test", replacement: "hedl cargo test", category: "Rust", savings: 90.0 },
    Rule { pattern: r"^cargo\s+build", replacement: "hedl cargo build", category: "Rust", savings: 75.0 },
    Rule { pattern: r"^cargo\s+check", replacement: "hedl cargo check", category: "Rust", savings: 75.0 },
    Rule { pattern: r"^cargo\s+clippy", replacement: "hedl cargo clippy", category: "Rust", savings: 80.0 },
    Rule { pattern: r"^cargo\s+install", replacement: "hedl cargo install", category: "Rust", savings: 70.0 },
    Rule { pattern: r"^cargo\s+nextest", replacement: "hedl cargo nextest", category: "Rust", savings: 85.0 },
    // Docker
    Rule { pattern: r"^docker\s+ps", replacement: "hedl docker ps", category: "Infra", savings: 70.0 },
    Rule { pattern: r"^docker\s+images", replacement: "hedl docker images", category: "Infra", savings: 65.0 },
    Rule { pattern: r"^docker\s+logs", replacement: "hedl docker logs", category: "Infra", savings: 85.0 },
    Rule { pattern: r"^docker\s+compose\s+ps", replacement: "hedl docker compose ps", category: "Infra", savings: 70.0 },
    Rule { pattern: r"^docker\s+compose\s+logs", replacement: "hedl docker compose logs", category: "Infra", savings: 85.0 },
    // Kubectl
    Rule { pattern: r"^kubectl\s+get\s+pods", replacement: "hedl kubectl get pods", category: "Infra", savings: 70.0 },
    Rule { pattern: r"^kubectl\s+get\s+services", replacement: "hedl kubectl get services", category: "Infra", savings: 70.0 },
    Rule { pattern: r"^kubectl\s+logs", replacement: "hedl kubectl logs", category: "Infra", savings: 85.0 },
    // System
    Rule { pattern: r"^ls\b", replacement: "hedl sys ls", category: "Files", savings: 65.0 },
    Rule { pattern: r"^find\b", replacement: "hedl sys find", category: "Files", savings: 70.0 },
    Rule { pattern: r"^grep\b", replacement: "hedl sys grep", category: "Files", savings: 75.0 },
    Rule { pattern: r"^rg\b", replacement: "hedl sys grep", category: "Files", savings: 75.0 },
    Rule { pattern: r"^env\b", replacement: "hedl sys env", category: "System", savings: 60.0 },
    Rule { pattern: r"^ps\b", replacement: "hedl sys ps", category: "System", savings: 65.0 },
    Rule { pattern: r"^df\b", replacement: "hedl sys df", category: "System", savings: 60.0 },
    Rule { pattern: r"^du\b", replacement: "hedl sys du", category: "System", savings: 60.0 },
    Rule { pattern: r"^ping\b", replacement: "hedl sys ping", category: "Network", savings: 70.0 },
    // JS/TS
    Rule { pattern: r"^npm\s+run", replacement: "hedl npm run", category: "JS", savings: 60.0 },
    Rule { pattern: r"^npx\b", replacement: "hedl npx", category: "JS", savings: 60.0 },
    Rule { pattern: r"^pnpm\b", replacement: "hedl pnpm", category: "JS", savings: 60.0 },
    Rule { pattern: r"^vitest\b", replacement: "hedl vitest", category: "JS", savings: 99.5 },
    Rule { pattern: r"^jest\b", replacement: "hedl jest", category: "JS", savings: 99.5 },
    Rule { pattern: r"^tsc\b", replacement: "hedl tsc", category: "JS", savings: 75.0 },
    Rule { pattern: r"^next\s+build", replacement: "hedl next build", category: "JS", savings: 70.0 },
    Rule { pattern: r"^eslint\b", replacement: "hedl lint", category: "JS", savings: 70.0 },
    Rule { pattern: r"^prettier\b", replacement: "hedl prettier", category: "JS", savings: 65.0 },
    Rule { pattern: r"^playwright\s+test", replacement: "hedl playwright test", category: "JS", savings: 94.0 },
    Rule { pattern: r"^prisma\b", replacement: "hedl prisma", category: "JS", savings: 60.0 },
    // Python
    Rule { pattern: r"^pytest\b", replacement: "hedl pytest", category: "Python", savings: 90.0 },
    Rule { pattern: r"^ruff\b", replacement: "hedl ruff", category: "Python", savings: 70.0 },
    Rule { pattern: r"^mypy\b", replacement: "hedl mypy", category: "Python", savings: 70.0 },
    Rule { pattern: r"^pip\b", replacement: "hedl pip", category: "Python", savings: 60.0 },
    // Ruby
    Rule { pattern: r"^rake\s+test", replacement: "hedl rake test", category: "Ruby", savings: 90.0 },
    Rule { pattern: r"^rspec\b", replacement: "hedl rspec", category: "Ruby", savings: 60.0 },
    Rule { pattern: r"^rubocop\b", replacement: "hedl rubocop", category: "Ruby", savings: 70.0 },
    // Go
    Rule { pattern: r"^go\s+test", replacement: "hedl go test", category: "Go", savings: 90.0 },
    Rule { pattern: r"^go\s+build", replacement: "hedl go build", category: "Go", savings: 70.0 },
    Rule { pattern: r"^go\s+vet", replacement: "hedl go vet", category: "Go", savings: 70.0 },
    Rule { pattern: r"^golangci-lint\b", replacement: "hedl golangci-lint", category: "Go", savings: 70.0 },
    // GitHub/GitLab
    Rule { pattern: r"^gh\b", replacement: "hedl gh", category: "GitHub", savings: 70.0 },
    Rule { pattern: r"^glab\b", replacement: "hedl glab", category: "GitLab", savings: 70.0 },
    // Cloud
    Rule { pattern: r"^aws\b", replacement: "hedl aws", category: "Cloud", savings: 65.0 },
    Rule { pattern: r"^psql\b", replacement: "hedl psql", category: "Cloud", savings: 60.0 },
    // .NET
    Rule { pattern: r"^dotnet\s+build", replacement: "hedl dotnet build", category: "Dotnet", savings: 70.0 },
    Rule { pattern: r"^dotnet\s+test", replacement: "hedl dotnet test", category: "Dotnet", savings: 70.0 },
    Rule { pattern: r"^dotnet\s+restore", replacement: "hedl dotnet restore", category: "Dotnet", savings: 60.0 },
    Rule { pattern: r"^dotnet\s+format", replacement: "hedl dotnet format", category: "Dotnet", savings: 60.0 },
    // File reading
    Rule { pattern: r"^cat\b", replacement: "hedl read", category: "Files", savings: 60.0 },
    Rule { pattern: r"^head\b", replacement: "hedl read", category: "Files", savings: 60.0 },
    Rule { pattern: r"^tail\b", replacement: "hedl read", category: "Files", savings: 60.0 },
    Rule { pattern: r"^tree\b", replacement: "hedl sys tree", category: "Files", savings: 70.0 },
];

lazy_static! {
    static ref REGEX_SET: RegexSet =
        RegexSet::new(RULES.iter().map(|r| r.pattern)).expect("invalid regex patterns");
    static ref COMPILED: Vec<Regex> = RULES
        .iter()
        .map(|r| Regex::new(r.pattern).expect("invalid regex"))
        .collect();
}

/// Classify a command string.
pub fn classify_command(cmd: &str) -> Classification {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return Classification::Ignored;
    }

    // Check if already hedl
    if trimmed.starts_with("hedl ") {
        return Classification::Ignored;
    }

    let matches: Vec<usize> = REGEX_SET.matches(trimmed).into_iter().collect();

    if matches.is_empty() {
        let base = trimmed.split_whitespace().next().unwrap_or(trimmed);
        return Classification::Unsupported {
            base_command: base.to_string(),
        };
    }

    let idx = matches[0];
    let rule = &RULES[idx];

    // Extract captures for replacement
    let rewritten = if let Some(_caps) = COMPILED[idx].captures(trimmed) {
        COMPILED[idx].replace(trimmed, rule.replacement).to_string()
    } else {
        rule.replacement.to_string()
    };

    Classification::Supported {
        hedl_equivalent: rewritten,
        category: rule.category.to_string(),
        estimated_savings_pct: rule.savings,
    }
}

/// Rewrite a command to its HEDL equivalent.
pub fn rewrite_command(cmd: &str) -> Option<String> {
    match classify_command(cmd) {
        Classification::Supported { hedl_equivalent, .. } => Some(hedl_equivalent),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status() {
        let c = classify_command("git status");
        assert!(matches!(c, Classification::Supported { .. }));
        assert_eq!(rewrite_command("git status"), Some("hedl git status".to_string()));
    }

    #[test]
    fn test_unsupported() {
        assert!(matches!(classify_command("htop"), Classification::Unsupported { .. }));
    }

    #[test]
    fn test_ignored() {
        assert_eq!(classify_command("hedl git status"), Classification::Ignored);
    }

    #[test]
    fn test_cargo_test() {
        assert_eq!(rewrite_command("cargo test"), Some("hedl cargo test".to_string()));
    }
}
