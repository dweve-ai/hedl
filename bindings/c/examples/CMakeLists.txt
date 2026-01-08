# Dweve HEDL - Hierarchical Entity Data Language
#
# Copyright (c) 2025 Dweve IP B.V. and individual contributors.
#
# SPDX-License-Identifier: Apache-2.0

# ============================================================================
# HEDL C Examples
# ============================================================================

# Use shared library by default, fall back to static
if(TARGET HEDL::hedl)
    set(HEDL_LINK_TARGET HEDL::hedl)
elseif(TARGET HEDL::hedl_static)
    set(HEDL_LINK_TARGET HEDL::hedl_static)
else()
    message(FATAL_ERROR "No HEDL library target available")
endif()

# ============================================================================
# Basic Example
# ============================================================================

add_executable(hedl_example_basic basic.c)
target_link_libraries(hedl_example_basic PRIVATE ${HEDL_LINK_TARGET})
target_include_directories(hedl_example_basic PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../include)

# ============================================================================
# Format Conversion Example
# ============================================================================

add_executable(hedl_example_convert convert.c)
target_link_libraries(hedl_example_convert PRIVATE ${HEDL_LINK_TARGET})
target_include_directories(hedl_example_convert PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../include)

# ============================================================================
# Error Handling Example
# ============================================================================

add_executable(hedl_example_errors error_handling.c)
target_link_libraries(hedl_example_errors PRIVATE ${HEDL_LINK_TARGET})
target_include_directories(hedl_example_errors PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../include)

# ============================================================================
# Performance Example
# ============================================================================

add_executable(hedl_example_performance performance.c)
target_link_libraries(hedl_example_performance PRIVATE ${HEDL_LINK_TARGET})
target_include_directories(hedl_example_performance PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../include)

# Add -lm for math library on Unix
if(UNIX)
    target_link_libraries(hedl_example_performance PRIVATE m)
endif()

# ============================================================================
# CMake Integration Example
# ============================================================================

# This shows how downstream projects would use find_package(HEDL)
add_executable(hedl_example_cmake_integration cmake_integration.c)
target_link_libraries(hedl_example_cmake_integration PRIVATE ${HEDL_LINK_TARGET})
target_include_directories(hedl_example_cmake_integration PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../include)

message(STATUS "Configured HEDL examples")
