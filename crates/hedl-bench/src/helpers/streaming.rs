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

//! Streaming parser helpers.
//!
//! Utilities for working with HEDL's streaming parser, including
//! convenient wrappers and event counting.

use crate::Result;
use std::io::{Cursor, Read};

/// Creates a streaming parser from HEDL content.
///
/// # Arguments
///
/// * `hedl` - The HEDL string to stream
///
/// # Returns
///
/// Streaming parser iterator.
#[inline]
pub fn create_stream_parser(
    hedl: &str,
) -> hedl_stream::StreamingParser<Cursor<&[u8]>> {
    let cursor = Cursor::new(hedl.as_bytes());
    hedl_stream::StreamingParser::new(cursor)
        .expect("Streaming parser creation should not fail")
}

/// Creates a streaming parser with custom configuration.
///
/// # Arguments
///
/// * `hedl` - The HEDL string to stream
/// * `config` - Parser configuration
///
/// # Returns
///
/// Streaming parser iterator.
#[inline]
pub fn create_stream_parser_with_config(
    hedl: &str,
    config: hedl_stream::StreamingParserConfig,
) -> hedl_stream::StreamingParser<Cursor<&[u8]>> {
    let cursor = Cursor::new(hedl.as_bytes());
    hedl_stream::StreamingParser::with_config(cursor, config)
        .expect("Streaming parser creation should not fail")
}

/// Streams HEDL from any reader.
///
/// # Arguments
///
/// * `reader` - Any type implementing Read
///
/// # Returns
///
/// Result containing streaming parser.
pub fn stream_from_reader<R: Read>(
    reader: R,
) -> Result<hedl_stream::StreamingParser<R>> {
    hedl_stream::StreamingParser::new(reader)
        .map_err(|e| crate::BenchError::StreamError(e.to_string()))
}

/// Counts node events from a streaming parser.
///
/// # Arguments
///
/// * `hedl` - The HEDL string to stream
///
/// # Returns
///
/// Count of node events.
pub fn count_node_events(hedl: &str) -> usize {
    let parser = create_stream_parser(hedl);
    parser
        .filter_map(Result::ok)
        .filter(|e| matches!(e, hedl_stream::NodeEvent::Node(_)))
        .count()
}

/// Counts all events from a streaming parser.
///
/// # Arguments
///
/// * `hedl` - The HEDL string to stream
///
/// # Returns
///
/// Count of all events.
pub fn count_all_events(hedl: &str) -> usize {
    let parser = create_stream_parser(hedl);
    parser.filter_map(Result::ok).count()
}

/// Collects all node events into a vector.
///
/// # Arguments
///
/// * `hedl` - The HEDL string to stream
///
/// # Returns
///
/// Vector of node events.
pub fn collect_events(hedl: &str) -> Vec<hedl_stream::NodeEvent> {
    let parser = create_stream_parser(hedl);
    parser.filter_map(Result::ok).collect()
}

/// Validates streaming parse completes without errors.
///
/// # Arguments
///
/// * `hedl` - The HEDL string to validate
///
/// # Returns
///
/// Result indicating success or first error encountered.
pub fn validate_stream_parse(hedl: &str) -> Result<()> {
    let parser = create_stream_parser(hedl);
    for result in parser {
        result.map_err(|e| crate::BenchError::StreamError(e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate_users;

    #[test]
    fn test_create_stream_parser() {
        let hedl = generate_users(10);
        let parser = create_stream_parser(&hedl);
        let count = parser.filter_map(Result::ok).count();
        assert!(count > 0);
    }

    #[test]
    fn test_count_node_events() {
        let hedl = generate_users(10);
        let count = count_node_events(&hedl);
        assert!(count > 0);
    }

    #[test]
    fn test_count_all_events() {
        let hedl = generate_users(10);
        let all_count = count_all_events(&hedl);
        let node_count = count_node_events(&hedl);
        assert!(all_count >= node_count);
    }

    #[test]
    fn test_collect_events() {
        let hedl = generate_users(5);
        let events = collect_events(&hedl);
        assert!(!events.is_empty());
    }

    #[test]
    fn test_validate_stream_parse() {
        let hedl = generate_users(10);
        assert!(validate_stream_parse(&hedl).is_ok());
    }
}
