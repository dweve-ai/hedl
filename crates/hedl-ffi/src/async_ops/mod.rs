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

mod ffi;
mod operations;
mod thread_pool;
mod types;

// Re-export public types
pub use types::{HedlAsyncOp, HedlCompletionCallback, HedlCompletionCallbackFn};

// Re-export public FFI functions
pub use ffi::{
    hedl_async_cancel, hedl_async_free, hedl_canonicalize_async, hedl_lint_async, hedl_parse_async,
};

#[cfg(feature = "csv")]
pub use ffi::hedl_to_csv_async;

#[cfg(feature = "json")]
pub use ffi::hedl_to_json_async;

#[cfg(feature = "neo4j")]
pub use ffi::hedl_to_neo4j_cypher_async;

#[cfg(feature = "toon")]
pub use ffi::hedl_to_toon_async;

#[cfg(feature = "xml")]
pub use ffi::hedl_to_xml_async;

#[cfg(feature = "yaml")]
pub use ffi::hedl_to_yaml_async;
