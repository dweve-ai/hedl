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

//! Comprehensive null pointer tests for HEDL FFI bindings.
//!
//! Tests that ALL FFI functions correctly handle NULL pointers without crashing.
//! This is critical for C interoperability where NULL is a common error condition.

use hedl_ffi::*;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn valid_hedl() -> CString {
    CString::new("%VERSION: 1.0\n---\nkey: value").unwrap()
}

fn parse_valid_doc() -> *mut HedlDocument {
    unsafe {
        let input = valid_hedl();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());
        doc
    }
}

fn get_valid_diagnostics(doc: *const HedlDocument) -> *mut HedlDiagnostics {
    unsafe {
        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        let result = hedl_lint(doc, &mut diag);
        assert_eq!(result, HEDL_OK);
        assert!(!diag.is_null());
        diag
    }
}

// =============================================================================
// HEDL_PARSE NULL POINTER TESTS
// =============================================================================

#[test]
fn test_hedl_parse_null_input() {
    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(ptr::null(), -1, 0, &mut doc);
        assert_eq!(
            result, HEDL_ERR_NULL_PTR,
            "NULL input should return NULL_PTR error"
        );
        assert!(doc.is_null(), "Output doc should remain null on error");
    }
}

#[test]
fn test_hedl_parse_null_out_doc() {
    unsafe {
        let input = valid_hedl();
        let result = hedl_parse(input.as_ptr(), -1, 0, ptr::null_mut());
        assert_eq!(
            result, HEDL_ERR_NULL_PTR,
            "NULL out_doc should return NULL_PTR error"
        );
    }
}

#[test]
fn test_hedl_parse_both_null() {
    unsafe {
        let result = hedl_parse(ptr::null(), -1, 0, ptr::null_mut());
        assert_eq!(
            result, HEDL_ERR_NULL_PTR,
            "Both NULL should return NULL_PTR error"
        );
    }
}

// =============================================================================
// HEDL_VALIDATE NULL POINTER TESTS
// =============================================================================

#[test]
fn test_hedl_validate_null_input() {
    unsafe {
        let result = hedl_validate(ptr::null(), -1, 0);
        assert_eq!(
            result, HEDL_ERR_NULL_PTR,
            "NULL input should return NULL_PTR error"
        );
    }
}

// =============================================================================
// HEDL_GET_VERSION NULL POINTER TESTS
// =============================================================================

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
        let doc = parse_valid_doc();
        let mut minor: c_int = 0;
        let result = hedl_get_version(doc, ptr::null_mut(), &mut minor);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_get_version_null_minor() {
    unsafe {
        let doc = parse_valid_doc();
        let mut major: c_int = 0;
        let result = hedl_get_version(doc, &mut major, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_get_version_all_null() {
    unsafe {
        let result = hedl_get_version(ptr::null(), ptr::null_mut(), ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

// =============================================================================
// HEDL_SCHEMA_COUNT NULL POINTER TESTS
// =============================================================================

#[test]
fn test_hedl_schema_count_null_doc() {
    unsafe {
        let count = hedl_schema_count(ptr::null());
        assert_eq!(count, -1, "NULL doc should return -1");
    }
}

// =============================================================================
// HEDL_ALIAS_COUNT NULL POINTER TESTS
// =============================================================================

#[test]
fn test_hedl_alias_count_null_doc() {
    unsafe {
        let count = hedl_alias_count(ptr::null());
        assert_eq!(count, -1, "NULL doc should return -1");
    }
}

// =============================================================================
// HEDL_ROOT_ITEM_COUNT NULL POINTER TESTS
// =============================================================================

#[test]
fn test_hedl_root_item_count_null_doc() {
    unsafe {
        let count = hedl_root_item_count(ptr::null());
        assert_eq!(count, -1, "NULL doc should return -1");
    }
}

// =============================================================================
// HEDL_CANONICALIZE NULL POINTER TESTS
// =============================================================================

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
        let doc = parse_valid_doc();
        let result = hedl_canonicalize(doc, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_canonicalize_both_null() {
    unsafe {
        let result = hedl_canonicalize(ptr::null(), ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

// =============================================================================
// HEDL_LINT NULL POINTER TESTS
// =============================================================================

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
        let doc = parse_valid_doc();
        let result = hedl_lint(doc, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_lint_both_null() {
    unsafe {
        let result = hedl_lint(ptr::null(), ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

// =============================================================================
// HEDL_DIAGNOSTICS_COUNT NULL POINTER TESTS
// =============================================================================

#[test]
fn test_hedl_diagnostics_count_null_diag() {
    unsafe {
        let count = hedl_diagnostics_count(ptr::null());
        assert_eq!(count, -1, "NULL diag should return -1");
    }
}

// =============================================================================
// HEDL_DIAGNOSTICS_GET NULL POINTER TESTS
// =============================================================================

#[test]
fn test_hedl_diagnostics_get_null_diag() {
    unsafe {
        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_diagnostics_get(ptr::null(), 0, &mut out_str);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert!(out_str.is_null());
    }
}

#[test]
fn test_hedl_diagnostics_get_null_out_str() {
    unsafe {
        let doc = parse_valid_doc();
        let diag = get_valid_diagnostics(doc);
        let result = hedl_diagnostics_get(diag, 0, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        hedl_free_diagnostics(diag);
        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_diagnostics_get_both_null() {
    unsafe {
        let result = hedl_diagnostics_get(ptr::null(), 0, ptr::null_mut());
        assert_eq!(result, HEDL_ERR_NULL_PTR);
    }
}

// =============================================================================
// HEDL_DIAGNOSTICS_SEVERITY NULL POINTER TESTS
// =============================================================================

#[test]
fn test_hedl_diagnostics_severity_null_diag() {
    unsafe {
        let severity = hedl_diagnostics_severity(ptr::null(), 0);
        assert_eq!(severity, -1, "NULL diag should return -1");
    }
}

// =============================================================================
// HEDL_FREE_* NULL POINTER TESTS
// =============================================================================

#[test]
fn test_hedl_free_document_null() {
    unsafe {
        // Should not crash
        hedl_free_document(ptr::null_mut());
    }
}

#[test]
fn test_hedl_free_diagnostics_null() {
    unsafe {
        // Should not crash
        hedl_free_diagnostics(ptr::null_mut());
    }
}

#[test]
fn test_hedl_free_string_null() {
    unsafe {
        // Should not crash
        hedl_free_string(ptr::null_mut());
    }
}

#[test]
fn test_hedl_free_bytes_null() {
    unsafe {
        // Should not crash
        hedl_free_bytes(ptr::null_mut(), 0);
        hedl_free_bytes(ptr::null_mut(), 100);
        hedl_free_bytes(ptr::null_mut(), usize::MAX);
    }
}

// =============================================================================
// ERROR FUNCTION NULL BEHAVIOR TESTS
// =============================================================================

#[test]
fn test_hedl_get_last_error_no_error() {
    // These functions are safe (not marked unsafe in the FFI)
    hedl_clear_error_threadsafe();
    let error = hedl_get_last_error();
    assert!(error.is_null(), "No error should mean null pointer");
}

#[test]
fn test_hedl_get_last_error_threadsafe_no_error() {
    // These functions are safe (not marked unsafe in the FFI)
    hedl_clear_error_threadsafe();
    let error = hedl_get_last_error_threadsafe();
    assert!(error.is_null(), "No error should mean null pointer");
}

// =============================================================================
// JSON FEATURE NULL POINTER TESTS
// =============================================================================

#[cfg(feature = "json")]
mod json_null_tests {
    use super::*;

    #[test]
    fn test_hedl_to_json_null_doc() {
        unsafe {
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_json(ptr::null(), 0, &mut out_str);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(out_str.is_null());
        }
    }

    #[test]
    fn test_hedl_to_json_null_out_str() {
        unsafe {
            let doc = parse_valid_doc();
            let result = hedl_to_json(doc, 0, ptr::null_mut());
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            hedl_free_document(doc);
        }
    }

    #[test]
    fn test_hedl_from_json_null_input() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            let result = hedl_from_json(ptr::null(), -1, &mut doc);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(doc.is_null());
        }
    }

    #[test]
    fn test_hedl_from_json_null_out_doc() {
        unsafe {
            let json = CString::new(r#"{"key": "value"}"#).unwrap();
            let result = hedl_from_json(json.as_ptr(), -1, ptr::null_mut());
            assert_eq!(result, HEDL_ERR_NULL_PTR);
        }
    }
}

// =============================================================================
// YAML FEATURE NULL POINTER TESTS
// =============================================================================

#[cfg(feature = "yaml")]
mod yaml_null_tests {
    use super::*;

    #[test]
    fn test_hedl_to_yaml_null_doc() {
        unsafe {
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_yaml(ptr::null(), 0, &mut out_str);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(out_str.is_null());
        }
    }

    #[test]
    fn test_hedl_to_yaml_null_out_str() {
        unsafe {
            let doc = parse_valid_doc();
            let result = hedl_to_yaml(doc, 0, ptr::null_mut());
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            hedl_free_document(doc);
        }
    }

    #[test]
    fn test_hedl_from_yaml_null_input() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            let result = hedl_from_yaml(ptr::null(), -1, &mut doc);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(doc.is_null());
        }
    }

    #[test]
    fn test_hedl_from_yaml_null_out_doc() {
        unsafe {
            let yaml = CString::new("key: value").unwrap();
            let result = hedl_from_yaml(yaml.as_ptr(), -1, ptr::null_mut());
            assert_eq!(result, HEDL_ERR_NULL_PTR);
        }
    }
}

// =============================================================================
// XML FEATURE NULL POINTER TESTS
// =============================================================================

#[cfg(feature = "xml")]
mod xml_null_tests {
    use super::*;

    #[test]
    fn test_hedl_to_xml_null_doc() {
        unsafe {
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_xml(ptr::null(), &mut out_str);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(out_str.is_null());
        }
    }

    #[test]
    fn test_hedl_to_xml_null_out_str() {
        unsafe {
            let doc = parse_valid_doc();
            let result = hedl_to_xml(doc, ptr::null_mut());
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            hedl_free_document(doc);
        }
    }

    #[test]
    fn test_hedl_from_xml_null_input() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            let result = hedl_from_xml(ptr::null(), -1, &mut doc);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(doc.is_null());
        }
    }

    #[test]
    fn test_hedl_from_xml_null_out_doc() {
        unsafe {
            let xml = CString::new("<root><key>value</key></root>").unwrap();
            let result = hedl_from_xml(xml.as_ptr(), -1, ptr::null_mut());
            assert_eq!(result, HEDL_ERR_NULL_PTR);
        }
    }
}

// =============================================================================
// CSV FEATURE NULL POINTER TESTS
// =============================================================================

#[cfg(feature = "csv")]
mod csv_null_tests {
    use super::*;

    #[test]
    fn test_hedl_to_csv_null_doc() {
        unsafe {
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_csv(ptr::null(), &mut out_str);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(out_str.is_null());
        }
    }

    #[test]
    fn test_hedl_to_csv_null_out_str() {
        unsafe {
            let doc = parse_valid_doc();
            let result = hedl_to_csv(doc, ptr::null_mut());
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            hedl_free_document(doc);
        }
    }
}

// =============================================================================
// PARQUET FEATURE NULL POINTER TESTS
// =============================================================================

#[cfg(feature = "parquet")]
mod parquet_null_tests {
    use super::*;

    #[test]
    fn test_hedl_to_parquet_null_doc() {
        unsafe {
            let mut data: *mut u8 = ptr::null_mut();
            let mut len: usize = 0;
            let result = hedl_to_parquet(ptr::null(), &mut data, &mut len);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(data.is_null());
            assert_eq!(len, 0);
        }
    }

    #[test]
    fn test_hedl_to_parquet_null_out_data() {
        unsafe {
            let doc = parse_valid_doc();
            let mut len: usize = 0;
            let result = hedl_to_parquet(doc, ptr::null_mut(), &mut len);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            hedl_free_document(doc);
        }
    }

    #[test]
    fn test_hedl_to_parquet_null_out_len() {
        unsafe {
            let doc = parse_valid_doc();
            let mut data: *mut u8 = ptr::null_mut();
            let result = hedl_to_parquet(doc, &mut data, ptr::null_mut());
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            hedl_free_document(doc);
        }
    }

    #[test]
    fn test_hedl_from_parquet_null_data() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            let result = hedl_from_parquet(ptr::null(), 100, &mut doc);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(doc.is_null());
        }
    }

    #[test]
    fn test_hedl_from_parquet_null_out_doc() {
        unsafe {
            let data = [0u8; 10];
            let result = hedl_from_parquet(data.as_ptr(), 10, ptr::null_mut());
            assert_eq!(result, HEDL_ERR_NULL_PTR);
        }
    }
}

// =============================================================================
// NEO4J FEATURE NULL POINTER TESTS
// =============================================================================

#[cfg(feature = "neo4j")]
mod neo4j_null_tests {
    use super::*;

    #[test]
    fn test_hedl_to_neo4j_cypher_null_doc() {
        unsafe {
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_neo4j_cypher(ptr::null(), 0, &mut out_str);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(out_str.is_null());
        }
    }

    #[test]
    fn test_hedl_to_neo4j_cypher_null_out_str() {
        unsafe {
            let doc = parse_valid_doc();
            let result = hedl_to_neo4j_cypher(doc, 0, ptr::null_mut());
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            hedl_free_document(doc);
        }
    }
}

// =============================================================================
// COMPREHENSIVE NULL COMBINATION TESTS
// =============================================================================

#[test]
fn test_all_count_functions_with_null() {
    unsafe {
        assert_eq!(hedl_schema_count(ptr::null()), -1);
        assert_eq!(hedl_alias_count(ptr::null()), -1);
        assert_eq!(hedl_root_item_count(ptr::null()), -1);
        assert_eq!(hedl_diagnostics_count(ptr::null()), -1);
    }
}

#[test]
fn test_all_free_functions_with_null() {
    unsafe {
        // None of these should crash
        hedl_free_document(ptr::null_mut());
        hedl_free_diagnostics(ptr::null_mut());
        hedl_free_string(ptr::null_mut());
        hedl_free_bytes(ptr::null_mut(), 0);
    }
}

#[test]
fn test_null_pointer_error_messages() {
    unsafe {
        hedl_clear_error_threadsafe();

        // Trigger null pointer error
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let result = hedl_parse(ptr::null(), -1, 0, &mut doc);
        assert_eq!(result, HEDL_ERR_NULL_PTR);

        // Should have appropriate error message
        let error = hedl_get_last_error();
        assert!(
            !error.is_null(),
            "Should have error message for null pointer"
        );
    }
}

// =============================================================================
// REPEATED NULL CALLS (IDEMPOTENCY)
// =============================================================================

#[test]
fn test_repeated_null_free_document() {
    unsafe {
        // Call multiple times - should all succeed without crash
        for _ in 0..100 {
            hedl_free_document(ptr::null_mut());
        }
    }
}

#[test]
fn test_repeated_null_free_diagnostics() {
    unsafe {
        for _ in 0..100 {
            hedl_free_diagnostics(ptr::null_mut());
        }
    }
}

#[test]
fn test_repeated_null_free_string() {
    unsafe {
        for _ in 0..100 {
            hedl_free_string(ptr::null_mut());
        }
    }
}

#[test]
fn test_repeated_null_free_bytes() {
    unsafe {
        for _ in 0..100 {
            hedl_free_bytes(ptr::null_mut(), 0);
        }
    }
}

// =============================================================================
// CONCURRENT NULL HANDLING
// =============================================================================

#[test]
fn test_concurrent_null_handling() {
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| unsafe {
                // All these should handle null safely without race conditions
                let mut doc: *mut HedlDocument = ptr::null_mut();
                let _ = hedl_parse(ptr::null(), -1, 0, &mut doc);
                assert!(doc.is_null());

                hedl_free_document(ptr::null_mut());
                hedl_free_diagnostics(ptr::null_mut());
                hedl_free_string(ptr::null_mut());

                let _ = hedl_schema_count(ptr::null());
                let _ = hedl_alias_count(ptr::null());
                let _ = hedl_root_item_count(ptr::null());
                let _ = hedl_diagnostics_count(ptr::null());
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}
