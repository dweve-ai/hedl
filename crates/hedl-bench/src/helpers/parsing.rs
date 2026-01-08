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

//! Parsing helpers for benchmarks.
//!
//! Provides convenient wrappers for HEDL parsing operations with
//! timing and batch processing capabilities.

use crate::Result;
use hedl_core::Document;
use std::time::{Duration, Instant};

/// Parses HEDL string with timing measurement.
///
/// # Arguments
///
/// * `input` - The HEDL string to parse
///
/// # Returns
///
/// Tuple of (parsed document, duration).
///
/// # Panics
///
/// Panics if parsing fails.
pub fn parse_with_timing(input: &str) -> (Document, Duration) {
    let start = Instant::now();
    let doc = hedl_core::parse(input.as_bytes()).expect("Parse should succeed for benchmark data");
    let duration = start.elapsed();
    (doc, duration)
}

/// Parses multiple HEDL strings in batch.
///
/// # Arguments
///
/// * `inputs` - Slice of HEDL strings to parse
///
/// # Returns
///
/// Vector of Results, one per input.
pub fn parse_batch(inputs: &[&str]) -> Vec<Result<Document>> {
    inputs
        .iter()
        .map(|input| {
            hedl_core::parse(input.as_bytes())
                .map_err(|e| crate::BenchError::ParseError(e.to_string()))
        })
        .collect()
}

/// Parses HEDL string safely, returning Result.
///
/// # Arguments
///
/// * `input` - The HEDL string to parse
///
/// # Returns
///
/// Result containing parsed document or error.
#[inline]
pub fn parse_safe(input: &str) -> Result<Document> {
    hedl_core::parse(input.as_bytes()).map_err(|e| crate::BenchError::ParseError(e.to_string()))
}

/// Parses HEDL string, panicking on error.
///
/// Use for benchmark data that should always be valid.
///
/// # Arguments
///
/// * `input` - The HEDL string to parse
///
/// # Returns
///
/// Parsed document.
#[inline]
pub fn parse_unchecked(input: &str) -> Document {
    hedl_core::parse(input.as_bytes()).expect("HEDL parsing should not fail for benchmark data")
}

/// Parses HEDL from bytes.
///
/// # Arguments
///
/// * `bytes` - The HEDL bytes to parse
///
/// # Returns
///
/// Parsed document.
#[inline]
pub fn parse_bytes(bytes: &[u8]) -> Document {
    hedl_core::parse(bytes).expect("HEDL parsing should not fail for benchmark data")
}

/// Parses HEDL from bytes safely.
///
/// # Arguments
///
/// * `bytes` - The HEDL bytes to parse
///
/// # Returns
///
/// Result containing parsed document or error.
#[inline]
pub fn parse_bytes_safe(bytes: &[u8]) -> Result<Document> {
    hedl_core::parse(bytes).map_err(|e| crate::BenchError::ParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_users;

    #[test]
    fn test_parse_with_timing() {
        let hedl = generate_users(10);
        let (doc, duration) = parse_with_timing(&hedl);
        assert!(!doc.root.is_empty());
        assert!(duration.as_nanos() > 0);
    }

    #[test]
    fn test_parse_batch() {
        let inputs = vec![generate_users(5), generate_users(10)];
        let refs: Vec<&str> = inputs.iter().map(|s| s.as_str()).collect();
        let results = parse_batch(&refs);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_parse_safe() {
        let hedl = generate_users(5);
        assert!(parse_safe(&hedl).is_ok());
    }

    #[test]
    fn test_parse_unchecked() {
        let hedl = generate_users(5);
        let doc = parse_unchecked(&hedl);
        assert!(!doc.root.is_empty());
    }
}
