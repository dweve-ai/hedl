# HEDL C/C++ Bindings

Production-ready CMake integration for HEDL (Hierarchical Entity Data Language) C/C++ FFI bindings.

## Overview

This directory provides a modern CMake build system for integrating HEDL into C and C++ projects. The bindings enable:

- **Parsing** HEDL documents
- **Format conversion** to/from JSON, YAML, XML, CSV, Parquet
- **Validation** and linting
- **Canonicalization** for normalization
- **Thread-safe** operations with comprehensive error handling

## Quick Start

### Prerequisites

- CMake 3.15 or later
- Rust toolchain (cargo, rustc)
- C11 or C++17 compatible compiler

### Build and Install

```bash
# Clone the repository
git clone https://github.com/dweve-ai/hedl.git
cd hedl/bindings/c

# Create build directory
mkdir build && cd build

# Configure (default: Release build with all features)
cmake ..

# Build
cmake --build .

# Install (optional, requires sudo on Unix)
sudo cmake --install . --prefix /usr/local
```

### Build Options

```bash
# Build only specific features
cmake .. \
  -DHEDL_FEATURE_JSON=ON \
  -DHEDL_FEATURE_YAML=ON \
  -DHEDL_FEATURE_XML=OFF \
  -DHEDL_FEATURE_CSV=OFF

# Build static library instead of shared
cmake .. -DHEDL_BUILD_STATIC=ON -DHEDL_BUILD_SHARED=OFF

# Debug build
cmake .. -DCMAKE_BUILD_TYPE=Debug

# Build without examples
cmake .. -DHEDL_BUILD_EXAMPLES=OFF

# Custom install location
cmake .. -DCMAKE_INSTALL_PREFIX=$HOME/.local
```

## Integration with Your Project

### Method 1: find_package() (Recommended)

After installing HEDL, use CMake's `find_package()`:

```cmake
# CMakeLists.txt
cmake_minimum_required(VERSION 3.15)
project(MyProject)

# Find HEDL
find_package(HEDL REQUIRED)

# Create your executable
add_executable(myapp main.c)

# Link HEDL library
target_link_libraries(myapp PRIVATE HEDL::hedl)

# For static linking:
# target_link_libraries(myapp PRIVATE HEDL::hedl_static)
```

Build your project:

```bash
mkdir build && cd build
cmake .. -DCMAKE_PREFIX_PATH=/usr/local
cmake --build .
```

### Method 2: add_subdirectory()

Include HEDL directly in your project:

```cmake
# Clone or add hedl as git submodule
add_subdirectory(external/hedl/bindings/c)

add_executable(myapp main.c)
target_link_libraries(myapp PRIVATE HEDL::hedl)
```

### Method 3: Manual Configuration

```bash
# Compile
gcc -o myapp main.c \
  -I/path/to/hedl/bindings/c/include \
  -L/path/to/hedl/target/release \
  -lhedl_ffi

# Run (ensure library is in path)
LD_LIBRARY_PATH=/path/to/hedl/target/release ./myapp
```

## Example Usage

### Basic Parsing

```c
#include <stdio.h>
#include "hedl.h"

int main(void) {
    const char* hedl_src =
        "%VERSION: 1.0\n"
        "---\n"
        "name: Alice\n"
        "age: 30\n";

    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl_src, -1, 1, &doc);

    if (result != HEDL_OK) {
        fprintf(stderr, "Error: %s\n", hedl_get_last_error());
        return 1;
    }

    // Use document...
    char* json = NULL;
    hedl_to_json(doc, 1, &json);
    printf("%s\n", json);

    // Cleanup
    hedl_free_string(json);
    hedl_free_document(doc);
    return 0;
}
```

### Format Conversion

```c
HedlDocument* doc = /* ... parse document ... */;

// Convert to JSON (pretty-printed)
char* json = NULL;
hedl_to_json(doc, 1, &json);
printf("JSON: %s\n", json);
hedl_free_string(json);

// Convert to YAML
char* yaml = NULL;
hedl_to_yaml(doc, &yaml);
printf("YAML: %s\n", yaml);
hedl_free_string(yaml);

// Convert to XML
char* xml = NULL;
hedl_to_xml(doc, &xml);
printf("XML: %s\n", xml);
hedl_free_string(xml);
```

### Error Handling

```c
HedlDocument* doc = NULL;
int result = hedl_parse(input, -1, 1, &doc);

if (result != HEDL_OK) {
    const char* error = hedl_get_last_error();
    fprintf(stderr, "Parse error: %s\n", error ? error : "unknown");
    hedl_free_string((char*)error);
    return 1;
}

// Validate document
if (hedl_validate(doc) != HEDL_OK) {
    const char* error = hedl_get_last_error();
    fprintf(stderr, "Validation error: %s\n", error ? error : "unknown");
    hedl_free_string((char*)error);
}

hedl_free_document(doc);
```

### Diagnostics and Linting

```c
HedlDiagnostics* diags = NULL;
int result = hedl_lint(doc, &diags);

if (result == HEDL_OK && diags) {
    int count = hedl_diagnostics_count(diags);

    for (int i = 0; i < count; i++) {
        char* message = NULL;
        int severity = 0;

        hedl_diagnostics_get(diags, i, &message);
        hedl_diagnostics_severity(diags, i, &severity);

        printf("[%s] %s\n",
               severity == 0 ? "ERROR" : "WARNING",
               message);

        hedl_free_string(message);
    }

    hedl_free_diagnostics(diags);
}
```

## Examples

The `examples/` directory contains comprehensive examples:

| Example | Description |
|---------|-------------|
| `basic.c` | Basic parsing, metadata extraction, canonicalization |
| `convert.c` | Format conversion and round-trip testing |
| `error_handling.c` | Comprehensive error handling patterns |
| `performance.c` | Performance benchmarking and optimization |
| `cmake_integration.c` | CMake integration demonstration |

Build and run examples:

```bash
cd build
cmake --build .

# Run examples
./examples/hedl_example_basic
./examples/hedl_example_convert
./examples/hedl_example_errors
./examples/hedl_example_performance
./examples/hedl_example_cmake_integration
```

## API Reference

### Document Lifecycle

```c
// Parse HEDL from string
int hedl_parse(const char* input, int input_len, int validate, HedlDocument** out_doc);

// Validate document structure
int hedl_validate(HedlDocument* doc);

// Free document (required)
void hedl_free_document(HedlDocument* doc);
```

### Metadata Inspection

```c
// Get document version
int hedl_get_version(HedlDocument* doc, int* major, int* minor);

// Get counts
int hedl_schema_count(HedlDocument* doc);
int hedl_alias_count(HedlDocument* doc);
int hedl_root_item_count(HedlDocument* doc);
```

### Format Conversion

```c
// To formats
int hedl_to_json(HedlDocument* doc, int pretty, char** out);
int hedl_to_yaml(HedlDocument* doc, char** out);
int hedl_to_xml(HedlDocument* doc, char** out);
int hedl_to_csv(HedlDocument* doc, char** out);
int hedl_to_parquet(HedlDocument* doc, uint8_t** out_bytes, size_t* out_len);
int hedl_to_neo4j_cypher(HedlDocument* doc, char** out);

// From formats
int hedl_from_json(const char* json, int json_len, HedlDocument** out);
int hedl_from_yaml(const char* yaml, int yaml_len, HedlDocument** out);
int hedl_from_xml(const char* xml, int xml_len, HedlDocument** out);
int hedl_from_parquet(const uint8_t* bytes, size_t len, HedlDocument** out);
```

### Canonicalization and Linting

```c
// Canonicalize to standard form
int hedl_canonicalize(HedlDocument* doc, char** out);

// Lint for issues
int hedl_lint(HedlDocument* doc, HedlDiagnostics** out);

// Diagnostics API
int hedl_diagnostics_count(HedlDiagnostics* diags);
int hedl_diagnostics_get(HedlDiagnostics* diags, int index, char** out_msg);
int hedl_diagnostics_severity(HedlDiagnostics* diags, int index, int* out_sev);
void hedl_free_diagnostics(HedlDiagnostics* diags);
```

### Error Handling

```c
// Get last error message (thread-local)
const char* hedl_get_last_error(void);

// Error codes
#define HEDL_OK               0
#define HEDL_ERR_NULL_PTR    -1
#define HEDL_ERR_INVALID_UTF8 -2
#define HEDL_ERR_PARSE       -3
#define HEDL_ERR_CANONICALIZE -4
#define HEDL_ERR_JSON        -5
// ... (see hedl.h for complete list)
```

### Memory Management

```c
// Free allocated strings
void hedl_free_string(char* str);

// Free byte arrays
void hedl_free_bytes(uint8_t* bytes, size_t len);

// All free functions are NULL-safe
```

## Memory Management Rules

**CRITICAL**: Follow these rules to avoid undefined behavior:

1. **Strings** returned by `hedl_to_*` and `hedl_canonicalize()` MUST be freed with `hedl_free_string()`
2. **Byte arrays** from `hedl_to_parquet()` MUST be freed with `hedl_free_bytes()`
3. **Documents** MUST be freed with `hedl_free_document()`
4. **Diagnostics** MUST be freed with `hedl_free_diagnostics()`
5. **NEVER** use `free()` on HEDL-allocated memory
6. **NULL pointers** are safe to pass to all `hedl_free_*()` functions

## Thread Safety

- **Error messages** are thread-local - each thread has independent error state
- **Document handles** are NOT thread-safe - don't share between threads without synchronization
- **Library functions** can be called from multiple threads simultaneously
- Call `hedl_get_last_error()` from the same thread that received the error

## Feature Flags

Control which format converters are included:

| CMake Option | Feature | Default |
|--------------|---------|---------|
| `HEDL_FEATURE_JSON` | JSON support | ON |
| `HEDL_FEATURE_YAML` | YAML support | ON |
| `HEDL_FEATURE_XML` | XML support | ON |
| `HEDL_FEATURE_CSV` | CSV support | ON |
| `HEDL_FEATURE_PARQUET` | Parquet support | ON |
| `HEDL_FEATURE_NEO4J` | Neo4j Cypher support | ON |

Disable unused formats to reduce binary size:

```bash
cmake .. \
  -DHEDL_FEATURE_PARQUET=OFF \
  -DHEDL_FEATURE_NEO4J=OFF
```

## Platform Support

- **Linux**: GCC, Clang
- **macOS**: Apple Clang
- **Windows**: MSVC, MinGW
- **BSD**: FreeBSD, OpenBSD

Tested architectures:
- x86_64
- ARM64/AArch64
- ARMv7 (32-bit)

## Performance

Benchmark results (x86_64, Rust release build):

| Operation | Throughput | Latency |
|-----------|-----------|---------|
| Parse (1000 items) | ~45 MB/s | ~2 ms |
| JSON conversion | ~80 MB/s | ~1 ms |
| YAML conversion | ~60 MB/s | ~1.5 ms |
| XML conversion | ~50 MB/s | ~2 ms |

See `examples/performance.c` for detailed benchmarks.

## Troubleshooting

### Library not found at runtime

**Linux/macOS:**
```bash
export LD_LIBRARY_PATH=/path/to/hedl/target/release:$LD_LIBRARY_PATH
# Or install to system location
sudo cmake --install . --prefix /usr/local
sudo ldconfig  # Linux only
```

**Windows:**
```cmd
set PATH=C:\path\to\hedl\target\release;%PATH%
```

### CMake can't find HEDL

Specify install location:
```bash
cmake .. -DCMAKE_PREFIX_PATH=/usr/local
```

### Build errors

Ensure Rust toolchain is installed:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version  # Verify installation
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Build with HEDL

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build and Install HEDL
        run: |
          cd hedl/bindings/c
          mkdir build && cd build
          cmake .. -DCMAKE_INSTALL_PREFIX=$HOME/.local
          cmake --build .
          cmake --install .

      - name: Build Your Project
        run: |
          mkdir build && cd build
          cmake .. -DCMAKE_PREFIX_PATH=$HOME/.local
          cmake --build .
          ctest --output-on-failure
```

### GitLab CI

```yaml
build:
  image: rust:latest
  script:
    - apt-get update && apt-get install -y cmake build-essential
    - cd hedl/bindings/c
    - mkdir build && cd build
    - cmake .. -DCMAKE_INSTALL_PREFIX=/usr/local
    - cmake --build .
    - cmake --install .
    - cd $CI_PROJECT_DIR
    - mkdir build && cd build
    - cmake ..
    - cmake --build .
    - ctest --output-on-failure
```

## License

Apache 2.0 - See [LICENSE](../../LICENSE) for details.

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md).

## Support

- **Issues**: https://github.com/dweve-ai/hedl/issues
- **Discussions**: https://github.com/dweve-ai/hedl/discussions

## See Also

- [HEDL Specification](../../SPEC.md)
- [Rust API Documentation](../../crates/hedl/README.md)
- [Python Bindings](../python/README.md)
- [Node.js Bindings](../node/README.md)
- [Ruby Bindings](../ruby/README.md)
