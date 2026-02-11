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

//! Helper functions for parser

use hedl_core::lex::strip_comment;

/// Split inline rows by `|` delimiter (respecting quotes and brackets).
///
/// # Arguments
///
/// * `content` - Line content starting with `|` delimiter
///
/// # Returns
///
/// A vector of row contents (without leading `|`).
pub(crate) fn split_inline_rows(content: &str) -> crate::error::StreamResult<Vec<&str>> {
    let content = content.trim();

    // Must start with |
    if !content.starts_with('|') {
        return Ok(vec![]);
    }

    let content = &content[1..]; // Skip leading |

    // Handle empty case
    if content.trim().is_empty() {
        return Ok(vec![]);
    }

    let mut rows = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    let mut bracket_depth: usize = 0; // For [] tensors
    let mut paren_depth: usize = 0; // For () lists
    let mut escape = false;

    let bytes = content.as_bytes();

    for (i, &c) in bytes.iter().enumerate() {
        if escape {
            escape = false;
            continue;
        }

        match c {
            b'\\' if in_quotes => escape = true,
            b'"' => in_quotes = !in_quotes,
            b'[' if !in_quotes => bracket_depth += 1,
            b']' if !in_quotes => bracket_depth = bracket_depth.saturating_sub(1),
            b'(' if !in_quotes => paren_depth += 1,
            b')' if !in_quotes => paren_depth = paren_depth.saturating_sub(1),
            b'|' if !in_quotes && bracket_depth == 0 && paren_depth == 0 => {
                // Found a delimiter
                let row = &content[start..i];
                if !row.trim().is_empty() {
                    rows.push(row);
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    // Add the last segment
    let last = &content[start..];
    // Strip trailing comment from last segment
    let last = strip_comment(last);
    if !last.trim().is_empty() {
        rows.push(last);
    }

    Ok(rows)
}
