// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! HEDL-native declarative filter engine.
//!
//! Filter definitions are stored in HEDL format (not TOML), deeply integrated
//! with the HEDL ecosystem. Uses hedl-core parser for robust parsing.

use hedl_core::{Document, Item, MatrixList, Node, Value};
use once_cell::sync::Lazy;
use regex::{Regex, RegexSet};
use std::collections::BTreeMap;

const BUILTIN_HEDL: &str = include_str!(concat!(env!("OUT_DIR"), "/builtin_filters.hedl"));

#[derive(Debug)]
struct CompiledMatchOutputRule {
    pattern: Regex,
    message: String,
    unless: Option<Regex>,
}

#[derive(Debug)]
struct CompiledReplaceRule {
    pattern: Regex,
    replacement: String,
}

#[derive(Debug)]
enum LineFilter {
    None,
    Strip(RegexSet),
    Keep(RegexSet),
}

#[derive(Debug)]
pub struct CompiledFilter {
    pub name: String,
    pub description: Option<String>,
    match_regex: Regex,
    strip_ansi: bool,
    replace: Vec<CompiledReplaceRule>,
    match_output: Vec<CompiledMatchOutputRule>,
    line_filter: LineFilter,
    truncate_lines_at: Option<usize>,
    head_lines: Option<usize>,
    tail_lines: Option<usize>,
    pub max_lines: Option<usize>,
    on_empty: Option<String>,
    pub filter_stderr: bool,
}

pub struct FilterRegistry {
    pub filters: Vec<CompiledFilter>,
}

impl FilterRegistry {
    fn load() -> Self {
        let mut filters = Vec::new();

        // Priority 1: project-local .hedl/filters.hedl
        let project_path = std::path::Path::new(".hedl/filters.hedl");
        if project_path.exists() {
            if let Ok(content) = std::fs::read_to_string(project_path) {
                match Self::parse_and_compile(&content, "project") {
                    Ok(f) => filters.extend(f),
                    Err(e) => eprintln!("[hedl-filter] warning: .hedl/filters.hedl: {}", e),
                }
            }
        }

        // Priority 2: user-global
        if let Some(config_dir) = dirs::config_dir() {
            let global_path = config_dir.join("hedl").join("filters.hedl");
            if let Ok(content) = std::fs::read_to_string(&global_path) {
                match Self::parse_and_compile(&content, "user-global") {
                    Ok(f) => filters.extend(f),
                    Err(e) => eprintln!("[hedl-filter] warning: {}: {}", global_path.display(), e),
                }
            }
        }

        // Priority 3: built-in
        match Self::parse_and_compile(BUILTIN_HEDL, "builtin") {
            Ok(f) => filters.extend(f),
            Err(e) => eprintln!("[hedl-filter] warning: builtin filters: {}", e),
        }

        FilterRegistry { filters }
    }

    pub fn parse_and_compile(content: &str, source: &str) -> Result<Vec<CompiledFilter>, String> {
        let doc = hedl_core::parse(content.as_bytes())
            .map_err(|e| format!("HEDL parse error in {}: {:?}", source, e))?;

        let mut defs = BTreeMap::new();

        // Parse filters
        if let Some(filters_list) = get_matrix_list(&doc, "filters") {
            for node in &filters_list.rows {
                let name = node.id.clone();
                let def = parse_filter_def(node)?;
                defs.insert(name, def);
            }
        }

        // Parse line rules
        if let Some(rules_list) = get_matrix_list(&doc, "line_rules") {
            for node in &rules_list.rows {
                let filter_name = get_string_field(node, 1).unwrap_or_default();
                let action = get_string_field(node, 2).unwrap_or_default();
                let pattern = get_string_field(node, 3).unwrap_or_default();
                if let Some(def) = defs.get_mut(&filter_name) {
                    match action.as_str() {
                        "strip" => def.strip_lines_matching.push(pattern),
                        "keep" => def.keep_lines_matching.push(pattern),
                        _ => {}
                    }
                }
            }
        }

        // Parse replace rules
        if let Some(rules_list) = get_matrix_list(&doc, "replace_rules") {
            for node in &rules_list.rows {
                let filter_name = get_string_field(node, 1).unwrap_or_default();
                let pattern = get_string_field(node, 2).unwrap_or_default();
                let replacement = get_string_field(node, 3).unwrap_or_default();
                if let Some(def) = defs.get_mut(&filter_name) {
                    def.replace.push(RawReplaceRule { pattern, replacement });
                }
            }
        }

        // Parse match rules
        if let Some(rules_list) = get_matrix_list(&doc, "match_rules") {
            for node in &rules_list.rows {
                let filter_name = get_string_field(node, 1).unwrap_or_default();
                let pattern = get_string_field(node, 2).unwrap_or_default();
                let message = get_string_field(node, 3).unwrap_or_default();
                let unless = get_string_field(node, 4);
                if let Some(def) = defs.get_mut(&filter_name) {
                    def.match_output.push(RawMatchOutputRule { pattern, message, unless });
                }
            }
        }

        // Compile all filters
        let mut compiled = Vec::new();
        for (name, def) in defs {
            match compile_filter(name.clone(), def) {
                Ok(f) => compiled.push(f),
                Err(e) => eprintln!("[hedl-filter] warning: filter '{}' in {}: {}", name, source, e),
            }
        }
        Ok(compiled)
    }
}

// Temporary structs for collecting raw data before compilation
struct RawReplaceRule {
    pattern: String,
    replacement: String,
}

struct RawMatchOutputRule {
    pattern: String,
    message: String,
    unless: Option<String>,
}

struct FilterDef {
    description: Option<String>,
    match_command: String,
    strip_ansi: bool,
    replace: Vec<RawReplaceRule>,
    match_output: Vec<RawMatchOutputRule>,
    strip_lines_matching: Vec<String>,
    keep_lines_matching: Vec<String>,
    truncate_lines_at: Option<usize>,
    head_lines: Option<usize>,
    tail_lines: Option<usize>,
    max_lines: Option<usize>,
    on_empty: Option<String>,
    filter_stderr: bool,
}

fn parse_filter_def(node: &Node) -> Result<FilterDef, String> {
    // Note: HEDL stores the id as both node.id AND as node.fields[0]
    let description = get_string_field(node, 1);
    let match_command = get_string_field(node, 2).ok_or("missing match_command")?;
    let strip_ansi = get_bool_field(node, 3).unwrap_or(false);
    let truncate_lines_at = get_usize_field(node, 4);
    let head_lines = get_usize_field(node, 5);
    let tail_lines = get_usize_field(node, 6);
    let max_lines = get_usize_field(node, 7);
    let on_empty = get_string_field(node, 8);
    let filter_stderr = get_bool_field(node, 9).unwrap_or(false);

    Ok(FilterDef {
        description,
        match_command,
        strip_ansi,
        replace: Vec::new(),
        match_output: Vec::new(),
        strip_lines_matching: Vec::new(),
        keep_lines_matching: Vec::new(),
        truncate_lines_at,
        head_lines,
        tail_lines,
        max_lines,
        on_empty,
        filter_stderr,
    })
}

fn get_matrix_list<'a>(doc: &'a Document, key: &str) -> Option<&'a MatrixList> {
    doc.root.get(key).and_then(|item| match item {
        Item::List(list) => Some(list),
        _ => None,
    })
}

fn get_string_field(node: &Node, index: usize) -> Option<String> {
    node.fields.get(index).and_then(|v| match v {
        Value::String(s) => Some(s.to_string()),
        _ => None,
    })
}

fn get_bool_field(node: &Node, index: usize) -> Option<bool> {
    node.fields.get(index).and_then(|v| match v {
        Value::Bool(b) => Some(*b),
        Value::String(s) if s.as_ref() == "true" => Some(true),
        Value::String(s) if s.as_ref() == "false" => Some(false),
        _ => None,
    })
}

fn get_usize_field(node: &Node, index: usize) -> Option<usize> {
    node.fields.get(index).and_then(|v| match v {
        Value::Int(i) => Some(*i as usize),
        Value::String(s) => s.parse().ok(),
        _ => None,
    })
}

fn compile_filter(name: String, def: FilterDef) -> Result<CompiledFilter, String> {
    if !def.strip_lines_matching.is_empty() && !def.keep_lines_matching.is_empty() {
        return Err("strip_lines_matching and keep_lines_matching are mutually exclusive".into());
    }

    let match_regex = Regex::new(&def.match_command)
        .map_err(|e| format!("invalid match_command regex: {}", e))?;

    let replace = def.replace.into_iter().map(|r| {
        let pat = r.pattern.clone();
        Regex::new(&r.pattern)
            .map(|pattern| CompiledReplaceRule { pattern, replacement: r.replacement })
            .map_err(|e| format!("invalid replace pattern '{}': {}", pat, e))
    }).collect::<Result<Vec<_>, _>>()?;

    let match_output = def.match_output.into_iter().map(|r| -> Result<CompiledMatchOutputRule, String> {
        let pat = r.pattern.clone();
        let pattern = Regex::new(&r.pattern)
            .map_err(|e| format!("invalid match_output pattern '{}': {}", pat, e))?;
        let unless = r.unless.as_deref().map(|u| {
            Regex::new(u).map_err(|e| format!("invalid match_output unless pattern '{}': {}", u, e))
        }).transpose().map_err(|e: String| e)?;
        Ok(CompiledMatchOutputRule { pattern, message: r.message, unless })
    }).collect::<Result<Vec<_>, _>>()?;

    let line_filter = if !def.strip_lines_matching.is_empty() {
        let set = RegexSet::new(&def.strip_lines_matching)
            .map_err(|e| format!("invalid strip_lines_matching regex: {}", e))?;
        LineFilter::Strip(set)
    } else if !def.keep_lines_matching.is_empty() {
        let set = RegexSet::new(&def.keep_lines_matching)
            .map_err(|e| format!("invalid keep_lines_matching regex: {}", e))?;
        LineFilter::Keep(set)
    } else {
        LineFilter::None
    };

    Ok(CompiledFilter {
        name, description: def.description, match_regex, strip_ansi: def.strip_ansi, replace,
        match_output, line_filter, truncate_lines_at: def.truncate_lines_at,
        head_lines: def.head_lines, tail_lines: def.tail_lines,
        max_lines: def.max_lines, on_empty: def.on_empty,
        filter_stderr: def.filter_stderr,
    })
}

static REGISTRY: Lazy<FilterRegistry> = Lazy::new(FilterRegistry::load);

pub fn find_filter_in<'a>(command: &str, filters: &'a [CompiledFilter]) -> Option<&'a CompiledFilter> {
    filters.iter().find(|f| f.match_regex.is_match(command))
}

pub fn apply_toml_filter(filter: &CompiledFilter, stdout: &str) -> String {
    let mut lines: Vec<String> = stdout.lines().map(String::from).collect();

    if filter.strip_ansi {
        lines = lines.into_iter().map(|l| crate::utils::strip_ansi(&l)).collect();
    }

    if !filter.replace.is_empty() {
        lines = lines.into_iter().map(|mut line| {
            for rule in &filter.replace {
                line = rule.pattern.replace_all(&line, rule.replacement.as_str()).into_owned();
            }
            line
        }).collect();
    }

    if !filter.match_output.is_empty() {
        let blob = lines.join("\n");
        for rule in &filter.match_output {
            if rule.pattern.is_match(&blob) {
                if let Some(ref unless_re) = rule.unless {
                    if unless_re.is_match(&blob) { continue; }
                }
                return rule.message.clone();
            }
        }
    }

    match &filter.line_filter {
        LineFilter::Strip(set) => lines.retain(|l| !set.is_match(l)),
        LineFilter::Keep(set) => lines.retain(|l| set.is_match(l)),
        LineFilter::None => {}
    }

    if let Some(max_chars) = filter.truncate_lines_at {
        lines = lines.into_iter().map(|l| crate::utils::truncate(&l, max_chars)).collect();
    }

    let total = lines.len();
    if let (Some(head), Some(tail)) = (filter.head_lines, filter.tail_lines) {
        if total > head + tail {
            let mut result = lines[..head].to_vec();
            result.push(format!("... ({} lines omitted)", total - head - tail));
            result.extend_from_slice(&lines[total - tail..]);
            lines = result;
        }
    } else if let Some(head) = filter.head_lines {
        if total > head {
            lines.truncate(head);
            lines.push(format!("... ({} lines omitted)", total - head));
        }
    } else if let Some(tail) = filter.tail_lines {
        if total > tail {
            let omitted = total - tail;
            lines = lines[omitted..].to_vec();
            lines.insert(0, format!("... ({} lines omitted)", omitted));
        }
    }

    if let Some(max) = filter.max_lines {
        if lines.len() > max {
            let truncated = lines.len() - max;
            lines.truncate(max);
            lines.push(format!("... ({} lines truncated)", truncated));
        }
    }

    let result = lines.join("\n");
    if result.trim().is_empty() {
        if let Some(ref msg) = filter.on_empty {
            return msg.clone();
        }
    }

    result
}

pub fn find_matching_filter(command: &str) -> Option<&'static CompiledFilter> {
    find_filter_in(command, &REGISTRY.filters)
}

pub struct TestOutcome {
    pub filter_name: String,
    pub test_name: String,
    pub passed: bool,
    pub actual: String,
    pub expected: String,
}

pub struct VerifyResults {
    pub outcomes: Vec<TestOutcome>,
    pub filters_without_tests: Vec<String>,
}

pub fn run_filter_tests(filter_name_opt: Option<&str>) -> VerifyResults {
    let mut outcomes = Vec::new();
    let mut all_names: Vec<String> = Vec::new();
    let mut tested: std::collections::HashSet<String> = std::collections::HashSet::new();

    collect_test_outcomes(BUILTIN_HEDL, filter_name_opt, &mut outcomes, &mut all_names, &mut tested);

    let project_path = std::path::Path::new(".hedl/filters.hedl");
    if project_path.exists() {
        if let Ok(content) = std::fs::read_to_string(project_path) {
            collect_test_outcomes(&content, filter_name_opt, &mut outcomes, &mut all_names, &mut tested);
        }
    }

    let filters_without_tests = all_names.into_iter()
        .filter(|name| filter_name_opt.is_none() || filter_name_opt == Some(name.as_str()))
        .filter(|name| !tested.contains(name))
        .collect();

    VerifyResults { outcomes, filters_without_tests }
}

fn collect_test_outcomes(
    content: &str,
    filter_name_opt: Option<&str>,
    outcomes: &mut Vec<TestOutcome>,
    all_names: &mut Vec<String>,
    tested: &mut std::collections::HashSet<String>,
) {
    let filters = match FilterRegistry::parse_and_compile(content, "test") {
        Ok(f) => f,
        Err(e) => { eprintln!("[hedl-filter] warning: HEDL parse error during verify: {}", e); return; }
    };

    let mut compiled: BTreeMap<String, CompiledFilter> = BTreeMap::new();
    for f in filters {
        all_names.push(f.name.clone());
        compiled.insert(f.name.clone(), f);
    }

    // Parse tests from HEDL
    let doc = match hedl_core::parse(content.as_bytes()) {
        Ok(d) => d,
        Err(_) => return,
    };

    if let Some(tests_list) = get_matrix_list(&doc, "tests") {
        for node in &tests_list.rows {
            let filter_name = get_string_field(node, 1).unwrap_or_default();
            if let Some(name) = filter_name_opt {
                if filter_name != name { continue; }
            }
            tested.insert(filter_name.clone());

            let compiled_filter = match compiled.get(&filter_name) {
                Some(f) => f,
                None => {
                    eprintln!("[hedl-filter] warning: test references unknown filter '{}'", filter_name);
                    continue;
                }
            };

            let test_name = get_string_field(node, 2).unwrap_or_default();
            let input = get_string_field(node, 3).unwrap_or_default();
            let expected = get_string_field(node, 4).unwrap_or_default();

            let actual = apply_toml_filter(compiled_filter, &input);
            let actual_cmp = actual.trim_end_matches('\n').to_string();
            let expected_cmp = expected.trim_end_matches('\n').to_string();
            outcomes.push(TestOutcome {
                filter_name: filter_name.clone(),
                test_name,
                passed: actual_cmp == expected_cmp,
                actual: actual_cmp,
                expected: expected_cmp,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_filters(hedl: &str) -> Vec<CompiledFilter> {
        FilterRegistry::parse_and_compile(hedl, "test").expect("test HEDL should be valid")
    }

    fn first_filter(hedl: &str) -> CompiledFilter {
        make_filters(hedl).into_iter().next().expect("expected at least one filter")
    }

    #[test]
    fn test_strip_ansi() {
        let f = first_filter(r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Filter:[name, description, match_command, strip_ansi, truncate_lines_at, head_lines, tail_lines, max_lines, on_empty, filter_stderr]
---
filters: @Filter
 |f,,^cmd,true,,,,,,false"#);
        assert!(f.strip_ansi, "strip_ansi should be true");
        let out = apply_toml_filter(&f, "\x1b[31mError\x1b[0m\nnormal");
        assert_eq!(out, "Error\nnormal");
    }

    #[test]
    fn test_strip_lines() {
        let f = first_filter(r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Filter:[name, description, match_command, strip_ansi, truncate_lines_at, head_lines, tail_lines, max_lines, on_empty, filter_stderr]
%S:LineRule:[id, filter_name, action, pattern]
---
filters: @Filter
 |f,,^cmd,false,,,,,,false
line_rules: @LineRule
 |lr1,f,strip,^noise
 |lr2,f,strip,^verbose"#);
        let out = apply_toml_filter(&f, "noise\nkeep\nverbose\nalso");
        assert_eq!(out, "keep\nalso");
    }

    #[test]
    fn test_match_output() {
        let f = first_filter(r#"%V:2.0
%NULL:~
%QUOTE:"
%S:Filter:[name, description, match_command, strip_ansi, truncate_lines_at, head_lines, tail_lines, max_lines, on_empty, filter_stderr]
%S:MatchRule:[id, filter_name, pattern, message, unless]
---
filters: @Filter
 |f,,^cmd,false,,,,,,false
match_rules: @MatchRule
 |mr1,f,Switched,ok,~"#);
        assert_eq!(apply_toml_filter(&f, "Switched to main"), "ok");
    }

    #[test]
    fn test_builtin_compile() {
        let result = FilterRegistry::parse_and_compile(BUILTIN_HEDL, "builtin");
        assert!(result.is_ok(), "builtin filters failed to compile: {:?}", result);
        assert!(!result.unwrap().is_empty());
    }
}
