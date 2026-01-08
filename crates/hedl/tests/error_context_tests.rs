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

//! Integration tests for error context helpers.

use hedl::{parse, HedlError, HedlErrorKind, HedlResultExt};

// ==================== Basic Context Tests ====================

#[test]
fn test_context_adds_context_to_error() {
    let result: Result<(), HedlError> = Err(HedlError::syntax("unexpected token", 10));
    let err = result.context("while parsing header").unwrap_err();

    assert_eq!(err.kind, HedlErrorKind::Syntax);
    assert_eq!(err.message, "unexpected token");
    assert_eq!(err.line, 10);
    assert_eq!(err.context, Some("while parsing header".to_string()));
}

#[test]
fn test_context_does_not_affect_ok() {
    let result: Result<i32, HedlError> = Ok(42);
    let value = result.context("this context is never used").unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_context_with_string() {
    let result: Result<(), HedlError> = Err(HedlError::reference("undefined @User:1", 15));
    let err = result
        .context("in users section".to_string())
        .unwrap_err();

    assert_eq!(err.context, Some("in users section".to_string()));
}

#[test]
fn test_context_with_format_macro() {
    let user = "alice";
    let line = 20;
    let result: Result<(), HedlError> = Err(HedlError::collision("duplicate ID", line));
    let err = result
        .context(format!("for user {} at line {}", user, line))
        .unwrap_err();

    let ctx = err.context.as_ref().unwrap();
    assert!(ctx.contains("alice"));
    assert!(ctx.contains("20"));
}

// ==================== Context Chaining Tests ====================

#[test]
fn test_context_can_be_chained() {
    let result: Result<(), HedlError> = Err(HedlError::schema("type mismatch", 5));
    let err = result
        .context("in struct definition")
        .context("while validating schema")
        .unwrap_err();

    let ctx = err.context.unwrap();
    assert!(ctx.contains("while validating schema"));
    assert!(ctx.contains("in struct definition"));
}

#[test]
fn test_multi_level_context_chaining() {
    fn level1() -> Result<(), HedlError> {
        Err(HedlError::semantic("invalid ditto usage", 3))
    }

    fn level2() -> Result<(), HedlError> {
        level1().context("in level2 function")
    }

    fn level3() -> Result<(), HedlError> {
        level2().context("in level3 function")
    }

    fn level4() -> Result<(), HedlError> {
        level3().context("in level4 function")
    }

    let err = level4().unwrap_err();
    let ctx = err.context.unwrap();

    // All levels should be present in context
    assert!(ctx.contains("level4"));
    assert!(ctx.contains("level3"));
    assert!(ctx.contains("level2"));
}

#[test]
fn test_context_preserves_order() {
    let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
    let err = result
        .context("first")
        .context("second")
        .context("third")
        .unwrap_err();

    let ctx = err.context.unwrap();
    let first_pos = ctx.find("first").unwrap();
    let second_pos = ctx.find("second").unwrap();
    let third_pos = ctx.find("third").unwrap();

    // Later contexts should appear earlier in the string
    assert!(third_pos < second_pos);
    assert!(second_pos < first_pos);
}

// ==================== Lazy Evaluation Tests ====================

#[test]
fn test_with_context_lazy_evaluation_on_ok() {
    let mut evaluated = false;
    let result: Result<i32, HedlError> = Ok(100);

    let value = result
        .with_context(|| {
            evaluated = true;
            "expensive context"
        })
        .unwrap();

    assert_eq!(value, 100);
    assert!(!evaluated, "Context closure should not be evaluated on Ok");
}

#[test]
fn test_with_context_evaluates_on_error() {
    let mut evaluated = false;
    let result: Result<(), HedlError> = Err(HedlError::alias("duplicate alias %user", 8));

    let err = result
        .with_context(|| {
            evaluated = true;
            "context was evaluated"
        })
        .unwrap_err();

    assert!(evaluated, "Context closure should be evaluated on Err");
    assert_eq!(err.context, Some("context was evaluated".to_string()));
}

#[test]
fn test_with_context_captures_environment() {
    let filename = "config.hedl";
    let section = "users";
    let line_number = 42;

    let result: Result<(), HedlError> = Err(HedlError::shape("wrong column count", line_number));
    let err = result
        .with_context(|| {
            format!(
                "in file {}, section {}, line {}",
                filename, section, line_number
            )
        })
        .unwrap_err();

    let ctx = err.context.unwrap();
    assert!(ctx.contains("config.hedl"));
    assert!(ctx.contains("users"));
    assert!(ctx.contains("42"));
}

#[test]
fn test_with_context_expensive_computation() {
    fn expensive_debug_info(data: &[u8]) -> String {
        // Simulate expensive operation
        format!("data length: {}, checksum: {}", data.len(), data.iter().map(|&b| b as u32).sum::<u32>())
    }

    let data = vec![1, 2, 3, 4, 5];
    let result: Result<(), HedlError> = Ok(());

    // This should not compute the expensive debug info
    let _ = result.with_context(|| expensive_debug_info(&data));

    // Now with an error
    let result_err: Result<(), HedlError> = Err(HedlError::io("read failed"));
    let err = result_err
        .with_context(|| expensive_debug_info(&data))
        .unwrap_err();

    assert!(err.context.unwrap().contains("data length: 5"));
}

#[test]
fn test_with_context_chaining() {
    let result: Result<(), HedlError> = Err(HedlError::orphan_row("row without parent", 12));
    let err = result
        .with_context(|| "in matrix list")
        .with_context(|| "in data section")
        .with_context(|| "while parsing document")
        .unwrap_err();

    let ctx = err.context.unwrap();
    assert!(ctx.contains("while parsing document"));
    assert!(ctx.contains("in data section"));
    assert!(ctx.contains("in matrix list"));
}

// ==================== Error Conversion Tests ====================

#[test]
fn test_map_err_to_hedl_io_error() {
    use std::fs;

    let result = fs::read_to_string("/path/that/does/not/exist");
    let hedl_result = result.map_err_to_hedl(|e| {
        HedlError::io(format!("Failed to read configuration: {}", e))
    });

    let err = hedl_result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
    assert!(err.message.contains("Failed to read configuration"));
}

#[test]
fn test_map_err_to_hedl_with_context() {
    use std::fs;

    let path = "/nonexistent/config.hedl";
    let result = fs::read_to_string(path)
        .map_err_to_hedl(|e| HedlError::io(format!("Cannot read {}: {}", path, e)))
        .context("while loading user configuration");

    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
    assert!(err.context.unwrap().contains("user configuration"));
}

#[test]
fn test_map_err_to_hedl_preserves_ok() {
    let io_result: Result<String, std::io::Error> = Ok("file content".to_string());
    let hedl_result = io_result.map_err_to_hedl(|e| HedlError::io(e.to_string()));

    assert_eq!(hedl_result.unwrap(), "file content");
}

#[test]
fn test_map_err_to_hedl_different_error_kinds() {
    use std::io;

    // Create different I/O error kinds and convert them
    let not_found = io::Error::new(io::ErrorKind::NotFound, "not found");
    let result1: Result<(), io::Error> = Err(not_found);
    let err1 = result1
        .map_err_to_hedl(|_| HedlError::io("NotFound error"))
        .unwrap_err();
    assert_eq!(err1.kind, HedlErrorKind::IO);

    let permission_denied = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let result2: Result<(), io::Error> = Err(permission_denied);
    let err2 = result2
        .map_err_to_hedl(|_| HedlError::io("Permission denied"))
        .unwrap_err();
    assert_eq!(err2.kind, HedlErrorKind::IO);
}

// ==================== Real-World Usage Tests ====================

#[test]
fn test_parse_with_context() {
    let invalid_hedl = "this is not valid HEDL syntax";
    let err = parse(invalid_hedl)
        .context("failed to parse user configuration")
        .unwrap_err();

    assert!(err.context.is_some());
    assert!(err.context.unwrap().contains("user configuration"));
}

#[test]
fn test_parse_with_lazy_context() {
    let invalid_hedl = "this is completely invalid HEDL syntax";
    let config_name = "production.hedl";

    let err = parse(invalid_hedl)
        .with_context(|| format!("while loading config file: {}", config_name))
        .unwrap_err();

    assert!(err.context.unwrap().contains("production.hedl"));
}

#[test]
fn test_nested_parse_operations() {
    fn parse_section(hedl: &str, section_name: &str) -> Result<hedl::Document, HedlError> {
        parse(hedl).with_context(|| format!("in section '{}'", section_name))
    }

    fn parse_document(hedl: &str, doc_name: &str) -> Result<hedl::Document, HedlError> {
        parse_section(hedl, "users")
            .with_context(|| format!("while parsing document '{}'", doc_name))
    }

    let invalid = "invalid hedl";
    let err = parse_document(invalid, "config.hedl").unwrap_err();

    let ctx = err.context.unwrap();
    assert!(ctx.contains("config.hedl"));
    assert!(ctx.contains("users"));
}

#[test]
fn test_file_io_with_parse() {
    use std::fs;

    fn load_and_parse(path: &str) -> Result<hedl::Document, HedlError> {
        let content = fs::read_to_string(path)
            .map_err_to_hedl(|e| HedlError::io(format!("Cannot read {}: {}", path, e)))
            .with_context(|| format!("accessing file {}", path))?;

        parse(&content).with_context(|| format!("parsing content of {}", path))
    }

    let err = load_and_parse("/this/does/not/exist.hedl").unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
}

// ==================== Edge Cases ====================

#[test]
fn test_empty_context_string() {
    let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
    let err = result.context("").unwrap_err();

    // Empty context should not be added
    assert_eq!(err.context, None);
}

#[test]
fn test_multiple_empty_contexts() {
    let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
    let err = result
        .context("")
        .context("")
        .context("real context")
        .context("")
        .unwrap_err();

    assert_eq!(err.context, Some("real context".to_string()));
}

#[test]
fn test_context_with_unicode() {
    let result: Result<(), HedlError> = Err(HedlError::semantic("invalid", 1));
    let err = result.context("日本語のエラー 🚀").unwrap_err();

    let ctx = err.context.unwrap();
    assert!(ctx.contains("日本語"));
    assert!(ctx.contains("🚀"));
}

#[test]
fn test_context_with_special_characters() {
    let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
    let err = result
        .context("special chars: \n\t\r\\\"'")
        .unwrap_err();

    let ctx = err.context.unwrap();
    assert!(ctx.contains("\\"));
    assert!(ctx.contains("\""));
    assert!(ctx.contains("'"));
}

#[test]
fn test_very_long_context() {
    let long_context = "x".repeat(10000);
    let result: Result<(), HedlError> = Err(HedlError::shape("error", 1));
    let err = result.context(long_context.clone()).unwrap_err();

    assert_eq!(err.context.unwrap().len(), 10000);
}

#[test]
fn test_context_with_newlines() {
    let multi_line = "Error occurred:\nLine 1\nLine 2\nLine 3";
    let result: Result<(), HedlError> = Err(HedlError::collision("error", 1));
    let err = result.context(multi_line).unwrap_err();

    let ctx = err.context.unwrap();
    assert!(ctx.contains("Line 1"));
    assert!(ctx.contains("Line 2"));
    assert!(ctx.contains("Line 3"));
}

#[test]
fn test_error_preserves_all_fields() {
    let original = HedlError::version("unsupported version 2.0", 1).with_column(10);
    let result: Result<(), HedlError> = Err(original);
    let err = result.context("additional context").unwrap_err();

    assert_eq!(err.kind, HedlErrorKind::Version);
    assert_eq!(err.message, "unsupported version 2.0");
    assert_eq!(err.line, 1);
    assert_eq!(err.column, Some(10));
    assert!(err.context.is_some());
}

// ==================== Combinatorial Tests ====================

#[test]
fn test_mix_context_and_with_context() {
    let result: Result<(), HedlError> = Err(HedlError::security("depth limit exceeded", 0));
    let err = result
        .context("layer 1")
        .with_context(|| "layer 2")
        .context("layer 3")
        .with_context(|| "layer 4")
        .unwrap_err();

    let ctx = err.context.unwrap();
    assert!(ctx.contains("layer 1"));
    assert!(ctx.contains("layer 2"));
    assert!(ctx.contains("layer 3"));
    assert!(ctx.contains("layer 4"));
}

#[test]
fn test_context_after_map_err_to_hedl() {
    use std::fs;

    let result = fs::read_to_string("/nonexistent")
        .map_err_to_hedl(|e| HedlError::io(e.to_string()))
        .context("in first layer")
        .context("in second layer");

    let err = result.unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
    let ctx = err.context.unwrap();
    assert!(ctx.contains("first layer"));
    assert!(ctx.contains("second layer"));
}

// ==================== Type Preservation Tests ====================

#[test]
fn test_context_preserves_error_kind_syntax() {
    let err = Err::<(), _>(HedlError::syntax("test", 1))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Syntax);
}

#[test]
fn test_context_preserves_error_kind_version() {
    let err = Err::<(), _>(HedlError::version("test", 1))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Version);
}

#[test]
fn test_context_preserves_error_kind_schema() {
    let err = Err::<(), _>(HedlError::schema("test", 1))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Schema);
}

#[test]
fn test_context_preserves_error_kind_alias() {
    let err = Err::<(), _>(HedlError::alias("test", 1))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Alias);
}

#[test]
fn test_context_preserves_error_kind_shape() {
    let err = Err::<(), _>(HedlError::shape("test", 1))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Shape);
}

#[test]
fn test_context_preserves_error_kind_semantic() {
    let err = Err::<(), _>(HedlError::semantic("test", 1))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Semantic);
}

#[test]
fn test_context_preserves_error_kind_orphan_row() {
    let err = Err::<(), _>(HedlError::orphan_row("test", 1))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::OrphanRow);
}

#[test]
fn test_context_preserves_error_kind_collision() {
    let err = Err::<(), _>(HedlError::collision("test", 1))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Collision);
}

#[test]
fn test_context_preserves_error_kind_reference() {
    let err = Err::<(), _>(HedlError::reference("test", 1))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Reference);
}

#[test]
fn test_context_preserves_error_kind_security() {
    let err = Err::<(), _>(HedlError::security("test", 1))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Security);
}

#[test]
fn test_context_preserves_error_kind_conversion() {
    let err = Err::<(), _>(HedlError::conversion("test"))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::Conversion);
}

#[test]
fn test_context_preserves_error_kind_io() {
    let err = Err::<(), _>(HedlError::io("test"))
        .context("ctx")
        .unwrap_err();
    assert_eq!(err.kind, HedlErrorKind::IO);
}

// ==================== Performance Characteristics ====================

#[test]
fn test_context_zero_cost_on_success() {
    // Verify that adding context to successful results is essentially free
    let result: Result<i32, HedlError> = Ok(42);

    let value = result
        .context("context 1")
        .context("context 2")
        .context("context 3")
        .context("context 4")
        .context("context 5")
        .context("context 6")
        .context("context 7")
        .context("context 8")
        .unwrap();

    assert_eq!(value, 42);
}

#[test]
fn test_with_context_single_evaluation() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    static EVAL_COUNT: AtomicUsize = AtomicUsize::new(0);

    let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
    let _ = result
        .with_context(|| {
            EVAL_COUNT.fetch_add(1, Ordering::SeqCst);
            "context"
        })
        .unwrap_err();

    assert_eq!(
        EVAL_COUNT.load(Ordering::SeqCst),
        1,
        "Context should be evaluated exactly once"
    );
}

#[test]
fn test_deep_context_chain() {
    // Test that deeply nested context chains work correctly
    let mut result: Result<(), HedlError> = Err(HedlError::syntax("root error", 1));

    for i in 0..100 {
        result = result.context(format!("layer {}", i));
    }

    let err = result.unwrap_err();
    let ctx = err.context.unwrap();

    // All layers should be present
    assert!(ctx.contains("layer 0"));
    assert!(ctx.contains("layer 50"));
    assert!(ctx.contains("layer 99"));
}
