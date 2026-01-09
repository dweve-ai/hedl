# HEDL-LSP Test Suite

This directory contains comprehensive tests for the HEDL Language Server Protocol implementation.

## Test Files

### Integration Tests

- **`lsp_tests.rs`** - Basic LSP functionality tests
- **`lru_memory_tests.rs`** - LRU cache and memory management tests
- **`header_cache_tests.rs`** - Header boundary caching tests
- **`reference_index_tests.rs`** - Reference index performance tests
- **`concurrency_tests.rs`** - **Comprehensive concurrency and thread safety tests** ⭐

### Test Documentation

- **`CONCURRENCY_TEST_REPORT.md`** - Detailed report on concurrency test results

---

## Concurrency Test Suite

### Overview

The `concurrency_tests.rs` file contains **16 comprehensive tests** validating thread safety across all LSP components. This is the most critical test file for verifying production readiness.

### Quick Start

```bash
# Run all concurrency tests
cargo test --test concurrency_tests

# Run with detailed output
cargo test --test concurrency_tests -- --nocapture

# Run single-threaded for debugging
cargo test --test concurrency_tests -- --test-threads=1
```

### What It Tests

1. **Concurrent Document Operations**
   - Multiple threads inserting different documents
   - Multiple threads updating the same document
   - Concurrent reads during writes
   - Dirty flag tracking under concurrency

2. **Concurrent Analysis Operations**
   - Reading analysis while documents update
   - Analysis rebuilds during queries
   - Internal consistency verification

3. **Cache Concurrency**
   - LRU eviction under load
   - Hot/cold access patterns
   - Cache hit rate validation

4. **Reference Index Thread Safety**
   - Concurrent definition lookups
   - Concurrent reference searches
   - Position-based queries

5. **Stress Testing**
   - 25+ concurrent threads
   - 58,000+ operations in 200ms
   - Mixed read/write/delete operations

6. **Deadlock Detection**
   - Circular access patterns
   - Nested operations
   - Different lock orderings

7. **Memory Consistency**
   - Atomic updates
   - No partial writes
   - Analysis consistency

8. **Performance Metrics**
   - Cache efficiency
   - Operation throughput
   - Eviction rates

### Test Results

See `CONCURRENCY_TEST_REPORT.md` for detailed results including:
- Performance metrics
- Thread safety validation
- Known characteristics
- Recommendations

---

## Advanced Testing

### Loom (Exhaustive Concurrency Testing)

[Loom](https://github.com/tokio-rs/loom) provides exhaustive testing by exploring all possible thread interleavings.

```bash
# Install and run loom tests (slower, but more thorough)
RUSTFLAGS="--cfg loom" cargo test --test concurrency_tests --release -- loom
```

**Included loom tests:**
- `loom_concurrent_insert` - All interleavings of concurrent inserts
- `loom_concurrent_update_same_doc` - All interleavings of concurrent updates

### Thread Sanitizer

Detect data races and threading bugs at runtime:

```bash
# Requires nightly Rust
RUSTFLAGS="-Z sanitizer=thread" \
  cargo +nightly test --test concurrency_tests \
  -Zbuild-std --target x86_64-unknown-linux-gnu
```

---

## Writing New Tests

### Test Template

```rust
#[test]
fn test_concurrent_operation() {
    // Setup
    let manager = Arc::new(DocumentManager::new(100, 1024 * 1024));
    let num_threads = 10;

    // Spawn threads
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            thread::spawn(move || {
                // Thread operations
            })
        })
        .collect();

    // Wait for completion
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify results
    assert!(/* expected behavior */);
}
```

### Best Practices

1. **Use Arc<DocumentManager>** for shared access
2. **Use AtomicBool** for stop flags
3. **Use AtomicUsize** for operation counting
4. **Test with 10+ threads** for realistic concurrency
5. **Use Duration::from_millis()** for time-based tests
6. **Assert no panics** with `.expect("Thread panicked")`
7. **Verify invariants** after concurrent operations

### Common Patterns

#### Reader/Writer Pattern
```rust
let stop_flag = Arc::new(AtomicBool::new(false));

// Readers
for _ in 0..num_readers {
    let manager = Arc::clone(&manager);
    let stop_flag = Arc::clone(&stop_flag);
    handles.push(thread::spawn(move || {
        while !stop_flag.load(Ordering::Relaxed) {
            // Read operations
        }
    }));
}

// Writers
for _ in 0..num_writers {
    let manager = Arc::clone(&manager);
    let stop_flag = Arc::clone(&stop_flag);
    handles.push(thread::spawn(move || {
        while !stop_flag.load(Ordering::Relaxed) {
            // Write operations
        }
    }));
}

// Run test
thread::sleep(Duration::from_millis(100));
stop_flag.store(true, Ordering::SeqCst);
```

#### Stress Test Pattern
```rust
let operation_count = Arc::new(AtomicUsize::new(0));

let handles: Vec<_> = (0..num_threads)
    .map(|thread_id| {
        let manager = Arc::clone(&manager);
        let operation_count = Arc::clone(&operation_count);
        thread::spawn(move || {
            let mut ops = 0;
            // Perform operations
            ops += 1;
            operation_count.fetch_add(ops, Ordering::SeqCst);
        })
    })
    .collect();

// Verify total operations
assert!(operation_count.load(Ordering::SeqCst) > expected);
```

---

## Test Quality Checklist

When adding new concurrency tests, verify:

- [ ] Tests with realistic thread counts (10+)
- [ ] Tests both success and edge cases
- [ ] No hardcoded sleeps (except for time-based tests)
- [ ] All threads are joined (no orphan threads)
- [ ] Assertions validate expected behavior
- [ ] Test names clearly describe what is tested
- [ ] Documentation explains why the test is important
- [ ] Cleanup occurs (documents are removed if needed)
- [ ] No resource leaks (URIs, memory, etc.)

---

## Debugging Failed Tests

### Enable Logging
```bash
RUST_LOG=debug cargo test --test concurrency_tests -- --nocapture
```

### Run Single Test
```bash
cargo test --test concurrency_tests test_concurrent_document_inserts -- --nocapture
```

### Run with Backtrace
```bash
RUST_BACKTRACE=1 cargo test --test concurrency_tests
```

### Use Thread Sanitizer
```bash
RUSTFLAGS="-Z sanitizer=thread" \
  cargo +nightly test --test concurrency_tests \
  -Zbuild-std --target x86_64-unknown-linux-gnu
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
- name: Run Concurrency Tests
  run: |
    cargo test --test concurrency_tests -- --nocapture

- name: Run Concurrency Tests (Thread Sanitizer)
  run: |
    rustup default nightly
    RUSTFLAGS="-Z sanitizer=thread" \
      cargo +nightly test --test concurrency_tests \
      -Zbuild-std --target x86_64-unknown-linux-gnu
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running concurrency tests..."
cargo test --test concurrency_tests --quiet

if [ $? -ne 0 ]; then
    echo "❌ Concurrency tests failed! Commit aborted."
    exit 1
fi

echo "✅ Concurrency tests passed!"
```

---

## Performance Benchmarks

See `benches/` directory for performance benchmarks:

- `header_boundary_cache.rs` - Header cache performance
- `reference_index.rs` - Reference lookup performance

Run benchmarks:
```bash
cargo bench
```

---

## Additional Resources

- [Rust Concurrency Book](https://rust-lang.github.io/async-book/)
- [Loom Documentation](https://docs.rs/loom/)
- [Thread Sanitizer Guide](https://doc.rust-lang.org/beta/unstable-book/compiler-flags/sanitizer.html)
- [DashMap Documentation](https://docs.rs/dashmap/)
- [parking_lot Documentation](https://docs.rs/parking_lot/)

