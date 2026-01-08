/*
 * Dweve HEDL - Hierarchical Entity Data Language
 *
 * Copyright (c) 2025 Dweve IP B.V. and individual contributors.
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at:
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * @file hedl.h
 * @brief HEDL C FFI Interface
 *
 * This header provides C bindings for the HEDL (Hierarchical Entity Data Language)
 * parser and converter library.
 *
 * # Memory Management
 *
 * **IMPORTANT:** Memory ownership follows strict rules:
 *
 * - Strings returned by hedl_* functions MUST be freed with hedl_free_string()
 * - Byte arrays returned by hedl_to_parquet() MUST be freed with hedl_free_bytes()
 * - Documents MUST be freed with hedl_free_document()
 * - Diagnostics MUST be freed with hedl_free_diagnostics()
 *
 * **WARNING:** The hedl_free_* functions ONLY accept pointers allocated by HEDL.
 * Passing pointers from malloc/calloc or other libraries causes undefined behavior.
 * NULL is safe and will be ignored.
 *
 * # Thread Safety
 *
 * Error messages are stored in thread-local storage. Each thread maintains its
 * own error state independently:
 *
 * - hedl_get_last_error() returns the error for the CALLING thread only
 * - Errors from one thread will NOT overwrite errors in another thread
 * - You MUST call hedl_get_last_error() from the same thread that received the error
 *
 * Document handles (HedlDocument*) are NOT thread-safe. Do not share document
 * handles between threads without external synchronization.
 *
 * # Error Handling
 *
 * All functions return error codes (HEDL_OK = 0 on success, negative on failure).
 * Use hedl_get_last_error() to get detailed error messages.
 *
 * # Example Usage
 *
 * @code{.c}
 * #include "hedl.h"
 * #include <stdio.h>
 * #include <stdlib.h>
 *
 * int main() {
 *     const char* hedl_source = "%VERSION: 1.0
---
name: Alice
age: 30
";
 *     HedlDocument* doc = NULL;
 *
 *     // Parse HEDL document
 *     int result = hedl_parse(hedl_source, -1, 1, &doc);
 *     if (result != HEDL_OK) {
 *         fprintf(stderr, "Parse error: %s
", hedl_get_last_error());
 *         return 1;
 *     }
 *
 *     // Convert to JSON
 *     char* json = NULL;
 *     result = hedl_to_json(doc, 0, &json);
 *     if (result == HEDL_OK) {
 *         printf("%s
", json);
 *         hedl_free_string(json);
 *     }
 *
 *     // Clean up
 *     hedl_free_document(doc);
 *     return 0;
 * }
 * @endcode
 */


#ifndef HEDL_H
#define HEDL_H

#pragma once

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
namespace hedl {
#endif  // __cplusplus

#define HEDL_OK 0

#define HEDL_ERR_NULL_PTR -1

#define HEDL_ERR_INVALID_UTF8 -2

#define HEDL_ERR_PARSE -3

#define HEDL_ERR_CANONICALIZE -4

#define HEDL_ERR_JSON -5

#define HEDL_ERR_ALLOC -6

#define HEDL_ERR_YAML -7

#define HEDL_ERR_XML -8

#define HEDL_ERR_CSV -9

#define HEDL_ERR_PARQUET -10

#define HEDL_ERR_LINT -11

#define HEDL_ERR_NEO4J -12

/*
 Opaque handle to lint diagnostics
 */
typedef struct HedlDiagnostics HedlDiagnostics;

/*
 Opaque handle to a HEDL document
 */
typedef struct HedlDocument HedlDocument;

/*
 Output callback function type for zero-copy string return.

 # Safety
 - The `data` pointer is only valid during the callback execution
 - Do NOT store the pointer for later use
 - The data is not null-terminated
 - The callback MUST NOT call back into HEDL functions
 */
typedef void (*HedlOutputCallback)(const char *data, uintptr_t len, void *user_data);

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/*
 Parse JSON into a HEDL document.

 # Arguments
 * `json` - UTF-8 encoded JSON string
 * `json_len` - Length of input in bytes, or -1 for null-terminated
 * `out_doc` - Pointer to store document handle

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid.

 # Feature
 Requires the "json" feature to be enabled.
 */
int hedl_from_json(const char *json, int json_len, struct HedlDocument **out_doc);

/*
 Parse YAML into a HEDL document.

 # Arguments
 * `yaml` - UTF-8 encoded YAML string
 * `yaml_len` - Length of input in bytes, or -1 for null-terminated
 * `out_doc` - Pointer to store document handle

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid.

 # Feature
 Requires the "yaml" feature to be enabled.
 */
int hedl_from_yaml(const char *yaml, int yaml_len, struct HedlDocument **out_doc);

/*
 Parse XML into a HEDL document.

 # Arguments
 * `xml` - UTF-8 encoded XML string
 * `xml_len` - Length of input in bytes, or -1 for null-terminated
 * `out_doc` - Pointer to store document handle

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid.

 # Feature
 Requires the "xml" feature to be enabled.
 */
int hedl_from_xml(const char *xml, int xml_len, struct HedlDocument **out_doc);

/*
 Parse Parquet bytes into a HEDL document.

 # Arguments
 * `data` - Parquet file bytes
 * `len` - Length of data
 * `out_doc` - Pointer to store document handle

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid.

 # Feature
 Requires the "parquet" feature to be enabled.
 */
int hedl_from_parquet(const uint8_t *data, uintptr_t len, struct HedlDocument **out_doc);

/*
 Convert a HEDL document to JSON.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `include_metadata` - Non-zero to include HEDL metadata (__type__, __schema__)
 * `out_str` - Pointer to store JSON output (must be freed with hedl_free_string)

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid.

 # Feature
 Requires the "json" feature to be enabled.
 */
int hedl_to_json(const struct HedlDocument *doc, int include_metadata, char **out_str);

/*
 Convert a HEDL document to YAML.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `include_metadata` - Non-zero to include HEDL metadata
 * `out_str` - Pointer to store YAML output (must be freed with hedl_free_string)

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid.

 # Feature
 Requires the "yaml" feature to be enabled.
 */
int hedl_to_yaml(const struct HedlDocument *doc, int include_metadata, char **out_str);

/*
 Convert a HEDL document to XML.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `out_str` - Pointer to store XML output (must be freed with hedl_free_string)

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid.

 # Feature
 Requires the "xml" feature to be enabled.
 */
int hedl_to_xml(const struct HedlDocument *doc, char **out_str);

/*
 Convert a HEDL document to CSV.

 Note: Only works for documents with matrix lists.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `out_str` - Pointer to store CSV output (must be freed with hedl_free_string)

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid.

 # Feature
 Requires the "csv" feature to be enabled.
 */
int hedl_to_csv(const struct HedlDocument *doc, char **out_str);

/*
 Convert a HEDL document to Parquet bytes.

 Note: Only works for documents with matrix lists.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `out_data` - Pointer to store output data pointer
 * `out_len` - Pointer to store output length

 # Returns
 HEDL_OK on success, error code on failure.
 The output data must be freed with hedl_free_bytes.

 # Safety
 All pointers must be valid.

 # Feature
 Requires the "parquet" feature to be enabled.
 */
int hedl_to_parquet(const struct HedlDocument *doc, uint8_t **out_data, uintptr_t *out_len);

/*
 Convert a HEDL document to Cypher queries for Neo4j.

 Generates CREATE/MERGE statements, constraints, and relationships.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `use_merge` - Non-zero to use MERGE (idempotent), zero for CREATE
 * `out_str` - Pointer to store Cypher output (must be freed with hedl_free_string)

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid.

 # Feature
 Requires the "neo4j" feature to be enabled.
 */
int hedl_to_neo4j_cypher(const struct HedlDocument *doc, int use_merge, char **out_str);

/*
 Convert a HEDL document to JSON using zero-copy callback pattern.

 For outputs >1MB, this avoids memory allocation by passing data directly
 to the callback. For smaller outputs, consider using `hedl_to_json`.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `include_metadata` - Non-zero to include HEDL metadata (__type__, __schema__)
 * `callback` - Function to receive the output data
 * `user_data` - User context pointer passed to callback

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 - All pointers must be valid
 - The callback MUST NOT call back into HEDL functions
 - The data pointer passed to callback is only valid during the callback

 # Feature
 Requires the "json" feature to be enabled.
 */
int hedl_to_json_callback(const struct HedlDocument *doc,
                          int include_metadata,
                          HedlOutputCallback callback,
                          void *user_data);

/*
 Convert a HEDL document to YAML using zero-copy callback pattern.

 For outputs >1MB, this avoids memory allocation by passing data directly
 to the callback. For smaller outputs, consider using `hedl_to_yaml`.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `include_metadata` - Non-zero to include HEDL metadata
 * `callback` - Function to receive the output data
 * `user_data` - User context pointer passed to callback

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 - All pointers must be valid
 - The callback MUST NOT call back into HEDL functions
 - The data pointer passed to callback is only valid during the callback

 # Feature
 Requires the "yaml" feature to be enabled.
 */
int hedl_to_yaml_callback(const struct HedlDocument *doc,
                          int include_metadata,
                          HedlOutputCallback callback,
                          void *user_data);

/*
 Convert a HEDL document to XML using zero-copy callback pattern.

 For outputs >1MB, this avoids memory allocation by passing data directly
 to the callback. For smaller outputs, consider using `hedl_to_xml`.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `callback` - Function to receive the output data
 * `user_data` - User context pointer passed to callback

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 - All pointers must be valid
 - The callback MUST NOT call back into HEDL functions
 - The data pointer passed to callback is only valid during the callback

 # Feature
 Requires the "xml" feature to be enabled.
 */
int hedl_to_xml_callback(const struct HedlDocument *doc,
                         HedlOutputCallback callback,
                         void *user_data);

/*
 Convert a HEDL document to CSV using zero-copy callback pattern.

 Note: Only works for documents with matrix lists.

 For outputs >1MB, this avoids memory allocation by passing data directly
 to the callback. For smaller outputs, consider using `hedl_to_csv`.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `callback` - Function to receive the output data
 * `user_data` - User context pointer passed to callback

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 - All pointers must be valid
 - The callback MUST NOT call back into HEDL functions
 - The data pointer passed to callback is only valid during the callback

 # Feature
 Requires the "csv" feature to be enabled.
 */
int hedl_to_csv_callback(const struct HedlDocument *doc,
                         HedlOutputCallback callback,
                         void *user_data);

/*
 Convert a HEDL document to Cypher queries using zero-copy callback pattern.

 Generates CREATE/MERGE statements, constraints, and relationships.

 For outputs >1MB, this avoids memory allocation by passing data directly
 to the callback. For smaller outputs, consider using `hedl_to_neo4j_cypher`.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `use_merge` - Non-zero to use MERGE (idempotent), zero for CREATE
 * `callback` - Function to receive the output data
 * `user_data` - User context pointer passed to callback

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 - All pointers must be valid
 - The callback MUST NOT call back into HEDL functions
 - The data pointer passed to callback is only valid during the callback

 # Feature
 Requires the "neo4j" feature to be enabled.
 */
int hedl_to_neo4j_cypher_callback(const struct HedlDocument *doc,
                                  int use_merge,
                                  HedlOutputCallback callback,
                                  void *user_data);

/*
 Canonicalize a HEDL document using zero-copy callback pattern.

 For outputs >1MB, this avoids memory allocation by passing data directly
 to the callback. For smaller outputs, consider using `hedl_canonicalize`.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `callback` - Function to receive the output data
 * `user_data` - User context pointer passed to callback

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 - All pointers must be valid
 - The callback MUST NOT call back into HEDL functions
 - The data pointer passed to callback is only valid during the callback
 */
int hedl_canonicalize_callback(const struct HedlDocument *doc,
                               HedlOutputCallback callback,
                               void *user_data);

/*
 Get the number of diagnostics.

 # Safety
 Pointer must be valid. Returns -1 if diag is NULL or poisoned.
 */
int hedl_diagnostics_count(const struct HedlDiagnostics *diag);

/*
 Get a diagnostic message.

 # Arguments
 * `diag` - Diagnostics handle
 * `index` - Diagnostic index
 * `out_str` - Pointer to store message (must be freed with hedl_free_string)

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid. Returns HEDL_ERR_NULL_PTR if diag is NULL or poisoned.
 */
int hedl_diagnostics_get(const struct HedlDiagnostics *diag, int index, char **out_str);

/*
 Get a diagnostic severity (0=Hint, 1=Warning, 2=Error).

 # Safety
 Pointer must be valid. Returns -1 if diag is NULL or poisoned.
 */
int hedl_diagnostics_severity(const struct HedlDiagnostics *diag, int index);

/*
 Get the last error message for the current thread.

 Returns NULL if no error occurred on this thread.

 # Thread Safety

 Error messages are stored in thread-local storage. This function returns
 the error message for the CALLING THREAD ONLY. You must call this function
 from the same thread that received the error code.

 Each thread maintains its own independent error state. Errors in one thread
 do not affect or overwrite errors in other threads. This makes the FFI safe
 to use in multi-threaded applications without external synchronization for
 error handling.

 # Lifetime

 The returned pointer is valid until the next `hedl_*` call on this thread.
 Copy the string immediately if you need to preserve it.

 # Example (C)

 ```c
 // Thread 1
 if (hedl_parse(input1, -1, 0, &doc1) != HEDL_OK) {
     const char* err1 = hedl_get_last_error();
     printf("Thread 1 error: %s\n", err1);
 }

 // Thread 2 (concurrent with Thread 1)
 if (hedl_parse(input2, -1, 0, &doc2) != HEDL_OK) {
     const char* err2 = hedl_get_last_error();
     printf("Thread 2 error: %s\n", err2);
 }
 // err1 and err2 are independent - no cross-thread pollution
 ```
 */
const char *hedl_get_last_error(void);

/*
 Clear the last error for the current thread.

 This function explicitly clears any error message stored for the calling thread.
 It is generally not necessary to call this function, as successful operations
 automatically clear errors. However, it can be useful in specific scenarios:

 - Testing error handling logic
 - Clearing stale errors before a sequence of operations
 - Resetting error state in long-running thread pools

 # Thread Safety

 Like `hedl_get_last_error()`, this function operates on thread-local storage
 and only affects the error state of the calling thread. Other threads' error
 states remain unchanged.

 # Example (C)

 ```c
 // Clear any previous errors
 hedl_clear_error_threadsafe();

 // Now perform operations with a clean error state
 if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
     // This error is definitely from hedl_parse, not a previous operation
     const char* err = hedl_get_last_error();
     handle_error(err);
 }
 ```
 */
void hedl_clear_error_threadsafe(void);

/*
 Get the last error message for the current thread (thread-safe variant).

 This is an explicitly named alias of `hedl_get_last_error()` to make the
 thread-safety guarantee clear in the function name.

 Returns NULL if no error occurred on this thread.

 # Thread Safety

 This function is fully thread-safe. Each thread maintains its own independent
 error state in thread-local storage. Concurrent calls from multiple threads
 will not interfere with each other.

 **Guarantees:**
 - Errors from thread A will never appear in thread B
 - No synchronization primitives (mutexes, locks) are required
 - Zero contention between threads accessing errors
 - Lock-free, wait-free operation

 # Lifetime

 The returned pointer is valid until the next `hedl_*` call on this thread.
 Copy the string immediately if you need to preserve it.

 # Example (C with pthreads)

 ```c
 void* worker_thread(void* arg) {
     HedlDocument* doc = NULL;
     const char* input = (const char*)arg;

     if (hedl_parse(input, -1, 0, &doc) != HEDL_OK) {
         // Get error for THIS thread only
         const char* err = hedl_get_last_error_threadsafe();
         fprintf(stderr, "Worker thread error: %s\n", err);
         return NULL;
     }

     // Process document...
     hedl_free_document(doc);
     return (void*)1;
 }

 int main() {
     pthread_t threads[4];
     const char* inputs[4] = { input1, input2, input3, input4 };

     for (int i = 0; i < 4; i++) {
         pthread_create(&threads[i], NULL, worker_thread, (void*)inputs[i]);
     }

     for (int i = 0; i < 4; i++) {
         pthread_join(threads[i], NULL);
     }
 }
 ```
 */
const char *hedl_get_last_error_threadsafe(void);

/*
 Free a string allocated by HEDL functions.

 # Safety

 **CRITICAL:** The pointer MUST have been returned by a `hedl_*` function
 that allocates strings (e.g., `hedl_to_json`, `hedl_canonicalize`).

 **Undefined behavior will occur if you pass:**
 - Pointers from `malloc`/`calloc`/`realloc`
 - Stack-allocated strings
 - Already-freed pointers (double free)
 - Pointers from other libraries

 NULL pointers are safely ignored.
 */
void hedl_free_string(char *s);

/*
 Free a document handle.

 # Safety

 The pointer must have been returned by hedl_parse or hedl_from_*.

 **Double-free protection:** If the pointer is NULL or the poison value,
 this function returns safely without attempting to free.

 **Note**: Since C passes pointers by value, we cannot modify the caller's
 pointer. Callers should manually set their pointers to NULL after freeing
 to avoid use-after-free bugs.
 */
void hedl_free_document(struct HedlDocument *doc);

/*
 Free a diagnostics handle.

 # Safety

 The pointer must have been returned by hedl_lint.

 **Double-free protection:** If the pointer is NULL or the poison value,
 this function returns safely without attempting to free.

 **Note**: Since C passes pointers by value, we cannot modify the caller's
 pointer. Callers should manually set their pointers to NULL after freeing
 to avoid use-after-free bugs.
 */
void hedl_free_diagnostics(struct HedlDiagnostics *diag);

/*
 Free byte array allocated by HEDL functions (e.g., `hedl_to_parquet`).

 # Arguments
 * `data` - Pointer to the byte array
 * `len` - Length that was returned with the data (MUST match exactly)

 # Safety

 **CRITICAL:** The pointer MUST have been returned by a `hedl_*` function
 that allocates byte arrays (e.g., `hedl_to_parquet`).

 **Undefined behavior will occur if you pass:**
 - Pointers from `malloc`/`calloc`/`realloc`
 - Stack-allocated arrays
 - Already-freed pointers (double free)
 - Pointers from other libraries
 - Incorrect length (MUST match the length returned by the allocating function)

 NULL pointers are safely ignored (when len is 0).

 # Feature
 Always available, but only useful with "parquet" feature.
 */
void hedl_free_bytes(uint8_t *data, uintptr_t len);

/*
 Canonicalize a HEDL document.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `out_str` - Pointer to store canonical output (must be freed with hedl_free_string)

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid. Returns HEDL_ERR_NULL_PTR if doc is NULL or poisoned.
 */
int hedl_canonicalize(const struct HedlDocument *doc, char **out_str);

/*
 Lint a HEDL document.

 # Arguments
 * `doc` - Document handle from hedl_parse
 * `out_diag` - Pointer to store diagnostics handle

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid. Returns HEDL_ERR_NULL_PTR if doc is NULL or poisoned.
 */
int hedl_lint(const struct HedlDocument *doc, struct HedlDiagnostics **out_diag);

/*
 Parse a HEDL document from a string.

 # Arguments
 * `input` - UTF-8 encoded HEDL document
 * `input_len` - Length of input in bytes, or -1 for null-terminated
 * `strict` - Non-zero for strict mode (validate references)
 * `out_doc` - Pointer to store document handle

 # Returns
 HEDL_OK on success, error code on failure.

 # Safety
 All pointers must be valid.
 */
int hedl_parse(const char *input, int input_len, int strict, struct HedlDocument **out_doc);

/*
 Validate a HEDL document string.

 # Arguments
 * `input` - UTF-8 encoded HEDL document
 * `input_len` - Length of input in bytes, or -1 for null-terminated
 * `strict` - Non-zero for strict mode

 # Returns
 HEDL_OK if valid, error code if invalid.

 # Safety
 Input pointer must be valid.
 */
int hedl_validate(const char *input, int input_len, int strict);

/*
 Get the HEDL version of a parsed document.

 # Safety
 All pointers must be valid. Returns HEDL_ERR_NULL_PTR if doc is NULL or poisoned.
 */
int hedl_get_version(const struct HedlDocument *doc, int *major, int *minor);

/*
 Get the number of struct definitions in a document.

 # Safety
 Doc pointer must be valid. Returns -1 if doc is NULL or poisoned.
 */
int hedl_schema_count(const struct HedlDocument *doc);

/*
 Get the number of aliases in a document.

 # Safety
 Doc pointer must be valid. Returns -1 if doc is NULL or poisoned.
 */
int hedl_alias_count(const struct HedlDocument *doc);

/*
 Get the number of root items in a document.

 # Safety
 Doc pointer must be valid. Returns -1 if doc is NULL or poisoned.
 */
int hedl_root_item_count(const struct HedlDocument *doc);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#ifdef __cplusplus
}  // namespace hedl
#endif  // __cplusplus

#endif  /* HEDL_H */
