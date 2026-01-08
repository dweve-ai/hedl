# hedl-neo4j

Bidirectional Neo4j integration for HEDL documents.

## Installation

```toml
[dependencies]
hedl-neo4j = "1.0"
```

## Usage

```rust
use hedl_core::parse;
use hedl_neo4j::{to_cypher, from_neo4j_records, ToCypherConfig};

// HEDL to Cypher
let doc = parse(hedl.as_bytes())?;
let cypher = to_cypher(&doc)?;
// Returns CREATE statements for Neo4j import

// From Neo4j records
let doc = from_neo4j_records(records, &config)?;
```

## Features

- **Cypher generation** - Generate CREATE statements from HEDL
- **Record import** - Convert Neo4j query results to HEDL
- **Reference mapping** - HEDL references become Neo4j relationships
- **Constraint generation** - Generate uniqueness constraints

## Example

HEDL input:
```hedl
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
  | bob, Bob
follows: @User:bob
```

Generated Cypher:
```cypher
CREATE (alice:User {id: "alice", name: "Alice"})
CREATE (bob:User {id: "bob", name: "Bob"})
CREATE (alice)-[:FOLLOWS]->(bob)
```

## License

Apache-2.0
