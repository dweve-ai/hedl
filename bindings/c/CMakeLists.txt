# Dweve HEDL - Hierarchical Entity Data Language
#
# Copyright (c) 2025 Dweve IP B.V. and individual contributors.
#
# SPDX-License-Identifier: Apache-2.0

cmake_minimum_required(VERSION 3.15)

project(HEDL
    VERSION 1.0.0
    DESCRIPTION "HEDL C/C++ FFI Bindings"
    LANGUAGES C CXX
)

# ============================================================================
# Build Options
# ============================================================================

option(HEDL_BUILD_SHARED "Build shared library" ON)
option(HEDL_BUILD_STATIC "Build static library" OFF)
option(HEDL_BUILD_EXAMPLES "Build examples" ON)
option(HEDL_BUILD_TESTS "Build tests" OFF)
option(HEDL_INSTALL "Install library and headers" ON)

# Feature flags for format converters
option(HEDL_FEATURE_JSON "Enable JSON support" ON)
option(HEDL_FEATURE_YAML "Enable YAML support" ON)
option(HEDL_FEATURE_XML "Enable XML support" ON)
option(HEDL_FEATURE_CSV "Enable CSV support" ON)
option(HEDL_FEATURE_PARQUET "Enable Parquet support" ON)
option(HEDL_FEATURE_NEO4J "Enable Neo4j Cypher support" ON)

# ============================================================================
# Build Configuration
# ============================================================================

# Set C/C++ standards
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Export compile commands for IDEs
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Position independent code for shared libraries
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# Build type defaults
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release CACHE STRING "Build type" FORCE)
    set_property(CACHE CMAKE_BUILD_TYPE PROPERTY STRINGS "Debug" "Release" "RelWithDebInfo" "MinSizeRel")
endif()

# ============================================================================
# Detect Rust Toolchain and Build Type
# ============================================================================

find_program(CARGO_EXECUTABLE cargo REQUIRED)
find_program(RUSTC_EXECUTABLE rustc REQUIRED)

# Get Rust version for diagnostics
execute_process(
    COMMAND ${RUSTC_EXECUTABLE} --version
    OUTPUT_VARIABLE RUST_VERSION
    OUTPUT_STRIP_TRAILING_WHITESPACE
)
message(STATUS "Found Rust: ${RUST_VERSION}")

# Determine Rust build profile based on CMAKE_BUILD_TYPE
if(CMAKE_BUILD_TYPE MATCHES "Debug")
    set(RUST_BUILD_TYPE "debug")
    set(RUST_BUILD_FLAG "")
else()
    set(RUST_BUILD_TYPE "release")
    set(RUST_BUILD_FLAG "--release")
endif()

# Determine Rust target directory
if(DEFINED ENV{CARGO_TARGET_DIR})
    set(RUST_TARGET_DIR "$ENV{CARGO_TARGET_DIR}")
else()
    set(RUST_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../target")
endif()

set(RUST_OUTPUT_DIR "${RUST_TARGET_DIR}/${RUST_BUILD_TYPE}")

message(STATUS "Rust build type: ${RUST_BUILD_TYPE}")
message(STATUS "Rust target directory: ${RUST_TARGET_DIR}")

# ============================================================================
# Determine Library Names by Platform
# ============================================================================

if(WIN32)
    set(HEDL_SHARED_LIB_NAME "hedl_ffi.dll")
    set(HEDL_STATIC_LIB_NAME "hedl_ffi.lib")
    set(HEDL_IMPLIB_NAME "hedl_ffi.dll.lib")
elseif(APPLE)
    set(HEDL_SHARED_LIB_NAME "libhedl_ffi.dylib")
    set(HEDL_STATIC_LIB_NAME "libhedl_ffi.a")
else()
    set(HEDL_SHARED_LIB_NAME "libhedl_ffi.so")
    set(HEDL_STATIC_LIB_NAME "libhedl_ffi.a")
endif()

# ============================================================================
# Build Feature String for Cargo
# ============================================================================

set(CARGO_FEATURES "")
set(FEATURE_LIST "")

if(HEDL_FEATURE_JSON)
    list(APPEND FEATURE_LIST "json")
endif()
if(HEDL_FEATURE_YAML)
    list(APPEND FEATURE_LIST "yaml")
endif()
if(HEDL_FEATURE_XML)
    list(APPEND FEATURE_LIST "xml")
endif()
if(HEDL_FEATURE_CSV)
    list(APPEND FEATURE_LIST "csv")
endif()
if(HEDL_FEATURE_PARQUET)
    list(APPEND FEATURE_LIST "parquet")
endif()
if(HEDL_FEATURE_NEO4J)
    list(APPEND FEATURE_LIST "neo4j")
endif()

# Join features with commas
if(FEATURE_LIST)
    string(REPLACE ";" "," CARGO_FEATURES "${FEATURE_LIST}")
    set(CARGO_FEATURES "--features=${CARGO_FEATURES}")
    message(STATUS "Enabled features: ${FEATURE_LIST}")
else()
    set(CARGO_FEATURES "--no-default-features")
    message(STATUS "Building minimal configuration (no format converters)")
endif()

# ============================================================================
# Custom Build Commands for Rust Library
# ============================================================================

set(HEDL_FFI_CRATE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../crates/hedl-ffi")
set(HEDL_HEADER_SOURCE "${HEDL_FFI_CRATE_DIR}/include/hedl.h")

# Build shared library if requested
if(HEDL_BUILD_SHARED)
    add_custom_command(
        OUTPUT "${RUST_OUTPUT_DIR}/${HEDL_SHARED_LIB_NAME}"
        COMMAND ${CARGO_EXECUTABLE} build
            ${RUST_BUILD_FLAG}
            ${CARGO_FEATURES}
            --manifest-path "${HEDL_FFI_CRATE_DIR}/Cargo.toml"
        WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../../"
        COMMENT "Building HEDL shared library with Cargo (${RUST_BUILD_TYPE})"
        VERBATIM
    )

    add_custom_target(hedl_ffi_build_shared
        DEPENDS "${RUST_OUTPUT_DIR}/${HEDL_SHARED_LIB_NAME}"
    )
endif()

# Build static library if requested
if(HEDL_BUILD_STATIC)
    add_custom_command(
        OUTPUT "${RUST_OUTPUT_DIR}/${HEDL_STATIC_LIB_NAME}"
        COMMAND ${CARGO_EXECUTABLE} build
            ${RUST_BUILD_FLAG}
            ${CARGO_FEATURES}
            --manifest-path "${HEDL_FFI_CRATE_DIR}/Cargo.toml"
        WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../../"
        COMMENT "Building HEDL static library with Cargo (${RUST_BUILD_TYPE})"
        VERBATIM
    )

    add_custom_target(hedl_ffi_build_static
        DEPENDS "${RUST_OUTPUT_DIR}/${HEDL_STATIC_LIB_NAME}"
    )
endif()

# ============================================================================
# Create CMake Import Targets
# ============================================================================

# Shared library target
if(HEDL_BUILD_SHARED)
    add_library(HEDL::hedl SHARED IMPORTED GLOBAL)
    add_dependencies(HEDL::hedl hedl_ffi_build_shared)

    set_target_properties(HEDL::hedl PROPERTIES
        IMPORTED_LOCATION "${RUST_OUTPUT_DIR}/${HEDL_SHARED_LIB_NAME}"
        INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/include"
    )

    # On Windows, set import library
    if(WIN32 AND EXISTS "${RUST_OUTPUT_DIR}/${HEDL_IMPLIB_NAME}")
        set_target_properties(HEDL::hedl PROPERTIES
            IMPORTED_IMPLIB "${RUST_OUTPUT_DIR}/${HEDL_IMPLIB_NAME}"
        )
    endif()

    # Install shared library
    if(HEDL_INSTALL)
        install(FILES "${RUST_OUTPUT_DIR}/${HEDL_SHARED_LIB_NAME}"
            DESTINATION lib
            COMPONENT runtime
        )
    endif()
endif()

# Static library target
if(HEDL_BUILD_STATIC)
    add_library(HEDL::hedl_static STATIC IMPORTED GLOBAL)
    add_dependencies(HEDL::hedl_static hedl_ffi_build_static)

    set_target_properties(HEDL::hedl_static PROPERTIES
        IMPORTED_LOCATION "${RUST_OUTPUT_DIR}/${HEDL_STATIC_LIB_NAME}"
        INTERFACE_INCLUDE_DIRECTORIES "${CMAKE_CURRENT_SOURCE_DIR}/include"
    )

    # Static library needs additional system libraries
    if(UNIX AND NOT APPLE)
        set_target_properties(HEDL::hedl_static PROPERTIES
            INTERFACE_LINK_LIBRARIES "pthread;dl;m"
        )
    elseif(APPLE)
        set_target_properties(HEDL::hedl_static PROPERTIES
            INTERFACE_LINK_LIBRARIES "-framework Security"
        )
    elseif(WIN32)
        set_target_properties(HEDL::hedl_static PROPERTIES
            INTERFACE_LINK_LIBRARIES "ws2_32;userenv;bcrypt;ntdll"
        )
    endif()

    # Install static library
    if(HEDL_INSTALL)
        install(FILES "${RUST_OUTPUT_DIR}/${HEDL_STATIC_LIB_NAME}"
            DESTINATION lib
            COMPONENT development
        )
    endif()
endif()

# ============================================================================
# Copy and Install Header Files
# ============================================================================

# Copy header to build include directory
file(MAKE_DIRECTORY "${CMAKE_CURRENT_BINARY_DIR}/include")
add_custom_command(
    OUTPUT "${CMAKE_CURRENT_BINARY_DIR}/include/hedl.h"
    COMMAND ${CMAKE_COMMAND} -E copy
        "${HEDL_HEADER_SOURCE}"
        "${CMAKE_CURRENT_BINARY_DIR}/include/hedl.h"
    DEPENDS "${HEDL_HEADER_SOURCE}"
    COMMENT "Copying hedl.h header file"
)

add_custom_target(hedl_copy_headers ALL
    DEPENDS "${CMAKE_CURRENT_BINARY_DIR}/include/hedl.h"
)

# Install headers
if(HEDL_INSTALL)
    install(FILES "${HEDL_HEADER_SOURCE}"
        DESTINATION include
        COMPONENT development
    )
endif()

# ============================================================================
# Install CMake Package Configuration
# ============================================================================

if(HEDL_INSTALL)
    include(CMakePackageConfigHelpers)

    # Generate version file
    write_basic_package_version_file(
        "${CMAKE_CURRENT_BINARY_DIR}/HEDLConfigVersion.cmake"
        VERSION ${PROJECT_VERSION}
        COMPATIBILITY SameMajorVersion
    )

    # Configure package config file
    configure_package_config_file(
        "${CMAKE_CURRENT_SOURCE_DIR}/cmake/HEDLConfig.cmake.in"
        "${CMAKE_CURRENT_BINARY_DIR}/HEDLConfig.cmake"
        INSTALL_DESTINATION lib/cmake/HEDL
    )

    # Install CMake config files
    install(
        FILES
            "${CMAKE_CURRENT_BINARY_DIR}/HEDLConfig.cmake"
            "${CMAKE_CURRENT_BINARY_DIR}/HEDLConfigVersion.cmake"
        DESTINATION lib/cmake/HEDL
        COMPONENT development
    )
endif()

# ============================================================================
# Examples
# ============================================================================

if(HEDL_BUILD_EXAMPLES)
    add_subdirectory(examples)
endif()

# ============================================================================
# Tests
# ============================================================================

if(HEDL_BUILD_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()

# ============================================================================
# Print Configuration Summary
# ============================================================================

message(STATUS "")
message(STATUS "HEDL Configuration Summary")
message(STATUS "==========================")
message(STATUS "Version:              ${PROJECT_VERSION}")
message(STATUS "Build type:           ${CMAKE_BUILD_TYPE}")
message(STATUS "Rust build type:      ${RUST_BUILD_TYPE}")
message(STATUS "Build shared:         ${HEDL_BUILD_SHARED}")
message(STATUS "Build static:         ${HEDL_BUILD_STATIC}")
message(STATUS "Build examples:       ${HEDL_BUILD_EXAMPLES}")
message(STATUS "Build tests:          ${HEDL_BUILD_TESTS}")
message(STATUS "Install:              ${HEDL_INSTALL}")
message(STATUS "")
message(STATUS "Enabled Features:")
message(STATUS "  JSON:               ${HEDL_FEATURE_JSON}")
message(STATUS "  YAML:               ${HEDL_FEATURE_YAML}")
message(STATUS "  XML:                ${HEDL_FEATURE_XML}")
message(STATUS "  CSV:                ${HEDL_FEATURE_CSV}")
message(STATUS "  Parquet:            ${HEDL_FEATURE_PARQUET}")
message(STATUS "  Neo4j:              ${HEDL_FEATURE_NEO4J}")
message(STATUS "")
