# JavaScript/TypeScript SDK Documentation

Complete SDK documentation for using HEDL in JavaScript and TypeScript via WebAssembly.

## Installation

```bash
# npm
npm install hedl-wasm

# yarn
yarn add hedl-wasm

# pnpm
pnpm add hedl-wasm
```

## Quick Start

### JavaScript (ES Modules)

```javascript
import init, { parse, toJson } from 'hedl-wasm';

// Initialize WASM module
await init();

const hedlText = `
%VERSION: 1.0
---
name: Alice
age: 30
`;

const doc = parse(hedlText);
const json = doc.toJsonString();  // Use doc.toJsonString()
console.log(json);
```

### TypeScript

```typescript
import init, { parse, toJson, validate } from 'hedl-wasm';

await init();

try {
    const doc = parse(hedlText);
    const json: string = doc.toJsonString();  // Use doc.toJsonString()
    console.log(json);
} catch (error) {
    console.error(`Parse error: ${error}`);
}
```

## API Functions

### Initialization

```typescript
function init(): Promise<void>
```

Must be called once before using any other functions.

### Parsing

```typescript
function parse(input: string): HedlDocument
```

Parse HEDL document and return a document object.

**Throws**: Error on parse failure

### Serialization

```typescript
function toJson(hedl: string, pretty?: boolean): string
function format(hedl: string, useDitto?: boolean): string
```

- `toJson()`: Convert HEDL string to JSON
- `format()`: Format/canonicalize HEDL

### Validation

```typescript
interface ValidationResult {
    valid: boolean;
    errors: Array<{
        line: number;
        message: string;
    }>;
}

function validate(input: string): ValidationResult
```

### Linting

```typescript
interface ValidationResult {
    valid: boolean;
    errors: ValidationError[];
    warnings: ValidationWarning[];
}

function validate(hedl: string, runLint?: boolean): ValidationResult
```

### Conversion

```typescript
function fromJson(json: string, useDitto?: boolean): string
```

Convert JSON to HEDL string.

### Statistics

```typescript
interface TokenStats {
    hedlTokens: number;
    jsonTokens: number;
    savingsPercent: number;
    compressionRatio: number;
}

function getStats(input: string): TokenStats
```

### Configuration

```typescript
function setMaxInputSize(size: number): void
function getMaxInputSize(): number
```

## TypeScript Definitions

```typescript
// Errors are thrown as standard JavaScript Error objects
export interface ValidationError {
    line: number;
    message: string;
    type: string;
}

export interface ValidationWarning {
    line: number;
    message: string;
    rule: string;
}

export type JsonValue = string | number | boolean | null | JsonObject | JsonArray;
export interface JsonObject { [key: string]: JsonValue; }
export interface JsonArray extends Array<JsonValue> {}
```

## Browser Integration

### HTML + Vanilla JS

```html
<!DOCTYPE html>
<html>
<head>
    <title>HEDL Parser</title>
</head>
<body>
    <textarea id="input"></textarea>
    <button onclick="parseHedl()">Parse</button>
    <pre id="output"></pre>

    <script type="module">
        import init, { parse, toJson } from './node_modules/hedl-wasm/hedl_wasm.js';

        await init();

        window.parseHedl = function() {
            const input = document.getElementById('input').value;
            try {
                const doc = parse(input);
                const json = doc.toJsonString();
                document.getElementById('output').textContent = JSON.stringify(JSON.parse(json), null, 2);
            } catch (error) {
                document.getElementById('output').textContent = `Error: ${error.message}`;
            }
        };
    </script>
</body>
</html>
```

### React

```tsx
import { useState, useEffect } from 'react';
import init, { parse, toJson } from 'hedl-wasm';

function HedlEditor() {
    const [ready, setReady] = useState(false);
    const [input, setInput] = useState('');
    const [output, setOutput] = useState('');

    useEffect(() => {
        init().then(() => setReady(true));
    }, []);

    const handleParse = () => {
        try {
            const doc = parse(input);
            const json = doc.toJsonString();
            setOutput(JSON.stringify(JSON.parse(json), null, 2));
        } catch (error) {
            setOutput(`Error: ${error.message}`);
        }
    };

    if (!ready) return <div>Loading...</div>;

    return (
        <div>
            <textarea value={input} onChange={(e) => setInput(e.target.value)} />
            <button onClick={handleParse}>Parse</button>
            <pre>{output}</pre>
        </div>
    );
}
```

### Vue

```vue
<template>
  <div>
    <textarea v-model="input"></textarea>
    <button @click="parseHedl">Parse</button>
    <pre>{{ output }}</pre>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import init, { parse, toJson } from 'hedl-wasm';

const input = ref('');
const output = ref('');
const ready = ref(false);

onMounted(async () => {
  await init();
  ready.value = true;
});

const parseHedl = () => {
  try {
    const doc = parse(input.value);
    const json = doc.toJsonString();
    output.value = JSON.stringify(JSON.parse(json), null, 2);
  } catch (error: any) {
    output.value = `Error: ${error.message}`;
  }
};
</script>
```

## Node.js Integration

```typescript
import init, { parse, toJson } from 'hedl-wasm';
import fs from 'fs/promises';

async function processHedlFile(path: string) {
    await init();

    const content = await fs.readFile(path, 'utf-8');
    const doc = parse(content);
    const json = doc.toJsonString();

    return JSON.parse(json);
}
```

## Error Handling

```typescript
function safeParseHedl(input: string) {
    try {
        const doc = parse(input);
        return { success: true, doc };
    } catch (error) {
        if (error instanceof HedlError) {
            return {
                success: false,
                error: {
                    kind: error.kind,
                    message: error.message,
                    line: error.line,
                }
            };
        }
        throw error;
    }
}
```

## Performance

### Lazy Loading

```javascript
let hedl = null;

async function getHedl() {
    if (!hedl) {
        hedl = await import('hedl-wasm');
        await hedl.default();
    }
    return hedl;
}
```

### Web Workers

```javascript
// worker.js
import init, { parse } from 'hedl-wasm';

let initialized = false;

self.onmessage = async (e) => {
    if (!initialized) {
        await init();
        initialized = true;
    }

    try {
        const doc = parse(e.data);
        self.postMessage({ success: true, doc });
    } catch (error) {
        self.postMessage({ success: false, error: error.message });
    }
};
```

## Examples

See [WASM Browser Tutorial](../tutorials/03-wasm-browser.md) for complete examples.

## Platform Support

- Modern browsers (Chrome, Firefox, Safari, Edge)
- Node.js 16+
- Deno
- Bun

## See Also

- [WASM API Reference](../wasm-api.md)
- [WASM Browser Tutorial](../tutorials/03-wasm-browser.md)
- [npm Package](https://www.npmjs.com/package/hedl-wasm)
