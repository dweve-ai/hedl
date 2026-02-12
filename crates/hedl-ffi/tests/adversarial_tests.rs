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

//! Adversarial tests for HEDL FFI bindings.
//!
//! Tests robustness against malformed inputs, boundary conditions,
//! invalid UTF-8 sequences, and other edge cases that could cause
//! undefined behavior or crashes in C FFI contexts.

use hedl_ffi::*;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn valid_hedl_cstring() -> CString {
    CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value").unwrap()
}

// =============================================================================
// INVALID UTF-8 TESTS
// =============================================================================

#[test]
fn test_invalid_utf8_single_byte() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Invalid UTF-8: 0xFF is never valid
        let invalid = [0xFFu8, 0x00];
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(invalid.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_ne!(result, HEDL_OK, "Should reject invalid UTF-8");
        assert!(doc.is_null());
    }
}

#[test]
fn test_invalid_utf8_overlong_encoding() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Overlong encoding of '/' (0xC0 0xAF instead of 0x2F)
        let overlong = [0xC0u8, 0xAF, 0x00];
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(overlong.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_ne!(result, HEDL_OK, "Should reject overlong UTF-8 encoding");
        assert!(doc.is_null());
    }
}

#[test]
fn test_invalid_utf8_truncated_sequence() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Start of 3-byte sequence (0xE0) followed by null terminator
        let truncated = [0xE0u8, 0x00];
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(truncated.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_ne!(result, HEDL_OK, "Should reject truncated UTF-8 sequence");
        assert!(doc.is_null());
    }
}

#[test]
fn test_invalid_utf8_surrogate_half() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // UTF-8 encoding of surrogate half (U+D800, invalid in UTF-8)
        let surrogate = [0xEDu8, 0xA0, 0x80, 0x00];
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(surrogate.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_ne!(result, HEDL_OK, "Should reject UTF-8 surrogate halves");
        assert!(doc.is_null());
    }
}

#[test]
fn test_invalid_utf8_continuation_without_start() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Continuation bytes without start byte
        let orphan = [0x80u8, 0x80, 0x80, 0x00];
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(orphan.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_ne!(result, HEDL_OK, "Should reject orphan continuation bytes");
        assert!(doc.is_null());
    }
}

#[test]
fn test_invalid_utf8_out_of_range() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Code point beyond U+10FFFF (4-byte encoding of invalid value)
        let beyond = [0xF4u8, 0x90, 0x80, 0x80, 0x00];
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(beyond.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_ne!(result, HEDL_OK, "Should reject out-of-range code points");
        assert!(doc.is_null());
    }
}

// =============================================================================
// INPUT LENGTH BOUNDARY TESTS
// =============================================================================

#[test]
fn test_empty_input_with_explicit_length() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let empty = CString::new("").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(empty.as_ptr(), 0, 0, &mut doc);
        // Empty input may fail to parse as valid HEDL (depends on parser)
        // The important thing is it doesn't crash
        if result == HEDL_OK && !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

#[test]
fn test_input_length_zero_with_content() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let content = valid_hedl_cstring();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        // Explicit length 0 should parse as empty
        let result = hedl_parse(content.as_ptr(), 0, 0, &mut doc);
        // Should not crash, may or may not succeed
        if result == HEDL_OK && !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

#[test]
fn test_input_length_shorter_than_content() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let content = valid_hedl_cstring();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        // Parse only first 10 bytes - incomplete HEDL
        let result = hedl_parse(content.as_ptr(), 10, 0, &mut doc);
        // Should fail due to incomplete content
        assert_ne!(result, HEDL_OK, "Truncated input should fail to parse");
        assert!(doc.is_null());
    }
}

#[test]
fn test_input_length_negative_one_null_terminated() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let content = valid_hedl_cstring();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(content.as_ptr(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());
        hedl_free_document(doc);
    }
}

#[test]
fn test_input_length_exact() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let content = valid_hedl_cstring();
        let len = content.as_bytes().len() as c_int;
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(content.as_ptr(), len, 0, &mut doc);
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());
        hedl_free_document(doc);
    }
}

// =============================================================================
// MALFORMED HEDL INPUT TESTS
// =============================================================================

#[test]
fn test_malformed_version_missing_colon() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let bad = CString::new("%VERSION 1.0\n---\nkey: value").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let _result = hedl_parse(bad.as_ptr(), -1, 0, &mut doc);
        // May or may not parse depending on strictness
        if !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

#[test]
fn test_malformed_version_invalid_number() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let bad = CString::new("%V:abc.xyz\n---\nkey: value").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(bad.as_ptr(), -1, 0, &mut doc);
        assert_ne!(result, HEDL_OK, "Invalid version should fail");
        assert!(doc.is_null());
    }
}

#[test]
fn test_deeply_nested_structure() {
    // SAFETY: Unsafe operation required for FFI boundary
    unsafe {
        // Create deeply nested structure
        let mut nested = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nroot:\n");
        for i in 0..100 {
            nested.push_str(&format!("{}level{}:\n", " ".repeat(i + 1), i));
        }
        let cstr = CString::new(nested).unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(cstr.as_ptr(), -1, 0, &mut doc);
        // Should handle deep nesting without stack overflow
        if result == HEDL_OK && !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

#[test]
fn test_very_long_key() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // 10KB key name
        let long_key = "k".repeat(10_000);
        let content = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n{long_key}: value");
        let cstr = CString::new(content).unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(cstr.as_ptr(), -1, 0, &mut doc);
        // Should handle long keys
        if result == HEDL_OK && !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

#[test]
fn test_very_long_value() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // 100KB value
        let long_value = "v".repeat(100_000);
        let content = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: {long_value}");
        let cstr = CString::new(content).unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(cstr.as_ptr(), -1, 0, &mut doc);
        // Should handle long values
        if result == HEDL_OK && !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

#[test]
fn test_many_keys() {
    // SAFETY: Unsafe operation required for FFI boundary
    unsafe {
        // 10,000 keys
        let mut content = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n");
        for i in 0..10_000 {
            content.push_str(&format!("key{i}: value{i}\n"));
        }
        let cstr = CString::new(content).unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(cstr.as_ptr(), -1, 0, &mut doc);
        // Should handle many keys
        if result == HEDL_OK && !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

// =============================================================================
// SPECIAL CHARACTERS TESTS
// =============================================================================

#[test]
fn test_embedded_null_character() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Create input with embedded null (requires explicit length)
        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: val\x00ue";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        // Use explicit length to include the null
        let result = hedl_parse(
            input.as_ptr().cast::<c_char>(),
            input.len() as c_int,
            0,
            &mut doc,
        );
        // May succeed or fail, but shouldn't crash
        if result == HEDL_OK && !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

#[test]
fn test_control_characters() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Various control characters
        let input =
            CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: \x01\x02\x03\x04\x05").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);
        // May succeed or fail
        if result == HEDL_OK && !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

#[test]
fn test_unicode_bom() {
    // SAFETY: Unsafe operation required for FFI boundary
    unsafe {
        // UTF-8 BOM followed by HEDL v2.0
        let mut input_vec: Vec<u8> = vec![0xEFu8, 0xBB, 0xBF]; // UTF-8 BOM
        input_vec.extend_from_slice(b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\x00");
        let input = input_vec;
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        // May succeed or fail depending on BOM handling
        if result == HEDL_OK && !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

#[test]
fn test_unicode_codepoints() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Various Unicode characters
        let input =
            CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: 日本語 中文 한국어 العربية")
                .unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK, "Should handle Unicode content");
        assert!(!doc.is_null());
        hedl_free_document(doc);
    }
}

#[test]
fn test_emoji() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: 🚀🎉💯🔥").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK, "Should handle emoji");
        assert!(!doc.is_null());
        hedl_free_document(doc);
    }
}

#[test]
fn test_zero_width_characters() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Zero-width space, zero-width joiner, etc.
        let input = CString::new(
            "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: a\u{200B}b\u{200C}c\u{200D}d\u{FEFF}e",
        )
        .unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);
        // Should handle zero-width characters
        if result == HEDL_OK && !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

// =============================================================================
// DIAGNOSTICS INDEX BOUNDARY TESTS
// =============================================================================

#[test]
fn test_diagnostics_negative_index() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = valid_hedl_cstring();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_diagnostics_get(diag, -1, &mut out_str);
        assert_ne!(result, HEDL_OK, "Negative index should fail");
        assert!(out_str.is_null());

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_diagnostics_max_int_index() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = valid_hedl_cstring();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_diagnostics_get(diag, i32::MAX, &mut out_str);
        assert_ne!(
            result, HEDL_OK,
            "MAX_INT index should fail for typical diag count"
        );
        assert!(out_str.is_null());

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_diagnostics_severity_negative_index() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = valid_hedl_cstring();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        let severity = hedl_diagnostics_severity(diag, -1);
        assert_eq!(severity, -1, "Negative index should return -1");

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

// =============================================================================
// VERSION EXTRACTION EDGE CASES
// =============================================================================

#[test]
fn test_version_extraction_valid() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = valid_hedl_cstring();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        let mut major: c_int = -1;
        let mut minor: c_int = -1;
        let result = hedl_get_version(doc, &mut major, &mut minor);
        assert_eq!(result, HEDL_OK);
        assert_eq!(major, 2);
        assert_eq!(minor, 0);

        hedl_free_document(doc);
    }
}

// =============================================================================
// STRICT MODE TESTS
// =============================================================================

#[test]
fn test_strict_mode_disabled() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);
        hedl_free_document(doc);
    }
}

#[test]
fn test_strict_mode_enabled() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr(), -1, 1, &mut doc);
        assert_eq!(result, HEDL_OK);
        hedl_free_document(doc);
    }
}

#[test]
fn test_strict_mode_nonzero_values() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Test various non-zero strict values
        for strict_val in [1, 2, 100, -1, i32::MAX] {
            let input = CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value").unwrap();
            let mut doc: *mut HedlDocument = ptr::null_mut();
            let result = hedl_parse(input.as_ptr(), -1, strict_val, &mut doc);
            // Should not crash regardless of strict value
            if result == HEDL_OK && !doc.is_null() {
                hedl_free_document(doc);
            }
        }
    }
}

// =============================================================================
// VALIDATE FUNCTION TESTS
// =============================================================================

#[test]
fn test_validate_valid_input() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = valid_hedl_cstring();
        let result = hedl_validate(input.as_ptr(), -1, 0);
        assert_eq!(result, HEDL_OK);
    }
}

#[test]
fn test_validate_invalid_input() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = CString::new("not valid hedl").unwrap();
        let result = hedl_validate(input.as_ptr(), -1, 0);
        assert_ne!(result, HEDL_OK);
    }
}

#[test]
fn test_validate_null_input() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let result = hedl_validate(ptr::null(), -1, 0);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

// =============================================================================
// CANONICALIZE EDGE CASES
// =============================================================================

#[test]
fn test_canonicalize_valid() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = valid_hedl_cstring();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_canonicalize(doc, &mut out_str);
        assert_eq!(result, HEDL_OK);
        assert!(!out_str.is_null());

        hedl_free_string(out_str);
        hedl_free_document(doc);
    }
}

// =============================================================================
// COUNT FUNCTIONS EDGE CASES
// =============================================================================

#[test]
fn test_schema_count_valid() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input =
            CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n%S:Person:[name, age]\n---\nkey: value")
                .unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        let count = hedl_schema_count(doc);
        assert!(count >= 0, "Schema count should be non-negative");

        hedl_free_document(doc);
    }
}

#[test]
fn test_alias_count_valid() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Correct ALIAS syntax: %A:%key:"value"
        let input =
            CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n%A:%short:\"long_name\"\n---\nkey: value")
                .unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        assert_eq!(result, HEDL_OK, "Should parse valid HEDL with alias");
        assert!(!doc.is_null());

        let count = hedl_alias_count(doc);
        assert!(count >= 0, "Alias count should be non-negative");
        assert_eq!(count, 1, "Should have exactly one alias");

        hedl_free_document(doc);
    }
}

#[test]
fn test_root_item_count_valid() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = valid_hedl_cstring();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        let count = hedl_root_item_count(doc);
        assert!(count >= 0, "Root item count should be non-negative");

        hedl_free_document(doc);
    }
}

// =============================================================================
// CONCURRENT PARSING STRESS TEST
// =============================================================================

#[test]
fn test_concurrent_parsing() {
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            // SAFETY: FFI function requires raw pointer for output parameter
            thread::spawn(move || unsafe {
                let content = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nthread{i}: value{i}");
                let cstr = CString::new(content).unwrap();
                let mut doc: *mut HedlDocument = ptr::null_mut();
                let result = hedl_parse(cstr.as_ptr(), -1, 0, &mut doc);
                assert_eq!(result, HEDL_OK);
                assert!(!doc.is_null());

                // Perform operations
                let _schema_count = hedl_schema_count(doc);
                let _alias_count = hedl_alias_count(doc);
                let _root_count = hedl_root_item_count(doc);

                hedl_free_document(doc);
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

// =============================================================================
// ERROR MESSAGE TESTS
// =============================================================================

#[test]
fn test_error_message_after_failure() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        hedl_clear_error_threadsafe();

        let invalid = CString::new("not valid hedl at all").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(invalid.as_ptr(), -1, 0, &mut doc);
        assert_ne!(result, HEDL_OK);

        let error = hedl_get_last_error();
        assert!(!error.is_null(), "Should have error message after failure");
    }
}

#[test]
fn test_error_message_cleared_on_success() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // First cause an error
        let invalid = CString::new("not valid").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let _ = hedl_parse(invalid.as_ptr(), -1, 0, &mut doc);

        // Then succeed
        let valid = valid_hedl_cstring();
        let result = hedl_parse(valid.as_ptr(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);

        // Error should be cleared
        let error = hedl_get_last_error();
        assert!(error.is_null(), "Error should be cleared on success");

        hedl_free_document(doc);
    }
}

#[test]
fn test_clear_error_threadsafe() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Cause an error
        let invalid = CString::new("invalid").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let _ = hedl_parse(invalid.as_ptr(), -1, 0, &mut doc);

        // Error should exist
        let error_before = hedl_get_last_error();
        assert!(!error_before.is_null());

        // Clear error
        hedl_clear_error_threadsafe();

        // Error should be gone
        let error_after = hedl_get_last_error();
        assert!(error_after.is_null());
    }
}

// =============================================================================
// THREAD-LOCAL ERROR ISOLATION
// =============================================================================

#[test]
fn test_thread_local_error_isolation() {
    use std::sync::{Arc, Barrier};
    use std::thread;

    let barrier = Arc::new(Barrier::new(2));

    let barrier1 = barrier.clone();
    // SAFETY: FFI call with valid C-compatible types and checked pointers
    let handle1 = thread::spawn(move || unsafe {
        // Clear any previous error
        hedl_clear_error_threadsafe();

        // Cause an error
        let invalid = CString::new("thread1 invalid").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let _ = hedl_parse(invalid.as_ptr(), -1, 0, &mut doc);

        // Synchronize
        barrier1.wait();

        // Error should still be from this thread
        let error = hedl_get_last_error_threadsafe();
        assert!(!error.is_null(), "Thread 1 should have its own error");
    });

    let barrier2 = barrier.clone();
    // SAFETY: FFI call with valid C-compatible types and checked pointers
    let handle2 = thread::spawn(move || unsafe {
        // Clear any previous error
        hedl_clear_error_threadsafe();

        // Parse successfully
        let valid = CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(valid.as_ptr(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);

        // Synchronize
        barrier2.wait();

        // Error should be null (success clears error)
        let error = hedl_get_last_error_threadsafe();
        assert!(error.is_null(), "Thread 2 should not have an error");

        hedl_free_document(doc);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}

// =============================================================================
// FREE FUNCTION EDGE CASES
// =============================================================================

#[test]
fn test_free_string_null() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Should not crash
        hedl_free_string(ptr::null_mut());
    }
}

#[test]
fn test_free_document_null() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Should not crash
        hedl_free_document(ptr::null_mut());
    }
}

#[test]
fn test_free_diagnostics_null() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Should not crash
        hedl_free_diagnostics(ptr::null_mut());
    }
}

#[test]
fn test_free_bytes_null() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Should not crash
        hedl_free_bytes(ptr::null_mut(), 0);
        hedl_free_bytes(ptr::null_mut(), 100);
    }
}

// =============================================================================
// LINT FUNCTION TESTS
// =============================================================================

#[test]
fn test_lint_valid_document() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let input = valid_hedl_cstring();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        let result = hedl_lint(doc, &mut diag);

        assert_eq!(result, HEDL_OK);
        assert!(!diag.is_null());

        let count = hedl_diagnostics_count(diag);
        assert!(count >= 0);

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_lint_iterate_all_diagnostics() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        // Create document that might produce diagnostics
        let input =
            CString::new("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\nunused_key: unused_value")
                .unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        hedl_lint(doc, &mut diag);

        let count = hedl_diagnostics_count(diag);

        // Iterate all diagnostics
        for i in 0..count {
            let mut msg: *mut c_char = ptr::null_mut();
            let result = hedl_diagnostics_get(diag, i, &mut msg);
            if result == HEDL_OK && !msg.is_null() {
                hedl_free_string(msg);
            }

            let severity = hedl_diagnostics_severity(diag, i);
            assert!((0..=2).contains(&severity));
        }

        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}
