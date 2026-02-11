// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Memory management edge case tests.

use hedl_ffi::*;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

#[test]
fn test_free_string_allocated_by_hedl() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();
        hedl_canonicalize(doc, &mut out_str);

        assert!(!out_str.is_null());

        // This should not crash
        hedl_free_string(out_str);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "parquet")]
#[test]
fn test_free_bytes_with_various_lengths() {
    // SAFETY: Unsafe operation required for FFI boundary
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ndata: [{ id: 1 }]\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut out_data: *mut u8 = ptr::null_mut();
        let mut out_len: usize = 0;
        let result = hedl_to_parquet(doc, &mut out_data, &mut out_len);

        if result == HEDL_OK && !out_data.is_null() {
            // Free with exact length
            hedl_free_bytes(out_data, out_len);
        }

        hedl_free_document(doc);
    }
}

#[test]
fn test_multiple_string_allocations_and_frees() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        for _ in 0..100 {
            let mut out_str: *mut c_char = ptr::null_mut();
            hedl_canonicalize(doc, &mut out_str);
            assert!(!out_str.is_null());
            hedl_free_string(out_str);
        }

        hedl_free_document(doc);
    }
}

#[test]
fn test_document_lifetime_separate_from_outputs() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut canon1: *mut c_char = ptr::null_mut();
        let mut canon2: *mut c_char = ptr::null_mut();

        hedl_canonicalize(doc, &mut canon1);
        hedl_canonicalize(doc, &mut canon2);

        // Free document first
        hedl_free_document(doc);

        // Should still be able to free strings
        hedl_free_string(canon1);
        hedl_free_string(canon2);
    }
}

#[test]
fn test_diagnostics_lifetime_separate_from_document() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        // Free document first
        hedl_free_document(doc);

        // Should still be able to access and free diagnostics
        let count = hedl_diagnostics_count(diag);
        assert!(count >= 0);

        hedl_free_diagnostics(diag);
    }
}

#[test]
fn test_output_strings_are_independent() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut str1: *mut c_char = ptr::null_mut();
        let mut str2: *mut c_char = ptr::null_mut();

        hedl_canonicalize(doc, &mut str1);
        hedl_canonicalize(doc, &mut str2);

        // Pointers should be different
        assert_ne!(str1, str2);

        // Free in different order
        hedl_free_string(str2);
        hedl_free_string(str1);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_json_string_allocation() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut json_str: *mut c_char = ptr::null_mut();
        hedl_to_json(doc, 0, &mut json_str);

        assert!(!json_str.is_null());

        // Verify we can read it
        use std::ffi::CStr;
        let json = CStr::from_ptr(json_str).to_str().unwrap();
        assert!(!json.is_empty());

        hedl_free_string(json_str);
        hedl_free_document(doc);
    }
}

#[test]
fn test_null_pointer_free_operations_are_safe() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // All of these should be safe no-ops
        hedl_free_string(ptr::null_mut());
        hedl_free_document(ptr::null_mut());
        hedl_free_diagnostics(ptr::null_mut());
        hedl_free_bytes(ptr::null_mut(), 0);
        hedl_free_bytes(ptr::null_mut(), 1000);
    }
}

#[test]
fn test_parse_output_pointer_reset_on_error() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();

        // Trigger parse error
        let invalid = b"invalid hedl\0";
        let result = hedl_parse(invalid.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        assert_ne!(result, HEDL_OK);
        assert!(doc.is_null());
    }
}

#[cfg(feature = "json")]
#[test]
fn test_conversion_output_pointer_reset_on_error() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        // Try to convert with null output pointer
        let result = hedl_to_json(doc, 0, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[test]
fn test_document_allocation_failure_handling() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();

        // Provide null input pointer - should fail without allocating
        let result = hedl_parse(ptr::null(), -1, 0, &mut doc);

        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert!(doc.is_null());
    }
}

#[test]
fn test_diagnostics_allocation() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        let result = hedl_lint(doc, &mut diag);

        assert_eq!(result, HEDL_OK);
        assert!(!diag.is_null());

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_sequential_operations_memory_safety() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Parse
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        // Canonicalize
        let mut canon: *mut c_char = ptr::null_mut();
        hedl_canonicalize(doc, &mut canon);

        // Lint
        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        // Get version
        let mut major = 0;
        let mut minor = 0;
        hedl_get_version(doc, &mut major, &mut minor);

        // Free in specific order
        hedl_free_string(canon);
        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_memory_usage_with_large_document() {
    // SAFETY: Unsafe operation required for FFI boundary
    unsafe {
        // Create large document
        let mut input = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n");
        for i in 0..10000 {
            input.push_str(&format!("item{i}: {i}\n"));
        }

        let c_input = CString::new(input).unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_parse(c_input.as_ptr(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);

        // Perform operations
        let mut canon: *mut c_char = ptr::null_mut();
        hedl_canonicalize(doc, &mut canon);

        // Clean up
        hedl_free_string(canon);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "parquet")]
#[test]
fn test_parquet_bytes_allocation() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ndata: [{ x: 1 }, { x: 2 }]\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut out_data: *mut u8 = ptr::null_mut();
        let mut out_len: usize = 0;

        let result = hedl_to_parquet(doc, &mut out_data, &mut out_len);

        if result == HEDL_OK {
            assert!(!out_data.is_null());
            assert!(out_len > 0);

            // Verify we can read the data
            let slice = std::slice::from_raw_parts(out_data, out_len);
            assert!(!slice.is_empty());

            hedl_free_bytes(out_data, out_len);
        }

        hedl_free_document(doc);
    }
}

#[test]
fn test_pointer_validity_across_operations() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc1: *mut HedlDocument = ptr::null_mut();
        let mut doc2: *mut HedlDocument = ptr::null_mut();

        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc1);
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc2);

        // Pointers should be different
        assert_ne!(doc1, doc2);

        // Both should be valid
        assert!(!doc1.is_null());
        assert!(!doc2.is_null());

        hedl_free_document(doc1);
        hedl_free_document(doc2);
    }
}

#[test]
fn test_string_allocation_with_empty_output() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Create document that might produce empty output
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut canon: *mut c_char = ptr::null_mut();
        let result = hedl_canonicalize(doc, &mut canon);

        assert_eq!(result, HEDL_OK);
        assert!(!canon.is_null());

        hedl_free_string(canon);
        hedl_free_document(doc);
    }
}
