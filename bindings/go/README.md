# HEDL Go Bindings

Go bindings for HEDL (Hierarchical Entity Data Language) - a token-efficient data format optimized for LLM context windows.

## Installation

```bash
go get github.com/dweve-ai/hedl/bindings/go
```

**Prerequisites:**
- The HEDL shared library must be installed (`libhedl_ffi.so`/`.dylib`)
- CGO must be enabled

### Building the shared library

```bash
cd /path/to/hedl
cargo build --release -p hedl-ffi
sudo cp target/release/libhedl_ffi.so /usr/local/lib/
sudo ldconfig
```

## Quick Start

```go
package main

import (
    "fmt"
    "log"

    "github.com/dweve-ai/hedl/bindings/go"
)

func main() {
    // Parse HEDL content
    doc, err := hedl.Parse(`
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
`, true)
    if err != nil {
        log.Fatal(err)
    }
    defer doc.Close()

    // Get document info
    major, minor, _ := doc.Version()
    fmt.Printf("Version: %d.%d\n", major, minor)

    schemaCount, _ := doc.SchemaCount()
    fmt.Printf("Schemas: %d\n", schemaCount)

    // Convert to JSON
    json, err := doc.ToJSON(false)
    if err != nil {
        log.Fatal(err)
    }
    fmt.Println(json)

    // Convert to other formats
    yaml, _ := doc.ToYAML(false)
    xml, _ := doc.ToXML()
    csv, _ := doc.ToCSV()
    cypher, _ := doc.ToCypher(true)
}
```

## API Reference

### Package Functions

| Function | Description |
|----------|-------------|
| `Parse(content, strict)` | Parse HEDL string |
| `Validate(content, strict)` | Validate without creating document |
| `FromJSON(content)` | Parse JSON to HEDL document |
| `FromYAML(content)` | Parse YAML to HEDL document |
| `FromXML(content)` | Parse XML to HEDL document |
| `FromParquet(data)` | Parse Parquet to HEDL document |

### Document Methods

| Method | Description |
|--------|-------------|
| `Version()` | Get (major, minor, error) |
| `SchemaCount()` | Get schema count |
| `AliasCount()` | Get alias count |
| `RootItemCount()` | Get root item count |
| `Canonicalize()` | Convert to canonical HEDL |
| `ToJSON(includeMetadata)` | Convert to JSON |
| `ToYAML(includeMetadata)` | Convert to YAML |
| `ToXML()` | Convert to XML |
| `ToCSV()` | Convert to CSV |
| `ToParquet()` | Convert to Parquet bytes |
| `ToCypher(useMerge)` | Convert to Neo4j Cypher |
| `Lint()` | Run linting |
| `Close()` | Free resources |

### Diagnostics

```go
diag, err := doc.Lint()
if err != nil {
    log.Fatal(err)
}
defer diag.Close()

fmt.Printf("Issues: %d\n", diag.Count())

errors, _ := diag.Errors()
warnings, _ := diag.Warnings()
```

### Error Handling

```go
doc, err := hedl.Parse("invalid content", true)
if err != nil {
    if hedlErr, ok := err.(*hedl.HedlError); ok {
        fmt.Printf("Error code: %d\n", hedlErr.Code)
        fmt.Printf("Message: %s\n", hedlErr.Message)
    }
}
```

## Environment Variables

| Variable | Description | Default | Recommended |
|----------|-------------|---------|-------------|
| `HEDL_MAX_OUTPUT_SIZE` | Maximum output size in bytes for conversions | 100 MB | 500 MB - 1 GB |

### Resource Limits

The `HEDL_MAX_OUTPUT_SIZE` environment variable controls the maximum size of output from conversion operations (`ToJSON()`, `ToYAML()`, `ToXML()`, etc.). The default of 100 MB is conservative and may be too restrictive for many real-world data processing scenarios.

**Setting the limit:**

```bash
# In your shell (before running Go)
export HEDL_MAX_OUTPUT_SIZE=1073741824  # 1 GB

# Or in Go (must be set BEFORE importing hedl)
package main

import (
    "os"
    _ "github.com/dweve-ai/hedl/bindings/go"  // Triggers init()
)

func init() {
    os.Setenv("HEDL_MAX_OUTPUT_SIZE", "1073741824")  // Must be before import
}
```

**Recommended values:**

- **Small configs (10-50 MB)**: Default 100 MB is usually sufficient
- **Medium datasets (100-500 MB)**: Set to `524288000` (500 MB)
- **Large datasets (500 MB - 5 GB)**: Set to `1073741824` or higher (1 GB+)
- **Very large datasets**: Set to `5368709120` (5 GB) or `10737418240` (10 GB)
- **No practical limit**: Set to a very high value appropriate for your system

**Error handling:**

When the output size exceeds the limit, an error will be returned:

```go
json, err := doc.ToJSON(false)
if err != nil {
    if hedlErr, ok := err.(*hedl.HedlError); ok && hedlErr.Code == hedl.ErrAlloc {
        fmt.Println("Output too large:", hedlErr.Message)
        fmt.Println("Increase HEDL_MAX_OUTPUT_SIZE environment variable")
    }
}
```

## Build Configuration

The bindings expect the library in standard paths. To customize:

```bash
# Set library path
export LD_LIBRARY_PATH=/path/to/lib:$LD_LIBRARY_PATH

# Or use CGO flags
CGO_LDFLAGS="-L/path/to/lib" go build
```

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
