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

pub const HEDL_OK: c_int = 0;
pub const HEDL_ERR_NULL_PTR: c_int = -1;
pub const HEDL_ERR_INVALID_UTF8: c_int = -2;
pub const HEDL_ERR_PARSE: c_int = -3;
pub const HEDL_ERR_CANONICALIZE: c_int = -4;
pub const HEDL_ERR_JSON: c_int = -5;
pub const HEDL_ERR_ALLOC: c_int = -6;
pub const HEDL_ERR_YAML: c_int = -7;
pub const HEDL_ERR_XML: c_int = -8;
pub const HEDL_ERR_CSV: c_int = -9;
pub const HEDL_ERR_PARQUET: c_int = -10;
pub const HEDL_ERR_LINT: c_int = -11;
pub const HEDL_ERR_NEO4J: c_int = -12;

// =============================================================================
// Opaque Types
// =============================================================================

/// Opaque handle to a HEDL document
pub struct HedlDocument {
    pub(crate) inner: Document,
}

/// Opaque handle to lint diagnostics
pub struct HedlDiagnostics {
    pub(crate) inner: Vec<hedl_lint::Diagnostic>,
}
