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

//! Type definitions for async operations.

use hedl_core::Document;
use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

// =============================================================================
// Public Types
// =============================================================================

/// Opaque handle to an async operation.
///
/// Returned by `hedl_*_async()` functions. Must be freed with `hedl_async_free()`.
#[repr(C)]
pub struct HedlAsyncOp {
    pub(crate) id: u64,
    pub(crate) cancelled: Arc<AtomicBool>,
    pub(crate) completed: Arc<AtomicBool>,
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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
pub(crate) struct WorkItem {
    pub(crate) id: u64,
    pub(crate) operation: Operation,
    pub(crate) callback: HedlCompletionCallbackFn,
    pub(crate) user_data: *mut c_void,
    pub(crate) cancelled: Arc<AtomicBool>,
}

// Safety: WorkItem is only sent between threads in a controlled manner.
// The user_data pointer is owned by the caller and must remain valid until the callback executes.
// The callback function pointer is inherently thread-safe (it's just a function pointer).
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
unsafe impl Send for WorkItem {}

/// Operation variants.
pub(crate) enum Operation {
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
