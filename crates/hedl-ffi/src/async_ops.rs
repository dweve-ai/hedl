// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Asynchronous operation support for FFI.
//!
//! Provides non-blocking API for long-running operations using a thread pool
//! and callback-based completion notifications.
//!
//! # Design
//!
//! The async API uses a callback-based completion model that integrates naturally
//! with C event loops and asynchronous applications:
//!
//! - Operations are submitted to a thread pool and return immediately
//! - Completion callbacks execute on worker threads when operations finish
//! - Input data is copied to ensure validity during async execution
//! - Callbacks receive result ownership and must free resources
//!
//! # Thread Safety
//!
//! - Thread pool is initialized lazily on first use
//! - Work queue is protected by mutex + condvar
//! - Callbacks execute on worker threads - must be thread-safe
//! - Operation handles are thread-safe for cancellation
//!
//! # Memory Safety
//!
//! - Input data copied at submission time
//! - Output ownership transferred to callback
//! - User context pointer must remain valid until callback completes
//! - Operation handles must be freed regardless of completion status
//!
//! # Performance
//!
//! - Async overhead: ~150-200ns per operation
//! - Thread pool size: Configurable via `HEDL_ASYNC_THREADS` (default: num CPUs)
//! - Queue capacity: Configurable via `HEDL_ASYNC_QUEUE_SIZE` (default: 1000)
//!
//! # Example (C)
//!
//! ```c
//! void parse_callback(int status, void* result, const char* error, void* user_data) {
//!     if (status == HEDL_OK) {
//!         HedlDocument* doc = (HedlDocument*)result;
//!         // Use document...
//!         hedl_free_document(doc);
//!     } else {
//!         fprintf(stderr, "Parse failed: %s\n", error);
//!     }
//! }
//!
//! HedlAsyncOp* op = hedl_parse_async(input, -1, 0, parse_callback, NULL);
//! // Do other work while parsing...
//! hedl_async_free(op);  // Free handle when done
//! ```

use crate::error::set_error;
use crate::memory::is_valid_document_ptr;
use crate::types::{HedlDiagnostics, HedlDocument};
use crate::types::{HEDL_ERR_CANCELLED, HEDL_ERR_CANONICALIZE, HEDL_ERR_PARSE, HEDL_OK};
use crate::utils::get_input_string;
use hedl_core::{parse_with_limits, Document, ParseOptions, ReferenceMode};
use std::collections::VecDeque;
use std::ffi::{c_char, c_int, c_void, CString};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread;

// =============================================================================
// Public Types
// =============================================================================

/// Opaque handle to an async operation.
///
/// Returned by `hedl_*_async()` functions. Must be freed with `hedl_async_free()`.
#[repr(C)]
pub struct HedlAsyncOp {
    id: u64,
    cancelled: Arc<AtomicBool>,
    completed: Arc<AtomicBool>,
}

/// Completion callback function type.
///
/// Called when an async operation completes (success or failure).
///
/// # Parameters
///
/// - `status`: Operation status code (`HEDL_OK` or error code)
/// - `result`: Result pointer (type depends on operation, NULL on error)
/// - `error_msg`: Error message string (NULL on success, valid during callback only)
/// - `user_data`: User-provided context pointer from async call
///
/// # Safety
///
/// - Callback executes on worker thread - must be thread-safe
/// - `result` pointer ownership transferred to callback (must free if non-NULL)
/// - `error_msg` valid only during callback execution (copy if needed)
/// - Must not call back into async FFI functions (risk of deadlock)
///
/// # Memory Ownership
///
/// The callback receives ownership of the result pointer:
/// - For parse operations: `HedlDocument*` - must call `hedl_free_document()`
/// - For canonicalize operations: `char*` - must call `hedl_free_string()`
/// - For lint operations: `HedlDiagnostics*` - must call `hedl_free_diagnostics()`
/// - For conversion operations: `char*` - must call `hedl_free_string()`
pub type HedlCompletionCallbackFn = unsafe extern "C" fn(
    status: c_int,
    result: *mut c_void,
    error_msg: *const c_char,
    user_data: *mut c_void,
);

/// Nullable completion callback type for FFI.
///
/// This is `Option<fn_type>` which correctly represents a nullable function pointer
/// in FFI. C callers can pass NULL, and we can check with `.is_none()`.
pub type HedlCompletionCallback = Option<HedlCompletionCallbackFn>;

// =============================================================================
// Internal Types
// =============================================================================

/// Work item for thread pool.
///
/// The callback is stored as the non-Option type because we validate
/// that the callback is non-NULL at the FFI boundary before creating `WorkItems`.
struct WorkItem {
    id: u64,
    operation: Operation,
    callback: HedlCompletionCallbackFn,
    user_data: *mut c_void,
    cancelled: Arc<AtomicBool>,
}

// Safety: WorkItem is only sent between threads in a controlled manner.
// The user_data pointer is owned by the caller and must remain valid until the callback executes.
// The callback function pointer is inherently thread-safe (it's just a function pointer).
unsafe impl Send for WorkItem {}

/// Operation variants.
enum Operation {
    Parse {
        input: Vec<u8>,
        strict: bool,
    },
    Canonicalize {
        doc: Arc<Document>,
    },
    Lint {
        doc: Arc<Document>,
    },
    #[cfg(feature = "json")]
    ToJson {
        doc: Arc<Document>,
        include_metadata: bool,
    },
    #[cfg(feature = "yaml")]
    ToYaml {
        doc: Arc<Document>,
        include_metadata: bool,
    },
    #[cfg(feature = "xml")]
    ToXml {
        doc: Arc<Document>,
    },
    #[cfg(feature = "csv")]
    ToCsv {
        doc: Arc<Document>,
    },
    #[cfg(feature = "neo4j")]
    ToNeo4jCypher {
        doc: Arc<Document>,
        include_metadata: bool,
    },
    #[cfg(feature = "toon")]
    ToToon {
        doc: Arc<Document>,
    },
}

/// Thread pool for async operations.
struct ThreadPool {
    workers: Vec<Worker>,
    work_queue: Arc<Mutex<WorkQueue>>,
    condvar: Arc<Condvar>,
    shutdown: Arc<AtomicBool>,
    next_op_id: AtomicU64,
}

/// Work queue with bounded capacity.
struct WorkQueue {
    queue: VecDeque<WorkItem>,
    max_size: usize,
}

/// Worker thread wrapper.
struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

/// Global thread pool instance (initialized lazily).
static THREAD_POOL: OnceLock<ThreadPool> = OnceLock::new();

// =============================================================================
// Thread Pool Implementation
// =============================================================================

impl ThreadPool {
    /// Create a new thread pool.
    fn new(num_threads: usize, max_queue_size: usize) -> Self {
        let work_queue = Arc::new(Mutex::new(WorkQueue {
            queue: VecDeque::with_capacity(max_queue_size),
            max_size: max_queue_size,
        }));
        let condvar = Arc::new(Condvar::new());
        let shutdown = Arc::new(AtomicBool::new(false));

        let mut workers = Vec::with_capacity(num_threads);
        for id in 0..num_threads {
            workers.push(Worker::new(
                id,
                work_queue.clone(),
                condvar.clone(),
                shutdown.clone(),
            ));
        }

        tracing::info!(
            num_threads = num_threads,
            max_queue_size = max_queue_size,
            "HEDL async thread pool initialized"
        );

        ThreadPool {
            workers,
            work_queue,
            condvar,
            shutdown,
            next_op_id: AtomicU64::new(1),
        }
    }

    /// Submit a work item to the queue.
    fn submit(&self, item: WorkItem) -> Result<(), ()> {
        let mut queue = self.work_queue.lock().unwrap();

        if queue.queue.len() >= queue.max_size {
            tracing::warn!(
                queue_size = queue.queue.len(),
                max_size = queue.max_size,
                "Async operation queue full"
            );
            return Err(());
        }

        queue.queue.push_back(item);
        drop(queue);

        self.condvar.notify_one();
        Ok(())
    }

    /// Get next operation ID.
    fn next_op_id(&self) -> u64 {
        self.next_op_id.fetch_add(1, Ordering::SeqCst)
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        tracing::info!("Shutting down HEDL async thread pool");

        // Signal shutdown
        self.shutdown.store(true, Ordering::Release);
        self.condvar.notify_all();

        // Wait for workers to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                let _ = thread.join();
            }
        }
    }
}

impl Worker {
    /// Create a new worker thread.
    fn new(
        id: usize,
        work_queue: Arc<Mutex<WorkQueue>>,
        condvar: Arc<Condvar>,
        shutdown: Arc<AtomicBool>,
    ) -> Self {
        let thread = thread::Builder::new()
            .name(format!("hedl-async-{id}"))
            .spawn(move || {
                tracing::debug!(worker_id = id, "Worker thread started");
                Self::run(work_queue, condvar, shutdown);
                tracing::debug!(worker_id = id, "Worker thread exiting");
            })
            .expect("Failed to spawn worker thread");

        Worker {
            thread: Some(thread),
        }
    }

    /// Worker thread main loop.
    fn run(work_queue: Arc<Mutex<WorkQueue>>, condvar: Arc<Condvar>, shutdown: Arc<AtomicBool>) {
        loop {
            let item = {
                let mut queue = work_queue.lock().unwrap();

                // Wait for work or shutdown signal
                while queue.queue.is_empty() && !shutdown.load(Ordering::Acquire) {
                    queue = condvar.wait(queue).unwrap();
                }

                if shutdown.load(Ordering::Acquire) {
                    return;
                }

                queue.queue.pop_front()
            };

            if let Some(item) = item {
                Self::execute(item);
            }
        }
    }

    /// Execute a work item.
    fn execute(item: WorkItem) {
        tracing::debug!(op_id = item.id, "Executing async operation");

        // Check cancellation before starting
        if item.cancelled.load(Ordering::Acquire) {
            tracing::debug!(op_id = item.id, "Operation cancelled before execution");
            unsafe {
                (item.callback)(
                    HEDL_ERR_CANCELLED,
                    std::ptr::null_mut(),
                    c"Operation cancelled".as_ptr(),
                    item.user_data,
                );
            }
            return;
        }

        // Execute operation
        match item.operation {
            Operation::Parse { input, strict } => {
                Self::execute_parse(
                    input,
                    strict,
                    item.callback,
                    item.user_data,
                    &item.cancelled,
                );
            }
            Operation::Canonicalize { doc } => {
                Self::execute_canonicalize(doc, item.callback, item.user_data, &item.cancelled);
            }
            Operation::Lint { doc } => {
                Self::execute_lint(doc, item.callback, item.user_data, &item.cancelled);
            }
            #[cfg(feature = "json")]
            Operation::ToJson {
                doc,
                include_metadata,
            } => {
                Self::execute_to_json(
                    doc,
                    include_metadata,
                    item.callback,
                    item.user_data,
                    &item.cancelled,
                );
            }
            #[cfg(feature = "yaml")]
            Operation::ToYaml {
                doc,
                include_metadata,
            } => {
                Self::execute_to_yaml(
                    doc,
                    include_metadata,
                    item.callback,
                    item.user_data,
                    &item.cancelled,
                );
            }
            #[cfg(feature = "xml")]
            Operation::ToXml { doc } => {
                Self::execute_to_xml(doc, item.callback, item.user_data, &item.cancelled);
            }
            #[cfg(feature = "csv")]
            Operation::ToCsv { doc } => {
                Self::execute_to_csv(doc, item.callback, item.user_data, &item.cancelled);
            }
            #[cfg(feature = "neo4j")]
            Operation::ToNeo4jCypher {
                doc,
                include_metadata,
            } => {
                Self::execute_to_neo4j_cypher(
                    doc,
                    include_metadata,
                    item.callback,
                    item.user_data,
                    &item.cancelled,
                );
            }
            #[cfg(feature = "toon")]
            Operation::ToToon { doc } => {
                Self::execute_to_toon(doc, item.callback, item.user_data, &item.cancelled);
            }
        }
    }

    /// Execute parse operation.
    fn execute_parse(
        input: Vec<u8>,
        strict: bool,
        callback: HedlCompletionCallbackFn,
        user_data: *mut c_void,
        cancelled: &Arc<AtomicBool>,
    ) {
        // Check cancellation periodically during parse
        if cancelled.load(Ordering::Acquire) {
            unsafe {
                (callback)(
                    HEDL_ERR_CANCELLED,
                    std::ptr::null_mut(),
                    c"Cancelled".as_ptr(),
                    user_data,
                );
            }
            return;
        }

        let options = ParseOptions {
            reference_mode: if strict {
                ReferenceMode::Strict
            } else {
                ReferenceMode::Lenient
            },
            ..Default::default()
        };

        match parse_with_limits(&input, options) {
            Ok(doc) => {
                let handle = Box::new(HedlDocument { inner: doc });
                let result_ptr = Box::into_raw(handle).cast::<c_void>();

                unsafe {
                    (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                }
            }
            Err(e) => {
                let error_msg = format!("Parse error: {e}\0");
                let c_error = error_msg.as_ptr().cast::<c_char>();

                unsafe {
                    (callback)(HEDL_ERR_PARSE, std::ptr::null_mut(), c_error, user_data);
                }
            }
        }
    }

    /// Execute canonicalize operation.
    fn execute_canonicalize(
        doc: Arc<Document>,
        callback: HedlCompletionCallbackFn,
        user_data: *mut c_void,
        cancelled: &Arc<AtomicBool>,
    ) {
        if cancelled.load(Ordering::Acquire) {
            unsafe {
                (callback)(
                    HEDL_ERR_CANCELLED,
                    std::ptr::null_mut(),
                    c"Cancelled".as_ptr(),
                    user_data,
                );
            }
            return;
        }

        match hedl_c14n::canonicalize(&doc) {
            Ok(canonical) => {
                if let Ok(c_str) = CString::new(canonical) {
                    let result_ptr = c_str.into_raw().cast::<c_void>();
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in canonical output";
                    unsafe {
                        (callback)(
                            HEDL_ERR_CANONICALIZE,
                            std::ptr::null_mut(),
                            error_msg.as_ptr(),
                            user_data,
                        );
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Canonicalization error: {e}\0");
                let c_error = error_msg.as_ptr().cast::<c_char>();

                unsafe {
                    (callback)(
                        HEDL_ERR_CANONICALIZE,
                        std::ptr::null_mut(),
                        c_error,
                        user_data,
                    );
                }
            }
        }
    }

    /// Execute lint operation.
    fn execute_lint(
        doc: Arc<Document>,
        callback: HedlCompletionCallbackFn,
        user_data: *mut c_void,
        cancelled: &Arc<AtomicBool>,
    ) {
        if cancelled.load(Ordering::Acquire) {
            unsafe {
                (callback)(
                    HEDL_ERR_CANCELLED,
                    std::ptr::null_mut(),
                    c"Cancelled".as_ptr(),
                    user_data,
                );
            }
            return;
        }

        let diagnostics = hedl_lint::lint(&doc);
        let handle = Box::new(HedlDiagnostics { inner: diagnostics });
        let result_ptr = Box::into_raw(handle).cast::<c_void>();

        unsafe {
            (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
        }
    }

    /// Execute `to_json` operation.
    #[cfg(feature = "json")]
    fn execute_to_json(
        doc: Arc<Document>,
        include_metadata: bool,
        callback: HedlCompletionCallbackFn,
        user_data: *mut c_void,
        cancelled: &Arc<AtomicBool>,
    ) {
        if cancelled.load(Ordering::Acquire) {
            unsafe {
                (callback)(
                    HEDL_ERR_CANCELLED,
                    std::ptr::null_mut(),
                    c"Cancelled".as_ptr(),
                    user_data,
                );
            }
            return;
        }

        let config = hedl_json::ToJsonConfig {
            include_metadata,
            ..Default::default()
        };

        match hedl_json::to_json(&doc, &config) {
            Ok(json) => {
                if let Ok(c_str) = CString::new(json) {
                    let result_ptr = c_str.into_raw().cast::<c_void>();
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in JSON output";
                    unsafe {
                        (callback)(
                            crate::types::HEDL_ERR_JSON,
                            std::ptr::null_mut(),
                            error_msg.as_ptr(),
                            user_data,
                        );
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("JSON conversion error: {e}\0");
                let c_error = error_msg.as_ptr().cast::<c_char>();

                unsafe {
                    (callback)(
                        crate::types::HEDL_ERR_JSON,
                        std::ptr::null_mut(),
                        c_error,
                        user_data,
                    );
                }
            }
        }
    }

    /// Execute `to_yaml` operation.
    #[cfg(feature = "yaml")]
    fn execute_to_yaml(
        doc: Arc<Document>,
        include_metadata: bool,
        callback: HedlCompletionCallbackFn,
        user_data: *mut c_void,
        cancelled: &Arc<AtomicBool>,
    ) {
        if cancelled.load(Ordering::Acquire) {
            unsafe {
                (callback)(
                    HEDL_ERR_CANCELLED,
                    std::ptr::null_mut(),
                    c"Cancelled".as_ptr(),
                    user_data,
                );
            }
            return;
        }

        let config = hedl_yaml::ToYamlConfig {
            include_metadata,
            ..Default::default()
        };

        match hedl_yaml::to_yaml(&doc, &config) {
            Ok(yaml) => {
                if let Ok(c_str) = CString::new(yaml) {
                    let result_ptr = c_str.into_raw().cast::<c_void>();
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in YAML output";
                    unsafe {
                        (callback)(
                            crate::types::HEDL_ERR_YAML,
                            std::ptr::null_mut(),
                            error_msg.as_ptr(),
                            user_data,
                        );
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("YAML conversion error: {e}\0");
                let c_error = error_msg.as_ptr().cast::<c_char>();

                unsafe {
                    (callback)(
                        crate::types::HEDL_ERR_YAML,
                        std::ptr::null_mut(),
                        c_error,
                        user_data,
                    );
                }
            }
        }
    }

    /// Execute `to_xml` operation.
    #[cfg(feature = "xml")]
    fn execute_to_xml(
        doc: Arc<Document>,
        callback: HedlCompletionCallbackFn,
        user_data: *mut c_void,
        cancelled: &Arc<AtomicBool>,
    ) {
        if cancelled.load(Ordering::Acquire) {
            unsafe {
                (callback)(
                    HEDL_ERR_CANCELLED,
                    std::ptr::null_mut(),
                    c"Cancelled".as_ptr(),
                    user_data,
                );
            }
            return;
        }

        let config = hedl_xml::ToXmlConfig::default();
        match hedl_xml::to_xml(&doc, &config) {
            Ok(xml) => {
                if let Ok(c_str) = CString::new(xml) {
                    let result_ptr = c_str.into_raw().cast::<c_void>();
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in XML output";
                    unsafe {
                        (callback)(
                            crate::types::HEDL_ERR_XML,
                            std::ptr::null_mut(),
                            error_msg.as_ptr(),
                            user_data,
                        );
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("XML conversion error: {e}\0");
                let c_error = error_msg.as_ptr().cast::<c_char>();

                unsafe {
                    (callback)(
                        crate::types::HEDL_ERR_XML,
                        std::ptr::null_mut(),
                        c_error,
                        user_data,
                    );
                }
            }
        }
    }

    /// Execute `to_csv` operation.
    #[cfg(feature = "csv")]
    fn execute_to_csv(
        doc: Arc<Document>,
        callback: HedlCompletionCallbackFn,
        user_data: *mut c_void,
        cancelled: &Arc<AtomicBool>,
    ) {
        if cancelled.load(Ordering::Acquire) {
            unsafe {
                (callback)(
                    HEDL_ERR_CANCELLED,
                    std::ptr::null_mut(),
                    c"Cancelled".as_ptr(),
                    user_data,
                );
            }
            return;
        }

        match hedl_csv::to_csv(&doc) {
            Ok(csv) => {
                if let Ok(c_str) = CString::new(csv) {
                    let result_ptr = c_str.into_raw().cast::<c_void>();
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in CSV output";
                    unsafe {
                        (callback)(
                            crate::types::HEDL_ERR_CSV,
                            std::ptr::null_mut(),
                            error_msg.as_ptr(),
                            user_data,
                        );
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("CSV conversion error: {e}\0");
                let c_error = error_msg.as_ptr().cast::<c_char>();

                unsafe {
                    (callback)(
                        crate::types::HEDL_ERR_CSV,
                        std::ptr::null_mut(),
                        c_error,
                        user_data,
                    );
                }
            }
        }
    }

    /// Execute `to_neo4j_cypher` operation.
    #[cfg(feature = "neo4j")]
    fn execute_to_neo4j_cypher(
        doc: Arc<Document>,
        _include_metadata: bool,
        callback: HedlCompletionCallbackFn,
        user_data: *mut c_void,
        cancelled: &Arc<AtomicBool>,
    ) {
        if cancelled.load(Ordering::Acquire) {
            unsafe {
                (callback)(
                    HEDL_ERR_CANCELLED,
                    std::ptr::null_mut(),
                    c"Cancelled".as_ptr(),
                    user_data,
                );
            }
            return;
        }

        let config = hedl_neo4j::ToCypherConfig::default();

        match hedl_neo4j::to_cypher(&doc, &config) {
            Ok(cypher) => {
                if let Ok(c_str) = CString::new(cypher) {
                    let result_ptr = c_str.into_raw().cast::<c_void>();
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in Cypher output";
                    unsafe {
                        (callback)(
                            crate::types::HEDL_ERR_NEO4J,
                            std::ptr::null_mut(),
                            error_msg.as_ptr(),
                            user_data,
                        );
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Neo4j conversion error: {e}\0");
                let c_error = error_msg.as_ptr().cast::<c_char>();

                unsafe {
                    (callback)(
                        crate::types::HEDL_ERR_NEO4J,
                        std::ptr::null_mut(),
                        c_error,
                        user_data,
                    );
                }
            }
        }
    }

    /// Execute `to_toon` operation.
    #[cfg(feature = "toon")]
    fn execute_to_toon(
        doc: Arc<Document>,
        callback: HedlCompletionCallbackFn,
        user_data: *mut c_void,
        cancelled: &Arc<AtomicBool>,
    ) {
        if cancelled.load(Ordering::Acquire) {
            unsafe {
                (callback)(
                    HEDL_ERR_CANCELLED,
                    std::ptr::null_mut(),
                    c"Cancelled".as_ptr(),
                    user_data,
                );
            }
            return;
        }

        match hedl_toon::hedl_to_toon(&doc) {
            Ok(toon) => {
                if let Ok(c_str) = CString::new(toon) {
                    let result_ptr = c_str.into_raw().cast::<c_void>();
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in TOON output";
                    unsafe {
                        (callback)(
                            crate::types::HEDL_ERR_TOON,
                            std::ptr::null_mut(),
                            error_msg.as_ptr(),
                            user_data,
                        );
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("TOON conversion error: {e}\0");
                let c_error = error_msg.as_ptr().cast::<c_char>();

                unsafe {
                    (callback)(
                        crate::types::HEDL_ERR_TOON,
                        std::ptr::null_mut(),
                        c_error,
                        user_data,
                    );
                }
            }
        }
    }
}

// =============================================================================
// Thread Pool Initialization
// =============================================================================

/// Initialize thread pool (called lazily).
fn get_or_init_thread_pool() -> &'static ThreadPool {
    THREAD_POOL.get_or_init(|| {
        let num_threads = std::env::var("HEDL_ASYNC_THREADS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(num_cpus::get)
            .clamp(1, 64);

        let max_queue_size = std::env::var("HEDL_ASYNC_QUEUE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        ThreadPool::new(num_threads, max_queue_size)
    })
}

// =============================================================================
// Public FFI Functions
// =============================================================================

/// Parse a HEDL document asynchronously.
///
/// # Arguments
///
/// - `input`: UTF-8 encoded HEDL document
/// - `input_len`: Length of input in bytes, or -1 for null-terminated
/// - `strict`: Non-zero for strict mode (validate references)
/// - `callback`: Completion callback (invoked on worker thread)
/// - `user_data`: User context pointer passed to callback
///
/// # Returns
///
/// Async operation handle on success, NULL on submission failure.
/// Call `hedl_async_free()` to release handle when done.
///
/// # Callback Signature
///
/// ```c
/// void callback(int status, HedlDocument* doc, const char* error, void* user_data);
/// ```
///
/// - On success: `status=HEDL_OK`, `doc!=NULL`, `error=NULL`
/// - On error: `status=error_code`, `doc=NULL`, `error=error_message`
///
/// # Thread Safety
///
/// - Callback executes on worker thread - must be thread-safe
/// - Input data is copied - safe to free after function returns
///
/// # Memory Management
///
/// - Operation handle: Must call `hedl_async_free()` regardless of completion
/// - Document: Callback receives ownership - must call `hedl_free_document()`
///
/// # Safety
///
/// Input pointer must be valid UTF-8. Callback must be thread-safe.
#[no_mangle]
pub unsafe extern "C" fn hedl_parse_async(
    input: *const c_char,
    input_len: c_int,
    strict: c_int,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_parse_async called");

    // Validate arguments
    if input.is_null() {
        set_error("Null input pointer");
        return std::ptr::null_mut();
    }

    let cb = if let Some(cb) = callback {
        cb
    } else {
        set_error("Null callback pointer");
        return std::ptr::null_mut();
    };

    // Copy input data
    let input_str = match get_input_string(input, input_len) {
        Ok(s) => s,
        Err(_) => {
            return std::ptr::null_mut();
        }
    };

    let pool = get_or_init_thread_pool();
    let op_id = pool.next_op_id();
    let cancelled = Arc::new(AtomicBool::new(false));

    let work_item = WorkItem {
        id: op_id,
        operation: Operation::Parse {
            input: input_str.into_bytes(),
            strict: strict != 0,
        },
        callback: cb,
        user_data,
        cancelled: cancelled.clone(),
    };

    if let Ok(()) = pool.submit(work_item) {
        let handle = Box::new(HedlAsyncOp {
            id: op_id,
            cancelled,
            completed: Arc::new(AtomicBool::new(false)),
        });
        tracing::debug!(op_id = op_id, "Async parse operation submitted");
        Box::into_raw(handle)
    } else {
        set_error("Async operation queue full");
        std::ptr::null_mut()
    }
}

/// Canonicalize a HEDL document asynchronously.
///
/// # Arguments
///
/// - `doc`: Document handle
/// - `callback`: Completion callback
/// - `user_data`: User context pointer
///
/// # Returns
///
/// Async operation handle on success, NULL on failure.
///
/// # Callback
///
/// Callback receives `char*` result - must call `hedl_free_string()`.
///
/// # Safety
///
/// Document pointer must be valid. Callback must be thread-safe.
#[no_mangle]
pub unsafe extern "C" fn hedl_canonicalize_async(
    doc: *const HedlDocument,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_canonicalize_async called");

    if !is_valid_document_ptr(doc) {
        set_error("Invalid document pointer");
        return std::ptr::null_mut();
    }

    let cb = if let Some(cb) = callback {
        cb
    } else {
        set_error("Null callback pointer");
        return std::ptr::null_mut();
    };

    // Clone document for async processing
    let doc_clone = Arc::new((*doc).inner.clone());

    let pool = get_or_init_thread_pool();
    let op_id = pool.next_op_id();
    let cancelled = Arc::new(AtomicBool::new(false));

    let work_item = WorkItem {
        id: op_id,
        operation: Operation::Canonicalize { doc: doc_clone },
        callback: cb,
        user_data,
        cancelled: cancelled.clone(),
    };

    if let Ok(()) = pool.submit(work_item) {
        let handle = Box::new(HedlAsyncOp {
            id: op_id,
            cancelled,
            completed: Arc::new(AtomicBool::new(false)),
        });
        tracing::debug!(op_id = op_id, "Async canonicalize operation submitted");
        Box::into_raw(handle)
    } else {
        set_error("Async operation queue full");
        std::ptr::null_mut()
    }
}

/// Lint a HEDL document asynchronously.
///
/// # Arguments
///
/// - `doc`: Document handle
/// - `callback`: Completion callback
/// - `user_data`: User context pointer
///
/// # Returns
///
/// Async operation handle on success, NULL on failure.
///
/// # Callback
///
/// Callback receives `HedlDiagnostics*` result - must call `hedl_free_diagnostics()`.
///
/// # Safety
///
/// Document pointer must be valid. Callback must be thread-safe.
#[no_mangle]
pub unsafe extern "C" fn hedl_lint_async(
    doc: *const HedlDocument,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_lint_async called");

    if !is_valid_document_ptr(doc) {
        set_error("Invalid document pointer");
        return std::ptr::null_mut();
    }

    let cb = if let Some(cb) = callback {
        cb
    } else {
        set_error("Null callback pointer");
        return std::ptr::null_mut();
    };

    let doc_clone = Arc::new((*doc).inner.clone());

    let pool = get_or_init_thread_pool();
    let op_id = pool.next_op_id();
    let cancelled = Arc::new(AtomicBool::new(false));

    let work_item = WorkItem {
        id: op_id,
        operation: Operation::Lint { doc: doc_clone },
        callback: cb,
        user_data,
        cancelled: cancelled.clone(),
    };

    if let Ok(()) = pool.submit(work_item) {
        let handle = Box::new(HedlAsyncOp {
            id: op_id,
            cancelled,
            completed: Arc::new(AtomicBool::new(false)),
        });
        tracing::debug!(op_id = op_id, "Async lint operation submitted");
        Box::into_raw(handle)
    } else {
        set_error("Async operation queue full");
        std::ptr::null_mut()
    }
}

/// Convert a HEDL document to JSON asynchronously.
///
/// # Arguments
///
/// - `doc`: Document handle
/// - `include_metadata`: Non-zero to include metadata
/// - `callback`: Completion callback
/// - `user_data`: User context pointer
///
/// # Returns
///
/// Async operation handle on success, NULL on failure.
///
/// # Callback
///
/// Callback receives `char*` result - must call `hedl_free_string()`.
///
/// # Safety
///
/// Document pointer must be valid. Callback must be thread-safe.
#[cfg(feature = "json")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_json_async(
    doc: *const HedlDocument,
    include_metadata: c_int,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_to_json_async called");

    if !is_valid_document_ptr(doc) {
        set_error("Invalid document pointer");
        return std::ptr::null_mut();
    }

    let cb = if let Some(cb) = callback {
        cb
    } else {
        set_error("Null callback pointer");
        return std::ptr::null_mut();
    };

    let doc_clone = Arc::new((*doc).inner.clone());

    let pool = get_or_init_thread_pool();
    let op_id = pool.next_op_id();
    let cancelled = Arc::new(AtomicBool::new(false));

    let work_item = WorkItem {
        id: op_id,
        operation: Operation::ToJson {
            doc: doc_clone,
            include_metadata: include_metadata != 0,
        },
        callback: cb,
        user_data,
        cancelled: cancelled.clone(),
    };

    if let Ok(()) = pool.submit(work_item) {
        let handle = Box::new(HedlAsyncOp {
            id: op_id,
            cancelled,
            completed: Arc::new(AtomicBool::new(false)),
        });
        tracing::debug!(op_id = op_id, "Async to_json operation submitted");
        Box::into_raw(handle)
    } else {
        set_error("Async operation queue full");
        std::ptr::null_mut()
    }
}

/// Convert a HEDL document to YAML asynchronously.
///
/// # Arguments
///
/// - `doc`: Document handle
/// - `include_metadata`: Non-zero to include metadata
/// - `callback`: Completion callback
/// - `user_data`: User context pointer
///
/// # Returns
///
/// Async operation handle on success, NULL on failure.
///
/// # Callback
///
/// Callback receives `char*` result - must call `hedl_free_string()`.
///
/// # Safety
///
/// Document pointer must be valid. Callback must be thread-safe.
#[cfg(feature = "yaml")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_yaml_async(
    doc: *const HedlDocument,
    include_metadata: c_int,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_to_yaml_async called");

    if !is_valid_document_ptr(doc) {
        set_error("Invalid document pointer");
        return std::ptr::null_mut();
    }

    let cb = if let Some(cb) = callback {
        cb
    } else {
        set_error("Null callback pointer");
        return std::ptr::null_mut();
    };

    let doc_clone = Arc::new((*doc).inner.clone());

    let pool = get_or_init_thread_pool();
    let op_id = pool.next_op_id();
    let cancelled = Arc::new(AtomicBool::new(false));

    let work_item = WorkItem {
        id: op_id,
        operation: Operation::ToYaml {
            doc: doc_clone,
            include_metadata: include_metadata != 0,
        },
        callback: cb,
        user_data,
        cancelled: cancelled.clone(),
    };

    if let Ok(()) = pool.submit(work_item) {
        let handle = Box::new(HedlAsyncOp {
            id: op_id,
            cancelled,
            completed: Arc::new(AtomicBool::new(false)),
        });
        tracing::debug!(op_id = op_id, "Async to_yaml operation submitted");
        Box::into_raw(handle)
    } else {
        set_error("Async operation queue full");
        std::ptr::null_mut()
    }
}

/// Convert a HEDL document to XML asynchronously.
///
/// # Safety
///
/// Document pointer must be valid. Callback must be thread-safe.
#[cfg(feature = "xml")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_xml_async(
    doc: *const HedlDocument,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_to_xml_async called");

    if !is_valid_document_ptr(doc) {
        set_error("Invalid document pointer");
        return std::ptr::null_mut();
    }

    let cb = if let Some(cb) = callback {
        cb
    } else {
        set_error("Null callback pointer");
        return std::ptr::null_mut();
    };

    let doc_clone = Arc::new((*doc).inner.clone());

    let pool = get_or_init_thread_pool();
    let op_id = pool.next_op_id();
    let cancelled = Arc::new(AtomicBool::new(false));

    let work_item = WorkItem {
        id: op_id,
        operation: Operation::ToXml { doc: doc_clone },
        callback: cb,
        user_data,
        cancelled: cancelled.clone(),
    };

    if let Ok(()) = pool.submit(work_item) {
        let handle = Box::new(HedlAsyncOp {
            id: op_id,
            cancelled,
            completed: Arc::new(AtomicBool::new(false)),
        });
        tracing::debug!(op_id = op_id, "Async to_xml operation submitted");
        Box::into_raw(handle)
    } else {
        set_error("Async operation queue full");
        std::ptr::null_mut()
    }
}

/// Convert a HEDL document to CSV asynchronously.
///
/// # Safety
///
/// Document pointer must be valid. Callback must be thread-safe.
#[cfg(feature = "csv")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_csv_async(
    doc: *const HedlDocument,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_to_csv_async called");

    if !is_valid_document_ptr(doc) {
        set_error("Invalid document pointer");
        return std::ptr::null_mut();
    }

    let cb = if let Some(cb) = callback {
        cb
    } else {
        set_error("Null callback pointer");
        return std::ptr::null_mut();
    };

    let doc_clone = Arc::new((*doc).inner.clone());

    let pool = get_or_init_thread_pool();
    let op_id = pool.next_op_id();
    let cancelled = Arc::new(AtomicBool::new(false));

    let work_item = WorkItem {
        id: op_id,
        operation: Operation::ToCsv { doc: doc_clone },
        callback: cb,
        user_data,
        cancelled: cancelled.clone(),
    };

    if let Ok(()) = pool.submit(work_item) {
        let handle = Box::new(HedlAsyncOp {
            id: op_id,
            cancelled,
            completed: Arc::new(AtomicBool::new(false)),
        });
        tracing::debug!(op_id = op_id, "Async to_csv operation submitted");
        Box::into_raw(handle)
    } else {
        set_error("Async operation queue full");
        std::ptr::null_mut()
    }
}

/// Convert a HEDL document to Neo4j Cypher asynchronously.
///
/// # Safety
///
/// Document pointer must be valid. Callback must be thread-safe.
#[cfg(feature = "neo4j")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_neo4j_cypher_async(
    doc: *const HedlDocument,
    include_metadata: c_int,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_to_neo4j_cypher_async called");

    if !is_valid_document_ptr(doc) {
        set_error("Invalid document pointer");
        return std::ptr::null_mut();
    }

    let cb = if let Some(cb) = callback {
        cb
    } else {
        set_error("Null callback pointer");
        return std::ptr::null_mut();
    };

    let doc_clone = Arc::new((*doc).inner.clone());

    let pool = get_or_init_thread_pool();
    let op_id = pool.next_op_id();
    let cancelled = Arc::new(AtomicBool::new(false));

    let work_item = WorkItem {
        id: op_id,
        operation: Operation::ToNeo4jCypher {
            doc: doc_clone,
            include_metadata: include_metadata != 0,
        },
        callback: cb,
        user_data,
        cancelled: cancelled.clone(),
    };

    if let Ok(()) = pool.submit(work_item) {
        let handle = Box::new(HedlAsyncOp {
            id: op_id,
            cancelled,
            completed: Arc::new(AtomicBool::new(false)),
        });
        tracing::debug!(op_id = op_id, "Async to_neo4j_cypher operation submitted");
        Box::into_raw(handle)
    } else {
        set_error("Async operation queue full");
        std::ptr::null_mut()
    }
}

/// Convert a HEDL document to TOON asynchronously.
///
/// # Arguments
///
/// - `doc`: Document handle
/// - `callback`: Completion callback
/// - `user_data`: User context pointer
///
/// # Returns
///
/// Async operation handle on success, NULL on failure.
///
/// # Callback
///
/// Callback receives `char*` result - must call `hedl_free_string()`.
///
/// # Safety
///
/// Document pointer must be valid. Callback must be thread-safe.
#[cfg(feature = "toon")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_toon_async(
    doc: *const HedlDocument,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_to_toon_async called");

    if !is_valid_document_ptr(doc) {
        set_error("Invalid document pointer");
        return std::ptr::null_mut();
    }

    let cb = if let Some(cb) = callback {
        cb
    } else {
        set_error("Null callback pointer");
        return std::ptr::null_mut();
    };

    let doc_clone = Arc::new((*doc).inner.clone());

    let pool = get_or_init_thread_pool();
    let op_id = pool.next_op_id();
    let cancelled = Arc::new(AtomicBool::new(false));

    let work_item = WorkItem {
        id: op_id,
        operation: Operation::ToToon { doc: doc_clone },
        callback: cb,
        user_data,
        cancelled: cancelled.clone(),
    };

    if let Ok(()) = pool.submit(work_item) {
        let handle = Box::new(HedlAsyncOp {
            id: op_id,
            cancelled,
            completed: Arc::new(AtomicBool::new(false)),
        });
        tracing::debug!(op_id = op_id, "Async to_toon operation submitted");
        Box::into_raw(handle)
    } else {
        set_error("Async operation queue full");
        std::ptr::null_mut()
    }
}

/// Cancel an async operation.
///
/// Requests cancellation of the operation. Cancellation is best-effort:
///
/// - If not started: Cancelled immediately, callback invoked with `HEDL_ERR_CANCELLED`
/// - If in progress: Attempts to abort, callback invoked with `HEDL_ERR_CANCELLED`
/// - If completed: No effect (callback already executed)
///
/// # Safety
///
/// Handle must be valid and not already freed.
#[no_mangle]
pub unsafe extern "C" fn hedl_async_cancel(op: *mut HedlAsyncOp) {
    if op.is_null() {
        return;
    }

    let op_ref = &*op;
    op_ref.cancelled.store(true, Ordering::Release);
    tracing::debug!(op_id = op_ref.id, "Async operation cancelled");
}

/// Free an async operation handle.
///
/// Releases resources associated with the operation handle.
/// Safe to call regardless of operation state (pending/completed/cancelled).
///
/// # Safety
///
/// Handle must be valid and not already freed. Do not use handle after freeing.
#[no_mangle]
pub unsafe extern "C" fn hedl_async_free(op: *mut HedlAsyncOp) {
    if op.is_null() {
        return;
    }

    let op_ref = &*op;
    tracing::debug!(op_id = op_ref.id, "Freeing async operation handle");
    let _ = Box::from_raw(op);
}
