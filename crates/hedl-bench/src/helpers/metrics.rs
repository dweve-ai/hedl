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

//! Metrics and measurement utilities for benchmarks.
//!
//! Provides common functionality for:
//! - Size comparison between formats
//! - Throughput measurement

/// Size comparison between formats.
#[derive(Debug, Clone)]
pub struct SizeComparison {
    /// HEDL size in bytes.
    pub hedl_bytes: usize,
    /// Other format size in bytes.
    pub other_bytes: usize,
    /// Size ratio (`other_bytes` / `hedl_bytes`).
    pub ratio: f64,
    /// Percentage saved by HEDL.
    pub hedl_savings_pct: f64,
}

/// Compare sizes between HEDL and another format.
///
/// # Arguments
///
/// * `hedl_bytes` - Size of HEDL representation in bytes
/// * `other_bytes` - Size of other format representation in bytes
///
/// # Returns
///
/// A `SizeComparison` with ratio and savings calculations.
///
/// # Example
///
/// ```
/// use hedl_bench::helpers::metrics::compare_sizes;
///
/// let comparison = compare_sizes(100, 200);
/// assert_eq!(comparison.ratio, 2.0);
/// assert_eq!(comparison.hedl_savings_pct, 50.0);
/// ```
#[must_use]
pub fn compare_sizes(hedl_bytes: usize, other_bytes: usize) -> SizeComparison {
    let ratio = if hedl_bytes > 0 {
        other_bytes as f64 / hedl_bytes as f64
    } else {
        0.0
    };

    let hedl_savings_pct = if other_bytes > 0 {
        ((other_bytes - hedl_bytes) as f64 / other_bytes as f64) * 100.0
    } else {
        0.0
    };

    SizeComparison {
        hedl_bytes,
        other_bytes,
        ratio,
        hedl_savings_pct,
    }
}

/// Measure throughput from raw nanoseconds.
///
/// # Arguments
///
/// * `bytes` - Number of bytes processed
/// * `nanos` - Time taken in nanoseconds
///
/// # Returns
///
/// Throughput in megabytes per second (MB/s).
///
/// # Example
///
/// ```
/// use hedl_bench::helpers::metrics::measure_throughput_ns;
///
/// // 10 MB in 1 second = 10 MB/s
/// let throughput = measure_throughput_ns(10_000_000, 1_000_000_000);
/// assert!((throughput - 10.0).abs() < 0.01);
/// ```
#[must_use]
pub fn measure_throughput_ns(bytes: usize, nanos: u64) -> f64 {
    if nanos > 0 {
        let bytes_per_sec = (bytes as f64 * 1e9) / nanos as f64;
        bytes_per_sec / 1_000_000.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_sizes() {
        let comparison = compare_sizes(100, 200);

        assert_eq!(comparison.hedl_bytes, 100);
        assert_eq!(comparison.other_bytes, 200);
        assert!((comparison.ratio - 2.0).abs() < 0.01);
        assert!((comparison.hedl_savings_pct - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_compare_sizes_zero_hedl() {
        let comparison = compare_sizes(0, 200);
        assert_eq!(comparison.ratio, 0.0);
    }

    #[test]
    fn test_compare_sizes_zero_other() {
        let comparison = compare_sizes(100, 0);
        assert_eq!(comparison.hedl_savings_pct, 0.0);
    }

    #[test]
    fn test_measure_throughput_ns() {
        let throughput = measure_throughput_ns(10_000_000, 1_000_000_000);
        assert!((throughput - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_measure_throughput_ns_zero_time() {
        let throughput = measure_throughput_ns(1000, 0);
        assert_eq!(throughput, 0.0);
    }
}
