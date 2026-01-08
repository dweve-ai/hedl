# HEDL C# Bindings

C# bindings for HEDL (Hierarchical Entity Data Language) - a token-efficient data format optimized for LLM context windows.

## Installation

```bash
dotnet add package Dweve.Hedl
```

**Prerequisites:** The HEDL native library (`hedl_ffi.dll`/`libhedl_ffi.so`/`libhedl_ffi.dylib`) must be in the library search path.

## Quick Start

```csharp
using Dweve.Hedl;

// Parse HEDL content
using var doc = Hedl.Parse(@"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
");

// Get document info
Console.WriteLine($"Version: {doc.Version}");      // (1, 0)
Console.WriteLine($"Schemas: {doc.SchemaCount}");  // 1

// Convert to JSON
string json = doc.ToJson();
Console.WriteLine(json);

// Convert to other formats
string yaml = doc.ToYaml();
string xml = doc.ToXml();
string csv = doc.ToCsv();
string cypher = doc.ToCypher();
```

## API Reference

### Static Methods (Hedl class)

| Method | Description |
|--------|-------------|
| `Hedl.Parse(content, strict)` | Parse HEDL string into a Document |
| `Hedl.Validate(content, strict)` | Validate without creating document |
| `Hedl.FromJson(content)` | Parse JSON to HEDL Document |
| `Hedl.FromYaml(content)` | Parse YAML to HEDL Document |
| `Hedl.FromXml(content)` | Parse XML to HEDL Document |
| `Hedl.FromParquet(data)` | Parse Parquet bytes to HEDL Document |

### Document Properties

| Property | Description |
|----------|-------------|
| `Version` | Get (Major, Minor) tuple |
| `SchemaCount` | Get number of schemas |
| `AliasCount` | Get number of aliases |
| `RootItemCount` | Get number of root items |

### Document Methods

| Method | Description |
|--------|-------------|
| `Canonicalize()` | Convert to canonical HEDL |
| `ToJson(includeMetadata)` | Convert to JSON |
| `ToYaml(includeMetadata)` | Convert to YAML |
| `ToXml()` | Convert to XML |
| `ToCsv()` | Convert to CSV |
| `ToParquet()` | Convert to Parquet bytes |
| `ToCypher(useMerge)` | Convert to Neo4j Cypher |
| `Lint()` | Run linting, returns Diagnostics |
| `Dispose()` | Free resources |

### Diagnostics

```csharp
using var diag = doc.Lint();
Console.WriteLine($"Issues: {diag.Count}");

for (int i = 0; i < diag.Count; i++)
{
    var item = diag[i];
    Console.WriteLine($"[{item.Severity}] {item.Message}");
}

// Get by severity
string[] errors = diag.GetErrors();
string[] warnings = diag.GetWarnings();
string[] hints = diag.GetHints();
```

### Error Handling

```csharp
try
{
    using var doc = Hedl.Parse("invalid content");
}
catch (HedlException ex)
{
    Console.WriteLine($"Error: {ex.Message}");
    Console.WriteLine($"Code: {ex.ErrorCode}");
}
```

## Environment Variables

| Variable | Description | Default | Recommended |
|----------|-------------|---------|-------------|
| `HEDL_MAX_OUTPUT_SIZE` | Maximum output size in bytes for conversions | 100 MB | 500 MB - 1 GB |

### Resource Limits

The `HEDL_MAX_OUTPUT_SIZE` environment variable controls the maximum size of output from conversion operations (`ToJson()`, `ToYaml()`, `ToXml()`, etc.). The default of 100 MB is conservative and may be too restrictive for many real-world data processing scenarios.

**Setting the limit:**

```bash
# Windows (Command Prompt)
set HEDL_MAX_OUTPUT_SIZE=1073741824

# Windows (PowerShell)
$env:HEDL_MAX_OUTPUT_SIZE = "1073741824"

# Linux/macOS
export HEDL_MAX_OUTPUT_SIZE=1073741824

# Or in C# (must be set BEFORE using Hedl)
Environment.SetEnvironmentVariable("HEDL_MAX_OUTPUT_SIZE", "1073741824");  // 1 GB
using Dweve.Hedl;
```

**Recommended values:**

- **Small configs (10-50 MB)**: Default 100 MB is usually sufficient
- **Medium datasets (100-500 MB)**: Set to `524288000` (500 MB)
- **Large datasets (500 MB - 5 GB)**: Set to `1073741824` or higher (1 GB+)
- **Very large datasets**: Set to `5368709120` (5 GB) or `10737418240` (10 GB)
- **No practical limit**: Set to a very high value appropriate for your system

**Error handling:**

When the output size exceeds the limit, a `HedlException` will be thrown:

```csharp
try
{
    string largeOutput = doc.ToJson();
}
catch (HedlException ex) when (ex.ErrorCode == HedlErrorCode.Alloc)
{
    Console.WriteLine($"Output too large: {ex.Message}");
    Console.WriteLine("Increase HEDL_MAX_OUTPUT_SIZE environment variable");
}
```

## Building the Native Library

```bash
cd /path/to/hedl
cargo build --release -p hedl-ffi
```

The library will be at:
- Linux: `target/release/libhedl_ffi.so`
- macOS: `target/release/libhedl_ffi.dylib`
- Windows: `target/release/hedl_ffi.dll`

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
