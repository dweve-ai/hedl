// Comprehensive tests for LineReader implementation
//
// Tests focus on:
// - Push-back and peek interactions
// - Line length limit enforcement across buffer boundaries
// - UTF-8 validation edge cases
// - CRLF handling edge cases
// - Memory efficiency

use hedl_stream::{LineReader, StreamError};
use std::io::Cursor;

// ==================== Push back and peek interaction tests ====================

#[test]
fn test_multiple_peek_then_push_back() {
    let input = "line1\nline2\nline3";
    let mut reader = LineReader::new(Cursor::new(input));

    // Peek multiple times
    let peeked1 = reader.peek_line().unwrap().cloned();
    let peeked2 = reader.peek_line().unwrap().cloned();
    assert_eq!(peeked1, peeked2);

    // Consume
    let line1 = reader.next_line().unwrap().unwrap();
    assert_eq!(line1, (1, "line1".to_string()));

    // Push back a different line
    reader.push_back(99, "pushed".to_string());

    // Peek should return pushed line
    let peeked3 = reader.peek_line().unwrap();
    assert_eq!(peeked3, Some(&(99, "pushed".to_string())));

    // Next should consume pushed line
    let pushed = reader.next_line().unwrap().unwrap();
    assert_eq!(pushed, (99, "pushed".to_string()));

    // Now should get line2
    let line2 = reader.next_line().unwrap().unwrap();
    assert_eq!(line2, (2, "line2".to_string()));
}

#[test]
fn test_push_back_then_peek_then_next() {
    let input = "line1\nline2";
    let mut reader = LineReader::new(Cursor::new(input));

    reader.push_back(5, "pushed".to_string());

    // Peek should return pushed
    assert_eq!(
        reader.peek_line().unwrap(),
        Some(&(5, "pushed".to_string()))
    );

    // Next should return pushed
    assert_eq!(reader.next_line().unwrap(), Some((5, "pushed".to_string())));

    // Now original content
    assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
}

#[test]
fn test_consecutive_push_backs() {
    let input = "line1";
    let mut reader = LineReader::new(Cursor::new(input));

    reader.push_back(1, "first".to_string());
    reader.push_back(2, "second".to_string()); // Overwrites first

    let line = reader.next_line().unwrap().unwrap();
    assert_eq!(line, (2, "second".to_string()));
}

// ==================== Line length edge cases ====================

#[test]
fn test_line_exactly_at_max_length_with_crlf() {
    let max_len = 100;
    // Create line of exactly max_len - 2 (for \r\n)
    let line = format!("{}\r\n", "A".repeat(max_len - 2));
    let mut reader = LineReader::with_max_length(Cursor::new(line), max_len);

    let result = reader.next_line();
    assert!(result.is_ok());
    let (_, content) = result.unwrap().unwrap();
    assert_eq!(content.len(), max_len - 2);
}

#[test]
fn test_line_one_over_max_length_with_crlf() {
    let max_len = 100;
    // Create line that exceeds max_len
    let line = format!("{}\r\n", "A".repeat(max_len + 1));
    let mut reader = LineReader::with_max_length(Cursor::new(line), max_len);

    let result = reader.next_line();
    assert!(result.is_err());
    assert!(matches!(result, Err(StreamError::LineTooLong { .. })));
}

#[test]
fn test_multiple_lines_mixed_short_and_long() {
    let max_len = 50;
    let input = format!(
        "short1\n{}\nshort2\n{}\nshort3\n",
        "A".repeat(60), // Too long
        "B".repeat(70)  // Also too long
    );
    let mut reader = LineReader::with_max_length(Cursor::new(input), max_len);

    // First short line OK
    assert!(reader.next_line().is_ok());

    // Second line too long
    let result = reader.next_line();
    assert!(result.is_err());
    if let Err(StreamError::LineTooLong { line, .. }) = result {
        assert_eq!(line, 2);
    }
}

#[test]
fn test_line_length_with_small_buffer() {
    let max_len = 200;
    let line = format!("{}\n", "A".repeat(250));
    let mut reader = LineReader::with_capacity_and_max_length(Cursor::new(line), 16, max_len);

    let result = reader.next_line();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        StreamError::LineTooLong { .. }
    ));
}

// ==================== UTF-8 validation edge cases ====================

#[test]
fn test_partial_utf8_at_buffer_boundary() {
    // Create input where a multi-byte UTF-8 character might be split across buffer boundaries
    // Using a 3-byte UTF-8 character (中)
    let chinese = "中".as_bytes();
    assert_eq!(chinese.len(), 3);

    // Create input with valid UTF-8
    let mut input = Vec::new();
    input.extend_from_slice(b"start");
    input.extend_from_slice(chinese);
    input.extend_from_slice(b"end\n");

    let mut reader = LineReader::with_capacity(Cursor::new(input), 8);

    let result = reader.next_line();
    assert!(result.is_ok());
    let (_, line) = result.unwrap().unwrap();
    assert_eq!(line, "start中end");
}

#[test]
fn test_multiple_multibyte_characters() {
    let input = "日本語テスト\n한국어시험\n测试中文\n";
    let mut reader = LineReader::new(Cursor::new(input));

    assert_eq!(
        reader.next_line().unwrap(),
        Some((1, "日本語テスト".to_string()))
    );
    assert_eq!(
        reader.next_line().unwrap(),
        Some((2, "한국어시험".to_string()))
    );
    assert_eq!(
        reader.next_line().unwrap(),
        Some((3, "测试中文".to_string()))
    );
}

#[test]
fn test_invalid_utf8_at_line_start() {
    let input = vec![0xFF, 0xFE, b'a', b'b', b'c', b'\n'];
    let mut reader = LineReader::new(Cursor::new(input));

    let result = reader.next_line();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        StreamError::InvalidUtf8 { .. }
    ));
}

#[test]
fn test_invalid_utf8_at_line_end() {
    let mut input = Vec::new();
    input.extend_from_slice(b"valid");
    input.extend_from_slice(&[0xFF, 0xFE]);
    input.push(b'\n');

    let mut reader = LineReader::new(Cursor::new(input));

    let result = reader.next_line();
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        StreamError::InvalidUtf8 { line: 1, .. }
    ));
}

// ==================== CRLF handling edge cases ====================

#[test]
fn test_standalone_cr_without_lf() {
    let input = "line1\rline2\n";
    let mut reader = LineReader::new(Cursor::new(input));

    // \r without \n should be preserved in the line
    let (_, line) = reader.next_line().unwrap().unwrap();
    assert!(line.contains('\r'));
}

#[test]
fn test_multiple_cr_before_lf() {
    let input = "line1\r\r\n";
    let mut reader = LineReader::new(Cursor::new(input));

    let (_, line) = reader.next_line().unwrap().unwrap();
    // Only the last \r before \n should be stripped
    assert_eq!(line, "line1\r");
}

#[test]
fn test_lf_followed_by_cr() {
    let input = "line1\n\rline2\n";
    let mut reader = LineReader::new(Cursor::new(input));

    let line1 = reader.next_line().unwrap().unwrap();
    assert_eq!(line1, (1, "line1".to_string()));

    let line2 = reader.next_line().unwrap().unwrap();
    assert_eq!(line2, (2, "\rline2".to_string()));
}

#[test]
fn test_only_crlf_lines() {
    let input = "\r\n\r\n\r\n";
    let mut reader = LineReader::new(Cursor::new(input));

    assert_eq!(reader.next_line().unwrap(), Some((1, String::new())));
    assert_eq!(reader.next_line().unwrap(), Some((2, String::new())));
    assert_eq!(reader.next_line().unwrap(), Some((3, String::new())));
    assert_eq!(reader.next_line().unwrap(), None);
}

// ==================== Edge case: Empty and whitespace ====================

#[test]
fn test_line_with_only_spaces() {
    let input = "     \n";
    let mut reader = LineReader::new(Cursor::new(input));

    let (_, line) = reader.next_line().unwrap().unwrap();
    assert_eq!(line, "     ");
}

#[test]
fn test_line_with_only_tabs() {
    let input = "\t\t\t\n";
    let mut reader = LineReader::new(Cursor::new(input));

    let (_, line) = reader.next_line().unwrap().unwrap();
    assert_eq!(line, "\t\t\t");
}

#[test]
fn test_alternating_empty_and_content() {
    let input = "\ncontent\n\ncontent2\n\n";
    let mut reader = LineReader::new(Cursor::new(input));

    assert_eq!(reader.next_line().unwrap(), Some((1, String::new())));
    assert_eq!(
        reader.next_line().unwrap(),
        Some((2, "content".to_string()))
    );
    assert_eq!(reader.next_line().unwrap(), Some((3, String::new())));
    assert_eq!(
        reader.next_line().unwrap(),
        Some((4, "content2".to_string()))
    );
    assert_eq!(reader.next_line().unwrap(), Some((5, String::new())));
    assert_eq!(reader.next_line().unwrap(), None);
}

// ==================== Buffer boundary tests ====================

#[test]
fn test_line_exactly_one_buffer_size() {
    let buffer_size = 64;
    let line = format!("{}\n", "A".repeat(buffer_size));
    let mut reader = LineReader::with_capacity(Cursor::new(line), buffer_size);

    let result = reader.next_line();
    assert!(result.is_ok());
    let (_, content) = result.unwrap().unwrap();
    assert_eq!(content.len(), buffer_size);
}

#[test]
fn test_line_crosses_multiple_buffer_boundaries() {
    let buffer_size = 32;
    let line = format!("{}\n", "A".repeat(buffer_size * 3 + 10));
    let mut reader = LineReader::with_capacity(Cursor::new(line), buffer_size);

    let result = reader.next_line();
    assert!(result.is_ok());
    let (_, content) = result.unwrap().unwrap();
    assert_eq!(content.len(), buffer_size * 3 + 10);
}

#[test]
fn test_newline_at_exact_buffer_boundary() {
    let buffer_size = 16;
    let line1 = "A".repeat(buffer_size);
    let input = format!("{line1}\nline2");
    let mut reader = LineReader::with_capacity(Cursor::new(input), buffer_size);

    let (_, l1) = reader.next_line().unwrap().unwrap();
    assert_eq!(l1.len(), buffer_size);

    let (_, l2) = reader.next_line().unwrap().unwrap();
    assert_eq!(l2, "line2");
}

// ==================== Line number tracking ====================

#[test]
fn test_line_number_with_peek() {
    let input = "line1\nline2\nline3";
    let mut reader = LineReader::new(Cursor::new(input));

    assert_eq!(reader.line_number(), 0);

    reader.peek_line().unwrap();
    // Peek reads the line internally, so line number advances
    assert_eq!(reader.line_number(), 1);

    reader.next_line().unwrap();
    assert_eq!(reader.line_number(), 1);

    reader.peek_line().unwrap();
    assert_eq!(reader.line_number(), 2); // Peek reads line2

    reader.next_line().unwrap();
    assert_eq!(reader.line_number(), 2);
}

#[test]
fn test_line_number_with_push_back() {
    let input = "line1\nline2";
    let mut reader = LineReader::new(Cursor::new(input));

    reader.next_line().unwrap();
    assert_eq!(reader.line_number(), 1);

    reader.push_back(99, "pushed".to_string());
    assert_eq!(reader.line_number(), 1); // Push back doesn't change line number

    reader.next_line().unwrap(); // Consume pushed
    assert_eq!(reader.line_number(), 1); // Still at 1 since we consumed pushed line

    reader.next_line().unwrap(); // Now read actual line2
    assert_eq!(reader.line_number(), 2);
}

// ==================== EOF handling ====================

#[test]
fn test_multiple_reads_after_eof() {
    let input = "line1";
    let mut reader = LineReader::new(Cursor::new(input));

    reader.next_line().unwrap();
    assert_eq!(reader.next_line().unwrap(), None);
    assert_eq!(reader.next_line().unwrap(), None);
    assert_eq!(reader.next_line().unwrap(), None);
}

#[test]
fn test_peek_after_eof() {
    let input = "line1";
    let mut reader = LineReader::new(Cursor::new(input));

    reader.next_line().unwrap();
    assert_eq!(reader.peek_line().unwrap(), None);
    assert_eq!(reader.peek_line().unwrap(), None);
}

#[test]
fn test_partial_line_at_eof() {
    let input = "line1\npartial";
    let mut reader = LineReader::new(Cursor::new(input));

    assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
    assert_eq!(
        reader.next_line().unwrap(),
        Some((2, "partial".to_string()))
    );
    assert_eq!(reader.next_line().unwrap(), None);
}

// ==================== Special characters ====================

#[test]
fn test_line_with_null_bytes() {
    let input = b"line1\x00with\x00nulls\n";
    let mut reader = LineReader::new(Cursor::new(input.to_vec()));

    let (_, line) = reader.next_line().unwrap().unwrap();
    assert!(line.contains('\0'));
    assert_eq!(line.len(), 16); // "line1\0with\0nulls"
}

#[test]
fn test_line_with_control_characters() {
    let input = "line\x01\x02\x03\x04\n";
    let mut reader = LineReader::new(Cursor::new(input));

    let (_, line) = reader.next_line().unwrap().unwrap();
    assert_eq!(line.len(), 8); // "line" + 4 control chars
}

// ==================== Performance characteristics ====================

#[test]
fn test_very_small_buffer_still_works() {
    let input = "line1\nline2\nline3";
    let mut reader = LineReader::with_capacity(Cursor::new(input), 1);

    assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
    assert_eq!(reader.next_line().unwrap(), Some((2, "line2".to_string())));
    assert_eq!(reader.next_line().unwrap(), Some((3, "line3".to_string())));
}

#[test]
fn test_huge_buffer_still_works() {
    let input = "line1\nline2";
    let mut reader = LineReader::with_capacity(Cursor::new(input), 1_000_000);

    assert_eq!(reader.next_line().unwrap(), Some((1, "line1".to_string())));
    assert_eq!(reader.next_line().unwrap(), Some((2, "line2".to_string())));
}
