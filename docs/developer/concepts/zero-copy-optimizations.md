# Memory Optimization Concepts

HEDL's parser is designed for high performance and memory efficiency.

## Design Philosophy

The HEDL parser follows a "pay for what you use" model with regards to memory:

1. **Streaming First**: The preferred way to handle large datasets is via the streaming API, which never loads the entire document into memory.
2. **Efficient Parsing**: The core parser minimizes temporary allocations during the parsing phase.
3. **Owned AST**: The resulting Abstract Syntax Tree (AST) uses owned `String` and `Box<str>` types.

## Why Not Full Zero-Copy?

Zero-copy parsing (using `&str` or `Cow<'a, str>` throughout) would be faster and use less memory because:

- **No allocation overhead**: Strings point directly into the input buffer
- **Better cache locality**: All string data lives in contiguous memory
- **Reduced memory pressure**: ~50% less memory for string-heavy documents

However, HEDL chose owned strings for the AST. Here's why:

### 1. API Simplicity

Zero-copy requires lifetime parameters on all types:

```rust
// Zero-copy (complex)
struct Document<'a> {
    root: BTreeMap<Cow<'a, str>, Item<'a>>,
}

// Owned (simple)
struct Document {
    root: BTreeMap<String, Item>,
}
```

Users would need to manage lifetimes, making the API harder to use.

### 2. Transformation Freedom

With owned strings, you can freely mutate the AST:

```rust
// Easy with owned strings
doc.root.insert("new_key".to_string(), Item::Scalar(Value::Int(42)));

// With zero-copy, you'd need to allocate anyway for new keys
```

### 3. Thread Safety

Owned data can be sent between threads without lifetime concerns:

```rust
// Easy: send document to another thread
std::thread::spawn(move || {
    process(doc);
});

// With zero-copy: input buffer must outlive the thread
```

### 4. FFI Compatibility

C bindings and WebAssembly require owned data. Zero-copy would need conversion at the FFI boundary anyway.

### 5. Real-World Performance

For typical HEDL use cases (config files, data interchange):

- Documents are small to medium (< 10 MB)
- Modern allocators (jemalloc, mimalloc) are very fast
- The parsing phase (which IS zero-copy) dominates performance
- String allocation is ~5-10% of total parse time

## Performance Characteristics

| Operation | Memory Strategy | Rationale |
|-----------|-----------------|-----------|
| Line Splitting | Zero-copy (iterators over slices) | Hot path, no allocation needed |
| Tokenization | Zero-copy (references to input) | Hot path, temporary references |
| Value Parsing | Direct conversion | Numbers parsed without intermediate strings |
| AST Construction | Allocation (owned `String`, `Box<str>`) | Simplicity, safety, FFI compatibility |

## When to Use Streaming Instead

For large documents where memory is critical, use the streaming API:

```rust
use hedl_stream::StreamingParser;

// Process nodes one at a time, never load full document
let parser = StreamingParser::new(reader).unwrap();
for node in parser {
    process_node(node?);
}
```

Streaming is zero-copy by design and handles files larger than available RAM.

## Optimization Techniques Used

### 1. Pre-allocation for CSV Rows

Field vectors are pre-allocated based on comma count during CSV row parsing:

```rust
// In CSV row parsing (lex/row.rs)
let estimated_fields = csv_string.bytes().filter(|&b| b == b',').count() + 1;
let mut fields = Vec::with_capacity(estimated_fields);
let estimated_field_capacity = (csv_string.len() / estimated_fields.max(1)).max(16);
let mut current_field = String::with_capacity(estimated_field_capacity);
```

Note: `BTreeMap` does not support pre-allocation; this optimization applies to `Vec` types.

### 2. Box<str> for Strings

`Value::String` uses `Box<str>` instead of `String` to reduce enum size:

```rust
pub enum Value {
    String(Box<str>),  // 16 bytes (pointer + length)
    // vs String which would be 24 bytes (pointer + length + capacity)
}
```

### 3. Direct Numeric Parsing

Numbers are parsed directly from slices without intermediate String allocation:

```rust
// Parse "42" directly to i64, no String::from("42")
let value = input[start..end].parse::<i64>()?;
```

### 4. Inverted Indices

Reference resolution uses inverted indices (`ID -> [Types]`) for O(1) lookups:

```rust
// Instead of scanning all nodes for @User:alice
let types = id_index.get("alice"); // O(1) lookup
```

## Future Considerations

A zero-copy mode could be added as an opt-in feature for advanced users:

```rust
// Potential future API (not implemented)
let doc: Document<'_> = parse_zero_copy(input)?;
// Use within input's lifetime
```

This would require significant API changes and is tracked in the roadmap.

## Related

- [AST Design](ast-design.md)
- [ADR-003: Zero-Copy Design](../../architecture/decisions/adr-003-zero-copy-design.md)
