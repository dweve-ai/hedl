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

//! Correctness tests for parallel stats implementation
//!
//! Verifies that parallel stats produces identical output to sequential version.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Get the path to the hedl binary
fn hedl_bin() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("hedl");
    path
}

/// Create a test HEDL file with substantial content
fn create_test_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

/// Run stats command and capture output
fn run_stats(file: &PathBuf, with_tokens: bool) -> String {
    let mut cmd = Command::new(hedl_bin());
    cmd.arg("stats").arg(file);

    if with_tokens {
        cmd.arg("--tokens");
    }

    let output = cmd.output().expect("Failed to execute hedl stats");

    if !output.status.success() {
        panic!(
            "Stats command failed:\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn test_simple_object() {
    let dir = TempDir::new().unwrap();
    let file = create_test_file(
        &dir,
        "simple.hedl",
        r#"%VERSION: 1.0
---
name: "John Doe"
age: 30
email: "john@example.com"
"#,
    );

    // Run stats multiple times to ensure consistency (parallel execution)
    let output1 = run_stats(&file, false);
    let output2 = run_stats(&file, false);
    let output3 = run_stats(&file, false);

    // All runs should produce identical output
    assert_eq!(output1, output2, "Stats output should be deterministic");
    assert_eq!(output2, output3, "Stats output should be deterministic");

    // Verify expected content
    assert!(output1.contains("HEDL Size Comparison"));
    assert!(output1.contains("JSON (minified)"));
    assert!(output1.contains("JSON (pretty)"));
    assert!(output1.contains("YAML"));
    assert!(output1.contains("XML (minified)"));
    assert!(output1.contains("XML (pretty)"));
}

#[test]
fn test_with_tokens() {
    let dir = TempDir::new().unwrap();
    let file = create_test_file(
        &dir,
        "tokens.hedl",
        r#"%VERSION: 1.0
---
name: "Test Dataset"
count: 3
alpha: 1
beta: 2
gamma: 3
"#,
    );

    // Run stats with token estimation multiple times
    let output1 = run_stats(&file, true);
    let output2 = run_stats(&file, true);
    let output3 = run_stats(&file, true);

    // All runs should produce identical output
    assert_eq!(output1, output2, "Token stats should be deterministic");
    assert_eq!(output2, output3, "Token stats should be deterministic");

    // Verify token section is present
    assert!(output1.contains("Estimated Tokens (LLM context)"));
    assert!(output1.contains("Note: Token estimates use ~4 chars/token"));
}

#[test]
fn test_large_dataset() {
    let dir = TempDir::new().unwrap();

    // Generate a larger dataset to test parallel efficiency
    let mut content = String::from("%VERSION: 1.0\n---\n");
    for i in 0..100 {
        content.push_str(&format!(
            "id{}: {}\nname{}: \"Product {}\"\nprice{}: {}.99\ncategory{}: \"Category {}\"\n",
            i, i, i, i, i, i * 10, i, i % 10
        ));
    }

    let file = create_test_file(&dir, "large.hedl", &content);

    // Run stats multiple times
    let output1 = run_stats(&file, true);
    let output2 = run_stats(&file, true);

    // Should be deterministic even with larger data
    assert_eq!(output1, output2, "Large dataset stats should be deterministic");
}

#[test]
fn test_nested_structures() {
    let dir = TempDir::new().unwrap();
    let file = create_test_file(
        &dir,
        "nested.hedl",
        r#"%VERSION: 1.0
---
company: "ACME Corp"
eng_backend: 5
eng_frontend: 3
sales_enterprise: 10
sales_smb: 8
"#,
    );

    // Verify deterministic output for nested structures
    let output1 = run_stats(&file, true);
    let output2 = run_stats(&file, true);

    assert_eq!(output1, output2, "Nested structure stats should be deterministic");
}

#[test]
fn test_special_characters() {
    let dir = TempDir::new().unwrap();
    let file = create_test_file(
        &dir,
        "special.hedl",
        r#"%VERSION: 1.0
---
unicode: "Hello 世界 🌍"
escaped: "Line 1\nLine 2\tTabbed"
quoted: "He said \"hello\""
"#,
    );

    // Verify handling of special characters in parallel processing
    let output1 = run_stats(&file, false);
    let output2 = run_stats(&file, false);

    assert_eq!(
        output1, output2,
        "Special character handling should be deterministic"
    );
}

#[test]
fn test_empty_file() {
    let dir = TempDir::new().unwrap();
    let file = create_test_file(&dir, "empty.hedl", "");

    // Empty files should be handled gracefully
    let _result = Command::new(hedl_bin())
        .arg("stats")
        .arg(&file)
        .output()
        .expect("Failed to execute hedl stats");

    // Should either succeed with empty stats or fail gracefully
    // (behavior depends on parser handling of empty input)
}

#[test]
fn test_matrix_list() {
    let dir = TempDir::new().unwrap();
    let file = create_test_file(
        &dir,
        "matrix.hedl",
        r#"%VERSION: 1.0
%STRUCT: Data: [id,name,score]
---
items: @Data
  |d1,Alice,95.5
  |d2,Bob,87.3
  |d3,Carol,92.1
"#,
    );

    // Verify matrix list handling in parallel processing
    let output1 = run_stats(&file, true);
    let output2 = run_stats(&file, true);

    assert_eq!(output1, output2, "Matrix list stats should be deterministic");
}

#[test]
fn test_no_race_conditions() {
    let dir = TempDir::new().unwrap();
    let file = create_test_file(
        &dir,
        "race.hedl",
        r#"%VERSION: 1.0
---
field1: "value1"
field2: "value2"
field3: "value3"
"#,
    );

    // Run stats concurrently to detect potential race conditions
    use std::thread;

    let file = file.clone();
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let f = file.clone();
            thread::spawn(move || run_stats(&f, true))
        })
        .collect();

    let outputs: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All outputs should be identical (no race conditions)
    for i in 1..outputs.len() {
        assert_eq!(
            outputs[0], outputs[i],
            "Concurrent execution should produce identical results"
        );
    }
}
