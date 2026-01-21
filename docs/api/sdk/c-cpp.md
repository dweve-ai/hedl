# C/C++ SDK Documentation

Complete SDK documentation for using HEDL in C and C++ via FFI.

## Installation

### Download Pre-built Library

```bash
# Linux
wget https://github.com/dweve/hedl/releases/download/v1.2.0/libhedl.so

# macOS
wget https://github.com/dweve/hedl/releases/download/v1.2.0/libhedl.dylib

# Windows
curl -O https://github.com/dweve/hedl/releases/download/v1.2.0/hedl.dll
```

### Build from Source

```bash
git clone https://github.com/dweve/hedl.git
cd hedl
cargo build --release -p hedl-ffi

# Library in target/release/
# Header in crates/hedl-ffi/include/hedl.h
```

## Quick Start (C)

```c
#include <stdio.h>
#include "hedl.h"

int main() {
    const char* input =
        "%VERSION: 1.0\n"
        "---\n"
        "name: Alice\n"
        "age: 30\n";

    HedlDocument* doc = NULL;
    if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
        fprintf(stderr, "Error: %s\n", hedl_get_last_error());
        return 1;
    }

    char* json = NULL;
    if (hedl_to_json(doc, 0, &json) != HEDL_OK) {
        fprintf(stderr, "Error: %s\n", hedl_get_last_error());
        hedl_free_document(doc);
        return 1;
    }

    printf("JSON: %s\n", json);

    hedl_free_string(json);
    hedl_free_document(doc);
    return 0;
}
```

## Quick Start (C++)

```cpp
#include <iostream>
#include <memory>
#include "hedl.h"

// RAII wrappers
struct HedlDocDeleter {
    void operator()(HedlDocument* p) const {
        if (p) hedl_free_document(p);
    }
};

struct HedlStringDeleter {
    void operator()(char* p) const {
        if (p) hedl_free_string(p);
    }
};

using HedlDocPtr = std::unique_ptr<HedlDocument, HedlDocDeleter>;
using HedlStringPtr = std::unique_ptr<char, HedlStringDeleter>;

int main() {
    const char* input = "%VERSION: 1.0\n---\nname: Alice\n";

    HedlDocument* raw_doc = nullptr;
    if (hedl_parse(input, -1, 0, &raw_doc) != HEDL_OK) {
        std::cerr << "Error: " << hedl_get_last_error() << std::endl;
        return 1;
    }
    HedlDocPtr doc(raw_doc);

    char* raw_json = nullptr;
    if (hedl_to_json(doc.get(), 0, &raw_json) != HEDL_OK) {
        std::cerr << "Error: " << hedl_get_last_error() << std::endl;
        return 1;
    }
    HedlStringPtr json(raw_json);

    std::cout << "JSON: " << json.get() << std::endl;
    return 0;
}
```

## API Functions

### Parsing

```c
int hedl_parse(
    const char* input,
    int length,          // -1 for null-terminated
    int strict,          // 1 for strict, 0 for lenient
    HedlDocument** out
);
```

### Conversion FROM Formats

```c
int hedl_from_json(const char* json, int json_len, HedlDocument** out_doc);
int hedl_from_yaml(const char* yaml, int yaml_len, HedlDocument** out_doc);
int hedl_from_xml(const char* xml, int xml_len, HedlDocument** out_doc);
int hedl_from_parquet(const uint8_t* data, uintptr_t len, HedlDocument** out_doc);
int hedl_from_toon(const char* toon, int toon_len, HedlDocument** out_doc);
```

### Conversion TO Formats

```c
int hedl_to_json(const HedlDocument* doc, int include_metadata, char** out);
int hedl_to_yaml(const HedlDocument* doc, int include_metadata, char** out);
int hedl_to_xml(const HedlDocument* doc, char** out);
int hedl_to_csv(const HedlDocument* doc, char** out);
int hedl_to_parquet(const HedlDocument* doc, uint8_t** out_data, uintptr_t* out_len);
int hedl_to_neo4j_cypher(const HedlDocument* doc, int use_merge, char** out);
int hedl_to_toon(const HedlDocument* doc, char** out);
int hedl_canonicalize(const HedlDocument* doc, char** out);
```

**Note:** For `hedl_to_parquet`, the output bytes must be freed with `hedl_free_bytes(data, len)` instead of `hedl_free_string`.

### Validation

```c
int hedl_validate(const char* input, int input_len, int strict);
int hedl_lint(const HedlDocument* doc, HedlDiagnostics** out);
int hedl_diagnostics_count(const HedlDiagnostics* diag);
int hedl_diagnostics_get(const HedlDiagnostics* diag, int index, char** out_str);
int hedl_diagnostics_severity(const HedlDiagnostics* diag, int index);
```

**Note:** `hedl_validate` validates without creating a document handle (faster for validation-only use cases).

### Document Introspection

```c
int hedl_get_version(const HedlDocument* doc, int* major, int* minor);
int hedl_schema_count(const HedlDocument* doc);
int hedl_alias_count(const HedlDocument* doc);
int hedl_root_item_count(const HedlDocument* doc);
```

**Returns:** Count value, or -1 if document is NULL or invalid.

### Memory Management

```c
void hedl_free_document(HedlDocument* doc);
void hedl_free_string(char* str);
void hedl_free_bytes(uint8_t* data, size_t len);
void hedl_free_diagnostics(HedlDiagnostics* diag);
```

### Error Handling

```c
const char* hedl_get_last_error();
const char* hedl_get_last_error_threadsafe();
void hedl_clear_error_threadsafe();
```

## Error Codes

```c
#define HEDL_OK                  0   // Success
#define HEDL_ERR_NULL_PTR       -1   // NULL pointer argument
#define HEDL_ERR_INVALID_UTF8   -2   // Invalid UTF-8 input
#define HEDL_ERR_PARSE          -3   // Parse error
#define HEDL_ERR_CANONICALIZE   -4   // Canonicalization error
#define HEDL_ERR_JSON           -5   // JSON conversion error
#define HEDL_ERR_ALLOC          -6   // Memory allocation error
#define HEDL_ERR_YAML           -7   // YAML conversion error
#define HEDL_ERR_XML            -8   // XML conversion error
#define HEDL_ERR_CSV            -9   // CSV conversion error
#define HEDL_ERR_PARQUET       -10   // Parquet conversion error
#define HEDL_ERR_LINT          -11   // Linting error
#define HEDL_ERR_NEO4J         -12   // Neo4j Cypher conversion error
#define HEDL_ERR_TOON          -13   // TOON conversion error
#define HEDL_ERR_REENTRANT_CALL -13  // Reentrant FFI call detected
#define HEDL_ERR_CANCELLED     -15   // Async operation cancelled
#define HEDL_ERR_QUEUE_FULL    -16   // Async operation queue full
#define HEDL_ERR_INVALID_HANDLE -17  // Invalid async operation handle
```

## Memory Management Rules

### Critical Rules

1. **Always free** with matching `hedl_free_*` functions
2. **Never** use `free()` or `delete` on HEDL allocations
3. **Never** double-free
4. **Never** use after free
5. **NULL is safe** to free (no-op)

### Example Patterns

```c
// Correct
HedlDocument* doc = NULL;
hedl_parse(input, -1, 0, &doc);
hedl_free_document(doc);

// Wrong - double free
hedl_free_document(doc);
hedl_free_document(doc);  // CRASH!

// Wrong - wrong allocator
char* json = NULL;
hedl_to_json(doc, 0, &json);  // 0 = no metadata
free(json);  // CRASH! Must use hedl_free_string

// Safe - NULL is okay
hedl_free_document(NULL);  // No-op
```

## C++ RAII Wrapper

```cpp
class HedlDocument {
public:
    HedlDocument(const std::string& input) {
        ::HedlDocument* raw = nullptr;
        int code = hedl_parse(input.c_str(), -1, 1, &raw);
        if (code != HEDL_OK) {
            throw std::runtime_error(hedl_get_last_error());
        }
        doc_.reset(raw);
    }

    std::string to_json(bool include_metadata = false) const {
        char* raw = nullptr;
        int code = hedl_to_json(doc_.get(), include_metadata ? 1 : 0, &raw);
        if (code != HEDL_OK) {
            throw std::runtime_error(hedl_get_last_error());
        }
        std::unique_ptr<char, decltype(&hedl_free_string)> json(raw, hedl_free_string);
        return std::string(json.get());
    }

private:
    std::unique_ptr<::HedlDocument, decltype(&hedl_free_document)> doc_{
        nullptr, hedl_free_document
    };
};
```

## Thread Safety

### Thread-Local Errors

```c
void* worker(void* arg) {
    HedlDocument* doc = NULL;
    if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
        // Thread-safe error access
        const char* err = hedl_get_last_error_threadsafe();
        fprintf(stderr, "Error: %s\n", err);
        return NULL;
    }
    hedl_free_document(doc);
    return (void*)1;
}
```

### Document Thread Safety

Documents are **NOT thread-safe**. Don't share across threads without external synchronization.

## Zero-Copy Callback API

For large outputs (>1MB), zero-copy callback functions avoid memory allocation by passing data directly to a callback. This is more efficient than allocating and copying large strings.

### Callback Type

```c
typedef void (*HedlOutputCallback)(const char* data, uintptr_t len, void* user_data);
```

**Safety requirements:**
- The `data` pointer is only valid during the callback execution
- Do NOT store the pointer for later use
- The data is not null-terminated
- The callback MUST NOT call back into HEDL functions

### Available Functions

```c
int hedl_to_json_callback(const HedlDocument* doc, int include_metadata,
                          HedlOutputCallback callback, void* user_data);
int hedl_to_yaml_callback(const HedlDocument* doc, int include_metadata,
                          HedlOutputCallback callback, void* user_data);
int hedl_to_xml_callback(const HedlDocument* doc,
                         HedlOutputCallback callback, void* user_data);
int hedl_to_csv_callback(const HedlDocument* doc,
                         HedlOutputCallback callback, void* user_data);
int hedl_to_neo4j_cypher_callback(const HedlDocument* doc, int use_merge,
                                  HedlOutputCallback callback, void* user_data);
int hedl_to_toon_callback(const HedlDocument* doc,
                          HedlOutputCallback callback, void* user_data);
int hedl_canonicalize_callback(const HedlDocument* doc,
                               HedlOutputCallback callback, void* user_data);
```

### Example (C)

```c
#include <stdio.h>
#include <string.h>
#include "hedl.h"

// Write data to file in chunks without allocating memory
void write_to_file_callback(const char* data, size_t len, void* user_data) {
    FILE* fp = (FILE*)user_data;
    fwrite(data, 1, len, fp);
}

int main() {
    HedlDocument* doc = NULL;
    hedl_parse(input, -1, 0, &doc);

    FILE* fp = fopen("output.json", "w");
    if (!fp) {
        fprintf(stderr, "Failed to open file\n");
        hedl_free_document(doc);
        return 1;
    }

    // Convert to JSON and stream directly to file
    int result = hedl_to_json_callback(doc, 0, write_to_file_callback, fp);

    fclose(fp);
    hedl_free_document(doc);

    if (result != HEDL_OK) {
        fprintf(stderr, "Error: %s\n", hedl_get_last_error());
        return 1;
    }

    return 0;
}
```

### Example (C++)

```cpp
#include <iostream>
#include <sstream>
#include "hedl.h"

int main() {
    HedlDocument* doc = nullptr;
    hedl_parse(input, -1, 0, &doc);

    std::ostringstream oss;

    // Lambda callback to append to stringstream
    auto callback = [](const char* data, size_t len, void* user_data) {
        auto* stream = static_cast<std::ostringstream*>(user_data);
        stream->write(data, len);
    };

    int result = hedl_to_json_callback(doc, 0, callback, &oss);
    hedl_free_document(doc);

    if (result == HEDL_OK) {
        std::cout << oss.str() << std::endl;
    } else {
        std::cerr << "Error: " << hedl_get_last_error() << std::endl;
        return 1;
    }

    return 0;
}
```

## Async API

The async API allows non-blocking operations with completion callbacks. All async functions execute on a worker thread pool and invoke callbacks on completion.

### Types

```c
// Opaque handle to an async operation
typedef struct HedlAsyncOp {
    uint64_t id;
    Arc_AtomicBool cancelled;
    Arc_AtomicBool completed;
} HedlAsyncOp;

// Completion callback function type
typedef struct Option_HedlCompletionCallbackFn HedlCompletionCallback;
```

### Parsing

```c
HedlAsyncOp* hedl_parse_async(
    const char* input,
    int input_len,        // -1 for null-terminated
    int strict,           // 1 for strict, 0 for lenient
    HedlCompletionCallback callback,
    void* user_data
);
```

**Callback signature:**
```c
void callback(int status, HedlDocument* doc, const char* error, void* user_data);
```

- On success: `status=HEDL_OK`, `doc!=NULL`, `error=NULL`
- On error: `status=error_code`, `doc=NULL`, `error=error_message`
- Callback receives ownership of document (must call `hedl_free_document()`)
- Callback executes on worker thread (must be thread-safe)

### Serialization

```c
HedlAsyncOp* hedl_to_json_async(const HedlDocument* doc, int include_metadata,
                                HedlCompletionCallback callback, void* user_data);
HedlAsyncOp* hedl_to_yaml_async(const HedlDocument* doc, int include_metadata,
                                HedlCompletionCallback callback, void* user_data);
HedlAsyncOp* hedl_to_xml_async(const HedlDocument* doc,
                               HedlCompletionCallback callback, void* user_data);
HedlAsyncOp* hedl_to_csv_async(const HedlDocument* doc,
                               HedlCompletionCallback callback, void* user_data);
HedlAsyncOp* hedl_to_neo4j_cypher_async(const HedlDocument* doc, int use_merge,
                                        HedlCompletionCallback callback, void* user_data);
HedlAsyncOp* hedl_to_toon_async(const HedlDocument* doc,
                                HedlCompletionCallback callback, void* user_data);
```

**Callback signature:**
```c
void callback(int status, char* result, const char* error, void* user_data);
```

- Callback receives ownership of result string (must call `hedl_free_string()`)

### Validation

```c
HedlAsyncOp* hedl_canonicalize_async(const HedlDocument* doc,
                                     HedlCompletionCallback callback, void* user_data);
HedlAsyncOp* hedl_lint_async(const HedlDocument* doc,
                             HedlCompletionCallback callback, void* user_data);
```

**Lint callback signature:**
```c
void callback(int status, HedlDiagnostics* diag, const char* error, void* user_data);
```

- Callback receives ownership of diagnostics (must call `hedl_free_diagnostics()`)

### Operation Management

```c
void hedl_async_cancel(HedlAsyncOp* op);
void hedl_async_free(HedlAsyncOp* op);
```

**Cancellation behavior:**
- If not started: Cancelled immediately, callback invoked with `HEDL_ERR_CANCELLED`
- If in progress: Attempts to abort, callback invoked with `HEDL_ERR_CANCELLED`
- If completed: No effect (callback already executed)

**Memory management:**
- Always call `hedl_async_free()` after operation completes
- Safe to call regardless of operation state (pending/completed/cancelled)

### Example (C)

```c
#include <stdio.h>
#include <pthread.h>
#include "hedl.h"

typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t cond;
    int done;
    char* result;
} AsyncContext;

void parse_callback(int status, HedlDocument* doc, const char* error, void* user_data) {
    AsyncContext* ctx = (AsyncContext*)user_data;

    pthread_mutex_lock(&ctx->mutex);

    if (status == HEDL_OK) {
        // Convert to JSON
        hedl_to_json(doc, 0, &ctx->result);
        hedl_free_document(doc);
    } else {
        fprintf(stderr, "Parse error: %s\n", error);
    }

    ctx->done = 1;
    pthread_cond_signal(&ctx->cond);
    pthread_mutex_unlock(&ctx->mutex);
}

int main() {
    const char* input = "%VERSION: 1.0\n---\nname: Alice\n";

    AsyncContext ctx = {
        .mutex = PTHREAD_MUTEX_INITIALIZER,
        .cond = PTHREAD_COND_INITIALIZER,
        .done = 0,
        .result = NULL
    };

    // Start async parse
    HedlAsyncOp* op = hedl_parse_async(input, -1, 0, parse_callback, &ctx);
    if (!op) {
        fprintf(stderr, "Failed to submit async operation\n");
        return 1;
    }

    // Wait for completion
    pthread_mutex_lock(&ctx.mutex);
    while (!ctx.done) {
        pthread_cond_wait(&ctx.cond, &ctx.mutex);
    }
    pthread_mutex_unlock(&ctx.mutex);

    if (ctx.result) {
        printf("Result: %s\n", ctx.result);
        hedl_free_string(ctx.result);
    }

    hedl_async_free(op);
    return 0;
}
```

### Example (C++)

```cpp
#include <iostream>
#include <future>
#include <memory>
#include "hedl.h"

class AsyncParse {
public:
    std::future<std::string> parse(const std::string& input) {
        auto promise = std::make_shared<std::promise<std::string>>();
        auto future = promise->get_future();

        auto callback = [](int status, HedlDocument* doc,
                          const char* error, void* user_data) {
            auto* p = static_cast<std::promise<std::string>*>(user_data);

            if (status == HEDL_OK) {
                char* json = nullptr;
                if (hedl_to_json(doc, 0, &json) == HEDL_OK) {
                    p->set_value(std::string(json));
                    hedl_free_string(json);
                } else {
                    p->set_exception(std::make_exception_ptr(
                        std::runtime_error(hedl_get_last_error())));
                }
                hedl_free_document(doc);
            } else {
                p->set_exception(std::make_exception_ptr(
                    std::runtime_error(error)));
            }
        };

        HedlAsyncOp* op = hedl_parse_async(input.c_str(), -1, 0,
                                          callback, promise.get());
        if (!op) {
            throw std::runtime_error("Failed to submit async operation");
        }

        // Store op handle for potential cancellation
        // Must call hedl_async_free(op) after future completes

        return future;
    }
};

int main() {
    AsyncParse parser;
    auto future = parser.parse("%VERSION: 1.0\n---\nname: Alice\n");

    try {
        std::string json = future.get();
        std::cout << "Result: " << json << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << std::endl;
        return 1;
    }

    return 0;
}
```

## CMake Integration

```cmake
cmake_minimum_required(VERSION 3.10)
project(myproject C)

include_directories(${CMAKE_SOURCE_DIR}/include)
link_directories(${CMAKE_SOURCE_DIR}/lib)

add_executable(myapp src/main.c)
target_link_libraries(myapp hedl)

# Set rpath
set_target_properties(myapp PROPERTIES
    BUILD_RPATH ${CMAKE_SOURCE_DIR}/lib
)
```

## Examples

See [FFI Integration Tutorial](../tutorials/02-ffi-integration.md) for complete examples.

## Platform Support

- Linux (glibc 2.17+)
- macOS (10.12+)
- Windows (Windows 7+)
- iOS
- Android

## See Also

- [FFI API Reference](../ffi-api.md)
- [FFI Integration Tutorial](../tutorials/02-ffi-integration.md)
- [Thread Safety Guide](../guides/thread-safety.md)
- [Memory Management Guide](../guides/memory-management.md)
