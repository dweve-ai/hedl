// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Git command filters with HEDL output support.

use crate::output;

pub fn filter_status(output: &str, _has_errors: bool, use_hedl: bool) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut branch = "unknown";
    let mut staged = Vec::new();
    let mut modified = Vec::new();
    let mut untracked = Vec::new();
    let mut deleted = Vec::new();
    let mut renamed = Vec::new();

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            branch = trimmed.strip_prefix("## ").unwrap_or(branch);
            if let Some(idx) = branch.find("...") { branch = &branch[..idx]; }
        } else if line.len() >= 2 {
            let status = &line[..2];
            let file = line[2..].trim();
            match status {
                "M " | "A " | "D " | "R " | "C " => staged.push(file.to_string()),
                " M" => modified.push(file.to_string()),
                " D" => deleted.push(file.to_string()),
                " R" => renamed.push(file.to_string()),
                "MM" | "AM" | "DM" | "RM" => { staged.push(file.to_string()); modified.push(file.to_string()); }
                "??" => untracked.push(file.to_string()),
                _ => {}
            }
        }
    }

    if use_hedl {
        return output::git_status_to_hedl(branch, &staged, &modified, &deleted, &untracked);
    }

    let mut result = format!("* {}\n", branch);
    if !staged.is_empty() {
        result.push_str(&format!("+ Staged: {}\n", staged.len()));
        for f in &staged[..staged.len().min(20)] { result.push_str(&format!("   {}\n", f)); }
        if staged.len() > 20 { result.push_str(&format!("   ... +{} more\n", staged.len() - 20)); }
    }
    if !modified.is_empty() {
        result.push_str(&format!("~ Modified: {}\n", modified.len()));
        for f in &modified[..modified.len().min(20)] { result.push_str(&format!("   {}\n", f)); }
        if modified.len() > 20 { result.push_str(&format!("   ... +{} more\n", modified.len() - 20)); }
    }
    if !deleted.is_empty() {
        result.push_str(&format!("- Deleted: {}\n", deleted.len()));
        for f in &deleted[..deleted.len().min(20)] { result.push_str(&format!("   {}\n", f)); }
    }
    if !renamed.is_empty() {
        result.push_str(&format!("> Renamed: {}\n", renamed.len()));
        for f in &renamed[..renamed.len().min(20)] { result.push_str(&format!("   {}\n", f)); }
    }
    if !untracked.is_empty() {
        result.push_str(&format!("? Untracked: {}\n", untracked.len()));
        for f in &untracked[..untracked.len().min(20)] { result.push_str(&format!("   {}\n", f)); }
        if untracked.len() > 20 { result.push_str(&format!("   ... +{} more\n", untracked.len() - 20)); }
    }
    if staged.is_empty() && modified.is_empty() && deleted.is_empty() && renamed.is_empty() && untracked.is_empty() {
        result.push_str("clean\n");
    }
    result.trim().to_string()
}

pub fn filter_log(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let re = regex::Regex::new(r"^([a-f0-9]+)\s+(.+)$").unwrap();
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(caps) = re.captures(trimmed) {
            let hash = &caps[1][..7.min(caps[1].len())];
            result.push(format!("{} {}", hash, &caps[2]));
        }
    }
    if result.is_empty() {
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.len() > 10 && trimmed.starts_with(|c: char| c.is_ascii_hexdigit()) {
                result.push(trimmed.to_string());
            }
        }
    }
    result.join("\n")
}

pub fn filter_diff(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut in_hunk = false;
    let mut hunk_lines = 0;
    for line in output.lines() {
        if line.starts_with("diff --git") { result.push(line.to_string()); in_hunk = false; hunk_lines = 0; }
        else if line.starts_with("@@") { result.push(line.to_string()); in_hunk = true; hunk_lines = 0; }
        else if in_hunk {
            if hunk_lines < 100 { result.push(line.to_string()); }
            else if hunk_lines == 100 { result.push("... (hunk truncated)".to_string()); }
            hunk_lines += 1;
        } else if line.starts_with("--- ") || line.starts_with("+++ ") || line.starts_with("index ") {
            result.push(line.to_string());
        }
    }
    result.join("\n")
}

pub fn filter_show(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    let mut in_diff = false;
    for line in output.lines() {
        if line.starts_with("commit ") { result.push(format!("commit {}", &line[7..15.min(line.len())])); }
        else if line.starts_with("Author: ") || line.starts_with("Date: ") || line.starts_with("    ") {
            result.push(line.to_string());
        } else if line.starts_with("diff --git") { in_diff = true; result.push(line.to_string()); }
        else if in_diff && (line.starts_with("@@") || line.starts_with("+") || line.starts_with("-")) {
            result.push(line.to_string());
        }
    }
    result.join("\n")
}

pub fn filter_branch(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('*') { result.push(format!("* {}", trimmed[1..].trim())); }
        else if !trimmed.is_empty() { result.push(trimmed.to_string()); }
    }
    result.join("\n")
}

pub fn filter_add(_output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    "ok".to_string()
}

pub fn filter_commit(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    for line in output.lines() {
        if line.starts_with('[') || line.starts_with("[") {
            if let Some(idx) = line.find(' ') {
                return format!("ok {}", &line[..idx]);
            }
        }
    }
    "ok".to_string()
}

pub fn filter_push(_output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    "ok".to_string()
}

pub fn filter_pull(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut files = 0;
    let mut insertions = 0;
    let mut deletions = 0;
    for line in output.lines() {
        if line.contains("file changed") || line.contains("files changed") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, p) in parts.iter().enumerate() {
                if *p == "file" || *p == "files" { files = parts[i-1].parse().unwrap_or(0); }
                if p.contains("insertion") { insertions = parts[i-1].parse().unwrap_or(0); }
                if p.contains("deletion") { deletions = parts[i-1].parse().unwrap_or(0); }
            }
        }
    }
    if files == 0 { "ok (up-to-date)".to_string() }
    else { format!("ok {} files +{} -{}", files, insertions, deletions) }
}

pub fn filter_fetch(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut new_refs = 0;
    for line in output.lines() {
        if line.contains("* [new branch]") || line.contains("* [new tag]") { new_refs += 1; }
    }
    format!("ok fetched ({} new refs)", new_refs)
}

pub fn filter_stash(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() { return "ok".to_string(); }
    let count = lines.len();
    format!("stash: {} entries", count)
}

pub fn filter_worktree(output: &str, _has_errors: bool, _use_hedl: bool) -> String {
    let mut result = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("worktree") {
            result.push(trimmed.to_string());
        }
    }
    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_status_text() {
        let input = "## main...origin/main\n M src/main.rs\n?? notes.md\nM  Cargo.toml\n";
        let out = filter_status(input, false, false);
        assert!(out.contains("* main"));
        assert!(out.contains("~ Modified: 1"));
        assert!(out.contains("? Untracked: 1"));
        assert!(out.contains("+ Staged: 1"));
    }

    #[test]
    fn test_filter_status_hedl() {
        let input = "## main...origin/main\n M src/main.rs\n?? notes.md\nM  Cargo.toml\n";
        let out = filter_status(input, false, true);
        assert!(out.contains("%V:2.0"));
        assert!(out.contains("%S:GitStatus:"));
    }

    #[test]
    fn test_filter_log() {
        let out = filter_log("abc1234 Fix bug\ndef5678 Add feature\n", false, false);
        assert!(out.contains("abc1234"));
        assert!(out.contains("Fix bug"));
    }

    #[test]
    fn test_filter_pull() {
        let out = filter_pull("3 files changed, 10 insertions(+), 2 deletions(-)\n", false, false);
        assert_eq!(out, "ok 3 files +10 -2");
    }
}
