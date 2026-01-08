// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Header directive parsing for HEDL.

use crate::error::HedlResult;
use crate::errors::messages;
use crate::lex::{is_valid_key_token, is_valid_type_name, strip_comment};
use crate::limits::Limits;
use std::collections::BTreeMap;

/// Parsed header data.
#[derive(Debug, Clone)]
pub struct Header {
    pub version: (u32, u32),
    pub aliases: BTreeMap<String, String>,
    pub structs: BTreeMap<String, Vec<String>>,
    pub nests: BTreeMap<String, String>,
    /// Struct instance counts from count hints (e.g., `users(5): @User`).
    /// Reserved for validation and optimization features.
    #[allow(dead_code)]
    pub struct_counts: BTreeMap<String, usize>,
    /// Line number after the separator (where body starts).
    #[allow(dead_code)]
    pub body_start_line: usize,
}

/// Parse the header section from preprocessed lines.
///
/// Returns the header data and the index where the body starts.
pub fn parse_header(lines: &[(usize, &str)], limits: &Limits) -> HedlResult<(Header, usize)> {
    let mut version: Option<(u32, u32)> = None;
    let mut aliases: BTreeMap<String, String> = BTreeMap::new();
    let mut structs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut nests: BTreeMap<String, String> = BTreeMap::new();
    let mut struct_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut first_directive = true;

    for (idx, &(line_num, line)) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check for separator
        if trimmed == "---" || trimmed.starts_with("--- ") || trimmed.starts_with("---#") {
            // Validate separator has no leading spaces
            if line.starts_with(' ') || line.starts_with('\t') {
                return Err(messages::invalid_separator_whitespace(line_num));
            }

            if version.is_none() {
                return Err(messages::missing_version_before_separator(line_num));
            }

            return Ok((
                Header {
                    version: version.unwrap(),
                    aliases,
                    structs,
                    nests,
                    struct_counts,
                    body_start_line: line_num + 1,
                },
                idx + 1,
            ));
        }

        // Skip blank and comment lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Must be a directive
        if !trimmed.starts_with('%') {
            return Err(messages::expected_directive(trimmed, line_num));
        }

        // Parse directive
        let colon_pos = trimmed
            .find(':')
            .ok_or_else(|| messages::directive_missing_colon(line_num))?;

        let directive_name = &trimmed[..colon_pos];
        let rest = &trimmed[colon_pos + 1..];

        // Must have space after colon
        if !rest.starts_with(' ') {
            return Err(messages::directive_missing_space_after_colon(line_num));
        }

        let payload = strip_comment(rest.trim_start());

        match directive_name {
            "%VERSION" => {
                if !first_directive {
                    return Err(messages::version_not_first(line_num));
                }
                version = Some(parse_version(payload, line_num)?);
            }
            "%STRUCT" => {
                let (type_name, columns, count) = parse_struct(payload, line_num, limits)?;
                if let Some(existing) = structs.get(&type_name) {
                    if existing != &columns {
                        return Err(messages::struct_redefined(&type_name, line_num));
                    }
                } else {
                    structs.insert(type_name.clone(), columns);
                }
                if let Some(count_val) = count {
                    struct_counts.insert(type_name, count_val);
                }
            }
            "%ALIAS" => {
                let (key, value) = parse_alias(payload, line_num)?;
                if aliases.contains_key(&key) {
                    return Err(messages::alias_already_defined(&key, line_num));
                }
                if aliases.len() >= limits.max_aliases {
                    return Err(messages::too_many_aliases(aliases.len(), limits.max_aliases, line_num));
                }
                aliases.insert(key, value);
            }
            "%NEST" => {
                let (parent, child) = parse_nest(payload, line_num, &structs)?;
                if nests.contains_key(&parent) {
                    return Err(messages::nest_multiple_rules(&parent, line_num));
                }
                nests.insert(parent, child);
            }
            _ => {
                return Err(messages::unknown_directive(directive_name, line_num));
            }
        }

        first_directive = false;
    }

    Err(messages::missing_separator(
        lines.last().map(|(n, _)| *n).unwrap_or(1),
    ))
}

fn parse_version(payload: &str, line_num: usize) -> HedlResult<(u32, u32)> {
    let parts: Vec<&str> = payload.split('.').collect();
    if parts.len() != 2 {
        return Err(messages::invalid_version_format(payload, line_num));
    }

    let major: u32 = parts[0].parse().map_err(|_| {
        messages::invalid_major_version(parts[0], line_num)
    })?;
    let minor: u32 = parts[1].parse().map_err(|_| {
        messages::invalid_minor_version(parts[1], line_num)
    })?;

    // Check for leading zeros
    if (parts[0].len() > 1 && parts[0].starts_with('0'))
        || (parts[1].len() > 1 && parts[1].starts_with('0'))
    {
        return Err(messages::version_leading_zeros(line_num));
    }

    Ok((major, minor))
}

fn parse_struct(
    payload: &str,
    line_num: usize,
    limits: &Limits,
) -> HedlResult<(String, Vec<String>, Option<usize>)> {
    // Format: TypeName: [col1, col2, ...] OR TypeName (N): [col1, col2, ...]
    let colon_pos = payload.find(':').ok_or_else(|| {
        messages::struct_missing_colon(line_num)
    })?;

    let before_colon = payload[..colon_pos].trim();

    // Check for optional count syntax: TypeName (N)
    let (type_name, count) = if let Some(paren_start) = before_colon.rfind('(') {
        // Found opening parenthesis - try to parse count
        let type_part = before_colon[..paren_start].trim();
        let count_part = &before_colon[paren_start + 1..];

        if let Some(paren_end) = count_part.find(')') {
            let count_str = count_part[..paren_end].trim();
            let remaining = count_part[paren_end + 1..].trim();

            // Ensure nothing after closing parenthesis
            if !remaining.is_empty() {
                return Err(messages::struct_count_unexpected_content(remaining, line_num));
            }

            // Parse count as usize
            let count_val: usize = count_str.parse().map_err(|_| {
                messages::struct_count_invalid(count_str, line_num)
            })?;

            // Check for leading zeros
            if count_str.len() > 1 && count_str.starts_with('0') {
                return Err(messages::struct_count_leading_zeros(line_num));
            }

            (type_part, Some(count_val))
        } else {
            // Opening parenthesis without closing - treat as part of type name (will fail validation)
            (before_colon, None)
        }
    } else {
        // No parenthesis found - no count specified
        (before_colon, None)
    };

    if !is_valid_type_name(type_name) {
        return Err(messages::invalid_type_name(type_name, line_num));
    }

    let columns_str = payload[colon_pos + 1..].trim();
    let columns = parse_column_list(columns_str, line_num, limits)?;

    Ok((type_name.to_string(), columns, count))
}

fn parse_column_list(s: &str, line_num: usize, limits: &Limits) -> HedlResult<Vec<String>> {
    let s = s.trim();
    if !s.starts_with('[') || !s.ends_with(']') {
        return Err(messages::column_list_not_bracketed(line_num));
    }

    let inner = &s[1..s.len() - 1];
    if inner.trim().is_empty() {
        return Err(messages::column_list_empty(line_num));
    }

    let mut columns = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for part in inner.split(',') {
        let col = part.trim();
        if col.is_empty() {
            continue;
        }

        if !is_valid_key_token(col) {
            return Err(messages::invalid_column_name(col, line_num));
        }

        if !seen.insert(col) {
            return Err(messages::duplicate_column_name(col, line_num));
        }

        columns.push(col.to_string());
    }

    if columns.is_empty() {
        return Err(messages::column_list_empty(line_num));
    }

    if columns.len() > limits.max_columns {
        return Err(messages::too_many_columns(columns.len(), limits.max_columns, line_num));
    }

    Ok(columns)
}

fn parse_alias(payload: &str, line_num: usize) -> HedlResult<(String, String)> {
    // Format: %key: "value"
    let colon_pos = payload
        .find(':')
        .ok_or_else(|| messages::alias_missing_colon(line_num))?;

    let key_part = payload[..colon_pos].trim();
    if !key_part.starts_with('%') {
        return Err(messages::alias_key_missing_percent(line_num));
    }

    let key = &key_part[1..];
    if !is_valid_key_token(key) {
        return Err(messages::invalid_alias_key(key, line_num));
    }

    let value_part = payload[colon_pos + 1..].trim();
    if !value_part.starts_with('"') || !value_part.ends_with('"') {
        return Err(messages::alias_value_not_quoted(line_num));
    }

    // Parse quoted string (handle "" escapes)
    let inner = &value_part[1..value_part.len() - 1];
    let value = inner.replace("\"\"", "\"");

    Ok((key.to_string(), value))
}

fn parse_nest(
    payload: &str,
    line_num: usize,
    structs: &BTreeMap<String, Vec<String>>,
) -> HedlResult<(String, String)> {
    // Format: ParentType > ChildType
    let parts: Vec<&str> = payload.split('>').collect();
    if parts.len() != 2 {
        return Err(messages::nest_invalid_syntax(line_num));
    }

    let parent = parts[0].trim();
    let child = parts[1].trim();

    if !is_valid_type_name(parent) {
        return Err(messages::nest_invalid_parent_type(parent, line_num));
    }

    if !is_valid_type_name(child) {
        return Err(messages::nest_invalid_child_type(child, line_num));
    }

    if !structs.contains_key(parent) {
        return Err(messages::nest_parent_not_defined(parent, line_num));
    }

    if !structs.contains_key(child) {
        return Err(messages::nest_child_not_defined(child, line_num));
    }

    Ok((parent.to_string(), child.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lines(s: &str) -> Vec<(usize, &str)> {
        s.lines().enumerate().map(|(i, l)| (i + 1, l)).collect()
    }

    fn default_limits() -> Limits {
        Limits::default()
    }

    // ==================== Minimal header tests ====================

    #[test]
    fn test_parse_minimal_header() {
        let input = "%VERSION: 1.0\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.version, (1, 0));
    }

    #[test]
    fn test_header_returns_body_start_index() {
        let input = "%VERSION: 1.0\n---";
        let lines = make_lines(input);
        let (_, body_idx) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(body_idx, 2); // Index after separator
    }

    #[test]
    fn test_header_with_comment() {
        let input = "%VERSION: 1.0\n# This is a comment\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.version, (1, 0));
    }

    #[test]
    fn test_header_with_blank_lines() {
        let input = "%VERSION: 1.0\n\n  \n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.version, (1, 0));
    }

    #[test]
    fn test_separator_with_comment() {
        let input = "%VERSION: 1.0\n---# comment after separator";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.version, (1, 0));
    }

    #[test]
    fn test_separator_with_space_comment() {
        let input = "%VERSION: 1.0\n--- # comment";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.version, (1, 0));
    }

    // ==================== %VERSION tests ====================

    #[test]
    fn test_version_zero_zero() {
        let input = "%VERSION: 0.0\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.version, (0, 0));
    }

    #[test]
    fn test_version_high_numbers() {
        let input = "%VERSION: 999.999\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.version, (999, 999));
    }

    #[test]
    fn test_version_leading_zero_error() {
        let input = "%VERSION: 01.0\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("leading zeros"));
    }

    #[test]
    fn test_version_minor_leading_zero_error() {
        let input = "%VERSION: 1.01\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
    }

    #[test]
    fn test_version_invalid_format_error() {
        let input = "%VERSION: 1\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("invalid version format"));
    }

    #[test]
    fn test_version_three_parts_error() {
        let input = "%VERSION: 1.0.0\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
    }

    #[test]
    fn test_version_non_numeric_error() {
        let input = "%VERSION: a.b\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid major"));
    }

    #[test]
    fn test_version_not_first_error() {
        let input = "%STRUCT: User: [id,name]\n%VERSION: 1.0\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("must be the first"));
    }

    // ==================== %STRUCT tests ====================

    #[test]
    fn test_parse_struct() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name,email]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(
            header.structs.get("User"),
            Some(&vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string()
            ])
        );
    }

    #[test]
    fn test_parse_struct_single_column() {
        let input = "%VERSION: 1.0\n%STRUCT: Point: [x]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.structs.get("Point"), Some(&vec!["x".to_string()]));
    }

    #[test]
    fn test_parse_multiple_structs() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Post: [id,title]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert!(header.structs.contains_key("User"));
        assert!(header.structs.contains_key("Post"));
    }

    #[test]
    fn test_struct_identical_redefinition_ok() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: User: [id,name]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_ok());
    }

    #[test]
    fn test_struct_different_redefinition_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: User: [id, email]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("redefined with different"));
    }

    #[test]
    fn test_struct_invalid_type_name_error() {
        let input = "%VERSION: 1.0\n%STRUCT: user: [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid type name"));
    }

    #[test]
    fn test_struct_invalid_column_name_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [Id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid column name"));
    }

    #[test]
    fn test_struct_duplicate_column_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id, name, id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("duplicate column"));
    }

    #[test]
    fn test_struct_empty_columns_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: []\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("cannot be empty"));
    }

    #[test]
    fn test_struct_missing_brackets_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: id, name\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("enclosed in []"));
    }

    #[test]
    fn test_struct_too_many_columns_error() {
        let limits = Limits {
            max_columns: 2,
            ..Limits::default()
        };
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name,email]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &limits);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("too many columns"));
    }

    // ==================== %ALIAS tests ====================

    #[test]
    fn test_parse_alias() {
        let input = "%VERSION: 1.0\n%ALIAS: %active: \"true\"\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.aliases.get("active"), Some(&"true".to_string()));
    }

    #[test]
    fn test_parse_alias_empty_value() {
        let input = "%VERSION: 1.0\n%ALIAS: %empty: \"\"\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.aliases.get("empty"), Some(&String::new()));
    }

    #[test]
    fn test_parse_alias_escaped_quotes() {
        let input = "%VERSION: 1.0\n%ALIAS: %quote: \"say \"\"hello\"\"\"\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(
            header.aliases.get("quote"),
            Some(&"say \"hello\"".to_string())
        );
    }

    #[test]
    fn test_parse_multiple_aliases() {
        let input = "%VERSION: 1.0\n%ALIAS: %a: \"1\"\n%ALIAS: %b: \"2\"\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.aliases.get("a"), Some(&"1".to_string()));
        assert_eq!(header.aliases.get("b"), Some(&"2".to_string()));
    }

    #[test]
    fn test_alias_duplicate_error() {
        let input = "%VERSION: 1.0\n%ALIAS: %key: \"a\"\n%ALIAS: %key: \"b\"\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("already defined"));
    }

    #[test]
    fn test_alias_missing_percent_error() {
        let input = "%VERSION: 1.0\n%ALIAS: key: \"value\"\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("must start with '%'"));
    }

    #[test]
    fn test_alias_unquoted_value_error() {
        let input = "%VERSION: 1.0\n%ALIAS: %key: value\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("quoted string"));
    }

    #[test]
    fn test_alias_too_many_error() {
        let limits = Limits {
            max_aliases: 1,
            ..Limits::default()
        };
        let input = "%VERSION: 1.0\n%ALIAS: %a: \"1\"\n%ALIAS: %b: \"2\"\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &limits);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("too many aliases"));
    }

    // ==================== %NEST tests ====================

    #[test]
    fn test_parse_nest() {
        let input =
            "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Post: [id,title]\n%NEST: User > Post\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.nests.get("User"), Some(&"Post".to_string()));
    }

    #[test]
    fn test_nest_undefined_parent_error() {
        let input = "%VERSION: 1.0\n%STRUCT: Post: [id,title]\n%NEST: User > Post\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not defined"));
    }

    #[test]
    fn test_nest_undefined_child_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%NEST: User > Post\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not defined"));
    }

    #[test]
    fn test_nest_multiple_for_parent_error() {
        let input = "%VERSION: 1.0\n%STRUCT: A: [id]\n%STRUCT: B: [id]\n%STRUCT: C: [id]\n%NEST: A > B\n%NEST: A > C\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("multiple NEST rules"));
    }

    #[test]
    fn test_nest_invalid_format_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%NEST: User\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Parent > Child"));
    }

    #[test]
    fn test_nest_invalid_parent_type_name_error() {
        let input =
            "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Post: [id,title]\n%NEST: user > Post\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid parent type"));
    }

    // ==================== General error cases ====================

    #[test]
    fn test_missing_version_error() {
        let input = "%STRUCT: User: [id,name]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("VERSION"));
    }

    #[test]
    fn test_missing_separator_error() {
        let input = "%VERSION: 1.0\na: 1";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        // Error is "expected directive starting with '%'" since 'a: 1' is not a directive
        assert!(result.unwrap_err().message.contains("directive"));
    }

    #[test]
    fn test_indented_separator_error() {
        let input = "%VERSION: 1.0\n  ---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("leading whitespace"));
    }

    #[test]
    fn test_unknown_directive_error() {
        let input = "%VERSION: 1.0\n%UNKNOWN: foo\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("unknown directive"));
    }

    #[test]
    fn test_directive_missing_colon_error() {
        let input = "%VERSION 1.0\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("missing ':'"));
    }

    #[test]
    fn test_directive_missing_space_after_colon_error() {
        let input = "%VERSION:1.0\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("followed by space"));
    }

    #[test]
    fn test_non_directive_in_header_error() {
        let input = "%VERSION: 1.0\nsome text\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("expected directive"));
    }

    // ==================== Header struct tests ====================

    #[test]
    fn test_header_clone() {
        let input = "%VERSION: 1.0\n%ALIAS: %x: \"1\"\n%STRUCT: User: [id,name]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        let cloned = header.clone();
        assert_eq!(cloned.version, header.version);
        assert_eq!(cloned.aliases, header.aliases);
        assert_eq!(cloned.structs, header.structs);
    }

    #[test]
    fn test_header_debug() {
        let input = "%VERSION: 1.0\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        let debug = format!("{:?}", header);
        assert!(debug.contains("version"));
        assert!(debug.contains("aliases"));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_empty_input() {
        let lines: Vec<(usize, &str)> = vec![];
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
    }

    #[test]
    fn test_comment_with_directive() {
        let input = "%VERSION: 1.0 # version comment\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.version, (1, 0));
    }

    #[test]
    fn test_struct_with_comment() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name] # columns\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert!(header.structs.contains_key("User"));
    }

    #[test]
    fn test_all_directives_combined() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n%STRUCT: Post: [id,title]\n%ALIAS: %active: \"true\"\n%NEST: User > Post\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.version, (1, 0));
        assert_eq!(header.structs.len(), 2);
        assert_eq!(header.aliases.len(), 1);
        assert_eq!(header.nests.len(), 1);
    }

    // ==================== %STRUCT with count tests ====================

    #[test]
    fn test_struct_with_count() {
        let input = "%VERSION: 1.0\n%STRUCT: Company (1): [id, name, founded, industry]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(
            header.structs.get("Company"),
            Some(&vec![
                "id".to_string(),
                "name".to_string(),
                "founded".to_string(),
                "industry".to_string()
            ])
        );
        assert_eq!(header.struct_counts.get("Company"), Some(&1));
    }

    #[test]
    fn test_struct_with_higher_count() {
        let input = "%VERSION: 1.0\n%STRUCT: Division (3): [id, name, head, budget]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(
            header.structs.get("Division"),
            Some(&vec![
                "id".to_string(),
                "name".to_string(),
                "head".to_string(),
                "budget".to_string()
            ])
        );
        assert_eq!(header.struct_counts.get("Division"), Some(&3));
    }

    #[test]
    fn test_struct_with_zero_count() {
        let input = "%VERSION: 1.0\n%STRUCT: Empty (0): [id]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.struct_counts.get("Empty"), Some(&0));
    }

    #[test]
    fn test_struct_without_count() {
        let input = "%VERSION: 1.0\n%STRUCT: User: [id,name]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(
            header.structs.get("User"),
            Some(&vec!["id".to_string(), "name".to_string()])
        );
        assert_eq!(header.struct_counts.get("User"), None);
    }

    #[test]
    fn test_struct_mixed_with_and_without_count() {
        let input =
            "%VERSION: 1.0\n%STRUCT: User (5): [id, name]\n%STRUCT: Post: [id,title]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.struct_counts.get("User"), Some(&5));
        assert_eq!(header.struct_counts.get("Post"), None);
    }

    #[test]
    fn test_struct_count_leading_zero_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User (01): [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("leading zeros"));
    }

    #[test]
    fn test_struct_count_invalid_number_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User (abc): [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid count"));
    }

    #[test]
    fn test_struct_count_negative_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User (-1): [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid count"));
    }

    #[test]
    fn test_struct_count_extra_content_after_paren_error() {
        let input = "%VERSION: 1.0\n%STRUCT: User (5) extra: [id]\n---";
        let lines = make_lines(input);
        let result = parse_header(&lines, &default_limits());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .message
            .contains("unexpected content after count"));
    }

    #[test]
    fn test_struct_count_whitespace_before_paren() {
        let input = "%VERSION: 1.0\n%STRUCT: Company (10): [id, name]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.struct_counts.get("Company"), Some(&10));
    }

    #[test]
    fn test_struct_count_whitespace_inside_paren() {
        let input = "%VERSION: 1.0\n%STRUCT: Company ( 10 ): [id, name]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.struct_counts.get("Company"), Some(&10));
    }

    #[test]
    fn test_struct_count_large_number() {
        let input = "%VERSION: 1.0\n%STRUCT: BigList (999999): [id]\n---";
        let lines = make_lines(input);
        let (header, _) = parse_header(&lines, &default_limits()).unwrap();
        assert_eq!(header.struct_counts.get("BigList"), Some(&999999));
    }
}
