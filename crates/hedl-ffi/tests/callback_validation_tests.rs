// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Callback function validation and execution tests.

use hedl_ffi::*;
use std::os::raw::{c_char, c_void};
use std::ptr;
#[cfg(feature = "json")]
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

// Thread-safe counter for use with user_data
struct CallbackCounter {
    count: AtomicUsize,
    data_size: AtomicUsize,
}

impl CallbackCounter {
    fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
            data_size: AtomicUsize::new(0),
        }
    }
}

// Callback that uses user_data for thread-safe counting
unsafe extern "C" fn counting_callback_with_userdata(
    data: *const c_char,
    len: usize,
    user_data: *mut c_void,
) {
    if !user_data.is_null() {
        let counter = &*(user_data as *const CallbackCounter);
        counter.count.fetch_add(1, Ordering::SeqCst);
        counter.data_size.store(len, Ordering::SeqCst);
    }
    assert!(!data.is_null());
}

// Callback that copies data for verification
fn get_callback_buffer() -> &'static Mutex<Vec<u8>> {
    static CALLBACK_BUFFER: OnceLock<Mutex<Vec<u8>>> = OnceLock::new();
    CALLBACK_BUFFER.get_or_init(|| Mutex::new(Vec::new()))
}

unsafe extern "C" fn copying_callback(data: *const c_char, len: usize, _user_data: *mut c_void) {
    let slice = std::slice::from_raw_parts(data.cast::<u8>(), len);
    *get_callback_buffer().lock().unwrap() = slice.to_vec();
}

// Callback that verifies user_data pointer
#[cfg(feature = "json")]
unsafe extern "C" fn userdata_callback(_data: *const c_char, _len: usize, user_data: *mut c_void) {
    if !user_data.is_null() {
        let value = *(user_data as *const i32);
        assert_eq!(value, 42);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_json_callback_basic() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let counter = CallbackCounter::new();

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: \"value\"\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let result = hedl_to_json_callback(
            doc,
            0,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(counter.count.load(Ordering::SeqCst), 1);
        assert!(counter.data_size.load(Ordering::SeqCst) > 0);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_json_callback_with_user_data() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let user_value: i32 = 42;

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ntest: 123\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let result = hedl_to_json_callback(
            doc,
            0,
            userdata_callback,
            std::ptr::addr_of!(user_value) as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_json_callback_data_content() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        get_callback_buffer().lock().unwrap().clear();

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: \"test_value\"\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let result = hedl_to_json_callback(doc, 0, copying_callback, ptr::null_mut());

        assert_eq!(result, HEDL_OK);
        {
            let buffer = get_callback_buffer().lock().unwrap();
            assert!(!buffer.is_empty());
            let data_str = String::from_utf8_lossy(&buffer);
            assert!(data_str.contains("test_value"));
        }

        hedl_free_document(doc);
    }
}

#[cfg(feature = "yaml")]
#[test]
fn test_yaml_callback_basic() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let counter = CallbackCounter::new();

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let result = hedl_to_yaml_callback(
            doc,
            0,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(counter.count.load(Ordering::SeqCst), 1);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "xml")]
#[test]
fn test_xml_callback_basic() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let counter = CallbackCounter::new();

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nroot: { child: \"value\" }\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let result = hedl_to_xml_callback(
            doc,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(counter.count.load(Ordering::SeqCst), 1);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "csv")]
#[test]
fn test_csv_callback_basic() {
    // SAFETY: Unsafe operation required for FFI boundary
    unsafe {
        let counter = CallbackCounter::new();

        let input =
            b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nrows: [\n{ a: 1, b: 2 },\n{ a: 3, b: 4 }\n]\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let result = hedl_to_csv_callback(
            doc,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );

        if result == HEDL_OK {
            assert_eq!(counter.count.load(Ordering::SeqCst), 1);
        }

        hedl_free_document(doc);
    }
}

#[cfg(feature = "neo4j")]
#[test]
fn test_neo4j_callback_basic() {
    // SAFETY: Unsafe operation required for FFI boundary
    unsafe {
        let counter = CallbackCounter::new();

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nnode: { id: 1 }\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        let parse_result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(parse_result, HEDL_OK);

        let result = hedl_to_neo4j_cypher_callback(
            doc,
            1,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(counter.count.load(Ordering::SeqCst), 1);

        hedl_free_document(doc);
    }
}

#[test]
fn test_canonicalize_callback_basic() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let counter = CallbackCounter::new();

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        let parse_result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(parse_result, HEDL_OK, "Parse failed");

        let result = hedl_canonicalize_callback(
            doc,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );

        if result != HEDL_OK {
            let err = hedl_get_last_error();
            if !err.is_null() {
                let err_str = std::ffi::CStr::from_ptr(err).to_str().unwrap();
                eprintln!("Error: {err_str}");
            }
        }

        assert_eq!(result, HEDL_OK);
        assert_eq!(counter.count.load(Ordering::SeqCst), 1);

        hedl_free_document(doc);
    }
}

#[test]
fn test_canonicalize_callback_data_content() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        get_callback_buffer().lock().unwrap().clear();

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let result = hedl_canonicalize_callback(doc, copying_callback, ptr::null_mut());

        assert_eq!(result, HEDL_OK);
        {
            let buffer = get_callback_buffer().lock().unwrap();
            assert!(!buffer.is_empty());
            let data_str = String::from_utf8_lossy(&buffer);
            assert!(data_str.contains("%V:"));
            assert!(data_str.contains("key"));
        }

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_callback_invoked_only_once() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let counter = CallbackCounter::new();

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ndata: \"test\"\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        hedl_to_json_callback(
            doc,
            0,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );

        assert_eq!(counter.count.load(Ordering::SeqCst), 1);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_callback_with_complex_data() {
    // SAFETY: Unsafe operation required for FFI boundary
    unsafe {
        let counter = CallbackCounter::new();

        let input =
            b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\ncomplex: { nested: { deep: { value: 42 } } }\0";

        let mut doc: *mut HedlDocument = ptr::null_mut();
        let parse_result = hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);
        assert_eq!(parse_result, HEDL_OK);

        let result = hedl_to_json_callback(
            doc,
            0,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(counter.count.load(Ordering::SeqCst), 1);
        assert!(counter.data_size.load(Ordering::SeqCst) > 0);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_callback_with_large_output() {
    // SAFETY: Unsafe operation required for FFI boundary
    unsafe {
        let counter = CallbackCounter::new();

        // Create a document that will produce large output
        let mut input = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nitems: [");
        for i in 0..1000 {
            if i > 0 {
                input.push_str(", ");
            }
            input.push_str(&format!("{{ id: {i}, value: \"item{i}\" }}"));
        }
        input.push_str("]\0");

        let c_input = std::ffi::CString::new(input.trim_end_matches('\0')).unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(c_input.as_ptr(), -1, 0, &mut doc);

        let result = hedl_to_json_callback(
            doc,
            0,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );

        assert_eq!(result, HEDL_OK);
        assert_eq!(counter.count.load(Ordering::SeqCst), 1);
        assert!(counter.data_size.load(Ordering::SeqCst) > 1000);

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_multiple_callbacks_different_formats() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        let counter = CallbackCounter::new();

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        // JSON callback
        hedl_to_json_callback(
            doc,
            0,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );
        let json_count = counter.count.load(Ordering::SeqCst);
        assert_eq!(json_count, 1);

        // Canonicalize callback
        hedl_canonicalize_callback(
            doc,
            counting_callback_with_userdata,
            std::ptr::addr_of!(counter) as *mut c_void,
        );
        let canon_count = counter.count.load(Ordering::SeqCst);
        assert_eq!(canon_count, 2);

        hedl_free_document(doc);
    }
}

// Test callback that tracks if it was called
#[cfg(feature = "json")]
static CALLBACK_INVOKED: AtomicBool = AtomicBool::new(false);

#[cfg(feature = "json")]
unsafe extern "C" fn tracking_callback(_data: *const c_char, _len: usize, _user_data: *mut c_void) {
    CALLBACK_INVOKED.store(true, Ordering::SeqCst);
}

#[cfg(feature = "json")]
#[test]
fn test_callback_definitely_invoked() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        CALLBACK_INVOKED.store(false, Ordering::SeqCst);

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        let result = hedl_to_json_callback(doc, 0, tracking_callback, ptr::null_mut());

        assert_eq!(result, HEDL_OK);
        assert!(CALLBACK_INVOKED.load(Ordering::SeqCst));

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_callback_data_lifetime() {
    // SAFETY: Unsafe operation required for FFI boundary
    unsafe {
        // This test verifies that data passed to callback is valid
        static mut DATA_PTR: *const c_char = ptr::null();
        static mut DATA_LEN: usize = 0;

        unsafe extern "C" fn lifetime_callback(
            data: *const c_char,
            len: usize,
            _user_data: *mut c_void,
        ) {
            DATA_PTR = data;
            DATA_LEN = len;

            // Data should be valid during callback
            assert!(!data.is_null());
            assert!(len > 0);

            // We can read the data
            let slice = std::slice::from_raw_parts(data.cast::<u8>(), len);
            let _ = String::from_utf8_lossy(slice);
        }

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        hedl_to_json_callback(doc, 0, lifetime_callback, ptr::null_mut());

        // After callback returns, we should not use the data pointer
        // (This is just documenting the contract, not testing it)

        hedl_free_document(doc);
    }
}

#[cfg(feature = "json")]
#[test]
fn test_callback_with_metadata_flag() {
    // SAFETY: Testing FFI function with known-valid input
    unsafe {
        get_callback_buffer().lock().unwrap().clear();

        let input = b"%V:2.0\n%NULL:~\n%QUOTE:\"\n---\nkey: value\0";
        let mut doc: *mut HedlDocument = ptr::null_mut();
        hedl_parse(input.as_ptr().cast::<c_char>(), -1, 0, &mut doc);

        // Test with metadata
        hedl_to_json_callback(doc, 1, copying_callback, ptr::null_mut());
        let with_metadata_len = {
            let buffer = get_callback_buffer().lock().unwrap();
            buffer.len()
        };

        // Test without metadata
        get_callback_buffer().lock().unwrap().clear();
        hedl_to_json_callback(doc, 0, copying_callback, ptr::null_mut());
        let without_metadata_len = {
            let buffer = get_callback_buffer().lock().unwrap();
            buffer.len()
        };

        // Both should produce output
        assert!(with_metadata_len > 0);
        assert!(without_metadata_len > 0);

        hedl_free_document(doc);
    }
}
