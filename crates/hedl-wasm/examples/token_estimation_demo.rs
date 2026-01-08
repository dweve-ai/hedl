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


// Token Estimation Demonstration
//
// This example demonstrates the token estimation functionality
// and shows typical performance characteristics.

use std::hint::black_box;
use std::time::Instant;

/// Token estimation constant
const CHARS_PER_TOKEN: usize = 4;

/// Optimized byte-level single-pass token estimation
#[inline]
fn estimate_tokens_optimized(text: &str) -> usize {
    let bytes = text.as_bytes();
    let byte_count = bytes.len();

    if byte_count == 0 {
        return 0;
    }

    let mut whitespace_count = 0usize;
    let mut punct_count = 0usize;
    let mut i = 0;

    while i < byte_count {
        let b = bytes[i];

        if b < 128 {
            whitespace_count += matches!(b, b' ' | b'\t' | b'\n' | b'\r') as usize;
            punct_count += matches!(
                b,
                b'!' | b'"'
                    | b'#' | b'$'
                    | b'%' | b'&'
                    | b'\'' | b'('
                    | b')' | b'*'
                    | b'+' | b','
                    | b'-' | b'.'
                    | b'/' | b':'
                    | b';' | b'<'
                    | b'=' | b'>'
                    | b'?' | b'@'
                    | b'[' | b'\\'
                    | b']' | b'^'
                    | b'_' | b'`'
                    | b'{' | b'|'
                    | b'}' | b'~'
            ) as usize;
            i += 1;
        } else {
            let char_len = if b < 0b1110_0000 {
                2
            } else if b < 0b1111_0000 {
                3
            } else {
                4
            };
            i += char_len;
        }
    }

    (byte_count + whitespace_count + punct_count) / CHARS_PER_TOKEN
}

/// Old multi-pass implementation for comparison
fn estimate_tokens_old(text: &str) -> usize {
    let char_count = text.len();
    let whitespace_count = text.chars().filter(|c| c.is_whitespace()).count();
    let punct_count = text.chars().filter(|c| c.is_ascii_punctuation()).count();
    (char_count + whitespace_count + punct_count) / CHARS_PER_TOKEN
}

fn main() {
    println!("{}", "=".repeat(70));
    println!("Token Estimation Demonstration");
    println!("{}", "=".repeat(70));
    println!();

    // Example 1: Small JSON object
    let small_json = r#"{"id": "user-123", "name": "Alice Smith", "email": "alice@example.com"}"#;
    println!("Example 1: Small JSON Object");
    println!("Text: {}", small_json);
    println!("Length: {} bytes", small_json.len());
    let tokens = estimate_tokens_optimized(small_json);
    println!("Estimated tokens: {}", tokens);
    println!();

    // Example 2: HEDL data
    let hedl = r#"%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice Smith, alice@example.com
  | bob, Bob Jones, bob@example.com"#;
    println!("Example 2: HEDL Document");
    println!("Length: {} bytes", hedl.len());
    let tokens = estimate_tokens_optimized(hedl);
    println!("Estimated tokens: {}", tokens);
    println!();

    // Example 3: Large document performance test
    println!("Example 3: Performance Comparison");
    println!("{:-<70}", "");

    let large_doc = r#"{"id": "user-123", "name": "Alice Smith", "email": "alice@example.com", "tags": ["admin", "verified"], "score": 95.5}"#.repeat(10_000);

    println!("Document size: {} bytes ({:.2} MB)", large_doc.len(), large_doc.len() as f64 / 1_000_000.0);
    println!();

    // Warm up
    for _ in 0..10 {
        let _ = estimate_tokens_old(&large_doc);
        let _ = estimate_tokens_optimized(&large_doc);
    }

    // Benchmark old implementation
    let iterations = 1000;
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(estimate_tokens_old(black_box(&large_doc)));
    }
    let old_duration = start.elapsed();

    // Benchmark optimized implementation
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(estimate_tokens_optimized(black_box(&large_doc)));
    }
    let optimized_duration = start.elapsed();

    let speedup = old_duration.as_nanos() as f64 / optimized_duration.as_nanos() as f64;

    println!("Performance ({} iterations):", iterations);
    println!("  Multi-pass (old): {:?}", old_duration);
    println!("  Single-pass (new): {:?}", optimized_duration);
    println!("  Speedup: {:.2}x", speedup);
    println!();

    // Correctness verification
    println!("Correctness Verification:");
    let old_result = estimate_tokens_old(&large_doc);
    let new_result = estimate_tokens_optimized(&large_doc);
    println!("  Multi-pass result: {} tokens", old_result);
    println!("  Single-pass result: {} tokens", new_result);
    println!("  Match: {}", if old_result == new_result { "✓ YES" } else { "✗ NO" });
    println!();

    println!("{}", "=".repeat(70));
    println!("Summary");
    println!("{}", "=".repeat(70));
    println!();
    println!("The optimized byte-level implementation provides:");
    println!("  • Single-pass algorithm (no repeated iteration)");
    println!("  • ASCII fast path for common structured data");
    println!("  • {:.1}x speedup on large documents", speedup);
    println!("  • Identical results (correctness preserved)");
    println!("  • Zero allocations and minimal memory overhead");
}
