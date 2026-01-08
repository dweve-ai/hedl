/**
 * HEDL FFI Zero-Copy Callback API Demo
 *
 * Demonstrates how to use the callback-based API for large outputs
 * to avoid memory allocation overhead.
 *
 * Compile:
 *   gcc -o callback_demo callback_demo.c -L../target/release -lhedl_ffi
 *
 * Run:
 *   LD_LIBRARY_PATH=../target/release ./callback_demo
 */

#include "../include/hedl.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ========================================================================
 * Example 1: Simple callback that writes to stdout
 * ======================================================================== */

void stdout_callback(const char* data, size_t len, void* user_data) {
    (void)user_data;  /* unused */
    fwrite(data, 1, len, stdout);
}

void example_simple_stdout(void) {
    printf("=== Example 1: Write to stdout ===\n");

    const char* hedl_input =
        "%VERSION: 1.0\n"
        "---\n"
        "person:\n"
        "  name: Alice\n"
        "  age: 30\n"
        "  city: New York\n";

    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl_input, -1, 0, &doc);

    if (result != HEDL_OK) {
        fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
        return;
    }

    printf("JSON output:\n");
    result = hedl_to_json_callback(doc, 0, stdout_callback, NULL);
    printf("\n\n");

    if (result != HEDL_OK) {
        fprintf(stderr, "Conversion error: %s\n", hedl_get_last_error());
    }

    hedl_free_document(doc);
}

/* ========================================================================
 * Example 2: Callback that writes to a file
 * ======================================================================== */

void file_callback(const char* data, size_t len, void* user_data) {
    FILE* f = (FILE*)user_data;
    fwrite(data, 1, len, f);
}

void example_write_to_file(void) {
    printf("=== Example 2: Write to file ===\n");

    const char* hedl_input =
        "%VERSION: 1.0\n"
        "---\n"
        "users: [\n"
        "  { name: Alice, role: admin }\n"
        "  { name: Bob, role: user }\n"
        "  { name: Charlie, role: user }\n"
        "]\n";

    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl_input, -1, 0, &doc);

    if (result != HEDL_OK) {
        fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
        return;
    }

    FILE* output = fopen("/tmp/hedl_output.json", "w");
    if (!output) {
        fprintf(stderr, "Failed to open output file\n");
        hedl_free_document(doc);
        return;
    }

    result = hedl_to_json_callback(doc, 0, file_callback, output);
    fclose(output);

    if (result == HEDL_OK) {
        printf("Successfully wrote JSON to /tmp/hedl_output.json\n\n");
    } else {
        fprintf(stderr, "Conversion error: %s\n", hedl_get_last_error());
    }

    hedl_free_document(doc);
}

/* ========================================================================
 * Example 3: Callback that accumulates data (demonstrates copying)
 * ======================================================================== */

typedef struct {
    char* buffer;
    size_t size;
    size_t capacity;
} AccumulatorContext;

void accumulator_callback(const char* data, size_t len, void* user_data) {
    AccumulatorContext* ctx = (AccumulatorContext*)user_data;

    /* Ensure we have enough capacity */
    while (ctx->size + len > ctx->capacity) {
        ctx->capacity = (ctx->capacity == 0) ? 4096 : ctx->capacity * 2;
        ctx->buffer = realloc(ctx->buffer, ctx->capacity);
    }

    /* Copy the data (MUST copy since data is only valid during callback) */
    memcpy(ctx->buffer + ctx->size, data, len);
    ctx->size += len;
}

void example_accumulator(void) {
    printf("=== Example 3: Accumulate data ===\n");

    const char* hedl_input =
        "%VERSION: 1.0\n"
        "---\n"
        "product:\n"
        "  id: 12345\n"
        "  name: Widget\n"
        "  price: 29.99\n"
        "  in_stock: true\n";

    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl_input, -1, 0, &doc);

    if (result != HEDL_OK) {
        fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
        return;
    }

    AccumulatorContext ctx = { NULL, 0, 0 };
    result = hedl_to_json_callback(doc, 0, accumulator_callback, &ctx);

    if (result == HEDL_OK) {
        printf("Accumulated %zu bytes:\n%.*s\n\n", ctx.size, (int)ctx.size, ctx.buffer);
    } else {
        fprintf(stderr, "Conversion error: %s\n", hedl_get_last_error());
    }

    free(ctx.buffer);
    hedl_free_document(doc);
}

/* ========================================================================
 * Example 4: Multiple format conversions with callbacks
 * ======================================================================== */

typedef struct {
    size_t total_bytes;
    size_t call_count;
} CounterContext;

void counter_callback(const char* data, size_t len, void* user_data) {
    CounterContext* ctx = (CounterContext*)user_data;
    ctx->total_bytes += len;
    ctx->call_count++;
}

void example_multiple_formats(void) {
    printf("=== Example 4: Multiple format conversions ===\n");

    const char* hedl_input =
        "%VERSION: 1.0\n"
        "---\n"
        "data:\n"
        "  field1: value1\n"
        "  field2: value2\n"
        "  field3: value3\n";

    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl_input, -1, 0, &doc);

    if (result != HEDL_OK) {
        fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
        return;
    }

    CounterContext json_ctx = { 0, 0 };
    CounterContext yaml_ctx = { 0, 0 };
    CounterContext xml_ctx = { 0, 0 };
    CounterContext canon_ctx = { 0, 0 };

    /* Convert to different formats */
    hedl_to_json_callback(doc, 0, counter_callback, &json_ctx);
    hedl_to_yaml_callback(doc, 0, counter_callback, &yaml_ctx);
    hedl_to_xml_callback(doc, counter_callback, &xml_ctx);
    hedl_canonicalize_callback(doc, counter_callback, &canon_ctx);

    printf("Format         Bytes  Calls\n");
    printf("------         -----  -----\n");
    printf("JSON           %5zu  %5zu\n", json_ctx.total_bytes, json_ctx.call_count);
    printf("YAML           %5zu  %5zu\n", yaml_ctx.total_bytes, yaml_ctx.call_count);
    printf("XML            %5zu  %5zu\n", xml_ctx.total_bytes, xml_ctx.call_count);
    printf("Canonical      %5zu  %5zu\n", canon_ctx.total_bytes, canon_ctx.call_count);
    printf("\n");

    hedl_free_document(doc);
}

/* ========================================================================
 * Example 5: Comparison - callback vs regular API
 * ======================================================================== */

void example_callback_vs_regular(void) {
    printf("=== Example 5: Callback vs Regular API ===\n");

    const char* hedl_input =
        "%VERSION: 1.0\n"
        "---\n"
        "message: Hello, HEDL!\n";

    HedlDocument* doc = NULL;
    hedl_parse(hedl_input, -1, 0, &doc);

    /* Regular API */
    char* json_regular = NULL;
    hedl_to_json(doc, 0, &json_regular);
    size_t regular_len = strlen(json_regular);

    /* Callback API */
    CounterContext ctx = { 0, 0 };
    hedl_to_json_callback(doc, 0, counter_callback, &ctx);

    printf("Regular API: %zu bytes (requires free)\n", regular_len);
    printf("Callback API: %zu bytes (zero-copy)\n", ctx.total_bytes);
    printf("Sizes match: %s\n\n", (regular_len == ctx.total_bytes) ? "YES" : "NO");

    hedl_free_string(json_regular);
    hedl_free_document(doc);
}

/* ========================================================================
 * Example 6: Large document handling
 * ======================================================================== */

void example_large_document(void) {
    printf("=== Example 6: Large document (>1MB recommendation) ===\n");

    /* Build a large HEDL document */
    size_t estimated_size = 0;
    char* large_hedl = malloc(10 * 1024 * 1024);  /* 10MB buffer */
    char* ptr = large_hedl;

    ptr += sprintf(ptr, "%%VERSION: 1.0\n---\n");

    /* Add many entities */
    for (int i = 0; i < 10000; i++) {
        ptr += sprintf(ptr, "entity%d:\n", i);
        ptr += sprintf(ptr, "  id: %d\n", i);
        ptr += sprintf(ptr, "  name: Entity_%d\n", i);
        ptr += sprintf(ptr, "  value: %.2f\n", i * 1.5);
        ptr += sprintf(ptr, "  active: %s\n", (i % 2 == 0) ? "true" : "false");
    }

    estimated_size = ptr - large_hedl;
    printf("Created HEDL document: %zu bytes\n", estimated_size);

    HedlDocument* doc = NULL;
    int result = hedl_parse(large_hedl, -1, 0, &doc);
    free(large_hedl);

    if (result != HEDL_OK) {
        fprintf(stderr, "Parse error: %s\n", hedl_get_last_error());
        return;
    }

    /* Use callback API for large output */
    CounterContext ctx = { 0, 0 };
    result = hedl_to_json_callback(doc, 0, counter_callback, &ctx);

    if (result == HEDL_OK) {
        printf("JSON output: %zu bytes (%s 1MB)\n",
               ctx.total_bytes,
               ctx.total_bytes > 1024*1024 ? ">" : "<");
        printf("Callback called: %zu time(s)\n", ctx.call_count);
        printf("For outputs >1MB, callback API is recommended to avoid allocation overhead\n\n");
    }

    hedl_free_document(doc);
}

/* ========================================================================
 * Main
 * ======================================================================== */

int main(void) {
    printf("HEDL FFI Zero-Copy Callback API Demo\n");
    printf("=====================================\n\n");

    example_simple_stdout();
    example_write_to_file();
    example_accumulator();
    example_multiple_formats();
    example_callback_vs_regular();
    example_large_document();

    printf("All examples completed successfully!\n");
    return 0;
}
