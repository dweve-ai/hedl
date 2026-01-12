# HEDL - Hierarchical Entity Data Language

**A high-performance data serialization format optimized for AI/ML applications**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Documentation](https://img.shields.io/badge/docs-latest-blue.svg)]()

---

## Overview

Every API call to GPT-5.2 costs $1.75 per million input tokens. Every context window has limits. Every byte transmitted adds latency.

**HEDL (Hierarchical Entity Data Language)** solves the fundamental tradeoff between token efficiency and data comprehension. While CSV is compact but loses structure, and JSON is expressive but verbose, HEDL delivers both: **62.2% LLM comprehension** (nearly matching JSON's 65%) while using **half the tokens**.

The result? When LLMs process HEDL, they get **93% more correct answers per token** than JSON. For high-volume AI applications, this isn't just an optimization—it's the difference between viable and prohibitively expensive.

HEDL combines CSV-style tabular efficiency with hierarchical structure, schema validation, and first-class support for references and relationships. It's what you'd design if you started from "how do LLMs actually parse data?" instead of "how did we do this in 1999?"

### Why HEDL?

**The Efficiency Story**: Test across GPT-5.1, Mistral Large 3, and DeepSeek v3.2. Ask 65 questions about structured data in different formats. HEDL delivers 23.89 correct answers per 1,000 tokens. JSON? 12.36. That's 93% more value per token. At scale, this compounds into dramatic cost savings.

**The Accuracy Story**: HEDL achieves 62.2% comprehension accuracy—only 2.7 percentage points behind JSON's 65%. But here's the key: HEDL does this with half the tokens (2,605 vs 5,253). It's not about choosing between accuracy and efficiency anymore.

**The Developer Story**: Schema validation catches errors before they reach production. LSP integration means autocomplete, validation, and hover docs in your editor. Type-safe references prevent broken relationships. It's the tooling you'd expect from a modern format, not a verbose interchange format from the '90s.

**The Ecosystem Story**: Parse, validate, lint, canonicalize, and convert to JSON, YAML, XML, CSV, Parquet, Neo4j Cypher, and TOON. Streaming for multi-GB files. FFI bindings for C/C++/Python. WASM for browsers. MCP server for AI agents. Built for real systems, not toy examples.

---

## Quick Start

**Install**: Add HEDL to your project in 30 seconds:

```toml
[dependencies]
hedl = "1.0.0"

# Or with all format converters
hedl = { version = "1.0.0", features = ["all-formats"] }
```

**Use**: Four core operations get you 90% of the way:

```rust
use hedl::{parse, to_json, canonicalize, validate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hedl_text = r#"
%VERSION: 1.0
%STRUCT: Product: [id, name, price, category]
---
products: @Product
  | laptop, ThinkPad X1, 1299.99, electronics
  | mouse, Wireless Mouse, 29.99, accessories
  | keyboard, Mechanical Keyboard, 149.99, accessories

store_name: Tech Depot
location: Amsterdam
"#;

    // Parse to AST (~1.5µs per record)
    let doc = parse(hedl_text)?;

    // Validate schema (0.5% overhead)
    validate(hedl_text)?;

    // Convert to JSON for existing APIs
    let json = to_json(&doc)?;

    // Canonicalize for version control
    let canonical = canonicalize(&doc)?;

    Ok(())
}
```

That's it. Parse, validate, convert, format. Everything else builds on these primitives.

---

## See The Difference

Here's the same data in HEDL and JSON. Notice how HEDL uses structured types and table syntax to eliminate JSON's repetitive key names:

### HEDL Syntax

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email, age]
%STRUCT: Post: [id, author, title, tags]
---
# Users with type-scoped IDs
users: @User
  | alice, Alice Smith, alice@example.com, 28
  | bob, Bob Johnson, bob@example.com, 35

# Posts with references to users
posts: @Post
  | p1, @User:alice, First Post, [tech, rust]
  | p2, @User:bob, Another Post, [programming, web]

# Ditto operator for repeated values
events: @Event
  | e1, 2024-01-15, conference, Berlin
  | e2, ^, workshop, ^
  | e3, 2024-01-16, meetup, ^
```

### Equivalent JSON

```json
{
  "users": [
    {"id": "alice", "name": "Alice Smith", "email": "alice@example.com", "age": 28},
    {"id": "bob", "name": "Bob Johnson", "email": "bob@example.com", "age": 35}
  ],
  "posts": [
    {"id": "p1", "author": {"@ref": "User:alice"}, "title": "First Post", "tags": ["tech", "rust"]},
    {"id": "p2", "author": {"@ref": "User:bob"}, "title": "Another Post", "tags": ["programming", "web"]}
  ],
  "events": [
    {"id": "e1", "date": "2024-01-15", "type": "conference", "location": "Berlin"},
    {"id": "e2", "date": "2024-01-15", "type": "workshop", "location": "Berlin"},
    {"id": "e3", "date": "2024-01-16", "type": "meetup", "location": "Berlin"}
  ]
}
```

**Token Savings**: 373 tokens (HEDL) vs 557 tokens (JSON) = **33% reduction** for this example

Notice what's eliminated: In JSON, you repeat `"id":`, `"name":`, `"email":` for every user. HEDL declares the structure once with `%STRUCT`, then each row is just values. The ditto operator (`^`) removes repetition in the events table. References use clean `@Type:id` syntax instead of verbose `{"@ref": "..."}` objects.

*Across real datasets, HEDL saves 46.7% tokens on average vs JSON (see [Performance](#performance) section)*

---

## Interoperability: HEDL Plays Well With Others

HEDL isn't a walled garden. Your data probably exists in multiple formats already. Your systems speak different protocols. You need a format that converts seamlessly without losing information.

| Format | Read | Write | Streaming | When You Need It |
|--------|------|-------|-----------|------------------|
| **JSON** | ✅ | ✅ | ✅ | REST APIs, web services, JavaScript frontends |
| **YAML** | ✅ | ✅ | ❌ | Kubernetes configs, CI/CD pipelines, human-edited files |
| **XML** | ✅ | ✅ | ✅ | SOAP services, enterprise systems, regulatory formats |
| **CSV** | ✅ | ✅ | ❌ | Excel exports, data analysis, simple tabular data |
| **Parquet** | ✅ | ✅ | ❌ | Data lakes, analytics pipelines, columnar storage |
| **Neo4j Cypher** | ✅ | ✅ | ✅ | Loading graph databases, relationship-heavy data |
| **TOON** | ✅ | ✅ | ❌ | Maximum token efficiency for LLM contexts |

Convert in both directions. Preserve semantics. Stream when you can, batch when you need to. The format adapts to your workflow.

---

## Command-Line Power User Tools

Install once, use everywhere:

```bash
cargo install hedl-cli
```

**Validation in CI/CD**: Catch schema errors before they reach production. `hedl validate` returns non-zero exit codes for invalid documents—perfect for pre-commit hooks and CI pipelines.

```bash
hedl validate config/*.hedl && echo "✓ All configs valid"
```

**Format Conversion at Scale**: Converting 1,000 files from HEDL to JSON? Batch operations maintain 98.6% efficiency:

```bash
hedl to-json data/*.hedl -o json/
```

**Deterministic Formatting**: Code review diffs are noisy when everyone's editor formats differently. Canonicalization solves this:

```bash
hedl format document.hedl -o canonical.hedl
# Same input always produces identical output
```

**Linting Best Practices**: Catch unused schemas, inconsistent naming, and anti-patterns:

```bash
hedl lint document.hedl
# Warning: unused struct definition 'OldUser'
# Warning: unqualified reference 'alice' (use @User:alice)
```

**Quick Stats**: How big is this document? How many entities? What's the nesting depth?

```bash
hedl stats large-file.hedl
# 1,247 entities, 3 levels deep, 15 struct types
```

---

## Ecosystem Integration

HEDL is written in Rust, but you're not locked into the Rust ecosystem. Use it from any language, any platform, any environment.

### C/C++/Python: FFI Bindings

Your Python service needs to parse HEDL. Your C++ backend needs to export HEDL. FFI overhead is 3.65%, negligible in real workloads.

```c
#include "hedl.h"

HedlDocument* doc = hedl_parse(hedl_text, text_len);
char* json = hedl_to_json(doc);
// Process JSON in your existing C/C++/Python code
hedl_free_string(json);
hedl_free_document(doc);
```

No memory leaks. Thread-safe. Production-tested.

### Browsers and Node.js: WebAssembly

Your web app needs client-side HEDL parsing. Your Node.js service needs format conversion without spawning processes.

```typescript
import init, { parse, toJson } from 'hedl-wasm';

await init();
const doc = parse(hedlText);
const json = toJson(doc);
// Use in React, Vue, Angular, or vanilla JS
```

WASM module loads in milliseconds. Zero-copy where possible. Same parser as the native Rust implementation.

### Editor Integration: Language Server Protocol

You're editing a 5,000-line HEDL config file. You mistype a struct name. You want autocomplete for entity IDs. You need to jump to a definition.

HEDL's LSP server gives you:
- **Syntax highlighting**: Distinguish structs, references, and values at a glance
- **Auto-completion**: Type `@Us` and get `@User:` suggestions
- **Real-time validation**: Red squiggles on invalid references before you save
- **Go-to-definition**: Click `@User:alice` to jump to alice's definition
- **Hover documentation**: See struct schemas without scrolling
- **Quick fixes**: "Unqualified reference—add @User prefix?"

Configure your editor (VSCode, Neovim, Emacs, Helix) to use `hedl-lsp` for `.hedl` files. Latency under 10ms for typical operations.

### AI Agent Integration: Model Context Protocol

You're building an AI agent that needs to read, transform, and validate structured data. MCP makes HEDL a first-class citizen in LLM tool use.

```bash
hedl-mcp --port 8080
```

Your agent can now:
- Parse and validate HEDL documents
- Convert between formats (HEDL ↔ JSON/YAML/CSV)
- Infer schemas from untyped data
- Query and transform structured data

MCP server handles 50+ concurrent requests. 2ms average latency. Cache hit rate 85%.

---

## Performance

Performance isn't just about raw speed—it's about scalability, predictability, and real-world throughput. HEDL is designed for production workloads where both latency and efficiency matter:

**Core Operations:**

| Operation | Throughput | Latency (Small Doc) |
|-----------|------------|---------------------|
| Parsing | 54.6 MB/s | 142 µs |
| Canonicalization | N/A | 30 µs |
| Linting | 72-931 MB/s | 3.67 µs |

**Format Conversion (HEDL → Other):**

| Target Format | Throughput | Latency |
|---------------|------------|---------|
| JSON | 1,549 MB/s | 292 µs |
| YAML | 246 MB/s | 1,834 µs |
| XML | 2,964 MB/s | 153 µs |
| CSV | Fast | Low overhead |

**Format Conversion (Other → HEDL):**

| Source Format | Throughput | Latency |
|---------------|------------|---------|
| JSON | 2,883 MB/s | 442 µs |
| YAML | 377 MB/s | 3,011 µs |
| XML | 953 MB/s | 1,130 µs |
| CSV | Fast | Low overhead |

### The LLM Comprehension Test

We tested 6 formats across 3 leading LLMs (GPT-5.1, Mistral Large 3, DeepSeek v3.2) with 65 questions about structured data. The results reveal a clear pattern: **accuracy costs tokens, but HEDL breaks that tradeoff**.

**The Results:**

| Format | Avg Accuracy | Tokens/Question | Accuracy per 1k tokens |
|--------|--------------|-----------------|------------------------|
| **JSON** | 64.95% 🥇 | 5,253 | 12.36 |
| **HEDL** | **62.23%** 🥈 | 2,605 | **23.89** 🥇 |
| **YAML** | 61.53% | 5,367 | 11.46 |
| **TOON** | 58.79% | 2,904 | 20.24 |
| **XML** | 50.42% | 4,599 | 10.96 |
| **CSV** | 23.56% | 1,188 | 19.83 |

**What This Means:**

JSON wins on raw accuracy (64.95%), but pays a steep token tax. HEDL achieves 62.2% accuracy—only 2.7 points behind—while using less than half the tokens. The efficiency metric tells the real story: **23.89 vs 12.36 correct answers per 1k tokens**. That's 93% better efficiency.

TOON, the previous token-efficiency champion, uses slightly more tokens than HEDL (2,904 vs 2,605) but scores 3.4 points lower on accuracy. YAML and XML are verbose without improving comprehension. CSV is compact but structurally impoverished—only 23.6% accuracy.

**HEDL vs TOON detailed comparison:**

| LLM Model | HEDL Accuracy | TOON Accuracy | HEDL Advantage |
|-----------|---------------|---------------|----------------|
| **GPT-5.1** | **71.8% ± 3.2%** | 68.2% ± 0.7% | **+3.6 points** 🥇 |
| **Mistral Large 3** | **51.8% ± 0.7%** | 45.1% ± 1.5% | **+6.7 points** 🥇 |
| **DeepSeek v3.2** | **63.1% ± 0.0%** | 63.1% ± 0.0% | **Tie** |
| **Average** | **62.2%** | **58.8%** | **+3.4 points** 🥇 |

> **Why HEDL wins**: HEDL's structured format (typed columns, references, hierarchy) enables LLMs to parse and comprehend data more reliably than minimalist formats, while maintaining exceptional token efficiency.

### Token Economics: HEDL vs JSON

Real data from 12 production-style datasets, tokenized with tiktoken (OpenAI's tokenizer). The savings compound across different data structures:

| Dataset Type | HEDL Tokens | JSON Tokens | Token Savings |
|--------------|-------------|-------------|---------------|
| **users_flat** | 3,409 | 6,478 | **47.4%** |
| **products_flat** | 3,106 | 6,219 | **50.1%** |
| **blog_nested** | 5,201 | 9,738 | **46.6%** |
| **orders_nested** | 835 | 1,600 | **47.8%** |
| **config_simple** | 237 | 476 | **50.2%** |
| **Average** | - | - | **46.7%** |

**Size Efficiency** - Storage and bandwidth:

| Dataset Type | HEDL Bytes | JSON Bytes | Size Savings | Ratio |
|--------------|------------|------------|--------------|-------|
| **users_flat** | ~81KB | ~180KB | **55.0%** | 2.2x smaller |
| **products_flat** | ~89KB | ~200KB | **55.5%** | 2.2x smaller |
| **blog_nested** | ~71KB | ~155KB | **54.5%** | 2.2x smaller |
| **Average** | - | - | **57.7%** | **2.4x smaller** |

**Conversion Performance** (bidirectional):

| Direction | Throughput | Latency |
|-----------|------------|---------|
| **HEDL → JSON** | 1,549 MB/s | 291.73 µs avg |
| **JSON → HEDL** | 2,883 MB/s | 442.43 µs avg |

**The Cost Calculation**: At GPT-5.2 pricing ($1.75/1M input tokens), a 1M token context in JSON costs $1.75. That same data in HEDL? $0.93. Scale that across millions of API calls, and token efficiency isn't academic—it's bottom-line impact. For a service processing 1B tokens monthly, HEDL saves approximately $820/month compared to JSON.

### Performance Characteristics

- **Linear Scaling**: O(1) per document, O(depth) for nesting - no exponential blowup
- **Zero-Copy Optimizations**: 5,550 allocations saved (33% reduction) for simple strings
- **Parallel Processing**: 6.19x speedup @ 8 threads, 98.6% batch efficiency
- **Streaming Support**: 1.2-2.1x faster than full parse for incremental processing
- **Peak Throughput**: 78.8 MB/s, LSP latency <10ms, MCP latency 2ms

---

## Modular by Design

HEDL is built as 19 specialized crates, not a monolith. Need JSON conversion but not XML? Only pay for what you use. Building a web service? Skip the CLI. Embedding in Python? Just the FFI layer.

**Core Components** - Start here:
- `hedl-core`: The parser. Zero dependencies beyond Rust std. Parse to AST in ~1.5µs per record.
- `hedl`: High-level API. Parse, validate, convert. Most users import only this.
- `hedl-stream`: Streaming parser for multi-GB files. 1.2-2.1x faster than full-document parsing.

**Format Converters** - Pick your targets:
- `hedl-json`, `hedl-yaml`, `hedl-xml`: Bidirectional conversion for web standards
- `hedl-csv`: Export tables to Excel-compatible formats
- `hedl-parquet`: Columnar storage for analytics pipelines
- `hedl-neo4j`: Generate Cypher for graph database loading (0.7ms per 1K nodes)
- `hedl-toon`: Token-optimized output for LLM contexts

**Developer Tools** - Catch errors early:
- `hedl-c14n`: Deterministic formatting. Same input → identical output. Git-friendly diffs.
- `hedl-lint`: Best-practice enforcement. 1% false positive rate. Sub-microsecond per rule.
- `hedl-cli`: Command-line Swiss Army knife. Validate, convert, lint, format, analyze.
- `hedl-test`: Shared test fixtures and property testing utilities.

**Cross-Language Integration** - Use everywhere:
- `hedl-ffi`: C ABI for Python/C++/Go/Node.js. 3.65% overhead. Zero memory leaks.
- `hedl-wasm`: Browser and Node.js. Same parser, compiles to WebAssembly.
- `hedl-lsp`: Editor integration. <10ms latency. Works with VSCode, Vim, Emacs.
- `hedl-mcp`: AI agent protocol. 50+ concurrent requests. 85% cache hit rate.

**Quality Assurance** - Trust but verify:
- `hedl-bench`: Criterion-based benchmarks. Regression detection. All numbers in this README are from these benchmarks.

---

## Learn More

**Start here**: [Language Specification](SPEC.md) - Complete HEDL syntax with examples and rationale for design decisions.

**Going deeper**:
- [API Documentation](https://docs.rs/hedl) - Rust API reference with examples
- [CLI Reference](crates/hedl-cli/README.md) - All command-line flags and batch operations
- [FFI Guide](crates/hedl-ffi/README.md) - Integrate with C/C++/Python/Go
- [WASM Guide](crates/hedl-wasm/README.md) - Use HEDL in browsers and Node.js

**Questions?** Open an [issue](https://github.com/dweve-ai/hedl/issues) or join the discussion.

---

## Where HEDL Shines

### High-Volume AI Services

You're building a RAG system that processes millions of API calls monthly. Every retrieval includes structured metadata—user context, document attributes, relationship graphs. With JSON, you're burning tokens on repetitive key names. With HEDL, you cut token usage by 46.7% while maintaining near-JSON comprehension (62.2% vs 65%). The efficiency gain (93% more answers per token) means your service scales further before hitting cost constraints.

### Real-Time Data Pipelines

Your ETL pipeline processes streaming data from multiple sources, converts formats, validates schemas, and exports to Neo4j and Parquet. HEDL's streaming parser is 1.2-2.1x faster than full document parsing. Batch processing maintains 98.6% efficiency at 100x scale. Parallel processing delivers 6.19x speedup on 8 cores. Convert to Neo4j Cypher at 0.7ms for 1,000 nodes. The format adapts to your infrastructure, not the other way around.

### Knowledge Graph Construction

You're building a graph database from heterogeneous sources. HEDL's typed references (@Type:id syntax) and first-class relationship support make entity resolution straightforward. Reference resolution processes 40 refs/ms with linear scaling—no exponential blowup as your graph grows. Export directly to Neo4j Cypher without intermediate transformations.

### Configuration-as-Code

Your service config spans multiple environments with complex nested structures. HEDL's schema validation catches typos and type errors before deployment. LSP integration provides autocomplete and hover docs in your editor. The ditto operator (^) eliminates repetition in tabular config. Deterministic canonicalization makes diffs readable. It's infrastructure-as-code that doesn't fight your workflow.

### When NOT to Use HEDL

**Maximum accuracy is non-negotiable**: JSON scores 2.7 percentage points higher on LLM comprehension. If those 2.7 points matter more than 50% token savings, stick with JSON.

**Ecosystem lock-in matters**: JSON has decades of tooling, libraries, and developer familiarity. Every language has multiple battle-tested JSON parsers. HEDL is new. If you need maximum ecosystem compatibility today, JSON is the safe choice.

---

## Contributing

HEDL is open source (Apache 2.0) and welcomes contributions. Found a bug? Have an idea for a new format converter? Want to optimize the parser?

**Quick start**:
```bash
git clone https://github.com/dweve-ai/hedl.git
cd hedl
cargo build --all-features
cargo test --all-features
```

**Quality bar**: We care about correctness, performance, and maintainability. Every PR goes through:
- Unit tests (all public APIs tested)
- Integration tests (cross-crate workflows)
- Property tests (fuzz testing with `proptest`)
- Benchmark regression checks (no performance regressions)
- Security review (input validation, resource limits)

**Areas we need help**:
- Format converters for your favorite format
- Performance optimization (SIMD, zero-copy, cache efficiency)
- Language bindings (Dart, Ruby, Zig, etc.)
- LSP features (refactoring, code actions)
- Documentation and examples

See an issue labeled "good first issue" or "help wanted"? That's a great place to start.

Maintained by [Dweve B.V.](https://dweve.com) and the open-source community.

---

## License & Links

Licensed under **Apache License 2.0**. Copyright © 2025 Dweve IP B.V. and contributors.

**Find us**: [Homepage](https://dweve.com) · [GitHub](https://github.com/dweve-ai/hedl) · [Crates.io](https://crates.io/crates/hedl) · [API Docs](https://docs.rs/hedl) · [Issues](https://github.com/dweve-ai/hedl/issues)

**Built on**: Rust stdlib · serde · quick-xml · parquet · criterion · tower-lsp

---

**Built with Rust 🦀 · Optimized for AI 🤖**
