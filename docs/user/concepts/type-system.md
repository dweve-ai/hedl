# The Type System: Teaching HEDL What Your Values Mean

Here's a riddle. Look at this value: `42`. What is it?

In one context, it's a number. The answer to life, the universe, and everything. You can add 1 to it and get 43.

In another context, it's a string. A label. A product code. Adding 1 to it would be meaningless.

In yet another context, it might be a reference. User #42. The forty-second user in your system.

The same characters, `4` and `2`, can mean completely different things depending on what you intend. And herein lies one of the eternal challenges of data formats: how do you communicate that intent?

JSON solves this with syntax. `42` is a number. `"42"` is a string. The quotes tell you. But this creates noise. If all your strings need quotes, you're paying for those quotes in every single value.

HEDL takes a different approach. It reads your values and figures out what they probably are. A bare `42` becomes a number. If you need the string "42", you quote it: `"42"`. But most strings don't need quotes because most strings don't look like numbers.

This page will teach you how HEDL's type inference works. You'll learn the rules, see the edge cases, understand when quotes matter, and gain the confidence to write HEDL that means exactly what you intend.

---

## The Philosophy: Smart Defaults, Explicit Overrides

HEDL's type system follows a simple philosophy: **do the obvious thing by default, and let the author override when needed.**

What does "obvious" mean? Consider these values:

```
Alice                 → Clearly a string. What else could it be?
42                    → Clearly a number. It's made of digits.
true                  → Clearly a boolean. It's a reserved keyword.
~                     → Clearly null. We declared that in the header.
alice@example.com     → Clearly a string. Has an @ but isn't a reference format.
@alice                → Clearly a reference. The @ prefix with an id.
[1,2,3]               → Clearly a tensor. Square brackets with numbers.
3.14159               → Clearly a float. Has a decimal point.
```

None of these need quotes. HEDL looks at the characters and understands what you meant. This is type inference.

But sometimes the obvious interpretation is wrong. What if `42` is a product code, not a quantity? What if `true` is a username, not a boolean? In those cases, you use quotes to override the inference:

```
"42"                  → String, not number
"true"                → String, not boolean
"hello, world"        → String with a comma (quotes needed to avoid confusion)
```

The system is lenient where it can be and strict where it must be. Let's see exactly how it works.

---

## Strings: The Default Destination

If HEDL can't figure out what something is, it assumes it's a string. This makes sense because strings are the most general type. Any data can be represented as a string.

### Bare Strings

Most strings need no quotes at all:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
user:
 name:Alice Chen
 city:San Francisco
 country:United States
 job_title:Senior Software Engineer
 department:Platform Engineering
```

These values contain spaces, letters, numbers, and periods. None of them look like numbers or booleans. None contain characters that would confuse the parser. HEDL reads them as strings without any ceremony.

Look at how clean that is compared to JSON:

```json
{
  "user": {
    "name": "Alice Chen",
    "city": "San Francisco",
    "country": "United States",
    "job_title": "Senior Software Engineer",
    "department": "Platform Engineering"
  }
}
```

Every string value in JSON needs quotes. That's 10 quote characters just in this tiny example. At scale, those characters add up.

### When Quotes Become Necessary

You need quotes when your string contains characters that would confuse the parser:

**Commas** are field separators in matrix lists:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Book:[isbn,title,author]
---
books:@Book
 |978-0-14-028329-7,"Tom Sawyer, The Adventures of",Mark Twain
```

Without quotes, that comma after "Sawyer" would make the parser think "The Adventures of" is a separate field.

**Colons** are key-value separators:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
timestamp:"2024-01-15T10:30:00Z"
note:"Important: read this carefully"
```

If you wrote `timestamp: 2024-01-15T10:30:00Z` without quotes, it would work because the parser is smart about ISO timestamps. But `note: Important: read this carefully` would fail because that second colon creates ambiguity.

**The quote character itself** needs escaping. Double it:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
dialogue:"She said ""yes"" and smiled"
```

Inside the quoted string, `""` becomes a single literal quote character.

**Leading or trailing whitespace** you want to preserve:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
indented_text:"   This has leading spaces"
padded:"centered   "
```

Bare strings have their whitespace trimmed. Quoted strings preserve it exactly.

### The Quotes-or-No-Quotes Decision Tree

When writing a string value, ask yourself:

1. Does it contain a comma? → Quote it.
2. Does it contain a colon? → Probably quote it (unless it's a format like timestamps that the parser handles).
3. Does it contain the quote character? → Quote it and double the internal quotes.
4. Does it have leading/trailing spaces you need? → Quote it.
5. Does it look exactly like a number, boolean, or null? → Quote it if you want a string.
6. None of the above? → Leave it bare.

In practice, you'll develop an intuition. Most strings are bare. You'll quote when you need to.

---

## Numbers: Integers and Floats

Numbers are straightforward. If a value looks like a number, it becomes a number.

### Integers

Whole numbers with optional leading minus sign:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
counts:
 users:1542
 orders:8934
 products:256
temperatures:
 high:32
 low:-5
 current:0
```

The parser sees digits (optionally preceded by a minus) and infers an integer.

Integer range is large: from -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807 (64-bit signed). You won't hit the limit in normal use.

### Floats

Numbers with decimal points:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
measurements:
 temperature:98.6
 weight_kg:72.5
 height_m:1.78
 balance:-125.50
```

The presence of a decimal point tells HEDL this is a float, not an integer.

You can also use scientific notation:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
scientific:
 avogadro:6.022e23
 planck:6.626e-34
 speed_of_light:2.998e8
```

The `e` notation works just like you'd expect. `6.022e23` means 6.022 × 10²³.

### Numbers That Aren't Numbers

Sometimes you have digits that shouldn't be treated as numbers:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
identifiers:
 product_code:"42"
 zip_code:"94102"
 phone:"555-1234"
 tracking_number:"1Z999AA10123456784"
```

Product codes, zip codes, phone numbers: these look numeric but have semantic meaning as strings. You might need to preserve leading zeros. You don't want to do arithmetic on them. Quote them.

The rule: **if you wouldn't add 1 to it, it's probably a string.**

---

## Booleans: True and False

HEDL has exactly two boolean values: `true` and `false`. These are keywords, not strings.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
feature_flags:
 dark_mode:true
 experimental_ui:false
 notifications:true
 analytics:false
```

The keywords are case-sensitive. `true` works. `True`, `TRUE`, `yes`, `on`: these are not booleans. They're strings.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
various_values:
 a:true                # Boolean
 b:"true"              # String (quoted)
 c:True                # String (wrong case)
 d:yes                 # String (not a boolean keyword)
 e:1                   # Number (not a boolean)
```

Why not support `yes`/`no` or `on`/`off` like YAML does? Because that creates ambiguity. Is `no` a boolean or a string? What about in Norwegian, where "no" might be a valid string value? By limiting booleans to exactly `true` and `false`, HEDL avoids an entire class of parsing surprises.

---

## Null: The Explicit Absence

When a value is missing, unknown, or not applicable, you use null. In HEDL, null is represented by the tilde character: `~`.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Employee:[id,name,manager,notes]
---
employees:@Employee
 |e1,Alice Chen,~,CEO of the company
 |e2,Bob Smith,@e1,~
 |e3,Carol Wu,@e1,On sabbatical
```

Alice has no manager (she's the CEO). Bob has no notes. The tilde makes this explicit.

Why tilde? Several reasons:

1. **It's a single character.** JSON's `null` is four characters. Four times the tokens.
2. **It's visually distinctive.** You won't confuse `~` with actual data.
3. **It's consistent.** Every null looks the same. No variation in spelling.

The `%NULL:~` header declaration makes this official. When the parser sees `~` in your data, it knows you mean null.

### Null vs. Empty String

These are different:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,bio]
---
users:@User
 |u1,Alice,~
 |u2,Bob,""
```

Alice has no bio (null). Bob has a bio that happens to be empty (empty string). The distinction matters in many applications. A null bio might mean "user hasn't filled this out yet." An empty bio might mean "user explicitly set their bio to empty."

---

## References: Typed Links

References are a type unique to HEDL. They link entities together with validation.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Author:[id,name]
%S:Book:[isbn,title,author]
---
authors:@Author
 |twain,Mark Twain
 |hemingway,Ernest Hemingway

books:@Book
 |978-0-14-028329-7,Tom Sawyer,@twain
 |978-0-7432-9737-9,Old Man and Sea,@hemingway
```

The value `@twain` is a reference. It points to the entity with id "twain". The parser validates that such an entity exists.

### Reference Validation

References are validated at parse time. The parser checks that the referenced entity exists:

```
Error at line 15: Unresolved reference
  @tolkien does not exist
  Available ids: twain, hemingway
```

This catches typos and broken links before your data reaches production. You'll never silently have a book pointing to an author that doesn't exist.

### References vs. Strings

A reference is not a string. They're different types:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Book:[isbn,title,author]
---
books:@Book
 |978-0-14-028329-7,Tom Sawyer,@twain
 |978-0-7432-9737-9,Old Man and Sea,"Mark Twain"
```

The first book's author is a reference (validated, linked). The second book's author is a string (just text, no validation).

Both might convert to similar JSON, but they have different semantics in HEDL.

We'll explore references deeply in the [References concept](references.md).

---

## Tensors and Lists: Collections of Values

HEDL has two ways to represent collections within a value: tensors and lists.

### Tensors: Numeric Arrays

Tensors use square brackets and contain only numbers:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
location:
 coordinates:[37.7749,-122.4194]
color:
 rgb:[255,128,64]
transformation:
 matrix:[[1,0,0],[0,1,0],[0,0,1]]
```

Why call them "tensors" instead of "arrays"? Because tensors have a specific meaning in scientific computing: multi-dimensional numeric arrays. HEDL uses this term to signal that these are optimized for numeric data. Machine learning features, coordinates, matrices, color values: these are tensor use cases.

Tensors can be nested for multi-dimensional data:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
neural_network:
 weights:[[0.1,0.2,0.3],[0.4,0.5,0.6],[0.7,0.8,0.9]]
 biases:[0.01,0.02,0.03]
```

### Lists: Mixed-Type Collections

Lists use round parentheses and can contain any types:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
article:
 tags:(python,machine-learning,tutorial)
product:
 features:(waterproof,lightweight,durable,eco-friendly)
mixed_data:
 values:(hello,42,true,@alice)
```

Lists are general-purpose. They can hold strings, numbers, booleans, references, even nested structures. Use them when your collection isn't purely numeric.

### Choosing Between Tensors and Lists

The rule is simple:

- **All numbers?** Use a tensor: `[1,2,3]`
- **Anything else?** Use a list: `(a,b,c)`

Tensors can be processed more efficiently because parsers know they're all numbers. Lists are more flexible but can't benefit from numeric optimizations.

---

## Type Inference in Matrix Lists

In matrix lists, type inference happens column by column. The parser looks at all values in a column and infers a consistent type.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Product:[sku,name,price,in_stock]
---
products:@Product
 |SKU-001,Laptop,999.99,true
 |SKU-002,Mouse,29.99,false
 |SKU-003,Keyboard,79.99,true
```

Column analysis:

- `sku`: All values are alphanumeric strings. Type: string.
- `name`: All values are word strings. Type: string.
- `price`: All values have decimal points. Type: float.
- `in_stock`: All values are `true` or `false`. Type: boolean.

### Type Consistency

For clean data, keep types consistent within columns. Don't mix:

```hedl
# AVOID: Mixed types in price column
products:@Product
 |SKU-001,Laptop,999.99,true
 |SKU-002,Mouse,"free",false     # String in a numeric column
```

This works (HEDL won't reject it), but it's confusing. If price is sometimes a number and sometimes a string, consuming code must handle both cases.

### Nulls Don't Break Consistency

Null values are allowed in any column without breaking type inference:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Employee:[id,name,salary,manager]
---
employees:@Employee
 |e1,Alice Chen,150000,~
 |e2,Bob Smith,120000,@e1
 |e3,Carol Wu,~,@e1
```

Alice has no manager (null). Carol has no salary listed (null). The column types remain clear: salary is numeric (with possible nulls), manager is a reference (with possible nulls).

---

## Edge Cases and Gotchas

Every type system has edge cases. Here are HEDL's:

### Numbers With Leading Zeros

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
values:
 a:007            # Number: 7 (leading zeros dropped)
 b:"007"          # String: "007" (preserved)
```

If you need to preserve leading zeros (like for codes or IDs), quote the value.

### Numeric Strings

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
values:
 age:25           # Number
 building:"25"    # String (it's a building number, not a quantity)
```

Ask yourself: would I do arithmetic with this value? If not, consider quoting it.

### Email Addresses

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
contact:
 email:alice@example.com    # String (not a reference, despite the @)
 manager:@bob               # Reference (starts with @ and then id)
```

Email addresses contain `@` but have additional characters around it. The parser correctly infers them as strings.

### Timestamps

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
event:
 timestamp:2024-01-15T10:30:00Z    # String (ISO 8601 format)
 unix_time:1705316400              # Number (seconds since epoch)
```

ISO timestamps are parsed as strings. Unix timestamps (just a number) are parsed as numbers. Choose whichever representation fits your use case.

### Negative Numbers

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
temperatures:
 high:32
 low:-15
 delta:-47
```

The minus sign works as expected. `-15` is a negative number.

### Very Large Numbers

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
astronomy:
 distance_km:149597870700          # Works: 64-bit integer
 atoms:602214076000000000000000    # Problem: exceeds 64-bit integer
```

If you have numbers larger than 9,223,372,036,854,775,807, use strings or scientific notation:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
astronomy:
 atoms:6.02214076e23              # Scientific notation: works
 huge:"602214076000000000000000"  # String: preserves precision
```

---

## Type Coercion on Export

When you convert HEDL to other formats, types map predictably.

### To JSON

HEDL types map directly to JSON types:

| HEDL Type | JSON Type |
|-----------|-----------|
| String | String |
| Integer | Number |
| Float | Number |
| Boolean | Boolean |
| Null | null |
| Tensor | Array |
| List | Array |
| Reference | String |
| Object | Object |

References serialize as strings like `"@alice"`. You can configure the converter to expand them inline if needed.

### To YAML

Similar mapping:

| HEDL Type | YAML Type |
|-----------|-----------|
| String | String |
| Integer | Integer |
| Float | Float |
| Boolean | Boolean |
| Null | null |
| Tensor | Sequence |
| List | Sequence |
| Reference | String |
| Object | Mapping |

### To CSV

CSV doesn't have types, so everything becomes strings:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,active,score]
---
users:@User
 |u1,Alice,true,95.5
 |u2,Bob,false,88.0
```

Exports as:

```csv
id,name,active,score
u1,Alice,true,95.5
u2,Bob,false,88.0
```

All values are text in CSV. The receiving system must parse them back to types if needed.

---

## Preserving Type Information

When you export HEDL to JSON and want to preserve type information, use the `--metadata` flag:

```bash
hedl to-json data.hedl --metadata -o typed.json
```

This adds HEDL-specific metadata to the JSON:

```json
{
  "_hedl": {
    "version": "2.0",
    "schemas": {
      "User": ["id", "name", "email"]
    }
  },
  "users": [
    {"id": "u1", "name": "Alice", "email": "alice@example.com"},
    {"id": "u2", "name": "Bob", "email": "bob@example.com"}
  ]
}
```

This metadata lets you round-trip back to HEDL without losing structure. The schemas are preserved, so `hedl from-json typed.json` can reconstruct the original HEDL document.

---

## Design Principles Recap

HEDL's type system follows clear principles:

**Inference over annotation.** The parser figures out types from syntax. You don't have to declare them.

**Explicit over implicit for ambiguity.** When the obvious interpretation might be wrong, use quotes to be explicit.

**Validation over assumption.** References are checked. Column counts are checked. Errors surface at parse time.

**Consistency over flexibility.** Booleans are exactly `true` and `false`. Null is exactly `~`. No YAML-style "yes/no/on/off" ambiguity.

**Efficiency over ceremony.** Bare strings save quotes. Single-character null saves tokens. Every design choice considers the cost of characters.

---

## What You've Learned

You now understand how HEDL interprets your values:

**Strings** are the default. Bare strings need no quotes. Use quotes for special characters, for values that look like other types, and for preserving whitespace.

**Numbers** are inferred from digits. Integers have no decimal point. Floats do. Scientific notation works.

**Booleans** are exactly `true` and `false`. Nothing else.

**Null** is `~`. One character. Visually distinctive.

**References** are `@id`. They're validated at parse time.

**Tensors** are `[1,2,3]`. Numeric arrays only.

**Lists** are `(a,b,c)`. Any types allowed.

**Consistency** matters in matrix list columns. Keep types consistent per column.

**Coercion** on export is predictable. HEDL types map cleanly to JSON, YAML, and other formats.

---

## Where to Go Next

The type system gives you values. The next concept gives you relationships:

**[References](references.md)** teaches you how to link entities together. You'll learn the full syntax, validation rules, and patterns for building graph structures.

After that, **[Canonicalization](canonicalization.md)** shows you how HEDL produces consistent, deterministic output. Every document has exactly one canonical form.

Or return to the [Concepts overview](README.md) to see the big picture.

Your values have types. Your types have meaning. Now let's connect them.
