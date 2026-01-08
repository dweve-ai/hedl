# How-To: Handle Errors

Practical guide for understanding, diagnosing, and fixing HEDL errors.

## Table of Contents

1. [Understanding Error Messages](#understanding-error-messages)
2. [Syntax Errors](#syntax-errors)
3. [Validation Errors](#validation-errors)
4. [Conversion Errors](#conversion-errors)
5. [Reference Errors](#reference-errors)
6. [Performance Errors](#performance-errors)
7. [Recovery Strategies](#recovery-strategies)

---

## Understanding Error Messages

HEDL provides detailed error messages with:
- **Location**: File and line number
- **Context**: Surrounding code
- **Explanation**: What went wrong
- **Suggestion**: How to fix it

**Example error:**
```
✗ users.hedl is invalid
Error on line 4, column 12:
  Expected closing quote
  Row: u2 "Bob Smith
           ^
  Suggestion: Add closing quote character (")
```

---

## Syntax Errors

### Missing Closing Quote

**Error:**
```
Error: Unclosed string literal on line 3
```

**Bad:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice  # Missing closing quote
```

**Good:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

### Invalid Indentation

**Error:**
```
Error: Invalid indentation on line 4
Expected: 2 spaces
Found: 3 spaces
```

**Bad:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
   | u1, Alice  # Wrong indentation
```

**Good:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice  # Correct: 2 spaces + |
```

### Missing Version Header

**Error:**
```
Error: Missing version header
First line must be "%VERSION: 1.0"
```

**Bad:**
```hedl
users: @User
  | u1, Alice
```

**Good:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
```

---

## Validation Errors

### Column Count Mismatch

**Error:**
```
Error on line 4: Expected 3 values but found 2
```

**Bad:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob  # Missing email
```

**Fix:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | u1, Alice, alice@example.com
  | u2, Bob, bob@example.com
```

### Type Mismatch

**Error:**
```
Error: Type mismatch on line 4, column age
Expected: Number
Found: String
```

**Bad:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User
  | u1, Alice, thirty  # Should be number
```

**Fix:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, age]
---
users: @User
  | u1, Alice, 30
```

### Duplicate IDs

**Error:**
```
Error: Duplicate ID 'u1' on line 5
Previous occurrence: line 3
```

**Bad:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
  | u1, Bob  # Duplicate ID
```

**Fix:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | u1, Alice
  | u2, Bob
```

---

## Conversion Errors

### JSON Parse Error

**Error:**
```
Error: Invalid JSON at line 5: unexpected token '}'
```

**Diagnosis:**
```bash
# Validate JSON first
jq empty input.json
```

**Fix JSON, then convert:**
```bash
jq . input.json > fixed.json
hedl from-json fixed.json -o output.hedl
```

### CSV Encoding Error

**Error:**
```
Error: Invalid UTF-8 sequence
```

# Fix:
```bash
# Convert to UTF-8
iconv -f ISO-8859-1 -t UTF-8 input.csv > input_utf8.csv
hedl from-csv input_utf8.csv --type-name Record -o output.hedl
```

### Parquet Schema Mismatch

**Error:**
```
Error: Cannot convert to Parquet: incompatible types
```

**Fix:** Ensure consistent types in columns:
```bash
# Inspect schema first
hedl inspect data.hedl

# Fix type inconsistencies
# Then convert
hedl to-parquet data.hedl -o output.parquet
```

---

## Reference Errors

### Unresolved Reference

**Error:**
```
Error: Unresolved reference @User:u3 on line 7
Available: u1, u2
```

**Bad:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | u1, Alice
  | u2, Bob

posts: @Post
  | p1, @User:u3  # u3 doesn't exist
```

**Fix:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | u1, Alice
  | u2, Bob

posts: @Post
  | p1, @User:u1  # Use existing ID
```

### Circular Reference

**Error:**
```
Error: Circular reference detected
```

**Bad:**
```hedl
%VERSION: 1.0
%STRUCT: Node: [id, parent]
---
nodes: @Node
  | n1, @Node:n2
  | n2, @Node:n1  # Circular
```

**Fix:** Restructure to avoid cycles or use nullable references.

---

## Performance Errors

### Out of Memory

**Error:**
```
Error: Out of memory while processing large file
```

**Fix:** Use batch processing for multiple files, or split large files.
(Streaming support for large files is currently in development).

---

## Recovery Strategies

### Automated Error Fixing

**Script to fix common issues:**
```bash
#!/bin/bash
# fix_hedl.sh

FILE="$1"

# Backup original
cp "$FILE" "${FILE}.backup"

# Fix indentation (convert to 2 spaces)
sed -i 's/^    /  /g' "$FILE"  # 4 → 2
sed -i 's/^\t/  /g' "$FILE"    # tab → 2 spaces

# Validate
if hedl validate "$FILE"; then
  echo "✓ Fixed: $FILE"
  rm "${FILE}.backup"
else
  echo "✗ Could not auto-fix: $FILE"
  mv "${FILE}.backup" "$FILE"
fi
```

### Error Logging

**Comprehensive error tracking:**
```bash
#!/bin/bash

LOG_FILE="errors_$(date +%Y%m%d_%H%M%S).log"

{
  echo "Error Log - $(date)"
  echo "===================="
  echo ""

  for file in data/*.hedl; do
    echo "Checking: $file"
    if ! hedl validate "$file" 2>&1; then
      echo "FAILED: $file"
      echo "---"
    fi
    echo ""
  done
} | tee "$LOG_FILE"
```

---

## Quick Reference

```bash
# Validate with detailed errors
hedl validate file.hedl

# Check JSON before converting
jq empty file.json && hedl from-json file.json -o output.hedl

# Fix encoding
iconv -f ISO-8859-1 -t UTF-8 input.csv > utf8.csv
```

---

**Related:**
- [Validate Documents](validate-documents.md)
- [Troubleshooting Guide](../troubleshooting.md)
- [FAQ](../faq.md)
