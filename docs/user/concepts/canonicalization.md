# Concept: Canonicalization

Understanding HEDL's deterministic formatting.

## What is Canonicalization?

**Canonicalization** (or **canonical form**) is the process of converting data to a standard, deterministic representation.

**Key property:** Given the same input data, canonicalization always produces **exactly the same output**, byte-for-byte.

## Why Canonical Form Matters

### 1. Version Control

**Without canonicalization:**
```
# Developer A formats like this
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
  | u2, Bob

# Developer B formats like this
%VERSION: 1.0
%STRUCT: User: [id,name]
---
users: @User
 | u1, Alice
 | u2, Bob
```

Git sees these as different → meaningless diffs.

**With canonicalization:**
Both produce identical output → clean diffs.

### 2. Cryptographic Hashing

Canonical form enables reliable hashing:

```bash
# Hash canonical form
hedl format data.hedl | sha256sum
# Always produces same hash for same data
```

**Use cases:**
- Content-addressable storage
- Data integrity verification
- Distributed consensus

### 3. Testing

Comparing outputs reliably:

```bash
# Test conversion roundtrip
hedl to-json data.hedl | hedl from-json - | hedl format - > roundtrip.hedl
diff <(hedl format data.hedl) roundtrip.hedl
```

### 4. Deduplication

Identify identical data:

```bash
# Find duplicate documents
for f in *.hedl; do
  hedl format "$f" | md5sum
done | sort | uniq -d
```

## Canonicalization Rules

HEDL's canonical form follows these rules:

### 1. Consistent Indentation

**Rule:** Exactly 2 spaces per level.

**Before:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
   | u1, Alice    # 3 spaces
```

**After:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice     # 2 spaces
```

### 2. Whitespace Normalization

**Rule:** Single space after colons and commas in schemas.

**Before:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1,  Alice,   alice@example.com
```

**After:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
```

### 3. Line Endings

**Rule:** LF (`\n`) line endings (Unix-style).

**Conversion:**
```bash
# CRLF → LF
hedl format windows_file.hedl -o unix_file.hedl
```

### 4. Field Ordering

**Rule:** Fields ordered as declared in columns.

This is already enforced by matrix list syntax.

### 5. No Trailing Whitespace

**Rule:** No spaces at end of lines.

**Before:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

**After:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

## Using Canonical Form

### Format Command

```bash
# Format to stdout
hedl format input.hedl

# Format to file
hedl format input.hedl -o output.hedl

# Format in-place
hedl format input.hedl -o input.hedl
```

### Pre-Commit Hook

Ensure all commits use canonical form:

```bash
#!/bin/bash
# .git/hooks/pre-commit

for file in $(git diff --cached --name-only | grep '\.hedl$'); do
  hedl format "$file" -o "$file"
  git add "$file"
done
```

### CI Check

Verify canonical form in CI:

```yaml
# .github/workflows/check.yml
- name: Check canonical form
  run: |
    for file in **/*.hedl; do
      if ! diff <(cat "$file") <(hedl format "$file"); then
        echo "Not canonical: $file"
        exit 1
      fi
    done
```

## Comparison with Other Formats

### JSON

**JSON has multiple canonical forms:**
- [RFC 8785](https://tools.ietf.org/html/rfc8785) (JCS)
- Sorted keys
- No whitespace variants

**HEDL has one canonical form.**

### YAML

**YAML canonicalization is complex:**
- Multiple ways to represent same data
- Anchor/alias handling
- Implicit vs explicit types

**HEDL is deterministic by design.**

### XML

**XML c14n (canonicalization):**
- Complex spec (XML C14N)
- Multiple versions (C14N, C14N11)
- Namespace handling

**HEDL is simpler.**

## Canonical Form Properties

### Idempotent

Formatting is idempotent:

```bash
hedl format file.hedl | hedl format - | hedl format -
# All produce identical output
```

### Lossless

Canonicalization preserves data:

```bash
# Data is unchanged
diff <(hedl to-json original.hedl) <(hedl format original.hedl | hedl to-json -)
# No diff
```

### Deterministic

Same input always produces same output:

```bash
# Hash is stable
sha256sum <(hedl format data.hedl)
# Always same hash
```

## Best Practices

### 1. Format Before Commit

```bash
git add data.hedl
hedl format data.hedl -o data.hedl
git add data.hedl
git commit
```

### 2. Use Format in CI/CD

Ensure all code is canonical:

```bash
hedl batch-format "**/*.hedl" --check
```

### 3. Format After Conversion

```bash
hedl from-json data.json | hedl format - -o data.hedl
```

### 4. Format Before Hashing

```bash
hedl format data.hedl | sha256sum > data.hedl.sha256
```

---

**Related:**
- [Data Model](data-model.md)
- [CLI Reference](../reference/cli-commands.md#format)
