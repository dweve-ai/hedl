/**
 * Example C program using HEDL FFI bindings
 *
 * Compile with:
 *   gcc -o example example.c -I../include -L../../../target/debug -lhedl_ffi
 *
 * Run with:
 *   LD_LIBRARY_PATH=../../../target/debug ./example
 */

#include <stdio.h>
#include <stdlib.h>
#include "../include/hedl.h"

int main(void) {
    const char* hedl_input =
        "%VERSION: 1.0\n"
        "%ALIAS: prod = production\n"
        "---\n"
        "environment: @prod\n"
        "port: 8080\n"
        "enabled: true\n";

    printf("HEDL FFI Example\n");
    printf("================\n\n");

    // Parse document
    printf("Parsing HEDL document...\n");
    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl_input, -1, 1, &doc);

    if (result != HEDL_OK) {
        const char* error = hedl_get_last_error();
        fprintf(stderr, "Parse error: %s\n", error ? error : "Unknown error");
        hedl_free_string((char*)error);
        return 1;
    }

    printf("Success!\n\n");

    // Get version
    int major = 0, minor = 0;
    result = hedl_get_version(doc, &major, &minor);
    if (result == HEDL_OK) {
        printf("Document version: %d.%d\n", major, minor);
    }

    // Get counts
    int schema_count = hedl_schema_count(doc);
    int alias_count = hedl_alias_count(doc);
    printf("Struct definitions: %d\n", schema_count);
    printf("Aliases: %d\n\n", alias_count);

    // Canonicalize
    printf("Canonicalizing document...\n");
    char* canonical = NULL;
    result = hedl_canonicalize(doc, &canonical);
    if (result == HEDL_OK) {
        printf("Canonical form:\n%s\n", canonical);
        hedl_free_string(canonical);
    } else {
        const char* error = hedl_get_last_error();
        fprintf(stderr, "Canonicalization error: %s\n", error ? error : "Unknown error");
        hedl_free_string((char*)error);
    }

    // Clean up
    hedl_free_document(doc);
    printf("\nDone!\n");

    return 0;
}
