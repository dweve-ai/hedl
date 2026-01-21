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

//! Poison pointer tests for HEDL FFI bindings.
//!
//! Tests the detection of double-free and use-after-free bugs using poison
//! pointer values. These tests verify that the FFI layer properly rejects
//! operations on poisoned pointers.
//!
//! Poison pointer values used:
//! - `POISON_PTR_DOCUMENT`: 0xDEADBEEF
//! - `POISON_PTR_DIAGNOSTICS`: 0xDEADC0DE

use hedl_ffi::*;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::ptr;

// =============================================================================
// POISON POINTER CONSTANTS
// =============================================================================

/// Poison pointer value for documents (matches memory.rs)
const POISON_PTR_DOCUMENT: usize = 0xDEADBEEF;

/// Poison pointer value for diagnostics (matches memory.rs)
const POISON_PTR_DIAGNOSTICS: usize = 0xDEADC0DE;

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

/// Create a poison document pointer
fn poison_document_ptr() -> *mut HedlDocument {
    POISON_PTR_DOCUMENT as *mut HedlDocument
}

/// Create a poison diagnostics pointer
fn poison_diagnostics_ptr() -> *mut HedlDiagnostics {
    POISON_PTR_DIAGNOSTICS as *mut HedlDiagnostics
}

// =============================================================================
// DOCUMENT POISON POINTER TESTS
// =============================================================================

#[test]
fn test_poison_document_get_version() {
    unsafe {
        let poison_doc = poison_document_ptr();
        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        let result = hedl_get_version(poison_doc, &mut major, &mut minor);
        assert_eq!(
            result, HEDL_ERR_NULL_PTR,
            "Poison document should be treated like null"
        );
    }
}

#[test]
fn test_poison_document_schema_count() {
    unsafe {
        let poison_doc = poison_document_ptr();
        let count = hedl_schema_count(poison_doc);
        assert_eq!(count, -1, "Poison document should return -1");
    }
}

#[test]
fn test_poison_document_alias_count() {
    unsafe {
        let poison_doc = poison_document_ptr();
        let count = hedl_alias_count(poison_doc);
        assert_eq!(count, -1, "Poison document should return -1");
    }
}

#[test]
fn test_poison_document_root_item_count() {
    unsafe {
        let poison_doc = poison_document_ptr();
        let count = hedl_root_item_count(poison_doc);
        assert_eq!(count, -1, "Poison document should return -1");
    }
}

#[test]
fn test_poison_document_canonicalize() {
    unsafe {
        let poison_doc = poison_document_ptr();
        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_canonicalize(poison_doc, &mut out_str);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert!(out_str.is_null());
    }
}

#[test]
fn test_poison_document_lint() {
    unsafe {
        let poison_doc = poison_document_ptr();
        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        let result = hedl_lint(poison_doc, &mut diag);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert!(diag.is_null());
    }
}

#[test]
fn test_poison_document_free() {
    unsafe {
        let poison_doc = poison_document_ptr();
        // Should not crash - poison pointers are safely ignored
        hedl_free_document(poison_doc);
    }
}

// =============================================================================
// DIAGNOSTICS POISON POINTER TESTS
// =============================================================================

#[test]
fn test_poison_diagnostics_count() {
    unsafe {
        let poison_diag = poison_diagnostics_ptr();
        let count = hedl_diagnostics_count(poison_diag);
        assert_eq!(count, -1, "Poison diagnostics should return -1");
    }
}

#[test]
fn test_poison_diagnostics_get() {
    unsafe {
        let poison_diag = poison_diagnostics_ptr();
        let mut out_str: *mut c_char = ptr::null_mut();
        let result = hedl_diagnostics_get(poison_diag, 0, &mut out_str);
        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert!(out_str.is_null());
    }
}

#[test]
fn test_poison_diagnostics_severity() {
    unsafe {
        let poison_diag = poison_diagnostics_ptr();
        let severity = hedl_diagnostics_severity(poison_diag, 0);
        assert_eq!(severity, -1, "Poison diagnostics should return -1");
    }
}

#[test]
fn test_poison_diagnostics_free() {
    unsafe {
        let poison_diag = poison_diagnostics_ptr();
        // Should not crash - poison pointers are safely ignored
        hedl_free_diagnostics(poison_diag);
    }
}

// =============================================================================
// JSON FEATURE POISON POINTER TESTS
// =============================================================================

#[cfg(feature = "json")]
mod json_poison_tests {
    use super::*;

    #[test]
    fn test_poison_document_to_json() {
        unsafe {
            let poison_doc = poison_document_ptr();
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_json(poison_doc, 0, &mut out_str);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(out_str.is_null());
        }
    }
}

// =============================================================================
// YAML FEATURE POISON POINTER TESTS
// =============================================================================

#[cfg(feature = "yaml")]
mod yaml_poison_tests {
    use super::*;

    #[test]
    fn test_poison_document_to_yaml() {
        unsafe {
            let poison_doc = poison_document_ptr();
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_yaml(poison_doc, 0, &mut out_str);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(out_str.is_null());
        }
    }
}

// =============================================================================
// XML FEATURE POISON POINTER TESTS
// =============================================================================

#[cfg(feature = "xml")]
mod xml_poison_tests {
    use super::*;

    #[test]
    fn test_poison_document_to_xml() {
        unsafe {
            let poison_doc = poison_document_ptr();
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_xml(poison_doc, &mut out_str);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(out_str.is_null());
        }
    }
}

// =============================================================================
// CSV FEATURE POISON POINTER TESTS
// =============================================================================

#[cfg(feature = "csv")]
mod csv_poison_tests {
    use super::*;

    #[test]
    fn test_poison_document_to_csv() {
        unsafe {
            let poison_doc = poison_document_ptr();
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_csv(poison_doc, &mut out_str);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(out_str.is_null());
        }
    }
}

// =============================================================================
// PARQUET FEATURE POISON POINTER TESTS
// =============================================================================

#[cfg(feature = "parquet")]
mod parquet_poison_tests {
    use super::*;

    #[test]
    fn test_poison_document_to_parquet() {
        unsafe {
            let poison_doc = poison_document_ptr();
            let mut data: *mut u8 = ptr::null_mut();
            let mut len: usize = 0;
            let result = hedl_to_parquet(poison_doc, &mut data, &mut len);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(data.is_null());
            assert_eq!(len, 0);
        }
    }
}

// =============================================================================
// NEO4J FEATURE POISON POINTER TESTS
// =============================================================================

#[cfg(feature = "neo4j")]
mod neo4j_poison_tests {
    use super::*;

    #[test]
    fn test_poison_document_to_neo4j_cypher() {
        unsafe {
            let poison_doc = poison_document_ptr();
            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_neo4j_cypher(poison_doc, 0, &mut out_str);
            assert_eq!(result, HEDL_ERR_NULL_PTR);
            assert!(out_str.is_null());
        }
    }
}

// =============================================================================
// SIMULATED DOUBLE-FREE SCENARIOS
// =============================================================================

#[test]
fn test_simulated_double_free_document() {
    unsafe {
        // Simulate what happens in C code that doesn't null out pointer after free
        let doc = parse_valid_doc();
        hedl_free_document(doc);

        // At this point, 'doc' still has the old value but memory is freed.
        // In a real double-free scenario, the caller might pass the same pointer again.
        // We can't detect this in Rust, but we can verify that passing a random
        // non-null, non-poison address doesn't crash (it's UB, but we test the boundaries).

        // Instead, test that repeatedly passing poison value is safe
        let poison = poison_document_ptr();
        hedl_free_document(poison);
        hedl_free_document(poison);
        hedl_free_document(poison);
    }
}

#[test]
fn test_simulated_double_free_diagnostics() {
    unsafe {
        let doc = parse_valid_doc();
        let diag = get_valid_diagnostics(doc);
        hedl_free_diagnostics(diag);

        // Test that repeatedly passing poison value is safe
        let poison = poison_diagnostics_ptr();
        hedl_free_diagnostics(poison);
        hedl_free_diagnostics(poison);
        hedl_free_diagnostics(poison);

        hedl_free_document(doc);
    }
}

// =============================================================================
// SIMULATED USE-AFTER-FREE SCENARIOS
// =============================================================================

#[test]
fn test_use_after_free_with_poison_document() {
    unsafe {
        // Simulate what a C programmer might do: use pointer after free
        // We test that poison pointers are rejected

        let poison = poison_document_ptr();

        // All accessor functions should reject poison pointers
        assert_eq!(hedl_schema_count(poison), -1);
        assert_eq!(hedl_alias_count(poison), -1);
        assert_eq!(hedl_root_item_count(poison), -1);

        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        assert_eq!(
            hedl_get_version(poison, &mut major, &mut minor),
            HEDL_ERR_NULL_PTR
        );

        let mut out_str: *mut c_char = ptr::null_mut();
        assert_eq!(hedl_canonicalize(poison, &mut out_str), HEDL_ERR_NULL_PTR);

        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        assert_eq!(hedl_lint(poison, &mut diag), HEDL_ERR_NULL_PTR);
    }
}

#[test]
fn test_use_after_free_with_poison_diagnostics() {
    unsafe {
        let poison = poison_diagnostics_ptr();

        assert_eq!(hedl_diagnostics_count(poison), -1);
        assert_eq!(hedl_diagnostics_severity(poison, 0), -1);

        let mut out_str: *mut c_char = ptr::null_mut();
        assert_eq!(
            hedl_diagnostics_get(poison, 0, &mut out_str),
            HEDL_ERR_NULL_PTR
        );
    }
}

// =============================================================================
// REPEATED POISON OPERATIONS (IDEMPOTENCY)
// =============================================================================

#[test]
fn test_repeated_poison_document_operations() {
    unsafe {
        let poison = poison_document_ptr();

        for _ in 0..100 {
            assert_eq!(hedl_schema_count(poison), -1);
            assert_eq!(hedl_alias_count(poison), -1);
            assert_eq!(hedl_root_item_count(poison), -1);
            hedl_free_document(poison);
        }
    }
}

#[test]
fn test_repeated_poison_diagnostics_operations() {
    unsafe {
        let poison = poison_diagnostics_ptr();

        for _ in 0..100 {
            assert_eq!(hedl_diagnostics_count(poison), -1);
            assert_eq!(hedl_diagnostics_severity(poison, 0), -1);
            hedl_free_diagnostics(poison);
        }
    }
}

// =============================================================================
// CONCURRENT POISON POINTER HANDLING
// =============================================================================

#[test]
fn test_concurrent_poison_document_handling() {
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| unsafe {
                let poison = poison_document_ptr();

                for _ in 0..100 {
                    assert_eq!(hedl_schema_count(poison), -1);
                    assert_eq!(hedl_alias_count(poison), -1);
                    assert_eq!(hedl_root_item_count(poison), -1);
                    hedl_free_document(poison);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

#[test]
fn test_concurrent_poison_diagnostics_handling() {
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| unsafe {
                let poison = poison_diagnostics_ptr();

                for _ in 0..100 {
                    assert_eq!(hedl_diagnostics_count(poison), -1);
                    assert_eq!(hedl_diagnostics_severity(poison, 0), -1);
                    hedl_free_diagnostics(poison);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("Thread panicked");
    }
}

// =============================================================================
// MIXED NULL AND POISON TESTS
// =============================================================================

#[test]
fn test_null_and_poison_document_same_behavior() {
    unsafe {
        let null_doc: *const HedlDocument = ptr::null();
        let poison_doc = poison_document_ptr();

        // Both should return same error codes
        assert_eq!(hedl_schema_count(null_doc), hedl_schema_count(poison_doc));
        assert_eq!(hedl_alias_count(null_doc), hedl_alias_count(poison_doc));
        assert_eq!(
            hedl_root_item_count(null_doc),
            hedl_root_item_count(poison_doc)
        );

        let mut major1: c_int = 0;
        let mut minor1: c_int = 0;
        let mut major2: c_int = 0;
        let mut minor2: c_int = 0;
        assert_eq!(
            hedl_get_version(null_doc, &mut major1, &mut minor1),
            hedl_get_version(poison_doc, &mut major2, &mut minor2)
        );
    }
}

#[test]
fn test_null_and_poison_diagnostics_same_behavior() {
    unsafe {
        let null_diag: *const HedlDiagnostics = ptr::null();
        let poison_diag = poison_diagnostics_ptr();

        assert_eq!(
            hedl_diagnostics_count(null_diag),
            hedl_diagnostics_count(poison_diag)
        );
        assert_eq!(
            hedl_diagnostics_severity(null_diag, 0),
            hedl_diagnostics_severity(poison_diag, 0)
        );
    }
}

// =============================================================================
// COMPREHENSIVE POISON POINTER SAFETY
// =============================================================================

#[test]
fn test_all_document_functions_with_poison() {
    unsafe {
        let poison = poison_document_ptr();

        // Count functions return -1
        assert_eq!(hedl_schema_count(poison), -1);
        assert_eq!(hedl_alias_count(poison), -1);
        assert_eq!(hedl_root_item_count(poison), -1);

        // Version returns error
        let mut major: c_int = 0;
        let mut minor: c_int = 0;
        assert_eq!(
            hedl_get_version(poison, &mut major, &mut minor),
            HEDL_ERR_NULL_PTR
        );

        // Canonicalize returns error
        let mut out_str: *mut c_char = ptr::null_mut();
        assert_eq!(hedl_canonicalize(poison, &mut out_str), HEDL_ERR_NULL_PTR);
        assert!(out_str.is_null());

        // Lint returns error
        let mut diag: *mut HedlDiagnostics = ptr::null_mut();
        assert_eq!(hedl_lint(poison, &mut diag), HEDL_ERR_NULL_PTR);
        assert!(diag.is_null());

        // Free is safe
        hedl_free_document(poison);
    }
}

#[test]
fn test_all_diagnostics_functions_with_poison() {
    unsafe {
        let poison = poison_diagnostics_ptr();

        // Count returns -1
        assert_eq!(hedl_diagnostics_count(poison), -1);

        // Severity returns -1
        assert_eq!(hedl_diagnostics_severity(poison, 0), -1);
        assert_eq!(hedl_diagnostics_severity(poison, 1), -1);
        assert_eq!(hedl_diagnostics_severity(poison, -1), -1);

        // Get returns error
        let mut out_str: *mut c_char = ptr::null_mut();
        assert_eq!(
            hedl_diagnostics_get(poison, 0, &mut out_str),
            HEDL_ERR_NULL_PTR
        );
        assert!(out_str.is_null());

        // Free is safe
        hedl_free_diagnostics(poison);
    }
}

// =============================================================================
// POISON VALUE DISTINCTNESS TEST
// =============================================================================

#[test]
fn test_poison_values_are_distinct() {
    // Verify the poison values are different for documents and diagnostics
    assert_ne!(
        POISON_PTR_DOCUMENT, POISON_PTR_DIAGNOSTICS,
        "Poison values should be distinct"
    );

    // Verify they are recognizable debug values
    assert_eq!(POISON_PTR_DOCUMENT, 0xDEADBEEF);
    assert_eq!(POISON_PTR_DIAGNOSTICS, 0xDEADC0DE);
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_poison_values_are_invalid_addresses() {
    // Poison values should be obviously invalid addresses
    // (odd alignment, recognizable pattern)
    assert!(
        POISON_PTR_DOCUMENT % 2 == 1,
        "POISON_PTR_DOCUMENT should be oddly aligned"
    );
    assert!(
        POISON_PTR_DIAGNOSTICS.is_multiple_of(2),
        "POISON_PTR_DIAGNOSTICS is evenly aligned but still invalid"
    );
}

// =============================================================================
// LIFECYCLE TESTS
// =============================================================================

#[test]
fn test_valid_document_lifecycle() {
    unsafe {
        // Parse
        let doc = parse_valid_doc();
        assert!(!doc.is_null());

        // Use
        let schema_count = hedl_schema_count(doc);
        assert!(schema_count >= 0);

        let alias_count = hedl_alias_count(doc);
        assert!(alias_count >= 0);

        let root_count = hedl_root_item_count(doc);
        assert!(root_count >= 0);

        // Free
        hedl_free_document(doc);

        // After free, we can't use doc anymore
        // But passing poison value is safe
        let poison = poison_document_ptr();
        assert_eq!(hedl_schema_count(poison), -1);
    }
}

#[test]
fn test_valid_diagnostics_lifecycle() {
    unsafe {
        let doc = parse_valid_doc();

        // Lint
        let diag = get_valid_diagnostics(doc);
        assert!(!diag.is_null());

        // Use
        let count = hedl_diagnostics_count(diag);
        assert!(count >= 0);

        // Free
        hedl_free_diagnostics(diag);

        // After free, poison value is safe
        let poison = poison_diagnostics_ptr();
        assert_eq!(hedl_diagnostics_count(poison), -1);

        hedl_free_document(doc);
    }
}

// =============================================================================
// STRESS TEST WITH POISON POINTERS
// =============================================================================

#[test]
fn test_stress_poison_pointers() {
    use std::thread;

    let handles: Vec<_> = (0..4)
        .map(|thread_id| {
            thread::spawn(move || unsafe {
                let poison_doc = poison_document_ptr();
                let poison_diag = poison_diagnostics_ptr();

                for iteration in 0..1000 {
                    // Document operations
                    assert_eq!(hedl_schema_count(poison_doc), -1);
                    assert_eq!(hedl_alias_count(poison_doc), -1);
                    assert_eq!(hedl_root_item_count(poison_doc), -1);
                    hedl_free_document(poison_doc);

                    // Diagnostics operations
                    assert_eq!(hedl_diagnostics_count(poison_diag), -1);
                    assert_eq!(hedl_diagnostics_severity(poison_diag, 0), -1);
                    hedl_free_diagnostics(poison_diag);

                    // Mix in some valid operations
                    if iteration % 100 == 0 {
                        let doc = parse_valid_doc();
                        let _ = hedl_schema_count(doc);
                        hedl_free_document(doc);
                    }
                }

                thread_id
            })
        })
        .collect();

    for handle in handles {
        let thread_id = handle.join().expect("Thread panicked");
        println!("Thread {thread_id} completed");
    }
}
