# Concept: Data Model

Understanding HEDL's hierarchical entity-based data model.

## Overview

HEDL organizes data into a hierarchy of entities with strong typing and minimal redundancy. This design combines the best aspects of relational tables (matrix lists), object structures (entities), and document databases (flexible nesting).

## Document Structure

### Basic Anatomy

Every HEDL document has three parts:

1. **Header** - Version declaration
2. **Entities** - Named data collections
3. **Values** - Actual data

**Example:**
```hedl
%VERSION: 1.0                                ← Header
%STRUCT: User: [id, name, email]             ← Struct declaration
---                                          ← Separator
users: @User                                 ← Entity declaration
  | u1, Alice, alice@example.com             ← Values (pipe-prefixed rows)
  | u2, Bob, bob@example.com
```

### Version Header

The first line specifies the HEDL version. For HEDL 1.0, the version is typically declared via directives:

```hedl
%VERSION: 1.0
```

This declares the HEDL version for:
- **Forward compatibility** - Parsers know what features to expect
- **Schema evolution** - Future versions can change syntax
- **Validation** - Ensure document meets version requirements

Note: In the actual Document structure, version is stored as a tuple `(u32, u32)` representing (major, minor).

## Entities

### What is an Entity?

An **entity** is a named collection of data, similar to:
- A JSON object key with array value
- A database table
- A YAML mapping key

**HEDL:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
  | u2, Bob
```

**JSON equivalent:**
```json
{
  "users": [
    {"id": "u1", "name": "Alice"},
    {"id": "u2", "name": "Bob"}
  ]
}
```

### Entity Types

Entities can be:

1. **Matrix lists** - Tabular data with schema
2. **Objects** - Single structured item
3. **Scalars** - Single values

## Matrix Lists

### Why Matrix Lists?

Matrix lists are HEDL's key innovation for token efficiency.

**Traditional approach (JSON):**
```json
{"users": [
  {"id": "u1", "name": "Alice", "email": "alice@example.com"},
  {"id": "u2", "name": "Bob", "email": "bob@example.com"},
  {"id": "u3", "name": "Carol", "email": "carol@example.com"}
]}
```
**Tokens:** ~95

**Matrix list approach (HEDL):**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, bob@example.com
  | u3, Carol, carol@example.com
```
**Tokens:** ~42 (56% fewer!)

### Matrix List Syntax

```hedl
%VERSION: 1.0
%STRUCT: TypeName: [col1, col2, col3]
---
entity_name: @TypeName
  | id1, value1, value2
  | id2, value3, value4
```

**Parts:**
- `%STRUCT:` - Declares the type and its columns in the header
- `entity_name:` - Key for this collection (with colon)
- `@TypeName` - References the declared struct type
- `| ` - Each row starts with pipe and space
- Comma-separated values matching columns

### Column Definitions

Columns define the structure once for all rows:

```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, category, price, in_stock]
---
products: @Product
  | p1, Laptop, Electronics, 999.99, true
  | p2, Mouse, Accessories, 29.99, true
  | p3, Keyboard, Accessories, 79.99, false
```

**Benefits:**
1. Field names not repeated (token savings)
2. Schema is explicit (validation)
3. Data is aligned (readability)
4. Type inference is easier

## Nested Hierarchies

### Simple Nesting

Entities can contain other entities:

```hedl
%VERSION: 1.0
%STRUCT: Employee: [id, name, role]
---
company:
  name: Acme Corp
  employees: @Employee
    | e1, Alice, Engineer
    | e2, Bob, Designer
```

**JSON equivalent:**
```json
{
  "company": {
    "name": "Acme Corp",
    "employees": [
      {"id": "e1", "name": "Alice", "role": "Engineer"},
      {"id": "e2", "name": "Bob", "role": "Designer"}
    ]
  }
}
```

### Deep Nesting

HEDL supports arbitrary nesting depth (up to limit):

```hedl
%VERSION: 1.0
%STRUCT: Department: [id, name]
%STRUCT: Team: [id, name]
%STRUCT: Member: [id, name]
---
organization:
  name: TechCorp
  departments: @Department
    | d1, Engineering
    | d2, Sales
```

Note: Deep nesting with nested matrix lists requires careful structure planning.

## Value Types

### Scalar Types

HEDL infers types from syntax:

| Type | Syntax | Example |
|------|--------|---------|
| **String** | `"..."` | `"Hello"` |
| **Number** | Digits | `42`, `3.14` |
| **Boolean** | Keywords | `true`, `false` |
| **Null** | Tilde | `~` |
| **Tensor** | Brackets | `[1, 2, 3]`, `[[1, 2], [3, 4]]` |
| **Expression** | `$()` | `$(variable)`, `$(concat(a, b))` |

### Objects

Nested key-value pairs:

```hedl
%VERSION: 1.0
---
config:
  host: localhost
  port: 8080
  ssl: true
```

### Special Values

**Ditto operator (`^`)** - Repeat previous value:
```hedl
%VERSION: 1.0
%STRUCT: Task: [id, status, priority]
---
tasks: @Task
  | t1, pending, high
  | t2, pending, high
  | t3, ^, ^             # Repeat status and priority from previous row
```

**Null (`~`)** - Absent value:
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, ~                    # No email
```

## Indentation Rules

HEDL uses **2-space indentation** for nesting:

**Correct:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
  | u2, Bob
```

**Incorrect:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
   | u1, Alice     # 3 spaces - wrong
 | u2, Bob         # 1 space - wrong
```

**Why 2 spaces?**
- Consistent with Python, YAML conventions
- Readable without being wasteful
- Easy to enforce programmatically

## Data Model Constraints

### ID Requirements

In matrix lists, first column is the ID:

**Rules:**
1. IDs must be unique within their entity
2. IDs can't be null or ditto
3. IDs are used for references

### Column Count

All rows must have the same number of columns:

**Valid:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User
  | u1, Alice, 30
  | u2, Bob, 25
```

**Invalid:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User
  | u1, Alice, 30
  | u2, Bob        # Missing age
```

### Nesting Depth

Default maximum: 50 levels (configurable).

Can be configured via environment variable.

## JSON Mapping

### Matrix List → Array

**HEDL:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
  | u2, Bob
```

**JSON:**
```json
{
  "users": [
    {"id": "u1", "name": "Alice"},
    {"id": "u2", "name": "Bob"}
  ]
}
```

### Entity → Object

**HEDL:**
```hedl
%VERSION: 1.0
---
config:
  host: localhost
  port: 8080
```

**JSON:**
```json
{
  "config": {
    "host": "localhost",
    "port": 8080
  }
}
```

## Design Trade-offs

### Advantages

1. **Token efficiency** - Matrix lists save 40-60% tokens
2. **Schema clarity** - Structure is explicit
3. **Validation** - Type and structure checking
4. **Readability** - Aligned tabular data

### Limitations

1. **Deeply nested trees** - JSON might be more concise
2. **Irregular structures** - Works best with consistent data
3. **Large text blocks** - String length limits apply

---

**Related Concepts:**
- [Type System](type-system.md) - How types work
- [References](references.md) - Linking entities
- [Canonicalization](canonicalization.md) - Standard formatting
