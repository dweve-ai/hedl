# hedl-ffi

C ABI bindings for HEDL, enabling use from C, C++, and other languages.

## Installation

```toml
[dependencies]
hedl-ffi = "1.0"
```

## Building

```bash
cargo build --release -p hedl-ffi
# Outputs: libhedl.so / libhedl.dylib / hedl.dll
```

## C API

```c
#include "hedl.h"

// Parse HEDL
HedlDocument* doc = hedl_parse(hedl_str, strlen(hedl_str));
if (!doc) {
    const char* err = hedl_last_error();
    // Handle error
}

// Convert to JSON
char* json = hedl_to_json(doc);
printf("%s\n", json);

// Cleanup
hedl_free_string(json);
hedl_free_document(doc);
```

## Features

- `json` - JSON conversion
- `yaml` - YAML conversion
- `xml` - XML conversion
- `csv` - CSV conversion
- `parquet` - Parquet conversion
- `neo4j` - Neo4j Cypher generation
- `toon` - TOON conversion
- `all-formats` - All format converters (default)

## Header File

The `hedl.h` header is generated automatically during build via cbindgen.

## Thread Safety

All functions are thread-safe. Documents can be shared across threads with proper synchronization.

## License

Apache-2.0
