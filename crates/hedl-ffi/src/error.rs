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

//! Error handling for FFI.

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

// =============================================================================
// Error Management (Thread-Local)
// =============================================================================

thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<CString>> = const { std::cell::RefCell::new(None) };
}

pub(crate) fn set_error(msg: &str) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = CString::new(msg).ok();
    });
}

pub(crate) fn clear_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

/// Get the last error message for the current thread.
///
/// Returns NULL if no error occurred on this thread.
///
/// # Thread Safety
///
/// Error messages are stored in thread-local storage. This function returns
/// the error message for the CALLING THREAD ONLY. You must call this function
/// from the same thread that received the error code.
///
/// Each thread maintains its own independent error state. Errors in one thread
/// do not affect or overwrite errors in other threads. This makes the FFI safe
/// to use in multi-threaded applications without external synchronization for
/// error handling.
///
/// # Lifetime
///
/// The returned pointer is valid until the next `hedl_*` call on this thread.
/// Copy the string immediately if you need to preserve it.
///
/// # Example (C)
///
/// ```c
/// // Thread 1
/// if (hedl_parse(input1, -1, 0, &doc1) != HEDL_OK) {
///     const char* err1 = hedl_get_last_error();
///     printf("Thread 1 error: %s\n", err1);
/// }
///
/// // Thread 2 (concurrent with Thread 1)
/// if (hedl_parse(input2, -1, 0, &doc2) != HEDL_OK) {
///     const char* err2 = hedl_get_last_error();
///     printf("Thread 2 error: %s\n", err2);
/// }
/// // err1 and err2 are independent - no cross-thread pollution
/// ```
#[no_mangle]
pub extern "C" fn hedl_get_last_error() -> *const c_char {
    LAST_ERROR.with(|e| match &*e.borrow() {
        Some(cstr) => cstr.as_ptr(),
        None => ptr::null(),
    })
}

/// Clear the last error for the current thread.
///
/// This function explicitly clears any error message stored for the calling thread.
/// It is generally not necessary to call this function, as successful operations
/// automatically clear errors. However, it can be useful in specific scenarios:
///
/// - Testing error handling logic
/// - Clearing stale errors before a sequence of operations
/// - Resetting error state in long-running thread pools
///
/// # Thread Safety
///
/// Like `hedl_get_last_error()`, this function operates on thread-local storage
/// and only affects the error state of the calling thread. Other threads' error
/// states remain unchanged.
///
/// # Example (C)
///
/// ```c
/// // Clear any previous errors
/// hedl_clear_error_threadsafe();
///
/// // Now perform operations with a clean error state
/// if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
///     // This error is definitely from hedl_parse, not a previous operation
///     const char* err = hedl_get_last_error();
///     handle_error(err);
/// }
/// ```
#[no_mangle]
pub extern "C" fn hedl_clear_error_threadsafe() {
    clear_error();
}

/// Get the last error message for the current thread (thread-safe variant).
///
/// This is an explicitly named alias of `hedl_get_last_error()` to make the
/// thread-safety guarantee clear in the function name.
///
/// Returns NULL if no error occurred on this thread.
///
/// # Thread Safety
///
/// This function is fully thread-safe. Each thread maintains its own independent
/// error state in thread-local storage. Concurrent calls from multiple threads
/// will not interfere with each other.
///
/// **Guarantees:**
/// - Errors from thread A will never appear in thread B
/// - No synchronization primitives (mutexes, locks) are required
/// - Zero contention between threads accessing errors
/// - Lock-free, wait-free operation
///
/// # Lifetime
///
/// The returned pointer is valid until the next `hedl_*` call on this thread.
/// Copy the string immediately if you need to preserve it.
///
/// # Example (C with pthreads)
///
/// ```c
/// void* worker_thread(void* arg) {
///     HedlDocument* doc = NULL;
///     const char* input = (const char*)arg;
///
///     if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
///         // Get error for THIS thread only
///         const char* err = hedl_get_last_error_threadsafe();
///         fprintf(stderr, "Worker thread error: %s\n", err);
///         return NULL;
///     }
///
///     // Process document...
///     hedl_free_document(doc);
///     return (void*)1;
/// }
///
/// int main() {
///     pthread_t threads[4];
///     const char* inputs[4] = { input1, input2, input3, input4 };
///
///     for (int i = 0; i < 4; i++) {
///         pthread_create(&threads[i], NULL, worker_thread, (void*)inputs[i]);
///     }
///
///     for (int i = 0; i < 4; i++) {
///         pthread_join(threads[i], NULL);
///     }
/// }
/// ```
#[no_mangle]
pub extern "C" fn hedl_get_last_error_threadsafe() -> *const c_char {
    hedl_get_last_error()
}

/// Get the thread-local error message as a String (for internal use).
///
/// Returns an empty string if no error occurred.
pub(crate) fn get_thread_local_error() -> String {
    LAST_ERROR.with(|e| match &*e.borrow() {
        Some(cstr) => cstr.to_string_lossy().to_string(),
        None => String::new(),
    })
}
