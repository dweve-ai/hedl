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

//! HEDL FFI Bindings
//!
//! Provides C-compatible interface for HEDL operations.
//! All functions use C-style error handling with return codes.
//!
//! # Memory Management
//!
//! **IMPORTANT:** Memory ownership follows strict rules:
//!
//! - Strings returned by `hedl_*` functions MUST be freed with `hedl_free_string`
//! - Byte arrays returned by `hedl_to_parquet` MUST be freed with `hedl_free_bytes`
//! - Documents MUST be freed with `hedl_free_document`
//! - Diagnostics MUST be freed with `hedl_free_diagnostics`
//!
//! **WARNING - Memory Safety Requirements:**
//!
//! The `hedl_free_*` functions ONLY accept pointers that were allocated by HEDL
//! functions. Passing any of the following will cause undefined behavior:
//!
//! - Pointers from `malloc`/`calloc`/`realloc` (wrong allocator)
//! - Stack-allocated variables
//! - Already-freed pointers (double free)
//! - Pointers from other libraries
//! - NULL is safe and will be ignored
//!
//! # Thread Safety
//!
//! ## Error Handling Thread Safety
//!
//! Error messages are stored in **thread-local storage**, providing lock-free,
//! wait-free error handling for multi-threaded applications.
//!
//! **Key Guarantees:**
//! - Each thread maintains its own independent error state
//! - `hedl_get_last_error()` / `hedl_get_last_error_threadsafe()` returns the
//!   error for the CALLING thread only
//! - Errors from one thread will NEVER appear in or overwrite errors in another thread
//! - No mutexes, locks, or other synchronization primitives are required
//! - Zero contention between threads accessing error messages
//! - You MUST call error functions from the same thread that received the error code
//!
//! **Thread-Safe Functions:**
//! - `hedl_get_last_error()` - Get error for current thread
//! - `hedl_get_last_error_threadsafe()` - Explicit thread-safe alias
//! - `hedl_clear_error_threadsafe()` - Clear error for current thread
//!
//! **Example (Multi-threaded C with pthreads):**
//! ```c
//! void* worker(void* arg) {
//!     const char* input = (const char*)arg;
//!     HedlDocument* doc = NULL;
//!
//!     if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
//!         // Get error for THIS thread - independent of other threads
//!         const char* err = hedl_get_last_error_threadsafe();
//!         fprintf(stderr, "Parse error: %s\n", err);
//!         return NULL;
//!     }
//!
//!     // Process document...
//!     hedl_free_document(doc);
//!     return (void*)1;
//! }
//!
//! int main() {
//!     pthread_t threads[8];
//!     const char* inputs[8] = { ... };
//!
//!     // Launch threads - each with independent error state
//!     for (int i = 0; i < 8; i++) {
//!         pthread_create(&threads[i], NULL, worker, (void*)inputs[i]);
//!     }
//!
//!     for (int i = 0; i < 8; i++) {
//!         pthread_join(threads[i], NULL);
//!     }
//! }
//! ```
//!
//! ## Document Handle Thread Safety
//!
//! Document handles (`HedlDocument*`) are **NOT thread-safe** by design for
//! performance reasons. Do not share document handles between threads without
//! external synchronization (mutexes, etc.).
//!
//! **Safe Pattern:**
//! - Each thread creates its own document handles
//! - Each thread frees its own document handles
//! - No sharing of document pointers across threads
//!
//! **Unsafe Pattern:**
//! - Passing a `HedlDocument*` to multiple threads (data race)
//! - Accessing the same document from multiple threads (undefined behavior)
//!
//! # Error Handling
//!
//! - All functions return error codes (HEDL_OK on success)
//! - Use `hedl_get_last_error` to get the error message for the current thread
//!
//! # Security
//!
//! ## Poison Pointers
//!
//! To detect double-free and use-after-free bugs, this library uses poison pointers:
//!
//! - After freeing a document or diagnostics, the internal pointer is checked against
//!   a poison value
//! - All accessor functions validate that pointers are not poisoned before use
//! - This provides defense-in-depth against memory safety bugs
//!
//! **Note**: Since C passes pointers by value, we cannot modify the caller's pointer
//! after freeing. However, we can detect if a freed pointer is passed back to us
//! by checking for the poison value in accessor functions.
//!
//! # Audit Logging
//!
//! This library provides comprehensive audit logging for all FFI function calls
//! using the `tracing` crate. The logging system captures:
//!
//! - Function entry/exit with timing information
//! - Sanitized parameters (pointer addresses are masked for security)
//! - Success/failure outcomes with error details
//! - Performance metrics (call duration)
//! - Thread context for correlation
//!
//! ## Configuring Logging
//!
//! To enable logging, initialize a tracing subscriber in your application:
//!
//! ```rust,no_run
//! use tracing_subscriber::{fmt, EnvFilter};
//!
//! fn main() {
//!     // Initialize the tracing subscriber
//!     tracing_subscriber::fmt()
//!         .with_env_filter(
//!             EnvFilter::try_from_default_env()
//!                 .unwrap_or_else(|_| EnvFilter::new("info"))
//!         )
//!         .with_target(true)
//!         .with_thread_ids(true)
//!         .with_line_number(true)
//!         .init();
//!
//!     // Now all FFI calls will be logged
//! }
//! ```
//!
//! ## Log Levels
//!
//! - `ERROR`: Function failures with error details
//! - `WARN`: Recoverable errors or unusual conditions
//! - `INFO`: Function call entry/exit with basic metrics
//! - `DEBUG`: Detailed parameter information (sanitized)
//!
//! ## Environment Variables
//!
//! Control logging via the `RUST_LOG` environment variable:
//!
//! ```bash
//! # Log all INFO and above
//! export RUST_LOG=info
//!
//! # Log only FFI audit events
//! export RUST_LOG=hedl_ffi::audit=debug
//!
//! # Log everything at DEBUG level
//! export RUST_LOG=debug
//! ```
//!
//! ## Example Output
//!
//! ```text
//! 2025-01-05T10:30:45.123Z INFO hedl_ffi::audit: FFI call started function="hedl_parse" thread_id=ThreadId(1) depth=0
//! 2025-01-05T10:30:45.125Z DEBUG hedl_ffi::audit: FFI call parameters function="hedl_parse" params=[("input_len", "1024")]
//! 2025-01-05T10:30:45.130Z INFO hedl_ffi::audit: FFI call completed function="hedl_parse" duration_ms=7.2 status="success"
//! ```
//!
//! See the [`audit`] module for more details on the logging implementation.

// =============================================================================
// Module Declarations
// =============================================================================

pub mod audit;
mod conversions;
mod diagnostics;
mod error;
mod memory;
mod operations;
mod parsing;
mod types;
mod utils;

// =============================================================================
// Re-exports
// =============================================================================

// Types and error codes
pub use types::{
    HedlDiagnostics, HedlDocument, HEDL_ERR_ALLOC, HEDL_ERR_CANONICALIZE, HEDL_ERR_CSV,
    HEDL_ERR_INVALID_UTF8, HEDL_ERR_JSON, HEDL_ERR_LINT, HEDL_ERR_NEO4J, HEDL_ERR_NULL_PTR,
    HEDL_ERR_PARQUET, HEDL_ERR_PARSE, HEDL_ERR_XML, HEDL_ERR_YAML, HEDL_OK,
};

// Error handling
pub use error::{
    hedl_clear_error_threadsafe, hedl_get_last_error, hedl_get_last_error_threadsafe,
};

// Memory management
pub use memory::{hedl_free_bytes, hedl_free_diagnostics, hedl_free_document, hedl_free_string};

// Parsing functions
pub use parsing::{
    hedl_alias_count, hedl_get_version, hedl_parse, hedl_root_item_count, hedl_schema_count,
    hedl_validate,
};

// Operations
pub use operations::{hedl_canonicalize, hedl_lint};

// Diagnostics
pub use diagnostics::{hedl_diagnostics_count, hedl_diagnostics_get, hedl_diagnostics_severity};

// Conversion functions (to_*)
#[cfg(feature = "json")]
pub use conversions::to_formats::hedl_to_json;

#[cfg(feature = "yaml")]
pub use conversions::to_formats::hedl_to_yaml;

#[cfg(feature = "xml")]
pub use conversions::to_formats::hedl_to_xml;

#[cfg(feature = "csv")]
pub use conversions::to_formats::hedl_to_csv;

#[cfg(feature = "parquet")]
pub use conversions::to_formats::hedl_to_parquet;

#[cfg(feature = "neo4j")]
pub use conversions::to_formats::hedl_to_neo4j_cypher;

// Zero-copy callback functions (to_*_callback)
pub use conversions::to_formats_callback::HedlOutputCallback;

#[cfg(feature = "json")]
pub use conversions::to_formats_callback::hedl_to_json_callback;

#[cfg(feature = "yaml")]
pub use conversions::to_formats_callback::hedl_to_yaml_callback;

#[cfg(feature = "xml")]
pub use conversions::to_formats_callback::hedl_to_xml_callback;

#[cfg(feature = "csv")]
pub use conversions::to_formats_callback::hedl_to_csv_callback;

#[cfg(feature = "neo4j")]
pub use conversions::to_formats_callback::hedl_to_neo4j_cypher_callback;

pub use conversions::to_formats_callback::hedl_canonicalize_callback;

// Conversion functions (from_*)
#[cfg(feature = "json")]
pub use conversions::from_formats::hedl_from_json;

#[cfg(feature = "yaml")]
pub use conversions::from_formats::hedl_from_yaml;

#[cfg(feature = "xml")]
pub use conversions::from_formats::hedl_from_xml;

#[cfg(feature = "parquet")]
pub use conversions::from_formats::hedl_from_parquet;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;
    use std::os::raw::c_char;
    use std::ptr;

    const VALID_HEDL: &[u8] = b"%VERSION: 1.0\n---\nkey: value\0";
    const INVALID_HEDL: &[u8] = b"not valid hedl\0";

    #[test]
    fn test_parse_and_free() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            let result = hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 1, &mut doc);

            assert_eq!(result, HEDL_OK);
            assert!(!doc.is_null());

            hedl_free_document(doc);
        }
    }

    #[test]
    fn test_validate_valid() {
        unsafe {
            assert_eq!(
                hedl_validate(VALID_HEDL.as_ptr() as *const c_char, -1, 1),
                HEDL_OK
            );
        }
    }

    #[test]
    fn test_validate_invalid() {
        unsafe {
            assert_ne!(
                hedl_validate(INVALID_HEDL.as_ptr() as *const c_char, -1, 1),
                HEDL_OK
            );
        }
    }

    #[test]
    fn test_null_ptr_handling() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            assert_eq!(hedl_parse(ptr::null(), -1, 0, &mut doc), HEDL_ERR_NULL_PTR);
        }
    }

    #[test]
    fn test_get_version() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

            let mut major: i32 = 0;
            let mut minor: i32 = 0;
            let result = hedl_get_version(doc, &mut major, &mut minor);

            assert_eq!(result, HEDL_OK);
            assert_eq!(major, 1);
            assert_eq!(minor, 0);

            hedl_free_document(doc);
        }
    }

    #[test]
    fn test_canonicalize() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_canonicalize(doc, &mut out_str);

            assert_eq!(result, HEDL_OK);
            assert!(!out_str.is_null());

            hedl_free_string(out_str);
            hedl_free_document(doc);
        }
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_to_json() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_json(doc, 0, &mut out_str);

            assert_eq!(result, HEDL_OK);
            assert!(!out_str.is_null());

            let json = CStr::from_ptr(out_str).to_str().unwrap();
            assert!(json.contains("key"));

            hedl_free_string(out_str);
            hedl_free_document(doc);
        }
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn test_to_yaml() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_yaml(doc, 0, &mut out_str);

            assert_eq!(result, HEDL_OK);
            assert!(!out_str.is_null());

            let yaml = CStr::from_ptr(out_str).to_str().unwrap();
            assert!(yaml.contains("key"));

            hedl_free_string(out_str);
            hedl_free_document(doc);
        }
    }

    #[cfg(feature = "xml")]
    #[test]
    fn test_to_xml() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_xml(doc, &mut out_str);

            assert_eq!(result, HEDL_OK);
            assert!(!out_str.is_null());

            let xml = CStr::from_ptr(out_str).to_str().unwrap();
            assert!(xml.contains("<?xml"));

            hedl_free_string(out_str);
            hedl_free_document(doc);
        }
    }

    #[test]
    fn test_lint() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

            let mut diag: *mut HedlDiagnostics = ptr::null_mut();
            let result = hedl_lint(doc, &mut diag);

            assert_eq!(result, HEDL_OK);
            assert!(!diag.is_null());

            let count = hedl_diagnostics_count(diag);
            assert!(count >= 0);

            hedl_free_diagnostics(diag);
            hedl_free_document(doc);
        }
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_from_json_roundtrip() {
        unsafe {
            // Parse original HEDL
            let mut doc1: *mut HedlDocument = ptr::null_mut();
            hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc1);

            // Convert to JSON
            let mut json_str: *mut c_char = ptr::null_mut();
            hedl_to_json(doc1, 1, &mut json_str);

            // Parse JSON back to HEDL
            let mut doc2: *mut HedlDocument = ptr::null_mut();
            let result = hedl_from_json(json_str, -1, &mut doc2);

            assert_eq!(result, HEDL_OK);
            assert!(!doc2.is_null());

            hedl_free_string(json_str);
            hedl_free_document(doc1);
            hedl_free_document(doc2);
        }
    }

    #[cfg(feature = "neo4j")]
    #[test]
    fn test_to_neo4j_cypher() {
        unsafe {
            let mut doc: *mut HedlDocument = ptr::null_mut();
            hedl_parse(VALID_HEDL.as_ptr() as *const c_char, -1, 0, &mut doc);

            let mut out_str: *mut c_char = ptr::null_mut();
            let result = hedl_to_neo4j_cypher(doc, 1, &mut out_str);

            assert_eq!(result, HEDL_OK);
            assert!(!out_str.is_null());

            // The simple key: value doc doesn't produce Cypher nodes,
            // but the function should succeed
            hedl_free_string(out_str);
            hedl_free_document(doc);
        }
    }
}
