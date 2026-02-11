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

//! HEDL header directive parsing
//!
//! Handles parsing of all HEDL header directives including:
//! - Version directives (%VERSION, %V:)
//! - Schema directives (%STRUCT, %S:)
//! - Alias directives (%ALIAS)
//! - Nesting directives (%NEST, %N:)
//! - Count hint directives (%C:)
//! - Configuration directives (%NULL:, %QUOTE:)

use crate::error::{StreamError, StreamResult};
use crate::event::HeaderInfo;
use hedl_core::lex::is_valid_type_name;

/// Strip inline comments from a directive line.
///
/// Handles `#` characters outside of quoted strings and brackets.
/// Returns the content before the first unquoted/unbracketed `#`.
pub(crate) fn strip_inline_comment(text: &str) -> &str {
    let mut in_quotes = false;
    let mut in_brackets = 0;
    let mut quote_char = '"';

    for (i, c) in text.char_indices() {
        match c {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = c;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            '[' if !in_quotes => in_brackets += 1,
            ']' if !in_quotes && in_brackets > 0 => in_brackets -= 1,
            '#' if !in_quotes && in_brackets == 0 => {
                return text[..i].trim_end();
            }
            _ => {}
        }
    }
    text
}

/// Parse a directive line and update the header.
pub(crate) fn parse_directive(
    line: &str,
    line_num: usize,
    header: &mut HeaderInfo,
    found_version: &mut bool,
) -> StreamResult<()> {
    // v2.0 compact directives
    if line.starts_with("%V:") {
        parse_v_directive(line, header, found_version)
    } else if line.starts_with("%NULL:") {
        parse_null_directive(line, line_num, header)
    } else if line.starts_with("%QUOTE:") {
        parse_quote_directive(line, line_num, header)
    } else if line.starts_with("%S:") {
        parse_s_directive(line, line_num, header)
    } else if line.starts_with("%N:") {
        parse_n_directive(line, line_num, header)
    } else if line.starts_with("%C:") {
        parse_c_directive(line, line_num, header)
    }
    // Legacy verbose directives (maintain backward compatibility)
    else if line.starts_with("%VERSION") {
        parse_version_directive(line, header, found_version)
    } else if line.starts_with("%STRUCT") {
        parse_struct_directive(line, line_num, header)
    } else if line.starts_with("%ALIAS") {
        parse_alias_directive(line, line_num, header)
    } else if line.starts_with("%NEST") {
        parse_nest_directive(line, line_num, header)
    } else if line.starts_with("%MODE") || line.starts_with("%PROMPT") {
        // %MODE and %PROMPT: recognized but not parsed in streaming mode.
        // These are parsed by the full parser (hedl-core) when needed.
        Ok(())
    } else {
        Ok(())
    }
}

// ==================== Legacy Verbose Directive Parsers ====================

fn parse_version_directive(
    line: &str,
    header: &mut HeaderInfo,
    found_version: &mut bool,
) -> StreamResult<()> {
    // Strip inline comments first
    let line = strip_inline_comment(line);
    // Safe: starts_with check guarantees prefix exists
    let rest = line.strip_prefix("%VERSION").expect("prefix exists").trim();
    // Handle both "%VERSION: 1.0" and "%VERSION: 1.0" formats
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();
    let parts: Vec<&str> = rest.split('.').collect();

    if parts.len() != 2 {
        return Err(StreamError::InvalidVersion(rest.to_string()));
    }

    let major: u32 = parts[0]
        .parse()
        .map_err(|_| StreamError::InvalidVersion(rest.to_string()))?;
    let minor: u32 = parts[1]
        .parse()
        .map_err(|_| StreamError::InvalidVersion(rest.to_string()))?;

    header.version = (major, minor);
    *found_version = true;
    Ok(())
}

fn parse_struct_directive(
    line: &str,
    line_num: usize,
    header: &mut HeaderInfo,
) -> StreamResult<()> {
    // Strip inline comments first
    let line = strip_inline_comment(line);
    // Safe: starts_with check guarantees prefix exists
    let rest = line.strip_prefix("%STRUCT").expect("prefix exists").trim();
    // Handle both "%STRUCT TypeName: [cols]" and "%STRUCT: TypeName: [cols]" formats
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

    let bracket_start = rest
        .find('[')
        .ok_or_else(|| StreamError::syntax(line_num, "missing '[' in %STRUCT"))?;
    let bracket_end = rest
        .find(']')
        .ok_or_else(|| StreamError::syntax(line_num, "missing ']' in %STRUCT"))?;

    // Type name may have trailing colon and optional count, strip them
    // Format: TypeName: or TypeName (N):
    let type_part = rest[..bracket_start].trim().trim_end_matches(':').trim();
    // Handle optional count: "TypeName (N)" -> extract just "TypeName"
    let type_name = if let Some(paren_pos) = type_part.find('(') {
        type_part[..paren_pos].trim()
    } else {
        type_part
    };
    if !is_valid_type_name(type_name) {
        return Err(StreamError::syntax(
            line_num,
            format!("invalid type name: {type_name}"),
        ));
    }

    let cols_str = &rest[bracket_start + 1..bracket_end];
    let columns: Vec<String> = cols_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if columns.is_empty() {
        return Err(StreamError::syntax(line_num, "empty schema"));
    }

    header.structs.insert(type_name.to_string(), columns);
    Ok(())
}

fn parse_alias_directive(line: &str, line_num: usize, header: &mut HeaderInfo) -> StreamResult<()> {
    // Strip inline comments first
    let line = strip_inline_comment(line);
    // Safe: starts_with check guarantees prefix exists
    let rest = line.strip_prefix("%ALIAS").expect("prefix exists").trim();
    // Handle both "%ALIAS: %short: = ..." and "%ALIAS: %short: ..." formats
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

    // Support both '=' and ':' as separators
    let sep_pos = rest
        .find('=')
        .or_else(|| rest.find(':'))
        .ok_or_else(|| StreamError::syntax(line_num, "missing '=' or ':' in %ALIAS"))?;

    let alias = rest[..sep_pos].trim();
    let value = rest[sep_pos + 1..].trim().trim_matches('"');

    header.aliases.insert(alias.to_string(), value.to_string());
    Ok(())
}

fn parse_nest_directive(line: &str, line_num: usize, header: &mut HeaderInfo) -> StreamResult<()> {
    // Strip inline comments first
    let line = strip_inline_comment(line);
    // Safe: starts_with check guarantees prefix exists
    let rest = line.strip_prefix("%NEST").expect("prefix exists").trim();
    // Handle both "%NEST: Parent > Child" and "%NEST: Parent > Child" formats
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

    let arrow_pos = rest
        .find('>')
        .ok_or_else(|| StreamError::syntax(line_num, "missing '>' in %NEST"))?;

    let parent = rest[..arrow_pos].trim();
    let child = rest[arrow_pos + 1..].trim();

    if !is_valid_type_name(parent) || !is_valid_type_name(child) {
        return Err(StreamError::syntax(line_num, "invalid type name in %NEST"));
    }

    header
        .nests
        .entry(parent.to_string())
        .or_default()
        .push(child.to_string());
    Ok(())
}

// ==================== v2.0 Compact Directive Parsers ====================

/// Parse %V:2.0 (v2.0 compact version directive).
fn parse_v_directive(
    line: &str,
    header: &mut HeaderInfo,
    found_version: &mut bool,
) -> StreamResult<()> {
    let line = strip_inline_comment(line);
    let rest = line.strip_prefix("%V:").expect("prefix exists").trim();
    let parts: Vec<&str> = rest.split('.').collect();

    if parts.len() != 2 {
        return Err(StreamError::InvalidVersion(rest.to_string()));
    }

    let major: u32 = parts[0]
        .parse()
        .map_err(|_| StreamError::InvalidVersion(rest.to_string()))?;
    let minor: u32 = parts[1]
        .parse()
        .map_err(|_| StreamError::InvalidVersion(rest.to_string()))?;

    header.version = (major, minor);
    *found_version = true;
    Ok(())
}

/// Parse %NULL:~ (v2.0 null literal character directive).
fn parse_null_directive(line: &str, line_num: usize, header: &mut HeaderInfo) -> StreamResult<()> {
    let line = strip_inline_comment(line);
    let rest = line.strip_prefix("%NULL:").expect("prefix exists").trim();

    if rest.is_empty() {
        return Err(StreamError::syntax(
            line_num,
            "missing null character in %NULL",
        ));
    }

    // SAFETY: is_empty() check guarantees at least one char exists
    let null_char = rest.chars().next().expect("non-empty string");
    header.null_char = null_char;
    Ok(())
}

/// Parse %QUOTE:" (v2.0 quote character directive).
fn parse_quote_directive(line: &str, line_num: usize, header: &mut HeaderInfo) -> StreamResult<()> {
    let line = strip_inline_comment(line);
    let rest = line.strip_prefix("%QUOTE:").expect("prefix exists").trim();

    if rest.is_empty() {
        return Err(StreamError::syntax(
            line_num,
            "missing quote character in %QUOTE",
        ));
    }

    // SAFETY: is_empty() check guarantees at least one char exists
    let quote_char = rest.chars().next().expect("non-empty string");
    header.quote_char = quote_char;
    Ok(())
}

/// Parse %S:Type:[col1,col2,...] (v2.0 compact schema directive).
fn parse_s_directive(line: &str, line_num: usize, header: &mut HeaderInfo) -> StreamResult<()> {
    let line = strip_inline_comment(line);
    let rest = line.strip_prefix("%S:").expect("prefix exists").trim();

    // Find the colon separating type name from column list
    let colon_pos = rest
        .find(':')
        .ok_or_else(|| StreamError::syntax(line_num, "missing ':' separator in %S"))?;

    let type_name = rest[..colon_pos].trim();
    if !is_valid_type_name(type_name) {
        return Err(StreamError::syntax(
            line_num,
            format!("invalid type name: {type_name}"),
        ));
    }

    let columns_part = rest[colon_pos + 1..].trim();
    let bracket_start = columns_part
        .find('[')
        .ok_or_else(|| StreamError::syntax(line_num, "missing '[' in %S"))?;
    let bracket_end = columns_part
        .find(']')
        .ok_or_else(|| StreamError::syntax(line_num, "missing ']' in %S"))?;

    let cols_str = &columns_part[bracket_start + 1..bracket_end];
    let columns: Vec<String> = cols_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if columns.is_empty() {
        return Err(StreamError::syntax(line_num, "empty schema in %S"));
    }

    header.structs.insert(type_name.to_string(), columns);
    Ok(())
}

/// Parse %N:Parent>Child (v2.0 compact nesting directive).
fn parse_n_directive(line: &str, line_num: usize, header: &mut HeaderInfo) -> StreamResult<()> {
    let line = strip_inline_comment(line);
    let rest = line.strip_prefix("%N:").expect("prefix exists").trim();

    let arrow_pos = rest
        .find('>')
        .ok_or_else(|| StreamError::syntax(line_num, "missing '>' in %N"))?;

    let parent = rest[..arrow_pos].trim();
    let child = rest[arrow_pos + 1..].trim();

    if !is_valid_type_name(parent) || !is_valid_type_name(child) {
        return Err(StreamError::syntax(line_num, "invalid type name in %N"));
    }

    header
        .nests
        .entry(parent.to_string())
        .or_default()
        .push(child.to_string());
    Ok(())
}

/// Parse %C:Type.total=N or %C:Type.field:val=N,... (v2.0 count hint directives).
fn parse_c_directive(line: &str, line_num: usize, header: &mut HeaderInfo) -> StreamResult<()> {
    let line = strip_inline_comment(line);
    let rest = line.strip_prefix("%C:").expect("prefix exists").trim();

    // Find the first dot to separate type name
    let dot_pos = rest
        .find('.')
        .ok_or_else(|| StreamError::syntax(line_num, "missing '.' in %C"))?;

    let type_name = rest[..dot_pos].trim();
    if !is_valid_type_name(type_name) {
        return Err(StreamError::syntax(
            line_num,
            format!("invalid type name: {type_name}"),
        ));
    }

    let remainder = &rest[dot_pos + 1..];

    // Check if it's a total count: Type.total=N
    if remainder.starts_with("total=") {
        // SAFETY: starts_with() guarantees strip_prefix succeeds
        let count_str = remainder
            .strip_prefix("total=")
            .expect("prefix exists")
            .trim();
        let count: usize = count_str
            .parse()
            .map_err(|_| StreamError::syntax(line_num, "invalid count in %C"))?;
        header.count_totals.insert(type_name.to_string(), count);
        return Ok(());
    }

    // Otherwise it's a field count: Type.field:val=N,val2=N2,...
    // Find the colon separating field name from values
    let colon_pos = remainder
        .find(':')
        .ok_or_else(|| StreamError::syntax(line_num, "missing ':' in %C field count"))?;

    let field_name = remainder[..colon_pos].trim();
    let values_part = &remainder[colon_pos + 1..];

    // Parse value=count pairs
    let field_key = format!("{type_name}.{field_name}");
    let field_counts = header.count_fields.entry(field_key).or_default();

    for pair in values_part.split(',') {
        let eq_pos = pair
            .find('=')
            .ok_or_else(|| StreamError::syntax(line_num, "missing '=' in %C field count"))?;

        let value = pair[..eq_pos].trim().to_string();
        let count_str = pair[eq_pos + 1..].trim();
        let count: usize = count_str
            .parse()
            .map_err(|_| StreamError::syntax(line_num, "invalid count in %C field count"))?;

        field_counts.insert(value, count);
    }

    Ok(())
}
