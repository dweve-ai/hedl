# SDK Documentation

Software Development Kits for using HEDL across different programming languages and platforms.

## Available SDKs

### Official SDKs

- **[Rust SDK](rust.md)** - Native Rust library with full type safety
- **[JavaScript/TypeScript SDK](javascript.md)** - WASM-based SDK for browsers and Node.js
- **[C/C++ SDK](c-cpp.md)** - FFI-based SDK for native applications
- **[Python SDK](python.md)** - Python bindings (if available)

## Quick Start by Language

### Rust
```bash
cargo add hedl
```
```rust
use hedl::{parse, to_json};

let doc = parse(hedl_text)?;
let json = to_json(&doc)?;
```
[Full Rust SDK Documentation →](rust.md)

### JavaScript/TypeScript
```bash
npm install hedl-wasm
```
```typescript
import init, { parse } from 'hedl-wasm';

await init();
const doc = parse(hedlText);
const json = doc.toJsonString();  // Method on HedlDocument
```
[Full JavaScript SDK Documentation →](javascript.md)

### C/C++
```c
#include "hedl.h"

HedlDocument* doc = NULL;
hedl_parse(input, -1, 0, &doc);
hedl_free_document(doc);
```
[Full C/C++ SDK Documentation →](c-cpp.md)

### Python
```python
import hedl

doc = hedl.parse(hedl_text)
json_str = hedl.to_json(doc)
```
[Full Python SDK Documentation →](python.md)

## Feature Comparison

| Feature | Rust | JavaScript | C/C++ | Python |
|---------|------|------------|-------|--------|
| Parsing | ✓ | ✓ | ✓ | ✓ |
| Serialization | ✓ | ✓ | ✓ | ✓ |
| Validation | ✓ | ✓ | ✓ | ✓ |
| Linting | ✓ | ✓ | ✓ | ✓ |
| JSON Conversion | ✓ | ✓ | ✓ | ✓ |
| YAML Conversion | ✓ | - | ✓ | ✓ |
| XML Conversion | ✓ | - | ✓ | ✓ |
| Parquet Conversion | ✓ | - | ✓ | ✓ |
| Neo4j/Cypher | ✓ | - | ✓ | ✓ |
| Streaming | ✓ | - | - | - |
| Async Support | ✓ | ✓ | - | ✓ |
| Type Safety | ✓✓ | ✓ | - | - |

## Installation Guides

### Rust
```toml
[dependencies]
hedl = "1.0"

# With optional features
hedl = { version = "1.0", features = ["yaml", "xml", "csv", "parquet", "neo4j"] }
```

### JavaScript/TypeScript
```bash
# npm
npm install hedl-wasm

# yarn
yarn add hedl-wasm

# pnpm
pnpm add hedl-wasm
```

### C/C++
Download pre-built libraries or build from source:
```bash
# Build from source
git clone https://github.com/dweve/hedl.git
cd hedl
cargo build --release -p hedl-ffi

# Library in target/release/
```

### Python
```bash
# pip (if bindings exist)
pip install hedl-python

# From source
git clone https://github.com/dweve/hedl.git
cd hedl/bindings/python
pip install .
```

## Platform Support

| Platform | Rust | JavaScript | C/C++ | Python |
|----------|------|------------|-------|--------|
| Linux | ✓ | ✓ | ✓ | ✓ |
| macOS | ✓ | ✓ | ✓ | ✓ |
| Windows | ✓ | ✓ | ✓ | ✓ |
| WebAssembly | - | ✓ | - | - |
| iOS | ✓ | - | ✓ | - |
| Android | ✓ | - | ✓ | - |

## Documentation Links

- **[Rust API Reference](../rust-api.md)** - Complete Rust API
- **[WASM API Reference](../wasm-api.md)** - JavaScript/WASM API
- **[FFI API Reference](../ffi-api.md)** - C/C++ FFI API
- **[Getting Started](../getting-started.md)** - Quick start guide
- **[Tutorials](../tutorials/README.md)** - Step-by-step tutorials

## Community SDKs

Third-party SDKs and bindings (not officially supported):

- **Go**: Community-maintained bindings
- **Ruby**: FFI-based bindings
- **Java/Kotlin**: JNI bindings
- **Swift**: Native Swift wrapper

## Contributing

Want to create a new language binding? See:
- [FFI API Reference](../ffi-api.md) for C interface
- [Contributing Guide](../../developer/contributing.md) for guidelines
- [Architecture Documentation](../../architecture/README.md) for internals

## Support

- **GitHub Issues**: Report SDK bugs and request features
- **Discussions**: Ask questions about SDKs
- **Documentation**: Browse complete API documentation
