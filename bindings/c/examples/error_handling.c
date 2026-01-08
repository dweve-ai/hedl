/**
 * Dweve HEDL - Hierarchical Entity Data Language
 *
 * Copyright (c) 2025 Dweve IP B.V. and individual contributors.
 * SPDX-License-Identifier: Apache-2.0
 *
 * @file error_handling.c
 * @brief Comprehensive error handling example
 *
 * Demonstrates:
 * - Error detection and handling patterns
 * - Thread-local error messages
 * - Diagnostic information retrieval
 * - Validation errors and linting
 * - Recovery strategies
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "hedl.h"

static void print_section(const char* title) {
    printf("\n");
    printf("=================================================\n");
    printf(" %s\n", title);
    printf("=================================================\n");
}

static void demonstrate_error(const char* description, const char* hedl_input) {
    printf("\n--- Test: %s ---\n", description);
    printf("Input:\n%s\n", hedl_input);

    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl_input, -1, 1, &doc);

    if (result != HEDL_OK) {
        const char* error = hedl_get_last_error();
        printf("Expected error: %s\n", error ? error : "unknown");
        // Note: Do NOT free error string from hedl_get_last_error()
    } else {
        printf("Unexpected success - document parsed\n");
        hedl_free_document(doc);
    }
}

int main(void) {
    printf("HEDL Error Handling Example\n");
    printf("============================\n\n");
    printf("This example demonstrates various error scenarios and handling patterns.\n");

    // ========================================================================
    // Test 1: NULL pointer errors
    // ========================================================================

    print_section("Test 1: NULL Pointer Handling");

    HedlDocument* doc = NULL;
    int result = hedl_parse(NULL, 0, 1, &doc);

    if (result == HEDL_ERR_NULL_PTR) {
        printf("✓ NULL pointer correctly detected\n");
        const char* error = hedl_get_last_error();
        printf("  Error message: %s\n", error ? error : "none");
        // Note: Do NOT free error string from hedl_get_last_error()
    }

    // ========================================================================
    // Test 2: Invalid UTF-8
    // ========================================================================

    print_section("Test 2: Invalid UTF-8");

    // Create string with invalid UTF-8 byte sequence
    char invalid_utf8[] = "name: \xFF\xFE invalid";
    demonstrate_error("Invalid UTF-8 sequence", invalid_utf8);

    // ========================================================================
    // Test 3: Parse errors
    // ========================================================================

    print_section("Test 3: Parse Errors");

    demonstrate_error("Missing colon in key-value",
        "%VERSION: 1.0\n"
        "---\n"
        "name Alice\n"  // Missing colon
    );

    demonstrate_error("Invalid version format",
        "%VERSION: abc\n"  // Not a valid version
        "---\n"
        "name: Alice\n"
    );

    demonstrate_error("Unterminated string",
        "%VERSION: 1.0\n"
        "---\n"
        "name: \"Alice\n"  // Missing closing quote
    );

    // ========================================================================
    // Test 4: Validation and linting
    // ========================================================================

    print_section("Test 4: Validation and Linting");

    const char* valid_input =
        "%VERSION: 1.0\n"
        "---\n"
        "environment: production\n"
        "port: 8080\n";

    doc = NULL;
    result = hedl_parse(valid_input, -1, 1, &doc);

    if (result == HEDL_OK) {
        printf("Document parsed successfully\n");
        printf("✓ Validation passed during parsing (strict mode)\n");

        // Lint
        HedlDiagnostics* diags = NULL;
        result = hedl_lint(doc, &diags);

        if (result == HEDL_OK && diags) {
            int count = hedl_diagnostics_count(diags);
            printf("\nLint diagnostics: %d\n", count);

            for (int i = 0; i < count; i++) {
                char* message = NULL;

                if (hedl_diagnostics_get(diags, i, &message) == HEDL_OK) {
                    int severity = hedl_diagnostics_severity(diags, i);
                    const char* level = (severity == 0) ? "ERROR" :
                                      (severity == 1) ? "WARNING" : "INFO";
                    printf("  [%s] %s\n", level, message ? message : "");
                    hedl_free_string(message);
                }
            }

            hedl_free_diagnostics(diags);
        }

        hedl_free_document(doc);
    }

    // ========================================================================
    // Test 5: Conversion errors
    // ========================================================================

    print_section("Test 5: Conversion Errors");

    const char* complex_input =
        "%VERSION: 1.0\n"
        "---\n"
        "data:\n"
        "  nested:\n"
        "    deep: value\n";

    doc = NULL;
    result = hedl_parse(complex_input, -1, 1, &doc);

    if (result == HEDL_OK) {
        // Try CSV conversion (may fail if structure is too complex)
        char* csv = NULL;
        result = hedl_to_csv(doc, &csv);

        if (result == HEDL_OK) {
            printf("CSV conversion successful:\n%s\n", csv);
            hedl_free_string(csv);
        } else {
            printf("CSV conversion failed (expected for nested structures)\n");
            const char* error = hedl_get_last_error();
            printf("  Error: %s\n", error ? error : "unknown");
            // Note: Do NOT free error string from hedl_get_last_error()
        }

        hedl_free_document(doc);
    }

    // ========================================================================
    // Test 6: Memory safety
    // ========================================================================

    print_section("Test 6: Memory Safety");

    printf("Demonstrating safe NULL handling:\n");

    // Free NULL pointers (should be safe)
    hedl_free_string(NULL);
    hedl_free_document(NULL);
    hedl_free_diagnostics(NULL);
    hedl_free_bytes(NULL, 0);

    printf("✓ NULL pointers safely handled\n");

    // Multiple frees of valid pointers (ownership transferred)
    doc = NULL;
    result = hedl_parse("%VERSION: 1.0\n---\ntest: true\n", -1, 1, &doc);
    if (result == HEDL_OK) {
        hedl_free_document(doc);
        // Don't free again - ownership transferred
        printf("✓ Single free for valid pointer\n");
    }

    // ========================================================================
    // Summary
    // ========================================================================

    print_section("Summary");

    printf("\nError handling best practices:\n");
    printf("  1. Always check return codes\n");
    printf("  2. Use hedl_get_last_error() for detailed messages\n");
    printf("  3. Free error strings with hedl_free_string()\n");
    printf("  4. NULL pointers are safe in free functions\n");
    printf("  5. Errors are thread-local (safe for multithreading)\n");
    printf("  6. Use diagnostics for detailed validation feedback\n");
    printf("\nExample complete!\n\n");

    return 0;
}
