# How-To: Write FFI Bindings

Create safe C-compatible FFI bindings for HEDL.

## Goal

Expose Rust functionality to C/C++ and other languages via FFI.

## Core Principles

1. **No panics across FFI boundary**
2. **Clear ownership semantics**
3. **Proper error handling**
4. **Memory safety guarantees**
5. **Thread safety where appropriate**

## Basic FFI Pattern

### Rust Side

File: `crates/hedl-ffi/src/lib.rs`

```rust
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use hedl_core::{parse, Document};

/// Opaque pointer to HEDL document
pub struct HedlDocument {
    inner: Document,
}

/// Error code enum
#[repr(C)]
pub enum HedlErrorCode {
    Success = 0,
    ParseError = 1,
    NullPointer = 2,
    Utf8Error = 3,
}

/// Parse HEDL from C string
///
/// # Safety
///
/// - `input` must be valid UTF-8 C string
/// - Caller owns returned document, must free with hedl_free_document
/// - Error messages are stored in thread-local storage, retrieve with hedl_get_last_error()
#[no_mangle]
pub unsafe extern "C" fn hedl_parse(
    input: *const c_char,
    input_len: c_int,
    strict: c_int,
    out_doc: *mut *mut HedlDocument,
) -> c_int {
    // Clear previous error
    clear_error();

    // Check for null
    if input.is_null() || out_doc.is_null() {
        set_error("Null pointer argument");
        return HEDL_ERR_NULL_PTR;
    }

    // Get input string (handles both null-terminated and length-specified)
    let input_str = match get_input_string(input, input_len) {
        Ok(s) => s,
        Err(code) => return code,
    };

    // Create parse options
    let options = ParseOptions {
        strict_refs: strict != 0,
        ..Default::default()
    };

    // Parse document
    match parse_with_limits(input_str.as_bytes(), options) {
        Ok(doc) => {
            let handle = Box::new(HedlDocument { inner: doc });
            *out_doc = Box::into_raw(handle);
            HEDL_OK
        }
        Err(e) => {
            let msg = format!("Parse error: {}", e);
            set_error(&msg);
            *out_doc = std::ptr::null_mut();
            HEDL_ERR_PARSE
        }
    }
}

/// Free document
///
/// # Safety
///
/// - `doc` must be valid pointer from hedl_parse
/// - Must only be called once per document
#[no_mangle]
pub unsafe extern "C" fn hedl_free_document(doc: *mut HedlDocument) {
    if !doc.is_null() {
        drop(Box::from_raw(doc));
    }
}

/// Free C string returned by HEDL
// Error handling helpers (thread-local storage)
use std::cell::RefCell;
use std::os::raw::c_void;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

fn set_error(msg: &str) {
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg.to_string()));
}

fn clear_error() {
    LAST_ERROR.with(|e| *e.borrow_mut() = None);
}

#[no_mangle]
pub extern "C" fn hedl_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|s| s.as_ptr() as *const c_char)
            .unwrap_or(std::ptr::null())
    })
}

// Helper function to get input string
fn get_input_string(input: *const c_char, input_len: c_int) -> Result<String, c_int> {
    unsafe {
        if input_len < 0 {
            // Null-terminated string
            CStr::from_ptr(input)
                .to_str()
                .map(|s| s.to_string())
                .map_err(|_| {
                    set_error("Invalid UTF-8");
                    HEDL_ERR_INVALID_UTF8
                })
        } else {
            // Length-specified string
            let slice = std::slice::from_raw_parts(input as *const u8, input_len as usize);
            std::str::from_utf8(slice)
                .map(|s| s.to_string())
                .map_err(|_| {
                    set_error("Invalid UTF-8");
                    HEDL_ERR_INVALID_UTF8
                })
        }
    }
}
```

### C Header

File: `crates/hedl-ffi/hedl.h`

```c
#ifndef HEDL_H
#define HEDL_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque types
typedef struct HedlDocument HedlDocument;

// Error codes
#define HEDL_OK 0
#define HEDL_ERR_NULL_PTR 1
#define HEDL_ERR_PARSE 2
#define HEDL_ERR_INVALID_UTF8 3
// ... other error codes

// Parse HEDL document
// input: UTF-8 encoded HEDL text
// input_len: Length in bytes, or -1 for null-terminated string
// strict: Non-zero for strict reference validation
// out_doc: Output pointer for document handle
// Returns: HEDL_OK on success, error code on failure
int hedl_parse(const char* input, int input_len, int strict, HedlDocument** out_doc);

// Get last error message for current thread
const char* hedl_get_last_error(void);

// Free document
void hedl_free_document(HedlDocument* doc);


#ifdef __cplusplus
}
#endif

#endif // HEDL_H
```

### C Usage

```c
#include "hedl.h"
#include <stdio.h>

int main() {
    const char* hedl_text = "%VERSION: 1.0\n---\nname: Alice\nage: 30";
    HedlDocument* doc = NULL;

    int result = hedl_parse(hedl_text, -1, 1, &doc);

    if (result != HEDL_OK) {
        const char* error = hedl_get_last_error();
        fprintf(stderr, "Parse error: %s\n", error);
        return 1;
    }

    printf("Parsed successfully!\n");

    hedl_free_document(doc);
    return 0;
}
```

## Advanced Patterns

### Callback-based Output (Zero-Copy Transfer)

The HEDL FFI provides callback-based functions for format conversions:

```rust
use std::os::raw::c_void;

/// Callback function type for receiving output data
pub type HedlOutputCallback = unsafe extern "C" fn(
    data: *const c_char,
    len: usize,
    user_data: *mut c_void,
);

#[no_mangle]
pub unsafe extern "C" fn hedl_to_json_callback(
    doc: *const HedlDocument,
    include_metadata: c_int,
    callback: HedlOutputCallback,
    user_data: *mut c_void,
) -> c_int {
    // ... validation ...
    let json = match hedl_json::to_json(&(*doc).inner, &config) {
        Ok(j) => j,
        Err(e) => {
            set_error(&format!("JSON conversion error: {}", e));
            return HEDL_ERR_JSON;
        }
    };

    // Call callback with JSON data
    let data = json.as_ptr() as *const c_char;
    let len = json.len();
    callback(data, len, user_data);
    HEDL_OK
}
```

### Thread Safety

The HEDL FFI uses **thread-local error storage** for thread-safe error handling:

```rust
use std::cell::RefCell;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

#[no_mangle]
pub extern "C" fn hedl_get_last_error_threadsafe() -> *const c_char {
    // Returns error for CURRENT thread only
    // No locks required - wait-free access
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|s| s.as_ptr() as *const c_char)
            .unwrap_or(std::ptr::null())
    })
}
```

**Note**: Document handles themselves are NOT thread-safe. Each thread should create and manage its own documents.

## Testing FFI

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use std::ptr;

    #[test]
    fn test_ffi_round_trip() {
        unsafe {
            let input = CString::new("%VERSION: 1.0\n---\nname: Alice").unwrap();
            let mut doc: *mut HedlDocument = ptr::null_mut();

            let result = hedl_parse(input.as_ptr(), -1, 1, &mut doc);
            assert_eq!(result, HEDL_OK);
            assert!(!doc.is_null());

            hedl_free_document(doc);
        }
    }

    #[test]
    fn test_ffi_error_handling() {
        unsafe {
            let input = CString::new("invalid: [[[").unwrap();
            let mut doc: *mut HedlDocument = ptr::null_mut();

            let result = hedl_parse(input.as_ptr(), -1, 1, &mut doc);
            assert_eq!(result, HEDL_ERR_PARSE);
            assert!(doc.is_null());

            let error = hedl_get_last_error();
            assert!(!error.is_null());
        }
    }
}
```

## Related

- [FFI API Reference](../../api/ffi-api.md)
- [Safety Guidelines](../guides/api-design.md)
