# hedl-ffi

**C ABI bindings for HEDL—use the HEDL ecosystem from C, C++, Python (ctypes/cffi), Ruby (FFI), Go (cgo), and any language with C interop.**

Many production systems use C/C++. Legacy code can't be rewritten in Rust overnight. Python, Ruby, Go need access to HEDL without rewriting the parser. Language bridges shouldn't sacrifice performance or correctness. Memory safety bugs in FFI code cause crashes and security vulnerabilities.

`hedl-ffi` provides production-grade C bindings to the complete HEDL implementation. Thread-safe error handling via thread-local storage. Memory safety through poison pointer detection and explicit freeing functions. Zero-copy callback patterns for large outputs. Comprehensive audit logging for production debugging.

## What's Implemented

Complete C API with safety and observability:

1. **Parsing & Validation**: Parse HEDL from strings, validate without parsing
2. **Format Conversion**: Convert HEDL to/from JSON, YAML, XML, CSV, Parquet, Neo4j Cypher, TOON (optional features)
3. **Operations**: Canonicalize, lint with diagnostics, gather statistics
4. **18 Error Codes**: Comprehensive error classification (OK, Parse, UTF-8, Alloc, etc.)
5. **Thread-Safe Error Handling**: Thread-local storage for error messages
6. **Memory Safety**: 4 explicit freeing functions (string, document, diagnostics, bytes)
7. **Poison Pointer Detection**: Use-after-free prevention with poison values
8. **UTF-8 Validation**: All string I/O checked for valid UTF-8
9. **Zero-Copy Callbacks**: Streaming output via callback functions (no full buffering)
10. **Async Operations**: Asynchronous operations with completion callbacks
11. **Audit Logging**: Operation logging via `tracing` crate for production debugging

## Installation

### Building

```bash
cargo build --release -p hedl-ffi

# Outputs:
#   Linux:   target/release/libhedl.so
#   macOS:   target/release/libhedl.dylib
#   Windows: target/release/hedl.dll
```

### Header File

```bash
cbindgen --config cbindgen.toml --crate hedl-ffi --output hedl.h
```

## Core API

### Parsing

```c
#include "hedl.h"
#include <stdio.h>

const char* hedl_str = "%VERSION: 1.0\n---\nname: Test\nvalue: 42\n";
HedlDocument* doc = NULL;
int result = hedl_parse(hedl_str, -1, 1, &doc);

if (result != HEDL_OK) {
    const char* error = hedl_get_last_error();
    fprintf(stderr, "Parse error: %s\n", error);
    return 1;
}

// Use document...

hedl_free_document(doc);
```

### Validation

```c
const char* hedl_str = "%VERSION: 1.0\n---\nname: Test\nvalue: 42\n";
int result = hedl_validate(hedl_str, -1, 1);

if (result != HEDL_OK) {
    const char* error = hedl_get_last_error();
    fprintf(stderr, "Validation error: %s\n", error);
    return 1;
}

printf("Document is valid\n");
```

### Format Conversion

```c
#include <string.h>

HedlDocument* doc = NULL;
hedl_parse(hedl_str, -1, 1, &doc);

// HEDL → JSON
char* json = NULL;
if (hedl_to_json(doc, 0, &json) == HEDL_OK) {
    printf("%s\n", json);
    hedl_free_string(json);
}

// JSON → HEDL
const char* json_input = "{\"name\": \"Test\"}";
HedlDocument* doc2 = NULL;
if (hedl_from_json(json_input, -1, &doc2) == HEDL_OK) {
    hedl_free_document(doc2);
}

// Other conversions
char* yaml = NULL;
if (hedl_to_yaml(doc, 0, &yaml) == HEDL_OK) {
    printf("%s\n", yaml);
    hedl_free_string(yaml);
}

char* xml = NULL;
if (hedl_to_xml(doc, &xml) == HEDL_OK) {
    printf("%s\n", xml);
    hedl_free_string(xml);
}

char* csv = NULL;
if (hedl_to_csv(doc, &csv) == HEDL_OK) {
    printf("%s\n", csv);
    hedl_free_string(csv);
}

hedl_free_document(doc);
```

### Canonicalization

```c
HedlDocument* doc = NULL;
hedl_parse(hedl_str, -1, 1, &doc);

char* canonical = NULL;
if (hedl_canonicalize(doc, &canonical) == HEDL_OK) {
    printf("%s\n", canonical);
    hedl_free_string(canonical);
}

hedl_free_document(doc);
```

### Linting

```c
HedlDocument* doc = NULL;
hedl_parse(hedl_str, -1, 1, &doc);

HedlDiagnostics* diags = NULL;
if (hedl_lint(doc, &diags) == HEDL_OK) {
    int count = hedl_diagnostics_count(diags);
    for (int i = 0; i < count; i++) {
        char* msg = NULL;
        hedl_diagnostics_get(diags, i, &msg);
        int severity = hedl_diagnostics_severity(diags, i);

        const char* sev_str = (severity == 0) ? "Hint" :
                              (severity == 1) ? "Warning" : "Error";
        printf("[%s] %s\n", sev_str, msg);
        hedl_free_string(msg);
    }
    hedl_free_diagnostics(diags);
}

hedl_free_document(doc);
```

### Document Information

```c
HedlDocument* doc = NULL;
hedl_parse(hedl_str, -1, 1, &doc);

// Get version
int major = 0, minor = 0;
if (hedl_get_version(doc, &major, &minor) == HEDL_OK) {
    printf("Version: %d.%d\n", major, minor);
}

// Get schema/alias/root counts
int schema_count = hedl_schema_count(doc);
int alias_count = hedl_alias_count(doc);
int root_count = hedl_root_item_count(doc);

printf("Schemas: %d\n", schema_count);
printf("Aliases: %d\n", alias_count);
printf("Root items: %d\n", root_count);

hedl_free_document(doc);
```

### Zero-Copy Output Callback

For large outputs, use callback pattern to avoid full buffering:

```c
// HedlOutputCallback signature:
// typedef void (*HedlOutputCallback)(const char *data, uintptr_t len, void *user_data);

void write_callback(const char* data, uintptr_t len, void* user_data) {
    FILE* fp = (FILE*)user_data;
    fwrite(data, 1, len, fp);
}

HedlDocument* doc = NULL;
hedl_parse(hedl_str, -1, 1, &doc);

FILE* fp = fopen("output.json", "w");
hedl_to_json_callback(doc, 0, write_callback, fp);
fclose(fp);

hedl_free_document(doc);
```

Available callback functions:
- `hedl_to_json_callback()` - JSON output via callback
- `hedl_to_yaml_callback()` - YAML output via callback
- `hedl_to_xml_callback()` - XML output via callback
- `hedl_to_csv_callback()` - CSV output via callback
- `hedl_to_neo4j_cypher_callback()` - Neo4j Cypher output via callback
- `hedl_canonicalize_callback()` - Canonical output via callback

## Error Handling

### Error Codes

All functions return an integer error code:

```c
#define HEDL_OK                  0    // Success
#define HEDL_ERR_NULL_PTR       -1    // NULL pointer argument
#define HEDL_ERR_INVALID_UTF8   -2    // Invalid UTF-8
#define HEDL_ERR_PARSE          -3    // Parse syntax error
#define HEDL_ERR_CANONICALIZE   -4    // Canonicalization error
#define HEDL_ERR_JSON           -5    // JSON conversion error
#define HEDL_ERR_ALLOC          -6    // Memory allocation failure
#define HEDL_ERR_YAML           -7    // YAML conversion error
#define HEDL_ERR_XML            -8    // XML conversion error
#define HEDL_ERR_CSV            -9    // CSV conversion error
#define HEDL_ERR_PARQUET       -10    // Parquet conversion error
#define HEDL_ERR_LINT          -11    // Linting error
#define HEDL_ERR_NEO4J         -12    // Neo4j conversion error
#define HEDL_ERR_TOON          -13    // TOON conversion error
#define HEDL_ERR_REENTRANT_CALL -14   // Reentrant call detected
#define HEDL_ERR_CANCELLED     -15    // Async operation cancelled
#define HEDL_ERR_QUEUE_FULL    -16    // Async queue full
#define HEDL_ERR_INVALID_HANDLE -17   // Invalid handle
```

### Thread-Local Error Messages

```c
// Get last error message (thread-local)
const char* error = hedl_get_last_error();

// Thread-safe alias (same as hedl_get_last_error)
const char* error = hedl_get_last_error_threadsafe();

// Clear error state
hedl_clear_error_threadsafe();
```

**Thread Safety**: Error messages stored in thread-local storage. Each thread maintains independent error state. Safe to call from multiple threads without synchronization.

## Memory Management

### Freeing Functions

```c
// Free string returned by hedl_to_* and hedl_canonicalize
void hedl_free_string(char* s);

// Free document returned by hedl_parse or hedl_from_*
void hedl_free_document(HedlDocument* doc);

// Free diagnostics returned by hedl_lint
void hedl_free_diagnostics(HedlDiagnostics* diags);

// Free byte array returned by hedl_to_parquet
void hedl_free_bytes(uint8_t* data, uintptr_t len);
```

**CRITICAL**: Always call appropriate freeing function for each allocated object. Memory leaks occur if not freed. Only pass pointers that were allocated by HEDL functions.

### Ownership and Lifetime

- Strings returned by `hedl_to_*` functions are allocated by HEDL and must be freed with `hedl_free_string()`
- Documents returned by `hedl_parse()` and `hedl_from_*()` are allocated by HEDL and must be freed with `hedl_free_document()`
- Diagnostics returned by `hedl_lint()` are allocated by HEDL and must be freed with `hedl_free_diagnostics()`
- Byte arrays returned by `hedl_to_parquet()` are allocated by HEDL and must be freed with `hedl_free_bytes(data, len)`
- Input parameters (HEDL/JSON/YAML strings) are NOT owned by the library; caller must manage their lifetime

### Use-After-Free Protection

Poison pointer detection prevents most use-after-free bugs:

```c
HedlDocument* doc = NULL;
hedl_parse(input, -1, 0, &doc);
hedl_free_document(doc);

// Using doc after free returns error code instead of crashing
char* json = NULL;
int result = hedl_to_json(doc, 0, &json);  // Returns HEDL_ERR_NULL_PTR
```

## UTF-8 Validation

All string inputs and outputs are validated:

```c
// Input validation
const char* invalid_utf8 = "\xFF\xFE invalid";
HedlDocument* doc = NULL;
int result = hedl_parse(invalid_utf8, -1, 0, &doc);
// Returns HEDL_ERR_INVALID_UTF8

// Output validation
// All hedl_to_* functions guarantee valid UTF-8 output
```

**Guarantees**:
- No invalid UTF-8 propagated through API
- Error code returned on malformed input
- Safe for UTF-8 expecting consumers

## Audit Logging

The FFI library integrates with the `tracing` crate for comprehensive operation logging. Logging captures:

- Function entry/exit with timing
- Sanitized parameters (pointer addresses masked)
- Success/failure outcomes with error details
- Performance metrics (call duration)
- Thread context

**Configuring Logging**:

Set the `RUST_LOG` environment variable:

```bash
# Log all INFO and above
export RUST_LOG=info

# Log only FFI audit events at DEBUG level
export RUST_LOG=hedl_ffi::audit=debug

# Log everything at DEBUG level
export RUST_LOG=debug
```

**In Application Code** (Rust):

```rust
use tracing_subscriber::EnvFilter;

tracing_subscriber::fmt()
    .with_env_filter(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"))
    )
    .with_target(true)
    .with_thread_ids(true)
    .with_line_number(true)
    .init();

// Now FFI calls are logged
```

**Log Output Example**:
```
2025-01-05T10:30:45.123Z INFO hedl_ffi::audit: FFI call started function="hedl_parse" thread_id=ThreadId(1) depth=0
2025-01-05T10:30:45.125Z DEBUG hedl_ffi::audit: FFI call parameters function="hedl_parse" params=[("input_len", "1024")]
2025-01-05T10:30:45.130Z INFO hedl_ffi::audit: FFI call completed function="hedl_parse" duration_ms=7.2 status="success"
```

## Language Bindings

### C

```c
#include "hedl.h"
#include <stdio.h>

const char* input = "%VERSION: 1.0\n---\nname: Test\n";
HedlDocument* doc = NULL;

if (hedl_parse(input, -1, 1, &doc) != HEDL_OK) {
    fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
    return 1;
}

char* json = NULL;
if (hedl_to_json(doc, 0, &json) == HEDL_OK) {
    printf("%s\n", json);
    hedl_free_string(json);
}

hedl_free_document(doc);
```

### C++

```cpp
#include "hedl.h"
#include <stdexcept>
#include <string>

class HedlDoc {
    HedlDocument* doc_;
public:
    HedlDoc(const char* input, int len = -1) : doc_(nullptr) {
        int result = hedl_parse(input, len, 1, &doc_);
        if (result != HEDL_OK) {
            throw std::runtime_error(hedl_get_last_error());
        }
    }

    ~HedlDoc() {
        if (doc_) hedl_free_document(doc_);
    }

    std::string to_json(int include_metadata = 0) {
        char* json = nullptr;
        int result = hedl_to_json(doc_, include_metadata, &json);
        if (result != HEDL_OK) {
            throw std::runtime_error(hedl_get_last_error());
        }
        std::string result_str(json);
        hedl_free_string(json);
        return result_str;
    }
};
```

### Python (ctypes)

```python
import ctypes

libhedl = ctypes.CDLL("libhedl.so")

# Define function signatures
libhedl.hedl_parse.argtypes = [ctypes.c_char_p, ctypes.c_int, ctypes.c_int, ctypes.POINTER(ctypes.c_void_p)]
libhedl.hedl_parse.restype = ctypes.c_int

libhedl.hedl_to_json.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.POINTER(ctypes.c_char_p)]
libhedl.hedl_to_json.restype = ctypes.c_int

libhedl.hedl_get_last_error.argtypes = []
libhedl.hedl_get_last_error.restype = ctypes.c_char_p

libhedl.hedl_free_document.argtypes = [ctypes.c_void_p]
libhedl.hedl_free_string.argtypes = [ctypes.c_char_p]

# Use
hedl_str = b"%VERSION: 1.0\n---\nname: Test"
doc = ctypes.c_void_p()
result = libhedl.hedl_parse(hedl_str, -1, 1, ctypes.byref(doc))

if result != 0:  # HEDL_OK
    error = libhedl.hedl_get_last_error()
    print(f"Parse error: {error.decode('utf-8')}")
else:
    json_ptr = ctypes.c_char_p()
    if libhedl.hedl_to_json(doc, 0, ctypes.byref(json_ptr)) == 0:
        json_str = json_ptr.value.decode('utf-8')
        print(json_str)
        libhedl.hedl_free_string(json_ptr)
    libhedl.hedl_free_document(doc)
```

### Go (cgo)

```go
package main

/*
#cgo LDFLAGS: -lhedl
#include "hedl.h"
#include <stdlib.h>
*/
import "C"
import (
    "fmt"
    "unsafe"
)

func Parse(input string) (string, error) {
    cinput := C.CString(input)
    defer C.free(unsafe.Pointer(cinput))

    var doc *C.struct_HedlDocument
    result := C.hedl_parse(cinput, -1, 1, &doc)
    if result != C.HEDL_OK {
        return "", fmt.Errorf("parse error: %s", C.GoString(C.hedl_get_last_error()))
    }
    defer C.hedl_free_document(doc)

    var json *C.char
    result = C.hedl_to_json(doc, 0, &json)
    if result != C.HEDL_OK {
        return "", fmt.Errorf("to_json error: %s", C.GoString(C.hedl_get_last_error()))
    }
    defer C.hedl_free_string(json)

    return C.GoString(json), nil
}
```

### Ruby (FFI)

```ruby
require 'ffi'

module Hedl
  extend FFI::Library
  ffi_lib 'hedl'

  # Error codes
  HEDL_OK = 0

  attach_function :hedl_parse, [:string, :int, :int, :pointer], :int
  attach_function :hedl_to_json, [:pointer, :int, :pointer], :int
  attach_function :hedl_get_last_error, [], :string
  attach_function :hedl_free_document, [:pointer], :void
  attach_function :hedl_free_string, [:pointer], :void

  def self.parse(input)
    doc = FFI::MemoryPointer.new(:pointer)
    result = hedl_parse(input, -1, 1, doc)

    unless result == HEDL_OK
      raise "Parse error: #{hedl_get_last_error()}"
    end

    doc_ptr = doc.read_pointer
    return nil if doc_ptr.null?

    json_ptr = FFI::MemoryPointer.new(:pointer)
    result = hedl_to_json(doc_ptr, 0, json_ptr)

    begin
      if result == HEDL_OK
        json_str = json_ptr.read_pointer.read_string
        return json_str
      else
        raise "to_json error: #{hedl_get_last_error()}"
      end
    ensure
      hedl_free_string(json_ptr.read_pointer)
      hedl_free_document(doc_ptr)
    end
  end
end
```

## Complete Function Reference

### Parsing & Validation

- `int hedl_parse(const char *input, int input_len, int strict, HedlDocument **out_doc)`
- `int hedl_validate(const char *input, int input_len, int strict)`

### Format Conversion (HEDL → Other)

- `int hedl_to_json(const HedlDocument *doc, int include_metadata, char **out_str)`
- `int hedl_to_yaml(const HedlDocument *doc, int include_metadata, char **out_str)`
- `int hedl_to_xml(const HedlDocument *doc, char **out_str)`
- `int hedl_to_csv(const HedlDocument *doc, char **out_str)`
- `int hedl_to_parquet(const HedlDocument *doc, uint8_t **out_data, uintptr_t *out_len)`
- `int hedl_to_neo4j_cypher(const HedlDocument *doc, int use_merge, char **out_str)`
- `int hedl_to_toon(const HedlDocument *doc, char **out_str)`

### Format Conversion (Other → HEDL)

- `int hedl_from_json(const char *json, int json_len, HedlDocument **out_doc)`
- `int hedl_from_yaml(const char *yaml, int yaml_len, HedlDocument **out_doc)`
- `int hedl_from_xml(const char *xml, int xml_len, HedlDocument **out_doc)`
- `int hedl_from_parquet(const uint8_t *data, uintptr_t len, HedlDocument **out_doc)`
- `int hedl_from_toon(const char *toon, int toon_len, HedlDocument **out_doc)`

### Operations

- `int hedl_canonicalize(const HedlDocument *doc, char **out_str)`
- `int hedl_lint(const HedlDocument *doc, HedlDiagnostics **out_diag)`
- `int hedl_get_version(const HedlDocument *doc, int *major, int *minor)`
- `int hedl_schema_count(const HedlDocument *doc)`
- `int hedl_alias_count(const HedlDocument *doc)`
- `int hedl_root_item_count(const HedlDocument *doc)`

### Diagnostics

- `int hedl_diagnostics_count(const HedlDiagnostics *diag)`
- `int hedl_diagnostics_get(const HedlDiagnostics *diag, int index, char **out_str)`
- `int hedl_diagnostics_severity(const HedlDiagnostics *diag, int index)`

### Memory Management

- `void hedl_free_string(char *s)`
- `void hedl_free_document(HedlDocument *doc)`
- `void hedl_free_diagnostics(HedlDiagnostics *diag)`
- `void hedl_free_bytes(uint8_t *data, uintptr_t len)`

### Error Handling

- `const char *hedl_get_last_error(void)`
- `const char *hedl_get_last_error_threadsafe(void)`
- `void hedl_clear_error_threadsafe(void)`

### Zero-Copy Callbacks

- `int hedl_to_json_callback(const HedlDocument *doc, int include_metadata, HedlOutputCallback callback, void *user_data)`
- `int hedl_to_yaml_callback(const HedlDocument *doc, int include_metadata, HedlOutputCallback callback, void *user_data)`
- `int hedl_to_xml_callback(const HedlDocument *doc, HedlOutputCallback callback, void *user_data)`
- `int hedl_to_csv_callback(const HedlDocument *doc, HedlOutputCallback callback, void *user_data)`
- `int hedl_to_neo4j_cypher_callback(const HedlDocument *doc, int use_merge, HedlOutputCallback callback, void *user_data)`
- `int hedl_canonicalize_callback(const HedlDocument *doc, HedlOutputCallback callback, void *user_data)`

### Async Operations

- `HedlAsyncOp *hedl_parse_async(const char *input, int input_len, int strict, HedlCompletionCallback callback, void *user_data)`
- `HedlAsyncOp *hedl_lint_async(const HedlDocument *doc, HedlCompletionCallback callback, void *user_data)`
- `HedlAsyncOp *hedl_canonicalize_async(const HedlDocument *doc, HedlCompletionCallback callback, void *user_data)`
- `HedlAsyncOp *hedl_to_json_async(const HedlDocument *doc, int include_metadata, HedlCompletionCallback callback, void *user_data)`
- `HedlAsyncOp *hedl_to_yaml_async(const HedlDocument *doc, int include_metadata, HedlCompletionCallback callback, void *user_data)`
- `HedlAsyncOp *hedl_to_xml_async(const HedlDocument *doc, HedlCompletionCallback callback, void *user_data)`
- `HedlAsyncOp *hedl_to_csv_async(const HedlDocument *doc, HedlCompletionCallback callback, void *user_data)`
- `HedlAsyncOp *hedl_to_neo4j_cypher_async(const HedlDocument *doc, int use_merge, HedlCompletionCallback callback, void *user_data)`
- `HedlAsyncOp *hedl_to_toon_async(const HedlDocument *doc, HedlCompletionCallback callback, void *user_data)`
- `void hedl_async_cancel(HedlAsyncOp *op)`
- `void hedl_async_free(HedlAsyncOp *op)`

## Performance Characteristics

**Overhead**: C FFI adds minimal overhead vs native Rust. Essentially zero-cost abstraction for parsing and conversion.

**Memory**: Same as Rust implementation. No additional allocations beyond necessary conversions.

**Thread Safety**: Functions are thread-safe. Error messages use thread-local storage. Documents are NOT thread-safe by design and should not be shared across threads without external synchronization.

**Callback Performance**: Zero-copy streaming via callbacks avoids full buffering. Suitable for large outputs (>1MB).

**Async Performance**: Non-blocking async operations suitable for I/O-bound tasks. Work-stealing thread pool for efficient parallelism.

## Building and Testing

### Building the FFI Library

```bash
# Build with all format support
cargo build --release -p hedl-ffi

# Build with specific formats
cargo build --release -p hedl-ffi --features json,yaml

# Build C header
cbindgen --config crates/hedl-ffi/cbindgen.toml --crate hedl-ffi --output crates/hedl-ffi/hedl.h
```

### Testing

```bash
# Run all FFI tests
cargo test -p hedl-ffi

# Run with logging
RUST_LOG=hedl_ffi::audit=debug cargo test -p hedl-ffi -- --nocapture
```

## Dependencies

- `hedl-core` - Core parser implementation
- `hedl-lint` - Linting engine
- `libc` 0.2 - C standard library bindings
- `tracing` 0.1 - Structured logging
- Various format libraries (json-ld, serde_yaml, etc.) behind feature gates

## License

Apache-2.0

## Contributing

Contributions welcome! Please ensure:
1. All example code compiles and runs
2. Tests pass with logging enabled
3. Function signatures match hedl.h
4. Memory management is correct (no leaks in examples)
