# hedl-json

**HEDL's integration with the JSON ecosystem—bidirectional conversion, JSONPath queries, schema generation, and streaming.**

JSON is the universal data interchange format. Your APIs speak it, your databases accept it, your monitoring tools consume it, your LLM providers require it. Every token in a JSON payload costs money. Every extra byte adds latency. Every API call compounds the inefficiency.

`hedl-json` bridges HEDL's efficiency with JSON's ubiquity. Use HEDL's compact matrix notation internally—save 46.7% on tokens, 57.7% on payload size. When you need JSON compatibility, `hedl-json` handles the conversion seamlessly. Query HEDL documents with JSONPath. Generate JSON Schema for validation. Stream large JSON files without loading everything into memory.

Part of the **HEDL format family** alongside `hedl-yaml`, `hedl-xml`, `hedl-csv`, and `hedl-parquet`—bringing HEDL's efficiency to every ecosystem you work in.

## What's Implemented

Based on 6,333 lines of Rust across 7 modules:

1. **Bidirectional Conversion**: HEDL ↔ JSON with configurable fidelity
2. **JSONPath Queries**: Query HEDL documents using standard JSONPath syntax
3. **JSON Schema Generation**: Generate JSON Schema Draft 7 from HEDL documents
4. **Streaming Parsers**: Process large JSON/JSONL files incrementally without full memory load
5. **Schema Caching**: LRU cache for repeated structure inference (30-50% speedup)
6. **Security Limits**: DoS protection with configurable resource limits

## Installation

```toml
[dependencies]
hedl-json = "1.2"
```

## Bidirectional Conversion

### HEDL → JSON: Export for APIs and LLMs

Convert HEDL's compact representation to JSON when you need API compatibility:

```rust
use hedl_json::{to_json, to_json_value, ToJsonConfig};

let doc = hedl_core::parse(br#"
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
"#)?;

// Configure JSON output
let config = ToJsonConfig {
    include_metadata: false,  // Don't add __type__, __schema__ fields
    flatten_lists: false,      // Keep matrix structure as object arrays
    include_children: true,    // Include nested entities
    ascii_safe: false,         // UTF-8 output (set true for ASCII-only)
};

// Convert to JSON string (for API responses)
let json_str = to_json(&doc, &config)?;
// {"users": [{"id": "alice", "name": "Alice Smith", "email": "alice@example.com"}, ...]}

// Or get serde_json::Value directly (for further processing)
let json_val = to_json_value(&doc, &config)?;
```

**Token Efficiency**: HEDL's matrix notation saves 46.7% tokens compared to verbose JSON arrays. Use HEDL internally, export to JSON only at system boundaries.

### JSON → HEDL: Import from APIs and Files

Parse JSON from external APIs into HEDL's structured data model:

```rust
use hedl_json::{from_json, from_json_value, from_json_value_owned, FromJsonConfig};

// From JSON string (e.g., API response)
let json = r#"{"name": "Alice", "age": 30, "active": true}"#;
let config = FromJsonConfig::default();
let doc = from_json(json, &config)?;

// From serde_json::Value (existing parsed JSON)
let value: serde_json::Value = serde_json::from_str(json)?;

// Borrows the value (value remains usable after conversion)
let doc = from_json_value(&value, &config)?;

// Or takes ownership for zero-copy efficiency
let doc = from_json_value_owned(value, &config)?;
```

## Security Limits: DoS Protection

`FromJsonConfig` enforces resource limits to prevent denial-of-service attacks from malicious JSON. Defaults are intentionally **high** for legitimate ML and data processing workloads:

```rust
use hedl_json::{from_json, FromJsonConfig};

// Default configuration (for trusted internal data)
let default = FromJsonConfig::default();
// max_depth: Some(10,000) levels (deep hierarchies, nested JSON)
// max_array_size: Some(10,000,000) elements (large datasets, batch processing)
// max_string_length: Some(100 MB) (embeddings, base64-encoded data)
// max_object_size: Some(100,000) keys (rich metadata, complex objects)

let json = r#"{"name": "Alice", "age": 30}"#;
let doc = from_json(json, &default)?;
```

For untrusted input (user uploads, external APIs, public endpoints), use stricter limits:

```rust
use hedl_json::{from_json, FromJsonConfig};

// Strict configuration (for untrusted external sources)
let strict = FromJsonConfig::builder()
    .max_depth(100)                        // 100 levels
    .max_array_size(10_000)                // 10K elements
    .max_string_length(1_000_000)          // 1 MB
    .max_object_size(1_000)                // 1K keys
    .build();

let json = r#"{"name": "Bob", "age": 25}"#;
let doc = from_json(json, &strict)?;
```

Exceeding limits returns `JsonConversionError` variants: `MaxDepthExceeded`, `MaxArraySizeExceeded`, `MaxStringLengthExceeded`, `MaxObjectSizeExceeded`.

## Schema Caching: 30-50% Speedup

When converting JSON arrays with repeated structure (common in API responses), `hedl-json` caches inferred schemas automatically:

```rust
use hedl_json::schema_cache::{SchemaCache, SchemaCacheKey};

let cache = SchemaCache::new(100);  // Capacity: 100 schemas

// Cache is used automatically during from_json() for uniform arrays
// Manual cache usage (for advanced control):
let key = SchemaCacheKey::new(vec!["id".to_string(), "name".to_string()]);
cache.insert(key.clone(), vec!["id".to_string(), "name".to_string()]);

if let Some(schema) = cache.get(&key) {
    // Hit: 30-50% faster than re-inferring schema
}

// Monitor cache performance
let stats = cache.statistics();
println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
println!("Hits: {}, Misses: {}, Evictions: {}",
    stats.hits, stats.misses, stats.evictions);
```

For 1000-row JSON arrays with repeated structure, schema caching provides 30-50% speedup over naive inference.

## JSONPath Queries

Query HEDL documents using standard JSONPath syntax (powered by `serde_json_path`):

```rust
use hedl_json::jsonpath::{query, query_first, query_single, query_exists, query_count, QueryConfig};

let doc = hedl_core::parse(br#"
users: @User[id, name, age]
  | alice, Alice Smith, 30
  | bob, Bob Jones, 25
  | carol, Carol White, 35
"#)?;

let config = QueryConfig::default();

// Get all matches
let results = query(&doc, "$.users[?(@.age > 30)].name", &config)?;
// Returns: [serde_json::Value("Carol White")]

// Get first match (returns Option)
let first = query_first(&doc, "$.users[0].name", &config)?;
// Returns: Some(serde_json::Value("Alice Smith"))

// Get exactly one match (errors if 0 or multiple matches)
let single = query_single(&doc, "$.users[?(@.id == 'alice')].name", &config)?;
// Returns: serde_json::Value("Alice Smith")

// Check if any matches exist
let exists = query_exists(&doc, "$.users[?(@.age > 40)]", &config)?;
// Returns: false

// Count matches
let count = query_count(&doc, "$.users[*]", &config)?;
// Returns: 3
```

### QueryConfig Options

```rust
use hedl_json::jsonpath::{QueryConfig, QueryConfigBuilder};

let config = QueryConfig {
    include_metadata: false,   // Don't add __type__ fields in results
    flatten_lists: false,       // Keep matrix structure
    include_children: true,     // Include nested data
    max_results: 100,           // Limit results (0 = unlimited)
};

// Or use builder
let config = QueryConfigBuilder::new()
    .include_metadata(false)
    .max_results(50)
    .build();
```

## JSON Schema Generation

Generate JSON Schema Draft 7 from HEDL documents for validation and documentation:

```rust
use hedl_json::schema_gen::{generate_schema, generate_schema_value, SchemaConfig};

let doc = hedl_core::parse(br#"
%STRUCT: User: [id, name, email, age]
---
users: @User
  | u1, Alice, alice@example.com, 30
"#)?;

let config = SchemaConfig::builder()
    .title("User API Schema")
    .description("Schema for user data endpoint")
    .schema_id("https://api.example.com/schemas/user.json")
    .strict(true)              // disallow additionalProperties
    .include_examples(true)    // add example values from data
    .include_metadata(true)    // include title/description/$id
    .build();

// Generate as JSON string (for documentation)
let schema_json = generate_schema(&doc, &config)?;

// Or as serde_json::Value (for programmatic use)
let schema_value = generate_schema_value(&doc, &config)?;
```

### Smart Type Inference

The schema generator infers JSON Schema formats from actual data:

**Value-Based Inference** (analyzed during schema generation):

```rust
// Field values → JSON Schema format annotation
"alice@example.com"              → {"type": "string", "format": "email"}
"https://example.com"            → {"type": "string", "format": "uri"}
"2024-01-15T10:30:00Z"          → {"type": "string", "format": "date-time"}
"550e8400-e29b-41d4-a716-..."   → {"type": "string", "format": "uuid"}
```

**Name-Based Inference** (fallback when values are ambiguous):

```rust
// Field names → format hints
"email" field      → format: "email"
"url" field        → format: "uri"
"created_at" field → format: "date-time"
"uuid" field       → format: "uuid"
```

### %NEST Relationships in Schemas

HEDL's `%NEST` declarations become nested object arrays in JSON Schema:

```rust
let doc = hedl_core::parse(br#"
%STRUCT: Team: [id, name]
%STRUCT: Member: [id, name, role]
%NEST: Team > Member
---
teams: @Team
  | t1, Engineering
"#)?;

let schema = generate_schema_value(&doc, &SchemaConfig::default())?;
// Team schema includes:
// {
//   "type": "object",
//   "properties": {
//     "id": {"type": "string"},
//     "name": {"type": "string"},
//     "members": {
//       "type": "array",
//       "items": {"$ref": "#/definitions/Member"}
//     }
//   }
// }
```

## Streaming: Process Large JSON Without Full Memory Load

### JSON Array Streaming

Stream elements from large JSON arrays incrementally:

```rust
use hedl_json::streaming::{JsonArrayStreamer, StreamConfig};
use std::fs::File;

// Open large JSON file: [{...}, {...}, {...}, ...]
let file = File::open("large_dataset.json")?;
let config = StreamConfig::default();
let streamer = JsonArrayStreamer::new(file, config)?;

let mut count = 0;
for result in streamer {
    let doc = result?;  // Each array element as HEDL document
    count += 1;
    // Process document: validate, transform, aggregate
}
println!("Processed {} documents", count);
```

**Performance**: Streaming is 1.2-2.1x faster than loading the full array and parsing.

### JSONL (JSON Lines) Streaming

Stream JSONL files line-by-line with robust error handling:

```rust
use hedl_json::streaming::{JsonLinesStreamer, StreamConfig};
use std::fs::File;

let file = File::open("logs.jsonl")?;  // One JSON object per line
let config = StreamConfig::default();
let streamer = JsonLinesStreamer::new(file, config);

for result in streamer {
    match result {
        Ok(doc) => {
            // Process valid log entry
        }
        Err(e) => {
            // Malformed line—log error and continue
            eprintln!("Skipping malformed line {}: {}",
                streamer.line_number(), e);
        }
    }
}
```

**JSONL Features**:
- Blank lines: automatically skipped
- Comments: lines starting with `#` are ignored
- Robust: continues processing on invalid lines (errors returned per line)
- Line tracking: `line_number()` method for debugging

### JSONL Writing

Write HEDL documents as JSONL for streaming output:

```rust
use hedl_json::streaming::JsonLinesWriter;
use std::fs::File;

let file = File::create("output.jsonl")?;
let mut writer = JsonLinesWriter::new(file);

for doc in documents {
    writer.write_document(&doc)?;  // One document per line
}

writer.flush()?;  // Ensure all data written
```

### StreamConfig Options

```rust
use hedl_json::streaming::StreamConfig;
use hedl_json::FromJsonConfig;

let config = StreamConfig {
    buffer_size: 64 * 1024,                 // 64 KB buffer (default)
    max_object_bytes: Some(10 * 1024 * 1024), // 10 MB per object (default)
    from_json: FromJsonConfig::default(),   // Security limits per object
    use_size_estimation: true,              // Efficient size checks (default)
    true_streaming: true,                   // Constant memory for arrays (default)
};

// Or use builder
let config = StreamConfig::builder()
    .buffer_size(128 * 1024)                    // 128 KB buffer
    .max_object_bytes(50 * 1024 * 1024)         // 50 MB per object
    .unlimited_object_size()                    // Disable limit (use with caution)
    .from_json_config(FromJsonConfig::builder()
        .max_depth(100)
        .build())
    .use_size_estimation(true)                  // Efficient size checks
    .true_streaming(true)                       // Constant memory mode
    .build();
```

## Format Mapping

### HEDL → JSON

| HEDL Type | JSON Output | Example |
|-----------|-------------|---------|
| Scalars (null, bool, number, string) | Direct mapping | `null`, `true`, `42`, `"text"` |
| Objects | JSON objects | `{"key": "value"}` |
| Arrays (tensors) | JSON arrays | `[1, 2, 3]` |
| `@User:alice` (reference) | `{"@ref": "@User:alice"}` | Special object format |
| `$(x + 1)` (expression) | `"$(x + 1)"` | String with `$()` wrapper |
| Matrix lists | Arrays of objects | `[{"id": "a", "name": "Alice"}, ...]` |

Example matrix list conversion:

```hedl
users: @User[id, name]
  | alice, Alice
  | bob, Bob
```

Becomes:

```json
{
  "users": [
    {"id": "alice", "name": "Alice"},
    {"id": "bob", "name": "Bob"}
  ]
}
```

### JSON → HEDL

| JSON Type | HEDL Result | Notes |
|-----------|-------------|-------|
| Objects | HEDL objects | Nested structures preserved |
| Arrays | HEDL arrays | Uniform objects become matrix lists |
| `{"@ref": "..."}` | HEDL reference | Special format recognized |
| `"$(...)"` strings | HEDL expression | Pattern triggers expression parsing |
| Primitives | Direct mapping | Null, bool, number, string |

**Schema Inference**: Uniform object arrays are automatically converted to matrix lists with inferred schemas. Fields are sorted alphabetically with `id` first if present.

## Use Cases

**API Integration**: Receive JSON from external APIs, convert to HEDL for structured processing, export back to JSON for responses. Save 46.7% on token costs for LLM API calls.

**Data Pipelines**: Read JSON logs/events, process with HEDL's structured model, export to CSV (`hedl-csv`) or Parquet (`hedl-parquet`) for analytics.

**Configuration Management**: Store configs in HEDL with schema validation (`hedl-lint`), export to JSON for runtime consumption by existing tools.

**LLM Context Optimization**: Convert verbose JSON prompts to HEDL (46.7% token savings), send compact HEDL to LLM provider's API (after JSON conversion at the boundary).

**Schema Documentation**: Generate JSON Schema from HEDL documents for API documentation, OpenAPI specs, and validation tools.

**Log Processing**: Stream large JSONL log files, filter/transform with HEDL's query API, aggregate statistics without full memory load.

## What This Crate Doesn't Do

**Schema Preservation**: JSON has no schema concept. HEDL's `%STRUCT`, `%NEST`, `%ALIAS` declarations are lost in JSON conversion. If you need validation after round-tripping through JSON, redefine schemas explicitly in HEDL.

**Validation**: Converts formats faithfully, doesn't validate data against schemas. For schema validation, use `hedl-lint`.

**Optimization**: Converts structures as-is, not optimally. Verbose JSON becomes verbose HEDL. To leverage HEDL's matrix efficiency, restructure data into uniform arrays intentionally.

**True Array Streaming**: `JsonArrayStreamer` loads the entire JSON array into memory (limitation of `serde_json`). For true incremental processing, use `JsonLinesStreamer` with JSONL format.

## Dependencies

- `serde_json` 1.0 - JSON parsing and serialization
- `serde_json_path` 0.7 - JSONPath query engine
- `hedl-core` 1.0 - HEDL parsing and data model
- `thiserror` 1.0 - Error type definitions

## Performance Characteristics

**Conversion**: HEDL → JSON is serialization-bound. JSON → HEDL is parsing-bound.

**Caching**: Schema inference with caching provides 30-50% speedup for repeated structures in JSON arrays.

**Streaming**:
- JSONL processing is O(1) memory per object
- JSON array streaming loads full array (use JSONL for large files)
- Streaming is 1.2-2.1x faster than full parse for large datasets

**JSONPath**: Query performance depends on `serde_json_path` implementation. Queries execute on JSON representation (HEDL → JSON conversion happens first).

Detailed performance benchmarks are available in the HEDL repository benchmark suite.

## License

Apache-2.0
