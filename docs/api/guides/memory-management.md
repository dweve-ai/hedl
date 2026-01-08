# Memory Management Guide

Comprehensive guide to efficient memory management patterns for HEDL across Rust, FFI, and WASM.

## Rust Memory Management

### RAII and Automatic Cleanup

```rust
use hedl::parse;

fn process_document() -> Result<(), HedlError> {
    let doc = parse(input)?; // Allocated on stack/heap

    // Use document...
    let json = hedl::to_json(&doc)?;

    // doc automatically dropped here (RAII)
    Ok(())
}
```

### Ownership and Borrowing

```rust
use hedl::Document;

// Take ownership - document moved
fn consume_document(doc: Document) {
    // doc is owned by this function
} // doc dropped here

// Borrow immutably - no ownership transfer
fn read_document(doc: &Document) -> usize {
    doc.root.len()
} // doc still owned by caller

// Borrow mutably - exclusive access
fn modify_document(doc: &mut Document) {
    doc.root.clear();
} // doc still owned by caller
```

### Cloning vs References

```rust
use std::sync::Arc;

// Expensive: full clone
fn clone_example(doc: &Document) -> Document {
    doc.clone()
}

// Cheap: reference counted
fn arc_example(doc: Arc<Document>) -> Arc<Document> {
    Arc::clone(&doc) // Only increments ref count
}

// Best: borrow when possible
fn borrow_example(doc: &Document) -> String {
    hedl::to_json(doc).unwrap()
}
```

### Pre-allocation

```rust
use std::collections::BTreeMap;
use hedl::{Item, Value};

// Pre-allocate capacity for known sizes
let mut items: BTreeMap<String, Item> = BTreeMap::new();
for i in 0..1000 {
    items.insert(
        format!("key_{}", i),
        Item::Scalar(Value::Int(i as i64))
    );
}
```

## FFI Memory Management

### Memory Ownership Rules

**Critical**: All FFI memory must be freed with matching `hedl_free_*` functions:

```c
// Correct pattern
HedlDocument* doc = NULL;
hedl_parse(input, -1, 0, &doc);
// Use doc...
hedl_free_document(doc);  // REQUIRED

char* json = NULL;
hedl_to_json(doc, 0, &json);  // 0 = no metadata
// Use json...
hedl_free_string(json);  // REQUIRED

HedlDiagnostics* diag = NULL;
hedl_lint(doc, &diag);
// Use diag...
hedl_free_diagnostics(diag);  // REQUIRED
```

### Common Memory Errors

#### Double Free
```c
// WRONG: Double free
HedlDocument* doc = NULL;
hedl_parse(input, -1, 0, &doc);
hedl_free_document(doc);
hedl_free_document(doc);  // CRASH!
```

#### Use After Free
```c
// WRONG: Use after free
HedlDocument* doc = NULL;
hedl_parse(input, -1, 0, &doc);
hedl_free_document(doc);
hedl_to_json(doc, 0, &json);  // CRASH!
```

#### Wrong Allocator
```c
// WRONG: freeing with wrong allocator
char* json = NULL;
hedl_to_json(doc, 0, &json);
free(json);  // CRASH! Must use hedl_free_string
```

### RAII in C++

```cpp
#include <memory>
#include "hedl.h"

// Custom deleters for RAII
struct HedlDocDeleter {
    void operator()(HedlDocument* p) const {
        if (p) hedl_free_document(p);
    }
};

struct HedlStringDeleter {
    void operator()(char* p) const {
        if (p) hedl_free_string(p);
    }
};

using HedlDocPtr = std::unique_ptr<HedlDocument, HedlDocDeleter>;
using HedlStringPtr = std::unique_ptr<char, HedlStringDeleter>;

// Usage with automatic cleanup
void process_hedl(const char* input) {
    HedlDocument* raw_doc = nullptr;
    if (hedl_parse(input, -1, 0, &raw_doc) != HEDL_OK) {
        throw std::runtime_error(hedl_get_last_error());
    }
    HedlDocPtr doc(raw_doc);  // Automatic cleanup

    char* raw_json = nullptr;
    if (hedl_to_json(doc.get(), 0, &raw_json) != HEDL_OK) {
        throw std::runtime_error(hedl_get_last_error());
    }
    HedlStringPtr json(raw_json);  // Automatic cleanup

    std::cout << json.get() << std::endl;
    // Automatic cleanup on scope exit
}
```

### Resource Pools

```c
typedef struct {
    char** strings;
    size_t count;
    size_t capacity;
} StringPool;

StringPool* pool_create(size_t capacity) {
    StringPool* pool = malloc(sizeof(StringPool));
    pool->strings = malloc(capacity * sizeof(char*));
    pool->count = 0;
    pool->capacity = capacity;
    return pool;
}

void pool_add(StringPool* pool, char* str) {
    if (pool->count < pool->capacity) {
        pool->strings[pool->count++] = str;
    }
}

void pool_destroy(StringPool* pool) {
    for (size_t i = 0; i < pool->count; i++) {
        hedl_free_string(pool->strings[i]);
    }
    free(pool->strings);
    free(pool);
}
```

## Memory Limits

### Configuring Limits

```rust
use hedl::{parse_with_limits, ParseOptions, Limits};

let limits = Limits {
    max_nest_depth: 20,               // Maximum nesting depth
    max_object_keys: 10_000,          // Keys per object
    max_total_keys: 100_000,          // Total keys in document
    max_block_string_size: 1_000_000, // 1 MB strings
    max_file_size: 100_000_000,       // 100 MB documents
    ..Default::default()
};

let options = ParseOptions { limits, strict_refs: true };
let doc = parse_with_limits(input.as_bytes(), options)?;
```

### FFI Limits

The FFI currently supports strict mode via the `strict` parameter to `hedl_parse`:

```c
HedlDocument* doc = NULL;
// strict=1 enables reference validation
hedl_parse(input, -1, 1, &doc);
```

Custom resource limits are not yet exposed via FFI. For advanced limit configuration,
use the Rust API directly with `parse_with_limits`.

## Memory Profiling

### Rust Memory Profiling

```rust
// Use jemalloc for better profiling
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(feature = "profiling")]
use jemalloc_ctl::{stats, epoch};

fn profile_memory_usage() {
    // Trigger stats update
    epoch::mib().unwrap().advance().unwrap();

    // Get current allocated bytes
    let allocated = stats::allocated::mib().unwrap().read().unwrap();
    let resident = stats::resident::mib().unwrap().read().unwrap();

    println!("Allocated: {} bytes", allocated);
    println!("Resident: {} bytes", resident);
}
```

### Tracking Allocations

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

fn current_allocated() -> usize {
    ALLOCATED.load(Ordering::Relaxed)
}
```

## Optimization Patterns

### Pre-allocation

The most effective memory optimization in the current HEDL implementation is pre-allocating capacity for collections (Vec, BTreeMap) when the size is known or can be estimated.

```rust
// Pre-allocate fields vector with exact schema size
let mut fields = Vec::with_capacity(schema.len());
```

### String Handling

While the AST uses owned `String` types for simplicity, format converters like `hedl-json` use zero-copy techniques during the conversion phase to minimize temporary allocations.

---

## Memory Leak Prevention

### Rust Leak Detection

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_memory_leaks() {
        let initial = current_allocated();

        {
            let doc = parse("%VERSION: 1.0\n---\nkey: value").unwrap();
            let _json = hedl::json::to_json(&doc).unwrap();
        }

        let final_alloc = current_allocated();
        assert_eq!(initial, final_alloc, "Memory leak detected");
    }
}
```

### FFI Leak Detection

```c
#include <valgrind/memcheck.h>

void test_memory_leaks() {
    VALGRIND_DO_LEAK_CHECK;

    HedlDocument* doc = NULL;
    hedl_parse(input, -1, 0, &doc);

    char* json = NULL;
    hedl_to_json(doc, 0, &json);

    // Intentionally forget to free
    // hedl_free_string(json);  // LEAK!
    // hedl_free_document(doc);  // LEAK!

    VALGRIND_DO_LEAK_CHECK;
}
```

## Best Practices

### Do's

1. **Free all allocated memory** with matching `hedl_free_*` functions
2. **Use RAII** in C++ for automatic cleanup
3. **Set appropriate limits** to prevent excessive memory usage
4. **Profile memory usage** in production workloads
5. **Prefer borrowing** over cloning in Rust

### Don'ts

1. **Don't mix allocators** (malloc/free with hedl_free_*)
2. **Don't double-free** memory
3. **Don't use after free** (check for null after free)
4. **Don't ignore resource limits** (can cause OOM)
5. **Don't create memory leaks** in long-running applications

## Platform-Specific Considerations

### Windows

```c
// Windows: Use HeapAlloc for FFI if needed
#ifdef _WIN32
#include <windows.h>

void* custom_alloc(size_t size) {
    return HeapAlloc(GetProcessHeap(), 0, size);
}

void custom_free(void* ptr) {
    HeapFree(GetProcessHeap(), 0, ptr);
}
#endif
```

### Embedded Systems

```rust
// For embedded: Use static allocation
static mut DOC_BUFFER: [u8; 4096] = [0; 4096];

fn parse_embedded(input: &str) -> Result<Document, HedlError> {
    // Use fixed-size buffer instead of heap
    unsafe {
        let buffer = &mut DOC_BUFFER;
        // Parse with buffer limits
        parse_with_limits(input.as_bytes(), embedded_options())
    }
}
```

## See Also

- [Thread Safety Guide](thread-safety.md)
- [Rust Best Practices](rust-best-practices.md)
- [Error Handling Guide](error-handling.md)
- [FFI API Reference](../ffi-api.md)
