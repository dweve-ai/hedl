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

//! FFI type definitions and error codes.

use hedl_core::Document;
use std::os::raw::c_int;

// =============================================================================
// Error Codes
// =============================================================================

/// Success return code.
pub const HEDL_OK: c_int = 0;
/// Null pointer argument error.
pub const HEDL_ERR_NULL_PTR: c_int = -1;
/// Invalid UTF-8 encoding error.
pub const HEDL_ERR_INVALID_UTF8: c_int = -2;
/// HEDL parsing error.
pub const HEDL_ERR_PARSE: c_int = -3;
/// Canonicalization error.
pub const HEDL_ERR_CANONICALIZE: c_int = -4;
/// JSON conversion error.
pub const HEDL_ERR_JSON: c_int = -5;
/// Memory allocation error.
pub const HEDL_ERR_ALLOC: c_int = -6;
/// YAML conversion error.
pub const HEDL_ERR_YAML: c_int = -7;
/// XML conversion error.
pub const HEDL_ERR_XML: c_int = -8;
/// CSV conversion error.
pub const HEDL_ERR_CSV: c_int = -9;
/// Parquet conversion error.
pub const HEDL_ERR_PARQUET: c_int = -10;
/// Lint validation error.
pub const HEDL_ERR_LINT: c_int = -11;
/// Neo4j export error.
pub const HEDL_ERR_NEO4J: c_int = -12;
/// TOON conversion error.
pub const HEDL_ERR_TOON: c_int = -13;
/// Reentrant call detected (thread safety violation).
pub const HEDL_ERR_REENTRANT_CALL: c_int = -14;
/// Operation was cancelled.
pub const HEDL_ERR_CANCELLED: c_int = -15;
/// Task queue is full.
pub const HEDL_ERR_QUEUE_FULL: c_int = -16;
/// Invalid handle provided.
pub const HEDL_ERR_INVALID_HANDLE: c_int = -17;

// =============================================================================
// Opaque Types
// =============================================================================

/// Opaque handle to a HEDL document
pub struct HedlDocument {
    /// The underlying parsed document.
    pub(crate) inner: Document,
}

/// Opaque handle to lint diagnostics
pub struct HedlDiagnostics {
    /// Vector of lint diagnostic messages.
    pub(crate) inner: Vec<hedl_lint::Diagnostic>,
}
