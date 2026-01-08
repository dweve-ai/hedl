# HEDL Node.js Bindings

Node.js bindings for HEDL (Hierarchical Entity Data Language) - a token-efficient data format optimized for LLM context windows.

## Installation

```bash
npm install hedl
```

**Note:** Requires the HEDL shared library (`libhedl_ffi.so`/`.dylib`/`.dll`) to be available.

## Quick Start

```typescript
import * as hedl from 'hedl';

// Parse HEDL content
const doc = hedl.parse(`
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
`);

// Get document info
console.log(`Version: ${doc.version}`); // [1, 0]
console.log(`Schemas: ${doc.schemaCount}`); // 1

// Convert to JSON
const jsonStr = doc.toJson();
console.log(jsonStr);

// Convert to other formats
const yamlStr = doc.toYaml();
const xmlStr = doc.toXml();
const csvStr = doc.toCsv();
const cypherStr = doc.toCypher();

// Clean up
doc.close();
```

## API Reference

### Module Functions (Synchronous)

| Function | Description |
|----------|-------------|
| `parse(content, strict?)` | Parse HEDL string |
| `validate(content, strict?)` | Validate without creating document |
| `fromJson(content)` | Parse JSON to HEDL document |
| `fromYaml(content)` | Parse YAML to HEDL document |
| `fromXml(content)` | Parse XML to HEDL document |
| `fromParquet(buffer)` | Parse Parquet to HEDL document |

### Module Functions (Asynchronous)

All parsing functions have async variants using `setImmediate` for non-blocking execution:

| Function | Description |
|----------|-------------|
| `parseAsync(content, strict?)` | Parse HEDL string asynchronously |
| `validateAsync(content, strict?)` | Validate asynchronously |
| `fromJsonAsync(content)` | Parse JSON to HEDL asynchronously |
| `fromYamlAsync(content)` | Parse YAML to HEDL asynchronously |
| `fromXmlAsync(content)` | Parse XML to HEDL asynchronously |
| `fromParquetAsync(buffer)` | Parse Parquet to HEDL asynchronously |

### Document Properties

| Property | Type | Description |
|----------|------|-------------|
| `version` | `[number, number]` | HEDL version tuple |
| `schemaCount` | `number` | Number of schemas |
| `aliasCount` | `number` | Number of aliases |
| `rootItemCount` | `number` | Number of root items |

### Document Methods (Synchronous)

| Method | Description |
|--------|-------------|
| `canonicalize()` | Convert to canonical HEDL |
| `toJson(includeMetadata?)` | Convert to JSON |
| `toYaml(includeMetadata?)` | Convert to YAML |
| `toXml()` | Convert to XML |
| `toCsv()` | Convert to CSV |
| `toParquet()` | Convert to Parquet Buffer |
| `toCypher(useMerge?)` | Convert to Neo4j Cypher |
| `lint()` | Run linting |
| `close()` | Free resources |

### Document Methods (Asynchronous)

All conversion methods have async variants using `setImmediate` for non-blocking execution:

| Method | Description |
|--------|-------------|
| `canonicalizeAsync()` | Convert to canonical HEDL asynchronously |
| `toJsonAsync(includeMetadata?)` | Convert to JSON asynchronously |
| `toYamlAsync(includeMetadata?)` | Convert to YAML asynchronously |
| `toXmlAsync()` | Convert to XML asynchronously |
| `toCsvAsync()` | Convert to CSV asynchronously |
| `toParquetAsync()` | Convert to Parquet Buffer asynchronously |
| `toCypherAsync(useMerge?)` | Convert to Neo4j Cypher asynchronously |
| `lintAsync()` | Run linting asynchronously |

## Async API

The library provides async variants of all blocking operations using Node.js `setImmediate` for non-blocking execution. This prevents the event loop from blocking during large parsing or conversion operations.

### When to Use Async

Use async variants when:
- Processing large documents that may block the event loop
- Building high-concurrency servers where blocking is unacceptable
- Processing multiple documents simultaneously
- Integrating with async/await-based codebases

### Async Examples

**Parse HEDL asynchronously:**

```typescript
import * as hedl from 'hedl';

// Async parsing
const doc = await hedl.parseAsync('%VERSION: 1.0\n---\nkey: value');
console.log(doc.version); // [1, 0]
doc.close();
```

**Async conversion:**

```typescript
const doc = await hedl.parseAsync(hedlContent);

// Convert asynchronously
const json = await doc.toJsonAsync();
const yaml = await doc.toYamlAsync();
const xml = await doc.toXmlAsync();

doc.close();
```

**Validate asynchronously:**

```typescript
const isValid = await hedl.validateAsync(hedlContent);
if (isValid) {
  const doc = await hedl.parseAsync(hedlContent);
  // ...
  doc.close();
}
```

**Convert between formats asynchronously:**

```typescript
// JSON to HEDL
const doc = await hedl.fromJsonAsync(jsonString);
const canonical = await doc.canonicalizeAsync();

// YAML to HEDL
const doc2 = await hedl.fromYamlAsync(yamlString);
const json = await doc2.toJsonAsync();

doc.close();
doc2.close();
```

**Async chaining:**

```typescript
try {
  // Parse async
  const doc = await hedl.parseAsync(hedlContent);

  // Convert to JSON async
  const json = await doc.toJsonAsync();

  // Parse JSON back to HEDL async
  const doc2 = await hedl.fromJsonAsync(json);

  // Convert to YAML async
  const yaml = await doc2.toYamlAsync();

  // Lint asynchronously
  const diag = await doc.lintAsync();
  console.log(`Found ${diag.length} issues`);

  // Clean up
  diag.close();
  doc.close();
  doc2.close();
} catch (err) {
  console.error('Error processing HEDL:', err);
}
```

**Concurrent processing:**

```typescript
// Process multiple documents concurrently
const [doc1, doc2, doc3] = await Promise.all([
  hedl.parseAsync(content1),
  hedl.parseAsync(content2),
  hedl.parseAsync(content3),
]);

const [json1, json2, json3] = await Promise.all([
  doc1.toJsonAsync(),
  doc2.toJsonAsync(),
  doc3.toJsonAsync(),
]);

// Use results...

doc1.close();
doc2.close();
doc3.close();
```

**Async server example:**

```typescript
import * as hedl from 'hedl';
import * as http from 'http';

const server = http.createServer(async (req, res) => {
  try {
    // Request body is HEDL format
    const hedlContent = await readBody(req);

    // Parse asynchronously (non-blocking)
    const doc = await hedl.parseAsync(hedlContent);

    // Convert to JSON asynchronously
    const json = await doc.toJsonAsync();

    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(json);

    doc.close();
  } catch (err) {
    res.writeHead(400, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ error: err.message }));
  }
});

server.listen(3000);
```

### Diagnostics

```typescript
const diag = doc.lint();
console.log(`Issues: ${diag.length}`);
console.log(`Errors: ${diag.errors}`);
console.log(`Warnings: ${diag.warnings}`);
diag.close();
```

**Async linting:**

```typescript
const doc = await hedl.parseAsync(content);
const diag = await doc.lintAsync();

console.log(`Issues: ${diag.length}`);
console.log(`Errors: ${diag.errors}`);
console.log(`Warnings: ${diag.warnings}`);

diag.close();
doc.close();
```

## Environment Variables

| Variable | Description | Default | Recommended |
|----------|-------------|---------|-------------|
| `HEDL_LIB_PATH` | Path to the HEDL shared library | Auto-detected | - |
| `HEDL_MAX_OUTPUT_SIZE` | Maximum output size in bytes for conversions | 100 MB | 500 MB - 1 GB |

### Resource Limits

The `HEDL_MAX_OUTPUT_SIZE` environment variable controls the maximum size of output from conversion operations (`toJson()`, `toYaml()`, `toXml()`, etc.). The default of 100 MB is conservative and may be too restrictive for many real-world data processing scenarios.

**Setting the limit:**

```bash
# In your shell (before running Node.js)
export HEDL_MAX_OUTPUT_SIZE=1073741824  # 1 GB

# Or in Node.js (must be set BEFORE import)
process.env.HEDL_MAX_OUTPUT_SIZE = '1073741824';  // 1 GB
import * as hedl from 'hedl';
```

**Recommended values:**

- **Small configs (10-50 MB)**: Default 100 MB is usually sufficient
- **Medium datasets (100-500 MB)**: Set to `524288000` (500 MB)
- **Large datasets (500 MB - 5 GB)**: Set to `1073741824` or higher (1 GB+)
- **Very large datasets**: Set to `5368709120` (5 GB) or `10737418240` (10 GB)
- **No practical limit**: Set to a very high value appropriate for your system

**Error handling:**

When the output size exceeds the limit, a `HedlError` will be thrown:

```typescript
import { HedlError, HEDL_ERR_ALLOC } from 'hedl';

try {
  const largeOutput = doc.toJson();
} catch (e) {
  if (e instanceof HedlError && e.code === HEDL_ERR_ALLOC) {
    console.error('Output too large:', e.message);
    console.error('Increase HEDL_MAX_OUTPUT_SIZE environment variable');
  }
}
```

## TypeScript

Full TypeScript support with type definitions included.

```typescript
import { Document, Diagnostics, HedlError, parse } from 'hedl';
```

## License

Licensed under either of Apache License 2.0 or MIT license at your option.
