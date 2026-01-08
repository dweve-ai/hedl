# WASM Browser Integration Tutorial

This tutorial demonstrates how to use HEDL in web browsers and Node.js via WebAssembly.

## Prerequisites

- Node.js 16+ or modern browser
- npm or yarn
- Basic JavaScript/TypeScript knowledge

## Installation

### npm

```bash
npm install hedl-wasm
```

### yarn

```bash
yarn add hedl-wasm
```

### CDN (Browser)

```html
<script type="module">
  import init, { parse, toJson } from 'https://cdn.jsdelivr.net/npm/hedl-wasm/hedl_wasm.js';

  await init();
  // Use HEDL functions
</script>
```

## Quick Start

### Node.js

```javascript
import init, { parse, toJson } from 'hedl-wasm';

// Initialize WASM module
await init();

const hedlText = `
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
`;

const doc = parse(hedlText);
console.log('Parsed:', doc);

// Convert to JSON (toJson takes HEDL string, returns JSON string)
const json = toJson(hedlText, true);
console.log('JSON:', json);
```

### Browser (ES Modules)

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>HEDL WASM Example</title>
</head>
<body>
    <h1>HEDL Parser</h1>
    <textarea id="input" rows="10" cols="80">
%VERSION: 1.0
---
name: Alice
age: 30
    </textarea>
    <button onclick="parseHedl()">Parse</button>
    <pre id="output"></pre>

    <script type="module">
        import init, { parse, toJson, validate } from './node_modules/hedl-wasm/hedl_wasm.js';

        await init();

        window.parseHedl = function() {
            const input = document.getElementById('input').value;
            const output = document.getElementById('output');

            try {
                // Parse and validate HEDL
                const doc = parse(input);
                // Convert to JSON
                const json = toJson(input);
                output.textContent = JSON.stringify(JSON.parse(json), null, 2);
            } catch (error) {
                output.textContent = `Error: ${error.message}`;
            }
        };
    </script>
</body>
</html>
```

### TypeScript

```typescript
import init, {
    parse,
    toJson,
    fromJson,
    format,
    validate,
    lint,
    getStats
} from 'hedl-wasm';

// Initialize (only needed once)
await init();

const hedlText: string = `
%VERSION: 1.0
---
name: Alice
age: 30
`;

// Parse HEDL
const doc = parse(hedlText);

// Convert to JSON
const json: string = toJson(hedlText);
console.log('JSON:', json);

// Validate
const validationResult = validate(hedlText);
if (!validationResult.valid) {
    console.error('Validation errors:', validationResult.errors);
}

// Get token statistics
const stats = getStats(hedlText);
console.log(`Token savings: ${stats.savingsPercent}%`);
```

## Core Functions

### Parsing

```javascript
import { parse, validate } from 'hedl-wasm';

// Basic parsing
try {
    const doc = parse(hedlText);
    console.log(`Parsed document version: ${doc.version}`);
} catch (error) {
    console.error(`Parse error: ${error.message}`);
}

// Validation
const result = validate(hedlText);
if (!result.valid) {
    result.errors.forEach(err => {
        console.error(`Line ${err.line}: ${err.message}`);
    });
}
```

### Serialization

```javascript
import { parse, toJson, format } from 'hedl-wasm';

const doc = parse(hedlText);

// Convert HEDL string to JSON string
const json = toJson(hedlText, true);
console.log('JSON:', json);

// Format/canonicalize HEDL
const canonical = format(hedlText, true);
console.log('Canonical HEDL:', canonical);
```

### Format Conversion

```javascript
import { fromJson, toJson } from 'hedl-wasm';

// JSON to HEDL (returns HEDL string)
const jsonInput = '{"name": "Alice", "age": 30}';
const hedlString = fromJson(jsonInput, true);

// HEDL to JSON (takes HEDL string, returns JSON string)
const jsonOutput = toJson(hedlString, true);
```

### Linting

```javascript
import { validate } from 'hedl-wasm';

// Validate with linting
const result = validate(hedlText, true);

if (!result.valid) {
    result.errors.forEach(e => {
        console.error(`[ERROR] Line ${e.line}: ${e.message}`);
    });
}

if (result.warnings) {
    result.warnings.forEach(w => {
        console.warn(`[WARNING] Line ${w.line}: ${w.message}`);
    });
}
```

### Token Statistics

```javascript
import { getStats } from 'hedl-wasm';

const stats = getStats(hedlText);
console.log(`HEDL tokens: ${stats.hedlTokens}`);
console.log(`JSON tokens: ${stats.jsonTokens}`);
console.log(`Savings: ${stats.savingsPercent}%`);
```

## React Integration

### React Component

```tsx
import React, { useState, useEffect } from 'react';
import init, { parse, toJson, validate } from 'hedl-wasm';

const HedlEditor: React.FC = () => {
    const [wasmReady, setWasmReady] = useState(false);
    const [input, setInput] = useState(`%VERSION: 1.0\n---\nname: Alice\nage: 30`);
    const [output, setOutput] = useState('');
    const [error, setError] = useState('');

    useEffect(() => {
        init().then(() => setWasmReady(true));
    }, []);

    const handleParse = () => {
        if (!wasmReady) {
            setError('WASM not initialized');
            return;
        }

        try {
            // Validate first
            const validation = validate(input);
            if (!validation.valid) {
                setError(validation.errors.map(e => e.message).join('\n'));
                return;
            }

            // Parse and convert
            const doc = parse(input);
            const json = toJson(input);
            setOutput(JSON.stringify(JSON.parse(json), null, 2));
            setError('');
        } catch (err: any) {
            setError(err.message);
            setOutput('');
        }
    };

    if (!wasmReady) {
        return <div>Loading HEDL parser...</div>;
    }

    return (
        <div className="hedl-editor">
            <h2>HEDL Editor</h2>
            <textarea
                value={input}
                onChange={(e) => setInput(e.target.value)}
                rows={15}
                cols={80}
                placeholder="Enter HEDL document..."
            />
            <button onClick={handleParse}>Parse & Convert to JSON</button>

            {error && (
                <div className="error">
                    <h3>Error:</h3>
                    <pre>{error}</pre>
                </div>
            )}

            {output && (
                <div className="output">
                    <h3>JSON Output:</h3>
                    <pre>{output}</pre>
                </div>
            )}
        </div>
    );
};

export default HedlEditor;
```

### React Hook

```typescript
import { useState, useEffect } from 'react';
import init, { parse, toJson, ValidationResult } from 'hedl-wasm';

interface UseHedlResult {
    ready: boolean;
    parse: (input: string) => any;
    toJson: (doc: any) => string;
    validate: (input: string) => ValidationResult;
}

export function useHedl(): UseHedlResult {
    const [ready, setReady] = useState(false);

    useEffect(() => {
        init().then(() => setReady(true));
    }, []);

    return {
        ready,
        parse: ready ? parse : () => { throw new Error('WASM not ready'); },
        toJson: ready ? toJson : () => { throw new Error('WASM not ready'); },
        validate: ready ? validate : () => ({ valid: false, errors: [] }),
    };
}

// Usage
function MyComponent() {
    const hedl = useHedl();

    const handleParse = (input: string) => {
        if (!hedl.ready) return;

        try {
            const doc = hedl.parse(input);
            const json = hedl.toJson(input);
            console.log(json);
        } catch (error) {
            console.error(error);
        }
    };

    return <div>...</div>;
}
```

## Vue Integration

```vue
<template>
  <div class="hedl-editor">
    <h2>HEDL Editor</h2>
    <textarea v-model="input" rows="15" cols="80"></textarea>
    <button @click="parseInput">Parse</button>

    <div v-if="error" class="error">
      <h3>Error:</h3>
      <pre>{{ error }}</pre>
    </div>

    <div v-if="output" class="output">
      <h3>JSON Output:</h3>
      <pre>{{ output }}</pre>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import init, { parse, toJson, validate } from 'hedl-wasm';

const input = ref(`%VERSION: 1.0\n---\nname: Alice\nage: 30`);
const output = ref('');
const error = ref('');
const ready = ref(false);

onMounted(async () => {
  await init();
  ready.value = true;
});

const parseInput = () => {
  if (!ready.value) {
    error.value = 'WASM not ready';
    return;
  }

  try {
    const validation = validate(input.value);
    if (!validation.valid) {
      error.value = validation.errors.map(e => e.message).join('\n');
      return;
    }

    const doc = parse(input.value);
    const json = toJson(input.value, true);
    output.value = JSON.stringify(JSON.parse(json), null, 2);
    error.value = '';
  } catch (err: any) {
    error.value = err.message;
    output.value = '';
  }
};
</script>
```

## Performance Optimization

### Lazy Loading

```javascript
let hedlModule = null;

async function getHedl() {
    if (!hedlModule) {
        hedlModule = await import('hedl-wasm');
        await hedlModule.default();
    }
    return hedlModule;
}

// Use only when needed
async function parseHedl(input) {
    const hedl = await getHedl();
    return hedl.parse(input);
}
```

### Web Workers

```javascript
// worker.js
import init, { parse, toJson } from 'hedl-wasm';

let initialized = false;

self.onmessage = async (e) => {
    if (!initialized) {
        await init();
        initialized = true;
    }

    const { type, data } = e.data;

    try {
        switch (type) {
            case 'parse':
                const doc = parse(data);
                self.postMessage({ success: true, result: doc });
                break;
            case 'toJson':
                const json = toJson(data);
                self.postMessage({ success: true, result: json });
                break;
        }
    } catch (error) {
        self.postMessage({ success: false, error: error.message });
    }
};
```

```javascript
// main.js
const worker = new Worker('worker.js', { type: 'module' });

worker.onmessage = (e) => {
    const { success, result, error } = e.data;
    if (success) {
        console.log('Result:', result);
    } else {
        console.error('Error:', error);
    }
};

worker.postMessage({ type: 'parse', data: hedlText });
```

### Input Size Limits

```javascript
import { setMaxInputSize, getMaxInputSize } from 'hedl-wasm';

// Default is 500 MB
console.log('Max input size:', getMaxInputSize());

// Increase for larger documents
setMaxInputSize(1024 * 1024 * 1024); // 1 GB

// Or decrease for constrained environments
setMaxInputSize(10 * 1024 * 1024); // 10 MB
```

## Error Handling

### Comprehensive Error Handling

```typescript
import { parse, HedlError } from 'hedl-wasm';

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
                    column: error.column,
                }
            };
        }
        return {
            success: false,
            error: { message: 'Unknown error' }
        };
    }
}
```

## Complete Example: Data Viewer

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>HEDL Data Viewer</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .container { display: flex; gap: 20px; }
        .editor, .viewer { flex: 1; }
        textarea { width: 100%; height: 400px; font-family: monospace; }
        .error { color: red; background: #fee; padding: 10px; }
        .stats { background: #efe; padding: 10px; margin: 10px 0; }
        pre { background: #f5f5f5; padding: 10px; overflow: auto; }
    </style>
</head>
<body>
    <h1>HEDL Data Viewer</h1>
    <div class="container">
        <div class="editor">
            <h2>HEDL Input</h2>
            <textarea id="hedl-input">%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com</textarea>
            <button onclick="processHedl()">Parse & Convert</button>
        </div>
        <div class="viewer">
            <h2>Output</h2>
            <div id="stats"></div>
            <div id="error"></div>
            <pre id="output"></pre>
        </div>
    </div>

    <script type="module">
        import init, { parse, toJson, validate, getStats } from './node_modules/hedl-wasm/hedl_wasm.js';

        await init();

        window.processHedl = function() {
            const input = document.getElementById('hedl-input').value;
            const statsDiv = document.getElementById('stats');
            const errorDiv = document.getElementById('error');
            const outputPre = document.getElementById('output');

            // Clear previous output
            statsDiv.innerHTML = '';
            errorDiv.innerHTML = '';
            outputPre.textContent = '';

            try {
                // Validate
                const validation = validate(input);
                if (!validation.valid) {
                    errorDiv.className = 'error';
                    errorDiv.textContent = validation.errors
                        .map(e => `Line ${e.line}: ${e.message}`)
                        .join('\n');
                    return;
                }

                // Get statistics
                const stats = getStats(input);
                statsDiv.className = 'stats';
                statsDiv.innerHTML = `
                    <strong>Token Statistics:</strong><br>
                    HEDL tokens: ${stats.hedlTokens}<br>
                    JSON tokens: ${stats.jsonTokens}<br>
                    Savings: ${stats.savingsPercent}%
                `;

                // Parse and convert
                const doc = parse(input);
                const json = toJson(input, true);
                outputPre.textContent = JSON.stringify(JSON.parse(json), null, 2);

            } catch (error) {
                errorDiv.className = 'error';
                errorDiv.textContent = `Error: ${error.message}`;
            }
        };

        // Auto-process on load
        processHedl();
    </script>
</body>
</html>
```

## Next Steps

- **[MCP Server Usage](04-mcp-server.md)** - AI/LLM integration
- **[JavaScript SDK](../sdk/javascript.md)** - Complete SDK documentation
- **[Examples](../examples.md)** - More code examples
- **[WASM API Reference](../wasm-api.md)** - Full API documentation

## Resources

- **[npm package](https://www.npmjs.com/package/hedl-wasm)** - Package page
- **[GitHub](https://github.com/dweve/hedl)** - Source code
- **[Examples](https://github.com/dweve/hedl/tree/main/examples)** - Example applications
