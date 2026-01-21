// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive error handling tests for CLI error types and edge cases

use hedl_cli::error::{CliError, ErrorCategory};
use std::io;
use std::path::PathBuf;

// ===== CliError Construction Tests =====

#[test]
fn test_io_error_construction() {
    let err = CliError::io_error(
        "test.hedl",
        io::Error::new(io::ErrorKind::NotFound, "file not found"),
    );

    let msg = err.to_string();
    assert!(msg.contains("test.hedl"));
    assert!(msg.contains("file not found"));
}

#[test]
fn test_io_error_with_pathbuf() {
    let path = PathBuf::from("/path/to/file.hedl");
    let err = CliError::io_error(
        path.clone(),
        io::Error::new(io::ErrorKind::PermissionDenied, "access denied"),
    );

    let msg = err.to_string();
    assert!(msg.contains("file.hedl"));
    assert!(msg.contains("access denied"));
}

#[test]
fn test_file_too_large_error() {
    let err = CliError::file_too_large("huge.hedl", 200_000_000, 100 * 1024 * 1024);

    let msg = err.to_string();
    assert!(msg.contains("huge.hedl"));
    assert!(msg.contains("200000000 bytes"));
    assert!(msg.contains("100 MB"));
}

#[test]
fn test_file_too_large_boundary() {
    let max_size = 1024 * 1024; // 1 MB
    let err = CliError::file_too_large("file.hedl", max_size + 1, max_size);

    let msg = err.to_string();
    assert!(msg.contains("1048577 bytes")); // max + 1
    assert!(msg.contains("1 MB"));
}

#[test]
fn test_io_timeout_error() {
    let err = CliError::io_timeout("/slow/filesystem/file.hedl", 30);

    let msg = err.to_string();
    assert!(msg.contains("/slow/filesystem/file.hedl"));
    assert!(msg.contains("30 seconds"));
}

#[test]
fn test_parse_error() {
    let err = CliError::parse("unexpected token at line 5");

    assert_eq!(err.to_string(), "Parse error: unexpected token at line 5");
}

#[test]
fn test_canonicalization_error() {
    let err = CliError::canonicalization("failed to canonicalize document");

    assert!(err.to_string().contains("failed to canonicalize document"));
}

#[test]
fn test_json_conversion_error() {
    let err = CliError::json_conversion("invalid JSON structure");

    assert!(err.to_string().contains("invalid JSON structure"));
}

#[test]
fn test_yaml_conversion_error() {
    let err = CliError::yaml_conversion("YAML indentation error");

    assert!(err.to_string().contains("YAML indentation error"));
}

#[test]
fn test_xml_conversion_error() {
    let err = CliError::xml_conversion("XML tag mismatch");

    assert!(err.to_string().contains("XML tag mismatch"));
}

#[test]
fn test_csv_conversion_error() {
    let err = CliError::csv_conversion("CSV column count mismatch");

    assert!(err.to_string().contains("CSV column count mismatch"));
}

#[test]
fn test_parquet_conversion_error() {
    let err = CliError::parquet_conversion("Parquet schema incompatible");

    assert!(err.to_string().contains("Parquet schema incompatible"));
}

#[test]
fn test_lint_errors() {
    let err = CliError::LintErrors;

    assert_eq!(err.to_string(), "Lint errors found");
}

#[test]
fn test_not_canonical_error() {
    let err = CliError::NotCanonical;

    assert_eq!(err.to_string(), "File is not in canonical form");
}

#[test]
fn test_invalid_input_error() {
    let err = CliError::invalid_input("Type name must be alphanumeric");

    assert_eq!(
        err.to_string(),
        "Invalid input: Type name must be alphanumeric"
    );
}

#[test]
fn test_thread_pool_error() {
    let err = CliError::thread_pool_error("thread count must be positive", 0);

    let msg = err.to_string();
    assert!(msg.contains("thread count must be positive"));
}

#[test]
fn test_glob_pattern_error() {
    let err = CliError::GlobPattern {
        pattern: "[invalid".to_string(),
        message: "unclosed character class".to_string(),
    };

    let msg = err.to_string();
    assert!(msg.contains("[invalid"));
    assert!(msg.contains("unclosed character class"));
}

#[test]
fn test_no_files_matched_error() {
    let err = CliError::NoFilesMatched {
        patterns: vec!["*.hedl".to_string(), "test/*.hedl".to_string()],
    };

    let msg = err.to_string();
    assert!(msg.contains("*.hedl"));
    assert!(msg.contains("test/*.hedl"));
}

#[test]
fn test_directory_traversal_error() {
    let err = CliError::DirectoryTraversal {
        path: PathBuf::from("/restricted"),
        message: "permission denied".to_string(),
    };

    let msg = err.to_string();
    assert!(msg.contains("/restricted"));
    assert!(msg.contains("permission denied"));
}

#[test]
fn test_resource_exhaustion_error() {
    let err = CliError::ResourceExhaustion {
        resource_type: "file_handles".to_string(),
        message: "too many open files".to_string(),
        current_usage: 1024,
        limit: 1024,
    };

    let msg = err.to_string();
    assert!(msg.contains("file_handles"));
    assert!(msg.contains("too many open files"));
    assert!(msg.contains("1024/1024"));
}

// ===== Error Similarity Tests =====

#[test]
fn test_similar_to_same_variant() {
    let err1 = CliError::parse("error 1");
    let err2 = CliError::parse("error 2");

    assert!(err1.similar_to(&err2));
}

#[test]
fn test_similar_to_different_variant() {
    let err1 = CliError::parse("parse error");
    let err2 = CliError::NotCanonical;

    assert!(!err1.similar_to(&err2));
}

#[test]
fn test_similar_to_io_errors() {
    let err1 = CliError::io_error("file1.hedl", io::Error::from(io::ErrorKind::NotFound));
    let err2 = CliError::io_error(
        "file2.hedl",
        io::Error::from(io::ErrorKind::PermissionDenied),
    );

    assert!(err1.similar_to(&err2));
}

#[test]
fn test_similar_to_conversion_errors() {
    let err1 = CliError::json_conversion("error 1");
    let err2 = CliError::json_conversion("error 2");

    assert!(err1.similar_to(&err2));

    let err3 = CliError::yaml_conversion("error 3");
    assert!(!err1.similar_to(&err3));
}

// ===== Error Category Tests =====

#[test]
fn test_error_category_io_error() {
    let err = CliError::io_error("file.hedl", io::Error::from(io::ErrorKind::NotFound));
    assert!(matches!(err.category(), ErrorCategory::IoError));
}

#[test]
fn test_error_category_file_too_large() {
    let err = CliError::file_too_large("file.hedl", 2000, 1000);
    assert!(matches!(err.category(), ErrorCategory::IoError));
}

#[test]
fn test_error_category_io_timeout() {
    let err = CliError::io_timeout("file.hedl", 30);
    assert!(matches!(err.category(), ErrorCategory::IoError));
}

#[test]
fn test_error_category_parse_error() {
    let err = CliError::parse("syntax error");
    assert!(matches!(err.category(), ErrorCategory::ParseError));
}

#[test]
fn test_error_category_format_error() {
    let err = CliError::canonicalization("canonicalization failed");
    assert!(matches!(err.category(), ErrorCategory::FormatError));

    let err = CliError::NotCanonical;
    assert!(matches!(err.category(), ErrorCategory::FormatError));
}

#[test]
fn test_error_category_lint_error() {
    let err = CliError::LintErrors;
    assert!(matches!(err.category(), ErrorCategory::LintError));
}

#[test]
fn test_error_category_file_discovery() {
    let err = CliError::GlobPattern {
        pattern: "invalid".to_string(),
        message: "error".to_string(),
    };
    assert!(matches!(err.category(), ErrorCategory::FileDiscoveryError));

    let err = CliError::NoFilesMatched {
        patterns: vec!["*.hedl".to_string()],
    };
    assert!(matches!(err.category(), ErrorCategory::FileDiscoveryError));

    let err = CliError::DirectoryTraversal {
        path: PathBuf::from("/path"),
        message: "error".to_string(),
    };
    assert!(matches!(err.category(), ErrorCategory::FileDiscoveryError));
}

#[test]
fn test_error_category_resource_error() {
    let err = CliError::ResourceExhaustion {
        resource_type: "memory".to_string(),
        message: "out of memory".to_string(),
        current_usage: 100,
        limit: 100,
    };
    assert!(matches!(err.category(), ErrorCategory::ResourceError));

    let err = CliError::thread_pool_error("error", 0);
    assert!(matches!(err.category(), ErrorCategory::ResourceError));
}

#[test]
fn test_error_category_conversion_error() {
    let err = CliError::json_conversion("error");
    assert!(matches!(err.category(), ErrorCategory::ConversionError));

    let err = CliError::yaml_conversion("error");
    assert!(matches!(err.category(), ErrorCategory::ConversionError));

    let err = CliError::xml_conversion("error");
    assert!(matches!(err.category(), ErrorCategory::ConversionError));

    let err = CliError::csv_conversion("error");
    assert!(matches!(err.category(), ErrorCategory::ConversionError));

    let err = CliError::parquet_conversion("error");
    assert!(matches!(err.category(), ErrorCategory::ConversionError));
}

#[test]
fn test_error_category_validation_error() {
    let err = CliError::invalid_input("invalid type name");
    assert!(matches!(err.category(), ErrorCategory::ValidationError));
}

// ===== Error Cloning Tests =====

#[test]
fn test_error_clone_preserves_message() {
    let err = CliError::parse("original error");
    let cloned = err.clone();

    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn test_error_clone_io_error() {
    let err = CliError::io_error("file.hedl", io::Error::from(io::ErrorKind::NotFound));
    let cloned = err.clone();

    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn test_error_clone_complex_variant() {
    let err = CliError::ResourceExhaustion {
        resource_type: "memory".to_string(),
        message: "exhausted".to_string(),
        current_usage: 1024,
        limit: 1024,
    };
    let cloned = err.clone();

    assert_eq!(err.to_string(), cloned.to_string());
}

// ===== serde_json::Error Conversion Test =====

#[test]
fn test_json_error_conversion() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let cli_err: CliError = json_err.into();

    assert!(matches!(cli_err, CliError::JsonFormat { .. }));
    assert!(cli_err.to_string().contains("JSON"));
}

// ===== Error Display Tests =====

#[test]
fn test_error_display_contains_context() {
    let err = CliError::io_error(
        "/path/to/file.hedl",
        io::Error::new(io::ErrorKind::PermissionDenied, "access denied"),
    );

    let display = format!("{err}");
    assert!(display.contains("/path/to/file.hedl"));
    assert!(display.contains("access denied"));
}

#[test]
fn test_error_display_file_too_large_readable() {
    let err = CliError::file_too_large("big.hedl", 500 * 1024 * 1024, 100 * 1024 * 1024);

    let display = format!("{err}");
    assert!(display.contains("big.hedl"));
    assert!(display.contains("100 MB")); // Readable format
}

// ===== Edge Cases =====

#[test]
fn test_empty_patterns_no_files_matched() {
    let err = CliError::NoFilesMatched { patterns: vec![] };

    let msg = err.to_string();
    assert!(msg.contains("no files matched"));
}

#[test]
fn test_zero_timeout() {
    let err = CliError::io_timeout("file.hedl", 0);

    let msg = err.to_string();
    assert!(msg.contains("0 seconds"));
}

#[test]
fn test_zero_thread_count() {
    let err = CliError::thread_pool_error("invalid thread count", 0);

    let msg = err.to_string();
    // Should contain the requested thread count
    assert!(msg.contains("thread"));
}

#[test]
fn test_resource_exhaustion_at_limit() {
    let err = CliError::ResourceExhaustion {
        resource_type: "connections".to_string(),
        message: "at limit".to_string(),
        current_usage: 100,
        limit: 100,
    };

    let msg = err.to_string();
    assert!(msg.contains("100/100"));
}

#[test]
fn test_resource_exhaustion_over_limit() {
    let err = CliError::ResourceExhaustion {
        resource_type: "memory".to_string(),
        message: "exceeded".to_string(),
        current_usage: 150,
        limit: 100,
    };

    let msg = err.to_string();
    assert!(msg.contains("150/100"));
}

// ===== ErrorCategory Equality Tests =====

#[test]
fn test_error_category_equality() {
    assert_eq!(ErrorCategory::IoError, ErrorCategory::IoError);
    assert_ne!(ErrorCategory::IoError, ErrorCategory::ParseError);
}

#[test]
fn test_error_category_debug() {
    let category = ErrorCategory::ConversionError;
    let debug_str = format!("{category:?}");
    assert!(debug_str.contains("ConversionError"));
}

#[test]
fn test_error_category_clone() {
    let category = ErrorCategory::ResourceError;
    let cloned = category;
    assert_eq!(category, cloned);
}
