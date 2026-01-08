// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for error kind classification in hedl-parquet.
//!
//! This module verifies that file I/O errors are correctly classified as
//! HedlErrorKind::IO rather than other error kinds.

use hedl_core::{Document, HedlErrorKind, Item, MatrixList, Node, Value};
use hedl_parquet::{from_parquet, to_parquet};
use std::path::Path;
use tempfile::TempDir;

// =============================================================================
// File I/O Error Tests
// =============================================================================

#[test]
fn test_file_not_found_returns_io_error() {
    let non_existent_path = Path::new("/tmp/nonexistent_hedl_parquet_file_12345.parquet");

    let result = from_parquet(non_existent_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.kind,
        HedlErrorKind::IO,
        "File not found should return IO error, got {:?}",
        err.kind
    );
    assert!(
        err.message.contains("Failed to open Parquet file"),
        "Error message should mention file opening: {}",
        err.message
    );
}

#[test]
fn test_write_to_invalid_path_returns_io_error() {
    let doc = Document::new((1, 0));

    // Try to write to a path that doesn't exist (parent directory doesn't exist)
    let invalid_path = Path::new("/tmp/nonexistent_dir_12345/output.parquet");

    let result = to_parquet(&doc, invalid_path);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(
        err.kind,
        HedlErrorKind::IO,
        "Write to invalid path should return IO error, got {:?}",
        err.kind
    );
    assert!(
        err.message.contains("Failed to write Parquet file"),
        "Error message should mention file writing: {}",
        err.message
    );
}

#[test]
fn test_write_to_read_only_directory_returns_io_error() {
    // Create a temporary directory
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.parquet");

    // Create a simple document
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(42)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    // First write should succeed
    let result = to_parquet(&doc, &file_path);
    assert!(result.is_ok(), "Initial write should succeed");

    // Make the file read-only
    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&file_path, perms).unwrap();

        // Try to write again - should fail with IO error
        let result = to_parquet(&doc, &file_path);

        if result.is_err() {
            let err = result.unwrap_err();
            assert_eq!(
                err.kind,
                HedlErrorKind::IO,
                "Write to read-only file should return IO error, got {:?}",
                err.kind
            );
        }
        // Note: On some systems, overwriting might still succeed, so we only check if it fails
    }
}

#[test]
fn test_read_corrupted_parquet_file_returns_io_error() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("corrupted.parquet");

    // Write some invalid data (not valid Parquet)
    std::fs::write(&file_path, b"not a valid parquet file").unwrap();

    let result = from_parquet(&file_path);

    assert!(result.is_err());
    let err = result.unwrap_err();

    // Should be IO error since the Parquet reader failed
    assert_eq!(
        err.kind,
        HedlErrorKind::IO,
        "Reading corrupted Parquet file should return IO error, got {:?}",
        err.kind
    );
}

// =============================================================================
// Successful I/O Tests (verify no IO errors on success)
// =============================================================================

#[test]
fn test_successful_write_and_read_no_io_error() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("success.parquet");

    // Create a simple document
    let mut doc = Document::new((1, 0));
    let mut list = MatrixList::new("Item", vec!["id".to_string(), "value".to_string()]);
    list.add_row(Node::new(
        "Item",
        "i1",
        vec![Value::String("i1".to_string()), Value::Int(42)],
    ));
    doc.root.insert("items".to_string(), Item::List(list));

    // Write should succeed
    let write_result = to_parquet(&doc, &file_path);
    assert!(write_result.is_ok(), "Write should succeed");

    // Read should succeed
    let read_result = from_parquet(&file_path);
    assert!(read_result.is_ok(), "Read should succeed");
}

// =============================================================================
// Error Message Tests
// =============================================================================

#[test]
fn test_io_error_messages_are_descriptive() {
    let non_existent_path = Path::new("/tmp/nonexistent_hedl_parquet_file_67890.parquet");

    let result = from_parquet(non_existent_path);

    assert!(result.is_err());
    let err = result.unwrap_err();

    // Error message should be descriptive
    assert!(
        err.message.len() > 20,
        "Error message should be descriptive, got: {}",
        err.message
    );

    // Should mention the specific operation
    assert!(
        err.message.contains("Parquet") || err.message.contains("file"),
        "Error message should mention Parquet or file: {}",
        err.message
    );
}

#[test]
fn test_io_error_display_format() {
    let err = hedl_core::HedlError::io("Test I/O error message");

    let display = format!("{}", err);

    assert!(
        display.contains("IOError"),
        "Display format should contain 'IOError': {}",
        display
    );
    assert!(
        display.contains("Test I/O error message"),
        "Display format should contain the message: {}",
        display
    );
}
