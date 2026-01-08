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

//! Demonstration of timeout protection for untrusted input

use hedl_stream::{StreamError, StreamingParser, StreamingParserConfig};
use std::io::Cursor;
use std::time::Duration;

fn main() {
    println!("HEDL Stream Timeout Protection Demo\n");

    // Example 1: Normal parsing without timeout
    println!("Example 1: Normal parsing (no timeout)");
    let normal_input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice Smith
  | bob, Bob Jones
"#;

    let parser = StreamingParser::new(Cursor::new(normal_input)).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    println!("  ✓ Parsed {} events successfully\n", events.len());

    // Example 2: Parsing with generous timeout (succeeds)
    println!("Example 2: Parsing with generous timeout (10s)");
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_secs(10)),
        ..Default::default()
    };

    let parser = StreamingParser::with_config(Cursor::new(normal_input), config).unwrap();
    let events: Vec<_> = parser.collect::<Result<Vec<_>, _>>().unwrap();
    println!("  ✓ Parsed {} events within timeout\n", events.len());

    // Example 3: Large input with timeout
    println!("Example 3: Large input with short timeout (100ms)");
    let mut large_input = String::from(
        r#"%VERSION: 1.0
%STRUCT: Data: [id, value]
---
data: @Data
"#,
    );

    // Generate 50,000 rows
    for i in 0..50_000 {
        large_input.push_str(&format!("  | row{}, value{}\n", i, i));
    }

    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(100)),
        ..Default::default()
    };

    match StreamingParser::with_config(Cursor::new(large_input), config) {
        Ok(parser) => {
            let mut count = 0;
            let mut timed_out = false;

            for result in parser {
                match result {
                    Ok(_) => count += 1,
                    Err(StreamError::Timeout { elapsed, limit }) => {
                        println!("  ⚠ Timeout detected!");
                        println!("    Elapsed: {:?}", elapsed);
                        println!("    Limit: {:?}", limit);
                        println!("    Events processed before timeout: {}", count);
                        timed_out = true;
                        break;
                    }
                    Err(e) => {
                        eprintln!("  ✗ Error: {}", e);
                        break;
                    }
                }
            }

            if !timed_out {
                println!("  ✓ Completed all {} events within timeout", count);
            }
        }
        Err(e) => {
            eprintln!("  ✗ Failed to create parser: {}", e);
        }
    }

    println!("\n");

    // Example 4: Malicious input with many schemas
    println!("Example 4: Malicious input with excessive schemas (50ms timeout)");
    let mut malicious_input = String::from("%VERSION: 1.0\n");

    // Add 10,000 type definitions (potential DoS)
    for i in 0..10_000 {
        malicious_input.push_str(&format!("%STRUCT: Type{}: [id, name, value]\n", i));
    }
    malicious_input.push_str("---\n");

    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(50)),
        ..Default::default()
    };

    match StreamingParser::with_config(Cursor::new(malicious_input), config) {
        Ok(_parser) => {
            println!("  ✓ Parser created (might timeout during iteration)");
        }
        Err(StreamError::Timeout { elapsed, limit }) => {
            println!("  ⚠ Timeout during parser creation!");
            println!("    Elapsed: {:?}", elapsed);
            println!("    Limit: {:?}", limit);
            println!("    ✓ Successfully protected against malicious header");
        }
        Err(e) => {
            eprintln!("  ✗ Error: {}", e);
        }
    }

    println!("\n=== Summary ===");
    println!("Timeout protection provides:");
    println!("  • Defense against infinite loops");
    println!("  • Protection from malicious input");
    println!("  • Graceful handling of large datasets");
    println!("  • Clear error reporting with elapsed time");
}
