# Memory Optimization Concepts

HEDL's parser is designed for high performance and memory efficiency.

## Design Philosophy

The HEDL parser follows a "pay for what you use" model with regards to memory:

1. **Streaming First**: The preferred way to handle large datasets is via the streaming API, which never loads the entire document into memory.
2. **Efficient Parsing**: The core parser minimizes temporary allocations during the parsing phase.
3. **Owned AST**: The resulting Abstract Syntax Tree (AST) uses owned `String` types. This decision prioritizes:
    - **Safety**: No lifetime management complexity for users.
    - **Simplicity**: Easier to mutate and transform the AST.
    - **Compatibility**: Better integration with other Rust crates and FFI.

## Performance Characteristics

| Operation | Memory Strategy |
|-----------|-----------------|
| Line Splitting | Zero-copy (iterators over slices) |
| Tokenization | Zero-copy (references to input) |
| AST Construction | Allocation (owned Strings) |

## Optimization Techniques

### 1. Pre-allocation
Vectors and maps are pre-allocated based on estimated sizes to prevent re-allocation during growth.

### 2. String Inference
Numeric values are parsed directly from string slices without intermediate string allocation.

### 3. Inverted Indices
Reference resolution uses inverted indices (`ID -> [Types]`) to make unqualified lookups O(1) instead of O(N).