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

//! UTF-16 surrogate handling for JSON strings

use super::config::{JsonConversionError, SurrogatePolicy};

pub(super) fn preprocess_json_for_surrogates(
    json: &str,
    policy: SurrogatePolicy,
) -> Result<String, JsonConversionError> {
    if policy == SurrogatePolicy::Reject {
        // Let serde_json handle rejection with its native error messages
        return Ok(json.to_string());
    }

    let bytes = json.as_bytes();
    let mut result = String::with_capacity(json.len());
    let mut i = 0;

    while i < bytes.len() {
        // Look for backslash followed by 'u'
        if i + 5 < bytes.len() && bytes[i] == b'\\' && bytes[i + 1] == b'u' {
            // Try to parse the 4 hex digits
            if let Some(code) = parse_unicode_escape(&bytes[i + 2..i + 6]) {
                let is_high_surrogate = (0xD800..=0xDBFF).contains(&code);
                let is_low_surrogate = (0xDC00..=0xDFFF).contains(&code);

                if is_high_surrogate {
                    // Check if followed by a valid low surrogate
                    let has_low_pair = i + 11 < bytes.len()
                        && bytes[i + 6] == b'\\'
                        && bytes[i + 7] == b'u'
                        && parse_unicode_escape(&bytes[i + 8..i + 12])
                            .is_some_and(|low| (0xDC00..=0xDFFF).contains(&low));

                    if has_low_pair {
                        // Valid surrogate pair - copy both escapes
                        result.push_str(&json[i..i + 12]);
                        i += 12;
                        continue;
                    }
                    // Unpaired high surrogate
                    match policy {
                        SurrogatePolicy::ReplaceWithFFFD => {
                            result.push_str("\\uFFFD");
                        }
                        SurrogatePolicy::Skip => {
                            // Skip the escape sequence entirely
                        }
                        SurrogatePolicy::Reject => unreachable!(),
                    }
                    i += 6;
                    continue;
                } else if is_low_surrogate {
                    // Low surrogate without preceding high - always unpaired
                    match policy {
                        SurrogatePolicy::ReplaceWithFFFD => {
                            result.push_str("\\uFFFD");
                        }
                        SurrogatePolicy::Skip => {
                            // Skip the escape sequence entirely
                        }
                        SurrogatePolicy::Reject => unreachable!(),
                    }
                    i += 6;
                    continue;
                }
            }
        }

        // Copy current byte as-is
        // SAFETY: i < json.len() guarantees at least one char exists
        let ch = json[i..].chars().next().expect("non-empty slice");
        result.push(ch);
        i += ch.len_utf8();
    }

    Ok(result)
}

/// Parse 4 hex digits into a u16 value
#[inline]
fn parse_unicode_escape(bytes: &[u8]) -> Option<u16> {
    if bytes.len() < 4 {
        return None;
    }

    let mut value: u16 = 0;
    for &b in &bytes[..4] {
        let digit = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => return None,
        };
        value = value * 16 + u16::from(digit);
    }
    Some(value)
}
