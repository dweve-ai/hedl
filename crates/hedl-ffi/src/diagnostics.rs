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

//! Diagnostics accessor functions for FFI.

use crate::error::set_error;
use crate::memory::is_valid_diagnostics_ptr;
use crate::types::{HedlDiagnostics, HEDL_ERR_LINT, HEDL_ERR_NULL_PTR};
use crate::utils::allocate_output_string;
use std::os::raw::{c_char, c_int};
use std::ptr;

// =============================================================================
// Diagnostics Accessors
// =============================================================================

/// Get the number of diagnostics.
///
/// # Safety
/// Pointer must be valid. Returns -1 if diag is NULL or poisoned.
#[no_mangle]
pub unsafe extern "C" fn hedl_diagnostics_count(diag: *const HedlDiagnostics) -> c_int {
    if !is_valid_diagnostics_ptr(diag) {
        return -1;
    }
    (*diag).inner.len() as c_int
}

/// Get a diagnostic message.
///
/// # Arguments
/// * `diag` - Diagnostics handle
/// * `index` - Diagnostic index
/// * `out_str` - Pointer to store message (must be freed with hedl_free_string)
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// All pointers must be valid. Returns HEDL_ERR_NULL_PTR if diag is NULL or poisoned.
#[no_mangle]
pub unsafe extern "C" fn hedl_diagnostics_get(
    diag: *const HedlDiagnostics,
    index: c_int,
    out_str: *mut *mut c_char,
) -> c_int {
    if !is_valid_diagnostics_ptr(diag) || out_str.is_null() {
        return HEDL_ERR_NULL_PTR;
    }

    let diagnostics = &(*diag).inner;
    if index < 0 || index as usize >= diagnostics.len() {
        set_error("Diagnostic index out of range");
        *out_str = ptr::null_mut();
        return HEDL_ERR_LINT;
    }

    let msg = diagnostics[index as usize].to_string();
    allocate_output_string(&msg, out_str, HEDL_ERR_LINT)
}

/// Get a diagnostic severity (0=Hint, 1=Warning, 2=Error).
///
/// # Safety
/// Pointer must be valid. Returns -1 if diag is NULL or poisoned.
#[no_mangle]
pub unsafe extern "C" fn hedl_diagnostics_severity(
    diag: *const HedlDiagnostics,
    index: c_int,
) -> c_int {
    if !is_valid_diagnostics_ptr(diag) {
        return -1;
    }

    let diagnostics = &(*diag).inner;
    if index < 0 || index as usize >= diagnostics.len() {
        return -1;
    }

    match diagnostics[index as usize].severity() {
        hedl_lint::Severity::Hint => 0,
        hedl_lint::Severity::Warning => 1,
        hedl_lint::Severity::Error => 2,
    }
}
