/**
 * Dweve HEDL - Hierarchical Entity Data Language
 *
 * Copyright (c) 2025 Dweve IP B.V. and individual contributors.
 * SPDX-License-Identifier: Apache-2.0
 *
 * @file cmake_integration.c
 * @brief Example demonstrating CMake integration with find_package()
 *
 * This file demonstrates how downstream projects integrate HEDL using CMake.
 *
 * CMakeLists.txt for your project:
 * --------------------------------
 * cmake_minimum_required(VERSION 3.15)
 * project(MyProject)
 *
 * # Find HEDL package
 * find_package(HEDL REQUIRED)
 *
 * # Create executable
 * add_executable(myapp main.c)
 *
 * # Link HEDL library
 * target_link_libraries(myapp PRIVATE HEDL::hedl)
 *
 * # Or link static library:
 * # target_link_libraries(myapp PRIVATE HEDL::hedl_static)
 * --------------------------------
 *
 * Build commands:
 * ---------------
 * mkdir build && cd build
 * cmake .. -DCMAKE_PREFIX_PATH=/path/to/hedl/install
 * cmake --build .
 * ./myapp
 */

#include <stdio.h>
#include <stdlib.h>
#include "hedl.h"

int main(void) {
    printf("========================================\n");
    printf("HEDL CMake Integration Example\n");
    printf("========================================\n\n");

    printf("This executable was built using CMake's find_package(HEDL).\n");
    printf("See the comments in this source file for integration details.\n\n");

    // Demonstrate basic functionality
    const char* hedl_input =
        "%VERSION: 1.0\n"
        "---\n"
        "project: MyApplication\n"
        "build_system: CMake\n"
        "hedl_integration: find_package\n";

    printf("Sample HEDL document:\n");
    printf("---\n%s---\n\n", hedl_input);

    HedlDocument* doc = NULL;
    int result = hedl_parse(hedl_input, -1, 1, &doc);

    if (result != HEDL_OK) {
        const char* error = hedl_get_last_error();
        fprintf(stderr, "Parse error: %s\n", error ? error : "unknown");
        hedl_free_string((char*)error);
        return 1;
    }

    printf("✓ Document parsed successfully\n");

    // Get metadata
    int major = 0, minor = 0;
    hedl_get_version(doc, &major, &minor);
    printf("✓ Document version: %d.%d\n", major, minor);

    // Convert to JSON
    char* json = NULL;
    result = hedl_to_json(doc, 1, &json);

    if (result == HEDL_OK) {
        printf("\nJSON output:\n%s\n", json);
        hedl_free_string(json);
    }

    hedl_free_document(doc);

    printf("\n========================================\n");
    printf("CMake Integration Guidelines\n");
    printf("========================================\n\n");

    printf("1. Installation:\n");
    printf("   cd /path/to/hedl/bindings/c\n");
    printf("   mkdir build && cd build\n");
    printf("   cmake .. -DCMAKE_INSTALL_PREFIX=/usr/local\n");
    printf("   cmake --build .\n");
    printf("   sudo cmake --install .\n\n");

    printf("2. In your CMakeLists.txt:\n");
    printf("   find_package(HEDL REQUIRED)\n");
    printf("   target_link_libraries(your_target PRIVATE HEDL::hedl)\n\n");

    printf("3. Build your project:\n");
    printf("   cmake .. -DCMAKE_PREFIX_PATH=/usr/local\n");
    printf("   cmake --build .\n\n");

    printf("4. Advanced options:\n");
    printf("   - Use HEDL::hedl_static for static linking\n");
    printf("   - Set HEDL_FEATURE_* options to control enabled formats\n");
    printf("   - Use pkg-config as alternative to CMake\n\n");

    printf("Integration successful!\n");

    return 0;
}
