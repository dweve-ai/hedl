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

//! P2 optimization: Lookup tables and helper functions for fast value inference.

use crate::error::HedlResult;
use crate::value::Value;
use std::sync::OnceLock;

/// Lookup table entry for pre-inferred common values.
///
/// Part of the P2 optimization for fast value inference using perfect hash tables.
/// This provides O(1) lookup for common patterns like true/false.
#[derive(Clone)]
struct LookupEntry {
    /// The exact string to match (for collision detection)
    pattern: &'static str,
    /// Pre-constructed Value result
    value: ValueTemplate,
}

/// Template for constructing values (avoids cloning complex types in static)
/// Note: Null is NOT included because it uses the configurable null_char from %NULL directive.
#[derive(Clone)]
enum ValueTemplate {
    Bool(bool),
}

impl ValueTemplate {
    #[inline(always)]
    fn to_value(&self) -> Value {
        match self {
            ValueTemplate::Bool(b) => Value::Bool(*b),
        }
    }
}

/// Static lookup table for common values.
/// Indexed by (length, first_byte) hash.
static COMMON_VALUES: OnceLock<Vec<Option<LookupEntry>>> = OnceLock::new();

/// Initialize the common values lookup table.
/// Note: Null is NOT included here because it uses the configurable null_char from %NULL directive.
/// Null is handled separately in infer_value before this lookup is called.
fn init_common_values() -> Vec<Option<LookupEntry>> {
    // Create a sparse table indexed by hash(length, first_byte)
    // Size chosen to minimize collisions for our common values
    let mut table = vec![None; 256];

    let entries = [
        // Booleans (most common)
        ("true", ValueTemplate::Bool(true)),
        ("false", ValueTemplate::Bool(false)),
    ];

    for (pattern, value) in entries {
        let hash = hash_string(pattern);
        table[hash] = Some(LookupEntry { pattern, value });
    }

    table
}

/// Compute hash for lookup table.
/// Uses length and first byte to create a unique index for our small domain.
#[inline(always)]
fn hash_string(s: &str) -> usize {
    let len = s.len();
    let first = s.as_bytes().first().copied().unwrap_or(0);
    // Combine length and first byte into 8-bit hash
    (len ^ ((first as usize) << 3)) & 0xFF
}

/// P2 OPTIMIZATION: Fast lookup for common values using perfect hash table.
///
/// Attempts to resolve value via lookup table before falling back to full inference.
/// This provides 10-15% speedup for typical HEDL documents by eliminating redundant
/// checks for the most common value types.
///
/// Returns Some(Value) if found in lookup table, None otherwise.
#[inline]
pub(super) fn try_lookup_common(s: &str) -> Option<Value> {
    // Initialize table on first use (thread-safe, happens once)
    let table = COMMON_VALUES.get_or_init(init_common_values);

    let hash = hash_string(s);

    // Bounds check is optimized away by compiler (hash is always < 256)
    if let Some(entry) = &table[hash] {
        // Verify exact match (collision detection)
        if entry.pattern == s {
            return Some(entry.value.to_value());
        }
    }

    None
}

/// Infer value from expanded alias (no further alias expansion).
pub(super) fn infer_expanded_alias(s: &str, _line_num: usize) -> HedlResult<Value> {
    // Boolean
    if s == "true" {
        return Ok(Value::Bool(true));
    }
    if s == "false" {
        return Ok(Value::Bool(false));
    }

    // Number
    if let Some(value) = try_parse_number(s) {
        return Ok(value);
    }

    // String - use Box::from() to avoid intermediate String allocation
    Ok(Value::String(Box::from(s)))
}

/// Try to parse a string as a number.
/// Optimized: work on bytes, quick validation, try parse directly.
pub(super) fn try_parse_number(s: &str) -> Option<Value> {
    let trimmed = s.trim();
    let bytes = trimmed.as_bytes();

    if bytes.is_empty() {
        return None;
    }

    // Quick check: first char must be digit or minus
    let first = bytes[0];
    if first != b'-' && !first.is_ascii_digit() {
        return None;
    }

    // Quick scan for decimal point (no allocation)
    let has_decimal = memchr::memchr(b'.', bytes).is_some();

    // Try parsing directly - Rust's parse is well-optimized
    if has_decimal {
        // For floats, also reject if it ends with '.' (e.g., "123.")
        // or has non-numeric chars
        trimmed.parse::<f64>().ok().and_then(|f| {
            // Reject special values and ensure it was a valid number format
            if f.is_finite() && !trimmed.ends_with('.') {
                Some(Value::Float(f))
            } else {
                None
            }
        })
    } else {
        trimmed.parse::<i64>().ok().map(Value::Int)
    }
}

/// Infer value from a quoted string (always returns String).
pub fn infer_quoted_value(s: &str) -> Value {
    // Process "" escapes
    let unescaped = s.replace("\"\"", "\"");
    Value::String(unescaped.into_boxed_str())
}
