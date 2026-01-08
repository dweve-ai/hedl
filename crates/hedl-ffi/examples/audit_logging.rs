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

//! Example demonstrating FFI audit logging configuration and usage.
//!
//! This example shows how to:
//! - Configure tracing subscriber for FFI audit logs
//! - Use different log levels to control verbosity
//! - Capture performance metrics from FFI calls
//! - Integrate audit logging in multi-threaded applications

use hedl_ffi::{
    hedl_canonicalize, hedl_free_document, hedl_free_string, hedl_get_last_error, hedl_parse,
    HedlDocument, HEDL_OK,
};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize tracing with custom configuration.
///
/// This function sets up a tracing subscriber that will capture all FFI
/// audit logs and format them for human-readable output.
fn init_audit_logging() {
    fmt()
        .with_env_filter(
            // Configure log levels:
            // - ERROR: Function failures only
            // - INFO: Function entry/exit with timing
            // - DEBUG: Detailed parameter information
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                EnvFilter::new("info")
                    // Set DEBUG level for FFI audit logs specifically
                    .add_directive("hedl_ffi::audit=debug".parse().unwrap())
            }),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(false)
        .init();
}

/// Example 1: Basic FFI call with audit logging.
///
/// This demonstrates how audit logs are automatically generated for
/// successful FFI operations.
fn example_successful_parse() {
    info!("=== Example 1: Successful Parse with Audit Logging ===");

    unsafe {
        let input = CString::new("%VERSION: 1.0\n---\nuser: Alice\nage: 30\n").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        info!("Calling hedl_parse...");
        let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        if result == HEDL_OK {
            info!("Parse successful!");
            hedl_free_document(doc);
        }
    }
}

/// Example 2: FFI call that fails, demonstrating error logging.
///
/// This shows how audit logs capture error details and timing even
/// when operations fail.
fn example_failed_parse() {
    info!("=== Example 2: Failed Parse with Error Logging ===");

    unsafe {
        let mut doc: *mut HedlDocument = ptr::null_mut();

        info!("Calling hedl_parse with NULL input (should fail)...");
        let result = hedl_parse(ptr::null(), -1, 0, &mut doc);

        if result != HEDL_OK {
            let err_msg = CStr::from_ptr(hedl_get_last_error());
            info!("Parse failed as expected: {:?}", err_msg);
        }
    }
}

/// Example 3: Multiple FFI calls showing performance metrics.
///
/// This demonstrates how audit logs track timing information for
/// performance analysis.
fn example_performance_tracking() {
    info!("=== Example 3: Performance Tracking ===");

    unsafe {
        let input = CString::new(
            "%VERSION: 1.0\n---\n\
             name: Test\n\
             items: [1, 2, 3, 4, 5]\n\
             data: { key1: value1, key2: value2 }\n",
        )
        .unwrap();

        let mut doc: *mut HedlDocument = ptr::null_mut();

        info!("Parsing document...");
        let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);

        if result == HEDL_OK {
            info!("Canonicalizing document...");
            let mut canonical: *mut c_char = ptr::null_mut();
            let result = hedl_canonicalize(doc, &mut canonical);

            if result == HEDL_OK {
                info!("Operations completed successfully");
                hedl_free_string(canonical);
            }

            hedl_free_document(doc);
        }
    }
}

/// Example 4: Multi-threaded FFI calls with audit logging.
///
/// This shows how audit logs maintain thread-local context and allow
/// tracking of concurrent operations independently.
fn example_multithreaded_operations() {
    info!("=== Example 4: Multi-threaded Operations ===");

    let threads: Vec<_> = (0..4)
        .map(|i| {
            std::thread::spawn(move || {
                unsafe {
                    let input = CString::new(format!(
                        "%VERSION: 1.0\n---\nthread: {}\ndata: test\n",
                        i
                    ))
                    .unwrap();

                    let mut doc: *mut HedlDocument = ptr::null_mut();

                    info!("Thread {} calling hedl_parse...", i);
                    let result = hedl_parse(input.as_ptr(), -1, 0, &mut doc);

                    if result == HEDL_OK {
                        info!("Thread {} parse successful", i);

                        // Simulate some work
                        std::thread::sleep(std::time::Duration::from_millis(10));

                        hedl_free_document(doc);
                    }
                }
            })
        })
        .collect();

    for thread in threads {
        thread.join().unwrap();
    }

    info!("All threads completed");
}

/// Example 5: Using environment variables to control logging.
///
/// This demonstrates how RUST_LOG environment variable can be used
/// to dynamically control audit log verbosity.
fn example_env_var_configuration() {
    info!("=== Example 5: Environment Variable Configuration ===");
    info!("Set RUST_LOG environment variable to control logging:");
    info!("  RUST_LOG=info                    - Standard logging");
    info!("  RUST_LOG=debug                   - Verbose logging");
    info!("  RUST_LOG=hedl_ffi::audit=debug   - FFI audit logs only");
    info!("  RUST_LOG=error                   - Errors only");

    unsafe {
        let input = CString::new("%VERSION: 1.0\n---\ntest: value\n").unwrap();
        let mut doc: *mut HedlDocument = ptr::null_mut();

        hedl_parse(input.as_ptr(), -1, 0, &mut doc);
        if !doc.is_null() {
            hedl_free_document(doc);
        }
    }
}

fn main() {
    // Initialize audit logging system
    init_audit_logging();

    info!("=== HEDL FFI Audit Logging Examples ===");
    info!("Demonstrating comprehensive audit logging for FFI operations");
    println!();

    // Run examples
    example_successful_parse();
    println!();

    example_failed_parse();
    println!();

    example_performance_tracking();
    println!();

    example_multithreaded_operations();
    println!();

    example_env_var_configuration();
    println!();

    info!("=== All Examples Completed ===");
}
