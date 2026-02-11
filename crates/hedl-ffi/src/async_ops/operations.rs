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

//! Operation execution logic for async operations.

use super::thread_pool::WorkQueue;
use super::types::{HedlCompletionCallbackFn, Operation, WorkItem};
use crate::types::{HedlDiagnostics, HedlDocument};
use crate::types::{HEDL_ERR_CANCELLED, HEDL_ERR_CANONICALIZE, HEDL_ERR_PARSE, HEDL_OK};
use hedl_core::{parse_with_limits, Document, ParseOptions, ReferenceMode};
use std::ffi::{c_char, c_void, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

// =============================================================================
// Worker Implementation
// =============================================================================

/// Worker thread wrapper.
pub(crate) struct Worker {
    pub(crate) thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new worker thread.
    pub(crate) fn new(
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
                let mut queue = work_queue.lock().expect("lock not poisoned");

                // Wait for work or shutdown signal
                while queue.queue.is_empty() && !shutdown.load(Ordering::Acquire) {
                    queue = condvar.wait(queue).expect("lock not poisoned");
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
            // SAFETY: Callback function pointer is valid and was verified non-null at submission.
            // `user_data` pointer is owned by caller and must remain valid until callback completes.
            // SAFETY: FFI function requires raw pointer for output parameter
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
            // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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

                // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
                unsafe {
                    (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                }
            }
            Err(e) => {
                let error_msg = format!("Parse error: {e}\0");
                let c_error = error_msg.as_ptr().cast::<c_char>();

                // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
            // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in canonical output";
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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

                // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
            // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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

        // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
            // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in JSON output";
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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

                // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
            // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in YAML output";
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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

                // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
            // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in XML output";
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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

                // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
            // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in CSV output";
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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

                // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
            // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in Cypher output";
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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

                // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
            // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
                    unsafe {
                        (callback)(HEDL_OK, result_ptr, std::ptr::null(), user_data);
                    }
                } else {
                    let error_msg = c"Invalid UTF-8 in TOON output";
                    // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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

                // SAFETY: Pointer is valid and non-null, checked by caller or validation function.
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
