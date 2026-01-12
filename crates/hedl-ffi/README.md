# hedl-ffi

**C ABI bindings for HEDL—use the HEDL ecosystem from C, C++, Python (ctypes/cffi), Ruby (FFI), Go (cgo), and any language with C interop.**

Many production systems use C/C++. Legacy code can't be rewritten in Rust overnight. Python, Ruby, Go need access to HEDL without rewriting the parser. Language bridges shouldn't sacrifice performance or correctness. Memory safety bugs in FFI code cause crashes and security vulnerabilities.

`hedl-ffi` provides production-grade C bindings to the complete HEDL implementation. 32 exported C functions covering parsing, validation, format conversion, canonicalization, linting, and statistics. Thread-safe error handling via thread-local storage. Memory safety through poison pointer detection (0xDEADBEEF/0xDEADC0DE) and explicit freeing functions. Zero-copy callback patterns for large outputs. Comprehensive audit logging for production debugging.

## What's Implemented

Complete C API with safety and observability:

1. **32 Exported Functions**: Parse, validate, convert, lint, canonicalize, stats, format
2. **13 Error Codes**: Comprehensive error classification (OK, Parse, Io, Schema, Reference, etc.)
3. **Thread-Safe Error Handling**: Thread-local storage for error messages
4. **Memory Safety**: 4 explicit freeing functions (string, document, diagnostics, bytes)
5. **Poison Pointer Detection**: 0xDEADBEEF (documents), 0xDEADC0DE (diagnostics) for use-after-free prevention
6. **UTF-8 Validation**: All string I/O checked for valid UTF-8
7. **Zero-Copy Callbacks**: Streaming output via callback functions (no full buffering)
8. **Audit Logging**: Performance metrics and operation logging for production debugging
9. **Format Support**: JSON, YAML, XML, CSV, Parquet, Neo4j, TOON (optional features)
10. **Auto-Generated Header**: `hedl.h` via cbindgen, maintained in sync with implementation

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

const char* hedl_str = "%VERSION: 1.0\n---\nname: Test\nvalue: 42\n";
HedlDocument* doc = hedl_parse(hedl_str, strlen(hedl_str));

if (doc == NULL) {
    const char* error = hedl_last_error();
    fprintf(stderr, "Parse error: %s\n", error);
    return 1;
}

// Use document...

hedl_free_document(doc);
```

### Validation

```c
HedlValidateOptions opts = {
    .strict = true,
    .lint = true,
    .max_size = 10 * 1024 * 1024  // 10 MB
};

HedlValidationResult* result = hedl_validate(hedl_str, strlen(hedl_str), &opts);

if (result->error_count > 0) {
    for (size_t i = 0; i < result->error_count; i++) {
        printf("Error at line %zu: %s\n",
            result->errors[i].line,
            result->errors[i].message);
    }
}

hedl_free_validation_result(result);
```

### Format Conversion

```c
// HEDL → JSON
char* json = hedl_to_json(doc, true, false);  // pretty, preserve_types
printf("%s\n", json);
hedl_free_string(json);

// JSON → HEDL
HedlDocument* doc2 = hedl_from_json(json_str, strlen(json_str));
hedl_free_document(doc2);

// Other conversions
char* yaml = hedl_to_yaml(doc);
char* xml = hedl_to_xml(doc, true);  // pretty
char* csv = hedl_to_csv(doc, ",");   // delimiter

hedl_free_string(yaml);
hedl_free_string(xml);
hedl_free_string(csv);
```

### Canonicalization

```c
HedlCanonicalizeOptions opts = {
    .use_ditto = true,
    .sort_keys = false,
    .inline_schemas = false,
    .indent_size = 2
};

char* canonical = hedl_canonicalize(doc, &opts);
printf("%s\n", canonical);
hedl_free_string(canonical);
```

### Linting

```c
HedlLintOptions opts = {
    .escalate_hints_to_warnings = false,
    .escalate_warnings_to_errors = false
};

HedlDiagnostics* diags = hedl_lint(doc, &opts);

for (size_t i = 0; i < diags->count; i++) {
    printf("[%s] Line %zu: %s\n",
        severity_to_string(diags->items[i].severity),
        diags->items[i].line,
        diags->items[i].message);
}

hedl_free_diagnostics(diags);
```

### Statistics

```c
HedlStats* stats = hedl_stats(doc);

printf("Entities: %zu\n", stats->entity_count);
printf("Fields: %zu\n", stats->field_count);
printf("Max depth: %zu\n", stats->max_depth);
printf("References: %zu\n", stats->reference_count);
printf("Lines: %zu\n", stats->line_count);
printf("Bytes: %zu\n", stats->byte_size);

// Entity types
for (size_t i = 0; i < stats->entity_type_count; i++) {
    printf("  Type: %s\n", stats->entity_types[i]);
}

hedl_free_stats(stats);
```

### Zero-Copy Output Callback

For large outputs, avoid full buffering:

```c
typedef void (*HedlOutputCallback)(const char* data, size_t len, void* user_data);

void my_callback(const char* data, size_t len, void* user_data) {
    FILE* fp = (FILE*)user_data;
    fwrite(data, 1, len, fp);
}

FILE* fp = fopen("output.json", "w");
hedl_to_json_stream(doc, my_callback, fp);
fclose(fp);
```

## Error Handling

### Error Codes

```c
typedef enum {
    HEDL_OK = 0,              // Success
    HEDL_ERR_PARSE = 1,       // Parse syntax error
    HEDL_ERR_IO = 2,          // I/O failure
    HEDL_ERR_UTF8 = 3,        // Invalid UTF-8
    HEDL_ERR_SCHEMA = 4,      // Schema mismatch
    HEDL_ERR_REFERENCE = 5,   // Unresolved reference
    HEDL_ERR_NULL_PTR = 6,    // NULL pointer argument
    HEDL_ERR_JSON = 7,        // JSON conversion error
    HEDL_ERR_YAML = 8,        // YAML conversion error
    HEDL_ERR_XML = 9,         // XML conversion error
    HEDL_ERR_CSV = 10,        // CSV conversion error
    HEDL_ERR_PARQUET = 11,    // Parquet conversion error
    HEDL_ERR_NEO4J = 12,      // Neo4j conversion error
} HedlErrorCode;
```

### Thread-Local Error Messages

```c
// Get last error message (thread-local)
const char* error = hedl_last_error();

// Get last error code (thread-local)
HedlErrorCode code = hedl_last_error_code();

// Clear error state
hedl_clear_error();
```

**Thread Safety**: Error messages stored in thread-local storage. Safe to call from multiple threads.

## Memory Management

### Freeing Functions

```c
// Free string returned by hedl_to_* functions
void hedl_free_string(char* s);

// Free document returned by hedl_parse or hedl_from_*
void hedl_free_document(HedlDocument* doc);

// Free diagnostics returned by hedl_lint
void hedl_free_diagnostics(HedlDiagnostics* diags);

// Free bytes returned by hedl_to_parquet or similar
void hedl_free_bytes(uint8_t* bytes);
```

**CRITICAL**: Always call appropriate freeing function for each allocated object. Memory leaks occur if not freed.

### Poison Pointer Detection

Automatic detection of use-after-free:

```c
HedlDocument* doc = hedl_parse(input, len);
hedl_free_document(doc);

// Using doc after free triggers poison pointer detection
char* json = hedl_to_json(doc, true, false);  // Error: HEDL_ERR_NULL_PTR
```

**Markers**:
- Documents: `0xDEADBEEF`
- Diagnostics: `0xDEADC0DE`

**Prevents**:
- Double-free bugs
- Use-after-free vulnerabilities
- Memory corruption

## UTF-8 Validation

All string inputs and outputs validated:

```c
// Input validation
const char* invalid_utf8 = "\xFF\xFE invalid";
HedlDocument* doc = hedl_parse(invalid_utf8, strlen(invalid_utf8));
// Returns NULL, hedl_last_error_code() == HEDL_ERR_UTF8

// Output validation
// All hedl_to_* functions guarantee valid UTF-8 output
```

**Guarantees**:
- No invalid UTF-8 propagated through API
- Error on malformed input
- Safe for UTF-8 expecting consumers

## Audit Logging

Performance metrics and operation logging:

```c
// Enable audit logging
hedl_enable_audit_log("hedl_audit.log");

// Operations logged with:
// - Operation name (e.g., "parse", "validate")
// - Duration (microseconds)
// - Input size (bytes)
// - Result (success/failure)

// Disable audit logging
hedl_disable_audit_log();
```

**Log Format**:
```
[2026-01-12 10:30:45.123] parse: 1523µs, 4096 bytes, success
[2026-01-12 10:30:45.126] to_json: 234µs, 4096 bytes, success
[2026-01-12 10:30:45.130] validate: 456µs, 4096 bytes, failure
```

**Use Cases**:
- Performance debugging
- Production monitoring
- Anomaly detection
- Capacity planning

## Language Bindings

### C

```c
#include "hedl.h"

HedlDocument* doc = hedl_parse(input, len);
char* json = hedl_to_json(doc, true, false);
hedl_free_string(json);
hedl_free_document(doc);
```

### C++

```cpp
#include "hedl.h"

class HedlDoc {
    HedlDocument* doc_;
public:
    HedlDoc(const char* input, size_t len) : doc_(hedl_parse(input, len)) {
        if (!doc_) throw std::runtime_error(hedl_last_error());
    }
    ~HedlDoc() { hedl_free_document(doc_); }

    std::string to_json(bool pretty = true) {
        char* json = hedl_to_json(doc_, pretty, false);
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

# Define function signatures
libhedl.hedl_parse.argtypes = [ctypes.c_char_p, ctypes.c_size_t]
libhedl.hedl_parse.restype = ctypes.c_void_p
libhedl.hedl_to_json.argtypes = [ctypes.c_void_p, ctypes.c_bool, ctypes.c_bool]
libhedl.hedl_to_json.restype = ctypes.c_char_p
libhedl.hedl_free_document.argtypes = [ctypes.c_void_p]
libhedl.hedl_free_string.argtypes = [ctypes.c_char_p]

# Use
hedl_str = b"%VERSION: 1.0\n---\nname: Test"
doc = libhedl.hedl_parse(hedl_str, len(hedl_str))
json_ptr = libhedl.hedl_to_json(doc, True, False)
json_str = ctypes.string_at(json_ptr).decode('utf-8')
libhedl.hedl_free_string(json_ptr)
libhedl.hedl_free_document(doc)
print(json_str)
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
import "unsafe"

func Parse(input string) (string, error) {
    cinput := C.CString(input)
    defer C.free(unsafe.Pointer(cinput))

    doc := C.hedl_parse(cinput, C.size_t(len(input)))
    if doc == nil {
        return "", fmt.Errorf("%s", C.GoString(C.hedl_last_error()))
    }
    defer C.hedl_free_document(doc)

    cjson := C.hedl_to_json(doc, C.bool(true), C.bool(false))
    defer C.hedl_free_string(cjson)

    return C.GoString(cjson), nil
}
```

### Ruby (FFI)

```ruby
require 'ffi'

module Hedl
  extend FFI::Library
  ffi_lib 'hedl'

  attach_function :hedl_parse, [:string, :size_t], :pointer
  attach_function :hedl_to_json, [:pointer, :bool, :bool], :string
  attach_function :hedl_free_document, [:pointer], :void
  attach_function :hedl_free_string, [:string], :void

  def self.parse(input)
    doc = hedl_parse(input, input.bytesize)
    return nil if doc.null?

    json = hedl_to_json(doc, true, false)
    hedl_free_document(doc)
    json
  ensure
    hedl_free_string(json) if json
  end
end
```

## Complete Function Reference

### Parsing & Validation

- `HedlDocument* hedl_parse(const char*, size_t)`
- `HedlValidationResult* hedl_validate(const char*, size_t, HedlValidateOptions*)`
- `bool hedl_is_valid(const char*, size_t)`

### Format Conversion (HEDL → Other)

- `char* hedl_to_json(HedlDocument*, bool pretty, bool preserve_types)`
- `char* hedl_to_yaml(HedlDocument*)`
- `char* hedl_to_xml(HedlDocument*, bool pretty)`
- `char* hedl_to_csv(HedlDocument*, const char* delimiter)`
- `uint8_t* hedl_to_parquet(HedlDocument*, size_t* out_len)`
- `char* hedl_to_cypher(HedlDocument*)`
- `char* hedl_to_toon(HedlDocument*)`

### Format Conversion (Other → HEDL)

- `HedlDocument* hedl_from_json(const char*, size_t)`
- `HedlDocument* hedl_from_yaml(const char*, size_t)`
- `HedlDocument* hedl_from_xml(const char*, size_t)`
- `HedlDocument* hedl_from_csv(const char*, size_t, const char* type_name)`
- `HedlDocument* hedl_from_parquet(const uint8_t*, size_t)`
- `HedlDocument* hedl_from_toon(const char*, size_t)`

### Operations

- `char* hedl_canonicalize(HedlDocument*, HedlCanonicalizeOptions*)`
- `char* hedl_format(HedlDocument*)`
- `HedlDiagnostics* hedl_lint(HedlDocument*, HedlLintOptions*)`
- `HedlStats* hedl_stats(HedlDocument*)`

### Memory Management

- `void hedl_free_string(char*)`
- `void hedl_free_document(HedlDocument*)`
- `void hedl_free_diagnostics(HedlDiagnostics*)`
- `void hedl_free_bytes(uint8_t*)`

### Error Handling

- `const char* hedl_last_error(void)`
- `HedlErrorCode hedl_last_error_code(void)`
- `void hedl_clear_error(void)`

### Audit Logging

- `void hedl_enable_audit_log(const char* path)`
- `void hedl_disable_audit_log(void)`

## Performance Characteristics

**Overhead**: C FFI adds <1% overhead vs native Rust. Essentially zero-cost abstraction.

**Memory**: Same as Rust implementation. No additional allocations beyond necessary conversions.

**Thread Safety**: All functions thread-safe. Documents can be shared across threads with synchronization.

**Callback Performance**: Zero-copy streaming avoids large allocations. Suitable for multi-GB outputs.

## Dependencies

- `hedl` 1.0 - Main facade crate (all formats)
- `libc` 0.2 - C standard library bindings

## License

Apache-2.0
