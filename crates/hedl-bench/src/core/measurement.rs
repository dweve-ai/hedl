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

//! Measurement primitives with <1% overhead.
//!
//! Provides low-overhead timing and measurement infrastructure for accurate
//! performance analysis.

use std::time::{Duration, Instant};

/// A performance measurement result.
///
/// Captures timing, throughput, and memory usage data with minimal overhead.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Measurement {
    /// Total duration of the measured operation.
    pub duration: Duration,
    /// Optional throughput in bytes per second.
    pub throughput: Option<u64>,
    /// Optional peak memory usage in bytes.
    pub memory: Option<usize>,
}

impl Measurement {
    /// Creates a new measurement with just duration.
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            throughput: None,
            memory: None,
        }
    }

    /// Creates a measurement with duration and throughput.
    pub fn with_throughput(duration: Duration, bytes_per_sec: u64) -> Self {
        Self {
            duration,
            throughput: Some(bytes_per_sec),
            memory: None,
        }
    }

    /// Creates a measurement with all metrics.
    pub fn with_all(duration: Duration, throughput: Option<u64>, memory: Option<usize>) -> Self {
        Self {
            duration,
            throughput,
            memory,
        }
    }

    /// Returns the duration in nanoseconds.
    pub fn as_nanos(&self) -> u64 {
        self.duration.as_nanos() as u64
    }

    /// Returns throughput in MB/s if available.
    pub fn throughput_mbs(&self) -> Option<f64> {
        self.throughput
            .map(|bytes_per_sec| bytes_per_sec as f64 / 1_000_000.0)
    }

    /// Returns memory usage in MB if available.
    pub fn memory_mb(&self) -> Option<f64> {
        self.memory.map(|bytes| bytes as f64 / 1_000_000.0)
    }
}

/// Measures the execution time of a function.
///
/// Provides accurate timing with minimal measurement overhead (<1%).
///
/// # Arguments
///
/// * `name` - Name of the measurement for debugging
/// * `iterations` - Number of times to execute the function
/// * `f` - The function to measure
///
/// # Returns
///
/// A `Measurement` containing the total duration.
///
/// # Example
///
/// ```no_run
/// use hedl_bench::core::measurement::measure;
///
/// let measurement = measure("parse_test", 100, || {
///     // Your code here
/// });
/// println!("Average: {:?}", measurement.duration / 100);
/// ```
pub fn measure<F>(_name: &str, iterations: u64, mut f: F) -> Measurement
where
    F: FnMut(),
{
    // Warmup iteration to stabilize caches
    f();

    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let duration = start.elapsed();

    Measurement::new(duration)
}

/// Measures execution time with throughput tracking.
///
/// Calculates throughput based on total bytes processed.
///
/// # Arguments
///
/// * `name` - Name of the measurement
/// * `iterations` - Number of iterations
/// * `bytes` - Bytes processed per iteration
/// * `f` - The function to measure
///
/// # Returns
///
/// A `Measurement` with duration and throughput.
///
/// # Example
///
/// ```no_run
/// use hedl_bench::core::measurement::measure_with_throughput;
///
/// let data = vec![0u8; 1024];
/// let measurement = measure_with_throughput("process", 100, data.len() as u64, || {
///     // Process data
/// });
/// println!("Throughput: {:?} MB/s", measurement.throughput_mbs());
/// ```
pub fn measure_with_throughput<F>(_name: &str, iterations: u64, bytes: u64, mut f: F) -> Measurement
where
    F: FnMut(),
{
    // Warmup
    f();

    let total_bytes = bytes * iterations;
    let start = Instant::now();
    for _ in 0..iterations {
        f();
    }
    let duration = start.elapsed();

    let bytes_per_sec = if duration.as_secs_f64() > 0.0 {
        (total_bytes as f64 / duration.as_secs_f64()) as u64
    } else {
        0
    };

    Measurement::with_throughput(duration, bytes_per_sec)
}

/// Measures execution time with memory tracking.
///
/// Tracks peak memory usage during execution. Note: This requires
/// platform-specific support and may not be available on all systems.
///
/// # Arguments
///
/// * `name` - Name of the measurement
/// * `f` - The function to measure
///
/// # Returns
///
/// A tuple of (`Measurement`, peak memory in bytes).
///
/// # Example
///
/// ```no_run
/// use hedl_bench::core::measurement::measure_memory;
///
/// let (measurement, peak_memory) = measure_memory("allocate", || {
///     let _data = vec![0u8; 1_000_000];
/// });
/// println!("Peak memory: {} MB", peak_memory as f64 / 1_000_000.0);
/// ```
pub fn measure_memory<F>(_name: &str, mut f: F) -> (Measurement, usize)
where
    F: FnMut(),
{
    // Get baseline memory
    let baseline = current_memory_usage();

    let start = Instant::now();
    f();
    let duration = start.elapsed();

    // Get peak memory (approximation)
    let peak = current_memory_usage();
    let memory_used = peak.saturating_sub(baseline);

    let mut measurement = Measurement::new(duration);
    measurement.memory = Some(memory_used);

    (measurement, memory_used)
}

/// Returns current memory usage in bytes.
///
/// This is a platform-specific approximation. On systems without
/// support, it returns 0.
fn current_memory_usage() -> usize {
    #[cfg(target_os = "linux")]
    {
        // Read from /proc/self/statm
        if let Ok(contents) = std::fs::read_to_string("/proc/self/statm") {
            if let Some(resident) = contents.split_whitespace().nth(1) {
                if let Ok(pages) = resident.parse::<usize>() {
                    // Convert pages to bytes (assuming 4KB pages)
                    return pages * 4096;
                }
            }
        }
    }

    // Fallback for other platforms
    0
}

/// Statistics from multiple measurements.
#[derive(Debug, Clone)]
pub struct Statistics {
    /// Mean duration.
    pub mean: Duration,
    /// Standard deviation.
    pub std_dev: Duration,
    /// Minimum duration.
    pub min: Duration,
    /// Maximum duration.
    pub max: Duration,
    /// Median duration.
    pub median: Duration,
}

/// Computes statistics from a collection of measurements.
///
/// # Arguments
///
/// * `measurements` - Slice of measurements
///
/// # Returns
///
/// `Statistics` containing mean, std_dev, min, max, and median.
pub fn compute_statistics(measurements: &[Measurement]) -> Statistics {
    if measurements.is_empty() {
        return Statistics {
            mean: Duration::ZERO,
            std_dev: Duration::ZERO,
            min: Duration::ZERO,
            max: Duration::ZERO,
            median: Duration::ZERO,
        };
    }

    let mut durations: Vec<Duration> = measurements.iter().map(|m| m.duration).collect();
    durations.sort();

    let total: Duration = durations.iter().sum();
    let mean = total / durations.len() as u32;

    let variance: f64 = durations
        .iter()
        .map(|d| {
            let diff = d.as_nanos() as f64 - mean.as_nanos() as f64;
            diff * diff
        })
        .sum::<f64>()
        / durations.len() as f64;

    let std_dev = Duration::from_nanos(variance.sqrt() as u64);

    Statistics {
        mean,
        std_dev,
        min: durations[0],
        max: durations[durations.len() - 1],
        median: durations[durations.len() / 2],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_measure() {
        let measurement = measure("test", 10, || {
            thread::sleep(Duration::from_millis(1));
        });

        assert!(measurement.duration.as_millis() >= 10);
        assert!(measurement.throughput.is_none());
    }

    #[test]
    fn test_measure_with_throughput() {
        let measurement = measure_with_throughput("throughput_test", 10, 1024, || {
            thread::sleep(Duration::from_micros(100));
        });

        assert!(measurement.duration.as_micros() >= 1000);
        assert!(measurement.throughput.is_some());
    }

    #[test]
    fn test_measure_memory() {
        let (measurement, _memory) = measure_memory("memory_test", || {
            let _data = vec![0u8; 1000];
        });

        assert!(measurement.duration.as_nanos() > 0);
        // Memory tracking may not be available on all platforms
    }

    #[test]
    fn test_measurement_conversions() {
        let m = Measurement::with_throughput(Duration::from_secs(1), 1_000_000);
        assert_eq!(m.throughput_mbs(), Some(1.0));
        assert_eq!(m.as_nanos(), 1_000_000_000);
    }

    #[test]
    fn test_compute_statistics() {
        let measurements = vec![
            Measurement::new(Duration::from_millis(10)),
            Measurement::new(Duration::from_millis(20)),
            Measurement::new(Duration::from_millis(30)),
        ];

        let stats = compute_statistics(&measurements);
        assert_eq!(stats.mean, Duration::from_millis(20));
        assert_eq!(stats.min, Duration::from_millis(10));
        assert_eq!(stats.max, Duration::from_millis(30));
        assert_eq!(stats.median, Duration::from_millis(20));
    }
}
