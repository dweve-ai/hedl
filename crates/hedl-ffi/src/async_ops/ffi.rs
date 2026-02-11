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

//! Public FFI functions for async operations.

use super::thread_pool::get_or_init_thread_pool;
use super::types::{HedlAsyncOp, HedlCompletionCallback, Operation, WorkItem};
use crate::error::set_error;
use crate::ffi_strings::get_input_string;
use crate::memory::is_valid_document_ptr;
use crate::types::HedlDocument;
use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
pub unsafe extern "C" fn hedl_canonicalize_async(
    doc: *const HedlDocument,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_canonicalize_async called");

    if doc.is_null() {
        set_error("Invalid document pointer");
        return std::ptr::null_mut();
    }

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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
pub unsafe extern "C" fn hedl_lint_async(
    doc: *const HedlDocument,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_lint_async called");

    if doc.is_null() {
        set_error("Null document pointer");
        return std::ptr::null_mut();
    }

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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
pub unsafe extern "C" fn hedl_to_json_async(
    doc: *const HedlDocument,
    include_metadata: c_int,
    callback: HedlCompletionCallback,
    user_data: *mut c_void,
) -> *mut HedlAsyncOp {
    tracing::debug!("hedl_to_json_async called");

    if doc.is_null() {
        set_error("Invalid document pointer");
        return std::ptr::null_mut();
    }

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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
// SAFETY: Pointer is valid and non-null, checked by caller or validation function.
pub unsafe extern "C" fn hedl_async_free(op: *mut HedlAsyncOp) {
    if op.is_null() {
        return;
    }

    let op_ref = &*op;
    tracing::debug!(op_id = op_ref.id, "Freeing async operation handle");
    let _ = Box::from_raw(op);
}
