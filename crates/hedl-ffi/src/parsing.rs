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

//! Parsing functions for FFI.

use crate::audit::{
    audit_call_failure, audit_call_start, audit_call_success, sanitize_c_string, sanitize_pointer,
};
use crate::error::{clear_error, set_error};
use crate::memory::{hedl_free_document, is_valid_document_ptr};
use crate::types::{HedlDocument, HEDL_ERR_NULL_PTR, HEDL_ERR_PARSE, HEDL_OK};
use crate::utils::get_input_string;
use hedl_core::{parse_with_limits, ParseOptions};
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::time::Instant;

// =============================================================================
// Parsing and Validation
// =============================================================================

/// Parse a HEDL document from a string.
///
/// # Arguments
/// * `input` - UTF-8 encoded HEDL document
/// * `input_len` - Length of input in bytes, or -1 for null-terminated
/// * `strict` - Non-zero for strict mode (validate references)
/// * `out_doc` - Pointer to store document handle
///
/// # Returns
/// HEDL_OK on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn hedl_parse(
    input: *const c_char,
    input_len: c_int,
    strict: c_int,
    out_doc: *mut *mut HedlDocument,
) -> c_int {
    let start = Instant::now();
    let input_preview = sanitize_c_string(input, 64);

    audit_call_start(
        "hedl_parse",
        &[
            ("input_ptr", &sanitize_pointer(input)),
            ("input_preview", &input_preview),
            ("input_len", &input_len.to_string()),
            ("strict", &strict.to_string()),
            ("out_doc", &sanitize_pointer(out_doc)),
        ],
    );

    clear_error();

    if input.is_null() || out_doc.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure("hedl_parse", HEDL_ERR_NULL_PTR, "Null pointer argument", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let input_str = match get_input_string(input, input_len) {
        Ok(s) => s,
        Err(code) => {
            let duration = start.elapsed();
            let msg = crate::error::get_thread_local_error();
            audit_call_failure("hedl_parse", code, &msg, duration);
            return code;
        }
    };

    let options = ParseOptions {
        strict_refs: strict != 0,
        ..Default::default()
    };

    match parse_with_limits(input_str.as_bytes(), options) {
        Ok(doc) => {
            let handle = Box::new(HedlDocument { inner: doc });
            *out_doc = Box::into_raw(handle);
            audit_call_success("hedl_parse", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("Parse error: {}", e);
            set_error(&msg);
            *out_doc = ptr::null_mut();
            audit_call_failure("hedl_parse", HEDL_ERR_PARSE, &msg, duration);
            HEDL_ERR_PARSE
        }
    }
}

/// Validate a HEDL document string.
///
/// # Arguments
/// * `input` - UTF-8 encoded HEDL document
/// * `input_len` - Length of input in bytes, or -1 for null-terminated
/// * `strict` - Non-zero for strict mode
///
/// # Returns
/// HEDL_OK if valid, error code if invalid.
///
/// # Safety
/// Input pointer must be valid.
#[no_mangle]
pub unsafe extern "C" fn hedl_validate(
    input: *const c_char,
    input_len: c_int,
    strict: c_int,
) -> c_int {
    let mut doc: *mut HedlDocument = ptr::null_mut();
    let result = hedl_parse(input, input_len, strict, &mut doc);
    if !doc.is_null() {
        hedl_free_document(doc);
    }
    result
}

// =============================================================================
// Document Information
// =============================================================================

/// Get the HEDL version of a parsed document.
///
/// # Safety
/// All pointers must be valid. Returns HEDL_ERR_NULL_PTR if doc is NULL or poisoned.
#[no_mangle]
pub unsafe extern "C" fn hedl_get_version(
    doc: *const HedlDocument,
    major: *mut c_int,
    minor: *mut c_int,
) -> c_int {
    if !is_valid_document_ptr(doc) || major.is_null() || minor.is_null() {
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;
    *major = doc_ref.version.0 as c_int;
    *minor = doc_ref.version.1 as c_int;
    HEDL_OK
}

/// Get the number of struct definitions in a document.
///
/// # Safety
/// Doc pointer must be valid. Returns -1 if doc is NULL or poisoned.
#[no_mangle]
pub unsafe extern "C" fn hedl_schema_count(doc: *const HedlDocument) -> c_int {
    if !is_valid_document_ptr(doc) {
        return -1;
    }
    (*doc).inner.structs.len() as c_int
}

/// Get the number of aliases in a document.
///
/// # Safety
/// Doc pointer must be valid. Returns -1 if doc is NULL or poisoned.
#[no_mangle]
pub unsafe extern "C" fn hedl_alias_count(doc: *const HedlDocument) -> c_int {
    if !is_valid_document_ptr(doc) {
        return -1;
    }
    (*doc).inner.aliases.len() as c_int
}

/// Get the number of root items in a document.
///
/// # Safety
/// Doc pointer must be valid. Returns -1 if doc is NULL or poisoned.
#[no_mangle]
pub unsafe extern "C" fn hedl_root_item_count(doc: *const HedlDocument) -> c_int {
    if !is_valid_document_ptr(doc) {
        return -1;
    }
    (*doc).inner.root.len() as c_int
}
