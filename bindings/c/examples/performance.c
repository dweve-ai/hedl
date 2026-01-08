/**
 * Dweve HEDL - Hierarchical Entity Data Language
 *
 * Copyright (c) 2025 Dweve IP B.V. and individual contributors.
 * SPDX-License-Identifier: Apache-2.0
 *
 * @file performance.c
 * @brief Performance benchmarking example
 *
 * Demonstrates:
 * - Parsing performance measurement
 * - Conversion performance benchmarking
 * - Memory usage tracking
 * - Large document handling
 * - Optimization techniques
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include "hedl.h"

#ifdef _WIN32
#include <windows.h>
#else
#include <sys/time.h>
#endif

/**
 * Get current time in microseconds
 */
static long long get_time_us(void) {
#ifdef _WIN32
    LARGE_INTEGER frequency, counter;
    QueryPerformanceFrequency(&frequency);
    QueryPerformanceCounter(&counter);
    return (counter.QuadPart * 1000000) / frequency.QuadPart;
#else
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (long long)tv.tv_sec * 1000000 + tv.tv_usec;
#endif
}

/**
 * Format microseconds as human-readable duration
 */
static void format_duration(long long us, char* buffer, size_t size) {
    if (us < 1000) {
        snprintf(buffer, size, "%lld μs", us);
    } else if (us < 1000000) {
        snprintf(buffer, size, "%.2f ms", us / 1000.0);
    } else {
        snprintf(buffer, size, "%.2f s", us / 1000000.0);
    }
}

/**
 * Generate test document with specified number of items
 */
static char* generate_test_document(int num_items) {
    size_t buffer_size = 1024 + (num_items * 128);
    char* buffer = malloc(buffer_size);
    if (!buffer) return NULL;

    int offset = snprintf(buffer, buffer_size,
        "%%VERSION: 1.0\n"
        "---\n"
        "generated: true\n"
        "item_count: %d\n", num_items);

    for (int i = 0; i < num_items; i++) {
        offset += snprintf(buffer + offset, buffer_size - offset,
            "item_%d_id: item_%d\n"
            "item_%d_value: %d\n"
            "item_%d_enabled: %s\n",
            i, i, i, i * 100, i, (i % 2) ? "true" : "false");
    }

    return buffer;
}

static void print_section(const char* title) {
    printf("\n");
    printf("=================================================\n");
    printf(" %s\n", title);
    printf("=================================================\n");
}

int main(void) {
    printf("HEDL Performance Benchmarking\n");
    printf("==============================\n\n");

    char duration[64];

    // ========================================================================
    // Benchmark 1: Parsing performance
    // ========================================================================

    print_section("Benchmark 1: Parsing Performance");

    const int sizes[] = {10, 100, 1000, 5000};
    const int num_sizes = sizeof(sizes) / sizeof(sizes[0]);

    printf("\n%-15s %-15s %-15s %-15s\n", "Items", "Parse Time", "Throughput", "Rate");
    printf("---------------------------------------------------------------\n");

    for (int i = 0; i < num_sizes; i++) {
        char* input = generate_test_document(sizes[i]);
        if (!input) {
            fprintf(stderr, "Failed to generate test document\n");
            continue;
        }

        int input_len = strlen(input);

        // Warmup
        HedlDocument* doc_warmup = NULL;
        hedl_parse(input, input_len, 1, &doc_warmup);
        hedl_free_document(doc_warmup);

        // Actual benchmark
        long long start = get_time_us();
        HedlDocument* doc = NULL;
        int result = hedl_parse(input, input_len, 1, &doc);
        long long end = get_time_us();

        if (result == HEDL_OK) {
            long long elapsed = end - start;
            format_duration(elapsed, duration, sizeof(duration));

            double mb_per_sec = (input_len / (1024.0 * 1024.0)) / (elapsed / 1000000.0);
            double items_per_sec = sizes[i] / (elapsed / 1000000.0);

            printf("%-15d %-15s %-10.2f MB/s %-10.0f items/s\n",
                   sizes[i], duration, mb_per_sec, items_per_sec);

            hedl_free_document(doc);
        } else {
            const char* error = hedl_get_last_error();
            printf("%-15d FAILED: %s\n", sizes[i], error ? error : "unknown");
        }

        free(input);
    }

    // ========================================================================
    // Benchmark 2: Format conversion performance
    // ========================================================================

    print_section("Benchmark 2: Format Conversion");

    const char* test_input = generate_test_document(1000);
    if (test_input) {
        HedlDocument* doc = NULL;
        int result = hedl_parse(test_input, -1, 1, &doc);

        if (result == HEDL_OK) {
            printf("\n%-15s %-15s %-15s\n", "Format", "Time", "Output Size");
            printf("-----------------------------------------------\n");

            // JSON conversion
            long long start = get_time_us();
            char* json = NULL;
            result = hedl_to_json(doc, 0, &json);
            long long end = get_time_us();

            if (result == HEDL_OK) {
                format_duration(end - start, duration, sizeof(duration));
                printf("%-15s %-15s %-15zu bytes\n", "JSON", duration, strlen(json));
                hedl_free_string(json);
            }

            // YAML conversion
            start = get_time_us();
            char* yaml = NULL;
            result = hedl_to_yaml(doc, 0, &yaml);  // include_metadata=0
            end = get_time_us();

            if (result == HEDL_OK) {
                format_duration(end - start, duration, sizeof(duration));
                printf("%-15s %-15s %-15zu bytes\n", "YAML", duration, strlen(yaml));
                hedl_free_string(yaml);
            }

            // XML conversion
            start = get_time_us();
            char* xml = NULL;
            result = hedl_to_xml(doc, &xml);
            end = get_time_us();

            if (result == HEDL_OK) {
                format_duration(end - start, duration, sizeof(duration));
                printf("%-15s %-15s %-15zu bytes\n", "XML", duration, strlen(xml));
                hedl_free_string(xml);
            }

            // Canonicalization
            start = get_time_us();
            char* canonical = NULL;
            result = hedl_canonicalize(doc, &canonical);
            end = get_time_us();

            if (result == HEDL_OK) {
                format_duration(end - start, duration, sizeof(duration));
                printf("%-15s %-15s %-15zu bytes\n", "Canonical", duration, strlen(canonical));
                hedl_free_string(canonical);
            }

            hedl_free_document(doc);
        }

        free((void*)test_input);
    }

    // ========================================================================
    // Benchmark 3: Memory reuse
    // ========================================================================

    print_section("Benchmark 3: Memory Reuse Pattern");

    printf("\nDemonstrating efficient pattern: parse once, convert many times\n\n");

    const char* reuse_input =
        "%VERSION: 1.0\n"
        "---\n"
        "name: Performance Test\n"
        "iterations: 1000\n";

    HedlDocument* doc = NULL;
    int result = hedl_parse(reuse_input, -1, 1, &doc);

    if (result == HEDL_OK) {
        const int iterations = 100;

        // Benchmark: single parse, multiple conversions
        long long start = get_time_us();
        for (int i = 0; i < iterations; i++) {
            char* json = NULL;
            hedl_to_json(doc, 0, &json);
            hedl_free_string(json);
        }
        long long end = get_time_us();

        format_duration((end - start) / iterations, duration, sizeof(duration));
        printf("Average conversion time (%d iterations): %s\n", iterations, duration);

        hedl_free_document(doc);
    }

    // ========================================================================
    // Summary
    // ========================================================================

    print_section("Performance Summary");

    printf("\nPerformance tips:\n");
    printf("  1. Reuse parsed documents for multiple conversions\n");
    printf("  2. Use batch processing for large datasets\n");
    printf("  3. Pre-allocate buffers when possible\n");
    printf("  4. Profile your specific use case\n");
    printf("  5. Consider parallel processing for independent documents\n");
    printf("\nBenchmarking complete!\n\n");

    return 0;
}
