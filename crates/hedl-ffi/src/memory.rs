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

//! Memory management functions for FFI.

use crate::types::{HedlDiagnostics, HedlDocument};
use std::ffi::CString;
use std::os::raw::c_char;

// =============================================================================
// Security Constants
// =============================================================================

/// Poison pointer value to detect double-free and use-after-free bugs.
///
/// This value is chosen to be:
/// - An invalid memory address (not aligned, in kernel space on most systems)
/// - Easily recognizable in debuggers (0xDEADBEEF)
/// - Likely to cause immediate crashes if dereferenced
///
/// While we cannot modify the caller's pointer in C, we use this value internally
/// to track freed memory and detect bugs in accessor functions.
pub(crate) const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;
pub(crate) const POISON_PTR_DIAGNOSTICS: usize = 0xDEADC0DE;

// =============================================================================
// Pointer Validation
// =============================================================================

/// Check if a document pointer is valid (not NULL and not poisoned).
///
/// # Safety
/// This function performs basic pointer validation but does not guarantee
/// the pointer points to valid memory. It only checks for NULL and poison values.
#[inline]
pub(crate) unsafe fn is_valid_document_ptr(doc: *const HedlDocument) -> bool {
    !doc.is_null() && (doc as usize) != POISON_PTR_DOCUMENT
}

/// Check if a diagnostics pointer is valid (not NULL and not poisoned).
///
/// # Safety
/// This function performs basic pointer validation but does not guarantee
/// the pointer points to valid memory. It only checks for NULL and poison values.
#[inline]
pub(crate) unsafe fn is_valid_diagnostics_ptr(diag: *const HedlDiagnostics) -> bool {
    !diag.is_null() && (diag as usize) != POISON_PTR_DIAGNOSTICS
}

// =============================================================================
// Memory Management Functions
// =============================================================================

/// Free a string allocated by HEDL functions.
///
/// # Safety
///
/// **CRITICAL:** The pointer MUST have been returned by a `hedl_*` function
/// that allocates strings (e.g., `hedl_to_json`, `hedl_canonicalize`).
///
/// **Undefined behavior will occur if you pass:**
/// - Pointers from `malloc`/`calloc`/`realloc`
/// - Stack-allocated strings
/// - Already-freed pointers (double free)
/// - Pointers from other libraries
///
/// NULL pointers are safely ignored.
#[no_mangle]
pub unsafe extern "C" fn hedl_free_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

/// Free a document handle.
///
/// # Safety
///
/// The pointer must have been returned by hedl_parse or hedl_from_*.
///
/// **Double-free protection:** If the pointer is NULL or the poison value,
/// this function returns safely without attempting to free.
///
/// **Note**: Since C passes pointers by value, we cannot modify the caller's
/// pointer. Callers should manually set their pointers to NULL after freeing
/// to avoid use-after-free bugs.
#[no_mangle]
pub unsafe extern "C" fn hedl_free_document(doc: *mut HedlDocument) {
    // Check for NULL or poison pointer
    if doc.is_null() || (doc as usize) == POISON_PTR_DOCUMENT {
        return;
    }

    // Free the actual document
    let _ = Box::from_raw(doc);

    // Note: We cannot modify the caller's pointer in C (passed by value),
    // but we've validated against the poison value to detect double-frees
    // if the caller maintains a poisoned pointer.
}

/// Free a diagnostics handle.
///
/// # Safety
///
/// The pointer must have been returned by hedl_lint.
///
/// **Double-free protection:** If the pointer is NULL or the poison value,
/// this function returns safely without attempting to free.
///
/// **Note**: Since C passes pointers by value, we cannot modify the caller's
/// pointer. Callers should manually set their pointers to NULL after freeing
/// to avoid use-after-free bugs.
#[no_mangle]
pub unsafe extern "C" fn hedl_free_diagnostics(diag: *mut HedlDiagnostics) {
    // Check for NULL or poison pointer
    if diag.is_null() || (diag as usize) == POISON_PTR_DIAGNOSTICS {
        return;
    }

    // Free the actual diagnostics
    let _ = Box::from_raw(diag);

    // Note: We cannot modify the caller's pointer in C (passed by value),
    // but we've validated against the poison value to detect double-frees
    // if the caller maintains a poisoned pointer.
}

/// Free byte array allocated by HEDL functions (e.g., `hedl_to_parquet`).
///
/// # Arguments
/// * `data` - Pointer to the byte array
/// * `len` - Length that was returned with the data (MUST match exactly)
///
/// # Safety
///
/// **CRITICAL:** The pointer MUST have been returned by a `hedl_*` function
/// that allocates byte arrays (e.g., `hedl_to_parquet`).
///
/// **Undefined behavior will occur if you pass:**
/// - Pointers from `malloc`/`calloc`/`realloc`
/// - Stack-allocated arrays
/// - Already-freed pointers (double free)
/// - Pointers from other libraries
/// - Incorrect length (MUST match the length returned by the allocating function)
///
/// NULL pointers are safely ignored (when len is 0).
///
/// # Feature
/// Always available, but only useful with "parquet" feature.
#[no_mangle]
pub unsafe extern "C" fn hedl_free_bytes(data: *mut u8, len: usize) {
    if !data.is_null() && len > 0 {
        let _ = Vec::from_raw_parts(data, len, len);
    }
}
