/**
 * HEDL C API
 *
 * Example usage:
 *
 *   HedlDocument* doc = NULL;
 *   int result = hedl_parse(hedl_string, -1, 1, &doc);
 *   if (result == HEDL_OK) {
 *       char* json = NULL;
 *       hedl_to_json(doc, 0, &json);
 *       printf("%s\n", json);
 *       hedl_free_string(json);
 *       hedl_free_document(doc);
 *   } else {
 *       printf("Error: %s\n", hedl_get_last_error());
 *   }
 *
 * Memory Management:
 * - Strings returned by hedl_* functions must be freed with hedl_free_string()
 * - Documents must be freed with hedl_free_document()
 * - Diagnostics must be freed with hedl_free_diagnostics()
 * - Byte arrays must be freed with hedl_free_bytes()
 */

#ifndef HEDL_H
#define HEDL_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ==========================================================================
 * Error Codes
 * ========================================================================== */

#define HEDL_OK               0
#define HEDL_ERR_NULL_PTR    -1
#define HEDL_ERR_INVALID_UTF8 -2
#define HEDL_ERR_PARSE       -3
#define HEDL_ERR_CANONICALIZE -4
#define HEDL_ERR_JSON        -5
#define HEDL_ERR_ALLOC       -6
#define HEDL_ERR_YAML        -7
#define HEDL_ERR_XML         -8
#define HEDL_ERR_CSV         -9
#define HEDL_ERR_PARQUET     -10
#define HEDL_ERR_LINT        -11

/* ==========================================================================
 * Opaque Types
 * ========================================================================== */

/** Opaque handle to a HEDL document */
typedef struct HedlDocument HedlDocument;

/** Opaque handle to lint diagnostics */
typedef struct HedlDiagnostics HedlDiagnostics;

/* ==========================================================================
 * Error Management
 * ========================================================================== */

/**
 * Get the last error message.
 * @return Error message string, or NULL if no error.
 *         Valid until the next hedl_* call. Do NOT free.
 */
const char* hedl_get_last_error(void);

/* ==========================================================================
 * Memory Management
 * ========================================================================== */

/** Free a string allocated by hedl functions. */
void hedl_free_string(char* s);

/** Free a document handle. */
void hedl_free_document(HedlDocument* doc);

/** Free a diagnostics handle. */
void hedl_free_diagnostics(HedlDiagnostics* diag);

/** Free bytes allocated by hedl functions (e.g., hedl_to_parquet). */
void hedl_free_bytes(uint8_t* data, size_t len);

/* ==========================================================================
 * Parsing and Validation
 * ========================================================================== */

/**
 * Parse a HEDL document from a string.
 * @param input UTF-8 encoded HEDL document
 * @param input_len Length in bytes, or -1 for null-terminated
 * @param strict Non-zero for strict mode (validate references)
 * @param out_doc Pointer to store document handle
 * @return HEDL_OK on success, error code on failure
 */
int hedl_parse(const char* input, int input_len, int strict, HedlDocument** out_doc);

/**
 * Validate a HEDL document string.
 * @return HEDL_OK if valid, error code if invalid
 */
int hedl_validate(const char* input, int input_len, int strict);

/* ==========================================================================
 * Document Information
 * ========================================================================== */

/** Get the HEDL version of a parsed document. */
int hedl_get_version(const HedlDocument* doc, int* major, int* minor);

/** Get the number of struct definitions. Returns -1 on error. */
int hedl_schema_count(const HedlDocument* doc);

/** Get the number of aliases. Returns -1 on error. */
int hedl_alias_count(const HedlDocument* doc);

/** Get the number of root items. Returns -1 on error. */
int hedl_root_item_count(const HedlDocument* doc);

/* ==========================================================================
 * Callback Type for Zero-Copy Output
 * ========================================================================== */

/**
 * Output callback function type for zero-copy string return.
 *
 * The callback receives:
 * - data: Pointer to output data (valid only during callback)
 * - len: Length of data in bytes (NOT null-terminated)
 * - user_data: User-provided context pointer
 *
 * CRITICAL: The data pointer is only valid during the callback execution.
 * Do NOT store the pointer for later use. If you need to keep the data,
 * copy it within the callback.
 *
 * The callback MUST NOT call back into HEDL functions.
 */
typedef void (*hedl_output_callback)(const char* data, size_t len, void* user_data);

/* ==========================================================================
 * Canonicalization
 * ========================================================================== */

/**
 * Canonicalize a HEDL document.
 * @param out_str Pointer to store output (must free with hedl_free_string)
 */
int hedl_canonicalize(const HedlDocument* doc, char** out_str);

/**
 * Canonicalize a HEDL document using zero-copy callback.
 * Recommended for large outputs (>1MB) to avoid memory allocation.
 * @param callback Function to receive the output data
 * @param user_data User context pointer passed to callback
 */
int hedl_canonicalize_callback(const HedlDocument* doc, hedl_output_callback callback, void* user_data);

/* ==========================================================================
 * JSON Conversion
 * ========================================================================== */

/**
 * Convert a HEDL document to JSON.
 * @param include_metadata Non-zero to include HEDL metadata (__type__, __schema__)
 * @param out_str Pointer to store output (must free with hedl_free_string)
 */
int hedl_to_json(const HedlDocument* doc, int include_metadata, char** out_str);

/**
 * Convert a HEDL document to JSON using zero-copy callback.
 * Recommended for large outputs (>1MB) to avoid memory allocation.
 * @param include_metadata Non-zero to include HEDL metadata
 * @param callback Function to receive the output data
 * @param user_data User context pointer passed to callback
 */
int hedl_to_json_callback(const HedlDocument* doc, int include_metadata, hedl_output_callback callback, void* user_data);

/**
 * Parse JSON into a HEDL document.
 * @param json_len Length in bytes, or -1 for null-terminated
 */
int hedl_from_json(const char* json, int json_len, HedlDocument** out_doc);

/* ==========================================================================
 * YAML Conversion
 * ========================================================================== */

/**
 * Convert a HEDL document to YAML.
 * @param include_metadata Non-zero to include HEDL metadata
 * @param out_str Pointer to store output (must free with hedl_free_string)
 */
int hedl_to_yaml(const HedlDocument* doc, int include_metadata, char** out_str);

/**
 * Convert a HEDL document to YAML using zero-copy callback.
 * Recommended for large outputs (>1MB) to avoid memory allocation.
 * @param include_metadata Non-zero to include HEDL metadata
 * @param callback Function to receive the output data
 * @param user_data User context pointer passed to callback
 */
int hedl_to_yaml_callback(const HedlDocument* doc, int include_metadata, hedl_output_callback callback, void* user_data);

/**
 * Parse YAML into a HEDL document.
 * @param yaml_len Length in bytes, or -1 for null-terminated
 */
int hedl_from_yaml(const char* yaml, int yaml_len, HedlDocument** out_doc);

/* ==========================================================================
 * XML Conversion
 * ========================================================================== */

/**
 * Convert a HEDL document to XML.
 * @param out_str Pointer to store output (must free with hedl_free_string)
 */
int hedl_to_xml(const HedlDocument* doc, char** out_str);

/**
 * Convert a HEDL document to XML using zero-copy callback.
 * Recommended for large outputs (>1MB) to avoid memory allocation.
 * @param callback Function to receive the output data
 * @param user_data User context pointer passed to callback
 */
int hedl_to_xml_callback(const HedlDocument* doc, hedl_output_callback callback, void* user_data);

/**
 * Parse XML into a HEDL document.
 * @param xml_len Length in bytes, or -1 for null-terminated
 */
int hedl_from_xml(const char* xml, int xml_len, HedlDocument** out_doc);

/* ==========================================================================
 * CSV Conversion
 * ========================================================================== */

/**
 * Convert a HEDL document to CSV.
 * Note: Only works for documents with matrix lists.
 * @param out_str Pointer to store output (must free with hedl_free_string)
 */
int hedl_to_csv(const HedlDocument* doc, char** out_str);

/**
 * Convert a HEDL document to CSV using zero-copy callback.
 * Note: Only works for documents with matrix lists.
 * Recommended for large outputs (>1MB) to avoid memory allocation.
 * @param callback Function to receive the output data
 * @param user_data User context pointer passed to callback
 */
int hedl_to_csv_callback(const HedlDocument* doc, hedl_output_callback callback, void* user_data);

/* ==========================================================================
 * Parquet Conversion
 * ========================================================================== */

/**
 * Convert a HEDL document to Parquet bytes.
 * Note: Only works for documents with matrix lists.
 * @param out_data Pointer to store output data (must free with hedl_free_bytes)
 * @param out_len Pointer to store output length
 */
int hedl_to_parquet(const HedlDocument* doc, uint8_t** out_data, size_t* out_len);

/**
 * Parse Parquet bytes into a HEDL document.
 * @param data Parquet file bytes
 * @param len Length of data
 */
int hedl_from_parquet(const uint8_t* data, size_t len, HedlDocument** out_doc);

/* ==========================================================================
 * Neo4j/Cypher Conversion
 * ========================================================================== */

/**
 * Convert a HEDL document to Cypher queries for Neo4j.
 * Generates CREATE/MERGE statements, constraints, and relationships.
 * @param use_merge Non-zero to use MERGE (idempotent), zero for CREATE
 * @param out_str Pointer to store output (must free with hedl_free_string)
 */
int hedl_to_neo4j_cypher(const HedlDocument* doc, int use_merge, char** out_str);

/**
 * Convert a HEDL document to Cypher queries using zero-copy callback.
 * Recommended for large outputs (>1MB) to avoid memory allocation.
 * @param use_merge Non-zero to use MERGE (idempotent), zero for CREATE
 * @param callback Function to receive the output data
 * @param user_data User context pointer passed to callback
 */
int hedl_to_neo4j_cypher_callback(const HedlDocument* doc, int use_merge, hedl_output_callback callback, void* user_data);

/* ==========================================================================
 * Linting
 * ========================================================================== */

/**
 * Lint a HEDL document.
 * @param out_diag Pointer to store diagnostics handle (must free with hedl_free_diagnostics)
 */
int hedl_lint(const HedlDocument* doc, HedlDiagnostics** out_diag);

/** Get the number of diagnostics. Returns -1 on error. */
int hedl_diagnostics_count(const HedlDiagnostics* diag);

/**
 * Get a diagnostic message.
 * @param out_str Pointer to store message (must free with hedl_free_string)
 */
int hedl_diagnostics_get(const HedlDiagnostics* diag, int index, char** out_str);

/**
 * Get a diagnostic severity.
 * @return Severity (0=Hint, 1=Warning, 2=Error), or -1 on error
 */
int hedl_diagnostics_severity(const HedlDiagnostics* diag, int index);

#ifdef __cplusplus
}
#endif

#endif /* HEDL_H */
