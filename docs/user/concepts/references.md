# Concept: References

Understanding HEDL's entity reference system.

## Overview

HEDL has **first-class support for references**, allowing entities to point to other entities using a typed reference syntax.

## Reference Syntax

### Basic Format

```
@TypeName:id
```

**Example:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author, title]
---
users: @User
  | u1, Alice
  | u2, Bob

posts: @Post
  | p1, @User:u1, First Post
  | p2, @User:u2, Second Post
```

**Parts:**
- `@` - Reference marker
- `TypeName` - Type of referenced entity
- `:` - Separator
- `id` - ID of specific entity

## Type-Scoped IDs

### Why Type Scoping?

Type-scoped IDs prevent ambiguity:

**Without type scoping (problematic):**
```
users: u1, u2
posts: p1, p2
comments: c1, c2

# Which entity does '1' refer to?
likes: 1, 2  # Ambiguous!
```

**With type scoping (clear):**
```hedl
%VERSION: 1.0
%STRUCT: Like: [id, target]
---
likes: @Like
  | l1, @Post:p1     # Clearly a post
  | l2, @User:u1     # Clearly a user
```

### Benefits

1. **Clarity** - Type is explicit
2. **Validation** - Check if target type/ID exists
3. **Documentation** - Self-documenting relationships
4. **Tooling** - IDEs can provide autocomplete

## Reference Resolution

### Forward References

References can appear before their target:

```hedl
%VERSION: 1.0
%STRUCT: Post: [id, author]
%STRUCT: User: [id, name]
---
posts: @Post
  | p1, @User:u1     # Reference defined first

users: @User
  | u1, Alice        # Target defined later
```

**Validation:** HEDL validates after parsing entire document.

### Cross-Entity References

Reference any entity from any other:

```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Product: [id, name]
%STRUCT: Order: [id, user, product]
---
users: @User
  | u1, Alice
  | u2, Bob

products: @Product
  | prod1, Laptop
  | prod2, Mouse

orders: @Order
  | o1, @User:u1, @Product:prod1
  | o2, @User:u2, @Product:prod2
```

## Use Cases

### One-to-Many Relationships

**Blog example:**
```hedl
%VERSION: 1.0
%STRUCT: Author: [id, name]
%STRUCT: Post: [id, author, title]
---
authors: @Author
  | a1, Alice
  | a2, Bob

posts: @Post
  | p1, @Author:a1, First Post
  | p2, @Author:a1, Second Post
  | p3, @Author:a2, Bob's Post
```

### Many-to-Many Relationships

**Tags example:**
```hedl
%VERSION: 1.0
%STRUCT: Post: [id, title]
%STRUCT: Tag: [id, name]
%STRUCT: PostTag: [id, post, tag]
---
posts: @Post
  | p1, Intro to Rust
  | p2, Web APIs

tags: @Tag
  | t1, rust
  | t2, programming
  | t3, web

post_tags: @PostTag
  | pt1, @Post:p1, @Tag:t1
  | pt2, @Post:p1, @Tag:t2
  | pt3, @Post:p2, @Tag:t2
  | pt4, @Post:p2, @Tag:t3
```

### Graph Structures

**Social network:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Follow: [id, follower, following]
---
users: @User
  | u1, Alice
  | u2, Bob
  | u3, Carol

follows: @Follow
  | f1, @User:u1, @User:u2
  | f2, @User:u2, @User:u3
  | f3, @User:u3, @User:u1
```

## Validation

### Reference Integrity

HEDL validates that references point to existing entities:

**Valid:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | u1, Alice

posts: @Post
  | p1, @User:u1     # u1 exists
```

**Invalid:**
```hedl
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: Post: [id, author]
---
users: @User
  | u1, Alice

posts: @Post
  | p1, @User:u99    # u99 doesn't exist
```

### Type Checking

References must match declared types:

**Valid:**
```hedl
%VERSION: 1.0
%STRUCT: Post: [id, author]
---
posts: @Post
  | p1, @User:u1     # Expects User type
```

**Invalid:**
```hedl
%VERSION: 1.0
%STRUCT: Post: [id, author]
---
posts: @Post
  | p1, @Product:prod1   # Wrong type
```

## JSON Representation

### Without Metadata

Plain references:

**HEDL:**
```hedl
%VERSION: 1.0
%STRUCT: Post: [id, author, title]
---
posts: @Post
  | p1, @User:u1, Title
```

**JSON:**
```json
{
  "posts": [
    {"id": "p1", "author": {"@ref": "User:u1"}, "title": "Title"}
  ]
}
```

### With Metadata

Enhanced references:

```bash
hedl to-json data.hedl --metadata -o output.json
```

**JSON:**
```json
{
  "_hedl_references": {
    "posts.0.author": {"type": "User", "id": "u1"}
  },
  "posts": [...]
}
```

### Graph Database Import

References map naturally to graph relationships (edges).

**Using Rust Library:**
```rust
use hedl_neo4j::to_cypher;
// ...
let cypher = to_cypher(&doc, &Default::default())?;
// Output: CREATE (:User {id: 'u1'}), (:User {id: 'u2'}), (:User {id: 'u3'})...
```

---

**Related:**
- [Data Model](data-model.md)
- [Type System](type-system.md)
