# WASM/JavaScript API Reference

**WebAssembly bindings for browser and Node.js environments**

---

## Quick Start

### Installation

```bash
npm install hedl-wasm
```

### Browser Usage

```typescript
import init, {
    parse,
    toJson, fromJson,
    toYaml, fromYaml,               // Requires 'yaml' feature
    toXml, fromXml,                 // Requires 'xml' feature
    toCsv, fromCsv,                 // Requires 'csv' feature
    toToon, fromToon,               // Requires 'toon' feature
    validate, format,
    getStats,                       // Requires 'statistics' feature
    compareTokens,                  // Requires 'token-tools' feature
    setMaxInputSize, getMaxInputSize,
    version
} from 'hedl-wasm';

// Initialize the WASM module
await init();

// Parse HEDL
const hedl = `
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |alice,Alice Smith, alice@example.com
`;

const doc = parse(hedl);
console.log(`Version: ${doc.version}`);
console.log(`Schemas: ${doc.schemaCount}`);

// Convert to JSON string
const json = toJson(hedl, true);
console.log(json);

// Validate
const result = validate(hedl, true);
if (result.valid) {
    console.log('Valid HEDL!');
} else {
    console.error('Errors:', result.errors);
}
```

### Node.js Usage

```javascript
const hedl = require('hedl-wasm');

async function main() {
    await hedl.default();  // Initialize

    const doc = hedl.parse('%V:2.0\n---\nkey: value');
    console.log(doc.version);
}

main();
```

---

## Configuration

### Input Size Limits

Default maximum input size is **500 MB**. Configure as needed:

```javascript
import { setMaxInputSize, getMaxInputSize } from 'hedl-wasm';

// Allow 1 GB documents
setMaxInputSize(1024 * 1024 * 1024);

// Check current limit
const limit = getMaxInputSize();
console.log(`Current limit: ${limit / (1024 * 1024)} MB`);
```

---

## Core Functions

### Parsing

#### `parse`

Parse a HEDL document and return a document object.

```typescript
function parse(input: string): HedlDocument
```

**Parameters**:
- `input`: HEDL document string

**Returns**: `HedlDocument` object

**Throws**: Error if parsing fails or input exceeds size limit

**Example**:
```typescript
try {
    const doc = parse(hedlString);
    console.log(`Parsed ${doc.rootItemCount} items`);
} catch (e) {
    console.error(`Parse error: ${e.message}`);
}
```

---

### Document Object

#### `HedlDocument`

Represents a parsed HEDL document.

**Properties & Methods**:
```typescript
interface HedlDocument {
    // Read-only properties
    readonly version: string;           // e.g., "2.0"
    readonly schemaCount: number;       // Number of STRUCT definitions
    readonly aliasCount: number;        // Number of ALIAS definitions
    readonly nestCount: number;         // Number of NEST definitions
    readonly rootItemCount: number;     // Number of root items

    // Methods for schema inspection
    getSchemaNames(): string[];
    getSchema(typeName: string): string[] |undefined;
    getAliases(): Record<string, string>;      // Returns alias mappings
    getNests(): Record<string, string[]>;      // Returns nest mappings (parent -> child types)

    // Entity operations
    countEntities(): { [typeName: string]: number };  // Count entities by type

    // Format conversion methods (standalone functions also available)
    toHedl(): string;                                        // Convert to HEDL string
    toJson(): JsonValue;                                    // Requires: json feature
    toJsonString(pretty?: boolean): string;                 // Requires: json feature

    // Entity querying
    query(typeName?: string, id?: string): EntityResult[];  // Requires: query-api feature
}
```

**Example**:
```typescript
const doc = parse(hedl);

// Get schemas
const schemas = doc.getSchemaNames();
console.log('Schemas:', schemas);

const userSchema = doc.getSchema('User');
console.log('User fields:', userSchema);

// Convert to JSON (method on document)
const json = doc.toJson();
console.log(json);

// Count entities
const counts = doc.countEntities();
console.log('Entity counts:', counts);

// Convert document to HEDL
const hedlStr = doc.toHedl(true);

// Convert document to JSON (method on document OR standalone function)
const jsonObj = doc.toJson();                    // Returns JsonValue object
const jsonStr = doc.toJsonString(true);         // Returns JSON string

// Standalone format conversion functions (take HEDL string as input)
const yaml = toYaml(hedl);     // Requires: yaml feature
const xml = toXml(hedl);       // Requires: xml feature
const csv = toCsv(hedl);       // Requires: csv feature
const toon = toToon(hedl);     // Requires: toon feature
```

---

#### `query`

Query entities by type and/or ID.

> **Feature Required**: This method requires the `query-api` build feature. It is not available in default builds.

```typescript
query(typeName?: string, id?: string): EntityResult[]
```

**Parameters**:
- `typeName`: Optional type filter (e.g., "User")
- `id`: Optional ID filter

**Returns**: Array of matching entities

```typescript
interface EntityResult {
    type: string;
    id: string;
    fields: Record<string, JsonValue>;
}
```

**Example**:
```typescript
// Find all users
const users = doc.query('User');

// Find specific user
const alice = doc.query('User', 'alice');

// Find all entities
const all = doc.query();
```

---

### Document Instance Methods

The `HedlDocument` class provides these instance methods for converting the parsed document:

#### `HedlDocument.toJson()` (instance method)

Convert the parsed document to a JSON object.

> **Feature Required**: This method requires the `json` build feature.

```typescript
toJson(): JsonValue
```

**Returns**: `JsonValue` (can be an object, array, or primitive)

**Example**:
```typescript
const doc = parse(hedl);
const jsonObj = doc.toJson();
console.log(jsonObj);  // JavaScript object/array/value
```

---

#### `HedlDocument.toJsonString()` (instance method)

Convert the parsed document to a JSON string.

> **Feature Required**: This method requires the `json` build feature.

```typescript
toJsonString(pretty?: boolean): string
```

**Parameters**:
- `pretty`: Pretty-print output (default: `true`)

**Returns**: JSON string

**Example**:
```typescript
const doc = parse(hedl);
const jsonStr = doc.toJsonString(true);
console.log(jsonStr);  // String with pretty formatting
```

---

#### `HedlDocument.toHedl()` (instance method)

Convert the parsed document back to canonical HEDL format.

```typescript
toHedl(): string
```

**Returns**: Canonical HEDL string

**Example**:
```typescript
const doc = parse(hedl);
const canonical = doc.toHedl(true);
console.log(canonical);  // Reformatted HEDL
```

---

### Standalone Format Conversion Functions

These functions take a HEDL string as input and return the converted format. Use these instead of instance methods when you haven't parsed the document yet.

#### `toJson(hedl, pretty)` (standalone function)

Convert HEDL string to JSON string.

```typescript
function toJson(hedl: string, pretty?: boolean): string
```

**Parameters**:
- `hedl`: HEDL document string
- `pretty`: Pretty-print output (default: `true`)

**Returns**: JSON string

**Throws**: Error if parsing or conversion fails

**Example**:
```typescript
const json = toJson(hedl, true);
console.log(json);
```

---

#### `fromJson(json)` (standalone function)

Convert JSON string to HEDL string.

```typescript
function fromJson(json: string): string
```

**Parameters**:
- `json`: JSON string

**Returns**: HEDL string

**Example**:
```typescript
const jsonData = '{"users": [{"id": "alice", "name": "Alice"}]}';
const hedl = fromJson(jsonData, true);
console.log(hedl);
```

---

#### `toYaml(hedl)` (standalone function)

Convert HEDL string to YAML string.

> **Feature Required**: This function requires the `yaml` build feature.

```typescript
function toYaml(hedl: string): string
```

**Parameters**:
- `hedl`: HEDL document string

**Returns**: YAML string

**Throws**: Error if parsing or conversion fails

**Example**:
```typescript
const yaml = toYaml(hedl);
console.log(yaml);
```

---

#### `fromYaml(yaml)` (standalone function)

Convert YAML string to HEDL string.

> **Feature Required**: This function requires the `yaml` build feature.

```typescript
function fromYaml(yaml: string): string
```

**Parameters**:
- `yaml`: YAML string

**Returns**: HEDL string

**Example**:
```typescript
const yamlData = 'users:\n  - id: alice\n    name: Alice';
const hedl = fromYaml(yamlData, true);
console.log(hedl);
```

---

#### `toXml(hedl)` (standalone function)

Convert HEDL string to XML string.

> **Feature Required**: This function requires the `xml` build feature.

```typescript
function toXml(hedl: string): string
```

**Parameters**:
- `hedl`: HEDL document string

**Returns**: XML string

**Throws**: Error if parsing or conversion fails

**Example**:
```typescript
const xml = toXml(hedl);
console.log(xml);
```

---

#### `fromXml(xml)` (standalone function)

Convert XML string to HEDL string.

> **Feature Required**: This function requires the `xml` build feature.

```typescript
function fromXml(xml: string): string
```

**Parameters**:
- `xml`: XML string

**Returns**: HEDL string

**Example**:
```typescript
const xmlData = '<root><user><id>alice</id><name>Alice</name></user></root>';
const hedl = fromXml(xmlData, true);
console.log(hedl);
```

---

#### `toCsv(hedl)` (standalone function)

Convert HEDL string to CSV string.

> **Feature Required**: This function requires the `csv` build feature.

```typescript
function toCsv(hedl: string): string
```

**Parameters**:
- `hedl`: HEDL document string

**Returns**: CSV string

**Throws**: Error if parsing or conversion fails

**Example**:
```typescript
const csv = toCsv(hedl);
console.log(csv);
```

---

#### `fromCsv(csv, typeName)` (standalone function)

Convert CSV string to HEDL string.

> **Feature Required**: This function requires the `csv` build feature.

```typescript
function fromCsv(csv: string, typeName?: string): string
```

**Parameters**:
- `csv`: CSV string (header row required)
- `typeName`: Type name for entities (default: `"Row"`)

**Returns**: HEDL string

**Example**:
```typescript
const csvData = 'id,name,email\nalice,Alice,alice@example.com';
const hedl = fromCsv(csvData, 'User', true);
console.log(hedl);
```

---

#### `toToon(hedl)` (standalone function)

Convert HEDL string to TOON (Typed Object Outline Notation) string.

> **Feature Required**: This function requires the `toon` build feature.

```typescript
function toToon(hedl: string): string
```

**Parameters**:
- `hedl`: HEDL document string

**Returns**: TOON string

**Throws**: Error if parsing or conversion fails

**Example**:
```typescript
const toon = toToon(hedl);
console.log(toon);
```

---

#### `fromToon(toon)` (standalone function)

Convert TOON string to HEDL string.

> **Feature Required**: This function requires the `toon` build feature.

```typescript
function fromToon(toon: string): string
```

**Parameters**:
- `toon`: TOON string

**Returns**: HEDL string

**Example**:
```typescript
const toonData = 'users\n\talice\n\t\tname\tAlice';
const hedl = fromToon(toonData, true);
console.log(hedl);
```

---

#### `format(hedl)` (standalone function)

Format HEDL to canonical form.

```typescript
function format(hedl: string): string
```

**Parameters**:
- `hedl`: HEDL document string

**Returns**: Formatted HEDL string

**Example**:
```typescript
const formatted = format(messyHedl, true);
console.log(formatted);
```

---

### Validation & Diagnostics

#### `validate(hedl, runLint)` (standalone function)

Validate HEDL and return detailed diagnostics.

```typescript
function validate(hedl: string, runLint?: boolean): ValidationResult
```

**Parameters**:
- `hedl`: HEDL document string
- `runLint`: Run linting rules (default: `true`)

> **Note**: The `runLint` parameter only enables full linting when the `full-validation` build feature is enabled. Without this feature, only syntax validation is performed regardless of the `runLint` value.

**Returns**: Validation result object

```typescript
interface ValidationResult {
    valid: boolean;
    errors: ValidationError[];
    warnings: ValidationWarning[];
}

interface ValidationError {
    line: number;
    message: string;
    type: string;
}

interface ValidationWarning {
    line: number;
    message: string;
    rule: string;
}
```

**Example**:
```typescript
const result = validate(hedl, true);

if (!result.valid) {
    result.errors.forEach(err => {
        console.error(`Line ${err.line}: ${err.message}`);
    });
}

result.warnings.forEach(warn => {
    console.warn(`Line ${warn.line}: ${warn.message} [${warn.rule}]`);
});
```

---

### Statistics & Token Analysis

#### `getStats(hedl)` (standalone function)

Get token usage statistics for HEDL document.

> **Feature Required**: This function requires the `statistics` build feature. It is not available in default builds.

```typescript
function getStats(hedl: string): TokenStats
```

**Parameters**:
- `hedl`: HEDL document string

**Returns**: Token statistics

```typescript
interface TokenStats {
    hedlBytes: number;      // Bytes in HEDL representation
    hedlTokens: number;     // Estimated tokens in HEDL
    hedlLines: number;      // Number of lines in HEDL
    jsonBytes: number;      // Bytes if converted to JSON
    jsonTokens: number;     // Estimated tokens in JSON equivalent
    savingsPercent: number; // Percentage saved vs JSON
    tokensSaved: number;    // Number of tokens saved
}
```

**Example**:
```typescript
const stats = getStats(hedl);
console.log(`Token savings: ${stats.savingsPercent}%`);
console.log(`HEDL: ${stats.hedlTokens} tokens`);
console.log(`JSON: ${stats.jsonTokens} tokens`);
console.log(`Saved: ${stats.tokensSaved} tokens`);
```

---

#### `compareTokens(hedl, json)` (standalone function)

Compare HEDL and JSON token counts side-by-side.

> **Feature Required**: This function requires the `token-tools` build feature. It is not available in default builds.

```typescript
function compareTokens(hedl: string, json: string): ComparisonResult
```

**Parameters**:
- `hedl`: HEDL document string
- `json`: JSON document string

**Returns**: Comparison result

```typescript
interface ComparisonResult {
    hedl: {
        bytes: number;   // Size in bytes
        tokens: number;  // Estimated tokens
        lines: number;   // Number of lines
    };
    json: {
        bytes: number;   // Size in bytes
        tokens: number;  // Estimated tokens
    };
    savings: {
        percent: number; // Percentage saved
        tokens: number;  // Tokens saved
    };
}
```

**Example**:
```typescript
const comparison = compareTokens(hedlStr, jsonStr);
console.log(`HEDL: ${comparison.hedl.tokens} tokens`);
console.log(`JSON: ${comparison.json.tokens} tokens`);
console.log(`Savings: ${comparison.savings.percent}%`);
```

---

### Utility Functions

#### `version()` (standalone function)

Get HEDL library version.

```typescript
function version(): string
```

**Returns**: Version string (e.g., "1.0.0")

**Example**:
```typescript
console.log(`HEDL version: ${version()}`);
```

---

## TypeScript Types

### JSON Types

```typescript
/**
 * Represents a JSON primitive value.
 */
export type JsonPrimitive = string |number |boolean |null;

/**
 * Represents a JSON array (recursive).
 */
export type JsonArray = JsonValue[];

/**
 * Represents a JSON object (recursive).
 */
export type JsonObject = { [key: string]: JsonValue };

/**
 * Represents any valid JSON value.
 */
export type JsonValue = JsonPrimitive |JsonObject |JsonArray;
```

---

## Error Handling

All functions throw JavaScript `Error` objects on failure:

```typescript
try {
    const doc = parse(hedl);
} catch (e) {
    if (e instanceof Error) {
        console.error(`Error: ${e.message}`);
    }
}
```

### Common Error Types

 |Error Message Pattern |Cause |
|----------------------|-------|
 |`Parse error at line N` |Syntax error in HEDL |
 |`Input size (X bytes) exceeds maximum` |Input too large |
 |`Invalid JSON` |Malformed JSON input |
 |`Conversion error` |Format conversion failed |

---

## Performance Optimization

### Token Estimation

The WASM module uses an optimized single-pass byte-level loop for token estimation:

- **Time complexity**: O(n) single pass
- **Space complexity**: O(1) constant
- **~3x faster** than multi-pass character iteration

**Formula**:
```
tokens = (byte_count + whitespace_count + punct_count) / CHARS_PER_TOKEN
```

Where `CHARS_PER_TOKEN = 4` for structured data.

---

### Memory Management

WASM memory is managed automatically by the JavaScript runtime. The module:

- Uses efficient Rust allocators
- Implements automatic cleanup via Drop traits
- Limits memory via input size constraints

---

## Browser Compatibility

Tested and working in:

- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+
- Node.js 14+

**Requirements**:
- WebAssembly support
- ES6 modules (or bundler)

---

## React Example

```typescript
import React, { useState, useEffect } from 'react';
import init, { parse, getStats } from 'hedl-wasm';

function HedlViewer() {
    const [initialized, setInitialized] = useState(false);
    const [hedl, setHedl] = useState('%V:2.0\n---\n');
    const [stats, setStats] = useState(null);

    useEffect(() => {
        init().then(() => setInitialized(true));
    }, []);

    useEffect(() => {
        if (!initialized) return;

        try {
            const s = getStats(hedl);
            setStats(s);
        } catch (e) {
            console.error(e);
        }
    }, [hedl, initialized]);

    if (!initialized) return <div>Loading...</div>;

    return (
        <div>
            <textarea
                value={hedl}
                onChange={(e) => setHedl(e.target.value)}
            />
            {stats && (
                <div>
                    <p>Tokens: {stats.hedlTokens}</p>
                    <p>Lines: {stats.hedlLines}</p>
                    <p>Savings: {stats.savingsPercent}%</p>
                </div>
            )}
        </div>
    );
}
```

---

## Vue Example

```vue
<template>
    <div>
        <textarea v-model="hedl"></textarea>
        <div v-if="stats">
            <p>Tokens: {{ stats.hedlTokens }}</p>
            <p>Savings: {{ stats.savingsPercent }}%</p>
        </div>
    </div>
</template>

<script>
import init, { getStats } from 'hedl-wasm';

export default {
    data() {
        return {
            hedl: '%V:2.0\n---\n',
            stats: null,
            initialized: false
        };
    },
    async mounted() {
        await init();
        this.initialized = true;
        this.updateStats();
    },
    watch: {
        hedl() {
            this.updateStats();
        }
    },
    methods: {
        updateStats() {
            if (!this.initialized) return;
            try {
                this.stats = getStats(this.hedl);
            } catch (e) {
                console.error(e);
            }
        }
    }
};
</script>
```

---

## Node.js CLI Example

```javascript
#!/usr/bin/env node

const hedl = require('hedl-wasm');
const fs = require('fs');

async function main() {
    await hedl.default();

    const input = fs.readFileSync(process.argv[2], 'utf-8');

    try {
        // toJson takes HEDL string, returns JSON string
        const json = hedl.toJson(input, true);
        console.log(json);
    } catch (e) {
        console.error('Error:', e.message);
        process.exit(1);
    }
}

main();
```

**Usage**:
```bash
node hedl-to-json.js input.hedl > output.json
```

---

## Webpack Configuration

```javascript
module.exports = {
    experiments: {
        asyncWebAssembly: true
    },
    module: {
        rules: [
            {
                test: /\.wasm$/,
                type: 'webassembly/async'
            }
        ]
    }
};
```

---

## Vite Configuration

```javascript
export default {
    optimizeDeps: {
        exclude: ['hedl-wasm']
    }
};
```

---

## Best Practices

### 1. Initialize Once

```typescript
// Good: Initialize at app startup
await init();

// Bad: Initialize before every operation
await init();
parse(hedl);
```

### 2. Handle Size Limits

```typescript
const MAX_SIZE = 10 * 1024 * 1024;  // 10 MB

if (input.length > MAX_SIZE) {
    throw new Error('Input too large');
}

const doc = parse(input);
```

### 3. Use Validation for User Input

```typescript
const result = validate(userInput, true);
if (!result.valid) {
    // Show errors to user
    displayErrors(result.errors);
    return;
}
```

### 4. Batch Operations

```typescript
// Process multiple documents efficiently
const docs = await Promise.all(
    inputs.map(input => parse(input))
);
```

---

**Next**: [MCP Server API Reference](mcp-api.md)
