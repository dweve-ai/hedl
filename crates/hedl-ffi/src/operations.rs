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

//! Operations (canonicalize, lint, validate) for FFI.

use crate::audit::{
    audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer,
};
use crate::error::{clear_error, set_error};
use crate::memory::is_valid_document_ptr;
use crate::types::{
    HedlDiagnostics, HedlDocument, HEDL_ERR_CANONICALIZE, HEDL_ERR_NULL_PTR, HEDL_OK,
};
use crate::utils::allocate_output_string;
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::time::Instant;

// =============================================================================
// Canonicalization
// =============================================================================

/// Canonicalize a HEDL document.
///
/// # Arguments
/// * `doc` - Document handle from hedl_parse
/// * `out_str` - Pointer to store canonical output (must be freed with hedl_free_string)
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// All pointers must be valid. Returns HEDL_ERR_NULL_PTR if doc is NULL or poisoned.
#[no_mangle]
pub unsafe extern "C" fn hedl_canonicalize(
    doc: *const HedlDocument,
    out_str: *mut *mut c_char,
) -> c_int {
    let start = Instant::now();

    audit_call_start(
        "hedl_canonicalize",
        &[
            ("doc", &sanitize_pointer(doc)),
            ("out_str", &sanitize_pointer(out_str)),
        ],
    );

    clear_error();

    if !is_valid_document_ptr(doc) || out_str.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure(
            "hedl_canonicalize",
            HEDL_ERR_NULL_PTR,
            "Null pointer argument",
            duration,
        );
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;

    match hedl_c14n::canonicalize(doc_ref) {
        Ok(canonical) => {
            let result = allocate_output_string(&canonical, out_str, HEDL_ERR_CANONICALIZE);
            if result == HEDL_OK {
                audit_call_success("hedl_canonicalize", start.elapsed());
            } else {
                let duration = start.elapsed();
                let msg = crate::error::get_thread_local_error();
                audit_call_failure("hedl_canonicalize", result, &msg, duration);
            }
            result
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("Canonicalization error: {}", e);
            set_error(&msg);
            *out_str = ptr::null_mut();
            audit_call_failure("hedl_canonicalize", HEDL_ERR_CANONICALIZE, &msg, duration);
            HEDL_ERR_CANONICALIZE
        }
    }
}

// =============================================================================
// Linting
// =============================================================================

/// Lint a HEDL document.
///
/// # Arguments
/// * `doc` - Document handle from hedl_parse
/// * `out_diag` - Pointer to store diagnostics handle
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// All pointers must be valid. Returns HEDL_ERR_NULL_PTR if doc is NULL or poisoned.
#[no_mangle]
pub unsafe extern "C" fn hedl_lint(
    doc: *const HedlDocument,
    out_diag: *mut *mut HedlDiagnostics,
) -> c_int {
    let start = Instant::now();

    audit_call_start(
        "hedl_lint",
        &[
            ("doc", &sanitize_pointer(doc)),
            ("out_diag", &sanitize_pointer(out_diag)),
        ],
    );

    clear_error();

    if !is_valid_document_ptr(doc) || out_diag.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure("hedl_lint", HEDL_ERR_NULL_PTR, "Null pointer argument", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;
    let diagnostics = hedl_lint::lint(doc_ref);

    let handle = Box::new(HedlDiagnostics { inner: diagnostics });
    *out_diag = Box::into_raw(handle);
    audit_call_success("hedl_lint", start.elapsed());
    HEDL_OK
}
