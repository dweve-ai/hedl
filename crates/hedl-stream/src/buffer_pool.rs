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

//! Buffer pooling for memory-efficient parsing.
//!
//! This module provides object pooling for frequently allocated objects
//! during parsing (strings and value vectors). Pooling reduces allocator
//! pressure in high-throughput scenarios.
//!
//! # Design
//!
//! - **Type-specific pools**: Separate pools for String and Vec<Value>
//! - **Capacity limits**: Configurable maximum pool sizes to prevent unbounded growth
//! - **Clear-on-release**: Buffers are cleared but capacity is retained
//! - **Lazy growth**: Pools grow on demand up to configured limits
//!
//! # Performance Characteristics
//!
//! - **Allocation elimination**: Reuses buffers instead of allocating new ones
//! - **Reduced GC pressure**: Fewer allocations means less garbage collection
//! - **Memory trade-off**: Holds buffers between operations (configurable)
//!
//! # Use Cases
//!
//! - **High-throughput parsing**: Many files processed in sequence
//! - **Server workloads**: Long-running processes with continuous parsing
//! - **Memory-constrained with high volume**: Amortize allocation cost

use hedl_core::Value;

/// Memory limits for buffer management.
///
/// Controls maximum buffer sizes, line lengths, and pool configuration
/// to prevent unbounded memory growth and handle memory-constrained environments.
///
/// # Examples
///
/// ## Default Configuration
///
/// ```rust
/// use hedl_stream::MemoryLimits;
///
/// let limits = MemoryLimits::default();
/// assert_eq!(limits.max_buffer_size, 1024 * 1024);
/// assert_eq!(limits.max_line_length, 1_000_000);
/// assert_eq!(limits.enable_buffer_pooling, true);
/// assert_eq!(limits.max_pool_size, 10);
/// ```
///
/// ## Memory-Constrained Configuration
///
/// ```rust
/// use hedl_stream::MemoryLimits;
///
/// let limits = MemoryLimits {
///     max_buffer_size: 64 * 1024,       // 64KB max I/O buffer
///     max_line_length: 100_000,         // 100KB max line
///     enable_buffer_pooling: false,     // Disable pooling
///     max_pool_size: 0,
/// };
/// ```
///
/// ## High-Throughput Configuration
///
/// ```rust
/// use hedl_stream::MemoryLimits;
///
/// let limits = MemoryLimits {
///     max_buffer_size: 2 * 1024 * 1024, // 2MB max I/O buffer
///     max_line_length: 10_000_000,      // 10MB max line
///     enable_buffer_pooling: true,
///     max_pool_size: 50,                // Large pool
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryLimits {
    /// Maximum I/O buffer size in bytes.
    ///
    /// Controls the size of the read buffer used by `BufReader`.
    /// Larger buffers reduce syscall overhead but use more memory.
    ///
    /// Default: 1MB
    pub max_buffer_size: usize,

    /// Maximum line length in bytes.
    ///
    /// Lines exceeding this length cause a parsing error.
    /// This protects against malformed input with extremely long lines.
    ///
    /// Default: 1,000,000 bytes (1MB)
    pub max_line_length: usize,

    /// Enable buffer pooling.
    ///
    /// When true, reuses string and value buffers across parsing operations.
    /// Reduces allocation overhead in high-throughput scenarios.
    ///
    /// Default: true
    pub enable_buffer_pooling: bool,

    /// Maximum number of buffers to pool.
    ///
    /// Limits pool growth to prevent unbounded memory usage.
    /// Only effective when `enable_buffer_pooling` is true.
    ///
    /// Default: 10 buffers
    pub max_pool_size: usize,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            max_buffer_size: 1024 * 1024, // 1MB max I/O buffer
            max_line_length: 1_000_000,   // 1MB max line
            enable_buffer_pooling: true,
            max_pool_size: 10, // Pool up to 10 buffers
        }
    }
}

impl MemoryLimits {
    /// Configuration for embedded systems or memory-constrained environments.
    ///
    /// Uses minimal buffer sizes and disables pooling to minimize memory footprint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::MemoryLimits;
    ///
    /// let limits = MemoryLimits::embedded();
    /// assert_eq!(limits.max_buffer_size, 8 * 1024);
    /// assert_eq!(limits.enable_buffer_pooling, false);
    /// ```
    #[must_use]
    pub fn embedded() -> Self {
        Self {
            max_buffer_size: 8 * 1024,    // 8KB buffer
            max_line_length: 10_000,      // 10KB max line
            enable_buffer_pooling: false, // No pooling on embedded
            max_pool_size: 0,
        }
    }

    /// Configuration for large file processing with high throughput.
    ///
    /// Uses large buffers and extensive pooling for maximum performance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::MemoryLimits;
    ///
    /// let limits = MemoryLimits::high_throughput();
    /// assert_eq!(limits.max_buffer_size, 2 * 1024 * 1024);
    /// assert_eq!(limits.max_pool_size, 50);
    /// ```
    #[must_use]
    pub fn high_throughput() -> Self {
        Self {
            max_buffer_size: 2 * 1024 * 1024, // 2MB buffer
            max_line_length: 10_000_000,      // 10MB max line
            enable_buffer_pooling: true,
            max_pool_size: 50, // Large pool
        }
    }

    /// Configuration for untrusted input.
    ///
    /// Uses conservative limits to protect against malicious input.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::MemoryLimits;
    ///
    /// let limits = MemoryLimits::untrusted();
    /// assert_eq!(limits.max_line_length, 100_000);
    /// ```
    #[must_use]
    pub fn untrusted() -> Self {
        Self {
            max_buffer_size: 64 * 1024, // 64KB buffer
            max_line_length: 100_000,   // 100KB max line
            enable_buffer_pooling: true,
            max_pool_size: 5, // Small pool
        }
    }
}

/// Buffer pool for String and Vec<Value> reuse.
///
/// Maintains pools of pre-allocated buffers to reduce allocation overhead
/// during high-throughput parsing operations.
///
/// # Memory Management
///
/// - Buffers are cleared (content removed) when released but capacity is retained
/// - Pool size is limited to prevent unbounded memory growth
/// - Acquire operations fall back to fresh allocation if pool is empty
/// - Release operations drop buffers if pool is full
///
/// # Thread Safety
///
/// This is NOT thread-safe. Each parser instance should have its own pool.
/// For multi-threaded scenarios, use one pool per thread.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use hedl_stream::BufferPool;
///
/// let mut pool = BufferPool::new(10);
///
/// // Acquire a string buffer
/// let mut s = pool.acquire_string();
/// s.push_str("hello");
///
/// // Release it back to pool
/// pool.release_string(s);
///
/// // Next acquire reuses the buffer
/// let s2 = pool.acquire_string();
/// assert_eq!(s2.len(), 0);  // Cleared but capacity retained
/// ```
///
/// ## With Capacity Hints
///
/// ```rust
/// use hedl_stream::BufferPool;
///
/// let mut pool = BufferPool::with_capacity_hints(10, 256, 16);
///
/// let s = pool.acquire_string();
/// assert!(s.capacity() >= 256);
///
/// let v = pool.acquire_value_vec();
/// assert!(v.capacity() >= 16);
/// ```
#[derive(Debug)]
pub struct BufferPool {
    string_pool: Vec<String>,
    value_pool: Vec<Vec<Value>>,
    max_pool_size: usize,
    string_capacity_hint: usize,
    value_capacity_hint: usize,
}

impl BufferPool {
    /// Create a new buffer pool with default capacity hints.
    ///
    /// # Parameters
    ///
    /// - `max_pool_size`: Maximum number of buffers to pool per type
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferPool;
    ///
    /// let pool = BufferPool::new(10);
    /// ```
    #[must_use]
    pub fn new(max_pool_size: usize) -> Self {
        Self {
            string_pool: Vec::with_capacity(max_pool_size.min(4)),
            value_pool: Vec::with_capacity(max_pool_size.min(4)),
            max_pool_size,
            string_capacity_hint: 256,
            value_capacity_hint: 16,
        }
    }

    /// Create a buffer pool with custom capacity hints.
    ///
    /// Capacity hints determine the initial capacity of newly allocated buffers
    /// when the pool is empty.
    ///
    /// # Parameters
    ///
    /// - `max_pool_size`: Maximum buffers to pool per type
    /// - `string_capacity_hint`: Initial capacity for String buffers
    /// - `value_capacity_hint`: Initial capacity for Vec<Value> buffers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferPool;
    ///
    /// // Pool for parsing with long lines and wide rows
    /// let pool = BufferPool::with_capacity_hints(20, 1024, 50);
    /// ```
    #[must_use]
    pub fn with_capacity_hints(
        max_pool_size: usize,
        string_capacity_hint: usize,
        value_capacity_hint: usize,
    ) -> Self {
        Self {
            string_pool: Vec::with_capacity(max_pool_size.min(4)),
            value_pool: Vec::with_capacity(max_pool_size.min(4)),
            max_pool_size,
            string_capacity_hint,
            value_capacity_hint,
        }
    }

    /// Acquire a String buffer from the pool.
    ///
    /// Returns a pooled buffer if available, otherwise allocates a new one
    /// with the configured capacity hint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferPool;
    ///
    /// let mut pool = BufferPool::new(10);
    /// let mut s = pool.acquire_string();
    /// s.push_str("data");
    /// ```
    #[inline]
    pub fn acquire_string(&mut self) -> String {
        self.string_pool
            .pop()
            .unwrap_or_else(|| String::with_capacity(self.string_capacity_hint))
    }

    /// Release a String buffer back to the pool.
    ///
    /// The buffer is cleared but retains its capacity. If the pool is full,
    /// the buffer is dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferPool;
    ///
    /// let mut pool = BufferPool::new(10);
    /// let mut s = pool.acquire_string();
    /// s.push_str("data");
    /// pool.release_string(s);
    ///
    /// // Buffer is reused with content cleared
    /// let s2 = pool.acquire_string();
    /// assert_eq!(s2.len(), 0);
    /// ```
    #[inline]
    pub fn release_string(&mut self, mut s: String) {
        if self.string_pool.len() < self.max_pool_size {
            s.clear();
            self.string_pool.push(s);
        }
        // Otherwise drop the buffer
    }

    /// Acquire a Vec<Value> buffer from the pool.
    ///
    /// Returns a pooled buffer if available, otherwise allocates a new one
    /// with the configured capacity hint.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferPool;
    /// use hedl_core::Value;
    ///
    /// let mut pool = BufferPool::new(10);
    /// let mut v = pool.acquire_value_vec();
    /// v.push(Value::Int(42));
    /// ```
    #[inline]
    pub fn acquire_value_vec(&mut self) -> Vec<Value> {
        self.value_pool
            .pop()
            .unwrap_or_else(|| Vec::with_capacity(self.value_capacity_hint))
    }

    /// Release a Vec<Value> buffer back to the pool.
    ///
    /// The buffer is cleared but retains its capacity. If the pool is full,
    /// the buffer is dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferPool;
    /// use hedl_core::Value;
    ///
    /// let mut pool = BufferPool::new(10);
    /// let mut v = pool.acquire_value_vec();
    /// v.push(Value::Int(42));
    /// pool.release_value_vec(v);
    ///
    /// // Buffer is reused with content cleared
    /// let v2 = pool.acquire_value_vec();
    /// assert_eq!(v2.len(), 0);
    /// ```
    #[inline]
    pub fn release_value_vec(&mut self, mut v: Vec<Value>) {
        if self.value_pool.len() < self.max_pool_size {
            v.clear();
            self.value_pool.push(v);
        }
        // Otherwise drop the buffer
    }

    /// Get the current number of String buffers in the pool.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferPool;
    ///
    /// let mut pool = BufferPool::new(10);
    /// assert_eq!(pool.string_pool_size(), 0);
    ///
    /// let s = pool.acquire_string();
    /// pool.release_string(s);
    /// assert_eq!(pool.string_pool_size(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn string_pool_size(&self) -> usize {
        self.string_pool.len()
    }

    /// Get the current number of Vec<Value> buffers in the pool.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferPool;
    ///
    /// let mut pool = BufferPool::new(10);
    /// assert_eq!(pool.value_pool_size(), 0);
    ///
    /// let v = pool.acquire_value_vec();
    /// pool.release_value_vec(v);
    /// assert_eq!(pool.value_pool_size(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub fn value_pool_size(&self) -> usize {
        self.value_pool.len()
    }

    /// Clear all buffers from the pool.
    ///
    /// This releases all pooled buffers, freeing memory back to the allocator.
    /// Useful for manual memory management or cleanup.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferPool;
    ///
    /// let mut pool = BufferPool::new(10);
    /// let s = pool.acquire_string();
    /// pool.release_string(s);
    /// assert_eq!(pool.string_pool_size(), 1);
    ///
    /// pool.clear();
    /// assert_eq!(pool.string_pool_size(), 0);
    /// ```
    pub fn clear(&mut self) {
        self.string_pool.clear();
        self.value_pool.clear();
    }

    /// Get the maximum pool size.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hedl_stream::BufferPool;
    ///
    /// let pool = BufferPool::new(15);
    /// assert_eq!(pool.max_pool_size(), 15);
    /// ```
    #[inline]
    #[must_use]
    pub fn max_pool_size(&self) -> usize {
        self.max_pool_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== MemoryLimits tests ====================

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
    fn test_memory_limits_custom() {
        let limits = MemoryLimits {
            max_buffer_size: 128 * 1024,
            max_line_length: 500_000,
            enable_buffer_pooling: false,
            max_pool_size: 3,
        };
        assert_eq!(limits.max_buffer_size, 128 * 1024);
        assert_eq!(limits.max_line_length, 500_000);
        assert!(!limits.enable_buffer_pooling);
        assert_eq!(limits.max_pool_size, 3);
    }

    #[test]
    fn test_memory_limits_clone() {
        let limits1 = MemoryLimits::default();
        let limits2 = limits1;
        assert_eq!(limits1, limits2);
    }

    #[test]
    fn test_memory_limits_debug() {
        let limits = MemoryLimits::default();
        let debug = format!("{limits:?}");
        assert!(debug.contains("MemoryLimits"));
    }

    // ==================== BufferPool basic tests ====================

    #[test]
    fn test_buffer_pool_new() {
        let pool = BufferPool::new(10);
        assert_eq!(pool.max_pool_size(), 10);
        assert_eq!(pool.string_pool_size(), 0);
        assert_eq!(pool.value_pool_size(), 0);
    }

    #[test]
    fn test_buffer_pool_with_capacity_hints() {
        let pool = BufferPool::with_capacity_hints(10, 512, 32);
        assert_eq!(pool.max_pool_size(), 10);
        assert_eq!(pool.string_capacity_hint, 512);
        assert_eq!(pool.value_capacity_hint, 32);
    }

    #[test]
    fn test_acquire_string_from_empty_pool() {
        let mut pool = BufferPool::new(10);
        let s = pool.acquire_string();
        assert_eq!(s.len(), 0);
        assert!(s.capacity() >= 256); // Default hint
    }

    #[test]
    fn test_release_and_acquire_string() {
        let mut pool = BufferPool::new(10);
        let mut s = pool.acquire_string();
        s.push_str("hello");
        assert_eq!(s.len(), 5);

        pool.release_string(s);
        assert_eq!(pool.string_pool_size(), 1);

        let s2 = pool.acquire_string();
        assert_eq!(s2.len(), 0); // Cleared
        assert!(s2.capacity() > 0); // Capacity retained
    }

    #[test]
    fn test_acquire_value_vec_from_empty_pool() {
        let mut pool = BufferPool::new(10);
        let v = pool.acquire_value_vec();
        assert_eq!(v.len(), 0);
        assert!(v.capacity() >= 16); // Default hint
    }

    #[test]
    fn test_release_and_acquire_value_vec() {
        let mut pool = BufferPool::new(10);
        let mut v = pool.acquire_value_vec();
        v.push(Value::Int(42));
        v.push(Value::Bool(true));
        assert_eq!(v.len(), 2);

        pool.release_value_vec(v);
        assert_eq!(pool.value_pool_size(), 1);

        let v2 = pool.acquire_value_vec();
        assert_eq!(v2.len(), 0); // Cleared
        assert!(v2.capacity() > 0); // Capacity retained
    }

    #[test]
    fn test_pool_size_limit_string() {
        let mut pool = BufferPool::new(2);

        // Acquire multiple buffers first
        let s1 = pool.acquire_string();
        let s2 = pool.acquire_string();
        let s3 = pool.acquire_string();

        // Release them to fill the pool
        pool.release_string(s1);
        pool.release_string(s2);
        assert_eq!(pool.string_pool_size(), 2);

        // Releasing another should not increase pool size (it gets dropped)
        pool.release_string(s3);
        assert_eq!(pool.string_pool_size(), 2);
    }

    #[test]
    fn test_pool_size_limit_value_vec() {
        let mut pool = BufferPool::new(2);

        // Acquire multiple buffers first
        let v1 = pool.acquire_value_vec();
        let v2 = pool.acquire_value_vec();
        let v3 = pool.acquire_value_vec();

        // Release them to fill the pool
        pool.release_value_vec(v1);
        pool.release_value_vec(v2);
        assert_eq!(pool.value_pool_size(), 2);

        // Releasing another should not increase pool size (it gets dropped)
        pool.release_value_vec(v3);
        assert_eq!(pool.value_pool_size(), 2);
    }

    #[test]
    fn test_clear_pool() {
        let mut pool = BufferPool::new(10);

        let s = pool.acquire_string();
        pool.release_string(s);
        let v = pool.acquire_value_vec();
        pool.release_value_vec(v);
        assert_eq!(pool.string_pool_size(), 1);
        assert_eq!(pool.value_pool_size(), 1);

        pool.clear();
        assert_eq!(pool.string_pool_size(), 0);
        assert_eq!(pool.value_pool_size(), 0);
    }

    #[test]
    fn test_multiple_acquire_release_cycles() {
        let mut pool = BufferPool::new(5);

        for i in 0..10 {
            let mut s = pool.acquire_string();
            s.push_str(&format!("iteration {i}"));
            pool.release_string(s);

            let mut v = pool.acquire_value_vec();
            v.push(Value::Int(i64::from(i)));
            pool.release_value_vec(v);
        }

        // Pool should be at max size
        assert!(pool.string_pool_size() <= 5);
        assert!(pool.value_pool_size() <= 5);
    }

    #[test]
    fn test_capacity_preserved_across_cycles() {
        let mut pool = BufferPool::new(10);

        let mut s = pool.acquire_string();
        s.push_str(&"x".repeat(1000)); // Force capacity growth
        let capacity_after_growth = s.capacity();
        pool.release_string(s);

        let s2 = pool.acquire_string();
        assert_eq!(s2.capacity(), capacity_after_growth);
    }

    #[test]
    fn test_custom_capacity_hints() {
        let mut pool = BufferPool::with_capacity_hints(10, 1024, 64);

        let s = pool.acquire_string();
        assert!(s.capacity() >= 1024);

        let v = pool.acquire_value_vec();
        assert!(v.capacity() >= 64);
    }

    #[test]
    fn test_zero_max_pool_size() {
        let mut pool = BufferPool::new(0);

        let s = pool.acquire_string();
        pool.release_string(s);
        let v = pool.acquire_value_vec();
        pool.release_value_vec(v);

        // Nothing should be pooled
        assert_eq!(pool.string_pool_size(), 0);
        assert_eq!(pool.value_pool_size(), 0);
    }

    #[test]
    fn test_large_pool_size() {
        let mut pool = BufferPool::new(100);

        // Acquire 50 strings
        let strings: Vec<_> = (0..50).map(|_| pool.acquire_string()).collect();

        // Release them all
        for s in strings {
            pool.release_string(s);
        }

        assert_eq!(pool.string_pool_size(), 50);

        // Acquire 50 value vecs
        let vecs: Vec<_> = (0..50).map(|_| pool.acquire_value_vec()).collect();

        // Release them all
        for v in vecs {
            pool.release_value_vec(v);
        }

        assert_eq!(pool.value_pool_size(), 50);
    }

    #[test]
    fn test_pool_independence() {
        let mut pool = BufferPool::new(10);

        // Acquire and release string buffers
        let strings: Vec<_> = (0..5).map(|_| pool.acquire_string()).collect();
        for s in strings {
            pool.release_string(s);
        }

        // Acquire and release value buffers
        let vecs: Vec<_> = (0..3).map(|_| pool.acquire_value_vec()).collect();
        for v in vecs {
            pool.release_value_vec(v);
        }

        assert_eq!(pool.string_pool_size(), 5);
        assert_eq!(pool.value_pool_size(), 3);

        // Acquiring from one pool doesn't affect the other
        let _ = pool.acquire_string();
        assert_eq!(pool.string_pool_size(), 4);
        assert_eq!(pool.value_pool_size(), 3);
    }

    #[test]
    fn test_string_content_cleared() {
        let mut pool = BufferPool::new(10);

        let mut s = pool.acquire_string();
        s.push_str("test data");
        pool.release_string(s);

        let s2 = pool.acquire_string();
        assert_eq!(s2.len(), 0);
        assert!(!s2.contains("test"));
    }

    #[test]
    fn test_value_vec_content_cleared() {
        let mut pool = BufferPool::new(10);

        let mut v = pool.acquire_value_vec();
        v.push(Value::Int(1));
        v.push(Value::Bool(true));
        v.push(Value::String("test".to_string().into()));
        pool.release_value_vec(v);

        let v2 = pool.acquire_value_vec();
        assert_eq!(v2.len(), 0);
    }

    #[test]
    fn test_pool_debug() {
        let pool = BufferPool::new(10);
        let debug = format!("{pool:?}");
        assert!(debug.contains("BufferPool"));
    }

    // ==================== Edge case tests ====================

    #[test]
    fn test_acquire_without_release() {
        let mut pool = BufferPool::new(10);

        // Acquire many without releasing
        for _ in 0..20 {
            let _s = pool.acquire_string();
            let _v = pool.acquire_value_vec();
        }

        // Pool should still be empty
        assert_eq!(pool.string_pool_size(), 0);
        assert_eq!(pool.value_pool_size(), 0);
    }

    #[test]
    fn test_release_without_acquire() {
        let mut pool = BufferPool::new(10);

        // Release externally created buffers
        pool.release_string(String::from("external"));
        pool.release_value_vec(vec![Value::Null]);

        assert_eq!(pool.string_pool_size(), 1);
        assert_eq!(pool.value_pool_size(), 1);
    }

    #[test]
    fn test_interleaved_operations() {
        let mut pool = BufferPool::new(5);

        let s1 = pool.acquire_string();
        let v1 = pool.acquire_value_vec();
        let s2 = pool.acquire_string();

        pool.release_string(s1);
        let v2 = pool.acquire_value_vec();
        pool.release_value_vec(v1);

        pool.release_string(s2);
        pool.release_value_vec(v2);

        assert_eq!(pool.string_pool_size(), 2);
        assert_eq!(pool.value_pool_size(), 2);
    }
}
