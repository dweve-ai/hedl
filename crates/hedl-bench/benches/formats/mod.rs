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

//! Shared utilities for format conversion benchmarks.
//!
//! Provides common functionality for comparing HEDL against other formats:
//! - Format comparison metrics
//! - Throughput measurement
//! - Cross-format validation

use std::time::Duration;

/// Result of comparing HEDL against another format.
#[derive(Debug, Clone)]
pub struct FormatComparison {
    /// Name of the format being compared (e.g., "JSON", "YAML")
    pub format_name: String,
    /// HEDL conversion time in nanoseconds
    pub hedl_time_ns: u64,
    /// Other format conversion time in nanoseconds
    pub other_time_ns: u64,
    /// Speedup factor (other_time / hedl_time)
    pub speedup: f64,
    /// Size comparison if available
    pub size_comparison: Option<SizeComparison>,
}

/// Size comparison between formats
#[derive(Debug, Clone)]
pub struct SizeComparison {
    /// HEDL size in bytes
    pub hedl_bytes: usize,
    /// Other format size in bytes
    pub other_bytes: usize,
    /// Size ratio (other_bytes / hedl_bytes)
    pub ratio: f64,
    /// Percentage saved by HEDL
    pub hedl_savings_pct: f64,
}

/// Compare conversion times between HEDL and another format.
///
/// # Arguments
///
/// * `hedl_time` - Time taken for HEDL conversion
/// * `other_time` - Time taken for other format conversion
/// * `format` - Name of the other format (e.g., "JSON", "YAML")
///
/// # Returns
///
/// A `FormatComparison` struct containing the comparison metrics.
pub fn compare_formats(
    hedl_time: Duration,
    other_time: Duration,
    format: &str,
) -> FormatComparison {
    let hedl_ns = hedl_time.as_nanos() as u64;
    let other_ns = other_time.as_nanos() as u64;

    let speedup = if hedl_ns > 0 {
        other_ns as f64 / hedl_ns as f64
    } else {
        0.0
    };

    FormatComparison {
        format_name: format.to_string(),
        hedl_time_ns: hedl_ns,
        other_time_ns: other_ns,
        speedup,
        size_comparison: None,
    }
}

/// Compare sizes between HEDL and another format.
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

/// Measure throughput in MB/s.
///
/// # Arguments
///
/// * `bytes` - Number of bytes processed
/// * `duration` - Time taken to process the bytes
///
/// # Returns
///
/// Throughput in megabytes per second (MB/s).
pub fn measure_throughput(bytes: usize, duration: Duration) -> f64 {
    let seconds = duration.as_secs_f64();
    if seconds > 0.0 {
        (bytes as f64 / 1_000_000.0) / seconds
    } else {
        0.0
    }
}

/// Measure throughput from raw nanoseconds.
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
    fn test_compare_formats() {
        let hedl_time = Duration::from_millis(10);
        let json_time = Duration::from_millis(20);

        let comparison = compare_formats(hedl_time, json_time, "JSON");

        assert_eq!(comparison.format_name, "JSON");
        assert_eq!(comparison.hedl_time_ns, 10_000_000);
        assert_eq!(comparison.other_time_ns, 20_000_000);
        assert!((comparison.speedup - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_compare_sizes() {
        let comparison = compare_sizes(100, 200);

        assert_eq!(comparison.hedl_bytes, 100);
        assert_eq!(comparison.other_bytes, 200);
        assert!((comparison.ratio - 2.0).abs() < 0.01);
        assert!((comparison.hedl_savings_pct - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_measure_throughput() {
        let duration = Duration::from_secs(1);
        let throughput = measure_throughput(10_000_000, duration);
        assert!((throughput - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_measure_throughput_ns() {
        let throughput = measure_throughput_ns(10_000_000, 1_000_000_000);
        assert!((throughput - 10.0).abs() < 0.01);
    }
}
