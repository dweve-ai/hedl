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

//! Export functions (to_*) for FFI.

// Imports are feature-gated since all functions in this module require format features.
#[cfg(any(
    feature = "json",
    feature = "yaml",
    feature = "xml",
    feature = "csv",
    feature = "parquet",
    feature = "neo4j",
    feature = "toon"
))]
use crate::audit::{audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer};
#[cfg(any(
    feature = "json",
    feature = "yaml",
    feature = "xml",
    feature = "csv",
    feature = "parquet",
    feature = "neo4j",
    feature = "toon"
))]
use crate::error::{clear_error, set_error};
#[cfg(any(
    feature = "json",
    feature = "yaml",
    feature = "xml",
    feature = "csv",
    feature = "neo4j",
    feature = "toon"
))]
use crate::ffi_strings::allocate_output_string;
#[cfg(any(
    feature = "json",
    feature = "yaml",
    feature = "xml",
    feature = "csv",
    feature = "parquet",
    feature = "neo4j",
    feature = "toon"
))]
use crate::memory::is_valid_document_ptr;
#[cfg(feature = "csv")]
use crate::types::HEDL_ERR_CSV;
#[cfg(feature = "json")]
use crate::types::HEDL_ERR_JSON;
#[cfg(feature = "neo4j")]
use crate::types::HEDL_ERR_NEO4J;
#[cfg(feature = "parquet")]
use crate::types::HEDL_ERR_PARQUET;
#[cfg(feature = "toon")]
use crate::types::HEDL_ERR_TOON;
#[cfg(feature = "xml")]
use crate::types::HEDL_ERR_XML;
#[cfg(feature = "yaml")]
use crate::types::HEDL_ERR_YAML;
#[cfg(any(
    feature = "json",
    feature = "yaml",
    feature = "xml",
    feature = "csv",
    feature = "parquet",
    feature = "neo4j",
    feature = "toon"
))]
use crate::types::{HedlDocument, HEDL_ERR_NULL_PTR, HEDL_OK};
#[cfg(any(
    feature = "json",
    feature = "yaml",
    feature = "xml",
    feature = "csv",
    feature = "neo4j",
    feature = "toon"
))]
use std::os::raw::c_char;
#[cfg(any(
    feature = "json",
    feature = "yaml",
    feature = "xml",
    feature = "csv",
    feature = "parquet",
    feature = "neo4j",
    feature = "toon"
))]
use std::os::raw::c_int;
#[cfg(any(
    feature = "json",
    feature = "yaml",
    feature = "xml",
    feature = "csv",
    feature = "parquet",
    feature = "neo4j",
    feature = "toon"
))]
use std::ptr;
#[cfg(any(
    feature = "json",
    feature = "yaml",
    feature = "xml",
    feature = "csv",
    feature = "parquet",
    feature = "neo4j",
    feature = "toon"
))]
use std::time::Instant;

// =============================================================================
// JSON Conversion (requires "json" feature)
// =============================================================================

/// Convert a HEDL document to JSON.
///
/// # Arguments
/// * `doc` - Document handle from `hedl_parse`
/// * `include_metadata` - Non-zero to include HEDL metadata (__type__, __schema__)
/// * `out_str` - Pointer to store JSON output (must be freed with `hedl_free_string`)
///
/// # Returns
/// `HEDL_OK` on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "json" feature to be enabled.
#[cfg(feature = "json")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_json(
    doc: *const HedlDocument,
    include_metadata: c_int,
    out_str: *mut *mut c_char,
) -> c_int {
    let start = Instant::now();

    audit_call_start(
        "hedl_to_json",
        &[
            ("doc", &sanitize_pointer(doc)),
            ("include_metadata", &include_metadata.to_string()),
            ("out_str", &sanitize_pointer(out_str)),
        ],
    );

    clear_error();

    // Validate pointers before any dereference. `doc` must be non-null and refer
    // to a live HedlDocument created by this library; `out_str` must be a
    // non-null pointer to a writable C string pointer.
    if doc.is_null() || !is_valid_document_ptr(doc) || out_str.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure(
            "hedl_to_json",
            HEDL_ERR_NULL_PTR,
            "Null pointer argument",
            duration,
        );
        return HEDL_ERR_NULL_PTR;
    }

    // SAFETY: We validated the pointer is non-null and not poisoned.
    // The document was allocated by Box::into_raw in hedl_parse.
    let doc_ref = &(*doc).inner;
    let config = hedl_json::ToJsonConfig {
        include_metadata: include_metadata != 0,
        ..Default::default()
    };

    match hedl_json::to_json(doc_ref, &config) {
        Ok(json) => {
            let result = allocate_output_string(&json, out_str, HEDL_ERR_JSON);
            if result == HEDL_OK {
                audit_call_success("hedl_to_json", start.elapsed());
            } else {
                let duration = start.elapsed();
                let msg = crate::error::get_thread_local_error();
                audit_call_failure("hedl_to_json", result, &msg, duration);
            }
            result
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("JSON conversion error: {e}");
            set_error(&msg);
            // SAFETY: We validated out_str is non-null above.
            *out_str = ptr::null_mut();
            audit_call_failure("hedl_to_json", HEDL_ERR_JSON, &msg, duration);
            HEDL_ERR_JSON
        }
    }
}

// =============================================================================
// YAML Conversion (requires "yaml" feature)
// =============================================================================

/// Convert a HEDL document to YAML.
///
/// # Arguments
/// * `doc` - Document handle from `hedl_parse`
/// * `include_metadata` - Non-zero to include HEDL metadata
/// * `out_str` - Pointer to store YAML output (must be freed with `hedl_free_string`)
///
/// # Returns
/// `HEDL_OK` on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "yaml" feature to be enabled.
#[cfg(feature = "yaml")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_yaml(
    doc: *const HedlDocument,
    include_metadata: c_int,
    out_str: *mut *mut c_char,
) -> c_int {
    let start = Instant::now();

    audit_call_start(
        "hedl_to_yaml",
        &[
            ("doc", &sanitize_pointer(doc)),
            ("include_metadata", &include_metadata.to_string()),
            ("out_str", &sanitize_pointer(out_str)),
        ],
    );

    clear_error();

    // Validate basic pointer arguments before any dereference.
    if doc.is_null() || out_str.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure(
            "hedl_to_yaml",
            HEDL_ERR_NULL_PTR,
            "Null pointer argument",
            duration,
        );
        return HEDL_ERR_NULL_PTR;
    }

    // Ensure the document pointer refers to a valid, non-poisoned HEDL document
    // before we dereference it.
    if !is_valid_document_ptr(doc) {
        let duration = start.elapsed();
        set_error("Invalid document handle");
        audit_call_failure(
            "hedl_to_yaml",
            HEDL_ERR_NULL_PTR,
            "Invalid document handle",
            duration,
        );
        return HEDL_ERR_NULL_PTR;
    }

    // SAFETY: We validated the pointer is non-null and not poisoned via
    // is_valid_document_ptr. The document was allocated by Box::into_raw in hedl_parse.
    let doc_ref = &(*doc).inner;
    let config = hedl_yaml::ToYamlConfig {
        include_metadata: include_metadata != 0,
        ..Default::default()
    };

    match hedl_yaml::to_yaml(doc_ref, &config) {
        Ok(yaml) => {
            let result = allocate_output_string(&yaml, out_str, HEDL_ERR_YAML);
            if result == HEDL_OK {
                audit_call_success("hedl_to_yaml", start.elapsed());
            } else {
                let duration = start.elapsed();
                let msg = crate::error::get_thread_local_error();
                audit_call_failure("hedl_to_yaml", result, &msg, duration);
            }
            result
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("YAML conversion error: {e}");
            set_error(&msg);
            // SAFETY: We validated out_str is non-null above.
            *out_str = ptr::null_mut();
            audit_call_failure("hedl_to_yaml", HEDL_ERR_YAML, &msg, duration);
            HEDL_ERR_YAML
        }
    }
}

// =============================================================================
// XML Conversion (requires "xml" feature)
// =============================================================================

/// Convert a HEDL document to XML.
///
/// # Arguments
/// * `doc` - Document handle from `hedl_parse`
/// * `out_str` - Pointer to store XML output (must be freed with `hedl_free_string`)
///
/// # Returns
/// `HEDL_OK` on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "xml" feature to be enabled.
#[cfg(feature = "xml")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_xml(doc: *const HedlDocument, out_str: *mut *mut c_char) -> c_int {
    let start = Instant::now();

    audit_call_start(
        "hedl_to_xml",
        &[
            ("doc", &sanitize_pointer(doc)),
            ("out_str", &sanitize_pointer(out_str)),
        ],
    );

    clear_error();

    if !is_valid_document_ptr(doc) || out_str.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure(
            "hedl_to_xml",
            HEDL_ERR_NULL_PTR,
            "Null pointer argument",
            duration,
        );
        return HEDL_ERR_NULL_PTR;
    }

    // SAFETY: We validated the pointer is non-null and not poisoned.
    // The document was allocated by Box::into_raw in hedl_parse.
    let doc_ref = &(*doc).inner;

    match hedl_xml::hedl_to_xml(doc_ref) {
        Ok(xml) => {
            let result = allocate_output_string(&xml, out_str, HEDL_ERR_XML);
            if result == HEDL_OK {
                audit_call_success("hedl_to_xml", start.elapsed());
            } else {
                let duration = start.elapsed();
                let msg = crate::error::get_thread_local_error();
                audit_call_failure("hedl_to_xml", result, &msg, duration);
            }
            result
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("XML conversion error: {e}");
            set_error(&msg);
            // SAFETY: We validated out_str is non-null above.
            *out_str = ptr::null_mut();
            audit_call_failure("hedl_to_xml", HEDL_ERR_XML, &msg, duration);
            HEDL_ERR_XML
        }
    }
}

// =============================================================================
// CSV Conversion (requires "csv" feature)
// =============================================================================

/// Convert a HEDL document to CSV.
///
/// Note: Only works for documents with matrix lists.
///
/// # Arguments
/// * `doc` - Document handle from `hedl_parse`
/// * `out_str` - Pointer to store CSV output (must be freed with `hedl_free_string`)
///
/// # Returns
/// `HEDL_OK` on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "csv" feature to be enabled.
#[cfg(feature = "csv")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_csv(doc: *const HedlDocument, out_str: *mut *mut c_char) -> c_int {
    let start = Instant::now();

    audit_call_start(
        "hedl_to_csv",
        &[
            ("doc", &sanitize_pointer(doc)),
            ("out_str", &sanitize_pointer(out_str)),
        ],
    );

    clear_error();

    if !is_valid_document_ptr(doc) || out_str.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure(
            "hedl_to_csv",
            HEDL_ERR_NULL_PTR,
            "Null pointer argument",
            duration,
        );
        return HEDL_ERR_NULL_PTR;
    }

    // SAFETY: We validated the pointer is non-null and not poisoned.
    // The document was allocated by Box::into_raw in hedl_parse.
    let doc_ref = &(*doc).inner;

    match hedl_csv::to_csv(doc_ref) {
        Ok(csv) => {
            let result = allocate_output_string(&csv, out_str, HEDL_ERR_CSV);
            if result == HEDL_OK {
                audit_call_success("hedl_to_csv", start.elapsed());
            } else {
                let duration = start.elapsed();
                let msg = crate::error::get_thread_local_error();
                audit_call_failure("hedl_to_csv", result, &msg, duration);
            }
            result
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("CSV conversion error: {e}");
            set_error(&msg);
            // SAFETY: We validated out_str is non-null above.
            *out_str = ptr::null_mut();
            audit_call_failure("hedl_to_csv", HEDL_ERR_CSV, &msg, duration);
            HEDL_ERR_CSV
        }
    }
}

// =============================================================================
// Parquet Conversion (requires "parquet" feature)
// =============================================================================

/// Convert a HEDL document to Parquet bytes.
///
/// Note: Only works for documents with matrix lists.
///
/// # Arguments
/// * `doc` - Document handle from `hedl_parse`
/// * `out_data` - Pointer to store output data pointer
/// * `out_len` - Pointer to store output length
///
/// # Returns
/// `HEDL_OK` on success, error code on failure.
/// The output data must be freed with `hedl_free_bytes`.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "parquet" feature to be enabled.
#[cfg(feature = "parquet")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_parquet(
    doc: *const HedlDocument,
    out_data: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    let start = Instant::now();

    audit_call_start(
        "hedl_to_parquet",
        &[
            ("doc", &sanitize_pointer(doc)),
            ("out_data", &sanitize_pointer(out_data)),
            ("out_len", &sanitize_pointer(out_len)),
        ],
    );

    clear_error();

    if !is_valid_document_ptr(doc) || out_data.is_null() || out_len.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure(
            "hedl_to_parquet",
            HEDL_ERR_NULL_PTR,
            "Null pointer argument",
            duration,
        );
        return HEDL_ERR_NULL_PTR;
    }

    // SAFETY: We validated the pointer is non-null and not poisoned.
    // The document was allocated by Box::into_raw in hedl_parse.
    let doc_ref = &(*doc).inner;

    match hedl_parquet::to_parquet_bytes(doc_ref) {
        Ok(bytes) => {
            let len = bytes.len();
            let ptr = Box::into_raw(bytes.into_boxed_slice()).cast::<u8>();
            *out_data = ptr;
            *out_len = len;
            audit_call_success("hedl_to_parquet", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("Parquet conversion error: {e}");
            set_error(&msg);
            *out_data = ptr::null_mut();
            *out_len = 0;
            audit_call_failure("hedl_to_parquet", HEDL_ERR_PARQUET, &msg, duration);
            HEDL_ERR_PARQUET
        }
    }
}

// =============================================================================
// Neo4j/Cypher Conversion (requires "neo4j" feature)
// =============================================================================

/// Convert a HEDL document to Cypher queries for Neo4j.
///
/// Generates CREATE/MERGE statements, constraints, and relationships.
///
/// # Arguments
/// * `doc` - Document handle from `hedl_parse`
/// * `use_merge` - Non-zero to use MERGE (idempotent), zero for CREATE
/// * `out_str` - Pointer to store Cypher output (must be freed with `hedl_free_string`)
///
/// # Returns
/// `HEDL_OK` on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "neo4j" feature to be enabled.
#[cfg(feature = "neo4j")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_neo4j_cypher(
    doc: *const HedlDocument,
    use_merge: c_int,
    out_str: *mut *mut c_char,
) -> c_int {
    let start = Instant::now();

    audit_call_start(
        "hedl_to_neo4j_cypher",
        &[
            ("doc", &sanitize_pointer(doc)),
            ("use_merge", &use_merge.to_string()),
            ("out_str", &sanitize_pointer(out_str)),
        ],
    );

    clear_error();

    if !is_valid_document_ptr(doc) || out_str.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure(
            "hedl_to_neo4j_cypher",
            HEDL_ERR_NULL_PTR,
            "Null pointer argument",
            duration,
        );
        return HEDL_ERR_NULL_PTR;
    }

    // SAFETY: We validated the pointer is non-null and not poisoned.
    // The document was allocated by Box::into_raw in hedl_parse.
    let doc_ref = &(*doc).inner;
    let config = if use_merge != 0 {
        hedl_neo4j::ToCypherConfig::default()
    } else {
        hedl_neo4j::ToCypherConfig::new().with_create()
    };

    match hedl_neo4j::to_cypher(doc_ref, &config) {
        Ok(cypher) => {
            let result = allocate_output_string(&cypher.clone(), out_str, HEDL_ERR_NEO4J);
            if result == HEDL_OK {
                audit_call_success("hedl_to_neo4j_cypher", start.elapsed());
            } else {
                let duration = start.elapsed();
                let msg = crate::error::get_thread_local_error();
                audit_call_failure("hedl_to_neo4j_cypher", result, &msg, duration);
            }
            result
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("Neo4j conversion error: {e}");
            set_error(&msg);
            // SAFETY: We validated out_str is non-null above.
            *out_str = ptr::null_mut();
            audit_call_failure("hedl_to_neo4j_cypher", HEDL_ERR_NEO4J, &msg, duration);
            HEDL_ERR_NEO4J
        }
    }
}

// =============================================================================
// TOON Conversion (requires "toon" feature)
// =============================================================================

/// Convert a HEDL document to TOON format.
///
/// TOON (Typed Object Outline Notation) is an external format specification
/// for human-readable data serialization.
///
/// # Arguments
/// * `doc` - Document handle from `hedl_parse`
/// * `out_str` - Pointer to store TOON output (must be freed with `hedl_free_string`)
///
/// # Returns
/// `HEDL_OK` on success, error code on failure.
///
/// # Safety
/// All pointers must be valid.
///
/// # Feature
/// Requires the "toon" feature to be enabled.
#[cfg(feature = "toon")]
#[no_mangle]
pub unsafe extern "C" fn hedl_to_toon(
    doc: *const HedlDocument,
    out_str: *mut *mut c_char,
) -> c_int {
    let start = Instant::now();

    audit_call_start(
        "hedl_to_toon",
        &[
            ("doc", &sanitize_pointer(doc)),
            ("out_str", &sanitize_pointer(out_str)),
        ],
    );

    clear_error();

    if !is_valid_document_ptr(doc) || out_str.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure(
            "hedl_to_toon",
            HEDL_ERR_NULL_PTR,
            "Null pointer argument",
            duration,
        );
        return HEDL_ERR_NULL_PTR;
    }

    // SAFETY: We validated the pointer is non-null and not poisoned.
    // The document was allocated by Box::into_raw in hedl_parse.
    let doc_ref = &(*doc).inner;

    match hedl_toon::hedl_to_toon(doc_ref) {
        Ok(toon) => {
            let result = allocate_output_string(&toon, out_str, HEDL_ERR_TOON);
            if result == HEDL_OK {
                audit_call_success("hedl_to_toon", start.elapsed());
            } else {
                let duration = start.elapsed();
                let msg = crate::error::get_thread_local_error();
                audit_call_failure("hedl_to_toon", result, &msg, duration);
            }
            result
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("TOON conversion error: {e}");
            set_error(&msg);
            // SAFETY: We validated out_str is non-null above.
            *out_str = ptr::null_mut();
            audit_call_failure("hedl_to_toon", HEDL_ERR_TOON, &msg, duration);
            HEDL_ERR_TOON
        }
    }
}
