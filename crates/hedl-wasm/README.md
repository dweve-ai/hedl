# hedl-wasm

**Parse, validate, and convert HEDL documents directly in browsers and Node.js with near-native performance.**

Every web developer has faced the same frustration: JSON is everywhere but loses type information, YAML parsers are heavy, and XML processing adds complexity. What if you could parse structured data at near-native speed, validate it with full schema checking, and convert between formats instantly, all without leaving JavaScript?

That's what `hedl-wasm` delivers. The complete Rust HEDL implementation, compiled to WebAssembly, running in your browser or Node.js environment. Parse multi-megabyte documents at 50-100 MB/s. Validate data structures client-side before they ever hit your server. Convert seamlessly between HEDL, JSON, YAML, XML, and CSV with zero compromises on correctness.

## Installation

```bash
npm install hedl-wasm
```

## Quick Start

### Browser (ESM)

```html
<script type="module">
  import init, { parse, validate, toJson } from './hedl_wasm.js';

  await init();

  const doc = parse(`
%V:2.0
%S:User:[id, name, age]
---
users: @User
 | alice, Alice Smith, 30
 | bob, Bob Jones, 25
  `);

  console.log('Parsed:', doc);
  console.log('JSON:', doc.toJson());
</script>
```

### Node.js (CommonJS)

```javascript
const hedl = require('hedl-wasm');

const doc = hedl.parse(`
%V:2.0
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

const result = validate(hedlContent);

if (result.valid) {
  console.log('Valid HEDL document');
} else {
  result.errors.forEach(err => {
    console.error(`Line ${err.line}: ${err.message}`);
  });
}
```

## Core API

### parse(input: string): HedlDocument

Parse HEDL content into a structured document.

```javascript
import { parse } from 'hedl-wasm';

const doc = parse(`
%V:2.0
%S:User:[id, name, email]
---
users: @User
 | alice, Alice Smith, alice@example.com
 | bob, Bob Jones, bob@example.com
`);

console.log('Version:', doc.version);
console.log('Schemas:', doc.getSchemaNames());
console.log('Aliases:', doc.getAliases());
```

**Returns**: `HedlDocument` with these properties:
- `version: string` - HEDL version (e.g., "2.0")
- `schemaCount: number` - Number of schema definitions
- `aliasCount: number` - Number of aliases
- `nestCount: number` - Number of nest relationships
- `rootItemCount: number` - Number of root items

**Methods**:
- `getSchemaNames(): string[]` - Get all schema type names
- `getSchema(typeName: string): string[] | null` - Get schema columns for a type
- `getAliases(): object` - Get all aliases as a JSON object
- `getNests(): object` - Get all nest relationships
- `countEntities(): object` - Count entities by type
- `query(typeName?: string, id?: string): Array<{ type: string, id: string, fields: object }>` - Query entities (requires "query-api" feature)
- `toJson(): JsonValue` - Convert to JSON object (requires "json" feature)
- `toJsonString(pretty?: boolean): string` - Convert to JSON string (requires "json" feature)
- `toHedl(): string` - Convert to canonical HEDL string

**Throws**: `JsError` on parse failure with line number information in the message.

### validate(input: string, runLint?: boolean): ValidationResult

Validate HEDL documents and return detailed diagnostics.

```javascript
import { validate } from 'hedl-wasm';

const result = validate(hedlContent, true);

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

**Returns**: `ValidationResult` with:
- `valid: boolean` - Whether document is valid
- `errors: Array<{ line: number, message: string, type: string }>` - Parse and validation errors
- `warnings: Array<{ line: number, message: string, rule: string }>` - Lint warnings

### format(input: string): string

Format and canonicalize HEDL documents.

```javascript
import { format } from 'hedl-wasm';

const formatted = format(messyHedl);
```

Returns normalized HEDL with consistent indentation, float representation, and spacing.

### version(): string

Get the HEDL library version.

```javascript
import { version } from 'hedl-wasm';

console.log('HEDL version:', version());
```

### setMaxInputSize(size: number): void

Configure the maximum input size in bytes for all parsing operations.

```javascript
import { setMaxInputSize } from 'hedl-wasm';

// Allow processing up to 1 GB documents
setMaxInputSize(1024 * 1024 * 1024);
```

Default is 500 MB. The size check runs before parsing to prevent memory exhaustion.

### getMaxInputSize(): number

Get the current maximum input size configuration.

```javascript
import { getMaxInputSize } from 'hedl-wasm';

const currentLimit = getMaxInputSize();
console.log(`Current limit: ${currentLimit / (1024 * 1024)} MB`);
```

## Format Conversion

All conversion functions work both as standalone functions on strings and as methods on `HedlDocument` objects.

### toJson(input: string, pretty?: boolean): string

Convert HEDL to JSON. Requires the **"json"** feature flag.

```javascript
import { toJson } from 'hedl-wasm';

const json = toJson(hedlContent, true); // true for pretty-print
```

### fromJson(json: string): string

Convert JSON to HEDL. Requires the **"json"** feature flag.

```javascript
import { fromJson } from 'hedl-wasm';

const hedl = fromJson(jsonString);
```

### toYaml(input: string): string

Convert HEDL to YAML. Requires the **"yaml"** feature flag.

```javascript
import { toYaml } from 'hedl-wasm';

const yaml = toYaml(hedlContent);
```

### fromYaml(yaml: string): string

Convert YAML to HEDL. Requires the **"yaml"** feature flag.

```javascript
import { fromYaml } from 'hedl-wasm';

const hedl = fromYaml(yamlContent);
```

### toXml(input: string): string

Convert HEDL to XML. Requires the **"xml"** feature flag.

```javascript
import { toXml } from 'hedl-wasm';

const xml = toXml(hedlContent);
```

### fromXml(xml: string): string

Convert XML to HEDL. Requires the **"xml"** feature flag.

```javascript
import { fromXml } from 'hedl-wasm';

const hedl = fromXml(xmlContent);
```

### toCsv(input: string): string

Convert HEDL to CSV (first entity list). Requires the **"csv"** feature flag.

```javascript
import { toCsv } from 'hedl-wasm';

const csv = toCsv(hedlContent);
```

### fromCsv(csv: string, typeName?: string): string

Convert CSV to HEDL. Requires the **"csv"** feature flag.

```javascript
import { fromCsv } from 'hedl-wasm';

const hedl = fromCsv(csvContent, 'User');
```

**Parameters**:
- `csv: string` - CSV content (must have header row)
- `typeName?: string` - Entity type name (default: "Row")

### toToon(input: string): string

Convert HEDL to TOON format. Requires the **"toon"** feature flag.

```javascript
import { toToon } from 'hedl-wasm';

const toon = toToon(hedlContent);
```

### fromToon(toon: string): string

Convert TOON to HEDL. Requires the **"toon"** feature flag.

```javascript
import { fromToon } from 'hedl-wasm';

const hedl = fromToon(toonContent);
```

## Statistics and Analysis

### getStats(input: string): TokenStats

Get token usage statistics comparing HEDL to JSON. Requires the **"statistics"** feature flag.

```javascript
import { getStats } from 'hedl-wasm';

const stats = getStats(hedlContent);

console.log('HEDL bytes:', stats.hedlBytes);
console.log('HEDL tokens:', stats.hedlTokens);
console.log('JSON bytes:', stats.jsonBytes);
console.log('JSON tokens:', stats.jsonTokens);
console.log('Savings:', stats.savingsPercent + '%');
```

**Returns**: `TokenStats` with:
- `hedlBytes: number` - Input HEDL size in bytes
- `hedlTokens: number` - Estimated token count for HEDL
- `hedlLines: number` - Line count in HEDL
- `jsonBytes: number` - Equivalent JSON size in bytes
- `jsonTokens: number` - Estimated token count for JSON
- `savingsPercent: number` - Token savings percentage
- `tokensSaved: number` - Absolute token count difference

### compareTokens(hedl: string, json: string): object

Compare token counts between HEDL and JSON. Requires the **"token-tools"** feature flag.

```javascript
import { compareTokens } from 'hedl-wasm';

const comparison = compareTokens(hedlString, jsonString);

console.log('HEDL tokens:', comparison.hedl.tokens);
console.log('JSON tokens:', comparison.json.tokens);
console.log('Savings:', comparison.savings.percent + '%');
```

**Returns**: Object with:
- `hedl: { bytes: number, tokens: number, lines: number }`
- `json: { bytes: number, tokens: number }`
- `savings: { percent: number, tokens: number }`

## Error Handling

All functions throw structured errors with line number information.

```javascript
try {
  const doc = parse(invalidHedl);
} catch (error) {
  console.error(`Parse error: ${error.message}`);
}
```

When using `validate()`, errors are returned in the `ValidationResult` object with:
- `line: number` - Source line number (1-indexed)
- `message: string` - Human-readable description
- `type: string` - Error category

**Error Types**:
- `Syntax` - Invalid HEDL syntax
- `Schema` - Type/schema mismatch
- `Reference` - Unresolved reference
- `ShapeMismatch` - Column count mismatch
- `OrphanRow` - Child without parent
- `Utf8` - Invalid UTF-8 encoding
- `MaxSizeExceeded` - Input too large

## TypeScript Support

Complete type definitions are included for full IntelliSense support.

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
- `JsonValue` - JSON value union type
- `JsonPrimitive` - JSON primitive types
- `JsonObject` - JSON object type
- `JsonArray` - JSON array type

## Bundle Sizes

Optimized for web delivery with `wasm-opt -Os`, tree-shaking support, and dead code elimination.

| Format | Size |
|--------|------|
| Uncompressed | ~600 KB |
| Gzipped | ~200 KB |
| Brotli | ~180 KB |

**Bundle Variants**:
- `hedl_wasm.js` - ESM for browsers
- `hedl_wasm_node.js` - CommonJS for Node.js
- `hedl_wasm_bg.wasm` - WebAssembly binary
- `hedl.d.ts` - TypeScript definitions

## Performance

Parsing runs at near-native speed, typically within 10% of the pure Rust implementation. Expect 50-100 MB/s throughput on modern browsers. Token estimation uses an O(1) memory algorithm with efficient byte-level iteration, running 3x faster than character-by-character approaches.

Memory scales linearly with document size. Initial WASM module load adds approximately 50-100ms overhead, a one-time cost per page load.

## Building from Source

### Prerequisites

```bash
cargo install wasm-pack
cargo install wasm-tools
cargo install wasm-bindgen-cli --version 0.2.108
```

### Build

```bash
cd crates/hedl-wasm

# Build for browsers
./build-wasm.sh web

# Build for bundlers (webpack, etc.)
./build-wasm.sh bundler

# Build for Node.js
./build-wasm.sh nodejs
```

Output is placed in `pkg/`:
- `hedl_wasm.js` - JavaScript glue code
- `hedl_wasm.d.ts` - TypeScript definitions
- `hedl_wasm_bg.wasm` - WebAssembly binary

## License

Apache-2.0
