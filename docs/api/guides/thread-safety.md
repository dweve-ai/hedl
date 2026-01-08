# Thread Safety Guide

Comprehensive guide to thread-safe HEDL usage across Rust, FFI, and WASM interfaces.

## Rust API Thread Safety

### Thread-Safe Types

All core HEDL types are `Send` and `Sync` where appropriate:

```rust
use hedl::{Document, Value};
use std::sync::Arc;
use std::thread;

// Documents are Send + Sync
let doc = Arc::new(hedl::parse(input)?);

let handles: Vec<_> = (0..4)
    .map(|i| {
        let doc = Arc::clone(&doc);
        thread::spawn(move || {
            // Safe: multiple threads reading the same document
            println!("Thread {}: {} items", i, doc.root.len());
        })
    })
    .collect();

for handle in handles {
    handle.join().unwrap();
}
```

### Concurrent Parsing

```rust
use rayon::prelude::*;

fn parse_many(inputs: &[String]) -> Vec<Result<Document, HedlError>> {
    inputs
        .par_iter()
        .map(|input| hedl::parse(input))
        .collect()
}
```

### Shared Mutable State

Use `Mutex` or `RwLock` for shared mutable state:

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

struct DocumentCache {
    cache: Arc<RwLock<HashMap<String, Document>>>,
}

impl DocumentCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<Document> {
        self.cache.read().unwrap().get(key).cloned()
    }

    pub fn insert(&self, key: String, doc: Document) {
        self.cache.write().unwrap().insert(key, doc);
    }
}
```

## FFI Thread Safety

### Thread-Local Error Messages

The FFI API uses **thread-local storage** for error messages:

```c
#include <pthread.h>
#include "hedl.h"

void* worker(void* arg) {
    const char* input = (const char*)arg;
    HedlDocument* doc = NULL;

    // Each thread has its own error state
    if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
        // Safe: gets error for THIS thread only (thread-local storage)
        const char* err = hedl_get_last_error();
        fprintf(stderr, "Thread error: %s\n", err);
        return NULL;
    }

    // Process document...
    hedl_free_document(doc);
    return (void*)1;
}

int main() {
    pthread_t threads[8];
    const char* inputs[8] = { /* ... */ };

    for (int i = 0; i < 8; i++) {
        pthread_create(&threads[i], NULL, worker, (void*)inputs[i]);
    }

    for (int i = 0; i < 8; i++) {
        pthread_join(threads[i], NULL);
    }

    return 0;
}
```

### Document Handle Safety

**IMPORTANT**: Document handles are **NOT thread-safe**:

```c
// SAFE: Each thread creates its own document
void* safe_worker(void* arg) {
    HedlDocument* doc = NULL;
    hedl_parse(input, -1, 0, &doc);
    // Use doc...
    hedl_free_document(doc);
}

// UNSAFE: Sharing documents across threads
HedlDocument* shared_doc = create_doc();
pthread_create(&t1, NULL, worker, shared_doc);  // DANGER!
pthread_create(&t2, NULL, worker, shared_doc);  // DATA RACE!
```

### Thread-Safe Wrapper (C++)

```cpp
#include <mutex>
#include <memory>
#include "hedl.h"

class ThreadSafeHedlDoc {
private:
    std::unique_ptr<HedlDocument, decltype(&hedl_free_document)> doc_;
    mutable std::mutex mutex_;

public:
    ThreadSafeHedlDoc(HedlDocument* doc)
        : doc_(doc, hedl_free_document) {}

    std::string to_json() const {
        std::lock_guard<std::mutex> lock(mutex_);

        char* json = nullptr;
        if (hedl_to_json(doc_.get(), 0, &json) != HEDL_OK) {
            throw std::runtime_error(hedl_get_last_error());
        }

        std::string result(json);
        hedl_free_string(json);
        return result;
    }

    std::string canonicalize() const {
        std::lock_guard<std::mutex> lock(mutex_);

        char* canonical = nullptr;
        if (hedl_canonicalize(doc_.get(), &canonical) != HEDL_OK) {
            throw std::runtime_error(hedl_get_last_error());
        }

        std::string result(canonical);
        hedl_free_string(canonical);
        return result;
    }
};
```

## Async Rust Patterns

### Tokio Integration

```rust
use tokio::fs;
use tokio::task;

async fn load_and_parse(path: &str) -> Result<Document, HedlError> {
    let content = fs::read_to_string(path).await?;

    // Parse in blocking thread pool
    task::spawn_blocking(move || hedl::parse(&content))
        .await
        .unwrap()
}
```

### Concurrent File Processing

```rust
use tokio::fs;
use futures::stream::{self, StreamExt};

async fn process_files(paths: Vec<String>) -> Vec<Result<Document, HedlError>> {
    stream::iter(paths)
        .map(|path| async move {
            let content = fs::read_to_string(&path).await?;
            task::spawn_blocking(move || hedl::parse(&content))
                .await
                .unwrap()
        })
        .buffer_unordered(10) // Process 10 concurrently
        .collect()
        .await
}
```

### Channel-Based Processing

```rust
use tokio::sync::mpsc;

async fn pipeline_processor(
    rx: mpsc::Receiver<String>,
    tx: mpsc::Sender<Document>,
) {
    let mut rx = rx;

    while let Some(input) = rx.recv().await {
        let result = task::spawn_blocking(move || hedl::parse(&input))
            .await
            .unwrap();

        if let Ok(doc) = result {
            tx.send(doc).await.ok();
        }
    }
}
```

## Race Condition Prevention

### Atomic Operations

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

struct ParseCounter {
    total: AtomicUsize,
    success: AtomicUsize,
    failed: AtomicUsize,
}

impl ParseCounter {
    pub fn new() -> Self {
        Self {
            total: AtomicUsize::new(0),
            success: AtomicUsize::new(0),
            failed: AtomicUsize::new(0),
        }
    }

    pub fn record_parse(&self, result: &Result<Document, HedlError>) {
        self.total.fetch_add(1, Ordering::Relaxed);
        match result {
            Ok(_) => { self.success.fetch_add(1, Ordering::Relaxed); }
            Err(_) => { self.failed.fetch_add(1, Ordering::Relaxed); }
        }
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        (
            self.total.load(Ordering::Relaxed),
            self.success.load(Ordering::Relaxed),
            self.failed.load(Ordering::Relaxed),
        )
    }
}
```

### Lock-Free Patterns

```rust
use crossbeam::queue::SegQueue;

struct DocumentQueue {
    queue: SegQueue<Document>,
}

impl DocumentQueue {
    pub fn new() -> Self {
        Self {
            queue: SegQueue::new(),
        }
    }

    pub fn push(&self, doc: Document) {
        self.queue.push(doc);
    }

    pub fn pop(&self) -> Option<Document> {
        self.queue.pop()
    }
}
```

## Deadlock Prevention

### Lock Ordering

```rust
use std::sync::{Mutex, MutexGuard};

struct TwoResourceSystem {
    resource_a: Mutex<Document>,
    resource_b: Mutex<Document>,
}

impl TwoResourceSystem {
    // Always acquire locks in the same order
    pub fn process(&self) {
        let lock_a = self.resource_a.lock().unwrap();
        let lock_b = self.resource_b.lock().unwrap();

        // Process with both locks held
        // Locks released in reverse order (RAII)
    }
}
```

### Try-Lock Pattern

```rust
use std::time::Duration;

fn try_process_with_timeout(
    doc_mutex: &Mutex<Document>,
    timeout: Duration,
) -> Result<String, Box<dyn std::error::Error>> {
    match doc_mutex.try_lock() {
        Ok(doc) => Ok(hedl::to_json(&doc)?),
        Err(_) => {
            std::thread::sleep(timeout);
            match doc_mutex.try_lock() {
                Ok(doc) => Ok(hedl::to_json(&doc)?),
                Err(_) => Err("Lock acquisition timeout".into()),
            }
        }
    }
}
```

## Best Practices

### Do's

1. **Use Arc for shared ownership** across threads
2. **Parse in parallel** for independent documents
3. **Use thread-local storage** for per-thread state
4. **Prefer RwLock over Mutex** for read-heavy workloads
5. **Profile concurrent code** to identify bottlenecks

### Don'ts

1. **Don't share document handles** across threads (FFI)
2. **Don't hold locks** during expensive operations
3. **Don't assume error messages** are shared across threads
4. **Don't create excessive threads** - use thread pools
5. **Don't ignore lock poisoning** - handle panics properly

## Performance Considerations

### Read-Heavy Workloads

```rust
use std::sync::RwLock;

struct DocStore {
    docs: RwLock<HashMap<String, Document>>,
}

impl DocStore {
    pub fn get(&self, key: &str) -> Option<Document> {
        // Multiple readers can acquire lock simultaneously
        self.docs.read().unwrap().get(key).cloned()
    }

    pub fn insert(&self, key: String, doc: Document) {
        // Writers have exclusive access
        self.docs.write().unwrap().insert(key, doc);
    }
}
```

### Lock-Free Alternatives

```rust
use arc_swap::ArcSwap;

struct ConfigStore {
    config: ArcSwap<Document>,
}

impl ConfigStore {
    pub fn new(initial: Document) -> Self {
        Self {
            config: ArcSwap::from_pointee(initial),
        }
    }

    pub fn get(&self) -> Arc<Document> {
        self.config.load_full()
    }

    pub fn update(&self, new_config: Document) {
        self.config.store(Arc::new(new_config));
    }
}
```

## Testing Thread Safety

### Race Condition Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_concurrent_parsing() {
        let inputs: Vec<String> = (0..100)
            .map(|i| format!("%VERSION: 1.0\n---\nkey_{}: value_{}", i, i))
            .collect();

        let handles: Vec<_> = inputs
            .into_iter()
            .map(|input| {
                thread::spawn(move || hedl::parse(&input))
            })
            .collect();

        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect();

        assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 100);
    }

    #[test]
    fn test_shared_cache() {
        let cache = Arc::new(DocumentCache::new());

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let cache = Arc::clone(&cache);
                thread::spawn(move || {
                    let doc = hedl::parse(&format!("%VERSION: 1.0\n---\nkey_{}: value_{}", i, i)).unwrap();
                    cache.insert(format!("doc_{}", i), doc);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all documents were inserted
        for i in 0..10 {
            assert!(cache.get(&format!("doc_{}", i)).is_some());
        }
    }
}
```

## See Also

- [Rust Best Practices](rust-best-practices.md)
- [Memory Management Guide](memory-management.md)
- [Error Handling Guide](error-handling.md)
- [FFI API Reference](../ffi-api.md)
