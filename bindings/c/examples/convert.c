/**
 * Dweve HEDL - Hierarchical Entity Data Language
 *
 * Copyright (c) 2025 Dweve IP B.V. and individual contributors.
 * SPDX-License-Identifier: Apache-2.0
 *
 * @file convert.c
 * @brief Format conversion example
 *
 * Demonstrates:
 * - Converting HEDL to JSON, YAML, XML, CSV
 * - Converting from JSON/YAML/XML back to HEDL
 * - Round-trip conversion validation
 * - Format-specific options (pretty-printing, etc.)
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

static int print_error(const char* operation) {
    const char* error = hedl_get_last_error();
    fprintf(stderr, "ERROR during %s: %s\n", operation, error ? error : "unknown error");
    // Note: hedl_get_last_error() returns a pointer that should NOT be freed
    return 1;
}

int main(void) {
    printf("HEDL Format Conversion Example\n");
    printf("===============================\n\n");

    const char* hedl_input =
        "%VERSION: 1.0\n"
        "---\n"
        "name: Alice Johnson\n"
        "age: 30\n"
        "email: alice@example.com\n"
        "city: Springfield\n";

    printf("Original HEDL:\n");
    printf("-----------------------------------\n");
    printf("%s", hedl_input);
    printf("-----------------------------------\n");

    // ========================================================================
    // Parse HEDL document
    // ========================================================================

    print_section("Parsing HEDL Document");

    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl_input, -1, 1, &doc);

    if (result != HEDL_OK) {
        return print_error("parsing");
    }

    printf("✓ Document parsed successfully\n");

    // ========================================================================
    // Convert to JSON
    // ========================================================================

    print_section("Convert to JSON");

    char* json = NULL;
    result = hedl_to_json(doc, 1, &json);  // pretty=1 for formatted output

    if (result == HEDL_OK) {
        printf("%s\n", json);
        hedl_free_string(json);
    } else {
        print_error("JSON conversion");
    }

    // ========================================================================
    // Convert to YAML
    // ========================================================================

    print_section("Convert to YAML");

    char* yaml = NULL;
    result = hedl_to_yaml(doc, 0, &yaml);  // include_metadata=0

    if (result == HEDL_OK) {
        printf("%s\n", yaml);
        hedl_free_string(yaml);
    } else {
        print_error("YAML conversion");
    }

    // ========================================================================
    // Convert to XML
    // ========================================================================

    print_section("Convert to XML");

    char* xml = NULL;
    result = hedl_to_xml(doc, &xml);

    if (result == HEDL_OK) {
        printf("%s\n", xml);
        hedl_free_string(xml);
    } else {
        print_error("XML conversion");
    }

    // ========================================================================
    // Convert to CSV
    // ========================================================================

    print_section("Convert to CSV");

    char* csv = NULL;
    result = hedl_to_csv(doc, &csv);

    if (result == HEDL_OK) {
        printf("%s\n", csv);
        hedl_free_string(csv);
    } else {
        print_error("CSV conversion");
    }

    // ========================================================================
    // Round-trip test: HEDL -> JSON -> HEDL
    // ========================================================================

    print_section("Round-trip Test: HEDL -> JSON -> HEDL");

    char* json_rt = NULL;
    result = hedl_to_json(doc, 0, &json_rt);  // pretty=0 for compact

    if (result != HEDL_OK) {
        print_error("JSON conversion for round-trip");
        hedl_free_document(doc);
        return 1;
    }

    printf("Intermediate JSON:\n%s\n\n", json_rt);

    // Convert JSON back to HEDL
    HedlDocument* doc2 = NULL;
    result = hedl_from_json(json_rt, -1, &doc2);
    hedl_free_string(json_rt);

    if (result != HEDL_OK) {
        print_error("JSON to HEDL conversion");
        hedl_free_document(doc);
        return 1;
    }

    // Canonicalize both documents for comparison
    char* canonical1 = NULL;
    char* canonical2 = NULL;

    hedl_canonicalize(doc, &canonical1);
    hedl_canonicalize(doc2, &canonical2);

    if (canonical1 && canonical2 && strcmp(canonical1, canonical2) == 0) {
        printf("✓ Round-trip successful: documents are equivalent\n");
    } else {
        printf("✗ Round-trip failed: documents differ\n");
    }

    hedl_free_string(canonical1);
    hedl_free_string(canonical2);
    hedl_free_document(doc2);

    // ========================================================================
    // Cleanup
    // ========================================================================

    print_section("Cleanup");

    hedl_free_document(doc);
    printf("✓ Memory freed successfully\n");

    printf("\nExample complete!\n\n");
    return 0;
}
