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

use crate::audit::{
    audit_call_failure, audit_call_start, audit_call_success, sanitize_pointer,
};
use crate::error::{clear_error, set_error};
use crate::memory::is_valid_document_ptr;
use crate::types::{
    HedlDocument, HEDL_ERR_CSV, HEDL_ERR_JSON, HEDL_ERR_NEO4J, HEDL_ERR_NULL_PTR,
    HEDL_ERR_PARQUET, HEDL_ERR_XML, HEDL_ERR_YAML, HEDL_OK,
};
use crate::utils::allocate_output_string;
use std::os::raw::{c_char, c_int};
use std::ptr;
use std::time::Instant;

// =============================================================================
// JSON Conversion (requires "json" feature)
// =============================================================================

/// Convert a HEDL document to JSON.
///
/// # Arguments
/// * `doc` - Document handle from hedl_parse
/// * `include_metadata` - Non-zero to include HEDL metadata (__type__, __schema__)
/// * `out_str` - Pointer to store JSON output (must be freed with hedl_free_string)
///
/// # Returns
/// HEDL_OK on success, error code on failure.
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

    if !is_valid_document_ptr(doc) || out_str.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure("hedl_to_json", HEDL_ERR_NULL_PTR, "Null pointer argument", duration);
        return HEDL_ERR_NULL_PTR;
    }

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
            let msg = format!("JSON conversion error: {}", e);
            set_error(&msg);
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
/// * `doc` - Document handle from hedl_parse
/// * `include_metadata` - Non-zero to include HEDL metadata
/// * `out_str` - Pointer to store YAML output (must be freed with hedl_free_string)
///
/// # Returns
/// HEDL_OK on success, error code on failure.
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

    if !is_valid_document_ptr(doc) || out_str.is_null() {
        let duration = start.elapsed();
        set_error("Null pointer argument");
        audit_call_failure("hedl_to_yaml", HEDL_ERR_NULL_PTR, "Null pointer argument", duration);
        return HEDL_ERR_NULL_PTR;
    }

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
            let msg = format!("YAML conversion error: {}", e);
            set_error(&msg);
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
/// * `doc` - Document handle from hedl_parse
/// * `out_str` - Pointer to store XML output (must be freed with hedl_free_string)
///
/// # Returns
/// HEDL_OK on success, error code on failure.
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
        audit_call_failure("hedl_to_xml", HEDL_ERR_NULL_PTR, "Null pointer argument", duration);
        return HEDL_ERR_NULL_PTR;
    }

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
            let msg = format!("XML conversion error: {}", e);
            set_error(&msg);
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
/// * `doc` - Document handle from hedl_parse
/// * `out_str` - Pointer to store CSV output (must be freed with hedl_free_string)
///
/// # Returns
/// HEDL_OK on success, error code on failure.
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
        audit_call_failure("hedl_to_csv", HEDL_ERR_NULL_PTR, "Null pointer argument", duration);
        return HEDL_ERR_NULL_PTR;
    }

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
            let msg = format!("CSV conversion error: {}", e);
            set_error(&msg);
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
/// * `doc` - Document handle from hedl_parse
/// * `out_data` - Pointer to store output data pointer
/// * `out_len` - Pointer to store output length
///
/// # Returns
/// HEDL_OK on success, error code on failure.
/// The output data must be freed with hedl_free_bytes.
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
        audit_call_failure("hedl_to_parquet", HEDL_ERR_NULL_PTR, "Null pointer argument", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;

    match hedl_parquet::to_parquet_bytes(doc_ref) {
        Ok(bytes) => {
            let len = bytes.len();
            let ptr = Box::into_raw(bytes.into_boxed_slice()) as *mut u8;
            *out_data = ptr;
            *out_len = len;
            audit_call_success("hedl_to_parquet", start.elapsed());
            HEDL_OK
        }
        Err(e) => {
            let duration = start.elapsed();
            let msg = format!("Parquet conversion error: {}", e);
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
/// * `doc` - Document handle from hedl_parse
/// * `use_merge` - Non-zero to use MERGE (idempotent), zero for CREATE
/// * `out_str` - Pointer to store Cypher output (must be freed with hedl_free_string)
///
/// # Returns
/// HEDL_OK on success, error code on failure.
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
        audit_call_failure("hedl_to_neo4j_cypher", HEDL_ERR_NULL_PTR, "Null pointer argument", duration);
        return HEDL_ERR_NULL_PTR;
    }

    let doc_ref = &(*doc).inner;
    let config = if use_merge != 0 {
        hedl_neo4j::ToCypherConfig::default()
    } else {
        hedl_neo4j::ToCypherConfig::new().with_create()
    };

    match hedl_neo4j::to_cypher(doc_ref, &config) {
        Ok(cypher) => {
            let result = allocate_output_string(&cypher.to_string(), out_str, HEDL_ERR_NEO4J);
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
            let msg = format!("Neo4j conversion error: {}", e);
            set_error(&msg);
            *out_str = ptr::null_mut();
            audit_call_failure("hedl_to_neo4j_cypher", HEDL_ERR_NEO4J, &msg, duration);
            HEDL_ERR_NEO4J
        }
    }
}
