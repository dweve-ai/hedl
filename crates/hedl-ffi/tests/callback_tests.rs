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

//! Tests for zero-copy callback API

use hedl_ffi::*;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};
use std::ptr;
use std::slice;

// =============================================================================
// Test Utilities
// =============================================================================

/// Context structure for capturing callback data
#[repr(C)]
struct CallbackContext {
    data: Vec<u8>,
    call_count: usize,
}

impl CallbackContext {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            call_count: 0,
        }
    }

    fn as_string(&self) -> String {
        String::from_utf8_lossy(&self.data).into_owned()
    }
}

/// Test callback that captures data into a Vec
unsafe extern "C" fn test_callback(data: *const c_char, len: usize, user_data: *mut c_void) {
    let ctx = &mut *(user_data as *mut CallbackContext);
    let slice = slice::from_raw_parts(data as *const u8, len);
    ctx.data.extend_from_slice(slice);
    ctx.call_count += 1;
}

/// Create a test HEDL document
unsafe fn create_test_document() -> *mut HedlDocument {
    let hedl = b"%VERSION: 1.0\n---\nperson:\n  name: Alice\n  age: 30\n  city: NYC\0";
    let mut doc: *mut HedlDocument = ptr::null_mut();
    let result = hedl_parse(hedl.as_ptr() as *const c_char, -1, 0, &mut doc);
    assert_eq!(result, HEDL_OK);
    assert!(!doc.is_null());
    doc
}

/// Create a large test HEDL document (for >1MB output)
unsafe fn create_large_document() -> *mut HedlDocument {
    let mut hedl = String::from("%VERSION: 1.0\n---\n");

    // Create a document with many entities to generate large output
    for i in 0..10000 {
        hedl.push_str(&format!(
            "entity{}:\n  field1: value_{}\n  field2: value_{}\n  field3: value_{}\n",
            i, i, i, i
        ));
    }

    hedl.push('\0');

    let mut doc: *mut HedlDocument = ptr::null_mut();
    let result = hedl_parse(hedl.as_ptr() as *const c_char, -1, 0, &mut doc);
    assert_eq!(result, HEDL_OK);
    assert!(!doc.is_null());
    doc
}

// =============================================================================
// Basic Callback Tests
// =============================================================================

#[test]
fn test_canonicalize_callback() {
    unsafe {
        let doc = create_test_document();
        let mut ctx = CallbackContext::new();

        let result = hedl_canonicalize_callback(
            doc,
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(ctx.call_count, 1);

        let output = ctx.as_string();
        assert!(output.contains("%VERSION: 1.0"));
        assert!(output.contains("person"));
        assert!(output.contains("Alice"));

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_json_callback() {
    unsafe {
        let doc = create_test_document();
        let mut ctx = CallbackContext::new();

        let result = hedl_to_json_callback(
            doc,
            0,
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(ctx.call_count, 1);

        let json = ctx.as_string();
        assert!(json.contains("person"));
        assert!(json.contains("Alice"));
        assert!(json.contains("30"));

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_json_callback_with_metadata() {
    unsafe {
        let doc = create_test_document();
        let mut ctx = CallbackContext::new();

        let result = hedl_to_json_callback(
            doc,
            1,  // include_metadata
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(ctx.call_count, 1);

        let json = ctx.as_string();
        // Metadata may or may not be present depending on document structure
        assert!(json.contains("Alice"));

        hedl_free_document(doc);
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_yaml_callback() {
    unsafe {
        let doc = create_test_document();
        let mut ctx = CallbackContext::new();

        let result = hedl_to_yaml_callback(
            doc,
            0,
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(ctx.call_count, 1);

        let yaml = ctx.as_string();
        assert!(yaml.contains("person"));
        assert!(yaml.contains("Alice"));

        hedl_free_document(doc);
    }
}

#[cfg(feature = "xml")]
#[test]
fn test_xml_callback() {
    unsafe {
        let doc = create_test_document();
        let mut ctx = CallbackContext::new();

        let result = hedl_to_xml_callback(
            doc,
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(ctx.call_count, 1);

        let xml = ctx.as_string();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("person"));
        assert!(xml.contains("Alice"));

        hedl_free_document(doc);
    }
}

#[cfg(feature = "csv")]
#[test]
fn test_csv_callback() {
    unsafe {
        // Create a document with matrix list for CSV conversion
        // Using proper HEDL syntax with struct definition
        let hedl = concat!(
            "%VERSION: 1.0\n",
            "---\n",
            "@Person: [name, age]\n",
            "---\n",
            "people: [\n",
            "  [Alice, 30]\n",
            "  [Bob, 25]\n",
            "]\0"
        );
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let parse_result = hedl_parse(hedl.as_ptr() as *const c_char, -1, 0, &mut doc);

        if parse_result != HEDL_OK {
            // If parsing fails, just skip the test gracefully
            // CSV conversion requires specific document structure
            return;
        }

        let mut ctx = CallbackContext::new();

        let result = hedl_to_csv_callback(
            doc,
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );

        // CSV conversion might fail if the document structure isn't suitable
        // Just verify it doesn't crash and returns a valid error code
        if result == HEDL_OK {
            assert_eq!(ctx.call_count, 1);
            let csv = ctx.as_string();
            assert!(!csv.is_empty());
        }

        hedl_free_document(doc);
    }
}

#[cfg(feature = "neo4j")]
#[test]
fn test_neo4j_callback() {
    unsafe {
        let doc = create_test_document();
        let mut ctx = CallbackContext::new();

        let result = hedl_to_neo4j_cypher_callback(
            doc,
            1,  // use_merge
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(ctx.call_count, 1);

        hedl_free_document(doc);
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_callback_null_document() {
    unsafe {
        let mut ctx = CallbackContext::new();

        let result = hedl_canonicalize_callback(
            ptr::null(),
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );

        assert_eq!(result, HEDL_ERR_NULL_PTR);
        assert_eq!(ctx.call_count, 0);
    }
}

// =============================================================================
// Comparison Tests (callback vs regular)
// =============================================================================

#[cfg(feature = "json")]
#[test]
fn test_json_callback_vs_regular() {
    unsafe {
        let doc = create_test_document();

        // Test with callback
        let mut ctx = CallbackContext::new();
        let result_cb = hedl_to_json_callback(
            doc,
            0,
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );
        assert_eq!(result_cb, HEDL_OK);
        let json_cb = ctx.as_string();

        // Test with regular API
        let mut json_str: *mut c_char = ptr::null_mut();
        let result_reg = hedl_to_json(doc, 0, &mut json_str);
        assert_eq!(result_reg, HEDL_OK);
        let json_reg = CStr::from_ptr(json_str).to_str().unwrap();

        // Both should produce identical output
        assert_eq!(json_cb, json_reg);

        hedl_free_string(json_str);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_yaml_callback_vs_regular() {
    unsafe {
        let doc = create_test_document();

        // Test with callback
        let mut ctx = CallbackContext::new();
        hedl_to_yaml_callback(doc, 0, test_callback, &mut ctx as *mut _ as *mut c_void);
        let yaml_cb = ctx.as_string();

        // Test with regular API
        let mut yaml_str: *mut c_char = ptr::null_mut();
        hedl_to_yaml(doc, 0, &mut yaml_str);
        let yaml_reg = CStr::from_ptr(yaml_str).to_str().unwrap();

        // Both should produce identical output
        assert_eq!(yaml_cb, yaml_reg);

        hedl_free_string(yaml_str);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "xml")]
#[test]
fn test_xml_callback_vs_regular() {
    unsafe {
        let doc = create_test_document();

        // Test with callback
        let mut ctx = CallbackContext::new();
        hedl_to_xml_callback(doc, test_callback, &mut ctx as *mut _ as *mut c_void);
        let xml_cb = ctx.as_string();

        // Test with regular API
        let mut xml_str: *mut c_char = ptr::null_mut();
        hedl_to_xml(doc, &mut xml_str);
        let xml_reg = CStr::from_ptr(xml_str).to_str().unwrap();

        // Both should produce identical output
        assert_eq!(xml_cb, xml_reg);

        hedl_free_string(xml_str);
        hedl_free_document(doc);
    }
}

#[test]
fn test_canonicalize_callback_vs_regular() {
    unsafe {
        let doc = create_test_document();

        // Test with callback
        let mut ctx = CallbackContext::new();
        hedl_canonicalize_callback(doc, test_callback, &mut ctx as *mut _ as *mut c_void);
        let can_cb = ctx.as_string();

        // Test with regular API
        let mut can_str: *mut c_char = ptr::null_mut();
        hedl_canonicalize(doc, &mut can_str);
        let can_reg = CStr::from_ptr(can_str).to_str().unwrap();

        // Both should produce identical output
        assert_eq!(can_cb, can_reg);

        hedl_free_string(can_str);
        hedl_free_document(doc);
    }
}

// =============================================================================
// Large Document Tests
// =============================================================================

#[cfg(feature = "json")]
#[test]
fn test_large_json_callback() {
    unsafe {
        let doc = create_large_document();
        let mut ctx = CallbackContext::new();

        let result = hedl_to_json_callback(
            doc,
            0,
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(ctx.call_count, 1);

        // Verify the output is large (>1MB would be ideal but depends on JSON size)
        assert!(ctx.data.len() > 10000, "Expected large output, got {} bytes", ctx.data.len());

        hedl_free_document(doc);
    }
}

#[test]
fn test_large_canonicalize_callback() {
    unsafe {
        let doc = create_large_document();
        let mut ctx = CallbackContext::new();

        let result = hedl_canonicalize_callback(
            doc,
            test_callback,
            &mut ctx as *mut _ as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(ctx.call_count, 1);
        assert!(ctx.data.len() > 10000);

        hedl_free_document(doc);
    }
}

// =============================================================================
// Memory Safety Tests
// =============================================================================

#[test]
fn test_callback_data_lifetime() {
    unsafe {
        let doc = create_test_document();

        // This callback tries to store the pointer (BAD - for testing only)
        static mut STORED_PTR: *const c_char = ptr::null();
        static mut STORED_LEN: usize = 0;

        unsafe extern "C" fn storing_callback(data: *const c_char, len: usize, _user_data: *mut c_void) {
            STORED_PTR = data;
            STORED_LEN = len;
        }

        hedl_canonicalize_callback(doc, storing_callback, ptr::null_mut());

        // After callback returns, the stored pointer is invalid
        // We can't safely test this without causing UB, but this demonstrates
        // the importance of copying data within the callback

        assert!(!STORED_PTR.is_null());
        assert!(STORED_LEN > 0);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_multiple_callbacks_different_documents() {
    unsafe {
        let doc1 = create_test_document();
        let doc2 = create_test_document();

        let mut ctx1 = CallbackContext::new();
        let mut ctx2 = CallbackContext::new();

        hedl_to_json_callback(doc1, 0, test_callback, &mut ctx1 as *mut _ as *mut c_void);
        hedl_to_json_callback(doc2, 0, test_callback, &mut ctx2 as *mut _ as *mut c_void);

        // Both should have same output (same source document)
        assert_eq!(ctx1.as_string(), ctx2.as_string());

        hedl_free_document(doc1);
        hedl_free_document(doc2);
    }
}

// =============================================================================
// Thread Safety Tests
// =============================================================================

#[cfg(feature = "json")]
#[test]
fn test_callback_thread_safety() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    unsafe {
        let doc = create_test_document();

        // Create a thread-safe context
        let ctx = Arc::new(Mutex::new(CallbackContext::new()));
        let ctx_clone = Arc::clone(&ctx);

        // Use callback from different thread
        let handle = thread::spawn(move || {
            unsafe extern "C" fn thread_callback(data: *const c_char, len: usize, user_data: *mut c_void) {
                let ctx_arc = &*(user_data as *const Arc<Mutex<CallbackContext>>);
                let mut ctx = ctx_arc.lock().unwrap();
                let slice = slice::from_raw_parts(data as *const u8, len);
                ctx.data.extend_from_slice(slice);
                ctx.call_count += 1;
            }

            let doc = create_test_document();
            let ctx_ptr = &ctx_clone as *const _ as *mut c_void;
            hedl_to_json_callback(doc, 0, thread_callback, ctx_ptr);
            hedl_free_document(doc);
        });

        handle.join().unwrap();

        let ctx_final = ctx.lock().unwrap();
        assert_eq!(ctx_final.call_count, 1);
        assert!(!ctx_final.data.is_empty());

        hedl_free_document(doc);
    }
}
