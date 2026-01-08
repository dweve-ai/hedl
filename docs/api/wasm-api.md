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
import init, { parse, toJson, fromJson, validate } from 'hedl-wasm';

// Initialize the WASM module
await init();

// Parse HEDL
const hedl = `
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
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

    const doc = hedl.parse('%VERSION: 1.0\n---\nkey: value');
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

**Properties**:
```typescript
interface HedlDocument {
    // Properties
    readonly version: string;           // e.g., "1.0"
    readonly schemaCount: number;       // Number of STRUCT definitions
    readonly aliasCount: number;        // Number of ALIAS definitions
    readonly nestCount: number;         // Number of NEST definitions
    readonly rootItemCount: number;     // Number of root items

    // Methods
    getSchemaNames(): string[];
    getSchema(typeName: string): string[] | undefined;
    getAliases(): JsonObject;
    getNests(): JsonObject;
    toJson(): JsonValue;
    toJsonString(pretty?: boolean): string;
    toHedl(useDitto?: boolean): string;
    countEntities(): { [typeName: string]: number };
    query(typeName?: string, id?: string): EntityResult[];
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

// Convert to JSON
const json = doc.toJson();
console.log(json);

// Count entities
const counts = doc.countEntities();
console.log('Entity counts:', counts);
```

---

#### `query`

Query entities by type and/or ID.

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
    fields: JsonObject;
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

### Format Conversion

#### `toJson`

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

#### `fromJson`

Convert JSON string to HEDL string.

```typescript
function fromJson(json: string, useDitto?: boolean): string
```

**Parameters**:
- `json`: JSON string
- `useDitto`: Enable ditto optimization (default: `true`)

**Returns**: HEDL string

**Example**:
```typescript
const jsonData = '{"users": [{"id": "alice", "name": "Alice"}]}';
const hedl = fromJson(jsonData, true);
console.log(hedl);
```

---

#### `format`

Format HEDL to canonical form.

```typescript
function format(hedl: string, useDitto?: boolean): string
```

**Parameters**:
- `hedl`: HEDL document string
- `useDitto`: Enable ditto optimization (default: `true`)

**Returns**: Formatted HEDL string

**Example**:
```typescript
const formatted = format(messyHedl, true);
console.log(formatted);
```

---

### Validation

#### `validate`

Validate HEDL and return detailed diagnostics.

```typescript
function validate(hedl: string, runLint?: boolean): ValidationResult
```

**Parameters**:
- `hedl`: HEDL document string
- `runLint`: Run linting rules (default: `true`)

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

### Statistics

#### `getStats`

Get token usage statistics.

```typescript
function getStats(hedl: string): TokenStats
```

**Returns**: Token statistics

```typescript
interface TokenStats {
    hedlBytes: number;
    hedlTokens: number;
    hedlLines: number;
    jsonBytes: number;
    jsonTokens: number;
    savingsPercent: number;
    tokensSaved: number;
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

#### `compareTokens`

Compare HEDL and JSON token counts.

```typescript
function compareTokens(hedl: string, json: string): ComparisonResult
```

**Returns**: Comparison result

```typescript
interface ComparisonResult {
    hedl: {
        bytes: number;
        tokens: number;
        lines: number;
    };
    json: {
        bytes: number;
        tokens: number;
    };
    savings: {
        percent: number;
        tokens: number;
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

#### `version`

Get HEDL library version.

```typescript
function version(): string
```

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
export type JsonPrimitive = string | number | boolean | null;

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
export type JsonValue = JsonPrimitive | JsonObject | JsonArray;
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

| Error Message Pattern | Cause |
|----------------------|-------|
| `Parse error at line N` | Syntax error in HEDL |
| `Input size (X bytes) exceeds maximum` | Input too large |
| `Invalid JSON` | Malformed JSON input |
| `Conversion error` | Format conversion failed |

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
    const [hedl, setHedl] = useState('%VERSION: 1.0\n---\n');
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
            hedl: '%VERSION: 1.0\n---\n',
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
