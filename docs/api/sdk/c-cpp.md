# C/C++ SDK Documentation

Complete SDK documentation for using HEDL in C and C++ via FFI.

## Installation

### Download Pre-built Library

```bash
# Linux
wget https://github.com/dweve/hedl/releases/download/v0.1.0/libhedl.so

# macOS
wget https://github.com/dweve/hedl/releases/download/v0.1.0/libhedl.dylib

# Windows
curl -O https://github.com/dweve/hedl/releases/download/v0.1.0/hedl.dll
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

### Serialization

```c
int hedl_to_json(const HedlDocument* doc, int include_metadata, char** out);
int hedl_canonicalize(const HedlDocument* doc, char** out);
```

### Validation

```c
int hedl_lint(const HedlDocument* doc, HedlDiagnostics** out);
int hedl_diagnostics_count(const HedlDiagnostics* diag);
int hedl_diagnostics_get(const HedlDiagnostics* diag, int index, char** out_str);
int hedl_diagnostics_severity(const HedlDiagnostics* diag, int index);
```

### Memory Management

```c
void hedl_free_document(HedlDocument* doc);
void hedl_free_string(char* str);
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
#define HEDL_OK 0
#define HEDL_ERR_NULL_PTR -1
#define HEDL_ERR_INVALID_UTF8 -2
#define HEDL_ERR_PARSE -3
#define HEDL_ERR_CANONICALIZE -4
#define HEDL_ERR_JSON -5
#define HEDL_ERR_ALLOC -6
#define HEDL_ERR_YAML -7
#define HEDL_ERR_XML -8
#define HEDL_ERR_CSV -9
#define HEDL_ERR_PARQUET -10
#define HEDL_ERR_LINT -11
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
