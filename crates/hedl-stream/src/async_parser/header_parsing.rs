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

//! Header parsing functionality for async streaming parser.

use super::{AsyncStreamingParser, HeaderInfo, StreamError, StreamResult};
use hedl_core::lex::is_valid_type_name;
use tokio::io::AsyncRead;

impl<R: AsyncRead + Unpin> AsyncStreamingParser<R> {
    pub(super) async fn parse_header(&mut self) -> StreamResult<()> {
        let mut header = HeaderInfo::new();
        let mut found_version = false;
        let mut _found_separator = false;

        while let Some((line_num, line)) = self.reader.next_line().await? {
            self.check_timeout()?;

            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed == "---" {
                _found_separator = true;
                break;
            }

            if trimmed.starts_with('%') {
                self.parse_directive(trimmed, line_num, &mut header, &mut found_version)?;
            } else {
                self.reader.push_back(line_num, line);
                break;
            }
        }

        if !found_version {
            return Err(StreamError::MissingVersion);
        }

        self.header = Some(header);
        Ok(())
    }

    pub(super) fn parse_directive(
        &self,
        line: &str,
        line_num: usize,
        header: &mut HeaderInfo,
        found_version: &mut bool,
    ) -> StreamResult<()> {
        if line.starts_with("%VERSION") {
            self.parse_version_directive(line, header, found_version)
        } else if line.starts_with("%STRUCT") {
            self.parse_struct_directive(line, line_num, header)
        } else if line.starts_with("%ALIAS") {
            self.parse_alias_directive(line, line_num, header)
        } else if line.starts_with("%NEST") {
            self.parse_nest_directive(line, line_num, header)
        } else if line.starts_with("%MODE") || line.starts_with("%PROMPT") {
            // %MODE and %PROMPT: recognized but not parsed in streaming mode.
            // These are parsed by the full parser (hedl-core) when needed.
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Strip inline comments from a directive line.
    ///
    /// Handles `#` characters outside of quoted strings and brackets.
    /// Returns the content before the first unquoted/unbracketed `#`.
    pub(super) fn strip_inline_comment(text: &str) -> &str {
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

    pub(super) fn parse_version_directive(
        &self,
        line: &str,
        header: &mut HeaderInfo,
        found_version: &mut bool,
    ) -> StreamResult<()> {
        // Strip inline comments first
        let line = Self::strip_inline_comment(line);
        let rest = line.strip_prefix("%VERSION").expect("prefix exists").trim();
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

    pub(super) fn parse_struct_directive(
        &self,
        line: &str,
        line_num: usize,
        header: &mut HeaderInfo,
    ) -> StreamResult<()> {
        // Strip inline comments first
        let line = Self::strip_inline_comment(line);
        let rest = line.strip_prefix("%STRUCT").expect("prefix exists").trim();
        let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

        let bracket_start = rest
            .find('[')
            .ok_or_else(|| StreamError::syntax(line_num, "missing '[' in %STRUCT"))?;
        let bracket_end = rest
            .find(']')
            .ok_or_else(|| StreamError::syntax(line_num, "missing ']' in %STRUCT"))?;

        let type_part = rest[..bracket_start].trim().trim_end_matches(':').trim();
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

    pub(super) fn parse_alias_directive(
        &self,
        line: &str,
        line_num: usize,
        header: &mut HeaderInfo,
    ) -> StreamResult<()> {
        // Strip inline comments first
        let line = Self::strip_inline_comment(line);
        let rest = line.strip_prefix("%ALIAS").expect("prefix exists").trim();
        let rest = rest.strip_prefix(':').unwrap_or(rest).trim();

        let sep_pos = rest
            .find('=')
            .or_else(|| rest.find(':'))
            .ok_or_else(|| StreamError::syntax(line_num, "missing '=' or ':' in %ALIAS"))?;

        let alias = rest[..sep_pos].trim();
        let value = rest[sep_pos + 1..].trim().trim_matches('"');

        header.aliases.insert(alias.to_string(), value.to_string());
        Ok(())
    }

    pub(super) fn parse_nest_directive(
        &self,
        line: &str,
        line_num: usize,
        header: &mut HeaderInfo,
    ) -> StreamResult<()> {
        // Strip inline comments first
        let line = Self::strip_inline_comment(line);
        let rest = line.strip_prefix("%NEST").expect("prefix exists").trim();
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
}
