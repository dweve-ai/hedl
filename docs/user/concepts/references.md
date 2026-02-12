# References: Teaching Your Data to Point

Imagine you're building a social network. Alice follows Bob. Bob follows Carol. Carol follows Alice. A triangle of connections.

Now imagine representing this in JSON:

```json
{
  "users": [
    {"id": "alice", "name": "Alice", "follows": ["bob"]},
    {"id": "bob", "name": "Bob", "follows": ["carol"]},
    {"id": "carol", "name": "Carol", "follows": ["alice"]}
  ]
}
```

Notice the problem? The `follows` field contains strings. Just plain strings. The JSON doesn't know that "bob" is supposed to point to the user with id "bob". It's just text. If you typo it as "bbo", JSON doesn't care. It happily stores your broken data and passes it along until something crashes in production.

Now imagine the same data in HEDL:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Follow:[follower,following]
---
users:@User
 |alice,Alice
 |bob,Bob
 |carol,Carol

follows:@Follow
 |@alice,@bob
 |@bob,@carol
 |@carol,@alice
```

See those `@alice` values? Those are **references**. They're not strings. They're validated pointers to specific entities. The HEDL parser validates them. If you write `@bbo`, the parser says "Error: no entity with id 'bbo' exists" and refuses to proceed.

This is the power of references. They transform your data from a loose bag of strings into a web of validated connections. This page will teach you everything about them.

---

## What Is a Reference?

A reference is a value that points to another entity in your document. It has a simple syntax:

```
@identifier
```

The `@` symbol marks it as a reference. The `identifier` tells you which specific entity is being referenced.

Let's break down `@alice`:

- `@` says "this is a reference"
- `alice` says "point to the entity with id 'alice'"

Together: "this value points to the entity whose id is 'alice'."

---

## Why References Use IDs

References point to entities by their unique identifier. This keeps things simple: if you know the id, you can reference it.

Consider this scenario:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Product:[sku,name]
%S:Order:[id,item]
---
users:@User
 |u001,Pro User Package

products:@Product
 |prod-001,Professional License

orders:@Order
 |order-001,@prod-001
```

The order references `@prod-001`, which clearly points to the product (not the user, which has id `u001`).

**Unique IDs give you:**

1. **Clarity.** When you read `@prod-001`, the naming convention tells you it's a product.

2. **Safety.** The parser validates that an entity with id `prod-001` exists.

3. **Simplicity.** One syntax for all references. No need to think about types.

4. **Flexibility.** You choose meaningful IDs that work for your domain.

---

## Declaring Entities That Can Be Referenced

For references to work, you need entities that can be referenced. This means entities with identifiers.

In matrix lists, the first column is always the identifier:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Author:[id,name,email]
---
authors:@Author
 |twain,Mark Twain,twain@example.com
 |hemingway,Ernest Hemingway,hemingway@example.com
 |fitzgerald,F. Scott Fitzgerald,fitzgerald@example.com
```

The `id` column provides the identifiers. Now other entities can reference these authors:

```hedl
%S:Book:[isbn,title,author]
---
books:@Book
 |978-0-14-028329-7,Tom Sawyer,@twain
 |978-0-7432-9737-9,Old Man and Sea,@hemingway
 |978-0-7432-7356-5,Great Gatsby,@fitzgerald
```

Each book's `author` field is a reference pointing to one of the defined authors.

---

## Forward References: Order Doesn't Matter

Here's something liberating: you can reference entities before they're defined.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Post:[id,title,author]
%S:Author:[id,name]
---
posts:@Post
 |post-001,My First Post,@alice
 |post-002,Another Post,@bob

authors:@Author
 |alice,Alice Chen
 |bob,Bob Smith
```

The posts appear before the authors in the document. The references `@alice` and `@bob` point to entities that haven't been defined yet. That's fine.

HEDL validation happens after the entire document is parsed. The parser reads everything first, builds the complete picture, and then checks that all references resolve. It doesn't matter what order things appear in.

This flexibility is important for real documents. You shouldn't have to carefully order your entities to satisfy reference dependencies. Put things in whatever order makes semantic sense for your domain.

---

## Reference Validation: Your Safety Net

Let's talk about what happens when references go wrong.

### Missing Entity

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Book:[isbn,title,author]
%S:Author:[id,name]
---
authors:@Author
 |twain,Mark Twain

books:@Book
 |978-0-14-028329-7,Tom Sawyer,@twain
 |978-0-7432-9737-9,Old Man and Sea,@hemingway
```

Run validation:

```
Error at line 12: Unresolved reference
  @hemingway does not exist
  Referenced from: books[1].author
  Available ids: twain
```

The parser caught the problem. Hemingway isn't defined. You get the line number, the reference that failed, and a helpful list of what IDs actually exist.

### Typos

```hedl
books:@Book
 |978-0-14-028329-7,Tom Sawyer,@twian
```

Typo: "twian" instead of "twain".

```
Error at line 10: Unresolved reference
  @twian does not exist
  Did you mean @twain?
```

The parser catches typos and even suggests corrections.

### Semantic Validation

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Author:[id,name]
%S:Publisher:[id,name]
%S:Book:[isbn,title,author]
---
authors:@Author
 |twain,Mark Twain

publishers:@Publisher
 |penguin,Penguin Books

books:@Book
 |978-0-14-028329-7,Tom Sawyer,@penguin
```

Wait. Books should have authors, not publishers. But `penguin` exists and the reference is syntactically valid. What happens?

HEDL validates that the referenced entity exists. The reference `@penguin` resolves successfully. Whether a book's author should point to an Author (not a Publisher) is a semantic constraint that depends on your application logic.

The point is: references give you existence validation. Your application can add additional semantic checks as needed.

---

## Building Relationships

References shine when modeling relationships. Let's explore the common patterns.

### One-to-Many: Authors and Books

An author can write many books. Each book has one author.

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Author:[id,name,nationality]
%S:Book:[isbn,title,author,year]
---
authors:@Author
 |twain,Mark Twain,American
 |dickens,Charles Dickens,British
 |dostoevsky,Fyodor Dostoevsky,Russian

books:@Book
 |978-0-14-028329-7,Tom Sawyer,@twain,1876
 |978-0-14-028336-5,Huckleberry Finn,@twain,1884
 |978-0-14-143960-0,Great Expectations,@dickens,1861
 |978-0-14-028330-3,Oliver Twist,@dickens,1838
 |978-0-14-044913-6,Crime and Punishment,@dostoevsky,1866
 |978-0-14-028331-0,The Idiot,@dostoevsky,1869
```

Twain wrote two books. Dickens wrote two books. Dostoevsky wrote two books. The `author` field in each book points back to the relevant author. The relationship is explicit and validated.

### Many-to-Many: Posts and Tags

A post can have many tags. A tag can apply to many posts. This requires a junction entity:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Post:[id,title,content]
%S:Tag:[id,name,description]
%S:PostTag:[id,post,tag]
---
posts:@Post
 |p1,Getting Started with Rust,Learn Rust basics...
 |p2,Web APIs in Python,Build APIs with Flask...
 |p3,Machine Learning 101,Introduction to ML...

tags:@Tag
 |rust,Rust,Systems programming language
 |python,Python,General-purpose programming language
 |web,Web Development,Building for the web
 |ml,Machine Learning,AI and data science
 |beginner,Beginner-Friendly,Good for newcomers

post_tags:@PostTag
 |pt1,@p1,@rust
 |pt2,@p1,@beginner
 |pt3,@p2,@python
 |pt4,@p2,@web
 |pt5,@p3,@python
 |pt6,@p3,@ml
 |pt7,@p3,@beginner
```

Post p1 (Rust basics) has tags: rust, beginner.
Post p2 (Python APIs) has tags: python, web.
Post p3 (ML intro) has tags: python, ml, beginner.

The PostTag entity creates the many-to-many relationship. Each row links one post to one tag. A post appears in multiple rows if it has multiple tags.

### Self-References: Trees and Hierarchies

Entities can reference other entities of the same type. This enables hierarchical structures:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Employee:[id,name,title,manager]
---
employees:@Employee
 |ceo,Alice Chen,CEO,~
 |vp_eng,Bob Smith,VP Engineering,@ceo
 |vp_sales,Carol Wu,VP Sales,@ceo
 |lead_platform,Dave Johnson,Platform Lead,@vp_eng
 |lead_mobile,Eve Martinez,Mobile Lead,@vp_eng
 |eng_senior,Frank Brown,Senior Engineer,@lead_platform
 |eng_junior,Grace Lee,Junior Engineer,@lead_platform
```

The CEO has no manager (null). The VPs report to the CEO. The leads report to their VP. The engineers report to their lead. It's an org chart expressed as data.

The key insight: `@ceo` references another Employee. This creates the tree structure.

### Graphs: Social Networks

When relationships can form cycles, you have a graph:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Person:[id,name,location]
%S:Connection:[id,person_a,person_b,relationship]
---
people:@Person
 |alice,Alice,San Francisco
 |bob,Bob,New York
 |carol,Carol,London
 |dave,Dave,Tokyo

connections:@Connection
 |c1,@alice,@bob,friend
 |c2,@bob,@carol,colleague
 |c3,@carol,@dave,friend
 |c4,@dave,@alice,colleague
 |c5,@alice,@carol,acquaintance
```

Alice knows Bob, who knows Carol, who knows Dave, who knows Alice. Plus Alice knows Carol directly. It's a web of relationships, not a hierarchy. References make this natural to express.

---

## References in the Real World

Let's see references in action with realistic examples.

### E-Commerce: Orders, Customers, Products

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Customer:[id,name,email,tier]
%S:Product:[sku,name,price,category]
%S:Order:[id,customer,date,status]
%S:OrderItem:[id,order,product,quantity,unit_price]
---
customers:@Customer
 |c001,Alice Chen,alice@example.com,gold
 |c002,Bob Smith,bob@example.com,silver
 |c003,Carol Wu,carol@example.com,bronze

products:@Product
 |SKU-001,MacBook Pro,2499.00,Electronics
 |SKU-002,Magic Mouse,99.00,Accessories
 |SKU-003,USB-C Hub,49.00,Accessories
 |SKU-004,Monitor Stand,199.00,Furniture

orders:@Order
 |ord-001,@c001,2024-01-15,shipped
 |ord-002,@c001,2024-01-20,delivered
 |ord-003,@c002,2024-01-18,processing

order_items:@OrderItem
 |item-001,@ord-001,@SKU-001,1,2499.00
 |item-002,@ord-001,@SKU-002,2,99.00
 |item-003,@ord-002,@SKU-003,3,49.00
 |item-004,@ord-003,@SKU-004,1,199.00
 |item-005,@ord-003,@SKU-002,1,99.00
```

The relationships form a clear model:
- Orders belong to Customers (`@c001`)
- Order items belong to Orders (`@ord-001`)
- Order items reference Products (`@SKU-001`)

Every relationship is validated. You can't create an order item for a product that doesn't exist. You can't assign an order to a customer that doesn't exist.

### Knowledge Graph: Concepts and Relationships

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Concept:[id,name,type,description]
%S:Relation:[id,subject,predicate,object]
---
concepts:@Concept
 |ml,Machine Learning,field,Learning algorithms from data
 |dl,Deep Learning,subfield,Multi-layer neural networks
 |nn,Neural Networks,technique,Brain-inspired computing
 |transformer,Transformer,architecture,Attention-based model
 |bert,BERT,model,Bidirectional encoder representations
 |gpt,GPT,model,Generative pre-trained transformer

relations:@Relation
 |r1,@dl,is_subfield_of,@ml
 |r2,@nn,is_technique_in,@dl
 |r3,@transformer,is_type_of,@nn
 |r4,@bert,is_based_on,@transformer
 |r5,@gpt,is_based_on,@transformer
 |r6,@bert,is_part_of,@ml
 |r7,@gpt,is_part_of,@ml
```

This is a knowledge graph. Concepts are nodes. Relations are edges. The predicate describes the relationship type. BERT is based on Transformer. GPT is based on Transformer. Both are part of Machine Learning.

When you export this to a graph database like Neo4j, the references become actual graph relationships. Your data already has the structure; the database just needs to store it.

### Configuration: Services and Dependencies

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:Service:[id,name,port,replicas]
%S:Database:[id,name,host,port]
%S:Dependency:[id,service,depends_on,type]
---
databases:@Database
 |pg_main,PostgreSQL Main,db-main.internal,5432
 |redis_cache,Redis Cache,cache.internal,6379
 |mongo_logs,MongoDB Logs,logs.internal,27017

services:@Service
 |api_gateway,API Gateway,8080,3
 |user_service,User Service,8001,2
 |order_service,Order Service,8002,2
 |notification_service,Notification Service,8003,1

dependencies:@Dependency
 |d1,@api_gateway,@user_service,http
 |d2,@api_gateway,@order_service,http
 |d3,@user_service,@pg_main,postgres
 |d4,@user_service,@redis_cache,redis
 |d5,@order_service,@pg_main,postgres
 |d6,@order_service,@notification_service,amqp
 |d7,@notification_service,@mongo_logs,mongo
```

The API gateway depends on user service and order service. User service depends on PostgreSQL and Redis. Order service depends on PostgreSQL and notification service. Notification service depends on MongoDB.

This isn't just documentation. It's a validated dependency graph. If someone renames a service, references break immediately. No silent drift. No mystery outages from misconfigured dependencies.

---

## References When Exporting

What happens to references when you convert HEDL to other formats?

### To JSON

By default, references become strings:

```hedl
books:@Book
 |978-0-14-028329-7,Tom Sawyer,@twain
```

Becomes:

```json
{
  "books": [
    {
      "isbn": "978-0-14-028329-7",
      "title": "Tom Sawyer",
      "author": "@twain"
    }
  ]
}
```

The reference is preserved as a string. Your application can parse it and resolve it.

With `--expand-refs`, references can be expanded inline:

```json
{
  "books": [
    {
      "isbn": "978-0-14-028329-7",
      "title": "Tom Sawyer",
      "author": {
        "id": "twain",
        "name": "Mark Twain"
      }
    }
  ]
}
```

Now the author data is embedded directly. This is useful when you need denormalized JSON for APIs or caching.

### To Neo4j

Graph databases love references. Each reference becomes a relationship:

```cypher
CREATE (twain:Author {id: 'twain', name: 'Mark Twain'})
CREATE (book:Book {isbn: '978-0-14-028329-7', title: 'Tom Sawyer'})
CREATE (book)-[:AUTHORED_BY]->(twain)
```

The reference `@twain` becomes an edge in the graph. This is natural: HEDL references are conceptually graph edges already. Exporting to Neo4j just makes that explicit.

### To Parquet

References become string columns. The columnar format stores them efficiently, and you can query on reference values:

```sql
SELECT * FROM books WHERE author = '@twain'
```

---

## Best Practices for References

After working with references extensively, here are the patterns that work well:

### Use Meaningful Identifiers

```hedl
# Good: IDs have meaning
authors:@Author
 |twain,Mark Twain
 |hemingway,Ernest Hemingway

# Less good: IDs are arbitrary
authors:@Author
 |a1,Mark Twain
 |a2,Ernest Hemingway
```

When you write `@twain`, anyone reading the document knows who you mean. `@a1` requires looking up what a1 means.

Meaningful IDs are especially valuable when debugging. Seeing `@twain` in an error message is more helpful than `@a1`.

### Keep IDs Stable

Once an entity has an ID, don't change it. Other parts of your system may depend on it. References in other documents may point to it.

If you need to change display names or other properties, change those. Leave the ID alone:

```hedl
# Initial
authors:@Author
 |twain,Mark Twain

# After learning his full name: Change the name, keep the ID
authors:@Author
 |twain,Samuel Langhorne Clemens (Mark Twain)
```

All references `@twain` continue to work.

### Document Relationship Semantics

References tell you *that* things are related. Consider documenting *how* they're related:

```hedl
%V:2.0
%NULL:~
%QUOTE:"
%S:User:[id,name]
%S:Post:[id,title,author,editor]
---
users:@User
 |alice,Alice Chen
 |bob,Bob Smith

posts:@Post
 |p1,My Post,@alice,@bob
```

Is Bob the editor who edited Alice's post? Or the other way around? The column name `editor` documents it. But in complex domains, consider additional documentation or using relationship entities with explicit predicates.

### Validate Early, Validate Often

Run `hedl validate` as part of your workflow. Catch broken references before they reach production.

```bash
# In your CI pipeline
hedl validate data/*.hedl || exit 1
```

This single command checks all references across all files. A typo in any reference fails the build.

---

## Common Questions

**Can I reference entities in other files?**

Within a single document, all references must resolve. Cross-file references require merging documents first or using external tooling.

**Can references be optional?**

Yes, use null:

```hedl
%S:Employee:[id,name,manager]
---
employees:@Employee
 |ceo,Alice Chen,~
 |vp,Bob Smith,@ceo
```

The CEO has no manager (null reference). The VP has a manager.

**Can I have multiple references in one field?**

Use a list:

```hedl
%S:Document:[id,title,authors]
---
documents:@Document
 |d1,Research Paper,(@alice,@bob,@carol)
```

The `authors` field contains a list of references.

**What if two entity types have the same ID?**

IDs should be unique across your document. If you have both a User and a Role that need the identifier "admin", use prefixes:

```hedl
users:@User
 |user_admin,Admin User

roles:@Role
 |role_admin,Administrator Role

# These are distinct:
some_field:@user_admin
other_field:@role_admin
```

Clear naming conventions prevent ID collisions.

---

## What You've Learned

References are HEDL's mechanism for expressing relationships:

**Syntax** is `@identifier`. Simple and clean.

**Forward references** work. Define things in any order. Validation happens after parsing.

**Validation** catches broken references and typos. Errors surface at parse time.

**Relationships** of any kind can be modeled: one-to-many, many-to-many, hierarchies, graphs.

**Exporting** preserves references as strings by default. Graph databases convert them to edges. Options exist to expand them inline.

**Best practices** include using meaningful IDs, keeping IDs stable, and validating often.

---

## Where to Go Next

You understand how entities connect. The final concept is about consistency:

**[Canonicalization](canonicalization.md)** explains how HEDL produces deterministic output. Every document has exactly one canonical form. This matters for version control, caching, and comparing documents.

Or return to the [Concepts overview](README.md) to see all four concepts together.

Your data now has structure, types, and connections. Let's make it consistent.
