# hedl-neo4j

**Bidirectional HEDL ↔ Neo4j integration—export structured data to graph databases and import query results back to HEDL with full type preservation.**

Graph databases excel at relationship queries. Neo4j powers knowledge graphs, fraud detection, recommendation engines. But loading structured data from HEDL into Neo4j shouldn't require custom ETL scripts. Querying Neo4j and converting results back to HEDL for further processing shouldn't lose type information or structural semantics.

`hedl-neo4j` provides bidirectional integration between HEDL and Neo4j. Export HEDL documents as Cypher CREATE/MERGE statements with automatic relationship detection from references. Generate uniqueness constraints for entity IDs. Batch large imports with UNWIND for optimal performance. Import Neo4j query results back to HEDL with full schema preservation. Stream large exports without loading entire documents into memory.

## What's Implemented

Comprehensive Neo4j integration with security and performance:

1. **Cypher Generation**: CREATE and MERGE strategies for nodes and relationships
2. **Automatic Relationship Detection**: Reference fields (`@Type:id`) become Neo4j relationships
3. **NEST Pattern Support**: Parent-child relationships become HAS_* edges (e.g., HAS_ITEMS)
4. **Constraint Generation**: Uniqueness constraints on entity IDs
5. **Batch Processing**: UNWIND-based batching (default 1000 nodes per statement)
6. **Streaming API**: Process large documents without full memory buffering
7. **Bidirectional Conversion**: Neo4j records → HEDL documents with schema preservation
8. **Security Hardening**: Unicode normalization, zero-width filtering, 100 MB string limits
9. **Property Mapping**: All HEDL types → Neo4j properties (Int, Float, String, Boolean, null)
10. **Configuration**: Strategy (CREATE/MERGE), batch size, constraint generation

## Installation

```toml
[dependencies]
hedl-neo4j = "1.0"
```

## Basic Usage

### HEDL → Cypher

Export HEDL document as Cypher statements:

```rust
use hedl_core::parse;
use hedl_neo4j::to_cypher;

let doc = parse(br#"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
%STRUCT: Post: [id, author, title, content]
%NEST: User: Post
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com
  posts: @Post
    | post1, @User:alice, Hello World, My first post
    | post2, @User:alice, Second Post, Another post
    | post3, @User:bob, Bob's Thoughts, Thinking...
"#)?;

let cypher = to_cypher(&doc)?;
println!("{}", cypher);
```

**Generated Cypher**:
```cypher
// Uniqueness constraints
CREATE CONSTRAINT user_id IF NOT EXISTS FOR (n:User) REQUIRE n.id IS UNIQUE;
CREATE CONSTRAINT post_id IF NOT EXISTS FOR (n:Post) REQUIRE n.id IS UNIQUE;

// Create nodes
CREATE (alice:User {id: "alice", name: "Alice Smith", email: "alice@example.com"});
CREATE (bob:User {id: "bob", name: "Bob Jones", email: "bob@example.com"});
CREATE (post1:Post {id: "post1", title: "Hello World", content: "My first post"});
CREATE (post2:Post {id: "post2", title: "Second Post", content: "Another post"});
CREATE (post3:Post {id: "post3", title: "Bob's Thoughts", content: "Thinking..."});

// Create relationships (from references)
MATCH (post1:Post {id: "post1"}), (alice:User {id: "alice"}) CREATE (post1)-[:AUTHOR]->(alice);
MATCH (post2:Post {id: "post2"}), (alice:User {id: "alice"}) CREATE (post2)-[:AUTHOR]->(alice);
MATCH (post3:Post {id: "post3"}), (bob:User {id: "bob"}) CREATE (post3)-[:AUTHOR]->(bob);

// Create relationships (from NEST pattern)
MATCH (alice:User {id: "alice"}), (post1:Post {id: "post1"}) CREATE (alice)-[:HAS_POST]->(post1);
MATCH (alice:User {id: "alice"}), (post2:Post {id: "post2"}) CREATE (alice)-[:HAS_POST]->(post2);
MATCH (bob:User {id: "bob"}), (post3:Post {id: "post3"}) CREATE (bob)-[:HAS_POST]->(post3);
```

### Custom Configuration

```rust
use hedl_neo4j::{to_cypher_with_config, CypherConfig, Strategy};

let config = CypherConfig::builder()
    .strategy(Strategy::Merge)           // Use MERGE instead of CREATE
    .batch_size(5000)                    // 5000 nodes per UNWIND
    .generate_constraints(true)          // Generate uniqueness constraints
    .relationship_prefix("REL_")         // Prefix for relationship types
    .build();

let cypher = to_cypher_with_config(&doc, &config)?;
```

### Neo4j → HEDL

Import Neo4j query results back to HEDL:

```rust
use hedl_neo4j::from_neo4j_records;
use neo4j::Record;

// Execute Neo4j query
let records: Vec<Record> = session.run(
    "MATCH (u:User)-[:AUTHORED]->(p:Post) RETURN u, p",
    None
).await?;

// Convert to HEDL
let doc = from_neo4j_records(&records)?;

// Use HEDL's structured API
for (type_name, entities) in &doc.entities {
    println!("{}: {} entities", type_name, entities.len());
}
```

## Cypher Generation Strategies

### CREATE Strategy (Default)

Creates new nodes unconditionally:

```cypher
CREATE (alice:User {id: "alice", name: "Alice"});
CREATE (bob:User {id: "bob", name: "Bob"});
```

**Use When**:
- Importing into empty database
- Guaranteed no duplicate IDs
- Maximum performance (no existence checks)

**Trade-off**: Fails if nodes already exist

### MERGE Strategy

Creates or updates existing nodes:

```cypher
MERGE (alice:User {id: "alice"})
ON CREATE SET alice.name = "Alice", alice.created = timestamp()
ON MATCH SET alice.name = "Alice", alice.updated = timestamp();

MERGE (bob:User {id: "bob"})
ON CREATE SET bob.name = "Bob", bob.created = timestamp()
ON MATCH SET bob.name = "Bob", bob.updated = timestamp();
```

**Use When**:
- Incremental updates to existing database
- Idempotent imports (safe to run multiple times)
- Uncertain about existing data

**Trade-off**: Slower than CREATE (requires existence check)

## Relationship Mapping

### Reference Fields → Relationships

Reference values in fields automatically become relationships:

```hedl
posts: @Post[id, author, title]
  | post1, @User:alice, Hello World
```

**Generated**:
```cypher
CREATE (post1:Post {id: "post1", title: "Hello World"});
MATCH (post1:Post {id: "post1"}), (alice:User {id: "alice"})
CREATE (post1)-[:AUTHOR]->(alice);
```

**Relationship Type**: Field name uppercased (e.g., `author` → `AUTHOR`)

### NEST Pattern → HAS_* Relationships

Parent-child nesting becomes HAS_* relationships:

```hedl
%NEST: User: Post
---
users: @User
  | alice, Alice
  posts: @Post
    | post1, Hello
```

**Generated**:
```cypher
CREATE (alice:User {id: "alice", name: "Alice"});
CREATE (post1:Post {id: "post1", title: "Hello"});
MATCH (alice:User {id: "alice"}), (post1:Post {id: "post1"})
CREATE (alice)-[:HAS_POST]->(post1);
```

**Pattern**: `HAS_{CHILD_TYPE}` (e.g., HAS_POST, HAS_COMMENT, HAS_ITEM)

## Constraint Generation

Automatic uniqueness constraints on entity IDs:

```cypher
CREATE CONSTRAINT user_id IF NOT EXISTS
FOR (n:User) REQUIRE n.id IS UNIQUE;

CREATE CONSTRAINT post_id IF NOT EXISTS
FOR (n:Post) REQUIRE n.id IS UNIQUE;
```

**Benefits**:
- Prevents duplicate nodes
- Improves query performance (indexed lookups)
- Enforces data integrity

**Configuration**:
```rust
.generate_constraints(true)  // Enable (default: true)
.generate_constraints(false) // Disable
```

## Batch Processing

UNWIND-based batching for large imports:

```cypher
// Instead of many CREATE statements...
UNWIND [
  {id: "alice", name: "Alice"},
  {id: "bob", name: "Bob"},
  {id: "carol", name: "Carol"},
  // ... 1000 nodes
] AS row
CREATE (n:User)
SET n = row;
```

**Benefits**:
- Reduces network round-trips (1 statement vs 1000)
- Faster execution (batch planning)
- Lower memory overhead on client

**Configuration**:
```rust
.batch_size(1000)   // Default: 1000 nodes per UNWIND
.batch_size(5000)   // Larger batches for high-throughput
.batch_size(100)    // Smaller batches for constrained environments
```

**Recommendation**: 1000-5000 for most use cases

## Streaming API

Process large documents without full buffering:

```rust
use hedl_neo4j::{stream_to_cypher, CypherConfig};
use std::fs::File;
use std::io::Write;

let output = File::create("import.cypher")?;
let config = CypherConfig::default();

stream_to_cypher(&doc, output, &config, |event| {
    match event {
        CypherEvent::Constraint(stmt) => println!("Constraint: {}", stmt),
        CypherEvent::NodeBatch(count) => println!("Created {} nodes", count),
        CypherEvent::Relationship(rel) => println!("Relationship: {}", rel),
        CypherEvent::Complete => println!("Export complete"),
    }
})?;
```

**Memory Usage**: O(batch_size) regardless of total document size

**Use Cases**:
- Multi-GB document exports
- Memory-constrained environments
- Incremental progress reporting

## Property Type Mapping

HEDL types map to Neo4j property types:

```rust
// HEDL Value → Neo4j Property
Value::Int(42)              → 42 (Long)
Value::Float(3.14)          → 3.14 (Double)
Value::String("alice")      → "alice" (String)
Value::Bool(true)           → true (Boolean)
Value::Null                 → null

// References become relationships (not properties)
Value::Reference(...)       → (relationship edge)

// Expressions evaluated then converted
Value::Expression("$(1+2)") → 3 (Long)
```

**Special Cases**:
- NaN/Infinity floats → `null`
- Empty strings → `""`
- Very long strings → truncated with warning (100 MB limit)

## Security Features

### Unicode Normalization

All strings normalized to NFC form:

```rust
// Input: "café" (e + combining accent)
// Output: "café" (single composed character)
```

**Prevents**:
- Homograph attacks
- Duplicate nodes from equivalent Unicode
- Sort order inconsistencies

### Zero-Width Character Filtering

Invisible characters removed:

```rust
// Input: "alice\u{200B}smith" (contains zero-width space)
// Output: "alicesmith"
```

**Prevents**:
- Hidden characters causing match failures
- Visual spoofing attacks
- Duplicate detection bypasses

### String Length Limits

Maximum string lengths enforced:

```rust
const MAX_STRING_LENGTH: usize = 100 * 1024 * 1024;  // 100 MB default
const MAX_STRING_LENGTH_UNTRUSTED: usize = 1024 * 1024;  // 1 MB for untrusted
```

**Protection Against**:
- Memory exhaustion
- Neo4j property size limits
- DoS via large strings

**Configuration**:
```rust
.max_string_length(10 * 1024 * 1024)  // 10 MB limit
```

## Bidirectional Conversion

### Export: HEDL → Neo4j

```rust
let cypher = to_cypher(&hedl_doc)?;
// Execute cypher statements in Neo4j
```

### Import: Neo4j → HEDL

```rust
// Query Neo4j
let result = session.run("MATCH (u:User) RETURN u", None).await?;

// Convert to HEDL
let hedl_doc = from_neo4j_records(&result.records)?;

// Now use HEDL APIs
let users = &hedl_doc.entities["User"];
```

**Preserved**:
- Entity types (node labels)
- Property values (all types)
- References (from relationships)
- Nesting structure (from HAS_* patterns)

**Not Preserved**:
- Relationship properties (converted to references only)
- Multiple labels per node (uses first label as type)
- Paths and graph structure (flattened to entities + references)

## Configuration Reference

### CypherConfig

```rust
use hedl_neo4j::{CypherConfig, Strategy};

let config = CypherConfig::builder()
    .strategy(Strategy::Create)              // CREATE or MERGE (default: CREATE)
    .batch_size(1000)                        // Nodes per UNWIND (default: 1000)
    .generate_constraints(true)              // Uniqueness constraints (default: true)
    .relationship_prefix("")                 // Relationship prefix (default: "")
    .max_string_length(100 * 1024 * 1024)   // 100 MB (default)
    .normalize_unicode(true)                 // NFC normalization (default: true)
    .filter_zero_width(true)                 // Remove invisible chars (default: true)
    .build();
```

### FromNeo4jConfig

```rust
use hedl_neo4j::FromNeo4jConfig;

let config = FromNeo4jConfig::builder()
    .infer_schemas(true)                     // Auto-generate %STRUCT (default: true)
    .preserve_node_labels(true)              // Keep original labels (default: true)
    .relationship_to_reference(true)         // Edges → references (default: true)
    .build();
```

## Error Handling

```rust
use hedl_neo4j::{to_cypher, Neo4jError};

match to_cypher(&doc) {
    Ok(cypher) => println!("{}", cypher),
    Err(Neo4jError::StringTooLong { length, max, field }) => {
        eprintln!("String too long in field '{}': {} bytes (max: {})",
            field, length, max);
    }
    Err(Neo4jError::InvalidReference { reference, line }) => {
        eprintln!("Invalid reference at line {}: {}", line, reference);
    }
    Err(Neo4jError::UnsupportedType { type_name, value }) => {
        eprintln!("Unsupported type '{}': {:?}", type_name, value);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### Error Types

- `StringTooLong` - String exceeds max length
- `InvalidReference` - Malformed reference
- `UnsupportedType` - Value type not supported by Neo4j
- `BatchSizeInvalid` - Batch size must be > 0
- `Io(std::io::Error)` - I/O failures
- `Neo4jDriver(String)` - Neo4j driver errors

## Use Cases

**Knowledge Graphs**: Export HEDL-structured knowledge bases to Neo4j for graph queries, path finding, centrality analysis.

**Fraud Detection**: Load transaction data from HEDL into Neo4j, run pattern-matching queries to detect fraud rings, export results back to HEDL for reporting.

**Recommendation Engines**: Import user-product interactions from HEDL, compute collaborative filtering in Neo4j, export recommendations as HEDL for integration.

**ETL Pipelines**: Read structured data from various sources (JSON/CSV/XML), convert to HEDL, transform with HEDL tools, export to Neo4j for graph analytics.

**Data Migration**: Migrate from other graph formats to Neo4j via HEDL intermediate representation. Export Neo4j databases to HEDL for backup or transformation.

**Graph Visualization**: Export Neo4j query results to HEDL, convert to JSON/XML for visualization tools, preserve full type information.

## What This Crate Doesn't Do

**Complex Cypher Queries**: Generates CREATE/MERGE statements, not arbitrary Cypher. For custom queries, use Neo4j driver directly and convert results with `from_neo4j_records`.

**Schema Evolution**: Doesn't handle schema migrations or versioning. For evolving schemas, manage migrations externally.

**Relationship Properties**: Relationships map from references (no properties). For rich relationships with properties, use Neo4j driver directly.

**Transaction Management**: Doesn't manage Neo4j transactions. Wrap generated Cypher in transactions via Neo4j driver.

**Multi-Database**: Targets single Neo4j database. For multi-database scenarios, run conversions separately per database.

## Performance Characteristics

**Cypher Generation**: O(n) where n = total entities + relationships. Single linear pass.

**Batch Processing**: Reduces Neo4j import time by 60-80% vs individual CREATE statements.

**Streaming**: O(batch_size) memory usage regardless of document size.

**Unicode Normalization**: O(string_length) per string. Adds <2% overhead.

**From Neo4j**: O(n * m) where n = records, m = average properties per record.

## Dependencies

- `hedl-core` 1.0 - Core HEDL implementation
- `thiserror` 1.0 - Error types
- `unicode-normalization` 0.1 - NFC normalization
- `neo4j` 0.42 (optional) - Neo4j driver for `from_neo4j_records`

## License

Apache-2.0
