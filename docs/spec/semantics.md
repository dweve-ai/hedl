# HEDL Semantics Specification

This document provides a formal specification of HEDL's semantic rules, covering type system, type inference, coercion, and reference resolution.

## Table of Contents

1. [Type System](#type-system)
2. [Type Inference](#type-inference)
3. [Type Coercion](#type-coercion)
4. [Reference Resolution](#reference-resolution)
5. [NEST Hierarchy](#nest-hierarchy)
6. [Security Limits](#security-limits)

## 1. Type System

### 1.1 Core Types

HEDL supports the following value types (defined in `Value` enum):

| Type |Syntax |Runtime Representation |Memory Size |
|------|--------|----------------------|-------------|
| Null | `~` | `Value::Null` |Inline (1 byte discriminant) |
| Bool | `true`, `false` | `Value::Bool(bool)` |Inline (2 bytes) |
| Int | `-42`, `100` | `Value::Int(i64)` |Inline (8 bytes) |
| Float | `3.5`, `-1.0` | `Value::Float(f64)` |Inline (8 bytes) |
| String | `"hello"`, `"""block"""` | `Value::String(Box<str>)` |Boxed (16 bytes pointer) |
| Tensor | `[1 2; 3 4]` | `Value::Tensor(Box<Tensor>)` |Boxed (16 bytes pointer) |
| Reference | `@User:id`, `@id` | `Value::Reference(Reference)` |Inline (32 bytes) |
| Expression | `$(now())` | `Value::Expression(Box<Expression>)` |Boxed (16 bytes pointer) |

**Memory Layout Optimization**: Total `Value` enum size is 40 bytes (discriminant + largest variant). Large variants are boxed to minimize memory overhead.

### 1.2 Expected Types

The `ExpectedType` enum represents type expectations for bidirectional type checking:

```rust
pub enum ExpectedType {
    Any,
    Null,
    Bool,
    Int,
    Float,
    Numeric,              // Int |Float
    String,
    Tensor {
        shape: Option<Vec<usize>>,
        dtype: Option<TensorDtype>,
    },
    Reference {
        target_type: Option<String>,
    },
    Expression,
    Union(Vec<ExpectedType>),
}
```

#### Type Matching Rules

Type matching is performed by `ExpectedType::matches(&Value)`:

1. **Any**: Matches all values
2. **Null**: Matches only `Value::Null`
3. **Bool**: Matches only `Value::Bool(_)`
4. **Int**: Matches only `Value::Int(_)`
5. **Float**: Matches only `Value::Float(_)` (NOT `Int`)
6. **Numeric**: Matches `Value::Int(_)` OR `Value::Float(_)`
7. **String**: Matches only `Value::String(_)`
8. **Tensor**: Matches `Value::Tensor(_)` with optional shape/dtype constraints
   - If `shape` is `Some(expected_shape)`, actual shape must match exactly
   - If `dtype` is `Some(expected_dtype)`, actual dtype must match
9. **Reference**: Matches `Value::Reference(_)` with optional type constraint
   - If `target_type` is `Some(type_name)`, reference must be qualified with that type
   - If `target_type` is `None`, any reference matches
   - Unqualified references (`@id`) do NOT match type-specific expectations
10. **Expression**: Matches only `Value::Expression(_)`
11. **Union**: Matches if ANY type in union matches

**Example**:
```rust
// Tensor with shape constraint
let tensor_type = ExpectedType::Tensor {
    shape: Some(vec![2, 3]),
    dtype: Some(TensorDtype::Float),
};
// Matches only 2x3 float tensors

// Reference with type constraint
let user_ref = ExpectedType::Reference {
    target_type: Some("User".to_string()),
};
// Matches @User:id but NOT @id or @Post:id
```

## 2. Type Inference

HEDL uses bidirectional type inference:

1. **Bottom-up (synthesis)**: Infer types from values
2. **Top-down (checking)**: Propagate expected types from context (schemas)

### 2.1 Value to Expected Type Conversion

The function `value_to_expected_type(&Value) -> ExpectedType` infers expected types:

| Value |Inferred Expected Type |
|-------|----------------------|
| `Value::Null` | `ExpectedType::Null` |
| `Value::Bool(_)` | `ExpectedType::Bool` |
| `Value::Int(_)` | `ExpectedType::Int` |
| `Value::Float(_)` | `ExpectedType::Float` |
| `Value::String(_)` | `ExpectedType::String` |
| `Value::Tensor(t)` | `ExpectedType::Tensor { shape: Some(t.shape()), dtype: Some(inferred) }` |
| `Value::Reference(r)` | `ExpectedType::Reference { target_type: r.type_name.clone() }` |
| `Value::Expression(_)` | `ExpectedType::Expression` |

### 2.2 Type Description

For error reporting, `ExpectedType::describe()` and `describe_value_type(&Value)` provide human-readable type names:

```rust
ExpectedType::Numeric.describe()         // "Numeric (Int or Float)"
ExpectedType::Union([Int, String])       // "Union(Int |String)"
ExpectedType::Tensor { ... }             // "Tensor (shape: [2, 3]) (dtype: Float)"

describe_value_type(&Value::Int(42))     // "Int"
describe_value_type(&Value::Reference(r)) // "Reference(User)" or "Reference"
```

## 3. Type Coercion

HEDL supports four levels of type coercion strictness.

### 3.1 Coercion Levels

Defined in `CoercionLevel` enum (replaces legacy `CoercionMode`):

```rust
pub enum CoercionLevel {
    None,        // No coercion allowed
    Strict,      // Safe coercions only (default)
    Standard,    // Safe coercions + string parsing
    Permissive,  // All coercions including lossy
}
```

### 3.2 Coercion Rules Matrix

| From Type |To Type |None |Strict |Standard |Permissive |Notes |
|-----------|---------|------|--------|----------|------------|-------|
| Int |Int | ✓ | ✓ | ✓ | ✓ |Exact match |
| Int |Float | ✗ | ✓ | ✓ | ✓ |Safe widening |
| Int |Numeric | ✗ | ✓ | ✓ | ✓ |Already numeric |
| Int |String | ✗ | ✗ | ✓ | ✓ | `"42"` |
| Float |Float | ✓ | ✓ | ✓ | ✓ |Exact match |
| Float |Int | ✗ | ✗ | ✗ | ✓ |Lossy (truncates) |
| Float |Numeric | ✗ | ✓ | ✓ | ✓ |Already numeric |
| Float |String | ✗ | ✗ | ✓ | ✓ | `"3.5"` |
| String |Int | ✗ | ✗ | ✓ | ✓ |Parses `"42"` |
| String |Float | ✗ | ✗ | ✓ | ✓ |Parses `"3.5"` |
| String |Bool | ✗ | ✗ | ✓ | ✓ |Parses `"true"`/`"false"` |
| String |Numeric | ✗ | ✗ | ✓ | ✓ |Tries Int, then Float |
| Bool |String | ✗ | ✗ | ✓ | ✓ | `"true"` or `"false"` |
| Null |Any | ✗ | ✗ | ✗ | ✓ |To default value |
| Null |Int | ✗ | ✗ | ✗ | ✓ | `0` |
| Null |Float | ✗ | ✗ | ✗ | ✓ | `0.0` |
| Null |Bool | ✗ | ✗ | ✗ | ✓ | `false` |
| Null |String | ✗ | ✗ | ✗ | ✓ | `""` |
| Reference |String | ✗ | ✗ | ✓ | ✓ | `"@User:id"` |

**Legend**: ✓ = allowed, ✗ = rejected

### 3.3 Coercion Configuration

The `CoercionConfig` struct provides fine-grained control:

```rust
pub struct CoercionConfig {
    level: CoercionLevel,
    allow_string_to_number: bool,
    allow_lossy_float_to_int: bool,
    bool_true_values: Vec<String>,
    bool_false_values: Vec<String>,
    null_as_default: bool,
}
```

#### Standard Configurations

```rust
// None: No coercion at all
CoercionConfig::none()
// level = None
// All flags = false

// Strict: Only safe coercions (Int → Float)
CoercionConfig::strict()
// level = Strict
// allow_string_to_number = true (but not used in Strict mode)
// allow_lossy_float_to_int = false

// Standard: Safe coercions + string parsing
CoercionConfig::standard()
// level = Standard
// allow_string_to_number = true
// bool_true_values = ["true"]
// bool_false_values = ["false"]

// Permissive: All coercions including lossy
CoercionConfig::permissive()
// level = Permissive
// allow_string_to_number = true
// allow_lossy_float_to_int = true
// null_as_default = true
// bool_true_values = ["true", "yes", "1"]
// bool_false_values = ["false", "no", "0"]
```

### 3.4 Coercion Result

Coercion returns a `CoercionResult`:

```rust
pub enum CoercionResult {
    Matched(Value),          // Value already matches (no coercion needed)
    Coerced(Value),          // Value was coerced
    Failed {
        value: Value,
        expected: ExpectedType,
        reason: String,
    },
}
```

**Semantic Invariants**:

1. `Matched` → value satisfies `expected.matches(&value)` BEFORE coercion
2. `Coerced` → value did NOT match but was successfully converted
3. `Failed` → conversion not possible in given coercion mode
4. Original value always preserved in `Failed` for error recovery

### 3.5 Special Coercion Behaviors

#### String Parsing

String coercion uses standard Rust parsing (`str::parse`) with trimming:

```rust
// Whitespace is trimmed before parsing
"  42  " → Int(42)       // OK
"  true  " → Bool(true)  // OK
"" → Int                 // FAIL (empty after trim)
"   " → Int              // FAIL (empty after trim)
```

#### Boolean Coercion

Boolean string coercion is case-sensitive and configurable:

```rust
// Standard mode (strict)
"true" → Bool(true)   // OK
"false" → Bool(false) // OK
"True" → FAIL         // Case-sensitive
"yes" → FAIL          // Not in standard set

// Permissive mode (extended)
"true" → Bool(true)
"yes" → Bool(true)
"1" → Bool(true)
"false" → Bool(false)
"no" → Bool(false)
"0" → Bool(false)
```

#### Numeric Coercion

String to Numeric tries Int first, then Float:

```rust
"42" → Numeric         // Int(42) (prefers Int)
"3.5" → Numeric        // Float(3.5) (can't be Int)
"inf" → FAIL           // Non-finite floats rejected
"NaN" → FAIL           // Non-finite floats rejected
```

#### Null to Default (Permissive Only)

```rust
// Only in CoercionLevel::Permissive with null_as_default = true
~ → Int             // 0
~ → Float           // 0.0
~ → Bool            // false
~ → String          // ""
~ → Numeric         // 0 (prefers Int)
```

#### Union Coercion

Union types try each option in order:

1. Try exact match (no coercion) for each type
2. Try coercion for each type in order
3. Return first successful coercion
4. Fail if no type accepts value

```rust
Union([Int, String]) + Value::Float(42.0) + Permissive
  → Try Int match: FAIL
  → Try String match: FAIL
  → Try coerce to Int: SUCCESS → Int(42)
```

### 3.6 Non-Coercible Types

The following types do NOT support coercion (always return `Failed` if not exact match):

- `ExpectedType::Null` (except exact match)
- `ExpectedType::Tensor` (no coercion rules defined)
- `ExpectedType::Reference` (no coercion rules defined)
- `ExpectedType::Expression` (no coercion rules defined)

## 4. Reference Resolution

References (`@Type:id` or `@id`) are resolved using a two-phase process:

1. **Registration Phase**: Build `TypeRegistry` with ID → Type mappings
2. **Validation Phase**: Resolve all references against registry

### 4.1 Reference Types

```rust
pub struct Reference {
    type_name: Option<Box<str>>,  // Qualified type (e.g., "User")
    id: Box<str>,                 // ID being referenced
}
```

#### Reference Syntax

| Syntax |Type |Description |Example |
|--------|------|-------------|---------|
| `@Type:id` |Qualified |Reference with explicit type | `@User:u1` |
| `@id` |Unqualified |Reference without type | `@active` |

**Creating References**:
```rust
Reference::qualified("User", "123")  // @User:123
Reference::local("id")               // @id
Reference::unqualified("id")         // @id (alias for local)
```

### 4.2 Type Registry

The `TypeRegistry` maintains bidirectional indices for O(1) reference lookups:

```rust
pub struct TypeRegistry {
    by_type: BTreeMap<String, BTreeMap<String, usize>>,  // Type → (ID → line)
    by_id: HashMap<String, Vec<String>>,                 // ID → [Types]
    total_ids: usize,                                    // Total registered IDs
}
```

**Index Usage**:

1. **Forward Index** (`by_type`): Qualified reference lookups
   - `@User:id` → `by_type["User"].contains_key("id")` → O(log n)

2. **Inverted Index** (`by_id`): Unqualified reference lookups (P0 optimization)
   - `@id` → `by_id["id"]` → O(1) instead of O(m) scan
   - Returns list of all types containing `id`

**Example Registry**:
```rust
// After registering:
//   User.u1 at line 10
//   User.u2 at line 11
//   Post.p1 at line 20
//   Post.u1 at line 21

by_type = {
    "User": {"u1" → 10, "u2" → 11},
    "Post": {"p1" → 20, "u1" → 21},
}

by_id = {
    "u1" → ["User", "Post"],
    "u2" → ["User"],
    "p1" → ["Post"],
}
```

### 4.3 Reference Resolution Modes

```rust
pub enum ReferenceMode {
    Strict,   // Unresolved references → Error (default)
    Lenient,  // Unresolved references → Silently ignored
}
```

#### Mode Behavior

| Condition |Strict |Lenient |
|-----------|--------|---------|
| Unresolved reference |Error |Ignored |
| Ambiguous reference |Error |Error |
| Resolved reference |OK |OK |

**Note**: Ambiguous references ALWAYS cause errors, regardless of mode.

### 4.4 Qualified Reference Resolution

Qualified references (`@Type:id`) are resolved by checking the specific type registry:

```rust
@User:u1 → registries.by_type["User"].contains_key("u1")
```

**Resolution Rules**:

1. Look up type in `by_type`
2. Check if ID exists in that type's registry
3. If not found and mode is Strict → Error
4. If not found and mode is Lenient → Ignored

**Example**:
```hedl
STRUCT User: id name
User:@User
  u1 "Alice"
  u2 "Bob"

value:@User:u1  # OK (u1 exists in User)
value:@User:u3  # ERROR in Strict, IGNORED in Lenient
value:@Post:p1  # ERROR in Strict (Post not defined), IGNORED in Lenient
```

### 4.5 Unqualified Reference Resolution

Unqualified references (`@id`) use context-dependent resolution:

#### 4.5.1 Matrix Context (SPEC 10.2, 10.3)

Inside a matrix list, unqualified references search ONLY the current type:

```rust
// Inside User matrix
@id → registries.by_type["User"].contains_key("id")
```

**Example**:
```hedl
STRUCT User: id name manager
STRUCT Post: id title

User:@User
  u1 "Alice" ~
  u2 "Bob" @u1      # Searches ONLY User type
  u3 "Charlie" @p1  # ERROR: p1 not in User (even if exists in Post)

Post:@Post
  p1 "Hello"
```

**Rationale**: Matrix context provides implicit type scope. This prevents accidental cross-type references and improves comprehension.

#### 4.5.2 Key-Value Context (SPEC 10.3.1)

Outside matrix lists (in key-value pairs), unqualified references search ALL types:

```rust
// In key-value context
@id → registries.by_id["id"]  // O(1) lookup via inverted index
```

**Resolution Rules**:

1. Look up ID in inverted index (`by_id`)
2. Check number of types containing ID:
   - 0 types → Unresolved (Error in Strict, Ignored in Lenient)
   - 1 type → Unambiguous (OK, resolves to that type)
   - 2+ types → Ambiguous (Error in ALL modes)

**Example**:
```hedl
STRUCT User: id name
STRUCT Post: id title

User:@User
  u1 "Alice"
  u2 "Bob"

Post:@Post
  p1 "Hello"
  u1 "World"    # Same ID in different type

# Key-value references
ref1:@u2  # OK (only in User)
ref2:@p1  # OK (only in Post)
ref3:@u1  # ERROR (ambiguous: exists in both User and Post)
ref4:@u3  # ERROR in Strict (not found), IGNORED in Lenient
```

### 4.6 Ambiguous Reference Handling

An ambiguous reference exists when an unqualified reference matches multiple types.

**Detection**:
```rust
let types = registries.by_id["id"];
if types.len() > 1 {
    return Error("Ambiguous reference @id matches types: [User, Post]")
}
```

**Always Error**: Ambiguous references cause errors in BOTH Strict and Lenient modes because they represent true semantic ambiguity, not just missing data.

**Resolution**: Use qualified references:
```hedl
# Instead of ambiguous:
value:@u1  # ERROR

# Use qualified:
value:@User:u1  # OK
```

### 4.7 Reference Resolution Algorithm

The complete resolution algorithm:

```rust
fn resolve_reference(
    reference: &Reference,
    registries: &TypeRegistry,
    mode: ReferenceMode,
    current_type: Option<&str>,  // Current matrix type, if any
) -> Result<bool, HedlError> {
    match reference.type_name {
        // Qualified reference:@Type:id
        Some(type_name) => {
            Ok(registries.contains_in_type(&type_name, &reference.id))
        }

        // Unqualified reference:@id
        None => {
            match current_type {
                // Matrix context: search only current type
                Some(type_name) => {
                    Ok(registries.contains_in_type(type_name, &reference.id))
                }

                // Key-value context: search all types, detect ambiguity
                None => {
                    let matching_types = registries.lookup_unqualified(&reference.id);
                    match matching_types.len() {
                        0 => Ok(false),  // Not found
                        1 => Ok(true),   // Unambiguous
                        _ => Err(Error::Ambiguous {
                            id: reference.id,
                            types: matching_types,
                        })
                    }
                }
            }
        }
    }
}
```

### 4.8 Collision Detection

ID collisions within the same type are detected during registration:

```rust
registries.register("User", "u1", line_10)?;  // OK
registries.register("User", "u1", line_20)?;  // ERROR: duplicate ID
```

**Error Example**:
```
Error: duplicate ID 'u1' in type 'User', previously defined at line 10
  at line 20
```

**Cross-Type IDs**: Same ID in different types is allowed and resolved via qualification:

```hedl
STRUCT User: id
STRUCT Post: id

User:@User
  admin      # OK

Post:@Post
  admin      # OK (different type)

ref:@admin  # ERROR (ambiguous)
ref:@User:admin  # OK
```

## 5. NEST Hierarchy

NEST relationships define parent-child hierarchies between types.

### 5.1 NEST Semantics

```hedl
NEST User Team      # User contains Team children
NEST Team Member    # Team contains Member children
```

**Document Structure**:
```rust
pub struct Document {
    nests: BTreeMap<String, String>,  // Parent → Child
    // ...
}
```

**Accessor**:
```rust
doc.get_child_type("User")  // Some("Team")
doc.get_child_type("Team")  // Some("Member")
doc.get_child_type("Member")  // None (leaf type)
```

### 5.2 Node Children

```rust
pub struct Node {
    type_name: String,
    id: String,
    fields: SmallVec<[Value; 4]>,
    children: Option<Box<BTreeMap<String, Vec<Node>>>>,  // Lazy allocation
    child_count: u16,  // Optional hint
}
```

**Child Management**:
```rust
node.add_child("Team", child_node);  // Adds child to "Team" type list

node.children()  // Option<&BTreeMap<String, Vec<Node>>>
node.children_mut()  // Option<&mut BTreeMap<String, Vec<Node>>>

node.set_child_count(5);    // Hint for LLM comprehension
node.get_child_count()      // Some(5) or None
```

**Lazy Allocation**: The `children` field is `None` until first child is added. This saves ~24 bytes per node for leaf nodes (~70% of typical documents).

### 5.3 Hierarchy Depth Limits

NEST hierarchies are bounded by `max_nest_depth` security limit:

```rust
pub struct Limits {
    max_nest_depth: usize,  // Default: 100
    // ...
}
```

**Enforcement**:
```rust
fn collect_node_ids(items: &BTreeMap<String, Item>, depth: usize, limits: &Limits) -> Result<()> {
    if depth > limits.max_nest_depth {
        return Err(Error::Security(
            "NEST hierarchy depth {depth} exceeds maximum {max_nest_depth}"
        ));
    }
    // Process items...
    collect_node_ids(child_items, depth + 1, limits)?;
}
```

**Semantic Effect**: Deeply nested documents are rejected during parsing, preventing stack overflow and excessive recursion.

### 5.4 Child Count Hints

The `child_count` field provides metadata for LLM comprehension:

```hedl
STRUCT User: id name teams(5)  # Hint: 5 teams expected
```

**Implementation**:
```rust
impl Node {
    pub fn set_child_count(&mut self, count: usize) {
        self.child_count = count.min(u16::MAX as usize) as u16;  // Saturates at 65,535
    }

    pub fn get_child_count(&self) -> Option<usize> {
        if self.child_count > 0 {
            Some(self.child_count as usize)
        } else {
            None  // No hint provided
        }
    }
}
```

**Semantics**:

- Hint is purely informational (not enforced)
- `0` means no hint provided
- Values > 65,535 saturate at 65,535
- Used for documentation and LLM parsing hints

## 6. Security Limits

HEDL enforces resource limits to prevent denial-of-service attacks and memory exhaustion.

### 6.1 Limit Categories

```rust
pub struct Limits {
    // Size limits
    max_file_size: usize,           // Default: 1GB
    max_line_length: usize,         // Default: 1MB
    max_block_string_size: usize,   // Default: 10MB

    // Structural limits
    max_indent_depth: usize,        // Default: 50
    max_nest_depth: usize,          // Default: 100
    max_columns: usize,             // Default: 100

    // Quantity limits
    max_nodes: usize,               // Default: 10M
    max_aliases: usize,             // Default: 10K
    max_object_keys: usize,         // Default: 10K (per object)
    max_total_keys: usize,          // Default: 10M (all objects)
    max_total_ids: usize,           // Default: 10M (all types)

    // Time limit
    timeout: Option<Duration>,      // Default: 30 seconds
}
```

### 6.2 Default Limits

| Limit |Default |Rationale |
|-------|---------|-----------|
| `max_file_size` |1GB |Prevents memory exhaustion from huge files |
| `max_line_length` |1MB |Prevents single-line DoS attacks |
| `max_indent_depth` |50 |Prevents deeply nested structures |
| `max_nodes` |10M |Limits total node count across document |
| `max_aliases` |10K |Prevents alias table bloat |
| `max_columns` |100 |Limits schema width |
| `max_nest_depth` |100 |Prevents stack overflow in recursion |
| `max_block_string_size` |10MB |Limits triple-quote block sizes |
| `max_object_keys` |10K |Limits keys per object/map |
| `max_total_keys` |10M |Defense-in-depth: limits total keys |
| `max_total_ids` |10M |Prevents TypeRegistry memory exhaustion |
| `timeout` |30s |Prevents parser hangs |

**Unlimited Mode** (testing only):
```rust
Limits::unlimited()  // All limits = usize::MAX, timeout = None
```

### 6.3 Semantic Effects

#### 6.3.1 max_file_size

**Effect**: File size is checked before parsing begins.

```rust
if file_size > limits.max_file_size {
    return Error::Security("File size exceeds limit");
}
```

**Use Case**: Reject malicious files before loading into memory.

#### 6.3.2 max_line_length

**Effect**: Each line's length is checked during lexing.

```rust
if line.len() > limits.max_line_length {
    return Error::Security("Line exceeds maximum length");
}
```

**Use Case**: Prevent single extremely long lines from consuming memory.

#### 6.3.3 max_indent_depth

**Effect**: Indentation depth is tracked during parsing.

```rust
if indent_depth > limits.max_indent_depth {
    return Error::Security("Indent depth exceeds limit");
}
```

**Use Case**: Prevent deeply nested key-value structures.

#### 6.3.4 max_nest_depth

**Effect**: NEST hierarchy depth is checked during tree traversal.

```rust
if nest_depth > limits.max_nest_depth {
    return Error::Security("NEST hierarchy depth exceeds limit");
}
```

**Use Case**: Prevent stack overflow in recursive tree traversal.

**Checked Operations**:
- ID collection during reference resolution
- Reference validation
- Tree serialization

#### 6.3.5 max_columns

**Effect**: Schema column count is validated during STRUCT parsing.

```rust
if columns.len() > limits.max_columns {
    return Error::Security("Schema has too many columns");
}
```

**Use Case**: Prevent extremely wide schemas that consume memory per node.

#### 6.3.6 max_nodes

**Effect**: Total node count is tracked during parsing.

```rust
if total_nodes >= limits.max_nodes {
    return Error::Security("Total nodes exceeds limit");
}
```

**Use Case**: Prevent unbounded node allocation.

#### 6.3.7 max_aliases

**Effect**: Alias count is checked when adding new aliases.

```rust
if doc.aliases.len() >= limits.max_aliases {
    return Error::Security("Alias count exceeds limit");
}
```

**Use Case**: Prevent alias table from consuming excessive memory.

#### 6.3.8 max_object_keys

**Effect**: Key count per object is tracked during object parsing.

```rust
if object.len() >= limits.max_object_keys {
    return Error::Security("Object has too many keys");
}
```

**Use Case**: Prevent individual objects from becoming too large.

#### 6.3.9 max_total_keys

**Effect**: Total key count across ALL objects is tracked.

```rust
if total_keys >= limits.max_total_keys {
    return Error::Security("Total key count exceeds limit");
}
```

**Use Case**: Defense-in-depth against memory exhaustion from many small objects.

**Example Attack**:
```hedl
# Each object has 100 keys (under max_object_keys = 10K)
# But 100,000 such objects = 10M total keys
obj1: { k1: 1, k2: 2, ..., k100: 100 }
obj2: { k1: 1, k2: 2, ..., k100: 100 }
...
obj100000: { k1: 1, k2: 2, ..., k100: 100 }
# Rejected when total_keys reaches max_total_keys
```

#### 6.3.10 max_total_ids

**Effect**: Total ID registrations across ALL types are tracked.

```rust
if registries.total_ids >= limits.max_total_ids {
    return Error::Security("Total ID registrations exceeds limit");
}
```

**Use Case**: Prevent TypeRegistry memory exhaustion from many IDs.

**Memory Impact**: Each ID registration consumes memory in BOTH indices:
- Forward index: `BTreeMap` entry (~48 bytes)
- Inverted index: `HashMap` entry (~48 bytes)
- Total: ~96 bytes per ID registration

**Example Attack**:
```hedl
STRUCT Type1: id
STRUCT Type2: id
...
STRUCT Type1000: id

Type1:@Type1
  id1 id2 id3 ... id10000  # 10K IDs

Type2:@Type2
  id1 id2 id3 ... id10000  # 10K IDs

# Total: 1000 types × 10K IDs = 10M IDs
# Memory: 10M × 96 bytes ≈ 960 MB just for indices
```

#### 6.3.11 timeout

**Effect**: Parsing duration is checked periodically (every 10,000 iterations).

```rust
pub struct TimeoutContext {
    start: Instant,
    timeout: Option<Duration>,
}

impl TimeoutContext {
    pub fn check_timeout(&self, line_num: usize) -> Result<()> {
        if let Some(timeout) = self.timeout {
            if self.start.elapsed() > timeout {
                return Error::Security("Parsing timeout exceeded");
            }
        }
        Ok(())
    }
}
```

**Check Interval**: Every 10,000 iterations (balances overhead vs responsiveness)
- Overhead: <0.01% at typical parsing speeds
- Detection latency: ~1ms worst-case

**Use Case**: Prevent malicious documents from hanging parser indefinitely.

### 6.4 Limit Interactions

Some limits provide defense-in-depth by protecting against different attack vectors:

#### Keys Limits

- `max_object_keys`: Prevents single large object
- `max_total_keys`: Prevents many small objects
- Together: Comprehensive protection against key-based DoS

#### ID Limits

- Per-type ID uniqueness: Prevents collisions within type
- `max_total_ids`: Prevents total registry exhaustion
- Together: Protects forward AND inverted indices

#### Depth Limits

- `max_indent_depth`: Protects against deeply nested key-value structures
- `max_nest_depth`: Protects against deeply nested NEST hierarchies
- Together: Prevents stack overflow from any source

### 6.5 Error Reporting

All limit violations produce `HedlError::Security` with detailed messages:

```rust
Error::Security {
    message: "NEST hierarchy depth 101 exceeds maximum allowed depth 100",
    line: 456,
}

Error::Security {
    message: "total ID registrations 10000000 exceeds limit 10000000",
    line: 789,
}

Error::Security {
    message: "parsing timeout exceeded: 30100ms > 30000ms",
    line: 1234,
}
```

---

## Appendix A: Type System Summary

### Value Types (8 total)

| Type |Discriminant |Size |Boxed |Coercible |
|------|--------------|------|-------|-----------|
| Null |1 byte |1 byte |No |No (except in Permissive) |
| Bool |1 byte |2 bytes |No |From String (Standard+) |
| Int |1 byte |9 bytes |No |To Float (Strict+), From String (Standard+) |
| Float |1 byte |9 bytes |No |From Int (Strict+), From String (Standard+), To Int (Permissive) |
| String |1 byte |17 bytes |Yes |From most types (Standard+) |
| Tensor |1 byte |17 bytes |Yes |No |
| Reference |1 byte |33 bytes |No |To String (Standard+) |
| Expression |1 byte |17 bytes |Yes |No |

### Expected Types (11 variants)

1. Any (matches all)
2. Null (exact match only)
3. Bool
4. Int
5. Float
6. Numeric (Int |Float)
7. String
8. Tensor (with optional shape/dtype constraints)
9. Reference (with optional type constraint)
10. Expression
11. Union (matches any member)

## Appendix B: Coercion Quick Reference

```
Coercion Levels:
  None      → No coercion at all
  Strict    → Int → Float only
  Standard  → Strict + String parsing
  Permissive → Standard + Lossy + Null defaults

Safe Coercions (Strict):
  Int → Float
  Int → Numeric
  Float → Numeric

String Parsing (Standard):
  String → Int     (via parse::<i64>)
  String → Float   (via parse::<f64>, rejects non-finite)
  String → Bool    (case-sensitive "true"/"false")
  String → Numeric (tries Int, then Float)
  Any → String     (via to_string)

Lossy Coercions (Permissive):
  Float → Int      (truncates via .trunc())
  Null → Int       (0)
  Null → Float     (0.0)
  Null → Bool      (false)
  Null → String    ("")
  Null → Numeric   (0)
```

## Appendix C: Reference Resolution Decision Tree

```
Reference:@Type:id or @id
 |
  +-- Has type_name?
 |     |
 |     +-- YES: Qualified Reference (@Type:id)
 |     |     |
 |     |     +-- Look up in by_type[Type][id]
 |     |     |
 |     |     +-- Found?
 |     |           |
 |     |           +-- YES: RESOLVED
 |     |           +-- NO:
 |     |                 |
 |     |                 +-- Strict mode? → ERROR
 |     |                 +-- Lenient mode? → IGNORED
 |     |
 |     +-- NO: Unqualified Reference (@id)
 |           |
 |           +-- In matrix context?
 |                 |
 |                 +-- YES: Search current type only
 |                 |     |
 |                 |     +-- Look up in by_type[CurrentType][id]
 |                 |     +-- Found? → RESOLVED : (Strict → ERROR, Lenient → IGNORED)
 |                 |
 |                 +-- NO: Key-value context
 |                       |
 |                       +-- Look up in by_id[id] (inverted index)
 |                       |
 |                       +-- Number of matching types?
 |                             |
 |                             +-- 0: Strict → ERROR, Lenient → IGNORED
 |                             +-- 1: RESOLVED (unambiguous)
 |                             +-- 2+: ERROR (ambiguous, always fails)
```

## Version History

- **Version 1.3** (2025-02): Current production version with required headers and 1-space indentation
- **Version 1.2** (2025-01): Legacy version with compact syntax and metadata directives
  - Compact directive syntax: `%V:`, `%S:`, `%A:`, `%N:`, `%C:`
  - `%NULL` directive for configurable null representation
  - `%QUOTE` directive for configurable quote character
  - `%COUNT` / `%C` directive for statistical metadata (total counts, distributions)
  - Struct count hints: `%S:Type(N):[cols]`
  - List literals: `(elem1, elem2, ...)` for ordered sequences
  - List literals: `(elem1, elem2, ...)` for ordered sequences
  - Header fields: `null_char`, `quote_char`, `counts` in Header struct

- **Version 1.0** (2025-01): Initial HEDL specification
  - Core types, coercion, references, NEST, and security limits
