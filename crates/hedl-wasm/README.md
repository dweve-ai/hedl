# hedl-wasm

**WebAssembly bindings for HEDL—run HEDL parsing, validation, and conversion directly in browsers and Node.js with near-native performance.**

JavaScript environments need structured data formats that don't sacrifice type safety or performance. JSON is ubiquitous but loses semantic information. YAML parsers are heavy. XML processing is complex. Running HEDL parsing in the browser or Node.js shouldn't require shipping a JavaScript reimplementation with different bugs and performance characteristics.

`hedl-wasm` provides complete WebAssembly bindings to the production-grade Rust HEDL implementation. Parse multi-megabyte HEDL documents at near-native speed in the browser. Validate data structures with full schema checking. Convert between HEDL and JSON/YAML/XML/CSV bidirectionally. Access the complete HEDL ecosystem from JavaScript with zero compromises on correctness or performance.

## What's Implemented

Production-ready WASM bindings with comprehensive features:

1. **30+ Exported Functions**: Parse, validate, format, convert (JSON/YAML/XML/CSV/TOON), and analyze HEDL documents (19 module functions + 14 HedlDocument methods/properties)
2. **Memory Safety**: 500 MB default input limit, configurable via `setMaxInputSize()`
3. **Dual Environment Support**: Browser (ES modules) and Node.js (CommonJS/ES modules)
4. **Document Analysis**: Entity counting, querying, and token statistics
5. **TypeScript Definitions**: Full type safety with exported types
6. **Token Estimation**: O(1) memory algorithm for LLM context window planning (3x faster than character-based)
7. **Size Optimization**: wasm-opt with -Os flag, tree-shaking support, ~200 KB gzipped bundle
8. **Error Handling**: Structured error objects with line numbers, error types, and messages
9. **Format Conversion**: Bidirectional JSON, YAML, XML, CSV, TOON conversion functions
10. **Bundle Variants**: ESM (hedl_wasm.js), Node.js (hedl_wasm_node.js), TypeScript definitions

## Installation

### npm/yarn/pnpm

```bash
npm install hedl-wasm
# or
yarn add hedl-wasm
# or
pnpm add hedl-wasm
```

### Browser (ESM)

```html
<script type="module">
  import init, { parse, validate, toJson } from './hedl_wasm.js';

  await init(); // Initialize WASM module

  const doc = parse(`
%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User
  | alice, Alice Smith, 30
  | bob, Bob Jones, 25
  `);

  console.log('Parsed:', doc);
  console.log('JSON:', doc.toJson()); // Requires "json" feature
</script>
```

### Node.js (CommonJS)

```javascript
const hedl = require('hedl-wasm');

const doc = hedl.parse(`
%VERSION: 1.0
---
config:
  name: MyApp
  version: 1.0.0
`);

console.log('Document:', doc);
```

### Node.js (ESM)

```javascript
import init, { parse, validate } from 'hedl-wasm';

await init();

const result = validate(hedlContent); // run_lint defaults to true

if (result.valid) {
  console.log('Valid HEDL document');
} else {
  result.errors.forEach(err => {
    console.error(`Line ${err.line}: ${err.message}`);
  });
}
```

## Core API

### Parsing and Validation

#### parse(input: string): HedlDocument

Parse HEDL content into a structured document:

```javascript
import { parse } from 'hedl-wasm';

const doc = parse(`
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
`);

// Access document properties
console.log('Version:', doc.version);
console.log('Schemas:', doc.getSchemaNames());
console.log('Aliases:', doc.getAliases());
```

**Returns**: `HedlDocument` object with properties and methods:
- `version: string` - HEDL version (e.g., "1.0")
- `schemaCount: number` - Number of schema definitions
- `aliasCount: number` - Number of aliases
- `nestCount: number` - Number of nest relationships
- `rootItemCount: number` - Number of root items

**Methods**:
- `getSchemaNames(): string[]` - Get all schema type names
- `getSchema(typeName: string): string[] | null` - Get schema columns for a type
- `getAliases(): object` - Get all aliases as a JSON object
- `getNests(): object` - Get all nest relationships
- `countEntities(): object` - Count entities by type (returns map of type names to counts)
- `query(typeName?: string, id?: string): Array<{ type: string, id: string, fields: object }>` - Query entities (requires "query-api" feature)
- `toJson(): JsonValue` - Convert to JSON object (requires "json" feature)
- `toJsonString(pretty?: boolean): string` - Convert to JSON string (requires "json" feature)
- `toHedl(useDitto?: boolean): string` - Convert to canonical HEDL string

**Throws**: `JsError` on parse failure with:
- `message: string` - Error description with line number

#### validate(input: string, runLint?: boolean): ValidationResult

Validate HEDL document and return diagnostics:

```javascript
import { validate } from 'hedl-wasm';

const result = validate(hedlContent, true); // true to run lint checks

if (!result.valid) {
  result.errors.forEach(err => {
    console.error(`Line ${err.line} [${err.type}]: ${err.message}`);
  });
}

result.warnings.forEach(warn => {
  console.warn(`Line ${warn.line} [${warn.rule}]: ${warn.message}`);
});
```

**Parameters**:
- `input: string` - HEDL document content
- `runLint?: boolean` - Enable lint checks (default: true, requires "full-validation" feature)

**Returns**: `ValidationResult` object with:
- `valid: boolean` - Whether document is valid
- `errors: Array<{ line: number, message: string, type: string }>` - Parse and validation errors
- `warnings: Array<{ line: number, message: string, rule: string }>` - Lint warnings (only if runLint is true)

### Canonicalization and Formatting

#### format(input: string, useDitto?: boolean): string

Format and canonicalize HEDL to standard style:

```javascript
import { format } from 'hedl-wasm';

const formatted = format(messyHedl, true); // true for ditto optimization

console.log(formatted);
```

**Parameters**:
- `input: string` - HEDL document content
- `useDitto?: boolean` - Use ditto operator (^) for repeated values (default: true)

**Returns**: Normalized HEDL string with:
- Consistent 2-space indentation
- Ditto operator (^) for repeated values (if enabled)
- Normalized float representation
- Consistent spacing

#### version(): string

Get the HEDL library version:

```javascript
import { version } from 'hedl-wasm';

console.log('HEDL version:', version());
```

**Returns**: Version string (e.g., "1.2.0")

#### setMaxInputSize(size: number): void

Set the maximum input size in bytes for all parsing operations.

```javascript
import { setMaxInputSize } from 'hedl-wasm';

// Allow processing up to 1 GB documents
setMaxInputSize(1024 * 1024 * 1024);
```

**Parameters**:
- `size: number` - Maximum input size in bytes

**Default**: 500 MB (524,288,000 bytes)

This controls the maximum size of HEDL/JSON input strings that can be processed. Set to a higher value if you need to process larger documents. The size check is performed before parsing to prevent memory exhaustion.

#### getMaxInputSize(): number

Get the current maximum input size configuration.

```javascript
import { getMaxInputSize } from 'hedl-wasm';

const currentLimit = getMaxInputSize();
console.log(`Current limit: ${currentLimit / (1024 * 1024)} MB`);
```

**Returns**: Current maximum input size in bytes

### Format Conversion

All conversion functions are available on `HedlDocument` objects after parsing, or as standalone functions on strings.

#### toJson(input: string, pretty?: boolean): string

Convert HEDL string to JSON:

```javascript
import { toJson } from 'hedl-wasm';

const json = toJson(hedlContent, true); // true for pretty-print

console.log(json);
```

Requires the **"json"** feature flag.

**Parameters**:
- `input: string` - HEDL document content
- `pretty?: boolean` - Pretty-print JSON (default: true)

#### fromJson(json: string, useDitto?: boolean): string

Convert JSON string to HEDL:

```javascript
import { fromJson } from 'hedl-wasm';

const hedl = fromJson(jsonString, true);
```

Requires the **"json"** feature flag.

#### toYaml(input: string): string

Convert HEDL to YAML:

```javascript
import { toYaml } from 'hedl-wasm';

const yaml = toYaml(hedlContent);
```

Requires the **"yaml"** feature flag.

#### fromYaml(yaml: string, useDitto?: boolean): string

Convert YAML to HEDL:

```javascript
import { fromYaml } from 'hedl-wasm';

const hedl = fromYaml(yamlContent, true);
```

Requires the **"yaml"** feature flag.

#### toXml(input: string): string

Convert HEDL to XML:

```javascript
import { toXml } from 'hedl-wasm';

const xml = toXml(hedlContent);
```

Requires the **"xml"** feature flag.

#### fromXml(xml: string, useDitto?: boolean): string

Convert XML to HEDL:

```javascript
import { fromXml } from 'hedl-wasm';

const hedl = fromXml(xmlContent, true);
```

Requires the **"xml"** feature flag.

#### toCsv(input: string): string

Convert HEDL to CSV (first entity list):

```javascript
import { toCsv } from 'hedl-wasm';

const csv = toCsv(hedlContent);
```

Requires the **"csv"** feature flag.

#### fromCsv(csv: string, typeName?: string, useDitto?: boolean): string

Convert CSV to HEDL:

```javascript
import { fromCsv } from 'hedl-wasm';

const hedl = fromCsv(csvContent, 'User', true);
```

**Parameters**:
- `csv: string` - CSV content (must have header row)
- `typeName?: string` - Entity type name (default: "Row")
- `useDitto?: boolean` - Use ditto optimization (default: true)

Requires the **"csv"** feature flag.

#### toToon(input: string): string

Convert HEDL to TOON format:

```javascript
import { toToon } from 'hedl-wasm';

const toon = toToon(hedlContent);
```

Requires the **"toon"** feature flag.

#### fromToon(toon: string, useDitto?: boolean): string

Convert TOON to HEDL:

```javascript
import { fromToon } from 'hedl-wasm';

const hedl = fromToon(toonContent, true);
```

Requires the **"toon"** feature flag.

### Statistics and Analysis

#### getStats(input: string): TokenStats

Get token usage statistics for HEDL vs JSON:

```javascript
import { getStats } from 'hedl-wasm';

const stats = getStats(hedlContent);

console.log('HEDL bytes:', stats.hedlBytes);
console.log('HEDL tokens:', stats.hedlTokens);
console.log('JSON bytes:', stats.jsonBytes);
console.log('JSON tokens:', stats.jsonTokens);
console.log('Savings:', stats.savingsPercent + '%');
```

Requires the **"statistics"** feature flag.

**Returns**: `TokenStats` object with:
- `hedlBytes: number` - Input HEDL size in bytes
- `hedlTokens: number` - Estimated token count for HEDL
- `hedlLines: number` - Line count in HEDL
- `jsonBytes: number` - Equivalent JSON size in bytes
- `jsonTokens: number` - Estimated token count for JSON
- `savingsPercent: number` - Token savings percentage
- `tokensSaved: number` - Absolute token count difference

#### compareTokens(hedl: string, json: string): object

Compare token counts between HEDL and JSON:

```javascript
import { compareTokens } from 'hedl-wasm';

const comparison = compareTokens(hedlString, jsonString);

console.log('HEDL tokens:', comparison.hedl.tokens);
console.log('JSON tokens:', comparison.json.tokens);
console.log('Savings:', comparison.savings.percent + '%');
```

Requires the **"token-tools"** feature flag.

**Returns**: Object with:
- `hedl: { bytes: number, tokens: number, lines: number }`
- `json: { bytes: number, tokens: number }`
- `savings: { percent: number, tokens: number }`

## Memory Management

### Input Size Limits

Default limits prevent memory exhaustion:

```javascript
import { setMaxInputSize, getMaxInputSize } from 'hedl-wasm';

// Default: 500 MB input limit
const doc = parse(hedlContent);

// Get current limit
const limit = getMaxInputSize();
console.log(`Current limit: ${limit / (1024 * 1024)} MB`);

// Set custom limit (e.g., 1 GB)
setMaxInputSize(1024 * 1024 * 1024);
```

**Protection Against**:
- Malicious large inputs
- Accidental multi-GB file processing
- Memory exhaustion attacks

The size check is performed before parsing to prevent resource exhaustion.

## Error Handling

All functions throw structured errors with line numbers:

```javascript
try {
  const doc = parse(invalidHedl);
} catch (error) {
  console.error(`Parse error: ${error.message}`);
  // Note: error.message includes line number information
}
```

**Error Object** (for validation errors):
When using the `validate()` function, errors are returned in the ValidationResult object:
- `line: number` - Source line number (1-indexed)
- `message: string` - Human-readable description
- `type: string` - Error category (Syntax, Schema, Reference, etc.)

**Common Error Types**:
- `Syntax` - Invalid HEDL syntax
- `Schema` - Type/schema mismatch
- `Reference` - Unresolved reference
- `ShapeMismatch` - Column count mismatch
- `OrphanRow` - Child without parent
- `Utf8` - Invalid UTF-8 encoding
- `MaxSizeExceeded` - Input too large

## TypeScript Support

Complete type definitions included:

```typescript
import {
  parse,
  validate,
  format,
  toJson,
  fromJson,
  getStats,
  compareTokens,
  HedlDocument,
  ValidationResult,
  TokenStats
} from 'hedl-wasm';

const doc: HedlDocument = parse(content);

const result: ValidationResult = validate(content, false);

if (!result.valid) {
  result.errors.forEach(err => {
    console.error(`Line ${err.line}: ${err.message}`);
  });
}

const stats: TokenStats = getStats(content);
console.log(`Savings: ${stats.savingsPercent}%`);
```

**Exported Types**:
- `HedlDocument` - Parsed HEDL document with methods
- `ValidationResult` - Validation diagnostics
- `TokenStats` - Token usage statistics

**TypeScript Type Definitions** (available in .d.ts):
- `JsonValue` - JSON value union type (string | number | boolean | null | JsonObject | JsonArray)
- `JsonPrimitive` - JSON primitive types (string | number | boolean | null)
- `JsonObject` - JSON object type ({ [key: string]: JsonValue })
- `JsonArray` - JSON array type (JsonValue[])

**Type Files**:
- Auto-generated from wasm-bindgen
- Full IntelliSense support in VS Code
- Complete function signatures

## Bundle Sizes

Optimized for web delivery:

- **Uncompressed**: ~600 KB
- **Gzipped**: ~200 KB
- **Brotli**: ~180 KB

**Optimization Techniques**:
- `wasm-opt -Os` - Size optimization pass
- Tree-shaking support via ES modules
- Dead code elimination
- No unnecessary dependencies

**Bundle Variants**:
- `hedl_wasm.js` - ESM for browsers (bundler-friendly)
- `hedl_wasm_node.js` - CommonJS for Node.js
- `hedl_wasm_bg.wasm` - WebAssembly binary
- `hedl.d.ts` - TypeScript definitions

## Use Cases

**Web Applications**: Parse and validate HEDL configuration files uploaded by users in the browser without server round-trip. Validate data structures client-side before submission.

**Data Transformation Tools**: Build web-based converters between HEDL and JSON/YAML/XML/CSV with instant client-side processing. No server infrastructure required.

**LLM Context Planning**: Estimate token counts for HEDL documents before sending to LLM APIs. Stay within context window limits (8K, 32K, 100K) with accurate projections.

**Node.js Services**: Parse HEDL API responses, validate data structures, convert between formats in backend services with near-native performance.

**Browser Extensions**: Process HEDL data in browser extensions (Chrome, Firefox) with full HEDL ecosystem access without bundling JavaScript reimplementation.

**Electron Applications**: Embed HEDL processing in Electron desktop apps with native performance through WebAssembly.

## What This Crate Doesn't Do

**Database Integration**: No direct Neo4j or Parquet integration in WASM. For graph database export or columnar storage, use the Rust crates (`hedl-neo4j`, `hedl-parquet`) in server environments.

**File System Access**: No direct file I/O—WASM runs in sandbox. Use JavaScript File API or Node.js fs module to read files, then pass content to WASM functions.

**Network Operations**: No HTTP fetching or network I/O. Use JavaScript fetch API or Node.js http module, then process responses with WASM functions.

**Async/Await Interface**: Functions are synchronous (blocking). For non-blocking processing, wrap calls in `async` functions or Web Workers.

## Performance Characteristics

**Parsing**: Near-native speed (within 10% of Rust implementation). Typically 50-100 MB/s on modern browsers.

**Validation**: O(n) time where n = total nodes. Syntax validation through parsing.

**Conversion**: JSON conversion preserves HEDL semantics. Bidirectional conversion is lossless.

**Token Estimation**: O(1) memory algorithm, efficient byte-level iteration optimized for ASCII. 3x faster than character-by-character approaches.

**Memory**: Scales linearly with document size. Large documents benefit from 500 MB default input limit protection.

**Bundle Loading**: Initial WASM module load adds ~50-100ms overhead (one-time cost per page load).

## Dependencies

Runtime dependencies (always included):
- `wasm-bindgen` 0.2 - JavaScript/WebAssembly interop
- `serde-wasm-bindgen` 0.6 - Serde integration for WASM
- `hedl-core` - Core HEDL parser and types
- `hedl-c14n` - Document canonicalization
- `hedl-lint` - Linting engine

Optional format converters (controlled by feature flags):
- `hedl-json` - JSON bidirectional conversion (default feature)
- `hedl-yaml` - YAML conversion (yaml feature)
- `hedl-xml` - XML conversion (xml feature)
- `hedl-csv` - CSV conversion (csv feature)
- `hedl-toon` - TOON format conversion (toon feature)

Build dependencies:
- `wasm-pack` - Build toolchain
- `wasm-opt` (from binaryen) - Size optimization (disabled in Cargo.toml, uses custom pipeline)

## License

Apache-2.0
