# Concept: Type System

Understanding HEDL's type inference and validation.

## Overview

HEDL uses **automatic type inference** to determine value types from syntax, combined with **optional type annotations** for validation and documentation.

## Type Inference

### How It Works

HEDL examines the syntax to infer types:

```hedl
%VERSION: 1.0
---
data:
  name: Alice         # String
  age: 30             # Number (digits)
  active: true        # Boolean (keyword)
  notes: ~            # Null (tilde)
```

### Inference Rules

| Syntax | Inferred Type | Example |
|--------|---------------|---------|
| `"..."` | String | `"hello"` |
| Digits | Number | `42`, `3.14`, `-5` |
| `true` / `false` | Boolean | `true` |
| `~` | Null | `~` |
| `@Type:id` | Reference | `@User:u1` |
| `[...]` | List | `[1,2,3]` |
| `[...]` (Num) | Tensor | `[1,2,3]`, `[[1,2],[3,4]]` |
| `$()` | Expression | `$(variable)`, `$(concat(a, b))` |
| Nested | Object | Indented content |

### Numbers

HEDL supports integers and floats:

```hedl
%VERSION: 1.0
---
integers: 3
  42
  -17
  0

floats: 3
  3.14
  -0.5
  1.23e-4
```

**Limits:**
- Integers: -2^63 to 2^63-1
- Floats: IEEE 754 double precision

## Type Annotations

### Syntax

Type annotations use `%STRUCT:` declarations and `@TypeName` references:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, bob@example.com
```

**Purpose:**
1. **Documentation** - Clarify intent
2. **Validation** - Ensure consistency
3. **References** - Enable `@User:id` syntax
4. **Metadata** - Preserved in JSON with `--metadata`

### When to Use

**Use type annotations when:**
- Data represents a domain entity (User, Product, Order)
- You want to use references (`@Type:id`)
- Converting from typed sources (database schemas)
- Building APIs with typed responses

**Skip when:**
- Data is generic or unstructured
- No references needed
- Simple configuration files

## Type Validation

### Column Type Consistency

Within a column, types should be consistent:

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
  | u2, Bob, twenty    # Type mismatch
```

### Null Handling

Use `~` for missing values:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, ~                     # No email
  | u3, Carol, carol@example.com
```

**Validation:**
- Nulls are allowed by default
- Schema can specify required fields (future feature)

## Type Coercion

### String Coercion

All values can be represented as strings:

```hedl
%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
  | d1, "42"       # String "42"
  | d2, 42         # Number 42
  | d3, "true"     # String "true"
```

### Number Coercion

Strings that look like numbers aren't auto-converted:

```hedl
%VERSION: 1.0
%STRUCT: Age: [id, value]
---
ages: @Age
  | a1, 30         # Number
  | a2, "30"       # String (quotes make it explicit)
```

## Reference Types

### Reference Syntax

References use `@TypeName:id`:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author, title]
---
users: @User
  | u1, Alice
  | u2, Bob

posts: @Post
  | p1, @User:u1, My Post
```

### Type Checking

References are validated:

**Valid:**
```hedl
%VERSION: 1.0
%STRUCT: Post: [id, author]
---
posts: @Post
  | p1, @User:u1     # u1 exists in users
```

**Invalid:**
```hedl
%VERSION: 1.0
%STRUCT: Post: [id, author]
---
posts: @Post
  | p1, @User:u99    # u99 doesn't exist
```

## Type Metadata

### Preserving Types in JSON

Use `--metadata` to include type information:

```bash
hedl to-json data.hedl --metadata -o output.json
```

**Output:**
```json
{
  "_hedl_version": "1.0",
  "_hedl_types": {
    "users": "User"
  },
  "users": [...]
}
```

---

**Related:**
- [Data Model](data-model.md)
- [References](references.md)
