# HEDL WASM Examples

Comprehensive usage examples for hedl-wasm in browser and Node.js environments.

## Table of Contents

- [Getting Started](#getting-started)
- [Installation](#installation)
- [Browser Usage](#browser-usage)
- [Node.js Usage](#nodejs-usage)
- [TypeScript Integration](#typescript-integration)
- [Error Handling](#error-handling)
- [Performance Best Practices](#performance-best-practices)
- [Troubleshooting](#troubleshooting)

---

## Getting Started

hedl-wasm provides WebAssembly bindings for HEDL, enabling efficient parsing and conversion in JavaScript/TypeScript environments. The package supports:

- **Browser environments** (Chrome 57+, Firefox 52+, Safari 11+, Edge 16+)
- **Node.js** (14+)
- **TypeScript** with full type definitions
- **React, Vue, Angular** and other frameworks
- **Bundlers** (webpack, rollup, vite, parcel)

### Quick Example

```javascript
import init, { parse } from 'hedl-wasm';

await init();

const doc = parse(`
%VERSION: 1.0
%STRUCT: User[id,name,email]
---
users: @User
  |alice,Alice Smith,alice@example.com
  |bob,Bob Jones,bob@example.com
`);

console.log(`Found ${doc.rootItemCount} items`);
const json = doc.toJsonString();
console.log(json);
```

---

## Installation

### NPM

```bash
npm install hedl-wasm
```

### Yarn

```bash
yarn add hedl-wasm
```

### PNPM

```bash
pnpm add hedl-wasm
```

### CDN (Browser)

```html
<!-- ES Modules -->
<script type="module">
  import init, { parse } from 'https://unpkg.com/hedl-wasm@latest/hedl_wasm.js';
  await init();
</script>

<!-- Specific version (recommended for production) -->
<script type="module">
  import init from 'https://unpkg.com/hedl-wasm@1.0.0/hedl_wasm.js';
</script>
```

---

## Browser Usage

### Basic HTML Example

See [browser-basic.html](browser-basic.html) for a complete working example.

```html
<!DOCTYPE html>
<html>
<head>
  <title>HEDL WASM Example</title>
</head>
<body>
  <textarea id="hedl-input" rows="10" cols="50">
%VERSION: 1.0
%STRUCT: User[id,name]
---
users: @User
  |alice,Alice Smith
  |bob,Bob Jones
  </textarea>
  <button id="convert">Convert to JSON</button>
  <pre id="output"></pre>

  <script type="module">
    import init, { parse } from 'https://unpkg.com/hedl-wasm@latest/hedl_wasm.js';

    await init();

    document.getElementById('convert').addEventListener('click', () => {
      const hedl = document.getElementById('hedl-input').value;
      try {
        const doc = parse(hedl);
        const json = doc.toJsonString(true);
        document.getElementById('output').textContent = json;
      } catch (e) {
        document.getElementById('output').textContent = 'Error: ' + e.message;
      }
    });
  </script>
</body>
</html>
```

### Live Editor Example

See [browser-editor.html](browser-editor.html) for a full-featured live editor with:
- Real-time HEDL → JSON conversion
- Syntax validation with error highlighting
- Token statistics display
- Format on save

### React Integration

See [react-example.jsx](react-example.jsx) for a complete React component.

```jsx
import { useEffect, useState } from 'react';
import init, { parse, validate } from 'hedl-wasm';

function HedlEditor() {
  const [initialized, setInitialized] = useState(false);
  const [hedl, setHedl] = useState('');
  const [json, setJson] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    init().then(() => setInitialized(true));
  }, []);

  const handleConvert = () => {
    if (!initialized) return;

    try {
      // Validate first
      const result = validate(hedl);
      if (!result.valid) {
        setError(result.errors.map(e =>
          `Line ${e.line}: ${e.message}`
        ).join('\n'));
        return;
      }

      // Parse and convert
      const doc = parse(hedl);
      setJson(doc.toJsonString(true));
      setError('');
    } catch (e) {
      setError(e.message);
    }
  };

  return (
    <div>
      <textarea value={hedl} onChange={e => setHedl(e.target.value)} />
      <button onClick={handleConvert} disabled={!initialized}>
        Convert
      </button>
      {error && <div className="error">{error}</div>}
      {json && <pre>{json}</pre>}
    </div>
  );
}
```

### Vue Integration

```vue
<template>
  <div>
    <textarea v-model="hedl" @input="convertToJson"></textarea>
    <div v-if="error" class="error">{{ error }}</div>
    <pre v-else>{{ json }}</pre>
  </div>
</template>

<script>
import { ref, onMounted } from 'vue';
import init, { parse } from 'hedl-wasm';

export default {
  setup() {
    const hedl = ref('');
    const json = ref('');
    const error = ref('');
    const initialized = ref(false);

    onMounted(async () => {
      await init();
      initialized.value = true;
    });

    const convertToJson = () => {
      if (!initialized.value) return;

      try {
        const doc = parse(hedl.value);
        json.value = doc.toJsonString(true);
        error.value = '';
      } catch (e) {
        error.value = e.message;
      }
    };

    return { hedl, json, error, convertToJson };
  }
};
</script>
```

---

## Node.js Usage

### Basic Example

See [node-basic.js](node-basic.js) for a complete example.

```javascript
const { readFileSync, writeFileSync } = require('fs');
const { parse, fromJson, validate } = require('hedl-wasm');

// Parse HEDL file
const hedl = readFileSync('data.hedl', 'utf8');
const doc = parse(hedl);

// Convert to JSON
const json = doc.toJsonString(true);
writeFileSync('output.json', json);

console.log(`Converted ${doc.rootItemCount} items`);
```

### File Conversion Script

See [node-convert.js](node-convert.js) for a CLI-style converter.

```javascript
#!/usr/bin/env node

const fs = require('fs');
const { parse, fromJson, format } = require('hedl-wasm');

const args = process.argv.slice(2);
if (args.length < 2) {
  console.error('Usage: node convert.js <input> <output> [--format json|hedl]');
  process.exit(1);
}

const [input, output] = args;
const outputFormat = args[2] === '--format' ? args[3] : 'json';

try {
  const content = fs.readFileSync(input, 'utf8');

  let result;
  if (outputFormat === 'json') {
    // HEDL → JSON
    const doc = parse(content);
    result = doc.toJsonString(true);
  } else {
    // JSON → HEDL
    result = fromJson(content, true);
  }

  fs.writeFileSync(output, result);
  console.log(`Converted ${input} → ${output}`);
} catch (e) {
  console.error('Conversion failed:', e.message);
  process.exit(1);
}
```

### Batch Processing

```javascript
const { parse } = require('hedl-wasm');
const { promisify } = require('util');
const fs = require('fs');
const path = require('path');

const readdir = promisify(fs.readdir);
const readFile = promisify(fs.readFile);
const writeFile = promisify(fs.writeFile);

async function convertDirectory(inputDir, outputDir) {
  const files = await readdir(inputDir);
  const hedlFiles = files.filter(f => f.endsWith('.hedl'));

  for (const file of hedlFiles) {
    const inputPath = path.join(inputDir, file);
    const outputPath = path.join(outputDir, file.replace('.hedl', '.json'));

    const hedl = await readFile(inputPath, 'utf8');
    const doc = parse(hedl);
    const json = doc.toJsonString(true);

    await writeFile(outputPath, json);
    console.log(`Converted: ${file}`);
  }
}

convertDirectory('./data', './output').catch(console.error);
```

### Stream Processing (Large Files)

```javascript
const { parse, setMaxInputSize } = require('hedl-wasm');
const fs = require('fs');
const readline = require('readline');

// Increase limit for large files
setMaxInputSize(1024 * 1024 * 1024); // 1GB

async function processLargeFile(filePath) {
  const rl = readline.createInterface({
    input: fs.createReadStream(filePath),
    crlfDelay: Infinity
  });

  let buffer = '';
  let chunkCount = 0;

  for await (const line of rl) {
    buffer += line + '\n';

    // Process in chunks of 10MB
    if (buffer.length > 10 * 1024 * 1024) {
      try {
        const doc = parse(buffer);
        console.log(`Chunk ${++chunkCount}: ${doc.rootItemCount} items`);
        buffer = '';
      } catch (e) {
        console.error(`Error in chunk ${chunkCount}:`, e.message);
      }
    }
  }

  // Process remaining buffer
  if (buffer.length > 0) {
    const doc = parse(buffer);
    console.log(`Final chunk: ${doc.rootItemCount} items`);
  }
}

processLargeFile('./large-data.hedl').catch(console.error);
```

---

## TypeScript Integration

### Type-Safe Usage

See [typescript-example.ts](typescript-example.ts) for comprehensive examples.

```typescript
import init, {
  parse,
  validate,
  getStats,
  HedlDocument,
  ValidationResult,
  TokenStats,
  JsonValue,
  JsonObject
} from 'hedl-wasm';

async function processHedl(hedl: string): Promise<JsonObject> {
  await init();

  // Validate with full type safety
  const validation: ValidationResult = validate(hedl);
  if (!validation.valid) {
    throw new Error(
      validation.errors.map(e => `Line ${e.line}: ${e.message}`).join('\n')
    );
  }

  // Parse with type inference
  const doc: HedlDocument = parse(hedl);

  // Type-safe JSON conversion
  const json: JsonValue = doc.toJson();

  // Type guard for object check
  if (typeof json === 'object' && json !== null && !Array.isArray(json)) {
    return json as JsonObject;
  }

  throw new Error('Expected JSON object');
}

// Get statistics with proper typing
async function analyzeEfficiency(hedl: string): Promise<void> {
  const stats: TokenStats = getStats(hedl);

  console.log(`Token savings: ${stats.savingsPercent}%`);
  console.log(`Tokens saved: ${stats.tokensSaved}`);
  console.log(`HEDL: ${stats.hedlTokens} tokens (${stats.hedlBytes} bytes)`);
  console.log(`JSON: ${stats.jsonTokens} tokens (${stats.jsonBytes} bytes)`);
}
```

### Type Guards and Helpers

```typescript
import { JsonValue, JsonObject, JsonArray } from 'hedl-wasm';

// Type guards
function isJsonObject(value: JsonValue): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isJsonArray(value: JsonValue): value is JsonArray {
  return Array.isArray(value);
}

function isString(value: JsonValue): value is string {
  return typeof value === 'string';
}

function isNumber(value: JsonValue): value is number {
  return typeof value === 'number';
}

// Helper functions
function getString(obj: JsonObject, key: string): string | null {
  const value = obj[key];
  return isString(value) ? value : null;
}

function getNumber(obj: JsonObject, key: string): number | null {
  const value = obj[key];
  return isNumber(value) ? value : null;
}

function getObject(obj: JsonObject, key: string): JsonObject | null {
  const value = obj[key];
  return isJsonObject(value) ? value : null;
}

function getArray(obj: JsonObject, key: string): JsonArray | null {
  const value = obj[key];
  return isJsonArray(value) ? value : null;
}

// Usage
const doc = parse(hedl);
const json = doc.toJson();

if (isJsonObject(json)) {
  const name = getString(json, 'name');
  const age = getNumber(json, 'age');
  const users = getArray(json, 'users');

  if (name) console.log('Name:', name);
  if (age !== null) console.log('Age:', age);
  if (users) console.log('User count:', users.length);
}
```

---

## Error Handling

### Comprehensive Error Handling

```javascript
import { parse, validate, setMaxInputSize, getMaxInputSize } from 'hedl-wasm';

async function safeConvert(hedl) {
  try {
    // Check input size
    const maxSize = getMaxInputSize();
    if (hedl.length > maxSize) {
      throw new Error(
        `Input too large: ${hedl.length} bytes (max: ${maxSize} bytes)`
      );
    }

    // Validate first
    const validation = validate(hedl, true);
    if (!validation.valid) {
      const errors = validation.errors.map(e =>
        `Line ${e.line}: ${e.message} (${e.type})`
      );
      throw new Error('Validation failed:\n' + errors.join('\n'));
    }

    // Log warnings
    if (validation.warnings.length > 0) {
      console.warn('Warnings:');
      validation.warnings.forEach(w =>
        console.warn(`  Line ${w.line}: ${w.message} [${w.rule}]`)
      );
    }

    // Parse and convert
    const doc = parse(hedl);
    return doc.toJsonString(true);

  } catch (e) {
    console.error('Conversion error:', e.message);

    // Re-throw with context
    throw new Error(`Failed to convert HEDL: ${e.message}`);
  }
}
```

### Error Recovery

```javascript
function parseWithFallback(hedl, fallbackValue = null) {
  try {
    return parse(hedl);
  } catch (e) {
    console.error('Parse error:', e.message);

    // Try to extract line number
    const match = e.message.match(/line (\d+)/i);
    if (match) {
      const line = parseInt(match[1]);
      console.error(`Error at line ${line}`);

      // Show context (lines around error)
      const lines = hedl.split('\n');
      const start = Math.max(0, line - 3);
      const end = Math.min(lines.length, line + 2);
      console.error('Context:');
      for (let i = start; i < end; i++) {
        const marker = i === line - 1 ? '>>> ' : '    ';
        console.error(`${marker}${i + 1}: ${lines[i]}`);
      }
    }

    return fallbackValue;
  }
}
```

### Custom Error Classes

```typescript
class HedlParseError extends Error {
  constructor(
    message: string,
    public line?: number,
    public column?: number
  ) {
    super(message);
    this.name = 'HedlParseError';
  }
}

class HedlValidationError extends Error {
  constructor(
    message: string,
    public errors: Array<{ line: number; message: string }>
  ) {
    super(message);
    this.name = 'HedlValidationError';
  }
}

function parseStrict(hedl: string): HedlDocument {
  const result = validate(hedl);
  if (!result.valid) {
    throw new HedlValidationError(
      'Validation failed',
      result.errors.map(e => ({ line: e.line, message: e.message }))
    );
  }

  try {
    return parse(hedl);
  } catch (e) {
    const match = e.message.match(/line (\d+)/i);
    const line = match ? parseInt(match[1]) : undefined;
    throw new HedlParseError(e.message, line);
  }
}
```

---

## Performance Best Practices

### 1. Initialize Once

```javascript
// ❌ Bad: Initialize on every call
async function convert(hedl) {
  await init(); // Repeated initialization
  return parse(hedl);
}

// ✅ Good: Initialize once at startup
let initialized = false;

async function ensureInit() {
  if (!initialized) {
    await init();
    initialized = true;
  }
}

async function convert(hedl) {
  await ensureInit();
  return parse(hedl);
}
```

### 2. Reuse Document Objects

```javascript
// ❌ Bad: Re-parse for each operation
const doc1 = parse(hedl);
const json = doc1.toJson();

const doc2 = parse(hedl);
const count = doc2.countEntities();

// ✅ Good: Parse once, use multiple times
const doc = parse(hedl);
const json = doc.toJson();
const count = doc.countEntities();
const schemas = doc.getSchemaNames();
```

### 3. Batch Operations

```javascript
// ❌ Bad: Process one at a time
for (const hedl of hedlDocuments) {
  const doc = parse(hedl);
  results.push(doc.toJson());
}

// ✅ Good: Batch with async/await
const results = await Promise.all(
  hedlDocuments.map(async hedl => {
    const doc = parse(hedl);
    return doc.toJson();
  })
);
```

### 4. Configure Input Limits

```javascript
import { setMaxInputSize } from 'hedl-wasm';

// Set appropriate limit for your use case
setMaxInputSize(100 * 1024 * 1024); // 100MB for large files

// Or keep default (500MB) for normal usage
```

### 5. Use Compact JSON When Possible

```javascript
// For transmission/storage
const compact = doc.toJsonString(false); // No pretty-print

// For display/debugging
const pretty = doc.toJsonString(true); // Pretty-print
```

### 6. Validation Strategy

```javascript
// Development: Full validation
if (process.env.NODE_ENV === 'development') {
  const result = validate(hedl, true); // With linting
  if (!result.valid) throw new Error('Invalid HEDL');
}

// Production: Parse-only validation (faster)
const doc = parse(hedl); // Throws on syntax errors
```

### 7. Memory Management for Large Files

```javascript
// Process in chunks to avoid memory issues
async function processLargeFile(hedl) {
  // Increase limit
  setMaxInputSize(2 * 1024 * 1024 * 1024); // 2GB

  try {
    const doc = parse(hedl);

    // Process incrementally
    const schemas = doc.getSchemaNames();
    for (const schema of schemas) {
      const entities = doc.query(schema);
      // Process entities in batches
      for (let i = 0; i < entities.length; i += 1000) {
        const batch = entities.slice(i, i + 1000);
        await processBatch(batch);
      }
    }
  } finally {
    // Reset to default
    setMaxInputSize(500 * 1024 * 1024);
  }
}
```

---

## Troubleshooting

### Common Issues

#### 1. "RuntimeError: Unreachable executed"

**Cause**: WASM module not initialized before use.

**Solution**:
```javascript
import init, { parse } from 'hedl-wasm';

// ❌ Wrong: Using before init
const doc = parse(hedl); // Error!

// ✅ Correct: Init first
await init();
const doc = parse(hedl); // Works!
```

#### 2. "Input size exceeds maximum allowed size"

**Cause**: Input larger than configured limit (default 500MB).

**Solution**:
```javascript
import { setMaxInputSize } from 'hedl-wasm';

// Increase limit for large files
setMaxInputSize(1024 * 1024 * 1024); // 1GB
```

#### 3. "Parse error at line X"

**Cause**: Invalid HEDL syntax.

**Solution**:
```javascript
// Use validate() to get detailed errors
const result = validate(hedl);
if (!result.valid) {
  result.errors.forEach(e => {
    console.error(`Line ${e.line}: ${e.message}`);
  });
}
```

#### 4. Module not found in bundlers

**Webpack**:
```javascript
// webpack.config.js
module.exports = {
  experiments: {
    asyncWebAssembly: true
  }
};
```

**Vite**:
```javascript
// vite.config.js
export default {
  optimizeDeps: {
    exclude: ['hedl-wasm']
  }
};
```

**Rollup**:
```javascript
// rollup.config.js
export default {
  plugins: [
    wasm()
  ]
};
```

#### 5. TypeScript type errors

**Cause**: Missing type definitions.

**Solution**:
```typescript
// Install @types if needed
npm install --save-dev @types/hedl-wasm

// Or add to tsconfig.json
{
  "compilerOptions": {
    "types": ["hedl-wasm"]
  }
}
```

#### 6. React hydration errors (SSR)

**Cause**: WASM initialization during server-side rendering.

**Solution**:
```jsx
import { useEffect, useState } from 'react';
import init from 'hedl-wasm';

function MyComponent() {
  const [initialized, setInitialized] = useState(false);

  useEffect(() => {
    // Only initialize in browser
    init().then(() => setInitialized(true));
  }, []);

  if (!initialized) return <div>Loading...</div>;

  // Use HEDL functions here
}
```

### Debugging Tips

#### Enable verbose logging

```javascript
// Development mode
if (process.env.NODE_ENV === 'development') {
  console.log('HEDL WASM version:', version());
  console.log('Max input size:', getMaxInputSize());
}
```

#### Inspect parsed structure

```javascript
const doc = parse(hedl);

console.log('Document structure:');
console.log('  Version:', doc.version);
console.log('  Schemas:', doc.getSchemaNames());
console.log('  Aliases:', doc.getAliases());
console.log('  Nests:', doc.getNests());
console.log('  Entity counts:', doc.countEntities());
```

#### Validate step-by-step

```javascript
// 1. Check syntax
const result = validate(hedl, false);
if (!result.valid) {
  console.error('Syntax errors:', result.errors);
  return;
}

// 2. Check linting
const lintResult = validate(hedl, true);
if (lintResult.warnings.length > 0) {
  console.warn('Lint warnings:', lintResult.warnings);
}

// 3. Parse
const doc = parse(hedl);

// 4. Convert
const json = doc.toJson();
```

### Getting Help

- **Documentation**: https://docs.rs/hedl
- **Examples**: https://github.com/dweve-ai/hedl/tree/master/crates/hedl-wasm/examples
- **Issues**: https://github.com/dweve-ai/hedl/issues
- **Discussions**: https://github.com/dweve-ai/hedl/discussions

---

## Example Files

This directory contains the following working examples:

- **[browser-basic.html](browser-basic.html)** - Simple browser example with CDN
- **[browser-editor.html](browser-editor.html)** - Live HEDL editor with validation
- **[node-basic.js](node-basic.js)** - Basic Node.js file conversion
- **[node-convert.js](node-convert.js)** - CLI-style batch converter
- **[react-example.jsx](react-example.jsx)** - React component integration
- **[typescript-example.ts](typescript-example.ts)** - Comprehensive TypeScript usage
- **[token_estimation_demo.rs](token_estimation_demo.rs)** - Performance benchmarks

All examples are copy-paste ready and production-quality.

---

## License

Licensed under Apache-2.0 or MIT. See [LICENSE](../../LICENSE) for details.
