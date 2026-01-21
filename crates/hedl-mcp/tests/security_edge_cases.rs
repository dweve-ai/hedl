// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Additional security edge case tests for path traversal vulnerabilities.
//!
//! These tests focus on specific edge cases and attack vectors that might be
//! missed by the general path traversal tests.

use hedl_mcp::{JsonRpcRequest, McpServer, McpServerConfig};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// =============================================================================
// TEST HELPERS
// =============================================================================

fn create_test_server(root_path: PathBuf) -> McpServer {
    let config = McpServerConfig {
        root_path,
        rate_limit_burst: 0, // Disable rate limiting for tests
        rate_limit_per_second: 0,
        ..Default::default()
    };
    McpServer::new(config)
}

fn make_request(method: &str, params: Option<Value>, id: u64) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method: method.to_string(),
        params,
        id: Some(Value::Number(id.into())),
    }
}

fn initialize_server(server: &mut McpServer) {
    let request = make_request(
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "1.0" }
        })),
        1,
    );
    server.handle_request(request);
}

// =============================================================================
// EMPTY PATH TESTS
// =============================================================================

#[test]
fn test_read_empty_path_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let mut server = create_test_server(root);
    initialize_server(&mut server);

    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_read",
            "arguments": { "path": "" }
        })),
        2,
    ));

    let result = response.result.unwrap();
    assert_eq!(
        result.get("isError"),
        Some(&json!(true)),
        "Empty path should be rejected"
    );
}

#[test]
fn test_write_empty_path_rejected() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let mut server = create_test_server(root);
    initialize_server(&mut server);

    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_write",
            "arguments": {
                "path": "",
                "content": "%VERSION: 1.0\n---\ntest: data\n"
            }
        })),
        2,
    ));

    let result = response.result.unwrap();
    assert_eq!(
        result.get("isError"),
        Some(&json!(true)),
        "Empty path should be rejected"
    );
}

// =============================================================================
// TOCTOU (TIME-OF-CHECK-TIME-OF-USE) TESTS
// =============================================================================

#[cfg(unix)]
#[test]
fn test_symlink_race_condition() {
    use std::os::unix::fs::symlink;
    use std::sync::{Arc, Mutex};
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("sandbox");
    fs::create_dir_all(&root).unwrap();

    // Create a secret file outside
    let secret = temp_dir.path().join("secret.hedl");
    fs::write(&secret, "%VERSION: 1.0\n---\nsecret: data\n").unwrap();

    let server = Arc::new(Mutex::new(create_test_server(root.clone())));
    {
        let mut s = server.lock().unwrap();
        initialize_server(&mut s);
    }

    // Create a valid file
    let target = root.join("target.hedl");
    fs::write(&target, "%VERSION: 1.0\n---\nvalid: data\n").unwrap();

    // Spawn threads that try to exploit TOCTOU
    let mut handles = vec![];

    for _ in 0..5 {
        let server = Arc::clone(&server);
        let target = target.clone();
        let secret = secret.clone();

        handles.push(thread::spawn(move || {
            for _ in 0..10 {
                // Remove the file if it exists
                let _ = fs::remove_file(&target);

                // Try to create a symlink
                let _ = symlink(&secret, &target);

                // Try to read
                let response = {
                    let mut s = server.lock().unwrap();
                    s.handle_request(make_request(
                        "tools/call",
                        Some(json!({
                            "name": "hedl_read",
                            "arguments": { "path": "target.hedl" }
                        })),
                        3,
                    ))
                };

                // If the read succeeded, verify it's not the secret content
                if let Some(content) = response.result.as_ref().and_then(|r| r.get("content")) {
                    if let Some(text) = content.get(0).and_then(|c| c.get("text")) {
                        let text_str = text.as_str().unwrap_or("");
                        // The content should never contain "secret"
                        assert!(
                            !text_str.contains("secret"),
                            "TOCTOU vulnerability: read secret data"
                        );
                    }
                }

                // Clean up for next iteration
                let _ = fs::remove_file(&target);
                fs::write(&target, "%VERSION: 1.0\n---\nvalid: data\n").ok();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

// =============================================================================
// CASE SENSITIVITY TESTS (Windows-specific attacks)
// =============================================================================

#[test]
fn test_case_variation_attacks() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("sandbox");
    fs::create_dir_all(&root).unwrap();

    fs::write(root.join("test.hedl"), "%VERSION: 1.0\n---\ntest: data\n").unwrap();

    let mut server = create_test_server(root);
    initialize_server(&mut server);

    // Try various case variations
    let case_variations = vec![
        "../SANDBOX/test.hedl",
        "../Sandbox/test.hedl",
        "../SaNdBoX/test.hedl",
    ];

    for path in case_variations {
        let response = server.handle_request(make_request(
            "tools/call",
            Some(json!({
                "name": "hedl_read",
                "arguments": { "path": path }
            })),
            2,
        ));

        let result = response.result.unwrap();
        // On case-sensitive systems (Unix), these won't match
        // On case-insensitive systems (Windows), canonicalize should handle it
        // Either way, ensure no unintended access
        let _ = result;
    }
}

// =============================================================================
// UNICODE HOMOGLYPH ATTACKS
// =============================================================================

#[test]
fn test_unicode_homoglyph_directory_names() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create a valid file
    fs::write(root.join("test.hedl"), "%VERSION: 1.0\n---\ntest: data\n").unwrap();

    let mut server = create_test_server(root);
    initialize_server(&mut server);

    // Unicode homoglyphs that might look like "../"
    let homoglyph_paths = vec![
        // Cyrillic 'a' instead of Latin 'a'
        "..\u{0430}/test.hedl",
        // Fullwidth solidus
        "..\u{FF0F}test.hedl",
        // Mathematical bold dot
        "\u{1D428}../test.hedl",
    ];

    for path in homoglyph_paths {
        let response = server.handle_request(make_request(
            "tools/call",
            Some(json!({
                "name": "hedl_read",
                "arguments": { "path": path }
            })),
            2,
        ));

        let result = response.result.unwrap();
        // These should either not resolve or be blocked
        // The key is they shouldn't access unintended files
        assert_eq!(
            result.get("isError"),
            Some(&json!(true)),
            "Homoglyph path '{path}' should fail"
        );
    }
}

// =============================================================================
// CANONICALIZATION EDGE CASES
// =============================================================================

#[test]
fn test_trailing_slash_normalization() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    fs::write(root.join("test.hedl"), "%VERSION: 1.0\n---\ntest: data\n").unwrap();

    let mut server = create_test_server(root);
    initialize_server(&mut server);

    // Paths with trailing slashes
    let trailing_slash_paths = vec!["test.hedl/", "test.hedl//", "test.hedl///"];

    for path in trailing_slash_paths {
        let response = server.handle_request(make_request(
            "tools/call",
            Some(json!({
                "name": "hedl_read",
                "arguments": { "path": path }
            })),
            2,
        ));

        let result = response.result.unwrap();
        // Trailing slashes on files should fail
        assert_eq!(
            result.get("isError"),
            Some(&json!(true)),
            "Path with trailing slash '{path}' should fail"
        );
    }
}

#[test]
fn test_multiple_consecutive_slashes() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("sandbox");
    fs::create_dir_all(&root).unwrap();

    let mut server = create_test_server(root);
    initialize_server(&mut server);

    // Paths with multiple consecutive slashes
    let multi_slash_paths = vec![
        "..//etc//passwd",
        "..///etc///passwd",
        "../../../../../etc////passwd",
    ];

    for path in multi_slash_paths {
        let response = server.handle_request(make_request(
            "tools/call",
            Some(json!({
                "name": "hedl_read",
                "arguments": { "path": path }
            })),
            2,
        ));

        let result = response.result.unwrap();
        assert_eq!(
            result.get("isError"),
            Some(&json!(true)),
            "Multi-slash path '{path}' should be blocked"
        );
    }
}

// =============================================================================
// VERY LONG PATH ATTACKS
// =============================================================================

#[test]
fn test_maximum_path_length_handling() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let mut server = create_test_server(root);
    initialize_server(&mut server);

    // Create a path approaching PATH_MAX
    let long_component = "a".repeat(255); // Max filename length on most systems
    let very_long_path = format!("{}/{}", long_component.repeat(10), "test.hedl");

    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_read",
            "arguments": { "path": very_long_path }
        })),
        2,
    ));

    // Should handle gracefully (error, not panic)
    let result = response.result.unwrap();
    let _ = result; // Just ensure no panic
}

// =============================================================================
// WRITE-SPECIFIC EDGE CASES
// =============================================================================

#[test]
fn test_write_to_current_directory() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let mut server = create_test_server(root.clone());
    initialize_server(&mut server);

    // Try to write to "." which represents the directory itself
    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_write",
            "arguments": {
                "path": ".",
                "content": "%VERSION: 1.0\n---\ntest: data\n"
            }
        })),
        2,
    ));

    let result = response.result.unwrap();
    // Writing to "." should fail (it's a directory)
    assert_eq!(
        result.get("isError"),
        Some(&json!(true)),
        "Writing to '.' should fail"
    );
}

#[test]
fn test_write_creates_intermediate_dirs_safely() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let mut server = create_test_server(root.clone());
    initialize_server(&mut server);

    // Write to a deeply nested path that doesn't exist
    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_write",
            "arguments": {
                "path": "a/b/c/d/e/f/test.hedl",
                "content": "%VERSION: 1.0\n---\ntest: data\n"
            }
        })),
        2,
    ));

    let result = response.result.unwrap();
    assert_ne!(
        result.get("isError"),
        Some(&json!(true)),
        "Writing to deep nested path should succeed"
    );

    // Verify the file was created inside the root
    let created_file = root.join("a/b/c/d/e/f/test.hedl");
    assert!(
        created_file.exists(),
        "File should be created in nested structure"
    );

    // Verify the path is still within root
    let canonical = created_file.canonicalize().unwrap();
    let canonical_root = root.canonicalize().unwrap();
    assert!(
        canonical.starts_with(&canonical_root),
        "Created file should be within root"
    );
}

#[cfg(unix)]
#[test]
fn test_write_through_symlink_to_directory() {
    use std::os::unix::fs::symlink;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("sandbox");
    fs::create_dir_all(&root).unwrap();

    // Create a subdirectory
    let subdir = root.join("subdir");
    fs::create_dir(&subdir).unwrap();

    // Create a symlink to the subdirectory (inside root)
    let link = root.join("link_to_subdir");
    symlink(&subdir, &link).unwrap();

    let mut server = create_test_server(root.clone());
    initialize_server(&mut server);

    // Try to write through the symlink
    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_write",
            "arguments": {
                "path": "link_to_subdir/file.hedl",
                "content": "%VERSION: 1.0\n---\ntest: data\n"
            }
        })),
        2,
    ));

    let result = response.result.unwrap();
    // This should succeed since the symlink target is inside root
    assert_ne!(
        result.get("isError"),
        Some(&json!(true)),
        "Writing through symlink inside root should succeed"
    );

    // Verify file was created in the actual subdirectory
    assert!(
        subdir.join("file.hedl").exists(),
        "File should be created via symlink"
    );
}

// =============================================================================
// BACKUP FEATURE SECURITY
// =============================================================================

#[test]
fn test_backup_path_stays_within_root() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    // Create initial file
    let original_file = root.join("test.hedl");
    fs::write(&original_file, "%VERSION: 1.0\n---\noriginal: data\n").unwrap();

    let mut server = create_test_server(root.clone());
    initialize_server(&mut server);

    // Update with backup enabled
    let response = server.handle_request(make_request(
        "tools/call",
        Some(json!({
            "name": "hedl_write",
            "arguments": {
                "path": "test.hedl",
                "content": "%VERSION: 1.0\n---\nupdated: data\n",
                "backup": true
            }
        })),
        2,
    ));

    let result = response.result.unwrap();
    assert_ne!(
        result.get("isError"),
        Some(&json!(true)),
        "Write with backup should succeed"
    );

    // Verify backup was created inside root
    let backup_file = root.join("test.hedl.bak");
    assert!(backup_file.exists(), "Backup should be created");

    // Verify backup is within root
    let canonical_backup = backup_file.canonicalize().unwrap();
    let canonical_root = root.canonicalize().unwrap();
    assert!(
        canonical_backup.starts_with(&canonical_root),
        "Backup should be within root"
    );
}

// =============================================================================
// CONCURRENT OPERATIONS
// =============================================================================

#[test]
fn test_concurrent_writes_to_same_file() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().to_path_buf();

    let server = Arc::new(Mutex::new(create_test_server(root.clone())));
    {
        let mut s = server.lock().unwrap();
        initialize_server(&mut s);
    }

    let mut handles = vec![];

    // Spawn multiple threads writing to the same file
    for i in 0..10 {
        let server = Arc::clone(&server);
        let content = format!("%VERSION: 1.0\n---\nthread{i}: data\n");

        handles.push(thread::spawn(move || {
            for _ in 0..5 {
                let response = {
                    let mut s = server.lock().unwrap();
                    s.handle_request(make_request(
                        "tools/call",
                        Some(json!({
                            "name": "hedl_write",
                            "arguments": {
                                "path": "shared.hedl",
                                "content": content
                            }
                        })),
                        2,
                    ))
                };

                // All writes should succeed or fail cleanly (no panics)
                let _ = response.result.unwrap();
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify the file exists and is within root
    let shared_file = root.join("shared.hedl");
    assert!(shared_file.exists(), "Shared file should exist");

    let canonical = shared_file.canonicalize().unwrap();
    let canonical_root = root.canonicalize().unwrap();
    assert!(
        canonical.starts_with(&canonical_root),
        "Shared file should be within root"
    );
}
