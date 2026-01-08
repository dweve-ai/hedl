# FFI/C API Reference

**C-compatible bindings for integration with C, Python, Go, Ruby, and other languages**

---

## Quick Start

```c
#include <hedl_ffi.h>
#include <stdio.h>

int main() {
    const char* hedl = "%VERSION: 1.0\n---\nkey: value\0";
    HedlDocument* doc = NULL;

    // Parse document
    if (hedl_parse(hedl, -1, 0, &doc) != HEDL_OK) {
        fprintf(stderr, "Error: %s\n", hedl_get_last_error());
        return 1;
    }

    // Convert to JSON
    char* json = NULL;
    if (hedl_to_json(doc, 0, &json) != HEDL_OK) {
        fprintf(stderr, "Error: %s\n", hedl_get_last_error());
        hedl_free_document(doc);
        return 1;
    }

    printf("JSON: %s\n", json);

    // Cleanup
    hedl_free_string(json);
    hedl_free_document(doc);

    return 0;
}
```

**Compile**:
```bash
gcc -o example example.c -lhedl_ffi
```

---

## Memory Management

### CRITICAL SAFETY RULES

All memory allocated by HEDL functions **MUST** be freed with the corresponding `hedl_free_*` function:

| Allocated By | Free With |
|--------------|-----------|
| `hedl_parse()` | `hedl_free_document()` |
| `hedl_to_json()`, `hedl_canonicalize()`, etc. | `hedl_free_string()` |
| `hedl_to_parquet()` | `hedl_free_bytes()` |
| `hedl_lint()` | `hedl_free_diagnostics()` |

### UNDEFINED BEHAVIOR WARNING

**NEVER** pass the following to `hedl_free_*` functions:
- Pointers from `malloc`/`calloc`/`realloc` (wrong allocator)
- Stack-allocated variables
- Already-freed pointers (double free)
- Pointers from other libraries
- **NULL is safe** and will be ignored

---

## Thread Safety

### Error Handling

Error messages are stored in **thread-local storage (TLS)**, providing lock-free, wait-free error handling.

**Key Guarantees**:
- Each thread has independent error state
- Zero contention between threads
- **Must** call `hedl_get_last_error()` from the same thread that got the error

**Example (pthreads)**:
```c
void* worker(void* arg) {
    const char* input = (const char*)arg;
    HedlDocument* doc = NULL;

    if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
        // Get error for THIS thread
        const char* err = hedl_get_last_error_threadsafe();
        fprintf(stderr, "Thread error: %s\n", err);
        return NULL;
    }

    hedl_free_document(doc);
    return (void*)1;
}

int main() {
    pthread_t threads[8];
    const char* inputs[8] = { /* ... */ };

    for (int i = 0; i < 8; i++) {
        pthread_create(&threads[i], NULL, worker, (void*)inputs[i]);
    }

    for (int i = 0; i < 8; i++) {
        pthread_join(threads[i], NULL);
    }
}
```

### Document Handles

`HedlDocument*` pointers are **NOT thread-safe**. Each thread must create its own document handles.

**Safe Pattern**:
```c
// Thread A
HedlDocument* doc_a = NULL;
hedl_parse(input_a, -1, 0, &doc_a);

// Thread B
HedlDocument* doc_b = NULL;
hedl_parse(input_b, -1, 0, &doc_b);
```

**Unsafe Pattern** (undefined behavior):
```c
// Thread A and B both use doc - DATA RACE!
HedlDocument* doc = NULL;
hedl_parse(input, -1, 0, &doc);
```

---

## Error Codes

```c
#define HEDL_OK              0      // Success
#define HEDL_ERR_NULL_PTR   -1      // Null pointer argument
#define HEDL_ERR_INVALID_UTF8 -2    // Invalid UTF-8
#define HEDL_ERR_PARSE      -3      // Parse error
#define HEDL_ERR_CANONICALIZE -4    // Canonicalization error
#define HEDL_ERR_JSON       -5      // JSON conversion error
#define HEDL_ERR_ALLOC      -6      // Memory allocation failed
#define HEDL_ERR_YAML       -7      // YAML conversion error
#define HEDL_ERR_XML        -8      // XML conversion error
#define HEDL_ERR_CSV        -9      // CSV conversion error
#define HEDL_ERR_PARQUET    -10     // Parquet conversion error
#define HEDL_ERR_LINT       -11     // Linting error
#define HEDL_ERR_NEO4J      -12     // Neo4j conversion error
```

---

## Core Functions

### Parsing

#### `hedl_parse`

Parse a HEDL document.

```c
int hedl_parse(
    const char* input,
    int32_t input_len,
    int32_t strict,
    HedlDocument** out_doc
);
```

**Parameters**:
- `input`: Null-terminated HEDL string or byte array
- `input_len`: Length of input, or `-1` if null-terminated
- `strict`: `1` for strict mode, `0` for lenient (unresolved refs → null)
- `out_doc`: Output parameter for parsed document

**Returns**: `HEDL_OK` on success, error code on failure

**Example**:
```c
HedlDocument* doc = NULL;
int result = hedl_parse(hedl_input, -1, 1, &doc);
if (result != HEDL_OK) {
    fprintf(stderr, "Parse failed: %s\n", hedl_get_last_error());
}
```

---

#### `hedl_validate`

Validate HEDL input without fully parsing.

```c
int hedl_validate(
    const char* input,
    int32_t input_len,
    int32_t strict
);
```

**Returns**: `HEDL_OK` if valid, error code if invalid

**Example**:
```c
if (hedl_validate(input, -1, 0) == HEDL_OK) {
    printf("Valid HEDL\n");
}
```

---

### Document Inspection

#### `hedl_get_version`

Get HEDL format version.

```c
int hedl_get_version(
    const HedlDocument* doc,
    int32_t* major,
    int32_t* minor
);
```

**Example**:
```c
int major, minor;
hedl_get_version(doc, &major, &minor);
printf("Version: %d.%d\n", major, minor);
```

---

#### `hedl_schema_count`

Get number of schema (struct) definitions.

```c
int hedl_schema_count(const HedlDocument* doc);
```

**Returns**: Number of schemas, or `-1` if doc is NULL or invalid.

---

#### `hedl_alias_count`

Get number of aliases.

```c
int hedl_alias_count(const HedlDocument* doc);
```

**Returns**: Number of aliases, or `-1` if doc is NULL or invalid.

---

#### `hedl_root_item_count`

Get number of root items.

```c
int hedl_root_item_count(const HedlDocument* doc);
```

**Returns**: Number of root items, or `-1` if doc is NULL or invalid.

---

### Canonicalization

#### `hedl_canonicalize`

Convert document to canonical form.

```c
int hedl_canonicalize(
    const HedlDocument* doc,
    char** out_str
);
```

**Parameters**:
- `doc`: Document to canonicalize
- `out_str`: Output string (must be freed with `hedl_free_string`)

**Example**:
```c
char* canonical = NULL;
if (hedl_canonicalize(doc, &canonical) == HEDL_OK) {
    printf("%s\n", canonical);
    hedl_free_string(canonical);
}
```

---

#### `hedl_canonicalize_callback`

Zero-copy canonicalization via callback.

```c
typedef void (*HedlOutputCallback)(const char* chunk, size_t len, void* user_data);

int hedl_canonicalize_callback(
    const HedlDocument* doc,
    HedlOutputCallback callback,
    void* user_data
);
```

**Example**:
```c
void write_chunk(const char* chunk, size_t len, void* fp) {
    fwrite(chunk, 1, len, (FILE*)fp);
}

FILE* fp = fopen("output.hedl", "w");
hedl_canonicalize_callback(doc, write_chunk, fp);
fclose(fp);
```

---

### Format Conversion

#### `hedl_to_json`

Convert HEDL to JSON.

```c
int hedl_to_json(
    const HedlDocument* doc,
    int32_t include_metadata,
    char** out_str
);
```

**Parameters**:
- `doc`: Document to convert
- `include_metadata`: `1` to include HEDL metadata (`__type__`, `__schema__`), `0` for plain JSON
- `out_str`: Output JSON string

**Example**:
```c
char* json = NULL;
if (hedl_to_json(doc, 1, &json) == HEDL_OK) {
    printf("%s\n", json);
    hedl_free_string(json);
}
```

---

#### `hedl_to_json_callback`

Convert HEDL to JSON using callback (zero-copy).

```c
int hedl_to_json_callback(
    const HedlDocument* doc,
    int32_t include_metadata,
    HedlOutputCallback callback,
    void* user_data
);
```

**Parameters**:
- `doc`: Document to convert
- `include_metadata`: `1` to include HEDL metadata, `0` for plain JSON
- `callback`: Function to receive output chunks
- `user_data`: Context pointer passed to callback

---

#### `hedl_from_json`

Convert JSON to HEDL document.

```c
int hedl_from_json(
    const char* json,
    int32_t json_len,
    HedlDocument** out_doc
);
```

**Example**:
```c
const char* json = "{\"key\": \"value\"}";
HedlDocument* doc = NULL;
if (hedl_from_json(json, -1, &doc) == HEDL_OK) {
    // Use document
    hedl_free_document(doc);
}
```

---

#### `hedl_to_yaml`

Convert HEDL to YAML (requires `yaml` feature).

```c
int hedl_to_yaml(
    const HedlDocument* doc,
    int32_t include_metadata,
    char** out_str
);
```

**Parameters**:
- `doc`: Document to convert
- `include_metadata`: `1` to include HEDL metadata, `0` for plain YAML
- `out_str`: Output YAML string

---

#### `hedl_to_yaml_callback`

Convert HEDL to YAML using callback (zero-copy).

```c
int hedl_to_yaml_callback(
    const HedlDocument* doc,
    int32_t include_metadata,
    HedlOutputCallback callback,
    void* user_data
);
```

---

#### `hedl_to_xml`

Convert HEDL to XML (requires `xml` feature).

```c
int hedl_to_xml(
    const HedlDocument* doc,
    char** out_str
);
```

---

#### `hedl_to_xml_callback`

Convert HEDL to XML using callback (zero-copy).

```c
int hedl_to_xml_callback(
    const HedlDocument* doc,
    HedlOutputCallback callback,
    void* user_data
);
```

---

#### `hedl_to_csv`

Convert HEDL to CSV (requires `csv` feature).

```c
int hedl_to_csv(
    const HedlDocument* doc,
    char** out_str
);
```

---

#### `hedl_to_csv_callback`

Convert HEDL to CSV using callback (zero-copy).

```c
int hedl_to_csv_callback(
    const HedlDocument* doc,
    HedlOutputCallback callback,
    void* user_data
);
```

---

#### `hedl_to_parquet`

Convert HEDL to Parquet binary format (requires `parquet` feature).

```c
int hedl_to_parquet(
    const HedlDocument* doc,
    uint8_t** out_bytes,
    size_t* out_len
);
```

**Note**: Free with `hedl_free_bytes()`, not `hedl_free_string()`

**Example**:
```c
uint8_t* bytes = NULL;
size_t len = 0;
if (hedl_to_parquet(doc, &bytes, &len) == HEDL_OK) {
    fwrite(bytes, 1, len, output_file);
    hedl_free_bytes(bytes, len);
}
```

---

#### `hedl_from_parquet`

Convert Parquet bytes to HEDL document.

```c
int hedl_from_parquet(
    const uint8_t* bytes,
    size_t len,
    HedlDocument** out_doc
);
```

---

#### `hedl_to_neo4j_cypher`

Convert HEDL to Neo4j Cypher statements (requires `neo4j` feature).

```c
int hedl_to_neo4j_cypher(
    const HedlDocument* doc,
    int32_t use_merge,
    char** out_str
);
```

**Parameters**:
- `doc`: Document to convert
- `use_merge`: `1` to use MERGE (idempotent), `0` for CREATE
- `out_str`: Output Cypher string

---

#### `hedl_to_neo4j_cypher_callback`

Convert HEDL to Neo4j Cypher using callback (zero-copy).

```c
int hedl_to_neo4j_cypher_callback(
    const HedlDocument* doc,
    int32_t use_merge,
    HedlOutputCallback callback,
    void* user_data
);
```

---

### Linting

#### `hedl_lint`

Run linting checks on document.

```c
int hedl_lint(
    const HedlDocument* doc,
    HedlDiagnostics** out_diag
);
```

**Example**:
```c
HedlDiagnostics* diag = NULL;
if (hedl_lint(doc, &diag) == HEDL_OK) {
    int count = hedl_diagnostics_count(diag);
    for (int i = 0; i < count; i++) {
        char* msg = NULL;
        if (hedl_diagnostics_get(diag, i, &msg) == HEDL_OK) {
            printf("Diagnostic: %s\n", msg);
            hedl_free_string(msg);
        }
    }
    hedl_free_diagnostics(diag);
}
```

---

#### `hedl_diagnostics_count`

Get number of diagnostics.

```c
int hedl_diagnostics_count(const HedlDiagnostics* diag);
```

**Returns**: Number of diagnostics, or `-1` if diag is NULL or invalid.

---

#### `hedl_diagnostics_get`

Get diagnostic message by index.

```c
int hedl_diagnostics_get(
    const HedlDiagnostics* diag,
    int32_t index,
    char** out_str
);
```

**Parameters**:
- `diag`: Diagnostics handle
- `index`: Diagnostic index
- `out_str`: Pointer to store message (must be freed with `hedl_free_string`)

**Returns**: `HEDL_OK` on success, error code on failure.

---

#### `hedl_diagnostics_severity`

Get diagnostic severity.

```c
int hedl_diagnostics_severity(
    const HedlDiagnostics* diag,
    int32_t index
);
```

**Returns**: Severity level (`0`=hint, `1`=warning, `2`=error), or `-1` if invalid.

---

### Error Handling

#### `hedl_get_last_error`

Get last error message for current thread.

```c
const char* hedl_get_last_error(void);
```

**Returns**: Error message string, or `"No error"` if no error occurred

**Example**:
```c
if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
    fprintf(stderr, "Error: %s\n", hedl_get_last_error());
}
```

---

#### `hedl_get_last_error_threadsafe`

Thread-safe alias for `hedl_get_last_error`.

```c
const char* hedl_get_last_error_threadsafe(void);
```

---

#### `hedl_clear_error_threadsafe`

Clear error state for current thread.

```c
void hedl_clear_error_threadsafe(void);
```

---

### Memory Management

#### `hedl_free_document`

Free a parsed document.

```c
void hedl_free_document(HedlDocument* doc);
```

**Safety**: NULL-safe, can be called with NULL pointer.

---

#### `hedl_free_string`

Free a string allocated by HEDL.

```c
void hedl_free_string(char* str);
```

**Safety**: NULL-safe, can be called with NULL pointer.

---

#### `hedl_free_bytes`

Free a byte array allocated by HEDL.

```c
void hedl_free_bytes(uint8_t* bytes, size_t len);
```

---

#### `hedl_free_diagnostics`

Free diagnostics object.

```c
void hedl_free_diagnostics(HedlDiagnostics* diag);
```

---

## Opaque Types

```c
typedef struct HedlDocument HedlDocument;
typedef struct HedlDiagnostics HedlDiagnostics;
```

These types are **opaque** - you cannot access their fields directly. Use accessor functions.

---

## Security Features

### Poison Pointers

The FFI layer uses poison pointers to detect use-after-free:

1. After `hedl_free_document()`, the internal pointer is poisoned
2. Accessor functions check for poison before use
3. Attempting to use a freed pointer returns an error

**Note**: Since C passes pointers by value, we cannot modify the caller's pointer. The caller should set pointers to NULL after freeing:

```c
hedl_free_document(doc);
doc = NULL;  // Good practice
```

---

### Audit Logging

The FFI layer provides comprehensive audit logging via the `tracing` crate.

**Enable logging in your application**:
```c
// Set environment variable before running
export RUST_LOG=info
```

**Log Levels**:
- `ERROR`: Function failures
- `WARN`: Recoverable errors
- `INFO`: Function entry/exit
- `DEBUG`: Detailed parameters

---

## Language-Specific Examples

### Python (ctypes)

```python
import ctypes

# Load library
hedl = ctypes.CDLL('libhedl_ffi.so')

# Define types
hedl.hedl_parse.argtypes = [ctypes.c_char_p, ctypes.c_int32, ctypes.c_int32, ctypes.POINTER(ctypes.c_void_p)]
hedl.hedl_parse.restype = ctypes.c_int
hedl.hedl_free_document.argtypes = [ctypes.c_void_p]
hedl.hedl_get_last_error.restype = ctypes.c_char_p

# Parse HEDL
input_str = b"%VERSION: 1.0\n---\nkey: value"
doc = ctypes.c_void_p()
result = hedl.hedl_parse(input_str, -1, 0, ctypes.byref(doc))

if result != 0:
    error = hedl.hedl_get_last_error()
    print(f"Error: {error.decode()}")
else:
    print("Success!")
    hedl.hedl_free_document(doc)
```

---

### Go (cgo)

```go
package main

/*
#cgo LDFLAGS: -lhedl_ffi
#include <hedl_ffi.h>
#include <stdlib.h>
*/
import "C"
import (
    "fmt"
    "unsafe"
)

func main() {
    input := C.CString("%VERSION: 1.0\n---\nkey: value")
    defer C.free(unsafe.Pointer(input))

    var doc *C.HedlDocument
    result := C.hedl_parse(input, -1, 0, &doc)

    if result != C.HEDL_OK {
        errMsg := C.hedl_get_last_error()
        fmt.Printf("Error: %s\n", C.GoString(errMsg))
        return
    }

    defer C.hedl_free_document(doc)
    fmt.Println("Success!")
}
```

---

### Ruby (FFI gem)

```ruby
require 'ffi'

module HedlFFI
  extend FFI::Library
  ffi_lib 'hedl_ffi'

  attach_function :hedl_parse, [:string, :int32, :int32, :pointer], :int
  attach_function :hedl_free_document, [:pointer], :void
  attach_function :hedl_get_last_error, [], :string
end

doc_ptr = FFI::MemoryPointer.new(:pointer)
result = HedlFFI.hedl_parse("%VERSION: 1.0\n---\nkey: value", -1, 0, doc_ptr)

if result != 0
  puts "Error: #{HedlFFI.hedl_get_last_error}"
else
  puts "Success!"
  HedlFFI.hedl_free_document(doc_ptr.read_pointer)
end
```

---

## Best Practices

### 1. Always Check Return Codes

```c
if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
    // Handle error
}
```

### 2. Free All Allocated Memory

```c
char* json = NULL;
if (hedl_to_json(doc, 1, &json) == HEDL_OK) {
    // Use json
    hedl_free_string(json);  // MUST free
}
```

### 3. Use Callbacks for Large Output

```c
// Instead of allocating huge string
hedl_to_json_callback(doc, write_to_file, fp);
```

### 4. Clear Errors Between Operations

```c
hedl_clear_error_threadsafe();
```

---

**Next**: [WASM/JavaScript API Reference](wasm-api.md)
