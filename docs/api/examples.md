# API Examples

**Cross-language code examples for all HEDL APIs**

---

## Table of Contents

1. [Rust Examples](#rust-examples)
2. [C/FFI Examples](#cffi-examples)
3. [Python Examples](#python-examples)
4. [JavaScript/TypeScript Examples](#javascripttypescript-examples)
5. [Go Examples](#go-examples)
6. [MCP Integration Examples](#mcp-integration-examples)
7. [End-to-End Workflows](#end-to-end-workflows)

---

## Rust Examples

### Example 1: Basic Parsing

```rust
use hedl::{parse, HedlError};

fn main() -> Result<(), HedlError> {
    let hedl = r#"
        %VERSION: 1.0
        %STRUCT: User: [id, name, email]
        ---
        users: @User
          | alice, Alice Smith, alice@example.com
          | bob, Bob Jones, bob@example.com
    "#;

    let doc = parse(hedl)?;

    println!("Version: {}.{}", doc.version.0, doc.version.1);
    println!("Schemas: {}", doc.structs.len());
    println!("Root items: {}", doc.root.len());

    Ok(())
}
```

---

### Example 2: JSON Round-Trip

```rust
use hedl::{parse, to_json, from_json};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse HEDL
    let hedl = "%VERSION: 1.0\n---\nusers: [{id: alice, name: Alice}]";
    let doc = parse(hedl)?;

    // Convert to JSON
    let json = to_json(&doc)?;
    println!("JSON: {}", json);

    // Convert back to HEDL
    let doc2 = from_json(&json)?;

    // Verify round-trip
    assert_eq!(doc.version, doc2.version);

    Ok(())
}
```

---

### Example 3: Validation with Error Handling

```rust
use hedl::{parse, validate, lint};

fn main() {
    let hedl = r#"
        %VERSION: 1.0
        %STRUCT: User: [id, name]
        ---
        users: @User
          | alice, Alice Smith
          | bob  # Missing field!
    "#;

    // Validate first
    match validate(hedl) {
        Ok(_) => println!("Valid HEDL"),
        Err(e) => {
            eprintln!("Validation error at line {}: {}", e.line, e.message);
            return;
        }
    }

    // Parse and lint
    let doc = parse(hedl).unwrap();
    let diagnostics = lint(&doc);

    for d in diagnostics {
        println!("[{}] {}: {}",
            d.severity(),
            d.rule_id(),
            d.message()
        );
    }
}
```

---

### Example 4: Multi-Format Conversion

```rust
use hedl::{parse, to_json};

#[cfg(feature = "yaml")]
use hedl::yaml::to_yaml;

#[cfg(feature = "xml")]
use hedl::xml::to_xml;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hedl = "%VERSION: 1.0\n---\nkey: value";
    let doc = parse(hedl)?;

    // JSON
    let json = to_json(&doc)?;
    println!("JSON:\n{}\n", json);

    // YAML
    #[cfg(feature = "yaml")]
    {
        let yaml = to_yaml(&doc, &Default::default())?;
        println!("YAML:\n{}\n", yaml);
    }

    // XML
    #[cfg(feature = "xml")]
    {
        let xml = to_xml(&doc, &Default::default())?;
        println!("XML:\n{}\n", xml);
    }

    Ok(())
}
```

---

### Example 5: Entity Traversal

```rust
use hedl::{parse, Item, Node};

fn print_entities(node: &Node, indent: usize) {
    println!("{:indent$}{} ({})", "", node.id, node.type_name, indent = indent);

    for (child_type, children) in &node.children {
        println!("{:indent$}  {}:", "", child_type, indent = indent);
        for child in children {
            print_entities(child, indent + 4);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hedl = r#"
        %VERSION: 1.0
        %STRUCT: User: [id, name]
        %STRUCT: Post: [id, title]
        %NEST: User > Post
        ---
        users: @User
          | alice, Alice
            | post1, Hello World
            | post2, Second Post
    "#;

    let doc = parse(hedl)?;

    for (key, item) in &doc.root {
        match item {
            Item::List(list) => {
                println!("{}:", key);
                for node in &list.rows {
                    print_entities(node, 2);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
```

---

## C/FFI Examples

### Example 1: Basic Parse and Free

```c
#include <hedl_ffi.h>
#include <stdio.h>
#include <stdlib.h>

int main() {
    const char* hedl =
        "%VERSION: 1.0\n"
        "%STRUCT: User: [id, name]\n"
        "---\n"
        "users: @User\n"
        "  | alice, Alice\n";

    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl, -1, 1, &doc);

    if (result != HEDL_OK) {
        fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
        return 1;
    }

    // Get version
    int major, minor;
    hedl_get_version(doc, &major, &minor);
    printf("HEDL version: %d.%d\n", major, minor);

    // Get counts
    int schema_count = hedl_schema_count(doc);
    int root_count = hedl_root_item_count(doc);
    printf("Schemas: %d, Root items: %d\n", schema_count, root_count);

    // Cleanup
    hedl_free_document(doc);
    return 0;
}
```

**Compile**:
```bash
gcc -o example example.c -lhedl_ffi
./example
```

---

### Example 2: JSON Conversion

```c
#include <hedl_ffi.h>
#include <stdio.h>

int main() {
    const char* hedl = "%VERSION: 1.0\n---\nkey: value\n";
    HedlDocument* doc = NULL;
    char* json = NULL;

    // Parse
    if (hedl_parse(hedl, -1, 0, &doc) != HEDL_OK) {
        fprintf(stderr, "Parse failed: %s\n", hedl_get_last_error());
        return 1;
    }

    // Convert to JSON
    if (hedl_to_json(doc, 0, &json) != HEDL_OK) {
        fprintf(stderr, "Conversion failed: %s\n", hedl_get_last_error());
        hedl_free_document(doc);
        return 1;
    }

    printf("JSON output:\n%s\n", json);

    // Cleanup
    hedl_free_string(json);
    hedl_free_document(doc);
    return 0;
}
```

---

### Example 3: Error Handling

```c
#include <hedl_ffi.h>
#include <stdio.h>

void process_hedl(const char* input) {
    HedlDocument* doc = NULL;

    // Clear any previous errors
    hedl_clear_error_threadsafe();

    int result = hedl_parse(input, -1, 0, &doc);

    switch (result) {
        case HEDL_OK:
            printf("Success!\n");
            hedl_free_document(doc);
            break;

        case HEDL_ERR_PARSE:
            fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
            break;

        case HEDL_ERR_NULL_PTR:
            fprintf(stderr, "Null pointer error\n");
            break;

        case HEDL_ERR_INVALID_UTF8:
            fprintf(stderr, "Invalid UTF-8: %s\n", hedl_get_last_error());
            break;

        default:
            fprintf(stderr, "Unknown error: %s\n", hedl_get_last_error());
            break;
    }
}

int main() {
    process_hedl("%VERSION: 1.0\n---\nkey: value");
    process_hedl("invalid hedl");
    return 0;
}
```

---

### Example 4: Zero-Copy Callback

```c
#include <hedl_ffi.h>
#include <stdio.h>

void write_to_file(const char* chunk, size_t len, void* user_data) {
    FILE* fp = (FILE*)user_data;
    fwrite(chunk, 1, len, fp);
}

int main() {
    const char* hedl = "%VERSION: 1.0\n---\nkey: value";
    HedlDocument* doc = NULL;

    if (hedl_parse(hedl, -1, 0, &doc) != HEDL_OK) {
        fprintf(stderr, "Error: %s\n", hedl_get_last_error());
        return 1;
    }

    FILE* fp = fopen("output.json", "w");
    if (!fp) {
        perror("fopen");
        hedl_free_document(doc);
        return 1;
    }

    // Zero-copy streaming to file
    hedl_to_json_callback(doc, write_to_file, fp);

    fclose(fp);
    hedl_free_document(doc);

    printf("Written to output.json\n");
    return 0;
}
```

---

## Python Examples

### Example 1: Basic Usage with ctypes

```python
import ctypes
from pathlib import Path

# Load library
hedl = ctypes.CDLL('libhedl_ffi.so')  # or .dylib on macOS, .dll on Windows

# Define function signatures
hedl.hedl_parse.argtypes = [
    ctypes.c_char_p,
    ctypes.c_int32,
    ctypes.c_int32,
    ctypes.POINTER(ctypes.c_void_p)
]
hedl.hedl_parse.restype = ctypes.c_int

hedl.hedl_to_json.argtypes = [
    ctypes.c_void_p,
    ctypes.c_int32,
    ctypes.POINTER(ctypes.c_char_p)
]
hedl.hedl_to_json.restype = ctypes.c_int

hedl.hedl_free_document.argtypes = [ctypes.c_void_p]
hedl.hedl_free_string.argtypes = [ctypes.c_char_p]
hedl.hedl_get_last_error.restype = ctypes.c_char_p

# Parse HEDL
hedl_input = b"%VERSION: 1.0\n---\nkey: value"
doc = ctypes.c_void_p()

result = hedl.hedl_parse(hedl_input, -1, 0, ctypes.byref(doc))

if result != 0:  # HEDL_OK = 0
    error = hedl.hedl_get_last_error()
    print(f"Error: {error.decode()}")
else:
    # Convert to JSON
    json_ptr = ctypes.c_char_p()
    if hedl.hedl_to_json(doc, 1, ctypes.byref(json_ptr)) == 0:
        json_str = ctypes.string_at(json_ptr).decode()
        print(f"JSON: {json_str}")
        hedl.hedl_free_string(json_ptr)

    hedl.hedl_free_document(doc)
```

---

### Example 2: Python Wrapper Class

```python
import ctypes

class HedlFFI:
    def __init__(self, lib_path='libhedl_ffi.so'):
        self.lib = ctypes.CDLL(lib_path)
        self._setup_functions()

    def _setup_functions(self):
        # hedl_parse
        self.lib.hedl_parse.argtypes = [
            ctypes.c_char_p, ctypes.c_int32, ctypes.c_int32,
            ctypes.POINTER(ctypes.c_void_p)
        ]
        self.lib.hedl_parse.restype = ctypes.c_int

        # hedl_to_json
        self.lib.hedl_to_json.argtypes = [
            ctypes.c_void_p, ctypes.c_int32,
            ctypes.POINTER(ctypes.c_char_p)
        ]
        self.lib.hedl_to_json.restype = ctypes.c_int

        # Cleanup
        self.lib.hedl_free_document.argtypes = [ctypes.c_void_p]
        self.lib.hedl_free_string.argtypes = [ctypes.c_char_p]
        self.lib.hedl_get_last_error.restype = ctypes.c_char_p

    def parse(self, hedl_str):
        doc = ctypes.c_void_p()
        result = self.lib.hedl_parse(
            hedl_str.encode(),
            -1,
            1,
            ctypes.byref(doc)
        )

        if result != 0:
            error = self.lib.hedl_get_last_error().decode()
            raise RuntimeError(f"Parse error: {error}")

        return doc

    def to_json(self, doc, pretty=True):
        json_ptr = ctypes.c_char_p()
        result = self.lib.hedl_to_json(
            doc,
            1 if pretty else 0,
            ctypes.byref(json_ptr)
        )

        if result != 0:
            error = self.lib.hedl_get_last_error().decode()
            raise RuntimeError(f"Conversion error: {error}")

        json_str = ctypes.string_at(json_ptr).decode()
        self.lib.hedl_free_string(json_ptr)
        return json_str

    def free_document(self, doc):
        self.lib.hedl_free_document(doc)

# Usage
hedl = HedlFFI()

doc = hedl.parse("%VERSION: 1.0\n---\nkey: value")
json = hedl.to_json(doc)
print(json)
hedl.free_document(doc)
```

---

## JavaScript/TypeScript Examples

### Example 1: Browser Usage

```typescript
import init, { parse, toJson, validate } from 'hedl-wasm';

async function main() {
    // Initialize WASM module
    await init();

    const hedl = `
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
`;

    try {
        // Parse
        const doc = parse(hedl);
        console.log(`Version: ${doc.version}`);
        console.log(`Schemas: ${doc.schemaCount}`);

        // Get schema info
        const schemas = doc.getSchemaNames();
        console.log('Available schemas:', schemas);

        const userSchema = doc.getSchema('User');
        console.log('User fields:', userSchema);

        // Convert to JSON
        const json = doc.toJsonString(true);
        console.log('JSON:', json);

        // Count entities
        const counts = doc.countEntities();
        console.log('Entity counts:', counts);

    } catch (e) {
        console.error('Error:', e.message);
    }
}

main();
```

---

### Example 2: Node.js File Processing

```javascript
const hedl = require('hedl-wasm');
const fs = require('fs').promises;

async function processHedlFile(inputPath, outputPath) {
    await hedl.default();  // Initialize

    try {
        // Read HEDL file
        const content = await fs.readFile(inputPath, 'utf-8');

        // Validate
        const validation = hedl.validate(content, true);
        if (!validation.valid) {
            console.error('Validation errors:');
            validation.errors.forEach(err => {
                console.error(`  Line ${err.line}: ${err.message}`);
            });
            return;
        }

        // Convert to JSON
        const json = hedl.toJson(content, true);

        // Write output
        await fs.writeFile(outputPath, json);
        console.log(`Converted ${inputPath} -> ${outputPath}`);

        // Get stats
        const stats = hedl.getStats(content);
        console.log(`Token savings: ${stats.savingsPercent}%`);

    } catch (e) {
        console.error('Error:', e.message);
    }
}

processHedlFile('input.hedl', 'output.json');
```

---

### Example 3: React Component

```typescript
import React, { useState, useEffect } from 'react';
import init, { parse, validate, getStats, HedlDocument } from 'hedl-wasm';

interface EditorProps {
    initialValue?: string;
}

export function HedlEditor({ initialValue = '' }: EditorProps) {
    const [initialized, setInitialized] = useState(false);
    const [hedl, setHedl] = useState(initialValue);
    const [doc, setDoc] = useState<HedlDocument | null>(null);
    const [errors, setErrors] = useState<string[]>([]);
    const [stats, setStats] = useState<any>(null);

    useEffect(() => {
        init().then(() => setInitialized(true));
    }, []);

    useEffect(() => {
        if (!initialized || !hedl) return;

        try {
            // Validate
            const validation = validate(hedl, true);
            if (!validation.valid) {
                setErrors(validation.errors.map(e =>
                    `Line ${e.line}: ${e.message}`
                ));
                setDoc(null);
                setStats(null);
                return;
            }

            // Parse
            const parsed = parse(hedl);
            setDoc(parsed);
            setErrors([]);

            // Get stats
            const s = getStats(hedl);
            setStats(s);

        } catch (e: any) {
            setErrors([e.message]);
            setDoc(null);
            setStats(null);
        }
    }, [hedl, initialized]);

    if (!initialized) {
        return <div>Loading WASM...</div>;
    }

    return (
        <div className="hedl-editor">
            <div className="editor-pane">
                <textarea
                    value={hedl}
                    onChange={(e) => setHedl(e.target.value)}
                    placeholder="Enter HEDL here..."
                    rows={20}
                    cols={80}
                />
            </div>

            <div className="info-pane">
                {errors.length > 0 && (
                    <div className="errors">
                        <h3>Errors</h3>
                        <ul>
                            {errors.map((err, i) => (
                                <li key={i}>{err}</li>
                            ))}
                        </ul>
                    </div>
                )}

                {doc && (
                    <div className="document-info">
                        <h3>Document Info</h3>
                        <p>Version: {doc.version}</p>
                        <p>Schemas: {doc.schemaCount}</p>
                        <p>Entities: {doc.rootItemCount}</p>
                    </div>
                )}

                {stats && (
                    <div className="stats">
                        <h3>Statistics</h3>
                        <p>HEDL: {stats.hedlTokens} tokens</p>
                        <p>JSON: {stats.jsonTokens} tokens</p>
                        <p>Savings: {stats.savingsPercent}%</p>
                    </div>
                )}
            </div>
        </div>
    );
}
```

---

## Go Examples

### Example 1: Basic FFI Usage

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
    hedl := C.CString("%VERSION: 1.0\n---\nkey: value")
    defer C.free(unsafe.Pointer(hedl))

    var doc *C.HedlDocument
    result := C.hedl_parse(hedl, -1, 0, &doc)

    if result != C.HEDL_OK {
        errMsg := C.hedl_get_last_error()
        fmt.Printf("Parse error: %s\n", C.GoString(errMsg))
        return
    }
    defer C.hedl_free_document(doc)

    // Get version
    var major, minor C.int
    C.hedl_get_version(doc, &major, &minor)
    fmt.Printf("Version: %d.%d\n", major, minor)

    // Convert to JSON
    var jsonPtr *C.char
    if C.hedl_to_json(doc, 1, &jsonPtr) == C.HEDL_OK {
        json := C.GoString(jsonPtr)
        fmt.Printf("JSON: %s\n", json)
        C.hedl_free_string(jsonPtr)
    }
}
```

---

### Example 2: Go Wrapper Package

```go
package hedl

/*
#cgo LDFLAGS: -lhedl_ffi
#include <hedl_ffi.h>
#include <stdlib.h>
*/
import "C"
import (
    "errors"
    "unsafe"
)

type Document struct {
    ptr *C.HedlDocument
}

func Parse(hedl string) (*Document, error) {
    cHedl := C.CString(hedl)
    defer C.free(unsafe.Pointer(cHedl))

    var doc *C.HedlDocument
    result := C.hedl_parse(cHedl, -1, 0, &doc)

    if result != C.HEDL_OK {
        errMsg := C.hedl_get_last_error()
        return nil, errors.New(C.GoString(errMsg))
    }

    return &Document{ptr: doc}, nil
}

func (d *Document) ToJSON(pretty bool) (string, error) {
    var prettyFlag C.int
    if pretty {
        prettyFlag = 1
    }

    var jsonPtr *C.char
    result := C.hedl_to_json(d.ptr, prettyFlag, &jsonPtr)

    if result != C.HEDL_OK {
        errMsg := C.hedl_get_last_error()
        return "", errors.New(C.GoString(errMsg))
    }
    defer C.hedl_free_string(jsonPtr)

    return C.GoString(jsonPtr), nil
}

func (d *Document) Free() {
    if d.ptr != nil {
        C.hedl_free_document(d.ptr)
        d.ptr = nil
    }
}

// Usage
func main() {
    doc, err := Parse("%VERSION: 1.0\n---\nkey: value")
    if err != nil {
        panic(err)
    }
    defer doc.Free()

    json, err := doc.ToJSON(true)
    if err != nil {
        panic(err)
    }

    fmt.Println(json)
}
```

---

## MCP Integration Examples

### Example 1: Claude Desktop Integration

```json
{
    "mcpServers": {
        "hedl": {
            "command": "hedl-mcp",
            "args": [
                "--root",
                "/Users/username/Documents/hedl-data"
            ]
        }
    }
}
```

**Usage in Claude**:
```
User: Read the users.hedl file

Claude: [Uses hedl_read tool]
{
    "name": "hedl_read",
    "arguments": {
        "path": "users.hedl",
        "include_json": true
    }
}
```

---

### Example 2: Custom MCP Client

```typescript
import { Client } from '@modelcontextprotocol/sdk/client/index.js';
import { StdioClientTransport } from '@modelcontextprotocol/sdk/client/stdio.js';

async function queryHedlData() {
    const transport = new StdioClientTransport({
        command: 'hedl-mcp',
        args: ['--root', '/path/to/data']
    });

    const client = new Client({
        name: 'hedl-analyzer',
        version: '1.0.0'
    }, {
        capabilities: {}
    });

    await client.connect(transport);

    // Read HEDL file
    const readResult = await client.request({
        method: 'tools/call',
        params: {
            name: 'hedl_read',
            arguments: {
                path: 'analytics.hedl',
                include_json: true
            }
        }
    });

    console.log('File data:', readResult);

    // Query specific entities
    const queryResult = await client.request({
        method: 'tools/call',
        params: {
            name: 'hedl_query',
            arguments: {
                hedl: readResult.content,
                type_name: 'User'
            }
        }
    });

    console.log('Users:', queryResult);

    await client.close();
}

queryHedlData();
```

---

## End-to-End Workflows

### Workflow 1: Data Pipeline (Rust)

```rust
use hedl::{parse, to_json, from_json, canonicalize, validate};
use std::fs;

fn process_data_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Read raw JSON data
    let json_input = fs::read_to_string("raw_data.json")?;

    // 2. Convert to HEDL for token-efficient storage
    let doc = from_json(&json_input)?;
    let hedl = canonicalize(&doc)?;
    fs::write("data.hedl", hedl)?;

    // 3. Later: Read HEDL and process
    let hedl_stored = fs::read_to_string("data.hedl")?;
    validate(&hedl_stored)?;
    let doc = parse(&hedl_stored)?;

    // 4. Convert back to JSON for downstream systems
    let json_output = to_json(&doc)?;
    fs::write("processed_data.json", json_output)?;

    println!("Pipeline completed successfully");
    Ok(())
}
```

---

### Workflow 2: Multi-Language Integration

**Step 1: Generate HEDL (Python)**
```python
import json
from hedl_wrapper import HedlFFI

hedl = HedlFFI()

# Read JSON
with open('input.json', 'r') as f:
    json_data = f.read()

# Convert to HEDL via FFI
doc = hedl.from_json(json_data)
hedl_str = hedl.canonicalize(doc)

# Save
with open('data.hedl', 'w') as f:
    f.write(hedl_str)

hedl.free_document(doc)
```

**Step 2: Process (Rust)**
```rust
use hedl::{parse, Item};

fn analyze_hedl() {
    let hedl = std::fs::read_to_string("data.hedl").unwrap();
    let doc = parse(&hedl).unwrap();

    for (key, item) in &doc.root {
        match item {
            Item::List(list) => {
                println!("{}: {} entities", key, list.rows.len());
            }
            _ => {}
        }
    }
}
```

**Step 3: Export (Go)**
```go
package main

import (
    "fmt"
    "io/ioutil"
    "hedl"  // Your Go wrapper
)

func main() {
    data, _ := ioutil.ReadFile("data.hedl")
    doc, _ := hedl.Parse(string(data))
    defer doc.Free()

    json, _ := doc.ToJSON(true)
    ioutil.WriteFile("output.json", []byte(json), 0644)

    fmt.Println("Export completed")
}
```

---

### Workflow 3: Web Application (Full Stack)

**Backend (Rust + MCP)**
```rust
// Server provides HEDL MCP service
hedl-mcp --root /data/hedl-files
```

**Frontend (TypeScript + React)**
```typescript
import { useEffect, useState } from 'react';
import init, { parse, getStats } from 'hedl-wasm';

function DataViewer() {
    const [data, setData] = useState(null);

    useEffect(() => {
        init().then(async () => {
            // Fetch HEDL from backend
            const response = await fetch('/api/data.hedl');
            const hedl = await response.text();

            // Parse in browser
            const doc = parse(hedl);
            const json = toJson(hedl, true);
            setData(JSON.parse(json));

            // Show savings
            const stats = getStats(hedl);
            console.log(`Saved ${stats.savingsPercent}% tokens`);
        });
    }, []);

    return <div>{JSON.stringify(data, null, 2)}</div>;
}
```

---

## Summary

These examples demonstrate:

1. **Rust**: Native, high-performance processing
2. **C/FFI**: Integration with system languages
3. **Python**: Scripting and data processing
4. **JavaScript**: Web and Node.js applications
5. **Go**: Backend services
6. **MCP**: AI/LLM integration
7. **Full Stack**: Complete application workflows

**Next Steps**:
- Choose the API that fits your use case
- Adapt the examples to your specific needs
- Refer to the specific API documentation for detailed function signatures

---

**See Also**:
- [Rust API Reference](rust-api.md)
- [FFI/C API Reference](ffi-api.md)
- [WASM/JavaScript API Reference](wasm-api.md)
- [MCP Server API Reference](mcp-api.md)
- [LSP API Reference](lsp-api.md)
