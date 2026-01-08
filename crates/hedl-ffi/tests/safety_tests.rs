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

//! Comprehensive safety tests for the HEDL FFI interface.
//!
//! These tests verify:
//! - NULL pointer handling
//! - Invalid UTF-8 handling
//! - Double-free protection
//! - Memory leak patterns
//! - Thread-local error storage
//! - Large input handling
//! - Error code correctness
//! - Feature-gated functions
//! - Round-trip conversions

use hedl_ffi::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::ptr;

// Test data
const VALID_HEDL: &[u8] = b"%VERSION: 1.0\n---\nkey: value\0";
const VALID_HEDL_WITH_SCHEMA: &[u8] = b"%VERSION: 1.0\n%STRUCT: Person: [name,age]\n---\ndata: @Person\n  | Alice, 30\0";

const INVALID_HEDL: &[u8] = b"not valid hedl\0";

// =============================================================================
// NULL Pointer Handling Tests
// =============================================================================

#[test]
fn test_hedl_parse_null_input() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(ptr::null(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert!(doc.is_null());

        let err = hedl_get_last_error();
        assert!(!err.is_null());
        let err_msg = CStr::from_ptr(err).to_str().unwrap();
        assert!(err_msg.contains("Null pointer"));
    }
}

#[test]
fn test_hedl_parse_null_out_doc() {
    unsafe {
        let result = hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        let err = hedl_get_last_error();
        assert!(!err.is_null());
    }
}

#[test]
fn test_hedl_validate_null_input() {
    unsafe {
        let result = hedl_validate(ptr::null(), -1, 0);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

#[test]
fn test_hedl_get_version_null_doc() {
    unsafe {
        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        let result = hedl_get_version(ptr::null(), &mut major, &mut minor);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

#[test]
fn test_hedl_get_version_null_major() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);
        assert!(!doc.is_null());

        let mut minor: c_int = 0;
        let result = hedl_get_version(doc, ptr::null_mut(), &mut minor);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_get_version_null_minor() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);
        assert!(!doc.is_null());

        let mut major: c_int = 0;
        let result = hedl_get_version(doc, &mut major, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_canonicalize_null_doc() {
    unsafe {
        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_canonicalize(ptr::null(), &mut out_str);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert!(out_str.is_null());
    }
}

#[test]
fn test_hedl_canonicalize_null_out_str() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let result = hedl_canonicalize(doc, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_lint_null_doc() {
    unsafe {
        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        let result = hedl_lint(ptr::null(), &mut diag);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert!(diag.is_null());
    }
}

#[test]
fn test_hedl_lint_null_out_diag() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let result = hedl_lint(doc, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_diagnostics_count_null() {
    unsafe {
        let count = hedl_diagnostics_count(ptr::null());
        assert_eq!(count, -1);
    }
}

#[test]
fn test_hedl_diagnostics_get_null_diag() {
    unsafe {
        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_diagnostics_get(ptr::null(), 0, &mut out_str);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

#[test]
fn test_hedl_diagnostics_get_null_out_str() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        let result = hedl_diagnostics_get(diag, 0, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_diagnostics_severity_null() {
    unsafe {
        let severity = hedl_diagnostics_severity(ptr::null(), 0);
        assert_eq!(severity, -1);
    }
}

#[test]
fn test_hedl_schema_count_null() {
    unsafe {
        let count = hedl_schema_count(ptr::null());
        assert_eq!(count, -1);
    }
}

#[test]
fn test_hedl_alias_count_null() {
    unsafe {
        let count = hedl_alias_count(ptr::null());
        assert_eq!(count, -1);
    }
}

#[test]
fn test_hedl_root_item_count_null() {
    unsafe {
        let count = hedl_root_item_count(ptr::null());
        assert_eq!(count, -1);
    }
}

// =============================================================================
// Invalid UTF-8 Handling
// =============================================================================

#[test]
fn test_hedl_parse_invalid_utf8() {
    unsafe {
        let invalid_utf8: &[u8] = &[0xFF, 0xFE, 0xFD, 0x00]; // Invalid UTF-8 sequence
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(
            invalid_utf8.as_ptr() as *const c_char,
            invalid_utf8.len() as c_int - 1, // -1 to exclude null terminator
            0,
            &mut doc,
        );

        assert_eq!(result, HEDL_ERR_INVALID_UTF8);
        assert!(doc.is_null());

        let err = hedl_get_last_error();
        assert!(!err.is_null());
        let err_msg = CStr::from_ptr(err).to_str().unwrap();
        assert!(err_msg.contains("Invalid UTF-8"));
    }
}

#[test]
fn test_hedl_validate_invalid_utf8() {
    unsafe {
        let invalid_utf8: &[u8] = &[0xFF, 0xFE, 0xFD, 0x00];
        let result = hedl_validate(
            invalid_utf8.as_ptr() as *const c_char,
            invalid_utf8.len() as c_int - 1,
            0,
        );

        assert_eq!(result, HEDL_ERR_INVALID_UTF8);
    }
}

// =============================================================================
// Double-Free Protection
// =============================================================================

#[test]
fn test_hedl_free_document_null_safe() {
    unsafe {
        // Freeing NULL should be safe
        hedl_free_document(ptr::null_mut());
        // No crash = success
    }
}

#[test]
fn test_hedl_free_string_null_safe() {
    unsafe {
        // Freeing NULL should be safe
        hedl_free_string(ptr::null_mut());
        // No crash = success
    }
}

#[test]
fn test_hedl_free_diagnostics_null_safe() {
    unsafe {
        // Freeing NULL should be safe
        hedl_free_diagnostics(ptr::null_mut());
        // No crash = success
    }
}

#[test]
fn test_hedl_free_bytes_null_safe() {
    unsafe {
        // Freeing NULL with len=0 should be safe
        hedl_free_bytes(ptr::null_mut(), 0);
        // No crash = success
    }
}

// Note: We cannot test actual double-free without causing UB in a safe way.
// However, we can test poison pointer detection by casting the poison value.

// =============================================================================
// Poison Pointer Detection Tests
// =============================================================================

/// Test that poison pointer values are rejected for documents
#[test]
fn test_poison_document_ptr_detection() {
    unsafe {
        // Create a poison pointer (simulating a freed document)
        const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;
        let poisoned_doc = POISON_PTR_DOCUMENT as *const HedlDocument;

        // All accessor functions should reject the poison pointer
        assert_eq!(hedl_schema_count(poisoned_doc), -1);
        assert_eq!(hedl_alias_count(poisoned_doc), -1);
        assert_eq!(hedl_root_item_count(poisoned_doc), -1);

        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        assert_eq!(
            hedl_get_version(poisoned_doc, &mut major, &mut minor),
            HEDL_ERR_NULL_PTR
        );

        let mut out_str: *mut c_char = ptr::null_mut();
        assert_eq!(hedl_canonicalize(poisoned_doc, &mut out_str), HEDL_ERR_NULL_PTR);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        assert_eq!(hedl_lint(poisoned_doc, &mut diag), HEDL_ERR_NULL_PTR);
    }
}

/// Test that poison pointer values are rejected for diagnostics
#[test]
fn test_poison_diagnostics_ptr_detection() {
    unsafe {
        // Create a poison pointer (simulating freed diagnostics)
        const POISON_PTR_DIAGNOSTICS: usize = 0xDEADC0DE;
        let poisoned_diag = POISON_PTR_DIAGNOSTICS as *const HedlDiagnostics;

        // All diagnostics accessor functions should reject the poison pointer
        assert_eq!(hedl_diagnostics_count(poisoned_diag), -1);
        assert_eq!(hedl_diagnostics_severity(poisoned_diag, 0), -1);

        let mut out_str: *mut c_char = ptr::null_mut();
        assert_eq!(
            hedl_diagnostics_get(poisoned_diag, 0, &mut out_str),
            HEDL_ERR_NULL_PTR
        );
    }
}

/// Test that freeing a poison document pointer is safe
#[test]
fn test_free_poison_document_ptr_safe() {
    unsafe {
        const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;
        let poisoned_doc = POISON_PTR_DOCUMENT as *mut HedlDocument;

        // Freeing a poison pointer should be safe (no crash)
        hedl_free_document(poisoned_doc);
        // No crash = success
    }
}

/// Test that freeing a poison diagnostics pointer is safe
#[test]
fn test_free_poison_diagnostics_ptr_safe() {
    unsafe {
        const POISON_PTR_DIAGNOSTICS: usize = 0xDEADC0DE;
        let poisoned_diag = POISON_PTR_DIAGNOSTICS as *mut HedlDiagnostics;

        // Freeing a poison pointer should be safe (no crash)
        hedl_free_diagnostics(poisoned_diag);
        // No crash = success
    }
}

/// Test that double-free protection works when caller sets pointer to poison value
///
/// Note: We cannot actually test double-free in Rust without causing UB.
/// This test documents the intended behavior: callers in C should set their
/// pointers to NULL after freeing to avoid use-after-free.
#[test]
fn test_double_free_prevention_documentation() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);
        assert!(!doc.is_null());

        // Free the document
        hedl_free_document(doc);

        // In C, the caller should set doc = NULL here to prevent use-after-free
        // Since we can't modify the caller's pointer from Rust, we can only
        // check the poison value if they accidentally pass it back

        // This test demonstrates that NULL pointers are safe to free
        hedl_free_document(ptr::null_mut());

        // And that poison pointers are also safe
        const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;
        hedl_free_document(POISON_PTR_DOCUMENT as *mut HedlDocument);
    }
}

/// Test that diagnostics double-free protection works
#[test]
fn test_diagnostics_double_free_prevention() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        // Free the diagnostics
        hedl_free_diagnostics(diag);

        // NULL pointer is safe to free
        hedl_free_diagnostics(ptr::null_mut());

        // Poison pointer is also safe
        const POISON_PTR_DIAGNOSTICS: usize = 0xDEADC0DE;
        hedl_free_diagnostics(POISON_PTR_DIAGNOSTICS as *mut HedlDiagnostics);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_poison_document_ptr_in_conversion_functions() {
    unsafe {
        const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;
        let poisoned_doc = POISON_PTR_DOCUMENT as *const HedlDocument;

        let mut out_str: *mut c_char = ptr::null_mut();

        // JSON conversion should reject poison pointer
        assert_eq!(hedl_to_json(poisoned_doc, 0, &mut out_str), HEDL_ERR_NULL_PTR);
        assert!(out_str.is_null());
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_poison_document_ptr_in_yaml_conversion() {
    unsafe {
        const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;
        let poisoned_doc = POISON_PTR_DOCUMENT as *const HedlDocument;

        let mut out_str: *mut c_char = ptr::null_mut();

        // YAML conversion should reject poison pointer
        assert_eq!(hedl_to_yaml(poisoned_doc, 0, &mut out_str), HEDL_ERR_NULL_PTR);
        assert!(out_str.is_null());
    }
}

#[cfg(feature = "xml")]
#[test]
fn test_poison_document_ptr_in_xml_conversion() {
    unsafe {
        const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;
        let poisoned_doc = POISON_PTR_DOCUMENT as *const HedlDocument;

        let mut out_str: *mut c_char = ptr::null_mut();

        // XML conversion should reject poison pointer
        assert_eq!(hedl_to_xml(poisoned_doc, &mut out_str), HEDL_ERR_NULL_PTR);
        assert!(out_str.is_null());
    }
}

#[cfg(feature = "csv")]
#[test]
fn test_poison_document_ptr_in_csv_conversion() {
    unsafe {
        const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;
        let poisoned_doc = POISON_PTR_DOCUMENT as *const HedlDocument;

        let mut out_str: *mut c_char = ptr::null_mut();

        // CSV conversion should reject poison pointer
        assert_eq!(hedl_to_csv(poisoned_doc, &mut out_str), HEDL_ERR_NULL_PTR);
        assert!(out_str.is_null());
    }
}

#[cfg(feature = "parquet")]
#[test]
fn test_poison_document_ptr_in_parquet_conversion() {
    unsafe {
        const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;
        let poisoned_doc = POISON_PTR_DOCUMENT as *const HedlDocument;

        let mut out_data: *mut u8 = ptr::null_mut();
        let mut out_len: usize = 0;

        // Parquet conversion should reject poison pointer
        assert_eq!(
            hedl_to_parquet(poisoned_doc, &mut out_data, &mut out_len),
            HEDL_ERR_NULL_PTR
        );
        assert!(out_data.is_null());
        assert_eq!(out_len, 0);
    }
}

#[cfg(feature = "neo4j")]
#[test]
fn test_poison_document_ptr_in_neo4j_conversion() {
    unsafe {
        const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;
        let poisoned_doc = POISON_PTR_DOCUMENT as *const HedlDocument;

        let mut out_str: *mut c_char = ptr::null_mut();

        // Neo4j conversion should reject poison pointer
        assert_eq!(
            hedl_to_neo4j_cypher(poisoned_doc, 1, &mut out_str),
            HEDL_ERR_NULL_PTR
        );
        assert!(out_str.is_null());
    }
}

// =============================================================================
// Memory Leak Detection Patterns
// =============================================================================

#[test]
fn test_hedl_parse_error_cleanup() {
    unsafe {
        // When parse fails, the out_doc should be NULL
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(INVALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        assert_ne!(result, HEDL_OK);
        assert!(doc.is_null()); // Should not leak memory
    }
}

#[test]
fn test_hedl_canonicalize_error_cleanup() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        // Try to canonicalize with NULL out_str
        let result = hedl_canonicalize(doc, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // Document should still be valid
        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        assert_eq!(hedl_get_version(doc, &mut major, &mut minor), HEDL_OK);

        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_multiple_operations_same_doc() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);
        assert!(!doc.is_null());

        // Multiple operations on same doc
        let mut out_str: *mut c_char = ptr::null_mut();
        assert_eq!(hedl_canonicalize(doc, &mut out_str), HEDL_OK);
        hedl_free_string(out_str);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        assert_eq!(hedl_lint(doc, &mut diag), HEDL_OK);
        hedl_free_diagnostics(diag);

        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        assert_eq!(hedl_get_version(doc, &mut major, &mut minor), HEDL_OK);

        // Clean up
        hedl_free_document(doc);
    }
}

// =============================================================================
// Thread-Local Error Storage
// =============================================================================

#[test]
fn test_hedl_get_last_error_null_when_no_error() {
    unsafe {
        // Parse successfully
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        // Error should be cleared after successful operation
        let err = hedl_get_last_error();
        assert!(err.is_null());

        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_get_last_error_persists() {
    unsafe {
        // Trigger an error
        let result = hedl_validate(INVALID_HEDL.as_ptr() as *const c_char, -1, 0);
        assert_ne!(result, HEDL_OK);

        // Error should be retrievable
        let err1 = hedl_get_last_error();
        assert!(!err1.is_null());
        let msg1 = CStr::from_ptr(err1).to_str().unwrap();

        // Multiple calls should return same pointer
        let err2 = hedl_get_last_error();
        assert_eq!(err1, err2);
        let msg2 = CStr::from_ptr(err2).to_str().unwrap();
        assert_eq!(msg1, msg2);
    }
}

#[test]
fn test_hedl_error_cleared_on_success() {
    unsafe {
        // Trigger an error first
        hedl_validate(INVALID_HEDL.as_ptr() as *const c_char, -1, 0);
        assert!(!hedl_get_last_error().is_null());

        // Successful operation should clear error
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let err = hedl_get_last_error();
        assert!(err.is_null());

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_hedl_thread_local_errors_in_threads() {
    use std::thread;
    use std::sync::Arc;

    unsafe {
        // Create errors in parallel threads
        let invalid = Arc::new(INVALID_HEDL.to_vec());
        let valid = Arc::new(VALID_HEDL.to_vec());

        let invalid_clone = invalid.clone();
        let handle1 = thread::spawn(move || {
            let result = hedl_validate(invalid_clone.as_ptr() as *const c_char, -1, 0);
            assert_ne!(result, HEDL_OK);

            let err = hedl_get_last_error();
            assert!(!err.is_null());
            let msg = CStr::from_ptr(err).to_str().unwrap();
            msg.to_string()
        });

        let valid_clone = valid.clone();
        let handle2 = thread::spawn(move || {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            hedl_parse(valid_clone.as_ptr() as *const c_char, -1, 0, &mut doc);

            let err = hedl_get_last_error();
            let has_error = !err.is_null();

            hedl_free_document(doc);
            has_error
        });

        let error_msg = handle1.join().unwrap();
        let had_error = handle2.join().unwrap();

        assert!(error_msg.contains("Parse error"));
        assert!(!had_error); // Thread 2 should not see thread 1's error
    }
}

// =============================================================================
// Large Input Handling
// =============================================================================

#[test]
fn test_hedl_parse_large_input() {
    unsafe {
        // Create a large but valid HEDL document (under MAX_FFI_INPUT_LEN)
        let large_size = 1024 * 1024; // 1MB
        let mut large_doc = Vec::with_capacity(large_size + 100);
        large_doc.extend_from_slice(b"%VERSION: 1.0\n---\n");

        // Add many key-value pairs
        for i in 0..10000 {
            large_doc.extend_from_slice(format!("key{}: value{}\n", i, i).as_bytes());
        }
        large_doc.push(0); // Null terminator

        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(large_doc.as_ptr() as *const c_char, -1, 0, &mut doc);

        // Should succeed
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_parse_exact_length_input() {
    unsafe {
        // Test with exact length (no null terminator)
        let input = b"%VERSION: 1.0\n---\nkey: value";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(
            input.as_ptr() as *const c_char,
            input.len() as c_int,
            0,
            &mut doc,
        );

        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_parse_extremely_large_input() {
    unsafe {
        // Test rejection of input exceeding MAX_FFI_INPUT_LEN (1GB)
        let huge_size = (1024u64 * 1024 * 1024 + 1) as c_int; // 1GB + 1
        let small_buf = b"test\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(
            small_buf.as_ptr() as *const c_char,
            huge_size,
            0,
            &mut doc,
        );

        // Should be rejected
        assert_eq!(result, HEDL_ERR_INVALID_UTF8);
        assert!(doc.is_null());

        let err = hedl_get_last_error();
        assert!(!err.is_null());
        let err_msg = CStr::from_ptr(err).to_str().unwrap();
        assert!(err_msg.contains("exceeds maximum"));
    }
}

// =============================================================================
// Error Code Verification
// =============================================================================

#[test]
fn test_all_error_codes_defined() {
    // Verify all error codes are distinct
    let codes = [
        HEDL_OK,
        HEDL_ERR_NULL_PTR,
        HEDL_ERR_INVALID_UTF8,
        HEDL_ERR_PARSE,
        HEDL_ERR_CANONICALIZE,
        HEDL_ERR_JSON,
        HEDL_ERR_ALLOC,
        HEDL_ERR_YAML,
        HEDL_ERR_XML,
        HEDL_ERR_CSV,
        HEDL_ERR_PARQUET,
        HEDL_ERR_LINT,
        HEDL_ERR_NEO4J,
    ];

    for (i, &code1) in codes.iter().enumerate() {
        for (j, &code2) in codes.iter().enumerate() {
            if i != j {
                assert_ne!(code1, code2, "Error codes {} and {} are equal", i, j);
            }
        }
    }

    // Verify HEDL_OK is 0
    assert_eq!(HEDL_OK, 0);

    // Verify all errors are negative
    for &code in &codes[1..] {
        assert!(code < 0, "Error code {} should be negative", code);
    }
}

#[test]
fn test_hedl_parse_returns_correct_error() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();

        // NULL pointer error
        let result = hedl_parse(ptr::null(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // Invalid UTF-8 error
        let invalid_utf8: &[u8] = &[0xFF, 0xFE, 0x00];
        let result = hedl_parse(
            invalid_utf8.as_ptr() as *const c_char,
            invalid_utf8.len() as c_int - 1,
            0,
            &mut doc,
        );
        assert_eq!(result, HEDL_ERR_INVALID_UTF8);

        // Parse error
        let result = hedl_parse(INVALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);
        assert_eq!(result, HEDL_ERR_PARSE);
    }
}

// =============================================================================
// Feature-Gated Functions
// =============================================================================

#[cfg(feature = "json")]
#[test]
fn test_hedl_to_json_null_checks() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();

        // NULL doc
        let result = hedl_to_json(ptr::null(), 0, &mut out_str);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_str
        let result = hedl_to_json(doc, 0, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_hedl_from_json_null_checks() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let json = b"{\"key\": \"value\"}\0";

        // NULL json
        let result = hedl_from_json(ptr::null(), -1, &mut doc);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_doc
        let result = hedl_from_json(json.as_ptr() as *const c_char, -1, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_hedl_to_yaml_null_checks() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();

        // NULL doc
        let result = hedl_to_yaml(ptr::null(), 0, &mut out_str);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_str
        let result = hedl_to_yaml(doc, 0, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_hedl_from_yaml_null_checks() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let yaml = b"key: value\0";

        // NULL yaml
        let result = hedl_from_yaml(ptr::null(), -1, &mut doc);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_doc
        let result = hedl_from_yaml(yaml.as_ptr() as *const c_char, -1, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

#[cfg(feature = "xml")]
#[test]
fn test_hedl_to_xml_null_checks() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();

        // NULL doc
        let result = hedl_to_xml(ptr::null(), &mut out_str);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_str
        let result = hedl_to_xml(doc, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "xml")]
#[test]
fn test_hedl_from_xml_null_checks() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let xml = b"<?xml version=\"1.0\"?><root/>\0";

        // NULL xml
        let result = hedl_from_xml(ptr::null(), -1, &mut doc);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_doc
        let result = hedl_from_xml(xml.as_ptr() as *const c_char, -1, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

#[cfg(feature = "csv")]
#[test]
fn test_hedl_to_csv_null_checks() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();

        // NULL doc
        let result = hedl_to_csv(ptr::null(), &mut out_str);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_str
        let result = hedl_to_csv(doc, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "parquet")]
#[test]
fn test_hedl_to_parquet_null_checks() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut out_data: *mut u8 = ptr::null_mut();
        let mut out_len: usize = 0;

        // NULL doc
        let result = hedl_to_parquet(ptr::null(), &mut out_data, &mut out_len);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_data
        let result = hedl_to_parquet(doc, ptr::null_mut(), &mut out_len);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_len
        let result = hedl_to_parquet(doc, &mut out_data, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "parquet")]
#[test]
fn test_hedl_from_parquet_null_checks() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let data = [0u8; 10];

        // NULL data
        let result = hedl_from_parquet(ptr::null(), 10, &mut doc);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_doc
        let result = hedl_from_parquet(data.as_ptr(), 10, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

#[cfg(feature = "neo4j")]
#[test]
fn test_hedl_to_neo4j_cypher_null_checks() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();

        // NULL doc
        let result = hedl_to_neo4j_cypher(ptr::null(), 1, &mut out_str);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // NULL out_str
        let result = hedl_to_neo4j_cypher(doc, 1, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

// =============================================================================
// Round-Trip Conversions
// =============================================================================

#[cfg(feature = "json")]
#[test]
fn test_json_roundtrip() {
    unsafe {
        // Parse HEDL
        let mut doc1: *mut HedlDocument = ptr::null_mut();
        assert_eq!(hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc1), HEDL_OK);

        // Convert to JSON with metadata
        let mut json_str: *mut c_char = ptr::null_mut();
        assert_eq!(hedl_to_json(doc1, 1, &mut json_str), HEDL_OK);
        assert!(!json_str.is_null());

        // Parse JSON back
        let mut doc2: *mut HedlDocument = ptr::null_mut();
        assert_eq!(hedl_from_json(json_str, -1, &mut doc2), HEDL_OK);
        assert!(!doc2.is_null());

        // Verify versions match
        let mut major1: c_int = 0;
        let mut minor1: c_int = 0;
        let mut major2: c_int = 0;
        let mut minor2: c_int = 0;

        assert_eq!(hedl_get_version(doc1, &mut major1, &mut minor1), HEDL_OK);
        assert_eq!(hedl_get_version(doc2, &mut major2, &mut minor2), HEDL_OK);

        assert_eq!(major1, major2);
        assert_eq!(minor1, minor2);

        hedl_free_string(json_str);
        hedl_free_document(doc1);
        hedl_free_document(doc2);
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_yaml_roundtrip() {
    unsafe {
        // Parse HEDL
        let mut doc1: *mut HedlDocument = ptr::null_mut();
        assert_eq!(hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc1), HEDL_OK);

        // Convert to YAML with metadata
        let mut yaml_str: *mut c_char = ptr::null_mut();
        assert_eq!(hedl_to_yaml(doc1, 1, &mut yaml_str), HEDL_OK);
        assert!(!yaml_str.is_null());

        // Parse YAML back
        let mut doc2: *mut HedlDocument = ptr::null_mut();
        assert_eq!(hedl_from_yaml(yaml_str, -1, &mut doc2), HEDL_OK);
        assert!(!doc2.is_null());

        // Verify versions match
        let mut major1: c_int = 0;
        let mut minor1: c_int = 0;
        let mut major2: c_int = 0;
        let mut minor2: c_int = 0;

        assert_eq!(hedl_get_version(doc1, &mut major1, &mut minor1), HEDL_OK);
        assert_eq!(hedl_get_version(doc2, &mut major2, &mut minor2), HEDL_OK);

        assert_eq!(major1, major2);
        assert_eq!(minor1, minor2);

        hedl_free_string(yaml_str);
        hedl_free_document(doc1);
        hedl_free_document(doc2);
    }
}

#[cfg(feature = "xml")]
#[test]
fn test_xml_roundtrip() {
    unsafe {
        // Parse HEDL
        let mut doc1: *mut HedlDocument = ptr::null_mut();
        assert_eq!(hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc1), HEDL_OK);

        // Convert to XML
        let mut xml_str: *mut c_char = ptr::null_mut();
        assert_eq!(hedl_to_xml(doc1, &mut xml_str), HEDL_OK);
        assert!(!xml_str.is_null());

        // Parse XML back
        let mut doc2: *mut HedlDocument = ptr::null_mut();
        assert_eq!(hedl_from_xml(xml_str, -1, &mut doc2), HEDL_OK);
        assert!(!doc2.is_null());

        // Verify versions match
        let mut major1: c_int = 0;
        let mut minor1: c_int = 0;
        let mut major2: c_int = 0;
        let mut minor2: c_int = 0;

        assert_eq!(hedl_get_version(doc1, &mut major1, &mut minor1), HEDL_OK);
        assert_eq!(hedl_get_version(doc2, &mut major2, &mut minor2), HEDL_OK);

        assert_eq!(major1, major2);
        assert_eq!(minor1, minor2);

        hedl_free_string(xml_str);
        hedl_free_document(doc1);
        hedl_free_document(doc2);
    }
}

// =============================================================================
// Document Count Functions
// =============================================================================

#[test]
fn test_hedl_schema_count() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(VALID_HEDL_WITH_SCHEMA.as_ptr() as *const c_char, -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK, "Failed to parse schema document");
        assert!(!doc.is_null());

        let count = hedl_schema_count(doc);
        assert_eq!(count, 1); // One struct defined

        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_root_item_count() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(VALID_HEDL_WITH_SCHEMA.as_ptr() as *const c_char, -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK, "Failed to parse schema document");
        assert!(!doc.is_null());

        let count = hedl_root_item_count(doc);
        assert_eq!(count, 1); // One instance in root

        hedl_free_document(doc);
    }
}

// =============================================================================
// Diagnostics Tests
// =============================================================================

#[test]
fn test_hedl_diagnostics_index_out_of_range() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        let count = hedl_diagnostics_count(diag);

        // Try to get diagnostic beyond range
        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_diagnostics_get(diag, count + 10, &mut out_str);
        assert_eq!(result, HEDL_ERR_LINT);
        assert!(out_str.is_null());

        // Try negative index
        let result = hedl_diagnostics_get(diag, -1, &mut out_str);
        assert_eq!(result, HEDL_ERR_LINT);

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_diagnostics_severity_out_of_range() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        let count = hedl_diagnostics_count(diag);

        // Try to get severity beyond range
        let severity = hedl_diagnostics_severity(diag, count + 10);
        assert_eq!(severity, -1);

        // Try negative index
        let severity = hedl_diagnostics_severity(diag, -1);
        assert_eq!(severity, -1);

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_diagnostics_all_severities() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        let count = hedl_diagnostics_count(diag);

        // Check all diagnostics have valid severities (0=Hint, 1=Warning, 2=Error)
        for i in 0..count {
            let severity = hedl_diagnostics_severity(diag, i);
            assert!(severity >= 0 && severity <= 2);

            // Also verify we can get the message
            let mut msg_str: *mut c_char = ptr::null_mut();
            let result = hedl_diagnostics_get(diag, i, &mut msg_str);
            assert_eq!(result, HEDL_OK);
            assert!(!msg_str.is_null());

            hedl_free_string(msg_str);
        }

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

// =============================================================================
// Canonicalization Tests
// =============================================================================

#[test]
fn test_hedl_canonicalize_deterministic() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

        // Canonicalize multiple times
        let mut canon1: *mut c_char = ptr::null_mut();
        assert_eq!(hedl_canonicalize(doc, &mut canon1), HEDL_OK);

        let mut canon2: *mut c_char = ptr::null_mut();
        assert_eq!(hedl_canonicalize(doc, &mut canon2), HEDL_OK);

        // Should be identical
        let str1 = CStr::from_ptr(canon1).to_str().unwrap();
        let str2 = CStr::from_ptr(canon2).to_str().unwrap();
        assert_eq!(str1, str2);

        hedl_free_string(canon1);
        hedl_free_string(canon2);
        hedl_free_document(doc);
    }
}

// =============================================================================
// Strict Mode Tests
// =============================================================================

#[test]
fn test_hedl_parse_strict_mode() {
    unsafe {
        // Valid document should parse in both modes
        let mut doc: *mut HedlDocument = ptr::null_mut();

        // Non-strict
        assert_eq!(hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc), HEDL_OK);
        hedl_free_document(doc);

        // Strict
        doc = ptr::null_mut();
        assert_eq!(hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 1, &mut doc), HEDL_OK);
        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_validate_strict_mode() {
    unsafe {
        // Valid document should validate in both modes
        assert_eq!(hedl_validate(VALID_HEDL.as_ptr() as *const c_char, -1, 0), HEDL_OK);
        assert_eq!(hedl_validate(VALID_HEDL.as_ptr() as *const c_char, -1, 1), HEDL_OK);
    }
}

// =============================================================================
// Integration Tests
// =============================================================================

#[test]
fn test_full_workflow() {
    unsafe {
        // 1. Parse
        let mut doc: *mut HedlDocument = ptr::null_mut();
        assert_eq!(hedl_parse(VALID_HEDL_WITH_SCHEMA.as_ptr() as *const c_char, -1, 0, &mut doc), HEDL_OK);
        assert!(!doc.is_null());

        // 2. Get version
        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        assert_eq!(hedl_get_version(doc, &mut major, &mut minor), HEDL_OK);
        assert_eq!(major, 1);
        assert_eq!(minor, 0);

        // 3. Get counts
        assert_eq!(hedl_schema_count(doc), 1);
        assert_eq!(hedl_root_item_count(doc), 1);

        // 4. Canonicalize
        let mut canon: *mut c_char = ptr::null_mut();
        assert_eq!(hedl_canonicalize(doc, &mut canon), HEDL_OK);
        assert!(!canon.is_null());

        // 5. Lint
        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        assert_eq!(hedl_lint(doc, &mut diag), HEDL_OK);
        assert!(!diag.is_null());

        let diag_count = hedl_diagnostics_count(diag);
        assert!(diag_count >= 0);

        // 6. Clean up
        hedl_free_string(canon);
        hedl_free_diagnostics(diag);
        hedl_free_document(doc);

        // 7. Verify error is cleared
        assert!(hedl_get_last_error().is_null());
    }
}
