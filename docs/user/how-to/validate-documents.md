# How-To: Validate Documents

Practical guide for validating HEDL documents to ensure correctness and quality.

## Table of Contents

1. [Basic Validation](#basic-validation)
2. [Schema Validation](#schema-validation)
3. [Reference Integrity](#reference-integrity)
4. [Custom Validation Rules](#custom-validation-rules)
5. [Batch Validation](#batch-validation)
6. [CI/CD Integration](#cicd-integration)
7. [Validation Reporting](#validation-reporting)

---

## Basic Validation

### Validate a Single File

**Goal:** Check if a HEDL file is syntactically correct.

**Command:**
```bash
hedl validate myfile.hedl
```

**Success output:**
```
✓ myfile.hedl is valid
```

**Error output:**
```
✗ myfile.hedl is invalid
Error on line 5: Expected 3 values but found 2
  Row: u2 "Bob"
  Expected: id, name, email
```

### Validate from Stdin

**Pipeline validation:**
```bash
cat data.hedl | hedl validate -
echo $?  # 0 = valid, 1 = invalid
```

### Exit Codes

- `0` - File is valid
- `1` - File is invalid
- `2` - File not found or unreadable

**Use in scripts:**
```bash
if hedl validate data.hedl; then
  echo "Validation passed"
  # Continue processing
else
  echo "Validation failed"
  exit 1
fi
```

---

## Schema Validation

### Type Checking

**Goal:** Ensure values match expected types.

**Example file with type errors:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name, age, active]
---
users: @User
  | u1, Alice, 30, true
  | u2, Bob, twenty-five, true  # Error: age should be number
```

**Validation detects type error:**
```bash
hedl validate users.hedl
```

Output:
```
✗ users.hedl is invalid
Error on line 4: Type mismatch
  Column: age
  Expected: Number
  Found: String ("twenty-five")
```

### Column Count Validation

**Ensure consistent structure:**
```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, price]
---
products: @Product
  | p1, Laptop, 999.99
  | p2, Mouse             # Missing price
```

```bash
hedl validate products.hedl
```

Output:
```
✗ products.hedl is invalid
Error on line 4: Column count mismatch
  Expected: 3 columns (id, name, price)
  Found: 2 columns
```

---

## Reference Integrity

### Validate References

**Goal:** Ensure all references point to existing entities.

**Example with broken reference:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author, title]
---
users: @User
  | u1, Alice
  | u2, Bob

posts: @Post
  | p1, @User:u3, My Post  # u3 doesn't exist
```

**Validation:**
```bash
hedl validate blog.hedl
```

Output:
```
✗ blog.hedl is invalid
Error on line 7: Unresolved reference
  Reference: @User:u3
  Available: u1, u2
```

### Cross-Entity References

**Validate complex relationships:**
```bash
hedl validate --strict complex.hedl
```

---

## Batch Validation

### Validate Multiple Files

**Simple batch:**
```bash
hedl batch-validate data/*.hedl
```

**Output:**
```
Validating 5 files...
✓ data/users.hedl (valid)
✓ data/products.hedl (valid)
✗ data/orders.hedl (invalid)
✓ data/reviews.hedl (valid)
✓ data/inventory.hedl (valid)

Summary: 4/5 files valid (80%)
```

### Parallel Batch Validation

**Use all CPU cores:**
```bash
hedl batch-validate data/*.hedl --parallel
```

### Validation Script

**Create `validate_all.sh`:**
```bash
#!/bin/bash

VALID=0
INVALID=0
ERRORS_FILE="validation_errors.log"

> "$ERRORS_FILE"  # Clear file

for file in data/*.hedl; do
  if hedl validate "$file" 2>&1 | tee -a "$ERRORS_FILE" | grep -q "valid"; then
    echo "✓ $file"
    VALID=$((VALID + 1))
  else
    echo "✗ $file"
    INVALID=$((INVALID + 1))
  fi
done

echo ""
echo "Results: $VALID valid, $INVALID invalid"
echo "Errors logged to: $ERRORS_FILE"

[ $INVALID -eq 0 ]  # Exit 0 if all valid
```

---

## CI/CD Integration

### GitHub Actions

**`.github/workflows/validate.yml`:**
```yaml
name: Validate HEDL Files

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install HEDL CLI
        run: cargo install hedl-cli

      - name: Validate all HEDL files
        run: |
          hedl batch-validate data/**/*.hedl --parallel
          if [ $? -ne 0 ]; then
            echo "Validation failed"
            exit 1
          fi

      - name: Check formatting
        run: |
          for file in data/**/*.hedl; do
            if ! diff <(cat "$file") <(hedl format "$file"); then
              echo "File not formatted: $file"
              exit 1
            fi
          done
```

### GitLab CI

**`.gitlab-ci.yml`:**
```yaml
validate-hedl:
  stage: test
  image: rust:latest
  before_script:
    - cargo install hedl-cli
  script:
    - hedl batch-validate data/**/*.hedl --parallel
  only:
    changes:
      - "**/*.hedl"
```

### Pre-Commit Hook

**`.git/hooks/pre-commit`:**
```bash
#!/bin/bash

# Get staged HEDL files
FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.hedl$')

if [ -n "$FILES" ]; then
  echo "Validating HEDL files..."

  for file in $FILES; do
    if ! hedl validate "$file"; then
      echo "✗ Validation failed: $file"
      echo "Commit aborted. Fix errors and try again."
      exit 1
    fi
  done

  echo "✓ All HEDL files valid"
fi
```

**Install hook:**
```bash
chmod +x .git/hooks/pre-commit
```

---

## Validation Reporting

### Generate Validation Report

**Script (`validation_report.sh`):**
```bash
#!/bin/bash

REPORT="validation_report.html"

cat > "$REPORT" << 'HTML'
<!DOCTYPE html>
<html>
<head>
  <title>HEDL Validation Report</title>
  <style>
    body { font-family: sans-serif; margin: 20px; }
    .valid { color: green; }
    .invalid { color: red; }
    table { border-collapse: collapse; width: 100%; }
    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
    th { background-color: #f2f2f2; }
  </style>
</head>
<body>
  <h1>HEDL Validation Report</h1>
  <p>Generated: $(date)</p>

  <h2>Summary</h2>
  <table>
    <tr><th>Metric</th><th>Value</th></tr>
HTML

TOTAL=0
VALID=0
INVALID=0

for file in data/*.hedl; do
  TOTAL=$((TOTAL + 1))
  if hedl validate "$file" 2>/dev/null; then
    VALID=$((VALID + 1))
  else
    INVALID=$((INVALID + 1))
  fi
done

cat >> "$REPORT" << HTML
    <tr><td>Total Files</td><td>$TOTAL</td></tr>
    <tr><td>Valid Files</td><td class="valid">$VALID</td></tr>
    <tr><td>Invalid Files</td><td class="invalid">$INVALID</td></tr>
    <tr><td>Success Rate</td><td>$(( VALID * 100 / TOTAL ))%</td></tr>
  </table>

  <h2>File Details</h2>
  <table>
    <tr><th>File</th><th>Status</th><th>Details</th></tr>
HTML

for file in data/*.hedl; do
  if hedl validate "$file" 2>/dev/null; then
    echo "    <tr><td>$file</td><td class='valid'>✓ Valid</td><td>-</td></tr>" >> "$REPORT"
  else
    errors=$(hedl validate "$file" 2>&1 | sed 's/</\&lt;/g; s/>/\&gt;/g')
    echo "    <tr><td>$file</td><td class='invalid'>✗ Invalid</td><td><pre>$errors</pre></td></tr>" >> "$REPORT"
  fi
done

cat >> "$REPORT" << 'HTML'
  </table>
</body>
</html>
HTML

echo "Report generated: $REPORT"
```

---

## Best Practices

### 1. Validate Early and Often

```bash
# In development workflow
edit file.hedl
hedl validate file.hedl
git add file.hedl
git commit
```

### 2. Automate Validation

- Pre-commit hooks
- CI/CD pipelines
- Automated tests

### 3. Use Batch Validation for Large Projects

```bash
# Daily validation
hedl batch-validate **/*.hedl --parallel > daily_validation.log
```

### 4. Fix Issues Immediately

Don't accumulate validation errors. Fix them as they occur.

### 5. Document Validation Requirements

Create a `VALIDATION.md` in your project explaining validation rules.

---

## Quick Reference

```bash
# Basic validation
hedl validate file.hedl

# Batch validation
hedl batch-validate *.hedl
hedl batch-validate **/*.hedl --parallel

# With custom rules
# (Custom rules support is planned for a future release)

# Exit code checking
hedl validate file.hedl && echo "Valid" || echo "Invalid"
```

---

**Related Guides:**
- [Handle Errors](handle-errors.md) - Fix validation errors
- [Convert Formats](convert-formats.md) - Validate after conversion
- [CLI Reference](../reference/cli-commands.md#validate) - Complete validation options
