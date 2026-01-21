// Dweve HEDL - Hierarchical Entity Data Language
//
// Copyright (c) 2025 Dweve IP B.V. and individual contributors.
//
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the LICENSE file at the
// root of this repository or at: http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Buffer sizing configuration.
//!
//! Provides size classes and hints for optimizing buffer allocation
//! based on workload characteristics.

/// Buffer size hints for different workload profiles.
///
/// These size classes provide pre-configured buffer sizes optimized
/// for common use cases, from embedded systems to high-throughput
/// data processing.
///
/// # Size Classes
///
/// - **Small (8KB)**: Embedded systems, small config files, memory-constrained environments
/// - **Medium (64KB)**: Default for general use, good balance of performance and memory
/// - **Large (256KB)**: Large files, high-throughput scenarios, server workloads
/// - **Huge (1MB)**: Multi-GB files, maximum performance, minimal syscall overhead
///
/// # Performance Characteristics
///
/// Larger buffers reduce system call overhead but use more memory.
/// The optimal size depends on:
/// - File size (larger files benefit from larger buffers)
/// - Available memory (constrained systems need smaller buffers)
/// - I/O characteristics (fast storage benefits more from large buffers)
/// - Access pattern (sequential vs. random)
///
/// # Examples
///
/// ## Automatic Selection
///
/// ```rust
/// use hedl_stream::{StreamingParserConfig, BufferSizeHint};
///
/// let config = StreamingParserConfig::default()
///     .with_buffer_hint(BufferSizeHint::Large);
///
/// assert_eq!(config.buffer_size, 256 * 1024);
/// ```
///
/// ## Custom Configuration
///
/// ```rust
/// use hedl_stream::{StreamingParserConfig, BufferSizeHint};
///
/// // Small embedded device
/// let embedded_config = StreamingParserConfig::default()
///     .with_buffer_hint(BufferSizeHint::Small);
///
/// // High-throughput server
/// let server_config = StreamingParserConfig::default()
///     .with_buffer_hint(BufferSizeHint::Huge);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BufferSizeHint {
    /// 8KB buffer - for embedded systems and small files.
    ///
    /// **Use when:**
    /// - Parsing config files (<1MB)
    /// - Running on embedded systems
    /// - Memory is very limited (<10MB available)
    /// - Processing many small files concurrently
    ///
    /// **Trade-offs:**
    /// - Minimal memory footprint
    /// - More system calls for large files
    /// - Lower throughput on fast storage
    Small,

    /// 64KB buffer - default for general use.
    ///
    /// **Use when:**
    /// - General-purpose parsing
    /// - No specific performance requirements
    /// - Mixed file sizes
    /// - Standard development environments
    ///
    /// **Trade-offs:**
    /// - Good balance of memory and performance
    /// - Suitable for most workloads
    /// - May not be optimal for extremes
    #[default]
    Medium,

    /// 256KB buffer - for large files and high throughput.
    ///
    /// **Use when:**
    /// - Parsing large files (>100MB)
    /// - High-throughput ETL pipelines
    /// - Fast storage (`NVMe` SSD)
    /// - Server environments with available memory
    ///
    /// **Trade-offs:**
    /// - Reduced syscall overhead
    /// - Better throughput on large files
    /// - Higher memory usage per parser
    Large,

    /// 1MB buffer - maximum performance for huge files.
    ///
    /// **Use when:**
    /// - Parsing multi-GB files
    /// - Maximum throughput required
    /// - Abundant memory available
    /// - Single-threaded processing
    ///
    /// **Trade-offs:**
    /// - Minimal syscall overhead
    /// - Maximum throughput
    /// - Significant memory per parser (limits concurrency)
    Huge,
}

impl BufferSizeHint {
    /// Get the buffer size in bytes for this hint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferSizeHint;
    ///
    /// assert_eq!(BufferSizeHint::Small.size(), 8 * 1024);
    /// assert_eq!(BufferSizeHint::Medium.size(), 64 * 1024);
    /// assert_eq!(BufferSizeHint::Large.size(), 256 * 1024);
    /// assert_eq!(BufferSizeHint::Huge.size(), 1024 * 1024);
    /// ```
    #[inline]
    #[must_use]
    pub const fn size(self) -> usize {
        match self {
            Self::Small => 8 * 1024,
            Self::Medium => 64 * 1024,
            Self::Large => 256 * 1024,
            Self::Huge => 1024 * 1024,
        }
    }

    /// Get a buffer size hint based on file size.
    ///
    /// Automatically selects an appropriate buffer size based on the
    /// total size of the file being parsed.
    ///
    /// # Heuristics
    ///
    /// - Files <1MB: Small (8KB)
    /// - Files 1-100MB: Medium (64KB)
    /// - Files 100MB-1GB: Large (256KB)
    /// - Files >1GB: Huge (1MB)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferSizeHint;
    ///
    /// let hint = BufferSizeHint::for_file_size(500 * 1024); // 500KB
    /// assert_eq!(hint, BufferSizeHint::Small);
    ///
    /// let hint = BufferSizeHint::for_file_size(50 * 1024 * 1024); // 50MB
    /// assert_eq!(hint, BufferSizeHint::Medium);
    ///
    /// let hint = BufferSizeHint::for_file_size(500 * 1024 * 1024); // 500MB
    /// assert_eq!(hint, BufferSizeHint::Large);
    ///
    /// let hint = BufferSizeHint::for_file_size(2 * 1024 * 1024 * 1024); // 2GB
    /// assert_eq!(hint, BufferSizeHint::Huge);
    /// ```
    #[must_use]
    pub fn for_file_size(size_bytes: u64) -> Self {
        const MB: u64 = 1024 * 1024;
        const GB: u64 = 1024 * MB;

        if size_bytes < MB {
            Self::Small
        } else if size_bytes < 100 * MB {
            Self::Medium
        } else if size_bytes < GB {
            Self::Large
        } else {
            Self::Huge
        }
    }

    /// Get a buffer size hint for memory-constrained environments.
    ///
    /// Recommends a buffer size that won't exceed the given memory budget
    /// when running `concurrent_parsers` simultaneously.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferSizeHint;
    ///
    /// // 10MB available, running 10 parsers concurrently
    /// let hint = BufferSizeHint::for_memory_budget(10 * 1024 * 1024, 10);
    /// // Should suggest Small (8KB) since 10 * 64KB = 640KB is reasonable
    /// ```
    #[must_use]
    pub fn for_memory_budget(available_memory: usize, concurrent_parsers: usize) -> Self {
        if concurrent_parsers == 0 {
            return Self::Medium;
        }

        let budget_per_parser = available_memory / concurrent_parsers;

        // Reserve 2x buffer size for other allocations (line buffers, etc.)
        let effective_budget = budget_per_parser / 2;

        if effective_budget >= Self::Huge.size() {
            Self::Huge
        } else if effective_budget >= Self::Large.size() {
            Self::Large
        } else if effective_budget >= Self::Medium.size() {
            Self::Medium
        } else {
            Self::Small
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== BufferSizeHint::size tests ====================

    #[test]
    fn test_buffer_size_hint_sizes() {
        assert_eq!(BufferSizeHint::Small.size(), 8 * 1024);
        assert_eq!(BufferSizeHint::Medium.size(), 64 * 1024);
        assert_eq!(BufferSizeHint::Large.size(), 256 * 1024);
        assert_eq!(BufferSizeHint::Huge.size(), 1024 * 1024);
    }

    #[test]
    fn test_buffer_size_hint_ordering() {
        assert!(BufferSizeHint::Small.size() < BufferSizeHint::Medium.size());
        assert!(BufferSizeHint::Medium.size() < BufferSizeHint::Large.size());
        assert!(BufferSizeHint::Large.size() < BufferSizeHint::Huge.size());
    }

    // ==================== BufferSizeHint::for_file_size tests ====================

    #[test]
    fn test_for_file_size_tiny() {
        let hint = BufferSizeHint::for_file_size(1024); // 1KB
        assert_eq!(hint, BufferSizeHint::Small);
    }

    #[test]
    fn test_for_file_size_small() {
        let hint = BufferSizeHint::for_file_size(500 * 1024); // 500KB
        assert_eq!(hint, BufferSizeHint::Small);
    }

    #[test]
    fn test_for_file_size_boundary_1mb() {
        let hint = BufferSizeHint::for_file_size(1024 * 1024 - 1); // Just under 1MB
        assert_eq!(hint, BufferSizeHint::Small);

        let hint = BufferSizeHint::for_file_size(1024 * 1024); // Exactly 1MB
        assert_eq!(hint, BufferSizeHint::Medium);

        let hint = BufferSizeHint::for_file_size(1024 * 1024 + 1); // Just over 1MB
        assert_eq!(hint, BufferSizeHint::Medium);
    }

    #[test]
    fn test_for_file_size_medium() {
        let hint = BufferSizeHint::for_file_size(10 * 1024 * 1024); // 10MB
        assert_eq!(hint, BufferSizeHint::Medium);

        let hint = BufferSizeHint::for_file_size(50 * 1024 * 1024); // 50MB
        assert_eq!(hint, BufferSizeHint::Medium);
    }

    #[test]
    fn test_for_file_size_boundary_100mb() {
        let hint = BufferSizeHint::for_file_size(100 * 1024 * 1024 - 1); // Just under 100MB
        assert_eq!(hint, BufferSizeHint::Medium);

        let hint = BufferSizeHint::for_file_size(100 * 1024 * 1024); // Exactly 100MB
        assert_eq!(hint, BufferSizeHint::Large);

        let hint = BufferSizeHint::for_file_size(100 * 1024 * 1024 + 1); // Just over 100MB
        assert_eq!(hint, BufferSizeHint::Large);
    }

    #[test]
    fn test_for_file_size_large() {
        let hint = BufferSizeHint::for_file_size(500 * 1024 * 1024); // 500MB
        assert_eq!(hint, BufferSizeHint::Large);
    }

    #[test]
    fn test_for_file_size_boundary_1gb() {
        let hint = BufferSizeHint::for_file_size(1024 * 1024 * 1024 - 1); // Just under 1GB
        assert_eq!(hint, BufferSizeHint::Large);

        let hint = BufferSizeHint::for_file_size(1024 * 1024 * 1024); // Exactly 1GB
        assert_eq!(hint, BufferSizeHint::Huge);

        let hint = BufferSizeHint::for_file_size(1024 * 1024 * 1024 + 1); // Just over 1GB
        assert_eq!(hint, BufferSizeHint::Huge);
    }

    #[test]
    fn test_for_file_size_huge() {
        let hint = BufferSizeHint::for_file_size(10 * 1024 * 1024 * 1024); // 10GB
        assert_eq!(hint, BufferSizeHint::Huge);
    }

    #[test]
    fn test_for_file_size_zero() {
        let hint = BufferSizeHint::for_file_size(0);
        assert_eq!(hint, BufferSizeHint::Small);
    }

    // ==================== BufferSizeHint::for_memory_budget tests ====================

    #[test]
    fn test_for_memory_budget_abundant() {
        // 100MB available, 1 parser -> should suggest Huge
        let hint = BufferSizeHint::for_memory_budget(100 * 1024 * 1024, 1);
        assert_eq!(hint, BufferSizeHint::Huge);
    }

    #[test]
    fn test_for_memory_budget_comfortable() {
        // 50MB available, 10 parsers -> 5MB per parser -> 2.5MB effective -> Huge
        let hint = BufferSizeHint::for_memory_budget(50 * 1024 * 1024, 10);
        assert_eq!(hint, BufferSizeHint::Huge);
    }

    #[test]
    fn test_for_memory_budget_moderate() {
        // 10MB available, 10 parsers -> 1MB per parser -> 512KB effective -> Large
        let hint = BufferSizeHint::for_memory_budget(10 * 1024 * 1024, 10);
        assert_eq!(hint, BufferSizeHint::Large);
    }

    #[test]
    fn test_for_memory_budget_constrained() {
        // 2MB available, 10 parsers -> 200KB per parser -> 100KB effective -> Medium
        let hint = BufferSizeHint::for_memory_budget(2 * 1024 * 1024, 10);
        assert_eq!(hint, BufferSizeHint::Medium);
    }

    #[test]
    fn test_for_memory_budget_very_constrained() {
        // 500KB available, 10 parsers -> 50KB per parser -> Small
        let hint = BufferSizeHint::for_memory_budget(500 * 1024, 10);
        assert_eq!(hint, BufferSizeHint::Small);
    }

    #[test]
    fn test_for_memory_budget_zero_parsers() {
        // Edge case: 0 parsers should default to Medium
        let hint = BufferSizeHint::for_memory_budget(100 * 1024 * 1024, 0);
        assert_eq!(hint, BufferSizeHint::Medium);
    }

    #[test]
    fn test_for_memory_budget_one_parser() {
        let hint = BufferSizeHint::for_memory_budget(10 * 1024 * 1024, 1);
        assert_eq!(hint, BufferSizeHint::Huge);
    }

    #[test]
    fn test_for_memory_budget_many_parsers() {
        // 10MB available, 100 parsers -> 100KB per parser -> Small
        let hint = BufferSizeHint::for_memory_budget(10 * 1024 * 1024, 100);
        assert_eq!(hint, BufferSizeHint::Small);
    }

    #[test]
    fn test_for_memory_budget_boundary_huge_to_large() {
        // Boundary between Huge and Large
        // Huge needs 1MB, with 2x overhead = 2MB per parser
        let hint = BufferSizeHint::for_memory_budget(4 * 1024 * 1024, 1); // 4MB total
        assert_eq!(hint, BufferSizeHint::Huge);

        let hint = BufferSizeHint::for_memory_budget(4 * 1024 * 1024, 2); // 2MB per parser
        assert_eq!(hint, BufferSizeHint::Huge);

        let hint = BufferSizeHint::for_memory_budget(4 * 1024 * 1024, 3); // ~1.3MB per parser
        assert_eq!(hint, BufferSizeHint::Large);
    }

    // ==================== Default and basic trait tests ====================

    #[test]
    fn test_buffer_size_hint_default() {
        assert_eq!(BufferSizeHint::default(), BufferSizeHint::Medium);
    }

    #[test]
    fn test_buffer_size_hint_debug() {
        let small = BufferSizeHint::Small;
        let debug = format!("{small:?}");
        assert!(debug.contains("Small"));
    }

    #[test]
    fn test_buffer_size_hint_clone() {
        let hint1 = BufferSizeHint::Large;
        let hint2 = hint1;
        assert_eq!(hint1, hint2);
    }

    #[test]
    fn test_buffer_size_hint_equality() {
        assert_eq!(BufferSizeHint::Small, BufferSizeHint::Small);
        assert_ne!(BufferSizeHint::Small, BufferSizeHint::Medium);
    }

    #[test]
    fn test_buffer_size_hint_hash() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(BufferSizeHint::Small, "small");
        map.insert(BufferSizeHint::Medium, "medium");

        assert_eq!(map.get(&BufferSizeHint::Small), Some(&"small"));
        assert_eq!(map.get(&BufferSizeHint::Medium), Some(&"medium"));
    }

    // ==================== const function tests ====================

    #[test]
    fn test_size_is_const() {
        // Verify that size() can be used in const contexts
        const SMALL_SIZE: usize = BufferSizeHint::Small.size();
        assert_eq!(SMALL_SIZE, 8 * 1024);
    }
}
