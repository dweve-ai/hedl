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

//! Value formatting implementation for canonical writer.

use super::constants::*;
use super::CanonicalWriter;
use crate::config::QuotingStrategy;
use hedl_core::Value;

impl CanonicalWriter {
    pub(super) fn format_value(&self, value: &Value) -> String {
        match value {
            Value::Null => "~".to_string(),
            Value::Bool(b) => b.to_string(),
            // P4 OPTIMIZATION: Use itoa for faster integer formatting (2-3x faster)
            Value::Int(n) => {
                let mut buf = itoa::Buffer::new();
                buf.format(*n).to_string()
            }
            Value::Float(f) => {
                if f.is_finite() && f.fract() == FLOAT_WHOLE_NUMBER_FRACTIONAL_PART {
                    format!("{f:.WHOLE_NUMBER_DECIMAL_PLACES$}")
                } else {
                    f.to_string()
                }
            }
            Value::String(s) => self.format_string(s),
            Value::Tensor(t) => self.format_tensor(t),
            Value::Reference(r) => r.to_ref_string(),
            Value::Expression(e) => format!("$({e})"),
            Value::List(items) => self.format_list(items),
        }
    }

    /// Format a string value, checking if it needs a block string for multiline content.
    /// Returns (`formatted_value`, `needs_block_string`) where `needs_block_string` indicates
    /// the caller should use block string format instead of inline format.
    pub(super) fn format_value_for_kv(&self, value: &Value) -> (String, bool) {
        match value {
            Value::String(s) if s.contains('\n') => {
                // Multiline strings need block string format
                (s.to_string(), true)
            }
            _ => (self.format_value(value), false),
        }
    }

    pub(super) fn format_cell_value_with_position(
        &self,
        value: &Value,
        is_last_col: bool,
    ) -> String {
        match value {
            Value::Null => "~".to_string(),
            Value::Bool(b) => b.to_string(),
            // P4 OPTIMIZATION: Use itoa for faster integer formatting (2-3x faster)
            Value::Int(n) => {
                let mut buf = itoa::Buffer::new();
                buf.format(*n).to_string()
            }
            Value::Float(f) => {
                if f.is_finite() && f.fract() == FLOAT_WHOLE_NUMBER_FRACTIONAL_PART {
                    format!("{f:.WHOLE_NUMBER_DECIMAL_PLACES$}")
                } else {
                    f.to_string()
                }
            }
            Value::String(s) => self.format_cell_string_with_position(s, is_last_col),
            Value::Tensor(t) => self.format_tensor(t),
            Value::Reference(r) => r.to_ref_string(),
            Value::Expression(e) => format!("$({e})"),
            Value::List(items) => self.format_list(items),
        }
    }

    pub(super) fn format_string(&self, s: &str) -> String {
        // P4 OPTIMIZATION: Fast path for ASCII alphanumeric + common punctuation
        // Avoids expensive needs_quoting check for common simple strings (30-40% faster)
        // Must exclude pure numeric strings (which need quoting), booleans, and strings starting with special chars
        if self.config.quoting != QuotingStrategy::Always
            && !s.is_empty()
            && s != "true"  // Exclude booleans
            && s != "false"
            && s.bytes()
                .all(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'.'))
            && !s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'.' | b'-'))  // Exclude pure numbers
            && !matches!(s.bytes().next(), Some(b'0'..=b'9' | b'-' | b'.'))
        // Exclude starting with number
        {
            return s.to_string();
        }

        // Slow path: full validation
        if self.config.quoting == QuotingStrategy::Always || self.needs_quoting_kv(s) {
            format!("\"{}\"", Self::escape_quoted(s))
        } else {
            s.to_string()
        }
    }

    pub(super) fn format_cell_string_with_position(&self, s: &str, is_last_col: bool) -> String {
        // Per SPEC.md Section 13.2: Empty strings in the last column MUST be quoted as ""
        // to avoid trailing comma syntax error
        if s.is_empty() && is_last_col {
            return "\"\"".to_string();
        }

        // Check if string has control characters that need escaping
        // Per SPEC.md Section E.2: EscapeSeq ::= '\n' | '\t' | '\r' | '\\' | '\"'
        let needs_escape =
            s.contains('\n') || s.contains('\t') || s.contains('\r') || s.contains('\\');

        if self.config.quoting == QuotingStrategy::Always
            || self.needs_quoting_cell(s)
            || needs_escape
        {
            format!("\"{}\"", Self::escape_cell_string(s))
        } else {
            s.to_string()
        }
    }

    pub(super) fn needs_quoting_kv(&self, s: &str) -> bool {
        if s.is_empty() {
            return true;
        }
        // Needs quoting if:
        // - Has leading/trailing whitespace
        // - Contains # (comment)
        // - Would trigger inference (starts with special chars)
        // - Contains quotes
        // SAFETY: is_empty() check at line 992-993 guarantees at least one char
        let first_char = s.chars().next().expect("non-empty string");
        s != s.trim()
            || s.contains('#')
            || s.contains('"')
            || matches!(first_char, '~' | '@' | '$' | '%' | '[' | '(')
            || s == "true"
            || s == "false"
            || s.parse::<i64>().is_ok()
            || s.parse::<f64>().is_ok()
    }

    pub(super) fn needs_quoting_cell(&self, s: &str) -> bool {
        if s.is_empty() {
            return false; // Empty cell is OK without quotes (except trailing)
        }
        // SAFETY: is_empty() check above guarantees at least one char
        let first_char = s.chars().next().expect("non-empty string");
        s != s.trim()
            || s.contains(',')
            || s.contains('|')
            || s.contains('#')
            || s.contains('"')
            || matches!(first_char, '~' | '@' | '$' | '%' | '^' | '[' | '(')
            || s == "true"
            || s == "false"
            || s.parse::<i64>().is_ok()
            || s.parse::<f64>().is_ok()
    }

    pub(super) fn escape_quoted(s: &str) -> String {
        s.replace('"', "\"\"")
    }

    /// Escape a string for matrix cell output, using escape sequences for control characters.
    /// P4 OPTIMIZATION: Fast path for strings without special characters (40-50% faster)
    /// Per SPEC.md Section E.2: EscapeSeq ::= '\n' | '\t' | '\r' | '\\' | '\"'
    pub(super) fn escape_cell_string(s: &str) -> String {
        // Fast path: no special characters
        if !s
            .bytes()
            .any(|b| matches!(b, b'"' | b'\n' | b'\t' | b'\r' | b'\\'))
        {
            return s.to_string();
        }

        // Slow path: escape special characters
        let mut result = String::with_capacity(s.len() + 8);
        for c in s.chars() {
            match c {
                '"' => result.push_str("\"\""),
                '\n' => result.push_str("\\n"),
                '\t' => result.push_str("\\t"),
                '\r' => result.push_str("\\r"),
                '\\' => result.push_str("\\\\"),
                _ => result.push(c),
            }
        }
        result
    }

    pub(super) fn format_tensor(&self, tensor: &hedl_core::Tensor) -> String {
        use hedl_core::Tensor;
        match tensor {
            Tensor::Scalar(n) => {
                if n.is_finite() && n.fract() == FLOAT_WHOLE_NUMBER_FRACTIONAL_PART {
                    format!("{n:.WHOLE_NUMBER_DECIMAL_PLACES$}")
                } else {
                    n.to_string()
                }
            }
            Tensor::Array(items) => {
                let inner: Vec<String> = items.iter().map(|t| self.format_tensor(t)).collect();
                format!("[{}]", inner.join(", "))
            }
        }
    }

    /// Format a list value as `(elem1, elem2, elem3)`.
    ///
    /// Lists use parentheses instead of square brackets to distinguish them from tensors.
    /// Elements are formatted recursively and separated by `, ` (comma space).
    ///
    /// # Examples
    ///
    /// - Empty list: `()`
    /// - String list: `(admin, editor, viewer)`
    /// - Boolean list: `(true, false)`
    /// - Mixed types: `(1, "two", true)`
    pub(super) fn format_list(&self, items: &[Value]) -> String {
        if items.is_empty() {
            return "()".to_string();
        }

        let formatted: Vec<String> = items.iter().map(|v| self.format_value(v)).collect();
        format!("({})", formatted.join(", "))
    }
}
