# hedl-wasm

WebAssembly bindings for HEDL, enabling use in browsers and Node.js.

## Installation

```bash
npm install hedl-wasm
```

## Usage (JavaScript/TypeScript)

```typescript
import init, { parse, toJson, fromJson } from 'hedl-wasm';

await init();

// Parse HEDL
const doc = parse(`
%VERSION: 1.0
---
name: Example
value: 42
`);

// Convert to JSON
const json = toJson(doc);
console.log(json);

// Parse from JSON
const doc2 = fromJson('{"name": "Example", "value": 42}');
```

## Browser Usage

```html
<script type="module">
  import init, { parse, toJson } from './hedl_wasm.js';

  async function main() {
    await init();
    const doc = parse(hedlString);
    console.log(toJson(doc));
  }

  main();
</script>
```

## Available Functions

- `parse(hedl: string)` - Parse HEDL string
- `toJson(doc)` - Convert to JSON
- `fromJson(json: string)` - Parse JSON to HEDL
- `validate(hedl: string)` - Validate HEDL syntax
- `format(hedl: string)` - Canonicalize HEDL
- `lint(hedl: string)` - Lint HEDL document

## Building from Source

```bash
wasm-pack build --target web
```

## License

Apache-2.0
