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

The HEDL FFI implementation follows a comprehensive design with proper error handling, thread safety, and memory management. The main modules are:

- `parsing.rs` - Parse functions
- `operations.rs` - Document operations
- `conversions/` - Format conversion functions
- `error.rs` - Error handling
- `memory.rs` - Memory management
- `types.rs` - Type definitions and constants

### Core Pattern

All FFI functions follow this pattern:

```rust
// 1. Use #[no_mangle] for C symbol export
// 2. Use unsafe extern "C" for C calling convention
// 3. Accept C types (*const c_char, c_int, etc.)
// 4. Return c_int error codes (0 = success)
// 5. Use output parameters via mutable pointers
// 6. Set thread-local error messages for failures

#[no_mangle]
pub unsafe extern "C" fn hedl_parse(
    input: *const c_char,
    input_len: c_int,
    strict: c_int,
    out_doc: *mut *mut HedlDocument,
) -> c_int {
    // Implementation validates all pointers, handles errors,
    // and returns appropriate error codes
}
```

### Error Handling Pattern

```rust
use std::cell::RefCell;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

fn set_error(msg: &str) {
    LAST_ERROR.with(|e| *e.borrow_mut() = Some(msg.to_string()));
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
```

See actual implementation in `crates/hedl-ffi/src/parsing.rs` and `crates/hedl-ffi/src/error.rs`.

### C Header

File: `crates/hedl-ffi/hedl.h`

The main header defines the C API with these key features:

**Opaque Types** (implementation details hidden):
```c
typedef struct HedlDocument HedlDocument;
typedef struct HedlDiagnostics HedlDiagnostics;
```

**Error Codes** (all negative on failure, 0 = success):
```c
#define HEDL_OK 0
#define HEDL_ERR_NULL_PTR -1
#define HEDL_ERR_INVALID_UTF8 -2
#define HEDL_ERR_PARSE -3
#define HEDL_ERR_JSON -5
// ... more format-specific errors
```

**Core Functions**:
```c
// Parse HEDL document
int hedl_parse(
    const char* input,      // UTF-8 text
    int input_len,          // -1 for null-terminated
    int strict,             // Non-zero for strict validation
    HedlDocument** out_doc  // Output handle
);

// Get error message (thread-safe)
const char* hedl_get_last_error(void);

// Convert to JSON
int hedl_to_json(
    HedlDocument* doc,
    int include_metadata,
    char** out_json
);

// Memory management
void hedl_free_document(HedlDocument* doc);
void hedl_free_string(char* str);
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

### Format Conversion Functions

The HEDL FFI provides functions to convert between formats:

```c
// Convert HEDL to JSON
int hedl_to_json(HedlDocument* doc, int include_metadata, char** out_json);

// Convert HEDL to YAML
int hedl_to_yaml(HedlDocument* doc, char** out_yaml);

// Convert HEDL to XML
int hedl_to_xml(HedlDocument* doc, int include_metadata, char** out_xml);

// Convert HEDL to CSV
int hedl_to_csv(HedlDocument* doc, char** out_csv);
```

**Example C Code**:
```c
HedlDocument* doc = NULL;
char* json_output = NULL;

// Parse HEDL
if (hedl_parse(hedl_text, -1, 1, &doc) != HEDL_OK) {
    fprintf(stderr, "Parse failed: %s\n", hedl_get_last_error());
    return;
}

// Convert to JSON
if (hedl_to_json(doc, 0, &json_output) == HEDL_OK) {
    printf("JSON: %s\n", json_output);
    hedl_free_string(json_output);  // Must free returned strings
}

hedl_free_document(doc);
```

See `crates/hedl-ffi/src/conversions/` for callback-based implementations.

### Thread Safety

**Error Handling**: Thread-safe via thread-local storage:

```c
// Each thread has independent error state
const char* hedl_get_last_error(void);              // Thread-local
const char* hedl_get_last_error_threadsafe(void);   // Explicit thread-safe version
```

**Multi-threaded Example**:
```c
#include <pthread.h>

void* worker_thread(void* arg) {
    const char* hedl_input = (const char*)arg;
    HedlDocument* doc = NULL;

    // Each thread gets independent error state
    int result = hedl_parse(hedl_input, -1, 1, &doc);
    if (result != HEDL_OK) {
        // This error is for THIS thread only - won't interfere with other threads
        fprintf(stderr, "[Thread %ld] Error: %s\n", pthread_self(),
                hedl_get_last_error_threadsafe());
        return NULL;
    }

    // Process document...
    hedl_free_document(doc);
    return (void*)1;
}

int main() {
    pthread_t threads[4];
    const char* inputs[4] = { /* ... */ };

    for (int i = 0; i < 4; i++) {
        pthread_create(&threads[i], NULL, worker_thread, (void*)inputs[i]);
    }

    for (int i = 0; i < 4; i++) {
        pthread_join(threads[i], NULL);
    }
}
```

**Document Handles**: NOT thread-safe - each thread must create/manage its own:
- Don't share `HedlDocument*` between threads
- Each thread should parse its own documents
- Use appropriate synchronization if necessary

## Testing FFI

FFI tests are located in `crates/hedl-ffi/tests/`. Run with:

```bash
cargo test -p hedl-ffi
```

**Example Rust test pattern** (from actual tests):

```rust
#[test]
fn test_hedl_parse_valid() {
    unsafe {
        let input = CString::new("%VERSION: 1.0\n---\nname: Alice\nage: 30").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_parse(input.as_ptr(), -1, 1, &mut doc);

        assert_eq!(result, HEDL_OK);
        assert!(!doc.is_null());

        hedl_free_document(doc);
    }
}

#[test]
fn test_hedl_parse_invalid() {
    unsafe {
        let input = CString::new("invalid: [[[").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        let result = hedl_parse(input.as_ptr(), -1, 1, &mut doc);

        // Parse error code is -3
        assert_eq!(result, HEDL_ERR_PARSE);
        assert!(doc.is_null());

        // Error message is available
        let error = hedl_get_last_error();
        assert!(!error.is_null());

        let err_str = CStr::from_ptr(error);
        println!("Error: {:?}", err_str);
    }
}
```

**Testing Checklist**:
- Valid HEDL parsing succeeds
- Invalid HEDL returns error code
- Error messages are populated
- Memory is properly freed
- Null pointer handling is correct
- Multi-threaded error isolation works

## Related

- [FFI API Reference](../../api/ffi-api.md)
- [Safety Guidelines](../guides/api-design.md)
