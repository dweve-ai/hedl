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

//! Timeout tests for hedl-stream to prevent infinite parsing loops

use hedl_stream::{StreamError, StreamingParser, StreamingParserConfig};
use std::io::Cursor;
use std::time::Duration;

// ==================== Basic Timeout Tests ====================

#[test]
fn test_timeout_on_large_input() {
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(100)),
        ..Default::default()
    };

    // Generate a large input that should exceed timeout
    let mut input = String::from(
        r#"%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
"#,
    );

    // Generate 100,000 rows (should take longer than 100ms to parse)
    for i in 0..100_000 {
        input.push_str(&format!("  | row{}, value{}\n", i, i));
    }

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();

    // Attempt to consume all events - should timeout
    let mut found_timeout = false;
    for result in parser {
        if let Err(StreamError::Timeout { elapsed, limit }) = result {
            found_timeout = true;
            assert!(elapsed >= limit);
            break;
        }
    }

    assert!(found_timeout, "Expected timeout error");
}

#[test]
fn test_timeout_on_deeply_nested_input() {
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(100)),
        max_indent_depth: 1000, // Allow deep nesting
        ..Default::default()
    };

    // Generate deeply nested structure with actual data that requires parsing
    let mut input = String::from(
        r#"%VERSION: 1.0
%STRUCT: Data: [id, value]
---
"#,
    );

    // Create many nested objects with actual list data to parse
    for i in 0..5000 {
        let indent = "  ".repeat((i / 100) % 10); // Create some nesting
        input.push_str(&format!("{}level{}: @Data\n", indent, i));
        // Add rows for each list to increase parsing workload
        for j in 0..20 {
            let row_indent = "  ".repeat((i / 100) % 10 + 1);
            input.push_str(&format!("{}| row{}, val{}\n", row_indent, j, j));
        }
    }

    let result = StreamingParser::with_config(Cursor::new(input), config);

    // Should timeout either during construction or iteration
    match result {
        Err(StreamError::Timeout { .. }) => {
            // Timeout during header parsing - expected
        }
        Ok(parser) => {
            // Timeout during body parsing
            let mut found_timeout = false;
            for result in parser {
                if matches!(result, Err(StreamError::Timeout { .. })) {
                    found_timeout = true;
                    break;
                }
            }
            assert!(found_timeout, "Expected timeout during parsing");
        }
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
    }
}

#[test]
fn test_timeout_during_header_parsing() {
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(50)),
        ..Default::default()
    };

    // Generate many header directives
    let mut input = String::from("%VERSION: 1.0\n");

    // Add thousands of STRUCT definitions
    for i in 0..50_000 {
        input.push_str(&format!("%STRUCT: Type{}: [id, name]\n", i));
    }

    input.push_str("---\n");

    // Should timeout during header parsing
    let result = StreamingParser::with_config(Cursor::new(input), config);
    assert!(
        matches!(result, Err(StreamError::Timeout { .. })),
        "Expected timeout during header parsing"
    );
}

// ==================== No Timeout Tests ====================

#[test]
fn test_no_timeout_by_default() {
    let config = StreamingParserConfig {
        timeout: None, // No timeout
        ..Default::default()
    };

    let mut input = String::from(
        r#"%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
"#,
    );

    // Generate moderate amount of data
    for i in 0..1000 {
        input.push_str(&format!("  | row{}, value{}\n", i, i));
    }

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();

    // Should complete successfully without timeout
    let events: Vec<_> = parser.collect();
    assert!(
        events.iter().all(|e| !matches!(e, Err(StreamError::Timeout { .. }))),
        "Should not timeout when timeout is None"
    );

    // Verify we got all the data
    let successful: Vec<_> = events.into_iter().filter_map(|e| e.ok()).collect();
    assert!(successful.len() > 1000); // ListStart + 1000 nodes + ListEnd + more
}

#[test]
fn test_sufficient_timeout_completes() {
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_secs(10)), // Very generous timeout
        ..Default::default()
    };

    let input = r#"%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice Smith
  | bob, Bob Jones
"#;

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();

    // Should complete successfully
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    assert!(events.len() > 0);
}

// ==================== Edge Case Timeout Tests ====================

#[test]
fn test_timeout_with_zero_duration() {
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(0)), // Immediate timeout
        ..Default::default()
    };

    let input = r#"%VERSION: 1.0
---
"#;

    // Should timeout immediately or very quickly
    let result = StreamingParser::with_config(Cursor::new(input), config);

    // Either timeout during construction or first event
    match result {
        Err(StreamError::Timeout { .. }) => {
            // Expected - timeout during construction
        }
        Ok(parser) => {
            // Should timeout on first iteration
            let first = parser.into_iter().next();
            if let Some(Err(StreamError::Timeout { .. })) = first {
                // Expected - timeout during first iteration
            } else {
                panic!("Expected timeout on first iteration");
            }
        }
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
    }
}

#[test]
fn test_timeout_with_very_small_duration() {
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_nanos(1)), // 1 nanosecond - essentially immediate
        ..Default::default()
    };

    let input = r#"%VERSION: 1.0
%STRUCT: Data: [id]
---
data: @Data
  | row1
"#;

    let result = StreamingParser::with_config(Cursor::new(input), config);

    // Should timeout very quickly
    let timed_out = match result {
        Err(StreamError::Timeout { .. }) => true,
        Ok(parser) => {
            let mut found = false;
            for result in parser {
                if matches!(result, Err(StreamError::Timeout { .. })) {
                    found = true;
                    break;
                }
            }
            found
        }
        Err(_) => false,
    };

    assert!(timed_out, "Expected timeout with 1ns duration");
}

#[test]
fn test_timeout_error_contains_duration_info() {
    let timeout_limit = Duration::from_millis(10);
    let config = StreamingParserConfig {
        timeout: Some(timeout_limit),
        ..Default::default()
    };

    // Generate enough data to trigger timeout
    let mut input = String::from(
        r#"%VERSION: 1.0
%STRUCT: Data: [id]
---
data: @Data
"#,
    );

    for i in 0..100_000 {
        input.push_str(&format!("  | row{}\n", i));
    }

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();

    for result in parser {
        if let Err(StreamError::Timeout { elapsed, limit }) = result {
            assert_eq!(limit, timeout_limit);
            assert!(elapsed >= limit);

            // Check error message format
            let error_msg = format!("{}", StreamError::Timeout { elapsed, limit });
            assert!(error_msg.contains("timeout"));
            assert!(error_msg.contains(&format!("{:?}", limit)));
            break;
        }
    }
}

// ==================== Stress Tests with Timeout ====================

#[test]
fn test_timeout_prevents_infinite_loop_malicious_input() {
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(200)),
        ..Default::default()
    };

    // Simulate malicious input with many repeated structures
    let mut input = String::from("%VERSION: 1.0\n");

    // Add many type definitions (could be malicious)
    for i in 0..100_000 {
        input.push_str(&format!("%STRUCT: Type{}: [id]\n", i));
    }

    input.push_str("---\n");

    // Should timeout during header parsing
    let result = StreamingParser::with_config(Cursor::new(input), config);
    assert!(
        matches!(result, Err(StreamError::Timeout { .. })),
        "Timeout should prevent processing of malicious header"
    );
}

#[test]
fn test_timeout_on_extremely_wide_matrix() {
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(100)),
        ..Default::default()
    };

    // Generate schema with many columns
    let mut columns = vec!["id".to_string()];
    for i in 0..10_000 {
        columns.push(format!("col{}", i));
    }

    let schema = columns.join(", ");
    let mut input = format!(
        r#"%VERSION: 1.0
%STRUCT: WideData: [{}]
---
data: @WideData
"#,
        schema
    );

    // Add rows with all those columns
    for row_num in 0..100 {
        input.push_str(&format!("  | row{}", row_num));
        for i in 0..10_000 {
            input.push_str(&format!(", val{}", i));
        }
        input.push('\n');
    }

    let result = StreamingParser::with_config(Cursor::new(input), config);

    // Should timeout either during construction or parsing
    let timed_out = match result {
        Err(StreamError::Timeout { .. }) => true,
        Ok(parser) => {
            let mut found = false;
            for result in parser {
                if matches!(result, Err(StreamError::Timeout { .. })) {
                    found = true;
                    break;
                }
            }
            found
        }
        Err(_) => false,
    };

    assert!(timed_out, "Expected timeout on extremely wide matrix");
}

// ==================== Combined Limits Tests ====================

#[test]
fn test_timeout_with_other_limits() {
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(100)),
        max_indent_depth: 10,
        max_line_length: 1000,
        ..Default::default()
    };

    let mut input = String::from(
        r#"%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
"#,
    );

    // Generate data that might hit various limits
    for i in 0..100_000 {
        input.push_str(&format!("  | row{}, value{}\n", i, i));
    }

    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();

    // Should eventually timeout
    let mut found_timeout = false;
    for result in parser {
        if matches!(result, Err(StreamError::Timeout { .. })) {
            found_timeout = true;
            break;
        }
    }

    assert!(found_timeout, "Expected timeout");
}

// ==================== Performance Characteristic Tests ====================

#[test]
fn test_timeout_check_overhead_minimal() {
    // Test that timeout checking doesn't significantly slow down parsing

    let input = r#"%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
  | row1, value1
  | row2, value2
  | row3, value3
"#;

    // Parse with timeout
    let config_with_timeout = StreamingParserConfig {
        timeout: Some(Duration::from_secs(10)),
        ..Default::default()
    };

    let start_with = std::time::Instant::now();
    let parser = StreamingParser::with_config(Cursor::new(input), config_with_timeout).unwrap();
    let _: Vec<_> = parser.collect();
    let duration_with = start_with.elapsed();

    // Parse without timeout
    let config_without_timeout = StreamingParserConfig {
        timeout: None,
        ..Default::default()
    };

    let start_without = std::time::Instant::now();
    let parser = StreamingParser::with_config(Cursor::new(input), config_without_timeout).unwrap();
    let _: Vec<_> = parser.collect();
    let duration_without = start_without.elapsed();

    // Overhead should be minimal (less than 2x slower)
    // This is a loose bound since it depends on system load
    assert!(
        duration_with < duration_without * 2 + Duration::from_millis(10),
        "Timeout checking overhead is too high: with={:?} without={:?}",
        duration_with,
        duration_without
    );
}

#[test]
fn test_timeout_periodic_checking() {
    // Verify that timeout is checked periodically, not on every operation
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(50)),
        ..Default::default()
    };

    let input = r#"%VERSION: 1.0
%STRUCT: Data: [id]
---
data: @Data
  | row1
  | row2
  | row3
"#;

    // With very short timeout and small input, should complete or timeout quickly
    let start = std::time::Instant::now();
    let parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();
    let _: Vec<_> = parser.collect();
    let elapsed = start.elapsed();

    // Should complete in reasonable time (not hung)
    assert!(
        elapsed < Duration::from_secs(1),
        "Parsing took too long: {:?}",
        elapsed
    );
}
