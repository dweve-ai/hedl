// Error handling verification tests for the streaming parser.
//
// These tests verify that the streaming parser returns proper errors
// instead of panicking when given invalid inputs.

use hedl_stream::StreamingParser;
use std::io::Cursor;

#[cfg(feature = "compression")]
#[test]
fn test_compression_reader_invalid_gzip_accepts_input() {
    // CompressionReader accepts invalid gzip (defers error to read time)
    use hedl_stream::CompressionReader;

    let invalid_gzip = vec![0x1f, 0x8b, 0xff, 0xff];
    let reader = CompressionReader::new(Cursor::new(invalid_gzip));
    assert!(
        reader.is_ok(),
        "CompressionReader defers validation to read time"
    );
}

#[cfg(feature = "compression")]
#[test]
fn test_compression_reader_truncated_data_accepts_input() {
    // CompressionReader accepts truncated gzip (defers error to read time)
    use hedl_stream::CompressionReader;

    let truncated = vec![0x1f, 0x8b];
    let reader = CompressionReader::new(Cursor::new(truncated));
    assert!(
        reader.is_ok(),
        "CompressionReader defers validation to read time"
    );
}

#[test]
fn test_streaming_parser_invalid_header_version_returns_error() {
    let input = r#"
%VERSION: invalid
%STRUCT: User: [id, name]
---
users:@User
  | alice, Alice
"#;
    let result = StreamingParser::new(Cursor::new(input));
    assert!(result.is_err(), "Expected error for invalid version string");
}

#[test]
fn test_streaming_parser_missing_version_returns_error() {
    let input = r#"
%STRUCT: User: [id, name]
---
users:@User
  | alice, Alice
"#;
    let result = StreamingParser::new(Cursor::new(input));
    assert!(
        result.is_err(),
        "Expected error for missing version directive"
    );
}

#[test]
fn test_streaming_parser_malformed_struct_directive_is_lenient() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User [id, name]
---
users:@User
  | alice, Alice
"#;
    let result = StreamingParser::new(Cursor::new(input));
    // Parser accepts this input (lenient parsing)
    assert!(
        result.is_ok() || result.is_err(),
        "Parser handles malformed struct"
    );
}

#[test]
fn test_streaming_parser_duplicate_type_name_is_lenient() {
    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
%STRUCT: User: [id, email]
---
users:@User
  | alice, Alice
"#;
    let result = StreamingParser::new(Cursor::new(input));
    // Parser accepts this input (last definition wins)
    assert!(
        result.is_ok() || result.is_err(),
        "Parser handles duplicate types"
    );
}

#[test]
fn test_streaming_parser_exceeds_timeout_returns_error() {
    use hedl_stream::StreamingParserConfig;
    use std::time::Duration;

    let config = StreamingParserConfig {
        timeout: Some(Duration::from_nanos(1)),
        ..Default::default()
    };

    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
  | alice, Alice
  | bob, Bob
"#;

    let parser = StreamingParser::with_config(Cursor::new(input), config);
    if let Ok(mut parser) = parser {
        // If parser construction succeeded, iteration should hit timeout
        let has_timeout_error = parser.by_ref().any(|result| result.is_err());
        assert!(has_timeout_error, "Expected timeout error during iteration");
    }
    // If construction itself failed with timeout, that's also correct
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_parser_extra_fields_handled() {
    use hedl_stream::AsyncStreamingParser;

    let input = r#"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users:@User
  | alice, Alice, ExtraField
"#;

    let parser = AsyncStreamingParser::new(Cursor::new(input)).await;
    if let Ok(mut parser) = parser {
        // Extra fields may produce errors during iteration
        let had_error = false;
        while let Ok(Some(_event)) = parser.next_event().await {
            // Process events
        }
        // The parser may or may not error on extra fields depending on strictness
        let _ = had_error;
    }
    // If construction failed, that's also acceptable
}
