# hedl-ffi

C ABI bindings for HEDL. Use the HEDL parser from C, C++, Python (ctypes/cffi), Ruby (FFI), Go (cgo), or any language with C interop.

## Why This Exists

HEDL is implemented in Rust, but most systems need to integrate with code written in other languages. Rather than maintaining separate parser implementations (with inevitable drift and bugs), this crate exposes the Rust implementation through a stable C interface. The bindings add minimal overhead while preserving memory safety guarantees through explicit ownership and defensive checks.

## Building

```bash
cargo build --release -p hedl-ffi
```

This produces `libhedl.so` on Linux, `libhedl.dylib` on macOS, or `hedl.dll` on Windows.

To generate the C header file:

```bash
cbindgen --config cbindgen.toml --crate hedl-ffi --output hedl.h
```

## Basic Usage

### Parsing and Converting

```c
#include "hedl.h"
#include <stdio.h>

const char* hedl_str = "%V:2.0\n---\nname: Test\nvalue: 42\n";
HedlDocument* doc = NULL;

// Parse with strict mode enabled (the 1 parameter)
int result = hedl_parse(hedl_str, -1, 1, &doc);
if (result != HEDL_OK) {
    fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
    return 1;
}

// Convert to JSON
char* json = NULL;
if (hedl_to_json(doc, 0, &json) == HEDL_OK) {
    printf("%s\n", json);
    hedl_free_string(json);
}

hedl_free_document(doc);
```

The `-1` for `input_len` tells the parser to use `strlen()` internally. Pass the actual length if you have binary data or want to avoid the extra scan.

### Validation Without Parsing

If you only need to check whether input is valid HEDL:

```c
int result = hedl_validate(hedl_str, -1, 1);
if (result != HEDL_OK) {
    fprintf(stderr, "Invalid: %s\n", hedl_get_last_error());
}
```

This is faster than full parsing when you do not need the resulting document.

### Format Conversions

The library supports bidirectional conversion with JSON, YAML, XML, CSV, Parquet, Neo4j Cypher, and TOON. Each format is behind a feature flag, so you only pay for what you use.

```c
// HEDL to other formats
hedl_to_json(doc, include_metadata, &out);
hedl_to_yaml(doc, include_metadata, &out);
hedl_to_xml(doc, &out);
hedl_to_csv(doc, &out);

// Other formats to HEDL
hedl_from_json(json_str, -1, &doc);
hedl_from_yaml(yaml_str, -1, &doc);
hedl_from_xml(xml_str, -1, &doc);
```

### Streaming Large Outputs

When converting large documents, buffering the entire output in memory is wasteful. Use the callback variants to stream output directly to a file or network socket:

```c
void write_callback(const char* data, uintptr_t len, void* user_data) {
    FILE* fp = (FILE*)user_data;
    fwrite(data, 1, len, fp);
}

FILE* fp = fopen("output.json", "w");
hedl_to_json_callback(doc, 0, write_callback, fp);
fclose(fp);
```

All conversion functions have callback variants: `hedl_to_json_callback`, `hedl_to_yaml_callback`, `hedl_to_xml_callback`, `hedl_to_csv_callback`, `hedl_to_neo4j_cypher_callback`, and `hedl_canonicalize_callback`.

### Linting

```c
HedlDiagnostics* diags = NULL;
if (hedl_lint(doc, &diags) == HEDL_OK) {
    int count = hedl_diagnostics_count(diags);
    for (int i = 0; i < count; i++) {
        char* msg = NULL;
        hedl_diagnostics_get(diags, i, &msg);
        int severity = hedl_diagnostics_severity(diags, i);
        // severity: 0 = Hint, 1 = Warning, 2 = Error
        printf("[%d] %s\n", severity, msg);
        hedl_free_string(msg);
    }
    hedl_free_diagnostics(diags);
}
```

### Document Inspection

```c
int major = 0, minor = 0;
hedl_get_version(doc, &major, &minor);

int schemas = hedl_schema_count(doc);
int aliases = hedl_alias_count(doc);
int roots = hedl_root_item_count(doc);
```

## Error Handling

All functions return an integer error code. Zero means success; negative values indicate specific failure modes:

| Code | Name | Meaning |
|------|------|---------|
| 0 | `HEDL_OK` | Success |
| -1 | `HEDL_ERR_NULL_PTR` | NULL pointer passed where valid pointer expected |
| -2 | `HEDL_ERR_INVALID_UTF8` | Input contains invalid UTF-8 sequences |
| -3 | `HEDL_ERR_PARSE` | HEDL syntax error |
| -4 | `HEDL_ERR_CANONICALIZE` | Canonicalization failed |
| -5 | `HEDL_ERR_JSON` | JSON conversion failed |
| -6 | `HEDL_ERR_ALLOC` | Memory allocation failed |
| -7 | `HEDL_ERR_YAML` | YAML conversion failed |
| -8 | `HEDL_ERR_XML` | XML conversion failed |
| -9 | `HEDL_ERR_CSV` | CSV conversion failed |
| -10 | `HEDL_ERR_PARQUET` | Parquet conversion failed |
| -11 | `HEDL_ERR_LINT` | Linting failed |
| -12 | `HEDL_ERR_NEO4J` | Neo4j Cypher conversion failed |
| -13 | `HEDL_ERR_TOON` | TOON conversion failed |
| -14 | `HEDL_ERR_REENTRANT_CALL` | Reentrant call from callback |
| -15 | `HEDL_ERR_CANCELLED` | Async operation was cancelled |
| -16 | `HEDL_ERR_QUEUE_FULL` | Async work queue is full |
| -17 | `HEDL_ERR_INVALID_HANDLE` | Invalid async operation handle |

### Error Messages

Error messages are stored in thread-local storage, so each thread maintains independent error state. This means you can safely call HEDL functions from multiple threads without locking:

```c
const char* error = hedl_get_last_error();
hedl_clear_error_threadsafe();  // Clear after handling
```

## Memory Management

The library uses explicit ownership: every pointer returned by HEDL must be freed with the matching free function. There are four types of allocations:

| Returned by | Free with |
|-------------|-----------|
| `hedl_to_*` string functions | `hedl_free_string()` |
| `hedl_parse`, `hedl_from_*` | `hedl_free_document()` |
| `hedl_lint` | `hedl_free_diagnostics()` |
| `hedl_to_parquet` | `hedl_free_bytes(data, len)` |

Input strings (the HEDL/JSON/YAML you pass in) remain owned by the caller. The library does not hold references to them after the function returns.

### Use-After-Free Protection

The library uses poison pointer detection to catch common mistakes:

```c
hedl_free_document(doc);
int result = hedl_to_json(doc, 0, &json);  // Returns HEDL_ERR_NULL_PTR, not a crash
```

This is a safety net, not a substitute for correct code. Do not rely on it.

### UTF-8 Validation

All string inputs are validated for UTF-8 correctness before processing. Invalid sequences return `HEDL_ERR_INVALID_UTF8`. All outputs are guaranteed to be valid UTF-8.

## Logging

The library uses the `tracing` crate for structured logging. Set the `RUST_LOG` environment variable to enable:

```bash
export RUST_LOG=hedl_ffi::audit=debug
```

Logs include function entry/exit with timing, sanitized parameters (pointer addresses are masked), and success/failure outcomes. Useful for debugging integration issues or tracking performance in production.

## Language-Specific Examples

### C++

Wrap the C API in RAII for automatic cleanup:

```cpp
#include "hedl.h"
#include <stdexcept>
#include <string>

class HedlDoc {
    HedlDocument* doc_;
public:
    explicit HedlDoc(const char* input, int len = -1) : doc_(nullptr) {
        if (hedl_parse(input, len, 1, &doc_) != HEDL_OK) {
            throw std::runtime_error(hedl_get_last_error());
        }
    }
    ~HedlDoc() { if (doc_) hedl_free_document(doc_); }
    HedlDoc(const HedlDoc&) = delete;
    HedlDoc& operator=(const HedlDoc&) = delete;

    std::string to_json(int include_metadata = 0) {
        char* json = nullptr;
        if (hedl_to_json(doc_, include_metadata, &json) != HEDL_OK) {
            throw std::runtime_error(hedl_get_last_error());
        }
        std::string result(json);
        hedl_free_string(json);
        return result;
    }
};
```

### Python (ctypes)

```python
import ctypes

libhedl = ctypes.CDLL("libhedl.so")

libhedl.hedl_parse.argtypes = [ctypes.c_char_p, ctypes.c_int, ctypes.c_int, ctypes.POINTER(ctypes.c_void_p)]
libhedl.hedl_parse.restype = ctypes.c_int
libhedl.hedl_to_json.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.POINTER(ctypes.c_char_p)]
libhedl.hedl_to_json.restype = ctypes.c_int
libhedl.hedl_get_last_error.restype = ctypes.c_char_p
libhedl.hedl_free_document.argtypes = [ctypes.c_void_p]
libhedl.hedl_free_string.argtypes = [ctypes.c_char_p]

hedl_str = b"%V:2.0\n---\nname: Test"
doc = ctypes.c_void_p()

if libhedl.hedl_parse(hedl_str, -1, 1, ctypes.byref(doc)) != 0:
    print(f"Parse error: {libhedl.hedl_get_last_error().decode()}")
else:
    json_ptr = ctypes.c_char_p()
    if libhedl.hedl_to_json(doc, 0, ctypes.byref(json_ptr)) == 0:
        print(json_ptr.value.decode())
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
    if C.hedl_parse(cinput, -1, 1, &doc) != C.HEDL_OK {
        return "", fmt.Errorf("parse: %s", C.GoString(C.hedl_get_last_error()))
    }
    defer C.hedl_free_document(doc)

    var json *C.char
    if C.hedl_to_json(doc, 0, &json) != C.HEDL_OK {
        return "", fmt.Errorf("to_json: %s", C.GoString(C.hedl_get_last_error()))
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

  attach_function :hedl_parse, [:string, :int, :int, :pointer], :int
  attach_function :hedl_to_json, [:pointer, :int, :pointer], :int
  attach_function :hedl_get_last_error, [], :string
  attach_function :hedl_free_document, [:pointer], :void
  attach_function :hedl_free_string, [:pointer], :void

  def self.parse_to_json(input)
    doc = FFI::MemoryPointer.new(:pointer)
    raise "Parse error: #{hedl_get_last_error}" unless hedl_parse(input, -1, 1, doc) == 0

    doc_ptr = doc.read_pointer
    json_ptr = FFI::MemoryPointer.new(:pointer)
    begin
      raise "to_json error: #{hedl_get_last_error}" unless hedl_to_json(doc_ptr, 0, json_ptr) == 0
      json_ptr.read_pointer.read_string
    ensure
      hedl_free_string(json_ptr.read_pointer)
      hedl_free_document(doc_ptr)
    end
  end
end
```

## API Reference

The full API is documented in the generated `hedl.h` header. Key function groups:

**Parsing:** `hedl_parse`, `hedl_validate`

**Conversion to other formats:** `hedl_to_json`, `hedl_to_yaml`, `hedl_to_xml`, `hedl_to_csv`, `hedl_to_parquet`, `hedl_to_neo4j_cypher`, `hedl_to_toon`

**Conversion from other formats:** `hedl_from_json`, `hedl_from_yaml`, `hedl_from_xml`, `hedl_from_parquet`, `hedl_from_toon`

**Operations:** `hedl_canonicalize`, `hedl_lint`, `hedl_get_version`, `hedl_schema_count`, `hedl_alias_count`, `hedl_root_item_count`

**Diagnostics:** `hedl_diagnostics_count`, `hedl_diagnostics_get`, `hedl_diagnostics_severity`

**Memory:** `hedl_free_string`, `hedl_free_document`, `hedl_free_diagnostics`, `hedl_free_bytes`

**Errors:** `hedl_get_last_error`, `hedl_clear_error_threadsafe`

**Callbacks:** All `hedl_to_*` functions have `_callback` variants for streaming output

**Async:** All major operations have `_async` variants with `hedl_async_cancel` and `hedl_async_free` for lifecycle management

## Thread Safety

Functions are thread-safe. You can call any function from any thread without synchronization. Error messages are stored in thread-local storage, so concurrent errors do not interfere.

Documents themselves are not thread-safe. Do not share an `HedlDocument*` across threads without your own synchronization. In practice, parse on one thread and use on that same thread.

## Testing

```bash
cargo test -p hedl-ffi
```

## License

Apache-2.0
