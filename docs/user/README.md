# The HEDL User Guide

Picture this: You've just exported 50,000 user records from your database. You need to send them to Claude for analysis. You serialize to JSON, and the file is 12 megabytes. At roughly 4 characters per token, that's 3 million tokens. At current pricing, this single request costs you $9.

But wait. Open that JSON file. Look at it. Really look at it.

```json
{"id": "u00001", "name": "Alice Chen", "email": "alice@company.com", "role": "engineer", "department": "backend"}
{"id": "u00002", "name": "Bob Smith", "email": "bob@company.com", "role": "designer", "department": "product"}
{"id": "u00003", "name": "Carol Wu", "email": "carol@company.com", "role": "manager", "department": "backend"}
```

Fifty thousand times, you're paying for `"id":`. Fifty thousand times for `"name":`. Fifty thousand times for `"email":`, `"role":`, `"department":`. Those five field names, repeated fifty thousand times, account for roughly 40% of your file size.

You're not paying $9 for data. You're paying $4 for data and $5 for repetition.

Now imagine the same data in HEDL:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email,role,department]
---
users:@User
 |u00001,Alice Chen,alice@company.com,engineer,backend
 |u00002,Bob Smith,bob@company.com,designer,product
 |u00003,Carol Wu,carol@company.com,manager,backend
```

The field names appear exactly once, in the schema declaration on line 4. After that, it's pure values. No keys. No quotes around simple strings. No curly braces.

The HEDL file is 5.2 megabytes. You just saved $3.90 on a single request.

This guide will teach you how to make that happen.

---

## Chapter 1: Your First HEDL Document

Let's write something. Open your terminal and create a file called `hello.hedl`:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
message:Hello, World!
```

Five lines. Let's understand each one.

**Line 1: `%V:2.0`**

This declares the HEDL version. Every HEDL document starts with this. It tells parsers which rules to follow.

**Line 2: `%NULL:~`**

This defines the null symbol. In HEDL, when something has no value, you write `~`. This line makes that official.

**Line 3: `%QUOTE:"`**

This defines the quote character. When your strings contain special characters (commas, colons), you'll wrap them in quotes. This line says we're using double quotes.

**Line 4: `---`**

This separator marks the end of the header and the beginning of your data. Everything above is configuration. Everything below is content.

**Line 5: `message: Hello, World!`**

Finally, actual data. A key-value pair. The key is `message`, the value is `Hello, World!`. Notice there are no quotes around the string. HEDL doesn't require them for simple values.

Now let's validate it:

```bash
hedl validate hello.hedl
```

If you see no output, congratulations. Silence means success. Your document is valid.

Let's convert it to JSON to see what we've got:

```bash
hedl to-json hello.hedl
```

Output:
```json
{"message":"Hello, World!"}
```

Same information. Different representation. You just wrote your first HEDL document.

---

## Chapter 2: The Three Headers You'll Always Write

Every HEDL document starts the same way:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
```

Memorize this. Your fingers will type it automatically after a few documents.

**Why are these required?**

Because explicitness prevents ambiguity. In YAML, is `null` a string or a null value? Depends on context. In HEDL, null is always `~` because we declared it. No surprises.

**Can I change them?**

The version (`%V:2.0`) is fixed. The null symbol and quote character are configurable, but `~` and `"` are the conventions. Stick with them unless you have a specific reason.

---

## Chapter 3: Key-Value Pairs

The simplest HEDL structure is key-value pairs:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
name:Alice Chen
email:alice@company.com
age:32
active:true
balance:1250.50
notes:~
```

Each line is a key, a colon, and a value. Values are automatically typed:

| Value | HEDL Type | Notes |
|-------|-----------|-------|
| `Alice Chen` | String | Bare strings work if they don't contain special characters |
| `32` | Integer | Whole numbers become integers |
| `true` | Boolean | `true` or `false` |
| `1250.50` | Float | Numbers with decimals become floats |
| `~` | Null | The absence of a value |

**When do you need quotes?**

When your string contains commas, colons, or the quote character itself:

```hedl
title:"Hello, World!"
description:"Contains a colon: like this"
quote:"She said ""yes"" immediately"
```

Inside quotes, double the quote character to include a literal quote.

---

## Chapter 4: Nesting

Real data has structure. HEDL handles nesting with indentation:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
user:
 name:Alice Chen
 email:alice@company.com
 address:
  street:742 Evergreen Terrace
  city:Springfield
  country:USA
 preferences:
  theme:dark
  notifications:true
```

**The indentation rule is simple: one space per level.**

Not two spaces. Not four. Not tabs. Exactly one space.

This might feel restrictive if you're used to YAML's flexible indentation. But think about it: with fixed indentation, every HEDL document looks the same. Diffs are meaningful. Formatting is deterministic. There's no debate about style.

The JSON equivalent of that document:

```json
{
  "user": {
    "name": "Alice Chen",
    "email": "alice@company.com",
    "address": {
      "street": "742 Evergreen Terrace",
      "city": "Springfield",
      "country": "USA"
    },
    "preferences": {
      "theme": "dark",
      "notifications": true
    }
  }
}
```

Count the characters. The JSON is 289. The HEDL is 218. That's 25% smaller, and we haven't even used schemas yet.

---

## Chapter 5: Lists with Inline Children

Here's where HEDL starts to shine. When you have a list of items:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
fruits:
 |apple
 |banana
 |cherry
 |date
```

Each `|` starts an **inline child**. It's a compact way to write list items.

The JSON equivalent:

```json
{"fruits": ["apple", "banana", "cherry", "date"]}
```

Similar size so far. But watch what happens when your list items are objects:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
users:
 |Alice,alice@company.com,32
 |Bob,bob@company.com,28
 |Carol,carol@company.com,35
```

Compare to JSON:

```json
{
  "users": [
    ["Alice", "alice@company.com", 32],
    ["Bob", "bob@company.com", 28],
    ["Carol", "carol@company.com", 35]
  ]
}
```

Wait. The JSON gives us arrays, not objects. We've lost the field names. That's not useful.

To get objects in JSON:

```json
{
  "users": [
    {"name": "Alice", "email": "alice@company.com", "age": 32},
    {"name": "Bob", "email": "bob@company.com", "age": 28},
    {"name": "Carol", "email": "carol@company.com", "age": 35}
  ]
}
```

Now we have `"name":`, `"email":`, `"age":` repeated three times. With three users, the overhead is minor. With three thousand users, it's massive.

HEDL solves this with schemas.

---

## Chapter 6: Schemas (The Magic)

This is the feature that makes HEDL worth learning. Watch:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[name,email,age]
---
users:@User
 |Alice,alice@company.com,32
 |Bob,bob@company.com,28
 |Carol,carol@company.com,35
```

Line 4 declares a schema: `%S:User:[name,email,age]`. This says "User records have three columns: name, email, and age."

Line 6 uses the schema: `users:@User`. The `@User` tells the parser "the inline children below follow the User schema."

When you convert this to JSON:

```json
{
  "users": [
    {"name": "Alice", "email": "alice@company.com", "age": 32},
    {"name": "Bob", "email": "bob@company.com", "age": 28},
    {"name": "Carol", "email": "carol@company.com", "age": 35}
  ]
}
```

You get proper objects with named fields. But in the HEDL source, the field names appear exactly once.

**The savings scale with your data:**

| Users | JSON Size | HEDL Size | Savings |
|-------|-----------|-----------|---------|
| 10 | 580 bytes | 280 bytes | 52% |
| 100 | 5,800 bytes | 2,350 bytes | 59% |
| 1,000 | 58,000 bytes | 23,100 bytes | 60% |
| 10,000 | 580,000 bytes | 231,000 bytes | 60% |

As your data grows, the relative overhead of JSON's repeated keys stays constant while your actual data grows. HEDL's single schema declaration becomes increasingly negligible.

---

## Chapter 7: References

Here's something JSON simply cannot do elegantly: linking between entities.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,department]
%S:Task:[id,title,assignee,reviewer]
---
users:@User
 |alice,Alice Chen,engineering
 |bob,Bob Smith,engineering
 |carol,Carol Wu,design

tasks:@Task
 |task-001,Implement login,@alice,@bob
 |task-002,Design dashboard,@carol,@alice
 |task-003,Write documentation,@bob,@carol
```

See those `@alice` values? Those are **references**. They're not strings. They're typed links to entities defined elsewhere in the document.

The parser validates these. If you write `@david` but there's no user with id `david`, you get an error at parse time. Not at runtime. Not when your LLM hallucinates about a nonexistent user. At parse time.

When you convert to JSON, references can expand to full objects or stay as reference strings, depending on your configuration. Either way, the relationships are explicit in the source.

This is powerful for:

**Graph databases:** Export directly to Neo4j Cypher. The references become relationships.

**Data integrity:** Catch broken references before they cause problems.

**Documentation:** Anyone reading the HEDL can see how entities relate.

---

## Chapter 8: The CLI

The `hedl` command-line tool is your Swiss Army knife. Let's cover the essentials.

### Validating

```bash
hedl validate document.hedl
```

Silence means success. Errors get detailed messages with line numbers:

```
Error at line 15: Unresolved reference @david
  Referenced from tasks.task-003.reviewer
  No entity with id 'david' exists
```

### Converting

```bash
# HEDL to JSON
hedl to-json data.hedl -o data.json
hedl to-json data.hedl --pretty  # Formatted output

# JSON to HEDL
hedl from-json data.json -o data.hedl

# Other formats
hedl to-yaml data.hedl
hedl from-yaml config.yaml -o config.hedl
hedl to-xml data.hedl
hedl to-csv data.hedl  # Exports first matrix list
```

### Formatting

```bash
# Canonicalize (deterministic formatting)
hedl format document.hedl -o formatted.hedl

# Check if already canonical (for CI)
hedl format --check document.hedl
```

### Statistics

```bash
hedl stats data.hedl --tokens
```

Output:
```
Format Comparison for 'data.hedl':

Sizes:
  HEDL:         2,458 bytes
  JSON:         3,841 bytes  (+56%)
  YAML:         4,105 bytes  (+67%)

Token Estimates:
  HEDL:         615 tokens
  JSON:         960 tokens   (+56%)
  YAML:         1,026 tokens (+67%)
```

### Batch Operations

```bash
# Validate everything in a directory
hedl batch-validate data/*.hedl

# Format all files
hedl batch-format configs/*.hedl --output-dir formatted/
```

Batch operations run in parallel. Expect 3-5x speedup on multi-core systems.

---

## Chapter 9: Editor Integration

Writing HEDL is much nicer with proper editor support. The `hedl-lsp` server provides:

- Syntax highlighting
- Error squiggles as you type
- Autocomplete for references (`@al` → `@alice`)
- Hover documentation
- Go-to-definition for references
- Format on save

### VS Code

Install the HEDL extension from the marketplace.

### Neovim

```lua
require('lspconfig').hedl_lsp.setup{}
```

### Other Editors

Any editor that supports LSP can use `hedl-lsp`. Run it as a language server and point your editor at it.

---

## Chapter 10: When to Use HEDL

HEDL isn't always the right choice. Let's be honest about when it shines and when you should stick with JSON.

### Use HEDL when:

**Token efficiency matters.** If you're paying per token or fighting context window limits, the 50-60% savings are significant.

**You have repetitive structures.** Lists of objects with the same fields are where schemas really pay off. A single user object? JSON is fine. Ten thousand? HEDL.

**Relationships exist between entities.** Type-safe references catch errors that would slip through JSON.

**Deterministic output matters.** Canonical formatting means identical documents produce identical bytes. Useful for caching, hashing, diffing.

### Stick with JSON when:

**Ecosystem compatibility is critical.** JSON has decades of tooling in every language. HEDL is newer.

**Your data is tiny.** A 50-token document won't benefit meaningfully from HEDL's compression.

**Your team doesn't want to learn something new.** Familiarity has value. Don't force a new format if people are happy with JSON.

---

## What's Next?

You now understand the core of HEDL. Here's where to go deeper:

**Want hands-on practice?**
[Getting Started Tutorial](getting-started.md) walks you through building a real project.

**Need the full CLI reference?**
[CLI Guide](cli-guide.md) documents every command and flag.

**Curious about specific concepts?**
- [Data Model](concepts/data-model.md) explains how HEDL represents different kinds of data
- [Type System](concepts/type-system.md) covers scalars, collections, and references
- [References](concepts/references.md) goes deep on entity linking
- [Canonicalization](concepts/canonicalization.md) explains deterministic formatting

**Want real-world examples?**
[Examples](examples.md) shows patterns for configs, APIs, knowledge graphs, and more.

**Having trouble?**
[FAQ](faq.md) and [Troubleshooting](troubleshooting.md) cover common issues.

---

## Quick Reference

### Document Structure

```hedl
%V:2.0              # Version (required)
%NULL:~             # Null symbol (required)
%QUOTE:"            # Quote character (required)
%S:Name:[col1,col2]  # Schema definition (optional)
---                 # Header/body separator
key:value          # Your data starts here
```

### Value Types

| Syntax | Type | Example |
|--------|------|---------|
| `hello` | String | Bare word |
| `"hello, world"` | String | Quoted (for special chars) |
| `42` | Integer | Whole number |
| `3.14` | Float | Decimal number |
| `true` / `false` | Boolean | |
| `~` | Null | Absence of value |
| `@id` | Reference | Link to entity |
| `[1,2,3]` | Tensor | Numeric array |
| `(a,b,c)` | List | Mixed-type list |

### Common Operations

```bash
hedl validate file.hedl          # Check syntax
hedl to-json file.hedl           # Convert to JSON
hedl from-json file.json         # Convert from JSON
hedl format file.hedl            # Canonicalize
hedl stats file.hedl --tokens    # Compare sizes
```

---

You're ready. Go build something.
