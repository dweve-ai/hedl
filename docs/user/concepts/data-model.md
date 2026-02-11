# The Data Model: How HEDL Thinks About Your Information

Close your eyes for a moment. Picture a spreadsheet. Rows and columns. Each column has a header. Each row has data. Simple, right?

Now picture a JSON document. Curly braces within curly braces. Arrays of objects. Objects containing arrays containing objects. Flexible, but chaotic. Every single piece of data carries its own label.

Now imagine you could have both. The efficiency of a spreadsheet's structure, where column names appear once. And the expressiveness of JSON's nesting, where data can contain data. That's what HEDL's data model gives you.

This page will take you deep into that model. By the end, you'll understand not just *what* HEDL can represent, but *why* it represents things the way it does. You'll see patterns you can apply to your own data. You'll understand when HEDL shines and when it doesn't. And you'll have the mental model you need to design clean, efficient HEDL documents.

Let's begin.

---

## The Anatomy of a HEDL Document

Every HEDL document tells a story in three acts. First, the setup: who we are and what rules we follow. Then, the structure: what kinds of things exist in this document. Finally, the content: the actual data itself.

Here's a complete document, annotated:

```hedl
%V:2.0                                    ┐
%NULL:~                                   ├── Act I: The Header
%QUOTE:"                                  │   (Rules and declarations)
%S:Author:[id,name,bio]                   │
%S:Book:[isbn,title,author,price]         ┘
---                                       ← The Curtain
authors:@Author                           ┐
 |twain,Mark Twain,American humorist      │
 |hemingway,Ernest Hemingway,Novelist     │
                                          ├── Act II & III: Structure + Content
books:@Book                               │   (Entities and their values)
 |978-0-14-028329-7,Tom Sawyer,@twain,12.99
 |978-0-7432-9737-9,Old Man and Sea,@hemingway,14.99
                                          ┘
```

Let's explore each act.

### Act I: The Header

The header appears before the `---` separator. It establishes the context for everything that follows.

**The Version Declaration** comes first. `%V:2.0` tells every parser, every tool, every system that reads this document exactly which dialect of HEDL they're dealing with. This matters because languages evolve. By declaring the version upfront, your document remains unambiguous even years from now. A parser from 2030 will know exactly how to interpret your 2024 document.

**The Null Symbol** is next. `%NULL:~` says "whenever you see a tilde in this document, it means 'no value here.'" Why a tilde? Because it's visually distinctive. You'll never confuse it with actual data. And it's a single character, saving tokens compared to JSON's four-character `null`.

**The Quote Character** follows. `%QUOTE:"` establishes that double quotes wrap strings that contain special characters. Need a comma in your string? Need a colon? Wrap it in quotes. Need a literal quote inside your string? Double it: `"She said ""hello"" loudly"`.

**Schema Declarations** complete the header. These are the game-changers. `%S:Author:[id,name,bio]` says "there's a type of thing called Author, and every Author has three fields: id, name, and bio." Notice what this accomplishes: you define the structure *once*. Every Author in your document follows this shape. The field names never repeat in the data itself.

Think about what this means for a document with ten thousand authors. In JSON, you'd write `"id":`, `"name":`, `"bio":` ten thousand times each. In HEDL, you write them exactly once, in the schema declaration. The rest is pure data.

### The Curtain

The `---` separator is simple but essential. It says "the rules are established; now the story begins." Everything above is metadata and structure declarations. Everything below is your actual content.

Why have a separator at all? Because it lets parsers be efficient. When a parser sees `---`, it knows the header is complete. It has all the schemas, all the configuration. Now it can process the data with that context already established.

### Acts II and III: Structure and Content

Below the separator, your data lives. In HEDL, data is organized into **entities**. An entity is a named collection of information. In our example, `authors` is an entity. So is `books`.

Each entity has a **type annotation**. When you write `authors:@Author`, you're saying "this entity contains things that follow the Author schema." The `@Author` reference links back to the `%S:Author:[id,name,bio]` declaration in the header.

The actual data appears as **inline children**, those lines starting with `|`. Each `|` introduces one item in the collection. The values are comma-separated, in the same order as the columns defined in the schema.

This is the heart of HEDL's efficiency. The schema says "Authors have id, name, bio." The data says "`twain, Mark Twain, American humorist`". The parser combines them and understands: this is an Author with id "twain", name "Mark Twain", and bio "American humorist". No keys repeated. No curly braces. Just data.

---

## Entities: The Building Blocks

Let's go deeper into entities. An entity is any named piece of data in your document. But entities come in different flavors, and understanding those flavors helps you design better documents.

### Matrix Lists: The Workhorse

The most powerful entity type is the **matrix list**. You've already seen examples. A matrix list is a collection of items that all share the same structure, presented in a compact tabular format.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Product:[sku,name,category,price,stock]
---
products:@Product
 |SKU-001,Mechanical Keyboard,Electronics,149.99,234
 |SKU-002,Ergonomic Mouse,Electronics,79.99,567
 |SKU-003,Monitor Stand,Furniture,89.99,123
 |SKU-004,USB-C Hub,Electronics,45.99,891
 |SKU-005,Desk Lamp,Lighting,35.99,456
```

Why "matrix"? Because you can think of it as a matrix in the mathematical sense: rows and columns. The columns are defined by the schema. The rows are your data.

Why is this so efficient? Let's do the math. Suppose you have 1,000 products. In JSON, each product object would include `"sku":`, `"name":`, `"category":`, `"price":`, `"stock":`. That's about 35 characters of field names per product, times 1,000 products: 35,000 characters just for keys.

In HEDL, the schema declaration `%S:Product:[sku,name,category,price,stock]` is about 40 characters. Total. For any number of products. Whether you have 10 products or 10 million, the field names appear exactly once.

The savings become dramatic at scale. For that 1,000-product example, HEDL's keys overhead is 45 characters versus JSON's 35,000. That's a 99.87% reduction in key overhead. Your actual data (the values) takes the same space in both formats. But HEDL eliminates almost all the structural noise.

### Objects: Single Structured Items

Not everything is a list. Sometimes you have a single structured item:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
server:
 host:api.example.com
 port:443
 timeout_ms:30000
 ssl:true

database:
 host:db.internal
 port:5432
 name:production
 pool_size:20
```

Here, `server` and `database` are entities containing objects. Each object has key-value pairs, indented under the entity name. This looks and feels like YAML, but with HEDL's explicit typing and validation rules.

When should you use objects versus matrix lists? The rule of thumb: if you have multiple items of the same type, use a matrix list. If you have a single structured item, use an object. In the example above, there's one server configuration and one database configuration. Objects make sense.

But if you had multiple server configurations:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Server:[name,host,port,timeout_ms,ssl]
---
servers:@Server
 |production,api.example.com,443,30000,true
 |staging,staging.example.com,443,30000,true
 |development,localhost,8080,5000,false
```

Now a matrix list is more appropriate. The structure is identical across items; only the values differ.

### Scalar Entities: Simple Values

Sometimes an entity is just a single value:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
app_name:My Application
version:2.5.3
debug_mode:false
max_connections:100
```

Each of these is a scalar entity: a name paired with a single value. No nesting, no structure, just key and value.

Scalar entities are the simplest building blocks. They're perfect for configuration values, metadata, and any piece of information that doesn't have internal structure.

---

## The Power of Nesting

Real data has depth. Organizations contain departments. Departments contain teams. Teams contain people. HEDL handles this with nested entities.

### Building Hierarchies

Consider a company structure:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Person:[id,name,role]
---
company:
 name:Acme Corporation
 founded:1985
 headquarters:
  city:San Francisco
  country:USA
  timezone:America/Los_Angeles
 engineering:
  head:@alice
  budget:5000000
  team:@Person
   |alice,Alice Chen,Director
   |bob,Bob Smith,Senior Engineer
   |carol,Carol Wu,Engineer
   |dave,Dave Johnson,Engineer
 sales:
  head:@eve
  budget:3000000
  team:@Person
   |eve,Eve Martinez,Director
   |frank,Frank Brown,Account Executive
```

Look at how naturally the hierarchy expresses itself. The company contains headquarters, engineering, and sales. Each department contains its own structure. The engineering department has a team, which is a matrix list of Person entities.

Notice the indentation: one space per level. This is HEDL's rule, and it's strict for a reason. Consistent indentation means every document looks the same. Diffs are meaningful. There's no debate about whether to use 2 spaces or 4 spaces or tabs. One space. Always.

### How Deep Can You Go?

HEDL supports nesting up to 50 levels deep by default. In practice, you'll rarely need more than 5 or 6. If your data is deeply nested, consider whether that's truly the right structure, or whether you could flatten some of it using references.

Here's an example that goes several levels deep:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
world:
 continents:
  europe:
   countries:
    germany:
     cities:
      berlin:
       districts:
        mitte:
         population:380000
         area_km2:39.47
```

This works, but it's getting unwieldy. At some point, you're better off defining schemas and using references to flatten the structure. We'll explore that in the [References concept](references.md).

### Mixing Nesting Styles

You can freely mix objects and matrix lists at any level:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:MenuItem:[id,name,price,vegetarian]
---
restaurant:
 name:The Hungry Coder
 cuisine:Fusion
 lunch:
  served:11:00-14:00
  items:@MenuItem
   |l1,Caesar Salad,12.99,true
   |l2,Grilled Salmon,18.99,false
   |l3,Veggie Burger,14.99,true
 dinner:
  served:17:00-22:00
  items:@MenuItem
   |d1,Filet Mignon,34.99,false
   |d2,Mushroom Risotto,22.99,true
   |d3,Lobster Tail,42.99,false
```

The restaurant object contains nested lunch and dinner objects. Each of those contains metadata (serving hours) and a matrix list of menu items. The structure is natural to read and efficient to process.

---

## Value Types: What Can Go in a Cell?

Now that you understand the structural containers (entities, matrix lists, objects), let's look at what actually goes inside them. HEDL has a small but complete set of value types.

### Strings

The most common value type. Strings can be written two ways:

**Bare strings** have no quotes and are the most efficient:

```hedl
name:Alice
city:San Francisco
message:Hello World
```

Bare strings work when your value doesn't contain special characters like commas, colons, or the quote character itself.

**Quoted strings** handle special characters:

```hedl
title:"Hello, World!"
description:"Contains: a colon"
dialogue:"She said ""yes"" immediately"
```

Inside quoted strings, double the quote character to include a literal quote.

When do you need quotes? When your string contains:
1. Commas (which would otherwise be interpreted as field separators)
2. Colons (which would otherwise be interpreted as key-value separators)
3. The quote character itself
4. Leading or trailing whitespace you want to preserve

The HEDL tooling will help you. If you forget quotes where they're needed, validation will catch it.

### Numbers

HEDL distinguishes integers from floats:

```hedl
count:42           # Integer
pi:3.14159         # Float
temperature:-17.5  # Negative float
population:7800000000  # Large integer
```

Type inference is automatic. If the number has a decimal point, it's a float. Otherwise, it's an integer.

Scientific notation works too:

```hedl
avogadro:6.022e23
planck:6.626e-34
```

### Booleans

Two keywords: `true` and `false`. Nothing else.

```hedl
is_active:true
is_deleted:false
```

These are keywords, not strings. Writing `"true"` gives you the string "true", not the boolean true. HEDL is explicit about this distinction.

### Null

The absence of a value is represented by the null symbol, which you declared in your header. Convention uses tilde:

```hedl
%NULL:~
---
user:
 name:Alice
 middle_name:~
 email:alice@example.com
```

Alice has no middle name. That's different from having an empty string as a middle name. Null means "this field has no value."

In matrix lists, null is particularly useful:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Employee:[id,name,manager,department]
---
employees:@Employee
 |e1,Alice Chen,~,Engineering
 |e2,Bob Smith,@e1,Engineering
 |e3,Carol Wu,@e1,Engineering
```

Alice has no manager (she's the CEO, perhaps). Bob and Carol report to Alice. The null value makes this clear.

### Tensors

For numerical arrays, HEDL has a special syntax using square brackets:

```hedl
coordinates:[37.7749,-122.4194]
rgb_color:[255,128,64]
matrix:[[1,2,3],[4,5,6],[7,8,9]]
```

Tensors are optimized for numeric data. They're not general-purpose arrays like JSON arrays. If you need an array of strings or mixed types, use a list (round parentheses) instead:

```hedl
tags:(python,machine-learning,data-science)
mixed:(hello,42,true)
```

Lists use round parentheses and can contain any value types. Tensors use square brackets and contain only numbers.

Why the distinction? Because numeric arrays are common in scientific and machine learning contexts, and HEDL can optimize their storage and processing when it knows they're purely numeric.

### References

References link entities together. They use the `@id` syntax:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Author:[id,name]
%S:Book:[isbn,title,author]
---
authors:@Author
 |tolkien,J.R.R. Tolkien
 |rowling,J.K. Rowling

books:@Book
 |978-0-618-00222-8,The Lord of the Rings,@tolkien
 |978-0-7475-3269-6,Harry Potter,@rowling
```

References are validated. The parser checks that `@tolkien` actually points to an entity with id "tolkien". If you typo it as `@tolken`, you get an error at parse time, not a runtime bug.

We'll explore references deeply in the [References concept](references.md).

---

## The First Column Is Special

In matrix lists, the first column has a special role: it's the **identifier**. This is how other entities reference items in the list.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |alice,Alice Chen,alice@example.com
 |bob,Bob Smith,bob@example.com
```

The id column (`alice`, `bob`) serves several purposes:

**Uniqueness.** Within a single entity, every id must be unique. You can't have two users with id "alice" in the same users list.

**Referenceability.** Other parts of the document can reference these items. `@alice` points to the first row. `@bob` points to the second.

**Semantic meaning.** The id often carries meaning. In this case, the id is a username. It could also be a database primary key, a product SKU, or any other identifier that makes sense in your domain.

What if your data doesn't have a natural identifier? You can use sequential numbers or generate IDs:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:LogEntry:[id,timestamp,level,message]
---
logs:@LogEntry
 |1,2024-01-15T10:30:00Z,INFO,Application started
 |2,2024-01-15T10:30:01Z,DEBUG,Loading configuration
 |3,2024-01-15T10:30:02Z,INFO,Configuration loaded
```

The ids (1, 2, 3) don't carry semantic meaning, but they ensure uniqueness and enable references if needed.

---

## Constraints and Validation

HEDL enforces rules. This is a feature, not a limitation. Catching errors at parse time is infinitely better than discovering them in production.

### Schema Matching

When you declare `users:@User` and the User schema has three columns, every row must have exactly three values:

```hedl
# This is INVALID
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |alice,Alice Chen,alice@example.com
 |bob,Bob Smith                       # ERROR: Missing email
```

The parser will reject this with a clear error message telling you exactly what's wrong and where.

### Unique Identifiers

IDs must be unique within their entity:

```hedl
# This is INVALID
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
---
users:@User
 |alice,Alice Chen
 |alice,Alice Smith    # ERROR: Duplicate id "alice"
```

The parser catches the duplicate and tells you.

### Reference Validation

References must point to things that exist:

```hedl
# This is INVALID
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Task:[id,title,assignee]
---
users:@User
 |alice,Alice Chen

tasks:@Task
 |t1,Write docs,@bob    # ERROR: No entity with id "bob"
```

The parser knows that `@bob` should point to an entity, but no such entity exists.

### Consistent Indentation

HEDL enforces one-space indentation:

```hedl
# This is INVALID
%V:2.0
%NULL:~
%QUOTE:"
---
config:
    host:localhost    # ERROR: Expected 1 space, found 4
```

This strictness eliminates a whole class of bugs related to inconsistent whitespace.

---

## Mapping to JSON

When you convert HEDL to JSON, the mapping is deterministic. Understanding this mapping helps you predict what your HEDL will become and ensures smooth integration with JSON-based systems.

### Matrix Lists Become Arrays of Objects

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name,email]
---
users:@User
 |alice,Alice Chen,alice@example.com
 |bob,Bob Smith,bob@example.com
```

Becomes:

```json
{
  "users": [
    {"id": "alice", "name": "Alice Chen", "email": "alice@example.com"},
    {"id": "bob", "name": "Bob Smith", "email": "bob@example.com"}
  ]
}
```

Each row becomes an object. The schema's column names become the object's keys. The conversion is lossless.

### Objects Stay Objects

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
config:
 host:localhost
 port:8080
 ssl:true
```

Becomes:

```json
{
  "config": {
    "host": "localhost",
    "port": 8080,
    "ssl": true
  }
}
```

Simple and direct.

### Nesting Preserves Structure

```hedl
%V:2.0
%NULL:~
%QUOTE:"
---
app:
 name:MyApp
 database:
  host:db.local
  port:5432
```

Becomes:

```json
{
  "app": {
    "name": "MyApp",
    "database": {
      "host": "db.local",
      "port": 5432
    }
  }
}
```

The hierarchy maps directly.

### References Have Options

By default, references serialize as strings:

```hedl
tasks:@Task
 |t1,Write docs,@alice
```

Becomes:

```json
{
  "tasks": [
    {"id": "t1", "title": "Write docs", "assignee": "@alice"}
  ]
}
```

The reference is preserved as a string. Your application can resolve it. Alternatively, you can configure the converter to expand references inline, embedding the referenced data directly. See the [Formats guide](../formats.md) for options.

---

## Designing Good Data Models

You now understand the mechanics. Let's talk about design. How do you structure your HEDL documents well?

### Favor Matrix Lists for Repetition

If you have ten items of the same type, use a matrix list. If you have a thousand, definitely use a matrix list. The token savings compound with scale.

**Instead of this:**

```hedl
user1:
 id:u1
 name:Alice
user2:
 id:u2
 name:Bob
user3:
 id:u3
 name:Carol
```

**Do this:**

```hedl
%S:User:[id,name]
---
users:@User
 |u1,Alice
 |u2,Bob
 |u3,Carol
```

### Use References for Relationships

When entities relate to each other, use references. Don't embed duplicated data.

**Instead of this:**

```hedl
orders:
 |o1,Alice Chen,alice@example.com,Laptop,999.99
 |o2,Alice Chen,alice@example.com,Mouse,29.99
 |o3,Bob Smith,bob@example.com,Keyboard,79.99
```

**Do this:**

```hedl
%S:User:[id,name,email]
%S:Order:[id,customer,product,amount]
---
users:@User
 |alice,Alice Chen,alice@example.com
 |bob,Bob Smith,bob@example.com

orders:@Order
 |o1,@alice,Laptop,999.99
 |o2,@alice,Mouse,29.99
 |o3,@bob,Keyboard,79.99
```

The user details appear once. The orders reference them.

### Keep Nesting Reasonable

Deep nesting (more than 4-5 levels) often signals that you should restructure. Consider using references to flatten the hierarchy.

### Choose Meaningful IDs

Your IDs should be stable and meaningful when possible. Usernames, SKUs, UUIDs: things that identify records consistently.

### Be Consistent

If you have Users and Products and Orders, give them all the same treatment. Use schemas for all of them. Use the same naming conventions. Consistency makes documents easier to read and maintain.

---

## The Mental Model

Here's how to think about HEDL's data model:

**Schemas are like database table definitions.** They say what columns exist. They're declared once and used everywhere.

**Matrix lists are like database tables.** Each row is a record. Each column is a field. The schema says what fields exist.

**Objects are like JSON objects.** Key-value pairs, nested as needed.

**Entities are like named things in your domain.** Users, products, orders, configurations. Each entity has a name and contains data.

**References are like foreign keys.** They link entities together. The parser validates them.

**The header is like a database schema file.** It declares all the structures. The body is like the data itself.

When you approach a new data modeling task in HEDL, ask yourself:

1. What are the entities in my domain?
2. Which entities have multiple instances of the same structure? (Those become matrix lists with schemas.)
3. Which entities are unique configurations or metadata? (Those become objects.)
4. How do entities relate to each other? (Those relationships become references.)

Answer those questions, and the document structure will emerge naturally.

---

## What You've Learned

This has been a deep dive. Let's recap the key points:

**Document structure** follows a clear pattern: header with version and schemas, separator, then body with entities and values.

**Schemas** define structure once. This is the core innovation that enables HEDL's token efficiency.

**Matrix lists** are the workhorse for collections of same-shaped items. They achieve 56%+ token savings compared to JSON.

**Objects** handle single structured items and configurations.

**Nesting** allows hierarchical data, with one-space-per-level indentation.

**Value types** include strings, numbers, booleans, null, tensors, lists, and references.

**The first column** in a matrix list is the identifier, used for uniqueness and references.

**Validation** catches errors at parse time: schema mismatches, duplicate IDs, broken references.

**JSON mapping** is deterministic and lossless.

---

## Where to Go Next

You understand the data model. Now deepen your knowledge:

**[Type System](type-system.md)** explores how HEDL infers and validates types. You'll learn when quotes are needed, how numbers work, and what happens with ambiguous values.

**[References](references.md)** goes deep on entity linking. You'll learn the full reference syntax, validation rules, and how to build graph structures.

**[Canonicalization](canonicalization.md)** explains deterministic formatting. You'll learn why every HEDL document can have one and only one canonical representation, and why that matters.

Or return to the [Concepts overview](README.md) to see how all four concepts fit together.

You have the foundation. Keep building.
