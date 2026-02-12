//! SIMD optimization validation tests
//!
//! These tests verify that SIMD-accelerated preprocessing
//! produces identical results to scalar implementation.

use hedl_core::{preprocess, Limits};

#[test]
fn test_simd_empty_input() {
    let input = b"";
    let result = preprocess(input, &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], (1, ""));
}

#[test]
fn test_simd_single_line_no_newline() {
    let input = b"hello world";
    let result = preprocess(input, &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], (1, "hello world"));
}

#[test]
fn test_simd_many_empty_lines() {
    let input = b"\n\n\n\n\n";
    let result = preprocess(input, &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();
    assert_eq!(lines.len(), 6);
    for (i, (line_num, content)) in lines.iter().enumerate() {
        assert_eq!(*line_num, i + 1);
        assert_eq!(*content, "");
    }
}

#[test]
fn test_simd_long_lines() {
    // Test SIMD performance on very long lines
    let long_line = "x".repeat(100_000);
    let mut input = format!("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n{long_line}\n");
    input.push_str("end: true\n");

    let result = preprocess(input.as_bytes(), &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    // v2.0 header: %V:2.0, %NULL:~, %QUOTE:", --- = 4 lines, plus long_line, end: true, empty = 7
    assert_eq!(lines.len(), 7);
    assert_eq!(lines[0].1, "%V:2.0");
    assert_eq!(lines[4].1.len(), 100_000);
    assert_eq!(lines[5].1, "end: true");
}

#[test]
fn test_simd_many_short_lines() {
    // Test SIMD efficiency on many short lines
    let mut input = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n");

    for i in 0..10_000 {
        input.push_str(&format!("k{i}: v{i}\n"));
    }

    let result = preprocess(input.as_bytes(), &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    // v2.0 header: %V:2.0, %NULL:~, %QUOTE:", --- = 4 lines, plus 10000 data lines, plus empty = 10005
    assert_eq!(lines.len(), 10_005);
    assert_eq!(lines[0].1, "%V:2.0");
    assert_eq!(lines[3].1, "---");
    assert_eq!(lines[4].1, "k0: v0");
    assert_eq!(lines[10_003].1, "k9999: v9999");
    assert_eq!(lines[10_004].1, ""); // Empty line after final newline
}

#[test]
fn test_simd_control_char_early() {
    // Control char on line 2 (early detection)
    let input = b"line1\nline2\x00\nline3\n";
    let result = preprocess(input, &Limits::default());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.line, 2);
    assert!(err.message.contains("U+0000"));
}

#[test]
fn test_simd_control_char_late() {
    // Control char on line 9999 (test early termination)
    let mut input = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n");

    for i in 0..9997 {
        input.push_str(&format!("k{i}: v{i}\n"));
    }

    // Add line with control char (line 10002 = 4 header lines + 9997 data lines + this line)
    input.push_str("bad\x00line\n");

    let result = preprocess(input.as_bytes(), &Limits::default());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.line, 10_002); // 4 header lines + 9997 data lines + 1 bad line = 10002
    assert!(err.message.contains("U+0000"));
}

#[test]
fn test_simd_line_length_limit_exact() {
    let limits = Limits {
        max_line_length: 10,
        ..Limits::default()
    };

    // Exactly at limit
    let input = b"0123456789\n";
    let result = preprocess(input, &limits);
    assert!(result.is_ok());

    // One byte over limit
    let input = b"01234567890\n";
    let result = preprocess(input, &limits);
    assert!(result.is_err());
}

#[test]
fn test_simd_mixed_line_lengths() {
    // Realistic scenario with varied line lengths
    let input = concat!(
        "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n",
        "---\n",
        "short: 1\n",
        "medium_key_with_longer_value: this is a medium length line\n",
        "k: v\n",
        "very_long_key_with_extremely_long_value_that_spans: ",
        "many many many many many characters in total\n",
        "end: true\n"
    );

    let result = preprocess(input.as_bytes(), &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    // 4 header lines + 6 data lines (including extra ---) + 1 empty = 11
    assert_eq!(lines.len(), 11);
    assert_eq!(lines[0].1, "%V:2.0");
    assert_eq!(lines[5].1, "short: 1");
}

#[test]
fn test_simd_unicode_with_newlines() {
    // Ensure SIMD newline finding works correctly with multi-byte UTF-8
    let input = "こんにちは\n世界\n🌍🌎🌏\n";

    let result = preprocess(input.as_bytes(), &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].1, "こんにちは");
    assert_eq!(lines[1].1, "世界");
    assert_eq!(lines[2].1, "🌍🌎🌏");
}

#[test]
fn test_simd_stress_100k_lines() {
    // Stress test: 100K lines
    let mut input = String::from("%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n");

    for i in 0..100_000 {
        input.push_str(&format!("key{i}: value{i}\n"));
    }

    let result = preprocess(input.as_bytes(), &Limits::unlimited()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    // v2.0 header: 4 lines + 100000 data lines + empty line after final \n = 100005
    assert_eq!(lines.len(), 100_005);
}

#[test]
fn test_simd_consecutive_control_chars() {
    // Multiple control chars in same line
    let input = b"hello\x00\x01\x02world\n";
    let result = preprocess(input, &Limits::default());

    assert!(result.is_err());
    // Should detect first control char
    let err = result.unwrap_err();
    assert!(err.message.contains("U+0000"));
}

#[test]
fn test_simd_control_char_at_line_start() {
    let input = b"\x00line1\nline2\n";
    let result = preprocess(input, &Limits::default());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.line, 1);
    assert!(err.message.contains("U+0000"));
}

#[test]
fn test_simd_control_char_at_line_end() {
    let input = b"line1\x00\nline2\n";
    let result = preprocess(input, &Limits::default());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.line, 1);
}

#[test]
fn test_simd_allowed_control_chars() {
    // Tabs and carriage returns (when normalized) should be allowed
    let input = b"a\tb\tc\r\nd\te\n";
    let result = preprocess(input, &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].1, "a\tb\tc");
    assert_eq!(lines[1].1, "d\te");
}

#[test]
fn test_simd_realistic_hedl_document() {
    let input = concat!(
        "%V:2.0\n%NULL:~\n%QUOTE:\"\n---\n",
        "---\n",
        "\n",
        "# User records\n",
        "\n",
        "user-1:\n",
        "  id: 1\n",
        "  name: Alice\n",
        "  email: alice@example.com\n",
        "\n",
        "user-2:\n",
        "  id: 2\n",
        "  name: Bob\n",
        "  email: bob@example.com\n",
        "\n",
        "# End of file\n"
    );

    let result = preprocess(input.as_bytes(), &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    // 4 header lines + 16 body lines (including extra --- and empty lines) = 20
    assert_eq!(lines.len(), 20);
    assert_eq!(lines[0].1, "%V:2.0");
    assert_eq!(lines[3].1, "---");
    assert_eq!(lines[6].1, "# User records");
}

#[test]
fn test_simd_line_length_last_line_no_newline() {
    let limits = Limits {
        max_line_length: 10,
        ..Limits::default()
    };

    // Last line without newline, within limit
    let input = b"short\n0123456789";
    let result = preprocess(input, &limits);
    assert!(result.is_ok());

    // Last line without newline, over limit
    let input = b"short\n01234567890";
    let result = preprocess(input, &limits);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.line, 2);
}

#[test]
fn test_simd_alternating_short_long_lines() {
    let mut input = String::new();

    for i in 0..1000 {
        if i % 2 == 0 {
            input.push_str("short\n");
        } else {
            input.push_str(&"x".repeat(100));
            input.push('\n');
        }
    }

    let result = preprocess(input.as_bytes(), &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    // 1000 lines + empty line after final \n = 1001
    assert_eq!(lines.len(), 1001);
    assert_eq!(lines[0].1, "short");
    assert_eq!(lines[1].1.len(), 100);
    assert_eq!(lines[1000].1, ""); // Empty line after final newline
}

#[test]
fn test_simd_edge_case_single_newline() {
    let input = b"\n";
    let result = preprocess(input, &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], (1, ""));
    assert_eq!(lines[1], (2, ""));
}

#[test]
fn test_simd_edge_case_no_content() {
    let input = b"";
    let result = preprocess(input, &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], (1, ""));
}

#[test]
fn test_simd_performance_dense_newlines() {
    // Test case with newlines every few characters
    let mut input = String::new();

    for i in 0..10_000 {
        input.push_str(&format!("{i}\n"));
    }

    let result = preprocess(input.as_bytes(), &Limits::default()).unwrap();
    let lines: Vec<_> = result.lines().collect();

    // 10000 lines + empty line after final \n = 10001
    assert_eq!(lines.len(), 10_001);
    assert_eq!(lines[0].1, "0");
    assert_eq!(lines[9999].1, "9999");
    assert_eq!(lines[10_000].1, ""); // Empty line after final newline
}
