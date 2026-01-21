// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! FFI boundary tests for input validation and edge cases.

use hedl_ffi::*;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

// Test helper: Create a C string from Rust string
#[allow(dead_code)]
unsafe fn to_c_string(s: &str) -> *mut c_char {
    CString::new(s).unwrap().into_raw()
}

// Test helper: Free C string
#[allow(dead_code)]
unsafe fn free_c_string(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

#[test]
fn test_parse_with_explicit_zero_length() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();

        // Pass 0 as explicit length - should work with empty input
        let result = hedl_parse(input.as_ptr().cast::<c_char>(), 0, 0, &mut doc);

        // Zero-length input should fail gracefully
        assert_ne!(result, HEDL_OK);
        assert!(doc.is_null());
    }
}

#[test]
fn test_parse_with_negative_length_other_than_minus_one() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();

        // Pass -2 as length (only -1 should be accepted for null-terminated)
        let result = hedl_parse(input.as_ptr().cast::<c_char>(), -2, 0, &mut doc);

        assert_eq!(result, HEDL_ERR_INVALID_UTF8);
        assert!(doc.is_null());

        let err = hedl_get_last_error();
        assert!(!err.is_null());
    }
}

#[test]
fn test_parse_with_invalid_negative_length() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();

        // Test various invalid negative lengths
        for invalid_len in &[-2, -3, -100, -9999, i32::MIN] {
            let result = hedl_parse(input.as_ptr().cast::<c_char>(), *invalid_len, 0, &mut doc);
            assert_eq!(result, HEDL_ERR_INVALID_UTF8);
            assert!(doc.is_null());
        }
    }
}

#[test]
fn test_parse_with_exact_length_no_null_terminator() {
    unsafe {
        // Input without null terminator
        let input = b"%VERSION: 1.0\n---\nkey: value";
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_parse(
            input.as_ptr().cast::<c_char>(),
            input.len() as i32,
            0,
            &mut doc,
        );

        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        hedl_free_document(doc);
    }
}

#[test]
fn test_parse_with_length_shorter_than_actual() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();

        // Pass a length that truncates the input
        let result = hedl_parse(input.as_ptr().cast::<c_char>(), 10, 0, &mut doc);

        // Truncated input should fail to parse
        assert_ne!(result, HEDL_OK);
        assert!(doc.is_null());
    }
}

#[test]
fn test_parse_with_embedded_null_bytes() {
    unsafe {
        // Input with embedded null bytes (should fail UTF-8 or parse validation)
        let input = b"%VERSION: 1.0\n---\nkey\0: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_parse(
            input.as_ptr().cast::<c_char>(),
            input.len() as i32,
            0,
            &mut doc,
        );

        // Should fail because of embedded nulls
        assert_ne!(result, HEDL_OK);
    }
}

#[test]
fn test_parse_with_invalid_utf8_at_specific_position() {
    unsafe {
        // Create input with invalid UTF-8 sequence
        let mut input = Vec::from(b"%VERSION: 1.0\n---\nkey: ");
        input.push(0xFF); // Invalid UTF-8
        input.push(0xFE); // Invalid UTF-8

        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(
            input.as_ptr().cast::<c_char>(),
            input.len() as i32,
            0,
            &mut doc,
        );

        assert_eq!(result, HEDL_ERR_INVALID_UTF8);
        assert!(doc.is_null());

        let err = hedl_get_last_error();
        assert!(!err.is_null());
    }
}

#[test]
fn test_validate_with_exact_length() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value";
        let result = hedl_validate(input.as_ptr().cast::<c_char>(), input.len() as i32, 0);

        assert_eq!(result, HEDL_OK);
    }
}

#[test]
fn test_get_version_with_all_null_outputs() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        // Test with null major pointer
        let mut minor = 0;
        let result = hedl_get_version(doc, ptr::null_mut(), &mut minor);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // Test with null minor pointer
        let mut major = 0;
        let result = hedl_get_version(doc, &mut major, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[test]
fn test_schema_count_with_multiple_structs() {
    unsafe {
        // Correct HEDL syntax: %STRUCT: Name: [field1, field2]
        let input = b"%VERSION: 1.0\n%STRUCT: Person: [name, age]\n%STRUCT: Company: [name]\n---\nusers: @Person\n  | alice, 30\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        let count = hedl_schema_count(doc);
        assert_eq!(count, 2);

        hedl_free_document(doc);
    }
}

#[test]
fn test_alias_count_with_multiple_aliases() {
    unsafe {
        // Correct HEDL syntax: %ALIAS: %key: "value"
        let input = b"%VERSION: 1.0\n%ALIAS: %id: \"123\"\n%ALIAS: %name: \"default\"\n%ALIAS: %age: \"30\"\n---\nvalue: %id\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        let count = hedl_alias_count(doc);
        assert_eq!(count, 3);

        hedl_free_document(doc);
    }
}

#[test]
fn test_root_item_count_with_complex_structure() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\n\
            item1: 100\n\
            item2: \"test\"\n\
            item3: [1, 2, 3]\n\
            item4: { nested: true }\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let count = hedl_root_item_count(doc);
        assert_eq!(count, 4);

        hedl_free_document(doc);
    }
}

#[test]
fn test_canonicalize_with_null_out_str() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let result = hedl_canonicalize(doc, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        hedl_free_document(doc);
    }
}

#[test]
fn test_lint_with_warnings_and_errors() {
    unsafe {
        // Create valid HEDL input that may generate lint diagnostics
        let input = b"%VERSION: 1.0\n---\nsome_key: 123\nanother_key: \"value\"\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        let parse_result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        assert_eq!(parse_result, HEDL_OK);
        assert!(!doc.is_null());

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        let lint_result = hedl_lint(doc, &mut diag);

        assert_eq!(lint_result, HEDL_OK);
        assert!(!diag.is_null());

        let count = hedl_diagnostics_count(diag);
        assert!(count >= 0);

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_diagnostics_get_with_negative_index() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_diagnostics_get(diag, -1, &mut out_str);

        assert_eq!(result, HEDL_ERR_LINT);
        assert!(out_str.is_null());

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_diagnostics_severity_with_negative_index() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        let severity = hedl_diagnostics_severity(diag, -1);
        assert_eq!(severity, -1);

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_clear_error_threadsafe() {
    unsafe {
        // Trigger an error
        let result = hedl_validate(ptr::null(), -1, 0);
        assert_ne!(result, HEDL_OK);

        // Verify error is set
        let err1 = hedl_get_last_error();
        assert!(!err1.is_null());

        // Clear error
        hedl_clear_error_threadsafe();

        // Verify error is cleared
        let err2 = hedl_get_last_error();
        assert!(err2.is_null());
    }
}

#[test]
fn test_error_messages_persist_across_operations() {
    unsafe {
        // First operation fails
        let result1 = hedl_validate(ptr::null(), -1, 0);
        assert_ne!(result1, HEDL_OK);
        let err1 = hedl_get_last_error();
        assert!(!err1.is_null());

        // Second operation succeeds - should clear error
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let result2 = hedl_validate(input.as_ptr().cast::<c_char>(), -1, 0);
        assert_eq!(result2, HEDL_OK);

        // Error should be cleared after successful operation
        let err2 = hedl_get_last_error();
        assert!(err2.is_null());
    }
}

#[test]
fn test_get_last_error_threadsafe_equivalence() {
    unsafe {
        // Trigger an error
        hedl_validate(ptr::null(), -1, 0);

        // Both functions should return same error
        let err1 = hedl_get_last_error();
        let err2 = hedl_get_last_error_threadsafe();

        assert!(!err1.is_null());
        assert!(!err2.is_null());
        assert_eq!(err1, err2);
    }
}

#[test]
fn test_free_string_with_null() {
    unsafe {
        // Should not crash
        hedl_free_string(ptr::null_mut());
    }
}

#[test]
fn test_free_document_with_null() {
    unsafe {
        // Should not crash
        hedl_free_document(ptr::null_mut());
    }
}

#[test]
fn test_free_diagnostics_with_null() {
    unsafe {
        // Should not crash
        hedl_free_diagnostics(ptr::null_mut());
    }
}

#[test]
fn test_free_bytes_with_null() {
    unsafe {
        // Should not crash
        hedl_free_bytes(ptr::null_mut(), 0);
        hedl_free_bytes(ptr::null_mut(), 100);
    }
}

#[test]
fn test_parse_with_very_long_valid_input() {
    unsafe {
        // Create a large valid HEDL document
        let mut input = String::from("%VERSION: 1.0\n---\n");
        for i in 0..1000 {
            input.push_str(&format!("key{i}: \"value{i}\"\n"));
        }

        let c_input = CString::new(input.clone()).unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_parse(c_input.as_ptr(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        // Verify root item count
        let count = hedl_root_item_count(doc);
        assert_eq!(count, 1000);

        hedl_free_document(doc);
    }
}

#[test]
fn test_multiple_parse_and_free_cycles() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\nkey: value\0";

        for _ in 0..100 {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            let result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
            assert_eq!(result, HEDL_OK);
            assert!(!doc.is_null());
            hedl_free_document(doc);
        }
    }
}

#[test]
fn test_error_after_successful_operation() {
    unsafe {
        // Start with successful operation
        let input = b"%VERSION: 1.0\n---\nkey: value\0";
        let result1 = hedl_validate(input.as_ptr().cast::<c_char>(), -1, 0);
        assert_eq!(result1, HEDL_OK);

        // Error should be None
        let err1 = hedl_get_last_error();
        assert!(err1.is_null());

        // Now trigger error
        let result2 = hedl_validate(ptr::null(), -1, 0);
        assert_ne!(result2, HEDL_OK);

        // Error should be set
        let err2 = hedl_get_last_error();
        assert!(!err2.is_null());
    }
}

#[test]
fn test_parse_with_whitespace_only() {
    unsafe {
        let input = b"   \n\n\t\t  \0";
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_ne!(result, HEDL_OK);
        assert!(doc.is_null());
    }
}

#[test]
fn test_parse_with_only_version_header() {
    unsafe {
        let input = b"%VERSION: 1.0\n---\n\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        let count = hedl_root_item_count(doc);
        assert_eq!(count, 0);

        hedl_free_document(doc);
    }
}
