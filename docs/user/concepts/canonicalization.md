# Canonicalization: The Art of One True Form

You're reviewing a pull request. The diff shows hundreds of lines changed. You scroll through, trying to understand what actually changed. But wait. The only real change was adding one user to a list. Everything else? Whitespace. Someone reformatted the file. Tabs became spaces. Trailing whitespace got trimmed. Line endings changed from Windows to Unix.

The actual change, the one line that matters, is buried in noise.

This is a solved problem. Code formatters like prettier, black, and rustfmt have trained us to expect deterministic formatting. You run the formatter, and your code looks the same as everyone else's code. Diffs are meaningful. Reviews focus on logic, not style.

HEDL brings this same discipline to data. Run `hedl format` on any HEDL document, and you get the **canonical form**: a single, deterministic representation that looks exactly the same whether you wrote it or your colleague wrote it, whether you're on Windows or Linux, whether you prefer two-space indentation or four.

This page will teach you what canonical form means, why it matters, and how to use it effectively.

---

## The Problem Canonicalization Solves

Let's make this concrete. Here's the same data, written two different ways:

**Alice's version:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
   |u1, Alice, alice@example.com
   |u2, Bob, bob@example.com
```

**Bob's version:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[ id,name,email ]
---
users:@User
 |u1,Alice,alice@example.com
 |u2,Bob,bob@example.com
```

Both documents represent the same data. Parse them, and you get identical structures. But as text files, they're different:

- Alice uses three-space indentation. Bob uses one space.
- Alice has spaces after commas in values. Bob doesn't.
- Both have inconsistent schema formatting.

If these files are in version control, every time one developer touches the file, the formatting might change. Diffs become noisy. Blame becomes useless. Code review becomes tedious.

**Canonical form solves this.** After formatting, both documents become:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |u1,Alice,alice@example.com
 |u2,Bob,bob@example.com
```

One representation. Every time. No matter who wrote it.

---

## What "Canonical" Means

A canonical form has a specific property: **given the same data, it always produces the exact same output**.

Not "similar" output. Not "equivalent" output. The **exact same bytes**. Character for character. If you hash the output, you get the same hash every time.

This property enables powerful operations:

**Equality checking.** Want to know if two documents contain the same data? Format both, compare the bytes. If they match, the data is identical.

**Hashing.** Want a unique identifier for your data? Format it, hash it. The hash is deterministic and stable.

**Diffing.** Want to see what changed? When both versions are canonical, the diff shows only real changes. No noise.

**Caching.** Want to avoid reprocessing unchanged data? Use the canonical form's hash as a cache key. Same data, same hash, cache hit.

**Verification.** Want to prove data hasn't been tampered with? Store its canonical form's hash. Later, recompute and compare.

---

## The Canonicalization Rules

HEDL's canonical form follows specific, documented rules. Here's every rule, explained:

### Rule 1: One Space Per Level

Indentation uses exactly one space per nesting level. Not two. Not four. Not tabs. One space.

**Before canonicalization:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
config:
        host:localhost
        port:8080
        database:
                name:production
                pool_size:20
```

**After canonicalization:**
```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
config:
 host:localhost
 port:8080
 database:
  name:production
  pool_size:20
```

Why one space? Because it's minimal. Every character in a HEDL document costs tokens when sent to an LLM. One-space indentation preserves hierarchy while minimizing overhead.

The rule is strict because strictness enables determinism. If you could choose between one or two spaces, different tools might choose differently. By mandating one space, every tool produces the same output.

### Rule 2: No Whitespace in Schemas

Schema declarations have no spaces after commas:

**Before:**
```hedl
%S:User:[id,name,email]
%S:Product:[sku, name,price]
```

**After:**
```hedl
%S:User:[id,name,email]
%S:Product:[sku,name,price]
```

Compact, consistent, deterministic.

### Rule 3: No Whitespace in Values

Inline child rows have no spaces after commas:

**Before:**
```hedl
users:@User
 |u1, Alice,   alice@example.com
 |u2,  Bob, bob@example.com
```

**After:**
```hedl
users:@User
 |u1,Alice,alice@example.com
 |u2,Bob,bob@example.com
```

Excess whitespace is trimmed.

### Rule 4: No Space Before Type Annotation

Entity type annotations have no space before the `@`:

**Before:**
```hedl
users: @User
```

**After:**
```hedl
users:@User
```

### Rule 5: Unix Line Endings

All lines end with LF (`\n`), the Unix convention. Windows CRLF (`\r\n`) gets converted.

This matters for cross-platform consistency. A file edited on Windows and a file edited on Linux produce the same canonical form.

### Rule 6: No Trailing Whitespace

Lines end with their content, not with invisible spaces:

**Before:**
```hedl
users:@User
 |u1,Alice
```
(Those trailing spaces are invisible but present)

**After:**
```hedl
users:@User
 |u1,Alice
```

Trailing whitespace is noise. It causes diffs. It wastes bytes. It's gone.

### Rule 7: Single Trailing Newline

The file ends with exactly one newline character. Not zero. Not two. One.

**Before:**
```hedl
users:@User
 |u1,Alice


```

**After:**
```hedl
users:@User
 |u1,Alice
```

### Rule 8: Preservation of Order

The order of entities is preserved. Canonicalization doesn't alphabetize or reorganize your document. If you put `books` before `authors`, it stays that way.

This is deliberate. Document order often carries semantic meaning. Authors might logically come before books (because books reference authors). Canonicalization respects your choices about ordering.

### Rule 9: Preservation of Data

Canonicalization never changes your data. Values are preserved exactly. Only formatting changes.

**Before:**
```hedl
 |u1,   Alice Chen   ,alice@example.com
```

**After:**
```hedl
 |u1,Alice Chen,alice@example.com
```

The name "Alice Chen" is preserved. The extra spaces around the name are gone because they were whitespace between fields, not part of the value. If you need spaces as part of a value, quote it.

---

## Using the Format Command

The `hedl format` command applies canonicalization:

### Basic Usage

```bash
# Format to standard output
hedl format document.hedl

# Format to a new file
hedl format document.hedl -o formatted.hedl

# Format in place (overwrites original)
hedl format document.hedl -o document.hedl
```

### Checking Without Modifying

The `--check` flag verifies whether a file is already canonical:

```bash
hedl format --check document.hedl
```

If the file is canonical: exit code 0, no output.
If the file is not canonical: exit code 1, shows what would change.

This is perfect for CI pipelines:

```bash
hedl format --check document.hedl || {
  echo "File is not in canonical form. Run: hedl format document.hedl"
  exit 1
}
```

### Batch Formatting

Format multiple files at once:

```bash
# Format all HEDL files in a directory
hedl batch-format data/*.hedl --output-dir formatted/

# Check all files without modifying
hedl batch-format data/*.hedl --check
```

Batch operations run in parallel. Large directories format quickly.

---

## Canonicalization in Your Workflow

Let's explore how canonicalization fits into real development workflows.

### Pre-Commit Hook

Automatically format files before every commit:

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Find staged HEDL files
staged_hedl_files=$(git diff --cached --name-only --diff-filter=ACM | grep '\.hedl$')

if [ -n "$staged_hedl_files" ]; then
  for file in $staged_hedl_files; do
    # Format the file
    hedl format "$file" -o "$file"
    # Re-add it (in case formatting changed it)
    git add "$file"
  done
fi
```

With this hook, you never commit non-canonical HEDL. The formatter runs automatically, and the formatted version is what gets committed.

### CI Pipeline Check

Catch non-canonical files in CI:

```yaml
# GitHub Actions example
name: Check HEDL Formatting

on: [push, pull_request]

jobs:
  format-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install HEDL CLI
        run: cargo install hedl-cli

      - name: Check canonical form
        run: |
          find . -name "*.hedl" -exec hedl format --check {} \; || {
            echo "Some files are not in canonical form."
            echo "Run 'hedl format <file>' locally and commit the result."
            exit 1
          }
```

This fails the build if any HEDL file isn't canonical. Developers get clear feedback about what to fix.

### Editor Integration

If you use the HEDL LSP server with your editor, you can enable format-on-save. Every time you save a HEDL file, it's automatically formatted to canonical form.

In VS Code with the HEDL extension, this is typically a setting:

```json
{
  "[hedl]": {
    "editor.formatOnSave": true
  }
}
```

Write your HEDL however you like. Save. It's canonical.

---

## The Power of Determinism

Let's see what deterministic formatting enables in practice.

### Content-Based Addressing

You can identify documents by their content:

```bash
# Create a content-based filename
hash=$(hedl format document.hedl | sha256sum | cut -d' ' -f1)
hedl format document.hedl -o "documents/${hash}.hedl"
```

Now the filename is the hash of the content. If the content changes, the hash changes, and it gets a new filename. If two files have the same content, they have the same hash and can be deduplicated.

This pattern is used in content-addressable storage, build caching, and distributed systems.

### Data Integrity Verification

Store a hash alongside your data:

```bash
# Create the hash
hedl format data.hedl | sha256sum > data.hedl.sha256

# Later, verify integrity
hedl format data.hedl | sha256sum -c data.hedl.sha256
```

If the data has changed (or been corrupted or tampered with), the hash won't match.

### Change Detection

Detect whether data actually changed, ignoring formatting noise:

```bash
# Compare two files by their canonical forms
diff <(hedl format old.hedl) <(hedl format new.hedl)
```

If the diff is empty, the data is the same, even if the files looked different.

### Reproducible Data Pipelines

When your pipeline produces HEDL output, canonical form ensures reproducibility:

```bash
# Pipeline step: transform data
transform_data input.hedl | hedl format - -o output.hedl

# Same input, same output, every time
```

This matters for debugging and auditing. If you can reproduce the exact output, you can verify the pipeline is working correctly.

---

## Idempotence: Format Once, Format Forever

Canonical form is **idempotent**: formatting an already-canonical file produces the same file.

```bash
# Format once
hedl format document.hedl -o document.hedl

# Format again
hedl format document.hedl -o document.hedl

# Format a third time
hedl format document.hedl -o document.hedl

# All produce identical output
```

This means you can safely format files multiple times. Pre-commit hooks, CI checks, manual formatting: they all compose. Nothing breaks if formatting runs twice.

Idempotence is a key property for tooling reliability. You don't have to track whether something was already formatted. Just format it. If it was already canonical, nothing changes.

---

## Losslessness: Your Data Is Safe

Canonicalization is **lossless**: it changes formatting, never data.

```bash
# Format the file
hedl format original.hedl -o formatted.hedl

# Convert both to JSON
hedl to-json original.hedl -o original.json
hedl to-json formatted.hedl -o formatted.json

# The JSON is identical
diff original.json formatted.json
# No output: files are the same
```

Your actual data, the entities, values, relationships, is preserved exactly. Only the textual representation changes.

This guarantee is crucial. You must be able to trust that formatting won't break your data. HEDL's canonicalization respects that trust.

---

## Comparison with Other Formats

Other data formats have canonicalization too, but it's often complicated.

### JSON Canonicalization

JSON has multiple competing standards for canonical form:

- [RFC 8785 (JCS)](https://tools.ietf.org/html/rfc8785): JSON Canonicalization Scheme
- Various "sorted keys" approaches
- Different whitespace conventions

Each standard produces different output. You have to know which one a system uses.

JSON also has ambiguities that complicate canonicalization:
- Should Unicode be escaped or literal?
- What precision for floating-point numbers?
- How to handle key ordering when keys are inserted dynamically?

HEDL avoids these issues by having one canonical form defined as part of the language.

### YAML Canonicalization

YAML is particularly difficult to canonicalize because the same data can be represented many ways:

```yaml
# All equivalent YAML:
name: Alice
"name": "Alice"
name: 'Alice'
? name
: Alice
```

Which is canonical? YAML doesn't say. Different libraries produce different output.

HEDL's simpler syntax means less ambiguity. A string is written one way. An entity is written one way. Canonicalization is straightforward.

### XML Canonicalization

XML has formal canonicalization specifications (C14N, C14N11) that handle:
- Namespace declarations
- Attribute ordering
- Whitespace in mixed content
- Character encoding
- Entity references

These specs are comprehensive but complex. Implementing them correctly is challenging.

HEDL doesn't have namespaces, attributes, or mixed content. Its canonicalization rules fit on one page and are easy to implement correctly.

---

## Best Practices

Based on experience with canonical forms in production, here are recommendations:

### Format Early, Format Often

Don't wait until commit time. Format as you work. If your editor supports format-on-save, enable it. Seeing canonical form immediately helps you internalize the style.

### Check in CI, Format Locally

CI should check that files are canonical. It shouldn't format them. The workflow is:

1. Developer runs `hedl format` locally
2. Developer commits the canonical file
3. CI checks that the file is canonical
4. If not canonical, CI fails and tells the developer to format

This keeps the responsibility with developers and avoids CI making commits.

### Format After Conversion

When converting from other formats, pipe through format:

```bash
# From JSON
hedl from-json data.json | hedl format - -o data.hedl

# From YAML
hedl from-yaml config.yaml | hedl format - -o config.hedl
```

The converter produces valid HEDL, but not necessarily canonical HEDL. The format step normalizes it.

### Format Before Hashing

Always format before computing hashes:

```bash
# Right
hedl format data.hedl | sha256sum

# Wrong (might vary based on input formatting)
sha256sum data.hedl
```

The first command gives you a content hash. The second gives you a file hash that includes formatting variations.

### Document Your Canonical Form Expectation

In your project's README or contributing guide, note that HEDL files should be in canonical form:

```markdown
## HEDL Files

All `.hedl` files in this repository should be in canonical form.
Run `hedl format <file>` before committing, or use the pre-commit hook.
```

This sets expectations for contributors.

---

## Troubleshooting

### "My file changes every time I format it"

This shouldn't happen. If it does:

1. Check that you're using the same version of the HEDL CLI
2. Ensure you're not mixing different formatting tools
3. File a bug report: canonicalization should be idempotent

### "The formatter changed my strings"

Canonicalization preserves string content. If a string changed, check:

1. Was there whitespace outside the quotes that got normalized?
2. Was the string unquoted when it should have been quoted?

Format doesn't change quoted string content. It normalizes the text around them.

### "Two files look the same but have different hashes"

They probably have invisible differences:

```bash
# Check for hidden characters
hexdump -C file1.hedl > hex1
hexdump -C file2.hedl > hex2
diff hex1 hex2
```

Common culprits: different line endings, trailing whitespace, BOM characters.

Format both files and the hashes will match.

---

## What You've Learned

Canonicalization gives you deterministic, reproducible formatting:

**One canonical form** means the same data always produces the same text.

**Specific rules** govern indentation (one space), whitespace (normalized), line endings (Unix), and more.

**The format command** applies canonicalization. Use `--check` to verify without modifying.

**Idempotence** means formatting is safe to repeat.

**Losslessness** means your data is never changed.

**Practical applications** include clean diffs, reliable hashing, content addressing, and integrity verification.

**Integration** through pre-commit hooks, CI checks, and editor format-on-save makes canonicalization automatic.

---

## Where to Go Next

You've completed the four core concepts! Let's recap your journey:

1. **[Data Model](data-model.md)** taught you how HEDL organizes information: entities, matrix lists, nesting.

2. **[Type System](type-system.md)** taught you how values get their types: inference, explicit annotations, validation.

3. **[References](references.md)** taught you how entities connect: the `@id` syntax, relationship patterns, validation.

4. **Canonicalization** (this page) taught you how formatting becomes deterministic: rules, workflows, integration.

Together, these concepts give you deep understanding of HEDL. You know not just what syntax to write, but why HEDL works the way it does.

Now go apply this knowledge:

- **[CLI Guide](../cli-guide.md)** covers every command and option in depth
- **[Examples](../examples.md)** shows real-world patterns you can adapt
- **[Getting Started](../getting-started.md)** provides a hands-on tutorial if you want more practice

Or return to the [Concepts overview](README.md) to review how all four concepts fit together.

You understand HEDL. Now use it.
