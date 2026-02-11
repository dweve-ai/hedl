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

//! Type definitions for HEDL header parsing.

use std::collections::BTreeMap;

/// Parsing mode for validation handling.
///
/// Controls how validation issues are handled during parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParseMode {
    /// Strict mode (default): First validation error halts parsing.
    #[default]
    Strict,
    /// Lenient mode: Validation issues become null (`~`), diagnostics emitted out-of-band.
    Lenient,
}

/// A count/statistics value from %C directives.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CountValue {
    /// Total count: `%C:Type.total=N`
    Total(usize),
    /// Distribution: `%C:Type.field:val1=N1,val2=N2,...`
    Distribution(BTreeMap<String, usize>),
}

/// Parsed header data.
#[derive(Debug, Clone)]
pub struct Header {
    /// HEDL format version as (major, minor).
    pub version: (u32, u32),
    /// Parsing mode (strict or lenient). Default: strict.
    pub mode: ParseMode,
    /// Type aliases mapping alias name to original type.
    pub aliases: BTreeMap<String, String>,
    /// Struct definitions mapping struct name to field names.
    pub structs: BTreeMap<String, Vec<String>>,
    /// Expected row counts for structs (from count hints).
    pub struct_counts: BTreeMap<String, usize>,
    /// Nesting relationships mapping parent type to child types.
    /// A parent type can have multiple child types (e.g., Customer > Address, Customer > Order).
    pub nests: BTreeMap<String, Vec<String>>,
    /// Prompt text for LLM/tooling hints (optional).
    pub prompt: Option<Box<str>>,
    /// Null character (default: `~`).
    pub null_char: char,
    /// Quote character (default: `"`).
    pub quote_char: char,
    /// Count statistics from %C directives.
    /// Key format: "Type.field" or "Type.total"
    pub counts: BTreeMap<String, CountValue>,
}

impl Header {
    /// Create a new Header with the given version and empty collections.
    pub fn new(version: (u32, u32)) -> Self {
        Self {
            version,
            mode: ParseMode::default(),
            aliases: BTreeMap::new(),
            structs: BTreeMap::new(),
            struct_counts: BTreeMap::new(),
            nests: BTreeMap::new(),
            prompt: None,
            null_char: '~',
            quote_char: '"',
            counts: BTreeMap::new(),
        }
    }
}

/// Normalize directive name from compact to verbose form.
///
/// Maps compact directives to their verbose equivalents:
/// - `%V` → `%VERSION`
/// - `%S` → `%STRUCT`
/// - `%N` → `%NEST`
/// - `%A` → `%ALIAS`
/// - `%C` → `%COUNT`
pub(super) fn normalize_directive_name(name: &str) -> &str {
    match name {
        "%V" => "%VERSION",
        "%S" => "%STRUCT",
        "%N" => "%NEST",
        "%A" => "%ALIAS",
        "%C" => "%COUNT",
        _ => name,
    }
}

/// Check if a directive uses compact syntax (no space after colon).
pub(super) fn is_compact_syntax(directive_name: &str) -> bool {
    matches!(
        directive_name,
        "%V" | "%S" | "%N" | "%A" | "%C" | "%NULL" | "%QUOTE"
    )
}
