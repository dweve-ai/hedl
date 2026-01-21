// Comprehensive tests for StreamingParserConfig and builder patterns
//
// Tests focus on:
// - Configuration builder pattern
// - Memory limits presets
// - Buffer size hint selection
// - Configuration interactions

use hedl_stream::{
    BufferSizeHint, MemoryLimits, StreamError, StreamingParser, StreamingParserConfig,
};
use std::io::Cursor;
use std::time::Duration;

// ==================== StreamingParserConfig builder tests ====================

#[test]
fn test_config_default_values() {
    let config = StreamingParserConfig::default();
    assert_eq!(config.max_line_length, 1_000_000);
    assert_eq!(config.max_indent_depth, 100);
    assert_eq!(config.buffer_size, 64 * 1024);
    assert_eq!(config.timeout, None);
    assert!(!config.enable_pooling);
}

#[test]
fn test_config_unlimited() {
    let config = StreamingParserConfig::unlimited();
    assert_eq!(config.max_line_length, usize::MAX);
    assert_eq!(config.max_indent_depth, 100);
}

#[test]
fn test_config_with_buffer_hint_small() {
    let config = StreamingParserConfig::default().with_buffer_hint(BufferSizeHint::Small);
    assert_eq!(config.buffer_size, 8 * 1024);
}

#[test]
fn test_config_with_buffer_hint_medium() {
    let config = StreamingParserConfig::default().with_buffer_hint(BufferSizeHint::Medium);
    assert_eq!(config.buffer_size, 64 * 1024);
}

#[test]
fn test_config_with_buffer_hint_large() {
    let config = StreamingParserConfig::default().with_buffer_hint(BufferSizeHint::Large);
    assert_eq!(config.buffer_size, 256 * 1024);
}

#[test]
fn test_config_with_buffer_hint_huge() {
    let config = StreamingParserConfig::default().with_buffer_hint(BufferSizeHint::Huge);
    assert_eq!(config.buffer_size, 1024 * 1024);
}

#[test]
fn test_config_with_buffer_pooling() {
    let config = StreamingParserConfig::default().with_buffer_pooling(true);
    assert!(config.enable_pooling);

    let config = StreamingParserConfig::default().with_buffer_pooling(false);
    assert!(!config.enable_pooling);
}

#[test]
fn test_config_with_memory_limits() {
    let limits = MemoryLimits::embedded();
    let config = StreamingParserConfig::default().with_memory_limits(limits);

    assert_eq!(config.memory_limits, limits);
    assert_eq!(config.max_line_length, limits.max_line_length);
}

#[test]
fn test_config_with_pool_size() {
    let config = StreamingParserConfig::default()
        .with_buffer_pooling(true)
        .with_pool_size(50);

    assert_eq!(config.memory_limits.max_pool_size, 50);
}

#[test]
fn test_config_builder_chaining() {
    let config = StreamingParserConfig::default()
        .with_buffer_hint(BufferSizeHint::Large)
        .with_buffer_pooling(true)
        .with_pool_size(25)
        .with_memory_limits(MemoryLimits::high_throughput());

    assert_eq!(config.buffer_size, 256 * 1024);
    assert!(config.enable_pooling);
    assert_eq!(config.memory_limits, MemoryLimits::high_throughput());
}

// ==================== MemoryLimits preset tests ====================

#[test]
fn test_memory_limits_default() {
    let limits = MemoryLimits::default();
    assert_eq!(limits.max_buffer_size, 1024 * 1024);
    assert_eq!(limits.max_line_length, 1_000_000);
    assert!(limits.enable_buffer_pooling);
    assert_eq!(limits.max_pool_size, 10);
}

#[test]
fn test_memory_limits_embedded() {
    let limits = MemoryLimits::embedded();
    assert_eq!(limits.max_buffer_size, 8 * 1024);
    assert_eq!(limits.max_line_length, 10_000);
    assert!(!limits.enable_buffer_pooling);
    assert_eq!(limits.max_pool_size, 0);
}

#[test]
fn test_memory_limits_high_throughput() {
    let limits = MemoryLimits::high_throughput();
    assert_eq!(limits.max_buffer_size, 2 * 1024 * 1024);
    assert_eq!(limits.max_line_length, 10_000_000);
    assert!(limits.enable_buffer_pooling);
    assert_eq!(limits.max_pool_size, 50);
}

#[test]
fn test_memory_limits_untrusted() {
    let limits = MemoryLimits::untrusted();
    assert_eq!(limits.max_buffer_size, 64 * 1024);
    assert_eq!(limits.max_line_length, 100_000);
    assert!(limits.enable_buffer_pooling);
    assert_eq!(limits.max_pool_size, 5);
}

#[test]
fn test_memory_limits_equality() {
    let limits1 = MemoryLimits::default();
    let limits2 = MemoryLimits::default();
    assert_eq!(limits1, limits2);

    let limits3 = MemoryLimits::embedded();
    assert_ne!(limits1, limits3);
}

// ==================== BufferSizeHint tests ====================

#[test]
fn test_buffer_size_hint_default() {
    assert_eq!(BufferSizeHint::default(), BufferSizeHint::Medium);
}

#[test]
fn test_buffer_size_hint_sizes() {
    assert_eq!(BufferSizeHint::Small.size(), 8 * 1024);
    assert_eq!(BufferSizeHint::Medium.size(), 64 * 1024);
    assert_eq!(BufferSizeHint::Large.size(), 256 * 1024);
    assert_eq!(BufferSizeHint::Huge.size(), 1024 * 1024);
}

#[test]
fn test_buffer_size_hint_for_file_size_boundaries() {
    // < 1MB -> Small
    assert_eq!(
        BufferSizeHint::for_file_size(1024 * 1024 - 1),
        BufferSizeHint::Small
    );
    assert_eq!(
        BufferSizeHint::for_file_size(1024 * 1024),
        BufferSizeHint::Medium
    );

    // 1MB - 100MB -> Medium
    assert_eq!(
        BufferSizeHint::for_file_size(50 * 1024 * 1024),
        BufferSizeHint::Medium
    );
    assert_eq!(
        BufferSizeHint::for_file_size(100 * 1024 * 1024 - 1),
        BufferSizeHint::Medium
    );
    assert_eq!(
        BufferSizeHint::for_file_size(100 * 1024 * 1024),
        BufferSizeHint::Large
    );

    // 100MB - 1GB -> Large
    assert_eq!(
        BufferSizeHint::for_file_size(500 * 1024 * 1024),
        BufferSizeHint::Large
    );
    assert_eq!(
        BufferSizeHint::for_file_size(1024 * 1024 * 1024 - 1),
        BufferSizeHint::Large
    );
    assert_eq!(
        BufferSizeHint::for_file_size(1024 * 1024 * 1024),
        BufferSizeHint::Huge
    );

    // > 1GB -> Huge
    assert_eq!(
        BufferSizeHint::for_file_size(10 * 1024 * 1024 * 1024),
        BufferSizeHint::Huge
    );
}

#[test]
fn test_buffer_size_hint_for_memory_budget() {
    // Abundant memory
    assert_eq!(
        BufferSizeHint::for_memory_budget(100 * 1024 * 1024, 1),
        BufferSizeHint::Huge
    );

    // Moderate memory
    assert_eq!(
        BufferSizeHint::for_memory_budget(10 * 1024 * 1024, 10),
        BufferSizeHint::Large
    );

    // Constrained memory
    assert_eq!(
        BufferSizeHint::for_memory_budget(2 * 1024 * 1024, 10),
        BufferSizeHint::Medium
    );

    // Very constrained
    assert_eq!(
        BufferSizeHint::for_memory_budget(500 * 1024, 10),
        BufferSizeHint::Small
    );
}

#[test]
fn test_buffer_size_hint_for_memory_budget_edge_cases() {
    // Zero parsers -> default to Medium
    assert_eq!(
        BufferSizeHint::for_memory_budget(100 * 1024 * 1024, 0),
        BufferSizeHint::Medium
    );

    // Single parser with large budget
    assert_eq!(
        BufferSizeHint::for_memory_budget(10 * 1024 * 1024, 1),
        BufferSizeHint::Huge
    );

    // Many parsers
    assert_eq!(
        BufferSizeHint::for_memory_budget(10 * 1024 * 1024, 100),
        BufferSizeHint::Small
    );
}

// ==================== Config integration with parser ====================

#[test]
fn test_parser_with_custom_config() {
    let config = StreamingParserConfig {
        max_line_length: 100,
        max_indent_depth: 10,
        buffer_size: 1024,
        timeout: Some(Duration::from_secs(5)),
        memory_limits: MemoryLimits::default(),
        enable_pooling: false,
    };

    let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name]
---
users: @User
  | alice, Alice
";

    let parser = StreamingParser::with_config(Cursor::new(input), config);
    assert!(parser.is_ok());
}

#[test]
fn test_parser_config_enforces_line_length() {
    let config = StreamingParserConfig {
        max_line_length: 20,
        ..Default::default()
    };

    let input = format!(
        "%VERSION: 1.0\n---\nkey: {}\n",
        "A".repeat(100) // Exceeds limit
    );

    let mut parser = StreamingParser::with_config(Cursor::new(input), config).unwrap();

    let mut found_error = false;
    for event in &mut parser {
        if let Err(StreamError::LineTooLong { .. }) = event {
            found_error = true;
            break;
        }
    }
    assert!(found_error);
}

#[test]
fn test_parser_config_with_timeout() {
    let config = StreamingParserConfig {
        timeout: Some(Duration::from_millis(1)),
        ..Default::default()
    };

    let input = r"
%VERSION: 1.0
---
key: value
";

    let parser = StreamingParser::with_config(Cursor::new(input), config);
    assert!(parser.is_ok());
}

#[test]
fn test_parser_unlimited_config() {
    let config = StreamingParserConfig::unlimited();

    let input = format!(
        "%VERSION: 1.0\n---\nkey: {}\n",
        "A".repeat(10_000) // Very long but should be OK with unlimited
    );

    let parser = StreamingParser::with_config(Cursor::new(input), config);
    assert!(parser.is_ok());
}

// ==================== Buffer hint integration tests ====================

#[test]
fn test_config_with_hint_for_small_file() {
    let config = StreamingParserConfig::default()
        .with_buffer_hint(BufferSizeHint::for_file_size(500 * 1024));

    assert_eq!(config.buffer_size, BufferSizeHint::Small.size());
}

#[test]
fn test_config_with_hint_for_large_file() {
    let config = StreamingParserConfig::default()
        .with_buffer_hint(BufferSizeHint::for_file_size(500 * 1024 * 1024));

    assert_eq!(config.buffer_size, BufferSizeHint::Large.size());
}

#[test]
fn test_config_with_hint_for_huge_file() {
    let config = StreamingParserConfig::default()
        .with_buffer_hint(BufferSizeHint::for_file_size(5 * 1024 * 1024 * 1024));

    assert_eq!(config.buffer_size, BufferSizeHint::Huge.size());
}

// ==================== Memory limits interaction tests ====================

#[test]
fn test_memory_limits_sync_with_config() {
    let limits = MemoryLimits {
        max_buffer_size: 128 * 1024,
        max_line_length: 50_000,
        enable_buffer_pooling: true,
        max_pool_size: 20,
    };

    let config = StreamingParserConfig::default().with_memory_limits(limits);

    assert_eq!(config.max_line_length, 50_000);
    assert_eq!(config.memory_limits.max_line_length, 50_000);
}

#[test]
fn test_embedded_config_full_chain() {
    let config = StreamingParserConfig::default()
        .with_memory_limits(MemoryLimits::embedded())
        .with_buffer_hint(BufferSizeHint::Small);

    assert_eq!(config.max_line_length, 10_000);
    assert_eq!(config.buffer_size, 8 * 1024);
    assert!(!config.enable_pooling);
}

#[test]
fn test_high_throughput_config_full_chain() {
    let config = StreamingParserConfig::default()
        .with_memory_limits(MemoryLimits::high_throughput())
        .with_buffer_hint(BufferSizeHint::Huge)
        .with_buffer_pooling(true)
        .with_pool_size(100);

    assert_eq!(config.max_line_length, 10_000_000);
    assert_eq!(config.buffer_size, 1024 * 1024);
    assert!(config.enable_pooling);
    assert_eq!(config.memory_limits.max_pool_size, 100);
}

// ==================== Clone and Debug trait tests ====================

#[test]
fn test_config_clone() {
    let config1 = StreamingParserConfig {
        max_line_length: 12345,
        max_indent_depth: 50,
        buffer_size: 32768,
        timeout: Some(Duration::from_secs(10)),
        memory_limits: MemoryLimits::default(),
        enable_pooling: true,
    };

    let config2 = config1.clone();
    assert_eq!(config1.max_line_length, config2.max_line_length);
    assert_eq!(config1.max_indent_depth, config2.max_indent_depth);
    assert_eq!(config1.buffer_size, config2.buffer_size);
    assert_eq!(config1.timeout, config2.timeout);
    assert_eq!(config1.enable_pooling, config2.enable_pooling);
}

#[test]
fn test_config_debug() {
    let config = StreamingParserConfig::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("StreamingParserConfig"));
}

#[test]
fn test_memory_limits_debug() {
    let limits = MemoryLimits::default();
    let debug_str = format!("{limits:?}");
    assert!(debug_str.contains("MemoryLimits"));
}

#[test]
fn test_buffer_size_hint_debug() {
    let hint = BufferSizeHint::Large;
    let debug_str = format!("{hint:?}");
    assert!(debug_str.contains("Large"));
}

// ==================== Real-world configuration scenarios ====================

#[test]
fn test_embedded_system_scenario() {
    let config = StreamingParserConfig::default()
        .with_memory_limits(MemoryLimits::embedded())
        .with_buffer_hint(BufferSizeHint::Small);

    let input = r"
%VERSION: 1.0
%STRUCT: Sensor: [id, value]
---
sensors: @Sensor
  | temp1, 23.5
  | temp2, 24.1
";

    let parser = StreamingParser::with_config(Cursor::new(input), config);
    assert!(parser.is_ok());
}

#[test]
fn test_server_workload_scenario() {
    let config = StreamingParserConfig::default()
        .with_memory_limits(MemoryLimits::high_throughput())
        .with_buffer_hint(BufferSizeHint::Large)
        .with_buffer_pooling(true)
        .with_pool_size(50);

    let input = r"
%VERSION: 1.0
%STRUCT: User: [id, name, email]
---
users: @User
  | alice, Alice, alice@example.com
  | bob, Bob, bob@example.com
";

    let parser = StreamingParser::with_config(Cursor::new(input), config);
    assert!(parser.is_ok());
}

#[test]
fn test_untrusted_input_scenario() {
    let config = StreamingParserConfig::default()
        .with_memory_limits(MemoryLimits::untrusted())
        .with_buffer_hint(BufferSizeHint::Medium);

    let input = r"
%VERSION: 1.0
---
data: untrusted
";

    let parser = StreamingParser::with_config(Cursor::new(input), config);
    assert!(parser.is_ok());
}
