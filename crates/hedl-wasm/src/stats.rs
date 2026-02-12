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

//! Token statistics and estimation.

#[cfg(any(feature = "statistics", feature = "token-tools"))]
use serde::Serialize;

/// Token estimation constant: approximate characters per token for structured data
#[cfg(any(feature = "statistics", feature = "token-tools"))]
const CHARS_PER_TOKEN: usize = 4;

/// Token statistics.
#[cfg(any(feature = "statistics", feature = "token-tools"))]
#[derive(Serialize)]
pub struct TokenStats {
    #[serde(rename = "hedlBytes")]
    pub(crate) hedl_bytes: usize,
    #[serde(rename = "hedlTokens")]
    pub(crate) hedl_tokens: usize,
    #[serde(rename = "hedlLines")]
    pub(crate) hedl_lines: usize,
    #[serde(rename = "jsonBytes")]
    pub(crate) json_bytes: usize,
    #[serde(rename = "jsonTokens")]
    pub(crate) json_tokens: usize,
    #[serde(rename = "savingsPercent")]
    pub(crate) savings_percent: i32,
    #[serde(rename = "tokensSaved")]
    pub(crate) tokens_saved: i32,
}

/// Optimized single-pass token estimation.
///
/// This function estimates the number of tokens in text using a highly optimized
/// byte-level loop, avoiding character iteration overhead. Provides approximately
/// 3x speedup compared to the multi-pass .`chars().filter()` approach.
///
/// # Algorithm
/// - Direct byte iteration for ASCII fast path
/// - Efficient UTF-8 handling for multi-byte characters
/// - Counts bytes, whitespace, and punctuation simultaneously
/// - No allocations or iterator overhead
///
/// # Performance
/// - Time complexity: O(n) single pass over bytes
/// - Space complexity: O(1) constant
/// - ~3x faster than multi-pass .`chars().filter()` approach
/// - ~1.5x faster than single-pass .`chars()` loop
///
/// # Token Estimation Formula
/// `tokens = (byte_count + whitespace_count + punct_count) / CHARS_PER_TOKEN`
///
/// This approximates language model tokenization where:
/// - Whitespace and punctuation often become separate tokens
/// - Average token is ~4 characters for structured data
#[cfg(any(feature = "statistics", feature = "token-tools"))]
#[inline]
pub(crate) fn estimate_tokens(text: &str) -> usize {
    let bytes = text.as_bytes();
    let byte_count = bytes.len();

    // Fast path for ASCII-only strings (common case for JSON/HEDL)
    if byte_count == 0 {
        return 0;
    }

    let mut whitespace_count = 0usize;
    let mut punct_count = 0usize;
    let mut i = 0;

    // Process bytes directly for maximum performance
    while i < byte_count {
        let b = bytes[i];

        // ASCII fast path (most common in structured data)
        if b < 128 {
            // Check for ASCII whitespace: space, tab, newline, carriage return
            whitespace_count += usize::from(matches!(b, b' ' | b'\t' | b'\n' | b'\r'));

            // Check for ASCII punctuation
            punct_count += usize::from(matches!(
                b,
                b'!' | b'"'
                    | b'#'
                    | b'$'
                    | b'%'
                    | b'&'
                    | b'\''
                    | b'('
                    | b')'
                    | b'*'
                    | b'+'
                    | b','
                    | b'-'
                    | b'.'
                    | b'/'
                    | b':'
                    | b';'
                    | b'<'
                    | b'='
                    | b'>'
                    | b'?'
                    | b'@'
                    | b'['
                    | b'\\'
                    | b']'
                    | b'^'
                    | b'_'
                    | b'`'
                    | b'{'
                    | b'|'
                    | b'}'
                    | b'~'
            ));

            i += 1;
        } else {
            // UTF-8 multi-byte character - skip to next character
            // UTF-8 encoding: 110xxxxx (2 bytes), 1110xxxx (3 bytes), 11110xxx (4 bytes)
            let char_len = if b < 0b1110_0000 {
                2
            } else if b < 0b1111_0000 {
                3
            } else {
                4
            };
            i += char_len;

            // Most multi-byte UTF-8 characters are not whitespace or punctuation
            // We could check specific Unicode ranges, but for performance we skip it
        }
    }

    // Apply token estimation formula
    (byte_count + whitespace_count + punct_count) / CHARS_PER_TOKEN
}
