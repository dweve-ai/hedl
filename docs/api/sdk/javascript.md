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
%V:2.0
%NULL:~
%QUOTE:"
---
name: Alice
age: 30
`;

const doc = parse(hedlText);
const json = doc.toJsonString();
console.log(json);
```

### TypeScript

```typescript
import init, { parse, toJson, validate } from 'hedl-wasm';

await init();

try {
    const doc = parse(hedlText);
    const json: string = doc.toJsonString();
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
// Parse to document then convert
function parse(input: string): HedlDocument
// Or use module-level functions
function toJson(hedl: string, pretty?: boolean): string
function format(hedl: string): string
```

- `parse()`: Parse HEDL to document, then use `doc.toJsonString()` to convert to JSON
- `toJson()`: Convert HEDL string directly to JSON
- `format()`: Format/canonicalize HEDL

### Validation

```typescript
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

interface ValidationResult {
    valid: boolean;
    errors: ValidationError[];
    warnings: ValidationWarning[];  // Populated when runLint=true
}

// Basic validation (syntax only)
function validate(hedl: string): ValidationResult

// Validation with optional linting
function validate(hedl: string, runLint?: boolean): ValidationResult
```

### Conversion

```typescript
// JSON conversion
function fromJson(json: string): string

// YAML conversion (requires yaml feature)
function toYaml(hedl: string): string
function fromYaml(yaml: string): string

// XML conversion (requires xml feature)
function toXml(hedl: string): string
function fromXml(xml: string): string

// CSV conversion (requires csv feature)
function toCsv(hedl: string): string
function fromCsv(csv: string, typeName?: string): string

// TOON conversion (requires toon feature)
function toToon(hedl: string): string
function fromToon(toon: string): string
```

Convert between HEDL and various data formats.

### Statistics

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

function getStats(hedl: string): TokenStats
```

### Configuration

```typescript
function setMaxInputSize(size: number): void
function getMaxInputSize(): number
```

## TypeScript Definitions

```typescript
// Validation types
export interface ValidationError {
    line: number;
    message: string;
    type: string;  // Error type, e.g., "SyntaxError"
}

export interface ValidationWarning {
    line: number;
    message: string;
    rule: string;  // Lint rule ID that generated the warning
}

export interface ValidationResult {
    valid: boolean;
    errors: ValidationError[];
    warnings: ValidationWarning[];
}

// JSON value types
export type JsonValue = string |number |boolean |null |JsonObject |JsonArray;
export interface JsonObject { [key: string]: JsonValue; }
export interface JsonArray extends Array<JsonValue> {}

// Token statistics
export interface TokenStats {
    hedlBytes: number;
    hedlTokens: number;
    hedlLines: number;
    jsonBytes: number;
    jsonTokens: number;
    savingsPercent: number;
    tokensSaved: number;
}
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

All HEDL WASM functions throw standard JavaScript `Error` objects on failure. The error message contains details about the failure, including line numbers for parse errors.

```typescript
function safeParseHedl(input: string) {
    try {
        const doc = parse(input);
        return { success: true, doc };
    } catch (error) {
        // Errors are standard JavaScript Error objects
        if (error instanceof Error) {
            return {
                success: false,
                error: {
                    message: error.message,
                    // Parse errors include line info in message, e.g.:
                    // "Parse error at line 5: unexpected token"
                }
            };
        }
        throw error;
    }
}

// Extract line number from parse error messages
function extractLineFromError(error: Error): number |null {
    const match = error.message.match(/line (\d+)/i);
    return match ? parseInt(match[1], 10) : null;
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
