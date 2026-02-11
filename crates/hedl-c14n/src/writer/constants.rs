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

//! Constants used throughout the canonical writer.

// ==================== Buffer Capacity Constants ====================

/// Initial buffer capacity for output string.
///
/// Pre-allocates 4KB to minimize reallocations for typical HEDL documents.
/// This optimization provides 1.2-1.3x speedup (P1 optimization).
///
/// Capacity chosen based on empirical testing:
/// - Most HEDL documents are < 4KB
/// - Larger documents still benefit from reduced early reallocations
/// - Memory overhead is minimal (4KB per writer instance)
pub(super) const INITIAL_OUTPUT_BUFFER_CAPACITY: usize = 4096;

// ==================== Nesting Depth Constants ====================

/// Maximum nesting depth for recursive object structures.
///
/// Prevents stack overflow denial-of-service attacks from deeply nested documents.
/// This limit is sufficient for all reasonable HEDL documents while protecting
/// against malicious input.
///
/// Based on typical stack sizes:
/// - Linux: ~2MB default stack (supports ~100K nesting with 20 bytes/frame)
/// - We use conservative 1000 limit for safety margin
pub(super) const MAX_NESTING_DEPTH: usize = 1000;

// ==================== Indentation Constants ====================

/// Number of spaces per indentation level.
///
/// HEDL v2.0 canonical format uses 1-space indentation for nested structures.
/// This matches the SPEC.md Section 13.2 canonical format requirements.
pub(super) const SPACES_PER_INDENT: usize = 1;

/// Indentation increment for nested objects.
///
/// When recursing into nested objects, indent by one level.
pub(super) const INDENT_INCREMENT: usize = 1;

/// Base indentation level for root document items.
///
/// Root-level items start at indent 0 (no indentation).
pub(super) const ROOT_INDENT_LEVEL: usize = 0;

/// Additional indentation for matrix list rows relative to list declaration.
///
/// Matrix list rows are indented one level beyond the list declaration.
pub(super) const MATRIX_ROW_INDENT_OFFSET: usize = 1;

/// Additional indentation for child rows in nested matrix lists.
///
/// Child rows are indented two levels beyond the parent list declaration.
pub(super) const MATRIX_CHILD_INDENT_OFFSET: usize = 2;

// ==================== Matrix Column Constants ====================

/// Index of the ID column in matrix list rows.
///
/// The first column (index 0) is always the ID column and must never use ditto.
pub(super) const ID_COLUMN_INDEX: usize = 0;

/// Offset for calculating last column index.
///
/// Last column index = `num_cols` - 1 (using this offset).
pub(super) const LAST_COLUMN_OFFSET: usize = 1;

/// Maximum number of children for inline child list format.
///
/// When a parent row has <= this many children of a given type,
/// use the compact inline format: `@ChildType#N:|child1|child2|...`
/// When exceeding this threshold, use expanded format with one child per line.
pub(super) const INLINE_CHILD_THRESHOLD: usize = 5;

// ==================== Error Reporting Constants ====================

/// Line number used for errors without specific source location.
///
/// Used when errors occur during output generation rather than parsing.
/// Since canonicalization operates on AST, not source text, line numbers
/// are not meaningful. Use 0 to indicate "no specific line".
pub(super) const ERROR_LINE_UNKNOWN: usize = 0;

// ==================== Count Initialization Constants ====================

/// Initial value for struct instance count accumulation.
///
/// When counting matrix list instances of each type, start at 0.
pub(super) const INITIAL_STRUCT_COUNT: usize = 0;

// ==================== Float Formatting Constants ====================

/// Fractional part value indicating a whole number.
///
/// For floats where `fract() == 0.0`, the value is a whole number.
/// Example: `42.0.fract()` == 0.0
pub(super) const FLOAT_WHOLE_NUMBER_FRACTIONAL_PART: f64 = 0.0;

/// Number of decimal places for whole number floats.
///
/// Whole numbers are formatted with .1 precision to ensure they display as "X.0".
/// This distinguishes floats from integers in the output.
/// Example: 42.0 formatted as "42.0" not "42"
pub(super) const WHOLE_NUMBER_DECIMAL_PLACES: usize = 1;
