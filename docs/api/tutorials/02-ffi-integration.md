# FFI Integration Tutorial

This tutorial demonstrates how to use HEDL from C and C++ via the Foreign Function Interface (FFI).

## Prerequisites

- C11 or C++17 compiler (GCC, Clang, MSVC)
- HEDL shared library (`.so` on Linux, `.dylib` on macOS, `.dll` on Windows)
- CMake or Make for building

## Installation

### Download Pre-built Library

Download the latest release from GitHub:

```bash
# Linux
wget https://github.com/dweve/hedl/releases/download/v0.1.0/libhedl.so

# macOS
wget https://github.com/dweve/hedl/releases/download/v0.1.0/libhedl.dylib

# Windows (PowerShell)
Invoke-WebRequest -Uri https://github.com/dweve/hedl/releases/download/v0.1.0/hedl.dll -OutFile hedl.dll
```

### Build from Source

```bash
git clone https://github.com/dweve/hedl.git
cd hedl
cargo build --release -p hedl-ffi

# Library will be in target/release/
ls target/release/libhedl.*
```

## Project Setup

### C Project Structure

```
my-project/
├── CMakeLists.txt
├── include/
│   └── hedl.h
├── lib/
│   └── libhedl.so
└── src/
    └── main.c
```

### CMakeLists.txt

```cmake
cmake_minimum_required(VERSION 3.10)
project(hedl_example C)

set(CMAKE_C_STANDARD 11)

# Include directories
include_directories(${CMAKE_SOURCE_DIR}/include)

# Link directories
link_directories(${CMAKE_SOURCE_DIR}/lib)

# Executable
add_executable(hedl_example src/main.c)

# Link HEDL library
target_link_libraries(hedl_example hedl)

# Set rpath for runtime library location
set_target_properties(hedl_example PROPERTIES
    BUILD_RPATH ${CMAKE_SOURCE_DIR}/lib
    INSTALL_RPATH ${CMAKE_SOURCE_DIR}/lib
)
```

## First C Example

### Basic Parsing

```c
#include <stdio.h>
#include <stdlib.h>
#include "hedl.h"

int main() {
    const char* hedl_input =
        "%VERSION: 1.0\n"
        "%STRUCT: User: [id, name, email]\n"
        "---\n"
        "users: @User\n"
        "  | alice, Alice Smith, alice@example.com\n"
        "  | bob, Bob Jones, bob@example.com\n";

    HedlDocument* doc = NULL;
    int code = hedl_parse(hedl_input, -1, 0, &doc);

    if (code != HEDL_OK) {
        const char* error = hedl_get_last_error();
        fprintf(stderr, "Parse error: %s\n", error);
        return 1;
    }

    printf("Parse successful!\n");

    // Clean up
    hedl_free_document(doc);
    return 0;
}
```

Build and run:

```bash
mkdir build && cd build
cmake ..
make
./hedl_example
```

### JSON Conversion

```c
#include <stdio.h>
#include <stdlib.h>
#include "hedl.h"

int main() {
    const char* hedl_input =
        "%VERSION: 1.0\n"
        "---\n"
        "name: Alice\n"
        "age: 30\n"
        "active: true\n";

    // Parse HEDL
    HedlDocument* doc = NULL;
    if (hedl_parse(hedl_input, -1, 0, &doc) != HEDL_OK) {
        fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
        return 1;
    }

    // Convert to JSON
    char* json = NULL;
    if (hedl_to_json(doc, 0, &json) != HEDL_OK) {
        fprintf(stderr, "JSON conversion error: %s\n", hedl_get_last_error());
        hedl_free_document(doc);
        return 1;
    }

    printf("JSON output:\n%s\n", json);

    // Clean up
    hedl_free_string(json);
    hedl_free_document(doc);
    return 0;
}
```

## Error Handling

### Comprehensive Error Handling

```c
#include <stdio.h>
#include <string.h>
#include "hedl.h"

int parse_hedl_file(const char* filename) {
    // Read file
    FILE* f = fopen(filename, "r");
    if (!f) {
        fprintf(stderr, "Failed to open file: %s\n", filename);
        return 1;
    }

    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);

    char* content = malloc(size + 1);
    if (!content) {
        fprintf(stderr, "Memory allocation failed\n");
        fclose(f);
        return 1;
    }

    fread(content, 1, size, f);
    content[size] = '\0';
    fclose(f);

    // Parse HEDL
    HedlDocument* doc = NULL;
    int code = hedl_parse(content, size, 0, &doc);

    if (code != HEDL_OK) {
        const char* error = hedl_get_last_error();

        switch (code) {
            case HEDL_ERR_PARSE:
                fprintf(stderr, "Parse error: %s\n", error);
                break;
            case HEDL_ERR_INVALID_UTF8:
                fprintf(stderr, "Invalid UTF-8: %s\n", error);
                break;
            case HEDL_ERR_NULL_PTR:
                fprintf(stderr, "Null pointer: %s\n", error);
                break;
            default:
                fprintf(stderr, "Error %d: %s\n", code, error);
                break;
        }

        free(content);
        return 1;
    }

    printf("Successfully parsed %ld bytes\n", size);

    // Clean up
    hedl_free_document(doc);
    free(content);
    return 0;
}
```

### Thread-Safe Error Handling

```c
#include <stdio.h>
#include <pthread.h>
#include "hedl.h"

void* worker_thread(void* arg) {
    const char* input = (const char*)arg;
    HedlDocument* doc = NULL;

    if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
        // Error messages are thread-local - safe to call from any thread
        const char* error = hedl_get_last_error();
        fprintf(stderr, "Thread error: %s\n", error);
        return NULL;
    }

    printf("Thread parsed successfully\n");
    hedl_free_document(doc);
    return (void*)1;
}

int main() {
    pthread_t threads[4];
    const char* inputs[4] = {
        "%VERSION: 1.0\n---\nkey1: value1",
        "%VERSION: 1.0\n---\nkey2: value2",
        "%VERSION: 1.0\n---\nkey3: value3",
        "%VERSION: 1.0\n---\nkey4: value4",
    };

    // Launch threads
    for (int i = 0; i < 4; i++) {
        pthread_create(&threads[i], NULL, worker_thread, (void*)inputs[i]);
    }

    // Wait for completion
    for (int i = 0; i < 4; i++) {
        pthread_join(threads[i], NULL);
    }

    return 0;
}
```

## Memory Management

### Proper Resource Cleanup

```c
#include <stdio.h>
#include "hedl.h"

void process_hedl(const char* input) {
    HedlDocument* doc = NULL;
    char* json = NULL;
    char* canonical = NULL;

    // Parse
    if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
        goto error;
    }

    // Convert to JSON
    if (hedl_to_json(doc, 0, &json) != HEDL_OK) {
        goto error;
    }

    // Canonicalize
    if (hedl_canonicalize(doc, &canonical) != HEDL_OK) {
        goto error;
    }

    printf("JSON: %s\n", json);
    printf("Canonical: %s\n", canonical);

error:
    // Clean up in reverse order of allocation
    if (canonical) hedl_free_string(canonical);
    if (json) hedl_free_string(json);
    if (doc) hedl_free_document(doc);
}
```

### RAII in C++ (Recommended)

```cpp
#include <iostream>
#include <memory>
#include "hedl.h"

// RAII wrapper for HedlDocument
struct HedlDocumentDeleter {
    void operator()(HedlDocument* doc) const {
        if (doc) hedl_free_document(doc);
    }
};

using HedlDocumentPtr = std::unique_ptr<HedlDocument, HedlDocumentDeleter>;

// RAII wrapper for strings
struct HedlStringDeleter {
    void operator()(char* str) const {
        if (str) hedl_free_string(str);
    }
};

using HedlStringPtr = std::unique_ptr<char, HedlStringDeleter>;

int main() {
    const char* input = "%VERSION: 1.0\n---\nkey: value";

    // Parse with automatic cleanup
    HedlDocument* raw_doc = nullptr;
    if (hedl_parse(input, -1, 0, &raw_doc) != HEDL_OK) {
        std::cerr << "Parse error: " << hedl_get_last_error() << std::endl;
        return 1;
    }
    HedlDocumentPtr doc(raw_doc);

    // Convert to JSON with automatic cleanup
    char* raw_json = nullptr;
    if (hedl_to_json(doc.get(), 0, &raw_json) != HEDL_OK) {
        std::cerr << "JSON error: " << hedl_get_last_error() << std::endl;
        return 1;
    }
    HedlStringPtr json(raw_json);

    std::cout << "JSON: " << json.get() << std::endl;

    // Automatic cleanup when leaving scope
    return 0;
}
```

## Linting and Validation

### Document Linting

```c
#include <stdio.h>
#include "hedl.h"

void lint_document(HedlDocument* doc) {
    HedlDiagnostics* diag = NULL;

    if (hedl_lint(doc, &diag) != HEDL_OK) {
        fprintf(stderr, "Lint error: %s\n", hedl_get_last_error());
        return;
    }

    int count = hedl_diagnostics_count(diag);
    printf("Found %d diagnostic messages:\n", count);

    for (int i = 0; i < count; i++) {
        int severity = hedl_diagnostics_severity(diag, i);
        char* message = NULL;
        hedl_diagnostics_get(diag, i, &message);

        printf("[%d] %s\n", severity, message);
        hedl_free_string(message);
    }

    hedl_free_diagnostics(diag);
}
```

## Complete Example: User Database

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "hedl.h"

typedef struct {
    char id[64];
    char name[128];
    char email[128];
    char role[32];
} User;

int load_users(const char* hedl_path, User** users, size_t* count) {
    // Read file
    FILE* f = fopen(hedl_path, "r");
    if (!f) return 1;

    fseek(f, 0, SEEK_END);
    long size = ftell(f);
    fseek(f, 0, SEEK_SET);

    char* content = malloc(size + 1);
    fread(content, 1, size, f);
    content[size] = '\0';
    fclose(f);

    // Parse HEDL
    HedlDocument* doc = NULL;
    if (hedl_parse(content, size, 0, &doc) != HEDL_OK) {
        fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
        free(content);
        return 1;
    }

    // Convert to JSON for easier processing
    char* json = NULL;
    if (hedl_to_json(doc, 0, &json) != HEDL_OK) {
        fprintf(stderr, "JSON error: %s\n", hedl_get_last_error());
        hedl_free_document(doc);
        free(content);
        return 1;
    }

    // Parse JSON and extract users
    // (In a real application, use a JSON library like cJSON)
    printf("JSON output:\n%s\n", json);

    // Clean up
    hedl_free_string(json);
    hedl_free_document(doc);
    free(content);
    return 0;
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <hedl_file>\n", argv[0]);
        return 1;
    }

    User* users = NULL;
    size_t count = 0;

    if (load_users(argv[1], &users, &count) != 0) {
        return 1;
    }

    printf("Loaded %zu users\n", count);

    if (users) {
        free(users);
    }

    return 0;
}
```

## Performance Tips

### Reusing Parsers

```c
// For batch processing
void process_batch(const char** inputs, size_t count) {
    for (size_t i = 0; i < count; i++) {
        HedlDocument* doc = NULL;

        if (hedl_parse(inputs[i], -1, 0, &doc) == HEDL_OK) {
            // Process document
            hedl_free_document(doc);
        }
    }
}
```

### Pre-allocating Buffers

```c
// Read multiple documents efficiently
void read_documents(const char** paths, size_t count) {
    char buffer[1024 * 1024];  // 1 MB buffer

    for (size_t i = 0; i < count; i++) {
        FILE* f = fopen(paths[i], "r");
        if (!f) continue;

        size_t size = fread(buffer, 1, sizeof(buffer) - 1, f);
        buffer[size] = '\0';
        fclose(f);

        HedlDocument* doc = NULL;
        hedl_parse(buffer, size, 0, &doc);
        // Process...
        hedl_free_document(doc);
    }
}
```

## Next Steps

- **[WASM Browser Integration](03-wasm-browser.md)** - Use HEDL in browsers
- **[Thread Safety Guide](../guides/thread-safety.md)** - Advanced thread safety
- **[Memory Management Guide](../guides/memory-management.md)** - Memory best practices
- **[FFI API Reference](../ffi-api.md)** - Complete FFI documentation

## Resources

- **[C/C++ SDK](../sdk/c-cpp.md)** - SDK documentation
- **[Examples](../examples.md)** - More code examples
- **[GitHub](https://github.com/dweve/hedl)** - Source code and issues
