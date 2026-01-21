# Neo4j Integration Guide

Guide for using HEDL with Neo4j graph databases.

---

## Overview

The `hedl-neo4j` crate provides seamless bidirectional conversion between HEDL documents and Neo4j graph databases. HEDL's hierarchical structure maps naturally to Neo4j's property graph model, making it ideal for:

- **Knowledge graphs**: Entities as nodes, relationships preserved
- **Data migration**: Import/export data to/from Neo4j
- **Graph analytics**: Leverage Neo4j's graph algorithms
- **Multi-model data**: Combine document and graph representations

---

## Quick Start

### Installation

```toml
# Cargo.toml
[dependencies]
hedl-neo4j = "1.2"
```

### Basic Usage

```rust
use hedl_core::parse;
use hedl_neo4j::{to_cypher, ToCypherConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse HEDL document
    let hedl = r#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
%STRUCT: Post: [id, title, content]
%NEST: User > Post
---
users: @User
  | alice, Alice Smith, alice@example.com
    | post1, Hello World, My first post
    | post2, Rust Tips, Advanced Rust patterns
  | bob, Bob Jones, bob@example.com
    | post3, Neo4j Guide, Getting started with Neo4j
"#;

    let doc = parse(hedl.as_bytes())?;

    // Convert to Cypher
    let config = ToCypherConfig::default();
    let cypher = to_cypher(&doc, &config)?;

    println!("{}", cypher);
    Ok(())
}
```

**Output**:

```cypher
// HEDL → Neo4j: users
CREATE CONSTRAINT user_id_unique IF NOT EXISTS FOR (n:User) REQUIRE n.id IS UNIQUE;

UNWIND [
  {id: 'alice', name: 'Alice Smith', email: 'alice@example.com'},
  {id: 'bob', name: 'Bob Jones', email: 'bob@example.com'}
] AS row
MERGE (n:User {id: row.id})
SET n += row;

// HEDL → Neo4j: posts (nested under users)
CREATE CONSTRAINT post_id_unique IF NOT EXISTS FOR (n:Post) REQUIRE n.id IS UNIQUE;

UNWIND [
  {id: 'post1', title: 'Hello World', content: 'My first post', _parent: 'alice'},
  {id: 'post2', title: 'Rust Tips', content: 'Advanced Rust patterns', _parent: 'alice'},
  {id: 'post3', title: 'Neo4j Guide', content: 'Getting started with Neo4j', _parent: 'bob'}
] AS row
MERGE (n:Post {id: row.id})
SET n += row
WITH n, row
MATCH (parent:User {id: row._parent})
MERGE (parent)-[:HAS_POST]->(n);
```

---

## Core Concepts

### Mapping Rules

| HEDL Concept | Neo4j Equivalent | Example |
|--------------|------------------|---------|
| **%STRUCT: User** | Node label `:User` | `(n:User)` |
| **Row** | Node with properties | `{id: 'alice', name: 'Alice'}` |
| **ID column** | Unique node property | `{id: 'alice'}` |
| **Other columns** | Node properties | `{name: 'Alice', email: '...'}` |
| **%NEST: User > Post** | Relationship | `(User)-[:HAS_POST]->(Post)` |
| **Reference `@user1`** | Relationship | `(n)-[:REFERS_TO]->(m)` |

---

## Configuration

### ToCypherConfig

```rust
use hedl_neo4j::ToCypherConfig;

let config = ToCypherConfig::default()
    .with_merge()              // Use MERGE instead of CREATE
    .with_constraints()         // Generate unique constraints
    .with_batch_size(1000)      // Batch size for UNWIND
    .with_relationship_type("CONTAINS"); // Custom relationship type

// Or customize specific options
let config = ToCypherConfig {
    use_merge: true,
    create_constraints: true,
    batch_size: 500,
    relationship_type: Some("LINKED_TO".to_string()),
};
```

**Options**:

- **use_merge**: `MERGE` (upsert) vs `CREATE` (insert only)
  - Default: `true` (MERGE)
  - Use CREATE for faster bulk imports (must ensure no duplicates)

- **create_constraints**: Generate `CREATE CONSTRAINT` statements
  - Default: `true`
  - Ensures ID uniqueness, improves query performance

- **batch_size**: Number of nodes per UNWIND statement
  - Default: `1000`
  - Larger batches = faster, but more memory

- **relationship_type**: Custom relationship name for NEST
  - Default: Derived from child type (e.g., `HAS_POST`)
  - Override for custom relationship names

---

## Advanced Examples

### Example 1: Knowledge Graph

```rust
use hedl_core::parse;
use hedl_neo4j::{to_cypher_statements, ToCypherConfig};

let hedl = r#"
%VERSION: 1.0
%STRUCT: Person: [id, name, born]
%STRUCT: Movie: [id, title, released]
%STRUCT: ActedIn: [person_id, movie_id, role]
---
people: @Person
  | keanu, Keanu Reeves, 1964
  | carrie, Carrie-Anne Moss, 1967

movies: @Movie
  | matrix, The Matrix, 1999
  | reloaded, The Matrix Reloaded, 2003

roles: @ActedIn
  | keanu, matrix, Neo
  | carrie, matrix, Trinity
"#;

let doc = parse(hedl.as_bytes())?;
let config = ToCypherConfig::default();
let statements = to_cypher_statements(&doc, &config)?;

// Execute each statement
for stmt in statements {
    println!("Executing: {}", stmt.comment);
    // neo4j_session.run(stmt.query, stmt.parameters)?;
}
```

**Generated Cypher**:

```cypher
// Nodes
MERGE (n:Person {id: 'keanu'}) SET n.name = 'Keanu Reeves', n.born = 1964;
MERGE (n:Person {id: 'carrie'}) SET n.name = 'Carrie-Anne Moss', n.born = 1967;
MERGE (n:Movie {id: 'matrix'}) SET n.title = 'The Matrix', n.released = 1999;

// Relationships
MATCH (p:Person {id: 'keanu'}), (m:Movie {id: 'matrix'})
MERGE (p)-[r:ACTED_IN]->(m)
SET r.role = 'Neo';
```

### Example 2: Hierarchical Data

```rust
let hedl = r#"
%VERSION: 1.0
%STRUCT: Department: [id, name]
%STRUCT: Team: [id, name]
%STRUCT: Employee: [id, name, role]
%NEST: Department > Team
%NEST: Team > Employee
---
departments: @Department
  | eng, Engineering
    | backend, Backend Team
      | alice, Alice Smith, Senior Engineer
      | bob, Bob Jones, Engineer
    | frontend, Frontend Team
      | charlie, Charlie Brown, Tech Lead
"#;

let doc = parse(hedl.as_bytes())?;
let cypher = to_cypher(&doc, &ToCypherConfig::default())?;
```

**Result**: Multi-level relationships:
- `(Department)-[:HAS_TEAM]->(Team)`
- `(Team)-[:HAS_EMPLOYEE]->(Employee)`

### Example 3: Bidirectional Conversion

```rust
use hedl_neo4j::{to_cypher, from_cypher};

// HEDL → Neo4j
let hedl = parse(hedl_string.as_bytes())?;
let cypher = to_cypher(&hedl, &config)?;

// Execute in Neo4j
// neo4j_session.run(cypher)?;

// Neo4j → HEDL
let query = "MATCH (n:User)-[r:HAS_POST]->(p:Post) RETURN n, r, p";
let hedl_doc = from_cypher(&neo4j_result)?;
```

---

## Relationship Mapping

### NEST Relationships

NEST directives create parent-child relationships:

```hedl
%NEST: User > Post
```

**Becomes**:

```cypher
(User)-[:HAS_POST]->(Post)
```

**Relationship type**: Derived from child type
- `Post` → `:HAS_POST`
- `Comment` → `:HAS_COMMENT`
- `OrderItem` → `:HAS_ORDERITEM`

**Custom relationship type**:

```rust
let config = ToCypherConfig::default()
    .with_relationship_type("AUTHORED");

// Result: (User)-[:AUTHORED]->(Post)
```

### Reference Relationships

HEDL references create relationships:

```hedl
%STRUCT: Order: [id, user_ref, product_ref]
---
orders: @Order
  | order1, @User:alice, @Product:widget
```

**Becomes**:

```cypher
MATCH (o:Order {id: 'order1'})
MATCH (u:User {id: 'alice'})
MATCH (p:Product {id: 'widget'})
MERGE (o)-[:REFERS_TO]->(u)
MERGE (o)-[:REFERS_TO]->(p);
```

---

## Performance Optimization

### Bulk Import

For large datasets, optimize with:

```rust
let config = ToCypherConfig::default()
    .with_create()           // CREATE instead of MERGE (faster)
    .with_batch_size(5000)   // Larger batches
    .without_constraints();  // Skip constraint creation

// Then execute
let cypher = to_cypher(&doc, &config)?;
```

**Performance**:
- **MERGE**: ~10K nodes/sec (upsert)
- **CREATE**: ~50K nodes/sec (insert only, assumes no duplicates)

### Indexing Strategy

```cypher
// Before bulk import
CREATE INDEX user_email FOR (n:User) ON (n.email);
CREATE INDEX post_title FOR (n:Post) ON (n.title);

// Then import
// ... execute HEDL-generated Cypher ...

// After import, create constraints
CREATE CONSTRAINT user_id_unique FOR (n:User) REQUIRE n.id IS UNIQUE;
```

### Parallelization

```rust
use rayon::prelude::*;

let statements = to_cypher_statements(&doc, &config)?;

// Execute in parallel (requires connection pool)
statements.par_iter().for_each(|stmt| {
    // neo4j_pool.execute(&stmt.query);
});
```

---

## Graph Traversal

### Cypher Queries on HEDL Data

After importing, leverage Neo4j's graph capabilities:

```cypher
// Find all posts by user
MATCH (u:User {id: 'alice'})-[:HAS_POST]->(p:Post)
RETURN p.title, p.content;

// Find user's social network (2 hops)
MATCH (u:User {id: 'alice'})-[:FOLLOWS*1..2]->(friend)
RETURN DISTINCT friend.name;

// Shortest path between users
MATCH path = shortestPath(
  (a:User {id: 'alice'})-[:FOLLOWS*]-(b:User {id: 'bob'})
)
RETURN path;

// PageRank on users
CALL gds.pageRank.stream('user-graph')
YIELD nodeId, score
RETURN gds.util.asNode(nodeId).name AS user, score
ORDER BY score DESC;
```

---

## Error Handling

### Common Errors

1. **Duplicate IDs** (without MERGE):
   ```
   Error: Node(0) already exists with label `User` and property `id` = 'alice'
   ```
   **Solution**: Use `.with_merge()` or ensure unique IDs

2. **Missing parent**:
   ```
   Error: Expected to find parent node User:alice for child Post:post1
   ```
   **Solution**: Ensure parent nodes created before children

3. **Schema mismatch**:
   ```
   Error: Property `email` not found on node User
   ```
   **Solution**: Verify HEDL schema matches Neo4j expectations

### Validation

```rust
use hedl_neo4j::validate_before_import;

// Validate before importing
let issues = validate_before_import(&doc)?;
for issue in issues {
    eprintln!("Warning: {}", issue);
}

// Then proceed if OK
if issues.is_empty() {
    let cypher = to_cypher(&doc, &config)?;
}
```

---

## Best Practices

### 1. ID Strategy

**Good**:
```hedl
%STRUCT: User: [id, name, email]
---
users: @User
  | user_123, Alice, alice@example.com  # UUID or sequential ID
```

**Bad**:
```hedl
| alice, Alice, alice@example.com  # Name as ID (not unique)
```

### 2. NEST Hierarchy

**Good**: Clear parent-child semantics
```hedl
%NEST: Company > Department
%NEST: Department > Team
```

**Avoid**: Deep nesting (>5 levels) - flatten instead

### 3. Relationship Types

**Good**: Use semantic names
```rust
.with_relationship_type("MANAGES")  // (Manager)-[:MANAGES]->(Employee)
.with_relationship_type("BELONGS_TO")  // (Post)-[:BELONGS_TO]->(Category)
```

### 4. Batch Size

- **Small datasets** (<10K nodes): Default (1000)
- **Large datasets** (>100K nodes): 5000-10000
- **Very large** (>1M nodes): Use CREATE + 10000

---

## Integration Examples

### Example 1: Neo4j Driver (Rust)

```rust
use neo4rs::{Graph, query};
use hedl_neo4j::to_cypher_statements;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to Neo4j
    let graph = Graph::new("neo4j://localhost:7687", "neo4j", "password").await?;

    // Convert HEDL to Cypher
    let doc = parse(hedl_string.as_bytes())?;
    let statements = to_cypher_statements(&doc, &ToCypherConfig::default())?;

    // Execute each statement
    for stmt in statements {
        graph.run(query(&stmt.query)).await?;
    }

    println!("Import complete!");
    Ok(())
}
```

### Example 2: Python (via FFI)

```python
from hedl import parse, to_cypher
from neo4j import GraphDatabase

# Parse HEDL
with open('data.hedl') as f:
    hedl = f.read()

doc = parse(hedl)
cypher = to_cypher(doc)

# Execute in Neo4j
driver = GraphDatabase.driver("neo4j://localhost:7687", auth=("neo4j", "password"))
with driver.session() as session:
    session.run(cypher)

driver.close()
```

### Example 3: Command-Line

```bash
# Convert HEDL to Cypher
hedl to-cypher data.hedl > import.cypher

# Execute in Neo4j
cat import.cypher | cypher-shell -u neo4j -p password

# Or use Neo4j browser
# 1. Open http://localhost:7474
# 2. Paste generated Cypher
# 3. Run query
```

---

## Troubleshooting

### Issue: Slow imports

**Diagnosis**:
```cypher
// Check constraint status
SHOW CONSTRAINTS;

// Check index status
SHOW INDEXES;
```

**Solutions**:
1. Drop constraints before bulk import
2. Create indexes after import
3. Increase batch size
4. Use CREATE instead of MERGE

### Issue: Relationship not created

**Diagnosis**:
```cypher
// Check if parent exists
MATCH (n:User {id: 'alice'}) RETURN n;

// Check if NEST directive present
```

**Solutions**:
1. Verify %NEST directive in HEDL
2. Ensure parent created before children
3. Check relationship type in query

### Issue: Memory errors

**Diagnosis**: Neo4j heap size too small

**Solutions**:
1. Increase Neo4j heap: `dbms.memory.heap.max_size=4G`
2. Reduce batch size
3. Process in chunks

---

## Security Considerations

### Cypher Injection Prevention

The library **automatically escapes** all user data:

```rust
// Safe - automatically escaped
let hedl = r#"
users: @User
  | evil, '; DROP DATABASE; --, email
"#;

let cypher = to_cypher(&doc, &config)?;
// Output: {id: '\'; DROP DATABASE; --', ...}
// Quotes are escaped, preventing injection
```

**All strings are quoted and escaped** - no manual escaping needed.

### Access Control

```cypher
// Create read-only role
CREATE ROLE reader;
GRANT MATCH ON GRAPH * TO reader;

// Assign to user
CREATE USER analyst SET PASSWORD 'pass';
GRANT ROLE reader TO analyst;
```

---

## References

- **Neo4j Cypher Manual**: https://neo4j.com/docs/cypher-manual/current/
- **HEDL SPEC**: Section 18.3 (Neo4j Format)
- **hedl-neo4j API Docs**: https://docs.rs/hedl-neo4j
- **Neo4j Rust Driver**: https://github.com/neo4j-labs/neo4rs

---

**Questions?** Open an issue on GitHub or email support@dweve.com
