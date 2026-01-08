# Tutorial: Your First Conversion

**Time:** 10 minutes | **Difficulty:** Beginner

In this tutorial, you'll learn the fundamentals of HEDL by converting a simple JSON file to HEDL format and back. This hands-on exercise will help you understand HEDL's syntax, structure, and token efficiency benefits.

## What You'll Learn

- Basic HEDL document structure
- Converting between JSON and HEDL
- Validating HEDL files
- Understanding token efficiency

## Prerequisites

- HEDL CLI installed (see [Getting Started](../getting-started.md))
- A text editor
- Basic command-line knowledge

## Step 1: Create a Sample JSON File

Let's start with a simple product catalog. Create a file named `products.json`:

```json
{
  "products": [
    {
      "id": "p1",
      "name": "Wireless Mouse",
      "category": "Electronics",
      "price": 29.99,
      "in_stock": true
    },
    {
      "id": "p2",
      "name": "Mechanical Keyboard",
      "category": "Electronics",
      "price": 149.99,
      "in_stock": true
    },
    {
      "id": "p3",
      "name": "USB-C Hub",
      "category": "Electronics",
      "price": 79.99,
      "in_stock": false
    },
    {
      "id": "p4",
      "name": "Monitor Stand",
      "category": "Accessories",
      "price": 44.99,
      "in_stock": true
    }
  ]
}
```

Save this file in your working directory.

## Step 2: Convert JSON to HEDL

Now let's convert this JSON file to HEDL format:

```bash
hedl from-json products.json -o products.hedl
```

**What happened?**
- HEDL read the JSON file
- Inferred the structure of the data
- Generated a compact HEDL representation
- Saved it to `products.hedl`

Open `products.hedl` to see the result:

```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, category, price, in_stock]
---
products: @Product
  | p1, Wireless Mouse, Electronics, 29.99, true
  | p2, Mechanical Keyboard, Electronics, 149.99, true
  | p3, USB-C Hub, Electronics, 79.99, false
  | p4, Monitor Stand, Accessories, 44.99, true
```

**Notice the differences:**
- **Header**: `%VERSION: 1.0` declares the format version, `%STRUCT:` defines the schema, followed by `---` separator
- **Matrix list**: `products: @Product` references the struct, with pipe-prefixed rows
- **Compact rows**: Each product is a single line with comma-separated values
- **No repetition**: Field names are defined once in `%STRUCT:`, not repeated for each item

## Step 3: Validate the HEDL File

Let's verify that our HEDL file is syntactically correct:

```bash
hedl validate products.hedl
```

You should see:
```
✓ products.hedl is valid
```

**What does validation check?**
- Syntax correctness (proper indentation, quotes, etc.)
- Structure consistency (matching column counts)
- Type compatibility (numbers, booleans, strings)

## Step 4: Convert Back to JSON

Now let's convert our HEDL file back to JSON to verify the roundtrip:

```bash
hedl to-json products.hedl --pretty
```

You should see the original JSON structure printed to the console. Save it to a file:

```bash
hedl to-json products.hedl --pretty -o products_roundtrip.json
```

Compare the files:

```bash
# On Linux/Mac
diff products.json products_roundtrip.json

# On Windows PowerShell
Compare-Object (Get-Content products.json) (Get-Content products_roundtrip.json)
```

The data should be identical (though formatting might differ slightly).

## Step 5: Compare Token Efficiency

Let's see how much more efficient HEDL is:

```bash
hedl stats products.hedl
```

You should see output like:

```
Format Comparison for products.hedl:
  HEDL:    187 bytes,  52 tokens (baseline)
  JSON:    456 bytes, 132 tokens (+143%, +80 tokens)
  YAML:    342 bytes,  98 tokens (+82%, +46 tokens)
  XML:     612 bytes, 168 tokens (+227%, +116 tokens)

Token Savings:
  vs JSON: 60% fewer tokens
  vs YAML: 47% fewer tokens
  vs XML:  69% fewer tokens
```

**Why is HEDL more efficient?**
- Field names defined once, not repeated
- Compact syntax with minimal punctuation
- No redundant nesting or brackets
- Type inference reduces explicit type annotations

## Step 6: Experiment with Formatting

HEDL has a canonical format. Try formatting your file:

```bash
hedl format products.hedl
```

The formatted output will have:
- Consistent spacing
- Standardized indentation
- Deterministic ordering

Save the formatted version:

```bash
hedl format products.hedl -o products_formatted.hedl
```

## Step 7: Add More Data with the Ditto Operator

Edit your `products.hedl` file to add more products, using the ditto operator (`^`) to repeat values:

```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, category, price, in_stock]
---
products: @Product
  | p1, Wireless Mouse, Electronics, 29.99, true
  | p2, Mechanical Keyboard, Electronics, 149.99, true
  | p3, USB-C Hub, Electronics, 79.99, false
  | p4, Monitor Stand, Accessories, 44.99, true
  | p5, Webcam HD, Electronics, 89.99, ^
  | p6, Desk Lamp, Accessories, 34.99, ^
  | p7, Cable Organizer, Accessories, 12.99, ^
```

**Note:** The `^` operator copies the value from the previous row's same column. In this case:
- Row p5: `^` means `true` (from p4's in_stock)
- Row p6: `^` means `true` (from p5's in_stock)
- Row p7: `^` means `true` (from p6's in_stock)

Validate the updated file:

```bash
hedl validate products.hedl
```

## Step 8: Lint for Best Practices

HEDL can check for potential issues and best practices:

```bash
hedl lint products.hedl
```

The linter checks for:
- Inefficient patterns (could use ditto but doesn't)
- Inconsistent naming conventions
- Potential data quality issues
- Suboptimal structure

## Understanding What You've Learned

### HEDL Document Structure

Every HEDL file has:

1. **Header**: `%VERSION: 1.0` (version declaration)
2. **Separator**: `---` (separates header from body)
3. **Entities**: Named collections of data
4. **Matrix Lists**: Typed entity collections with a schema
5. **Values**: Individual data items (strings, numbers, booleans)

### Matrix List Syntax

```hedl
%STRUCT: TypeName: [col1, col2, col3]
---
entity_name: @TypeName
  | id1, value1, value2, value3
  | id2, value1, value2, value3
```

Where:
- `%STRUCT:` declaration defines the type and columns
- `entity_name:` is the key for this collection
- `@TypeName` references the struct type
- Each row starts with `| ` (pipe and space)
- Values are comma-separated

### Conversion Workflow

```
JSON ←→ HEDL ←→ YAML
         ↕
       CSV/Parquet/XML
```

HEDL acts as a universal interchange format.

## Practical Applications

Now that you understand the basics, you can:

1. **Convert API responses** - Turn JSON from APIs into compact HEDL for storage
2. **Reduce token costs** - Use HEDL to save 40-60% on LLM token usage
3. **Process CSV data** - Import CSV files, manipulate them, export to other formats
4. **Validate data** - Ensure data consistency with HEDL's validation

## Common Pitfalls

### ❌ Forgetting the header
```hedl
products: @Product  ← Missing "%VERSION: 1.0", %STRUCT, and "---"
  | p1, Test
```

### ✓ Always include the version header, struct, and separator
```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name]
---
products: @Product
  | p1, Test
```

### ❌ Inconsistent indentation
```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name]
---
products: @Product
 | p1, Test     ← Wrong indentation
  | p2, Test2   ← Correct (2 spaces + |)
```

### ✓ Use exactly 2 spaces for indentation
```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name]
---
products: @Product
  | p1, Test
  | p2, Test2
```

### ❌ Mismatched column count
```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, price]
---
products: @Product
  | p1, Test, 99.99
  | p2, Test2        ← Missing price!
```

### ✓ All rows must have the same number of columns
```hedl
%VERSION: 1.0
%STRUCT: Product: [id, name, price]
---
products: @Product
  | p1, Test, 99.99
  | p2, Test2, 149.99
```

## Practice Exercises

Try these exercises to reinforce your learning:

### Exercise 1: Personal Library
Create a HEDL file for a personal book library with:
- Book ID, title, author, year, genre
- At least 5 books
- Use the ditto operator for books by the same author

<details>
<summary>Solution</summary>

```hedl
%VERSION: 1.0
%STRUCT: Book: [id, title, author, year, genre]
---
books: @Book
  | b1, The Rust Programming Language, Steve Klabnik, 2018, Programming
  | b2, Programming Rust, Jim Blandy, 2021, ^
  | b3, Rust in Action, Tim McNamara, ^, ^
  | b4, Zero to Production in Rust, Luca Palmieri, 2022, ^
  | b5, Clean Code, Robert Martin, 2008, Software Engineering
```
</details>

### Exercise 2: Convert and Validate
1. Create a JSON file with user data (name, email, age)
2. Convert it to HEDL
3. Validate the HEDL file
4. Format it to canonical form
5. Convert back to JSON and verify it matches

### Exercise 3: Token Comparison
Take a JSON API response from any public API (or create a sample) and:
1. Save it as JSON
2. Convert to HEDL
3. Run `hedl stats` to compare sizes
4. Calculate the percentage savings

## Next Steps

Congratulations! You've completed your first conversion and understand the basics of HEDL.

**Continue your learning:**
- [Tutorial 2: CLI Basics](02-cli-basics.md) - Master the command-line tools
- [How-To: Convert Formats](../how-to/convert-formats.md) - More conversion recipes
- [Concepts: Data Model](../concepts/data-model.md) - Deep dive into HEDL's structure

**Quick Reference:**
```bash
# Convert to HEDL
hedl from-json file.json -o file.hedl

# Convert from HEDL
hedl to-json file.hedl --pretty -o file.json

# Validate
hedl validate file.hedl

# Format
hedl format file.hedl -o formatted.hedl

# Compare sizes
hedl stats file.hedl
```

---

**Questions?** Check the [FAQ](../faq.md) or [Troubleshooting](../troubleshooting.md) guides!
