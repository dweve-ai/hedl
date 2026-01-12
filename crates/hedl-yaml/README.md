# hedl-yaml

**HEDL's bridge to the YAML ecosystem—bidirectional conversion with metadata awareness.**

YAML dominates DevOps configuration: Kubernetes manifests, GitHub Actions workflows, Ansible playbooks, Docker Compose files. Your infrastructure runs on it. Your CI/CD pipelines depend on it. But YAML lacks schemas, types, and structure enforcement.

`hedl-yaml` integrates HEDL with the YAML ecosystem. Convert YAML configuration to HEDL documents for structured editing, validation, and analysis. Export HEDL back to YAML when you need compatibility with existing tools. Work with HEDL's typed matrices and entity references, then seamlessly deploy as YAML.

Part of the **HEDL format family** alongside `hedl-json`, `hedl-xml`, `hedl-csv`, and `hedl-parquet`—bringing HEDL's efficiency and structure to every ecosystem you work in.

## Critical: Metadata Loss in YAML Round-Trips

YAML is a data serialization format, not a schema language. HEDL's schema declarations cannot be preserved in YAML:

**Lost in YAML Conversion**:
- `%STRUCT` schema definitions (field types, constraints, validation rules)
- `%NEST` hierarchy declarations (parent-child relationships)
- `%ALIAS` type aliases (semantic type names)
- Validation constraints (min/max, format patterns, required fields)
- Document metadata (version, author, license)

**Preserved in YAML Conversion**:
- All data values (strings, numbers, booleans, null)
- Entity references (`@User:alice` → `{"@ref": "@User:alice"}`)
- Computed expressions (`$(revenue * 0.1)` → `"$(revenue * 0.1)"`)
- Tensor arrays (multi-dimensional numeric data)
- Object structure and nested hierarchies
- Matrix lists (converted to object arrays with inferred schemas)

When converting HEDL → YAML → HEDL, you get the data back but schemas must be redefined. If you need schema validation after round-tripping through YAML, use HEDL's schema system (`%STRUCT` declarations) with `hedl-lint` for validation.

## Installation

```toml
[dependencies]
hedl-yaml = "1.0"
```

## Bidirectional Conversion

### HEDL → YAML: Export for DevOps Tools

Convert HEDL documents to YAML for deployment in existing infrastructure:

```rust
use hedl_yaml::{to_yaml, ToYamlConfig};

let doc = hedl_core::parse(br#"
%STRUCT: Service: [name, image, port]
%NEST: Service > Environment
---
services: @Service
  | web, nginx:latest, 80
  | api, node:18, 3000
"#)?;

// Configure YAML output
let config = ToYamlConfig {
    include_metadata: true,   // Add __type__ hints for reconstruction
    flatten_lists: false,      // Preserve matrix structure
    include_children: true,    // Include nested entities
};

let yaml_output = to_yaml(&doc, &config)?;
```

Generated YAML (ready for Docker Compose, Kubernetes, etc.):

```yaml
services:
  - name: web
    image: nginx:latest
    port: 80
    __type__: Service
  - name: api
    image: node:18
    port: 3000
    __type__: Service
```

### YAML → HEDL: Import Configuration for Analysis

Parse existing YAML configuration as structured HEDL documents:

```rust
use hedl_yaml::{from_yaml, FromYamlConfig};

let yaml = r#"
database:
  host: localhost
  port: 5432
  credentials:
    username: admin
    password: secret
replicas: 3
features:
  - caching
  - monitoring
"#;

let config = FromYamlConfig::default();
let hedl_doc = from_yaml(yaml, &config)?;

// Now use HEDL's structured API to query, validate, transform
```

## Security Limits: DoS Protection

`FromYamlConfig` enforces limits to prevent denial-of-service attacks from malicious YAML files. Defaults are intentionally **high** for legitimate use cases (large Kubernetes manifests, CI/CD configs, data processing):

```rust
use hedl_yaml::FromYamlConfig;

// Default configuration (500 MB documents, 10M elements, 10K depth)
let default_config = FromYamlConfig::default();
// - max_document_size: 500 MB (handles large Kubernetes manifests)
// - max_array_length: 10,000,000 elements (data processing workloads)
// - max_nesting_depth: 10,000 levels (complex configuration hierarchies)

let doc = from_yaml(large_yaml, &default_config)?;
```

For untrusted input (user uploads, external APIs), use stricter limits:

```rust
// Strict configuration for untrusted sources
let strict_config = FromYamlConfig::builder()
    .max_document_size(10 * 1024 * 1024)  // 10 MB limit
    .max_array_length(100_000)             // 100K elements
    .max_nesting_depth(100)                // 100 levels
    .build();

let doc = from_yaml(untrusted_yaml, &strict_config)?;
```

Exceeding limits raises `YamlError::ResourceLimitExceeded`, `MaxDepthExceeded`, `DocumentTooLarge`, or `ArrayTooLong`.

## Configuration Options

### ToYamlConfig: HEDL → YAML Conversion

Control how HEDL structures are represented in YAML:

```rust
use hedl_yaml::ToYamlConfig;

let config = ToYamlConfig {
    include_metadata: true,   // Add __type__ and __schema__ hints
    flatten_lists: false,      // Keep structured matrix lists vs plain arrays
    include_children: true,    // Include nested entities in output
};
```

**include_metadata**: When `true`, adds `__type__` (entity type name) and `__schema__` (field names) to YAML output. Helps reconstruction but clutters output. Use for round-tripping, omit for clean deployment YAML.

**flatten_lists**: When `false`, matrix lists remain as arrays of objects. When `true`, converts to plain arrays where possible (loses structure).

**include_children**: When `true`, nested entities are included in parent output. When `false`, only top-level entities are exported.

### FromYamlConfig: YAML → HEDL Conversion

Uses a builder pattern for security limit configuration:

```rust
use hedl_yaml::FromYamlConfig;

let config = FromYamlConfig::builder()
    .max_document_size(100 * 1024 * 1024)  // 100 MB
    .max_array_length(1_000_000)            // 1M elements
    .max_nesting_depth(500)                 // 500 levels
    .build();
```

## YAML-Specific Features

### Anchors and Aliases

YAML anchors and aliases are automatically resolved during parsing (handled by `serde_yaml`):

```yaml
defaults: &defaults
  timeout: 30
  retries: 3
  log_level: info

production:
  <<: *defaults
  host: prod.example.com

staging:
  <<: *defaults
  host: staging.example.com
```

Your HEDL document receives the fully expanded data with all anchor references resolved.

### Multi-Line Strings

YAML's literal (`|`) and folded (`>`) block scalars work as expected:

```yaml
description: |
  This is a multi-line
  string with preserved
  line breaks.

summary: >
  This is a folded
  multi-line string
  that becomes one line.
```

HEDL preserves the string content with appropriate whitespace handling.

## Format Mapping

### HEDL → YAML

| HEDL Type | YAML Output | Notes |
|-----------|-------------|-------|
| Scalars (null, bool, number, string) | Direct mapping | Native YAML types |
| Objects | YAML mappings | Key-value pairs |
| Arrays (tensors) | YAML sequences | Multi-dimensional arrays flattened |
| `@User:alice` (reference) | `{"@ref": "@User:alice"}` | Mapping to distinguish from strings |
| `$(expr)` (expression) | `"$(expr)"` | String with `$()` wrapper |
| Matrix lists | Array of mappings | Optional `__type__` and `__schema__` metadata |

Example:

```hedl
users: @User[id, name]
  | alice, Alice Smith
  | bob, Bob Jones
```

Becomes:

```yaml
users:
  - id: alice
    name: Alice Smith
  - id: bob
    name: Bob Jones
```

### YAML → HEDL

| YAML Type | HEDL Result | Notes |
|-----------|-------------|-------|
| Mappings | HEDL objects | Nested structures preserved |
| Sequences | HEDL arrays | Uniform objects become matrix lists |
| `{"@ref": "..."}` | HEDL reference | Special mapping format recognized |
| `"$(...)"` | HEDL expression | String pattern triggers expression parsing |
| Primitives | Direct mapping | Null, bool, number, string |

Schema inference for matrix lists: When converting uniform object arrays, field names are collected and sorted alphabetically with `id` first if present.

## Round-Trip Behavior

```rust
use hedl_yaml::{to_yaml, from_yaml, ToYamlConfig, FromYamlConfig};

// Original HEDL with schema declaration
let original = hedl_core::parse(br#"
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
"#)?;

// Convert to YAML
let yaml = to_yaml(&original, &ToYamlConfig::default())?;
// YAML contains data but not %STRUCT declaration

// Convert back to HEDL
let restored = from_yaml(&yaml, &FromYamlConfig::default())?;
// Data preserved: users with alice and bob entries
// Schema INFERRED: [email, id, name] (alphabetically sorted with id first)
// Schema LOST: Original %STRUCT declaration and field order
```

The restored document has all data values and a matrix list with **inferred** schema. The original `%STRUCT` declaration is gone—if you need validation, redefine schemas in HEDL.

## Use Cases

**Configuration Migration**: Convert legacy YAML configs to HEDL for structured editing with LSP support, schema validation, and type safety. Export back to YAML for deployment.

**Kubernetes Manifest Analysis**: Parse thousands of K8s manifests as HEDL documents, query with structured access, validate against organizational policies, generate compliance reports.

**CI/CD Pipeline Generation**: Define pipelines in HEDL with schema validation and reusable templates. Export to YAML for GitHub Actions, GitLab CI, or CircleCI execution.

**Ansible Playbook Refactoring**: Convert playbooks to HEDL for programmatic manipulation, deduplication, and optimization. Export back to YAML for execution.

**Multi-Format Workflows**: Read YAML configs, combine with JSON APIs (`hedl-json`), query with structured access, export to CSV (`hedl-csv`) for reporting—all through HEDL's unified data model.

## What This Crate Doesn't Do

**Schema Preservation**: YAML has no schema concept. HEDL's `%STRUCT`, `%NEST`, and `%ALIAS` declarations are lost in conversion. If you need schema validation after YAML round-tripping, redefine schemas explicitly in HEDL.

**Validation**: Converts between formats faithfully, doesn't validate data. For schema validation against HEDL rules, use `hedl-lint`.

**Optimization**: Converts structures as-is, not optimally. Verbose YAML becomes verbose HEDL. To leverage HEDL's matrix list efficiency, restructure data intentionally into uniform arrays.

**YAML Comments**: Comments in YAML files are discarded during parsing (limitation of `serde_yaml`). Use HEDL's comment syntax in `.hedl` files for preserved documentation.

## Dependencies

- `serde_yaml` 0.9 - YAML parsing and serialization (handles anchors, aliases, multi-line strings)
- `hedl-core` 1.0 - HEDL parsing and data model
- `thiserror` 1.0 - Error type definitions

## Performance Characteristics

**Conversion Speed**: HEDL → YAML is serialization-bound (~200-400 MB/s). YAML → HEDL is parsing-bound (~100-200 MB/s depending on document complexity).

**Memory Usage**: `from_yaml()` loads entire document into memory (YAML spec limitation). For large datasets, consider `hedl-json` with streaming support or split YAML files.

**Schema Inference**: Uniform object arrays trigger schema inference with O(n*k) complexity (n objects, k unique keys). For 10K objects with 20 fields, inference adds ~1-2ms overhead.

Detailed performance benchmarks are available in the HEDL repository benchmark suite.

## License

Apache-2.0
