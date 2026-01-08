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

//! Error context helpers for improved ergonomics.
//!
//! This module provides extension traits for `Result<T, HedlError>` that make it easy
//! to add contextual information to errors as they propagate through the call stack.
//!
//! # Examples
//!
//! ## Basic Context
//!
//! ```rust
//! use hedl::{parse, HedlResultExt};
//!
//! fn load_config(path: &str) -> Result<hedl::Document, hedl::HedlError> {
//!     let content = std::fs::read_to_string(path)
//!         .map_err(|e| hedl::HedlError::io(format!("Failed to read {}: {}", path, e)))?;
//!
//!     parse(&content)
//!         .context(format!("while parsing config file {}", path))
//! }
//! ```
//!
//! ## Lazy Context with Closures
//!
//! Use `with_context` when the context message is expensive to compute:
//!
//! ```rust
//! use hedl::{parse, HedlResultExt};
//!
//! fn process_document(id: u64, content: &str) -> Result<(), hedl::HedlError> {
//!     let doc = parse(content)
//!         .with_context(|| format!("processing document {} with {} bytes", id, content.len()))?;
//!
//!     // Process the document...
//!     Ok(())
//! }
//! ```
//!
//! ## Converting Foreign Errors
//!
//! ```rust
//! use hedl::{parse, HedlResultExt, HedlError};
//! use std::io;
//!
//! fn read_and_parse(path: &str) -> Result<hedl::Document, HedlError> {
//!     let content = std::fs::read_to_string(path)
//!         .map_err_to_hedl(|e| HedlError::io(format!("Failed to read {}: {}", path, e)))?;
//!
//!     parse(&content)
//! }
//! ```
//!
//! ## Chaining Context
//!
//! Context can be chained through multiple layers:
//!
//! ```rust
//! use hedl::{parse, HedlResultExt};
//!
//! fn validate_user_data(user_id: &str, data: &str) -> Result<(), hedl::HedlError> {
//!     let doc = parse(data)
//!         .context("failed to parse user data")?;
//!
//!     // Validate the document (example validation)
//!     if doc.root.is_empty() {
//!         return Err(hedl::HedlError::semantic("empty document", 0))
//!             .context(format!("validation failed for user {}", user_id));
//!     }
//!
//!     Ok(())
//! }
//! ```

use crate::HedlError;
use std::fmt;

/// Extension trait for adding context to `Result<T, HedlError>`.
///
/// This trait provides ergonomic methods for annotating errors with additional
/// context as they propagate up the call stack. Context is added to the error's
/// `context` field without modifying the original error message.
///
/// # Design Philosophy
///
/// - **Composable**: Context methods can be chained naturally
/// - **Zero-cost abstractions**: Lazy evaluation with closures
/// - **Type-safe**: Preserves error type information
/// - **Ergonomic**: Follows Rust's `?` operator conventions
///
/// # Performance
///
/// - `context()`: Immediate evaluation, suitable for simple strings
/// - `with_context()`: Lazy evaluation, only computes context on error path
/// - `map_err_to_hedl()`: Zero-cost conversion for foreign error types
pub trait HedlResultExt<T> {
    /// Add context to an error.
    ///
    /// This method immediately evaluates the context message and adds it to the
    /// error if one occurs. For expensive context computations, prefer [`with_context`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl::{parse, HedlResultExt};
    ///
    /// # fn example() -> Result<(), hedl::HedlError> {
    /// let doc = parse("invalid hedl")
    ///     .context("failed to parse configuration")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Context is appended to any existing context:
    ///
    /// ```rust
    /// use hedl::{parse, HedlResultExt};
    ///
    /// let result = parse("invalid")
    ///     .context("in section A")
    ///     .context("while loading config");
    ///
    /// if let Err(e) = result {
    ///     // Context will contain both messages
    ///     assert!(e.context.unwrap().contains("in section A"));
    /// }
    /// ```
    ///
    /// [`with_context`]: HedlResultExt::with_context
    fn context<C>(self, context: C) -> Result<T, HedlError>
    where
        C: fmt::Display;

    /// Add context to an error using a closure.
    ///
    /// This method lazily evaluates the context message only when an error occurs.
    /// This is more efficient than [`context`] when the context string is expensive
    /// to construct (e.g., involves formatting, allocation, or computation).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl::{parse, HedlResultExt};
    ///
    /// # fn example() -> Result<(), hedl::HedlError> {
    /// fn expensive_debug_info() -> String {
    ///     // This only runs if there's an error
    ///     format!("expensive computation result: {}", 42)
    /// }
    ///
    /// let doc = parse("invalid")
    ///     .with_context(|| expensive_debug_info())?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Capturing context from the environment:
    ///
    /// ```rust
    /// use hedl::{parse, HedlResultExt};
    ///
    /// fn process_file(path: &str, content: &str) -> Result<(), hedl::HedlError> {
    ///     parse(content)
    ///         .with_context(|| format!("in file {} ({} bytes)", path, content.len()))?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [`context`]: HedlResultExt::context
    fn with_context<C, F>(self, f: F) -> Result<T, HedlError>
    where
        C: fmt::Display,
        F: FnOnce() -> C;

    /// Convert a foreign error type to `HedlError`.
    ///
    /// This method allows converting errors from other crates (like `std::io::Error`,
    /// `serde_json::Error`, etc.) into `HedlError` while preserving context.
    ///
    /// # Type Signature
    ///
    /// The closure receives the original error and must return a `HedlError`.
    /// This allows you to inspect the original error before converting it.
    ///
    /// # Examples
    ///
    /// Converting I/O errors:
    ///
    /// ```rust
    /// use hedl::{HedlError, HedlResultExt};
    /// use std::fs;
    ///
    /// fn read_config(path: &str) -> Result<String, HedlError> {
    ///     fs::read_to_string(path)
    ///         .map_err_to_hedl(|e| HedlError::io(format!("Failed to read {}: {}", path, e)))
    /// }
    /// ```
    ///
    /// Converting JSON errors:
    ///
    /// ```text
    /// use hedl::{HedlError, HedlResultExt};
    ///
    /// fn parse_json_config(json: &str) -> Result<serde_json::Value, HedlError> {
    ///     serde_json::from_str(json)
    ///         .map_err_to_hedl(|e| HedlError::conversion(format!("Invalid JSON: {}", e)))
    /// }
    /// ```
    fn map_err_to_hedl<F>(self, f: F) -> Result<T, HedlError>
    where
        F: FnOnce(Self::ErrorType) -> HedlError,
        Self: Sized;

    /// The error type for this Result
    type ErrorType;
}


// Specialized implementation for concrete error types
impl<T> HedlResultExt<T> for Result<T, HedlError> {
    type ErrorType = HedlError;

    fn context<C>(self, context: C) -> Result<T, HedlError>
    where
        C: fmt::Display,
    {
        self.map_err(|e| add_context_to_error(e, context.to_string()))
    }

    fn with_context<C, F>(self, f: F) -> Result<T, HedlError>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        self.map_err(|e| add_context_to_error(e, f().to_string()))
    }

    fn map_err_to_hedl<F>(self, _f: F) -> Result<T, HedlError>
    where
        F: FnOnce(Self::ErrorType) -> HedlError,
    {
        // This method doesn't make sense for Result<T, HedlError>
        // since the error is already HedlError
        // We simply return self without any conversion
        self
    }
}

// Specialized implementations for common error types
impl<T> HedlResultExt<T> for Result<T, std::io::Error> {
    type ErrorType = std::io::Error;

    fn context<C>(self, context: C) -> Result<T, HedlError>
    where
        C: fmt::Display,
    {
        self.map_err(|e| {
            let mut err = HedlError::io(e.to_string());
            err.context = Some(context.to_string());
            err
        })
    }

    fn with_context<C, F>(self, f: F) -> Result<T, HedlError>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let mut err = HedlError::io(e.to_string());
            err.context = Some(f().to_string());
            err
        })
    }

    fn map_err_to_hedl<F>(self, f: F) -> Result<T, HedlError>
    where
        F: FnOnce(Self::ErrorType) -> HedlError,
    {
        self.map_err(f)
    }
}

#[cfg(feature = "serde")]
impl<T> HedlResultExt<T> for Result<T, serde_json::Error> {
    type ErrorType = serde_json::Error;

    fn context<C>(self, context: C) -> Result<T, HedlError>
    where
        C: fmt::Display,
    {
        self.map_err(|e| {
            let mut err = HedlError::conversion(e.to_string());
            err.context = Some(context.to_string());
            err
        })
    }

    fn with_context<C, F>(self, f: F) -> Result<T, HedlError>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let mut err = HedlError::conversion(e.to_string());
            err.context = Some(f().to_string());
            err
        })
    }

    fn map_err_to_hedl<F>(self, f: F) -> Result<T, HedlError>
    where
        F: FnOnce(Self::ErrorType) -> HedlError,
    {
        self.map_err(f)
    }
}

/// Helper function to add context to an existing HedlError.
///
/// If the error already has context, the new context is prepended with a separator.
/// This allows building up a context chain through multiple layers of the call stack.
fn add_context_to_error(mut error: HedlError, new_context: String) -> HedlError {
    if new_context.is_empty() {
        return error;
    }

    error.context = Some(match error.context {
        Some(existing) => {
            // Prepend new context with existing context
            // Format: "new context; existing context"
            format!("{}; {}", new_context, existing)
        }
        None => new_context,
    });

    error
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse, HedlErrorKind};

    // ==================== context() tests ====================

    #[test]
    fn test_context_on_error() {
        let result: Result<(), HedlError> = Err(HedlError::syntax("bad token", 5));
        let err = result.context("in function foo").unwrap_err();

        assert_eq!(err.context, Some("in function foo".to_string()));
        assert_eq!(err.line, 5);
        assert_eq!(err.kind, HedlErrorKind::Syntax);
    }

    #[test]
    fn test_context_on_ok() {
        let result: Result<i32, HedlError> = Ok(42);
        let value = result.context("this should not be evaluated").unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_context_chaining() {
        let result: Result<(), HedlError> = Err(HedlError::reference("unresolved @User:1", 10));
        let err = result
            .context("in section users")
            .context("while validating document")
            .unwrap_err();

        let ctx = err.context.unwrap();
        assert!(ctx.contains("while validating document"));
        assert!(ctx.contains("in section users"));
    }

    #[test]
    fn test_context_with_format() {
        let user_id = "alice";
        let result: Result<(), HedlError> = Err(HedlError::collision("duplicate ID", 7));
        let err = result
            .context(format!("for user {}", user_id))
            .unwrap_err();

        assert_eq!(err.context, Some("for user alice".to_string()));
    }

    #[test]
    fn test_context_preserves_error_fields() {
        let original = HedlError::schema("type mismatch", 15).with_column(20);
        let result: Result<(), HedlError> = Err(original);
        let err = result.context("additional info").unwrap_err();

        assert_eq!(err.line, 15);
        assert_eq!(err.column, Some(20));
        assert_eq!(err.kind, HedlErrorKind::Schema);
        assert_eq!(err.message, "type mismatch");
    }

    #[test]
    fn test_context_empty_string() {
        let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
        let err = result.context("").unwrap_err();

        // Empty context should not be added
        assert_eq!(err.context, None);
    }

    // ==================== with_context() tests ====================

    #[test]
    fn test_with_context_lazy_evaluation() {
        let mut evaluated = false;
        let result: Result<i32, HedlError> = Ok(42);

        let value = result
            .with_context(|| {
                evaluated = true;
                "expensive computation"
            })
            .unwrap();

        assert_eq!(value, 42);
        assert!(!evaluated, "Context should not be evaluated on Ok");
    }

    #[test]
    fn test_with_context_on_error() {
        let mut evaluated = false;
        let result: Result<(), HedlError> = Err(HedlError::alias("duplicate alias", 3));

        let err = result
            .with_context(|| {
                evaluated = true;
                "this should be evaluated"
            })
            .unwrap_err();

        assert!(evaluated, "Context should be evaluated on Err");
        assert_eq!(err.context, Some("this should be evaluated".to_string()));
    }

    #[test]
    fn test_with_context_closure_captures() {
        let filename = "config.hedl";
        let line_count = 42;
        let result: Result<(), HedlError> = Err(HedlError::semantic("invalid value", 20));

        let err = result
            .with_context(|| format!("in file {} at line {}/{}", filename, 20, line_count))
            .unwrap_err();

        let ctx = err.context.as_ref().unwrap();
        assert!(ctx.contains("config.hedl"));
        assert!(ctx.contains("42"));
    }

    #[test]
    fn test_with_context_chaining() {
        let result: Result<(), HedlError> = Err(HedlError::shape("wrong column count", 8));

        let err = result
            .with_context(|| "in matrix list")
            .with_context(|| "while parsing data section")
            .unwrap_err();

        let ctx = err.context.unwrap();
        assert!(ctx.contains("while parsing data section"));
        assert!(ctx.contains("in matrix list"));
    }

    // ==================== map_err_to_hedl() tests ====================

    #[test]
    fn test_map_err_to_hedl_io_error() {
        let io_result: Result<String, std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));

        let hedl_result = io_result.map_err_to_hedl(|e: std::io::Error| {
            HedlError::io(format!("Failed to read config: {}", e))
        });

        let err = hedl_result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::IO);
        assert!(err.message.contains("Failed to read config"));
        assert!(err.message.contains("file not found"));
    }

    #[test]
    fn test_map_err_to_hedl_preserves_ok() {
        let io_result: Result<String, std::io::Error> = Ok("content".to_string());
        let hedl_result = io_result.map_err_to_hedl(|e: std::io::Error| HedlError::io(e.to_string()));

        assert_eq!(hedl_result.unwrap(), "content");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_map_err_to_hedl_json_error() {
        let json_result: Result<serde_json::Value, serde_json::Error> =
            serde_json::from_str("invalid json");

        let hedl_result = json_result.map_err_to_hedl(|e: serde_json::Error| {
            HedlError::conversion(format!("JSON parse error: {}", e))
        });

        let err = hedl_result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::Conversion);
        assert!(err.message.contains("JSON parse error"));
    }

    // ==================== Integration tests ====================

    #[test]
    fn test_real_world_parse_with_context() {
        let invalid_hedl = "this is not valid HEDL";
        let err = parse(invalid_hedl)
            .context("failed to parse user configuration")
            .unwrap_err();

        assert!(err.context.is_some());
        assert!(err.context.unwrap().contains("user configuration"));
    }

    #[test]
    fn test_nested_function_context() {
        fn inner() -> Result<(), HedlError> {
            Err(HedlError::reference("undefined @User:1", 5))
        }

        fn middle() -> Result<(), HedlError> {
            inner().context("in middle layer")
        }

        fn outer() -> Result<(), HedlError> {
            middle().context("in outer layer")
        }

        let err = outer().unwrap_err();
        let ctx = err.context.unwrap();

        // Context should contain both layers
        assert!(ctx.contains("outer layer"));
        assert!(ctx.contains("middle layer"));
    }

    #[test]
    fn test_io_context() {
        use std::fs;

        let result = fs::read_to_string("/this/path/does/not/exist")
            .context("failed to load configuration file");

        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::IO);
        assert!(err.context.is_some());
    }

    #[test]
    fn test_io_with_context() {
        use std::fs;

        let path = "/nonexistent/path";
        let result = fs::read_to_string(path)
            .with_context(|| format!("while reading {}", path));

        let err = result.unwrap_err();
        assert_eq!(err.kind, HedlErrorKind::IO);
        assert!(err.context.unwrap().contains("nonexistent"));
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_unicode_in_context() {
        let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
        let err = result.context("日本語エラー 🎉").unwrap_err();

        assert!(err.context.unwrap().contains("🎉"));
    }

    #[test]
    fn test_very_long_context() {
        let long_context = "x".repeat(10000);
        let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
        let err = result.context(long_context.clone()).unwrap_err();

        assert_eq!(err.context, Some(long_context));
    }

    #[test]
    fn test_context_with_newlines() {
        let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
        let err = result
            .context("line 1\nline 2\nline 3")
            .unwrap_err();

        let ctx = err.context.unwrap();
        assert!(ctx.contains("line 1"));
        assert!(ctx.contains("line 2"));
        assert!(ctx.contains("line 3"));
    }

    #[test]
    fn test_multiple_empty_contexts() {
        let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
        let err = result
            .context("")
            .context("")
            .context("real context")
            .unwrap_err();

        assert_eq!(err.context, Some("real context".to_string()));
    }

    // ==================== Performance characteristics ====================

    #[test]
    fn test_context_is_zero_cost_on_ok() {
        // This test verifies that context methods don't allocate or perform
        // work when the result is Ok
        let result: Result<i32, HedlError> = Ok(42);

        // Multiple context calls should be essentially free
        let value = result
            .context("ctx1")
            .context("ctx2")
            .context("ctx3")
            .context("ctx4")
            .context("ctx5")
            .unwrap();

        assert_eq!(value, 42);
    }

    #[test]
    fn test_with_context_only_evaluates_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static EVAL_COUNT: AtomicUsize = AtomicUsize::new(0);

        let result: Result<(), HedlError> = Err(HedlError::syntax("error", 1));
        let _ = result
            .with_context(|| {
                EVAL_COUNT.fetch_add(1, Ordering::SeqCst);
                "context"
            })
            .unwrap_err();

        assert_eq!(EVAL_COUNT.load(Ordering::SeqCst), 1);
    }
}
