# hedl-neo4j

Graph databases see relationships that tabular data hides. Neo4j powers fraud detection, recommendation engines, knowledge graphs. But getting structured data into Neo4j usually means writing custom ETL scripts, and pulling query results back out means losing type information along the way.

`hedl-neo4j` handles both directions. HEDL documents become Cypher statements. Neo4j query results become HEDL documents. References in your data automatically become graph relationships.

## Why This Exists

HEDL's reference system (`@User:alice`) already describes relationships between entities. That's exactly what graph databases excel at querying. The connection was obvious: HEDL references should become Neo4j edges, and Neo4j relationships should become HEDL references.

We built this to avoid the manual mapping that plagues most graph database integrations. Define your data once in HEDL, and the graph structure emerges naturally from the references you've already declared.

## How It Works

The conversion follows a simple principle: entities become nodes, references become relationships.

When you write:

```hedl
%S:User:[id, name, email]
%S:Post:[id, author, title]
%N:User>Post
---
users: @User
 | alice, Alice Smith, alice@example.com
  posts: @Post
    | post1, @User:alice, Hello World
```

The `@User:alice` reference in the post's author field becomes a relationship. The `%N:User>Post` nest declaration creates a `HAS_POST` edge from parent to child. No configuration needed -the structure in your data determines the graph structure.

Generated Cypher:

```cypher
CREATE CONSTRAINT user_id IF NOT EXISTS FOR (n:User) REQUIRE n.id IS UNIQUE;
CREATE CONSTRAINT post_id IF NOT EXISTS FOR (n:Post) REQUIRE n.id IS UNIQUE;

CREATE (alice:User {id: "alice", name: "Alice Smith", email: "alice@example.com"});
CREATE (post1:Post {id: "post1", title: "Hello World"});

MATCH (post1:Post {id: "post1"}), (alice:User {id: "alice"}) CREATE (post1)-[:AUTHOR]->(alice);
MATCH (alice:User {id: "alice"}), (post1:Post {id: "post1"}) CREATE (alice)-[:HAS_POST]->(post1);
```

## Installation

```toml
[dependencies]
hedl-neo4j = "1"
```

## Basic Usage

### Exporting to Cypher

```rust
use hedl_core::parse;
use hedl_neo4j::{to_cypher, ToCypherConfig};

let doc = parse(br#"
%S:User:[id, name]
---
users: @User
 | alice, Alice Smith
 | bob, Bob Jones
"#)?;

let cypher = to_cypher(&doc, &ToCypherConfig::new())?;
// Execute this in Neo4j
```

### Importing from Neo4j

```rust
use hedl_neo4j::{neo4j_to_hedl, Neo4jRecord, Neo4jNode};

let node = Neo4jNode::new("User", "alice")
    .with_property("name", "Alice Smith");
let records = vec![Neo4jRecord::new(node)];

let doc = neo4j_to_hedl(&records)?;
```

## CREATE vs MERGE

Two strategies for generating Cypher, depending on your situation.

**MERGE** (the default) creates nodes if they don't exist, updates them if they do. Safe to run multiple times. Use this when you're not sure what's already in the database, or when you're doing incremental updates.

```rust
let config = ToCypherConfig::builder()
    .use_merge(true)
    .build();
```

**CREATE** assumes nodes don't exist and fails if they do. Faster because it skips the existence check. Use this when importing into an empty database or when you're certain there are no duplicates.

```rust
let config = ToCypherConfig::builder()
    .use_merge(false)
    .build();
```

## Batch Processing

Individual CREATE statements are slow. For every node, Neo4j parses a statement, plans execution, runs it, returns a result. With thousands of nodes, that overhead dominates.

UNWIND batching sends multiple nodes in a single statement:

```cypher
UNWIND [{id: "alice", name: "Alice"}, {id: "bob", name: "Bob"}, ...] AS row
CREATE (n:User) SET n = row;
```

One parse, one plan, one execution for the whole batch. We default to 1000 nodes per batch, which balances memory usage against round-trip overhead. Adjust based on your environment:

```rust
let config = ToCypherConfig::builder()
    .batch_size(5000)  // Larger batches for high-throughput imports
    .build();
```

## Streaming Large Documents

For documents too large to hold in memory, stream the Cypher output:

```rust
use hedl_neo4j::{to_cypher_stream, ToCypherConfig};
use std::fs::File;

let output = File::create("import.cypher")?;
to_cypher_stream(&doc, output, &ToCypherConfig::new())?;
```

Memory usage stays constant regardless of document size -bounded by the batch size rather than total entities.

## Security Considerations

Strings from untrusted sources need careful handling before they become Cypher. We apply several protections:

**Unicode normalization** converts equivalent representations to the same form. "café" with a combining accent becomes "café" with a single composed character. This prevents duplicate nodes from visually identical but byte-different strings.

**Zero-width character filtering** removes invisible characters that could cause matching failures or visual spoofing.

**String length limits** prevent memory exhaustion from malicious payloads. The default is 100 MB per string -generous for legitimate use, protective against attacks.

```rust
let config = ToCypherConfig::builder()
    .max_string_length(10 * 1024 * 1024)  // 10 MB limit for stricter environments
    .build();
```

## Async Support

For applications with concurrent database operations, enable the async feature:

```toml
[dependencies]
hedl-neo4j = { version = "1", features = ["async"] }
```

```rust
use hedl_neo4j::AsyncNeo4jClient;

let client = AsyncNeo4jClient::connect("bolt://localhost:7687", "neo4j", "password").await?;
client.import_document(&doc).await?;
```

The async API runs batch operations concurrently, providing 3-5x speedup for large imports. Connection pooling handles multiple concurrent operations efficiently.

## Type Mapping

HEDL types map directly to Neo4j property types:

| HEDL | Neo4j |
|------|-------|
| Int | Long |
| Float | Double |
| String | String |
| Bool | Boolean |
| Null | null |
| Reference | Relationship (not a property) |

Special cases: NaN and Infinity become null (Neo4j doesn't support them). Very long strings get truncated with a warning.

## Error Handling

Errors tell you what went wrong and where:

```rust
match to_cypher(&doc, &ToCypherConfig::default()) {
    Ok(cypher) => { /* use it */ }
    Err(Neo4jError::UnresolvedReference { type_name, id }) => {
        eprintln!("Reference @{}:{} doesn't exist", type_name.unwrap_or_default(), id);
    }
    Err(Neo4jError::StringLengthExceeded { length, max_length, property }) => {
        eprintln!("Property '{}' has {} bytes, max is {}", property, length, max_length);
    }
    Err(e) => eprintln!("{}", e),
}
```

## Practical Applications

**Knowledge graphs**: Export HEDL-structured knowledge to Neo4j, run path queries and centrality analysis, export results back for reporting.

**Fraud detection**: Load transaction networks, find suspicious patterns with graph algorithms, flag results in your HEDL-based reporting system.

**Recommendations**: Import user-item interactions, compute collaborative filtering scores, export recommendations for your application.

## License

Apache-2.0
