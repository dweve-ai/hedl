// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Source code filtering with language-aware comment stripping.

use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterLevel {
    None,
    Minimal,
    Aggressive,
}

impl FromStr for FilterLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(FilterLevel::None),
            "minimal" => Ok(FilterLevel::Minimal),
            "aggressive" => Ok(FilterLevel::Aggressive),
            _ => Err(format!("Unknown filter level: {}", s)),
        }
    }
}

impl std::fmt::Display for FilterLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterLevel::None => write!(f, "none"),
            FilterLevel::Minimal => write!(f, "minimal"),
            FilterLevel::Aggressive => write!(f, "aggressive"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust, Python, JavaScript, TypeScript, Go, C, Cpp, Java, Ruby, Shell, Data, Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" | "pyw" => Language::Python,
            "js" | "mjs" | "cjs" => Language::JavaScript,
            "ts" | "tsx" => Language::TypeScript,
            "go" => Language::Go,
            "c" | "h" => Language::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hh" => Language::Cpp,
            "java" => Language::Java,
            "rb" => Language::Ruby,
            "sh" | "bash" | "zsh" => Language::Shell,
            "json" | "jsonc" | "json5" | "yaml" | "yml" | "toml" | "xml" | "csv" | "tsv"
            | "graphql" | "gql" | "sql" | "md" | "markdown" | "txt" | "env" | "lock" => Language::Data,
            _ => Language::Unknown,
        }
    }

    pub fn comment_patterns(&self) -> CommentPatterns {
        match self {
            Language::Rust => CommentPatterns {
                line: Some("//"), block_start: Some("/*"), block_end: Some("*/"),
                doc_line: Some("///"), doc_block_start: Some("/**"),
            },
            Language::Python => CommentPatterns {
                line: Some("#"), block_start: Some("\"\"\""), block_end: Some("\"\"\""),
                doc_line: None, doc_block_start: Some("\"\"\""),
            },
            Language::JavaScript | Language::TypeScript | Language::Go | Language::C |
            Language::Cpp | Language::Java => CommentPatterns {
                line: Some("//"), block_start: Some("/*"), block_end: Some("*/"),
                doc_line: None, doc_block_start: Some("/**"),
            },
            Language::Ruby => CommentPatterns {
                line: Some("#"), block_start: Some("=begin"), block_end: Some("=end"),
                doc_line: None, doc_block_start: None,
            },
            Language::Shell => CommentPatterns {
                line: Some("#"), block_start: None, block_end: None,
                doc_line: None, doc_block_start: None,
            },
            Language::Data | Language::Unknown => CommentPatterns {
                line: None, block_start: None, block_end: None,
                doc_line: None, doc_block_start: None,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommentPatterns {
    pub line: Option<&'static str>,
    pub block_start: Option<&'static str>,
    pub block_end: Option<&'static str>,
    pub doc_line: Option<&'static str>,
    pub doc_block_start: Option<&'static str>,
}

pub fn filter_source(content: &str, level: FilterLevel, lang: Language) -> String {
    match level {
        FilterLevel::None => content.to_string(),
        FilterLevel::Minimal => filter_minimal(content, &lang),
        FilterLevel::Aggressive => filter_aggressive(content, &lang),
    }
}

fn filter_minimal(content: &str, lang: &Language) -> String {
    let patterns = lang.comment_patterns();
    let mut result = String::with_capacity(content.len());
    let mut in_block = false;
    let mut in_docstring = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if let (Some(start), Some(end)) = (patterns.block_start, patterns.block_end) {
            if !in_docstring && trimmed.contains(start) && !trimmed.starts_with(patterns.doc_block_start.unwrap_or("###")) {
                in_block = true;
            }
            if in_block {
                if trimmed.contains(end) { in_block = false; }
                continue;
            }
        }

        if *lang == Language::Python && trimmed.starts_with("\"\"\"") {
            in_docstring = !in_docstring;
            result.push_str(line); result.push('\n');
            continue;
        }
        if in_docstring {
            result.push_str(line); result.push('\n');
            continue;
        }

        if let Some(lc) = patterns.line {
            if trimmed.starts_with(lc) {
                if let Some(doc) = patterns.doc_line {
                    if trimmed.starts_with(doc) {
                        result.push_str(line); result.push('\n');
                    }
                }
                continue;
            }
        }

        if trimmed.is_empty() { result.push('\n'); continue; }
        result.push_str(line); result.push('\n');
    }

    let re = Regex::new(r"\n{3,}").unwrap();
    re.replace_all(&result, "\n\n").trim().to_string()
}

fn filter_aggressive(content: &str, lang: &Language) -> String {
    if *lang == Language::Data { return filter_minimal(content, lang); }

    lazy_static! {
        static ref IMPORT: Regex = Regex::new(r"^(use |import |from |require\(|#include)").unwrap();
        static ref SIG: Regex = Regex::new(r"^(pub\s+)?(async\s+)?(fn|def|function|func|class|struct|enum|trait|interface|type)\s+\w+").unwrap();
    }

    let minimal = filter_minimal(content, lang);
    let mut result = String::with_capacity(minimal.len() / 2);
    let mut brace_depth = 0i32;
    let mut in_body = false;

    for line in minimal.lines() {
        let trimmed = line.trim();

        if IMPORT.is_match(trimmed) {
            result.push_str(line); result.push('\n'); continue;
        }
        if SIG.is_match(trimmed) {
            result.push_str(line); result.push('\n');
            in_body = true; brace_depth = 0;
            continue;
        }

        let open = trimmed.matches('{').count() as i32;
        let close = trimmed.matches('}').count() as i32;

        if in_body {
            brace_depth += open; brace_depth -= close;
            if brace_depth <= 1 && (trimmed == "{" || trimmed == "}" || trimmed.ends_with('{')) {
                result.push_str(line); result.push('\n');
            }
            if brace_depth <= 0 {
                in_body = false;
                if !trimmed.is_empty() && trimmed != "}" {
                    result.push_str("    // ... implementation\n");
                }
            }
            continue;
        }

        if trimmed.starts_with("const ") || trimmed.starts_with("static ") ||
           trimmed.starts_with("let ") || trimmed.starts_with("pub const ") ||
           trimmed.starts_with("pub static ") {
            result.push_str(line); result.push('\n');
        }
    }

    result.trim().to_string()
}

pub fn smart_truncate(content: &str, max_lines: usize, _lang: &Language) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= max_lines { return content.to_string(); }

    let mut result = Vec::with_capacity(max_lines + 1);
    let mut kept = 0;

    lazy_static! {
        static ref IMPORT: Regex = Regex::new(r"^(use |import |from |require\(|#include)").unwrap();
        static ref SIG: Regex = Regex::new(r"^(pub\s+)?(async\s+)?(fn|def|function|func|class|struct|enum|trait|interface|type)\s+\w+").unwrap();
    }

    for line in &lines {
        let trimmed = line.trim();
        let important = SIG.is_match(trimmed) || IMPORT.is_match(trimmed) ||
            trimmed.starts_with("pub ") || trimmed.starts_with("export ") ||
            trimmed == "}" || trimmed == "{";

        if important || kept < max_lines / 2 {
            result.push((*line).to_string());
            kept += 1;
        }
        if kept >= max_lines - 1 { break; }
    }

    result.push(format!("[{} more lines]", lines.len() - kept));
    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_level_parsing() {
        assert_eq!(FilterLevel::from_str("none").unwrap(), FilterLevel::None);
        assert_eq!(FilterLevel::from_str("minimal").unwrap(), FilterLevel::Minimal);
        assert_eq!(FilterLevel::from_str("aggressive").unwrap(), FilterLevel::Aggressive);
    }

    #[test]
    fn test_minimal_filter() {
        let code = "// comment\nfn main() {}\n";
        let result = filter_minimal(code, &Language::Rust);
        assert!(!result.contains("// comment"));
        assert!(result.contains("fn main()"));
    }

    #[test]
    fn test_json_preserved() {
        let json = r#"{"packages/*": "test"}"#;
        let result = filter_minimal(json, &Language::Data);
        assert!(result.contains("packages/*"));
    }
}
