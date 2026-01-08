/**
 * Dweve HEDL - Hierarchical Entity Data Language
 *
 * Copyright (c) 2025 Dweve IP B.V. and individual contributors.
 * SPDX-License-Identifier: Apache-2.0
 *
 * @file basic.c
 * @brief Basic HEDL parsing and inspection example
 *
 * Demonstrates:
 * - Parsing HEDL documents
 * - Extracting metadata (version, schema count, alias count)
 * - Canonicalization
 * - Proper memory management
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "hedl.h"

/**
 * Print section header for better readability
 */
static void print_section(const char* title) {
    printf("\n");
    printf("=================================================\n");
    printf(" %s\n", title);
    printf("=================================================\n");
}

/**
 * Print error message and return error code
 */
static int print_error(const char* operation) {
    const char* error = hedl_get_last_error();
    fprintf(stderr, "ERROR during %s: %s\n", operation, error ? error : "unknown error");
    // Note: hedl_get_last_error() returns a pointer that should NOT be freed
    return 1;
}

int main(void) {
    printf("HEDL Basic Example\n");
    printf("==================\n\n");
    printf("This example demonstrates basic HEDL parsing and document inspection.\n");

    // Sample HEDL document with version and data
    const char* hedl_input =
        "%VERSION: 1.0\n"
        "---\n"
        "name: Alice\n"
        "age: 30\n"
        "email: alice@example.com\n";

    printf("Input HEDL document:\n");
    printf("-----------------------------------\n");
    printf("%s", hedl_input);
    printf("-----------------------------------\n");

    // ========================================================================
    // Step 1: Parse the document
    // ========================================================================

    print_section("Step 1: Parsing Document");

    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl_input, -1, 1, &doc);

    if (result != HEDL_OK) {
        return print_error("parsing");
    }

    printf("✓ Document parsed successfully\n");

    // ========================================================================
    // Step 2: Extract metadata
    // ========================================================================

    print_section("Step 2: Document Metadata");

    // Get version
    int major = 0, minor = 0;
    result = hedl_get_version(doc, &major, &minor);
    if (result == HEDL_OK) {
        printf("Document version: %d.%d\n", major, minor);
    } else {
        printf("Warning: Could not retrieve version\n");
    }

    // Get counts
    int schema_count = hedl_schema_count(doc);
    int alias_count = hedl_alias_count(doc);
    int root_items = hedl_root_item_count(doc);

    printf("Schema definitions: %d\n", schema_count);
    printf("Alias definitions: %d\n", alias_count);
    printf("Root items: %d\n", root_items);

    // ========================================================================
    // Step 3: Document is already validated during parsing
    // ========================================================================

    print_section("Step 3: Document Structure");

    printf("✓ Document structure validated during parsing\n");
    printf("  (strict mode ensures all references are valid)\n");

    // ========================================================================
    // Step 4: Canonicalize (normalize to standard form)
    // ========================================================================

    print_section("Step 4: Canonicalization");

    char* canonical = NULL;
    result = hedl_canonicalize(doc, &canonical);

    if (result != HEDL_OK) {
        print_error("canonicalization");
        hedl_free_document(doc);
        return 1;
    }

    printf("Canonical form:\n");
    printf("-----------------------------------\n");
    printf("%s", canonical);
    printf("-----------------------------------\n");

    hedl_free_string(canonical);

    // ========================================================================
    // Step 5: Cleanup
    // ========================================================================

    print_section("Step 5: Cleanup");

    hedl_free_document(doc);
    printf("✓ Memory freed successfully\n");

    print_section("Example Complete");
    printf("\nAll operations completed successfully!\n\n");

    return 0;
}
