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

//! Header parsing implementation for HEDL directives.

use super::types::{is_compact_syntax, normalize_directive_name, CountValue, Header, ParseMode};
use crate::error::HedlResult;
use crate::errors::messages;
use crate::lex::{is_valid_key_token, is_valid_type_name, strip_comment};
use crate::limits::{Limits, TimeoutContext};
use std::collections::BTreeMap;

/// Parse the header section from preprocessed lines.
///
/// Returns the header data and the index where the body starts.
///
/// Supports both verbose syntax (space after colon) and compact syntax:
/// - Verbose: `%VERSION: 2.0`, `%STRUCT: User: [id, name]`
/// - Compact: `%V:2.0`, `%S:User:[id,name]` (no spaces)
pub fn parse_header(
    lines: &[(usize, &str)],
    limits: &Limits,
    timeout_ctx: &TimeoutContext,
) -> HedlResult<(Header, usize)> {
    let mut version: Option<(u32, u32)> = None;
    let mut aliases: BTreeMap<String, String> = BTreeMap::new();
    let mut structs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut struct_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut nests: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut first_directive = true;

    let mut parsed_mode: Option<ParseMode> = None;
    let mut header_mode_set = false;
    let mut prompt: Option<Box<str>> = None;
    let mut null_char: char = '~';
    let mut quote_char: char = '"';
    let mut counts: BTreeMap<String, CountValue> = BTreeMap::new();
    let mut null_char_set = false;
    let mut quote_char_set = false;

    for (idx, &(line_num, line)) in lines.iter().enumerate() {
        // Periodic timeout check (every 10,000 iterations to minimize overhead)
        if idx % 10_000 == 0 {
            timeout_ctx.check_timeout(line_num)?;
        }

        let trimmed = line.trim();

        // Check for separator
        if trimmed == "---" || trimmed.starts_with("--- ") || trimmed.starts_with("---#") {
            // Validate separator has no leading spaces
            if line.starts_with(' ') || line.starts_with('\t') {
                return Err(messages::invalid_separator_whitespace(line_num));
            }

            let ver = match version {
                Some(v) => v,
                None => return Err(messages::missing_version_before_separator(line_num)),
            };

            // v2.0 requires %NULL and %QUOTE directives
            if ver >= (2, 0) {
                if !null_char_set {
                    return Err(messages::v20_null_required(line_num));
                }
                if !quote_char_set {
                    return Err(messages::v20_quote_required(line_num));
                }
            }

            return Ok((
                Header {
                    version: ver,
                    mode: parsed_mode.unwrap_or_default(),
                    aliases,
                    structs,
                    struct_counts,
                    nests,
                    prompt,
                    null_char,
                    quote_char,
                    counts,
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

        // Determine if using compact or verbose syntax
        let is_compact = is_compact_syntax(directive_name);

        // For verbose syntax, must have space after colon
        // For compact syntax, no space is required
        let payload = if is_compact {
            strip_comment(rest.trim_start())
        } else {
            if !rest.starts_with(' ') {
                return Err(messages::directive_missing_space_after_colon(line_num));
            }
            strip_comment(rest.trim_start())
        };

        // Normalize compact directive names to verbose for matching
        let normalized_name = normalize_directive_name(directive_name);

        match normalized_name {
            "%VERSION" => {
                if !first_directive {
                    return Err(messages::version_not_first(line_num));
                }
                let parsed_version = parse_version(payload, line_num)?;

                // v2.0 requires compact syntax - reject verbose %VERSION:
                if parsed_version >= (2, 0) && !is_compact {
                    return Err(messages::v20_verbose_syntax_not_allowed(
                        "%VERSION", line_num,
                    ));
                }

                version = Some(parsed_version);
            }
            "%STRUCT" => {
                // v2.0 requires compact syntax - reject verbose %STRUCT:
                if let Some(v) = version {
                    if v >= (2, 0) && !is_compact {
                        return Err(messages::v20_verbose_syntax_not_allowed(
                            "%STRUCT", line_num,
                        ));
                    }
                }

                let (type_name, columns, count) = parse_struct(payload, line_num, limits)?;
                if let Some(existing) = structs.get(&type_name) {
                    if existing != &columns {
                        return Err(messages::struct_redefined(&type_name, line_num));
                    }
                } else {
                    structs.insert(type_name.clone(), columns);
                    if let Some(c) = count {
                        struct_counts.insert(type_name, c);
                    }
                }
            }
            "%ALIAS" => {
                // v2.0 requires compact syntax - reject verbose %ALIAS:
                if let Some(v) = version {
                    if v >= (2, 0) && !is_compact {
                        return Err(messages::v20_verbose_syntax_not_allowed("%ALIAS", line_num));
                    }
                }

                let (key, value) = parse_alias(payload, line_num)?;
                if aliases.contains_key(&key) {
                    return Err(messages::alias_already_defined(&key, line_num));
                }
                if aliases.len() >= limits.max_aliases {
                    return Err(messages::too_many_aliases(
                        aliases.len(),
                        limits.max_aliases,
                        line_num,
                    ));
                }
                aliases.insert(key, value);
            }
            "%NEST" => {
                // v2.0 requires compact syntax - reject verbose %NEST:
                if let Some(v) = version {
                    if v >= (2, 0) && !is_compact {
                        return Err(messages::v20_verbose_syntax_not_allowed("%NEST", line_num));
                    }
                }

                let (parent, child) = parse_nest(payload, line_num, &structs)?;
                // Check for duplicate (parent, child) pairs
                if let Some(children) = nests.get(&parent) {
                    if children.contains(&child) {
                        return Err(messages::nest_duplicate_pair(&parent, &child, line_num));
                    }
                }
                nests.entry(parent).or_default().push(child);
            }
            "%MODE" => {
                let mode = parse_mode(payload, line_num)?;
                if header_mode_set {
                    return Err(messages::mode_already_defined(line_num));
                }
                parsed_mode = Some(mode);
                header_mode_set = true;
            }
            "%ENUM" | "%DICT" | "%CONSTRAINT" => {
                return Err(messages::removed_directive(directive_name, line_num));
            }
            "%PROMPT" => {
                if prompt.is_some() {
                    return Err(messages::prompt_already_defined(line_num));
                }
                prompt = Some(parse_prompt(payload, line_num)?);
            }
            "%NULL" => {
                if null_char_set {
                    return Err(messages::null_char_already_defined(line_num));
                }
                null_char = parse_null_directive(payload, line_num)?;
                null_char_set = true;
            }
            "%QUOTE" => {
                if quote_char_set {
                    return Err(messages::quote_char_already_defined(line_num));
                }
                quote_char = parse_quote_directive(payload, line_num)?;
                quote_char_set = true;
            }
            "%COUNT" => {
                let (key, value) = parse_count_directive(payload, line_num)?;
                counts.insert(key, value);
            }
            _ if directive_name.starts_with("%X-") => {
                // Experimental directives: warn and skip
                eprintln!(
                    "Warning: Ignoring experimental directive {} at line {}",
                    directive_name, line_num
                );
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

    let major: u32 = parts[0]
        .parse()
        .map_err(|_| messages::invalid_major_version(parts[0], line_num))?;
    let minor: u32 = parts[1]
        .parse()
        .map_err(|_| messages::invalid_minor_version(parts[1], line_num))?;

    // Check for leading zeros
    if (parts[0].len() > 1 && parts[0].starts_with('0'))
        || (parts[1].len() > 1 && parts[1].starts_with('0'))
    {
        return Err(messages::version_leading_zeros(line_num));
    }

    Ok((major, minor))
}

/// Parse a struct definition.
///
/// Validates the type name, column list, and optional count hint syntax.
/// Returns the type name, columns, and optional count.
fn parse_struct(
    payload: &str,
    line_num: usize,
    limits: &Limits,
) -> HedlResult<(String, Vec<String>, Option<usize>)> {
    // Format: TypeName: [col1, col2, ...] OR TypeName (N): [col1, col2, ...]
    let colon_pos = payload
        .find(':')
        .ok_or_else(|| messages::struct_missing_colon(line_num))?;

    let before_colon = payload[..colon_pos].trim();

    // Parse optional count hint using shared parser
    let (type_name, count) = {
        use crate::lex::count_hint::{parse_parenthesized_count, CountParsingConfig};
        parse_parenthesized_count(before_colon, CountParsingConfig::STRUCT_HINT, line_num)?
    };

    if !is_valid_type_name(&type_name) {
        return Err(messages::invalid_type_name(&type_name, line_num));
    }

    let columns_str = payload[colon_pos + 1..].trim();
    let columns = parse_column_list(columns_str, line_num, limits)?;

    Ok((type_name.to_string(), columns, count))
}

fn parse_column_list(s: &str, line_num: usize, limits: &Limits) -> HedlResult<Vec<String>> {
    let trimmed = s.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(messages::column_list_not_bracketed(line_num));
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Err(messages::column_list_empty(line_num));
    }

    // Pre-allocate based on comma count (bounded by max_columns)
    let estimated_count = inner.matches(',').count() + 1;
    let mut columns = Vec::with_capacity(estimated_count.min(limits.max_columns));
    let mut seen =
        std::collections::HashSet::with_capacity(estimated_count.min(limits.max_columns));

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
        return Err(messages::too_many_columns(
            columns.len(),
            limits.max_columns,
            line_num,
        ));
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

/// Parse %MODE directive: `strict` or `lenient`
fn parse_mode(payload: &str, line_num: usize) -> HedlResult<ParseMode> {
    match payload.trim().to_lowercase().as_str() {
        "strict" => Ok(ParseMode::Strict),
        "lenient" => Ok(ParseMode::Lenient),
        other => Err(messages::invalid_mode(other, line_num)),
    }
}

/// Parse %PROMPT directive: quoted string
fn parse_prompt(payload: &str, line_num: usize) -> HedlResult<Box<str>> {
    let trimmed = payload.trim();
    if !trimmed.starts_with('"') || !trimmed.ends_with('"') {
        return Err(messages::prompt_not_quoted(line_num));
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    // Use Box::from() to avoid intermediate String allocation
    Ok(Box::from(inner))
}

/// Parse %NULL directive: single character for null representation.
///
/// Syntax: `%NULL:~` or `%NULL: ~`
fn parse_null_directive(payload: &str, line_num: usize) -> HedlResult<char> {
    let trimmed = payload.trim();
    let mut chars = trimmed.chars();

    let ch = chars
        .next()
        .ok_or_else(|| messages::null_char_empty(line_num))?;

    // Check for trailing content (only single char allowed)
    if chars.next().is_some() {
        return Err(messages::null_char_multiple(line_num));
    }

    Ok(ch)
}

/// Parse %QUOTE directive: single character for quote representation.
///
/// Syntax: `%QUOTE:"` or `%QUOTE: "`
fn parse_quote_directive(payload: &str, line_num: usize) -> HedlResult<char> {
    let trimmed = payload.trim();
    let mut chars = trimmed.chars();

    let ch = chars
        .next()
        .ok_or_else(|| messages::quote_char_empty(line_num))?;

    // Check for trailing content (only single char allowed)
    if chars.next().is_some() {
        return Err(messages::quote_char_multiple(line_num));
    }

    Ok(ch)
}

/// Parse %COUNT / %C directive: statistics about data.
///
/// Syntax variants:
/// - Total count: `%C:Type.total=N`
/// - Distribution: `%C:Type.field:val1=N1,val2=N2,...`
fn parse_count_directive(payload: &str, line_num: usize) -> HedlResult<(String, CountValue)> {
    let trimmed = payload.trim();

    // Find the `=` for total count or `:` for distribution
    if let Some(eq_pos) = trimmed.find('=') {
        // Check if this is a simple total count (Type.total=N) or distribution
        let before_eq = &trimmed[..eq_pos];
        let after_eq = &trimmed[eq_pos + 1..];

        // If there's a `:` before the `=`, it's a distribution
        if let Some(colon_pos) = before_eq.find(':') {
            // Distribution format: Type.field:val1=N1,val2=N2,...
            let key = before_eq[..colon_pos].trim();
            let pairs_str = &trimmed[colon_pos + 1..];

            let mut distribution = BTreeMap::new();
            for pair in pairs_str.split(',') {
                let pair = pair.trim();
                if pair.is_empty() {
                    continue;
                }
                let pair_eq = pair
                    .find('=')
                    .ok_or_else(|| messages::count_pair_missing_equals(line_num))?;
                let value_name = pair[..pair_eq].trim();
                let count_str = pair[pair_eq + 1..].trim();
                let count: usize = count_str
                    .parse()
                    .map_err(|_| messages::count_invalid_number(count_str, line_num))?;
                distribution.insert(value_name.to_string(), count);
            }

            Ok((key.to_string(), CountValue::Distribution(distribution)))
        } else {
            // Simple total: Type.total=N
            let count: usize = after_eq
                .trim()
                .parse()
                .map_err(|_| messages::count_invalid_number(after_eq.trim(), line_num))?;
            Ok((before_eq.to_string(), CountValue::Total(count)))
        }
    } else {
        Err(messages::count_missing_equals(line_num))
    }
}
