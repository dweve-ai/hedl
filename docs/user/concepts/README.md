# HEDL Concepts

Deep-dive explanations of HEDL's design, architecture, and key concepts. These guides help you understand *why* HEDL works the way it does.

## What are Concepts?

Concept guides are **understanding-oriented** and explain the theory behind HEDL. Unlike how-to guides (which solve problems) and tutorials (which teach skills), concept guides illuminate principles and design decisions.

## Core Concepts

### 1. [Data Model](data-model.md)
**Understanding HEDL's structure**

Learn about HEDL's hierarchical entity-based data model:
- Document structure and versioning
- Entities and collections
- Matrix lists and their efficiency
- Nested hierarchies
- Value types and literals

**Read this to understand:**
- How HEDL organizes data
- Why matrix lists are token-efficient
- The relationship between entities and JSON objects
- How nesting works in HEDL

---

### 2. [Type System](type-system.md)
**How HEDL handles types**

Explore HEDL's type inference and validation system:
- Automatic type inference
- Type annotations and schemas
- Type checking and validation
- Polymorphic values
- Type coercion rules

**Read this to understand:**
- How HEDL determines value types
- When to use type annotations
- How type inference improves ergonomics
- Type compatibility between formats

---

### 3. [References](references.md)
**Entity relationships in HEDL**

Master HEDL's reference system for connecting entities:
- Reference syntax (`@Type:id`)
- Type-scoped IDs
- Reference resolution
- Cross-entity relationships
- Circular reference handling

**Read this to understand:**
- How to model relationships
- Why type-scoped IDs matter
- Reference integrity checking
- Graph-like data structures in HEDL

---

### 4. [Canonicalization](canonicalization.md)
**Deterministic formatting**

Learn about HEDL's canonical form:
- What canonicalization means
- Formatting rules
- Deterministic ordering
- Use cases (git, hashing, diffing)
- Comparison with other formats

**Read this to understand:**
- Why canonical form is important
- How HEDL ensures consistency
- Benefits for version control
- Cryptographic hashing of documents

---

## Design Principles

### Token Efficiency

HEDL is designed to minimize token count for LLM applications:

**Key techniques:**
1. **Matrix lists** - Define structure once, not per-item
2. **Ditto operator** - Repeat values without rewriting
3. **Minimal syntax** - Few delimiters and keywords
4. **Type inference** - Omit type annotations when unneeded

**Result:** 35-60% fewer tokens than JSON

---

### Human Readability

Despite being compact, HEDL remains readable:

**Design choices:**
1. **Whitespace-based structure** - Like Python or YAML
2. **Clear column headers** - Self-documenting data
3. **Intuitive syntax** - Minimal learning curve
4. **Comments support** - Document your data

**Result:** Easier to read than JSON for tabular data

---

### Interoperability

HEDL works seamlessly with existing formats:

**Conversion fidelity:**
- JSON ↔ HEDL: Lossless
- YAML ↔ HEDL: Lossless
- CSV ↔ HEDL: Lossless (with schema)
- XML ↔ HEDL: High fidelity
- Parquet ↔ HEDL: Lossless

**Result:** Drop-in replacement for existing workflows

---

## Architecture Overview

### Parsing Pipeline

```
Input Text
    ↓
Lexer (tokenization)
    ↓
Parser (syntax tree)
    ↓
Validator (type checking)
    ↓
Document (in-memory representation)
    ↓
Serializer (output format)
    ↓
Output
```

### Streaming Architecture

```
Input Stream
    ↓
Chunk Reader
    ↓
Incremental Parser
    ↓
Validator (streaming)
    ↓
Transform (optional)
    ↓
Serializer (streaming)
    ↓
Output Stream
```

---

## Key Terminology

| Term | Definition |
|------|------------|
| **Entity** | A named collection of data (like a JSON object key) |
| **Matrix List** | A typed collection with a schema and rows |
| **Type Annotation** | `@TypeName` prefix indicating entity type |
| **Column Definition** | `[col1,col2,col3]` specifying structure |
| **Ditto Operator** | `^` symbol repeating previous value |
| **Reference** | `@Type:id` pointer to another entity |
| **Canonical Form** | Deterministic standardized formatting |
| **Type Inference** | Automatic determination of value types |

---

## Conceptual Comparisons

### HEDL vs JSON

| Aspect | HEDL | JSON |
|--------|------|------|
| **Structure** | Matrix lists + objects | Objects + arrays |
| **Tokens** | 35-60% fewer | Baseline |
| **Readability** | Better for tables | Better for nested |
| **Types** | Strong typing with structs | Schema-less |
| **References** | Built-in with @Type:id | Manual |

### HEDL vs CSV

| Aspect | HEDL | CSV |
|--------|------|------|
| **Nesting** | Full hierarchy | Flat only |
| **Types** | Strong typing | Text only |
| **Metadata** | Version, types | None |
| **Flexibility** | Multiple entities | Single table |

### HEDL vs YAML

| Aspect | HEDL | YAML |
|--------|------|------|
| **Tokens** | Fewer for tables | More verbose |
| **Parsing** | Faster | Slower |
| **Ambiguity** | Less | More (Norway problem) |
| **Tables** | Native support | Manual |

---

## When to Use HEDL

### Ideal Use Cases

1. **LLM Applications** - Minimize token costs
2. **Tabular Data** - More efficient than JSON/YAML
3. **Data Pipelines** - Interoperable format
4. **Configuration** - Human-readable and validated
5. **Knowledge Graphs** - Built-in references

### Not Ideal For

1. **Deeply nested trees** - JSON might be simpler
2. **Binary data** - Use Parquet or binary formats
3. **Streaming logs** - Use JSONL or structured logging
4. **Large text content** - Consider Markdown or plain text

---

## Further Reading

After understanding these concepts:

- **Apply your knowledge:** See [How-To Guides](../how-to/)
- **Learn by doing:** Try the [Tutorials](../tutorials/)
- **Look up specifics:** Check the [Reference](../reference/)
- **Get help:** Read the [FAQ](../faq.md)

---

**Choose a concept to explore:**

- [Data Model](data-model.md) - Structure and organization
- [Type System](type-system.md) - Types and inference
- [References](references.md) - Entity relationships
- [Canonicalization](canonicalization.md) - Deterministic formatting
