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

//! Metric collection for benchmarks.
//!
//! Collects performance, memory, and throughput metrics from benchmark runs.

use crate::core::Measurement;
use crate::harness::runner::BenchResult;
use std::collections::HashMap;

/// Performance metrics collected from measurements.
#[derive(Debug, Clone)]
pub struct PerfMetrics {
    /// Average duration in nanoseconds.
    pub avg_ns: u64,
    /// Total duration in nanoseconds.
    pub total_ns: u64,
    /// Operations per second.
    pub ops_per_sec: f64,
}

/// Memory metrics collected from measurements.
#[derive(Debug, Clone)]
pub struct MemMetrics {
    /// Peak memory usage in bytes.
    pub peak_bytes: usize,
    /// Average memory usage in bytes.
    pub avg_bytes: usize,
    /// Memory usage in MB.
    pub mb: f64,
}

/// Throughput metrics collected from measurements.
#[derive(Debug, Clone)]
pub struct ThroughputMetrics {
    /// Bytes per second.
    pub bytes_per_sec: u64,
    /// Megabytes per second.
    pub mbs: f64,
}

/// Collects and aggregates metrics from benchmark runs.
pub struct MetricCollector {
    perf_metrics: HashMap<String, Vec<PerfMetrics>>,
    mem_metrics: HashMap<String, Vec<MemMetrics>>,
    throughput_metrics: HashMap<String, Vec<ThroughputMetrics>>,
}

impl MetricCollector {
    /// Creates a new metric collector.
    pub fn new() -> Self {
        Self {
            perf_metrics: HashMap::new(),
            mem_metrics: HashMap::new(),
            throughput_metrics: HashMap::new(),
        }
    }

    /// Collects metrics from a benchmark result.
    pub fn collect(&mut self, result: &BenchResult) {
        let perf = collect_performance(&result.measurement);
        self.perf_metrics
            .entry(result.name.clone())
            .or_default()
            .push(perf);

        if result.measurement.memory.is_some() {
            let mem = collect_memory(&result.measurement);
            self.mem_metrics
                .entry(result.name.clone())
                .or_default()
                .push(mem);
        }

        if result.measurement.throughput.is_some() {
            let throughput = collect_throughput(&result.measurement);
            self.throughput_metrics
                .entry(result.name.clone())
                .or_default()
                .push(throughput);
        }
    }

    /// Gets performance metrics for a benchmark.
    pub fn get_perf_metrics(&self, name: &str) -> Option<&Vec<PerfMetrics>> {
        self.perf_metrics.get(name)
    }

    /// Gets memory metrics for a benchmark.
    pub fn get_mem_metrics(&self, name: &str) -> Option<&Vec<MemMetrics>> {
        self.mem_metrics.get(name)
    }

    /// Gets throughput metrics for a benchmark.
    pub fn get_throughput_metrics(&self, name: &str) -> Option<&Vec<ThroughputMetrics>> {
        self.throughput_metrics.get(name)
    }
}

impl Default for MetricCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Collects performance metrics from a measurement.
pub fn collect_performance(measurement: &Measurement) -> PerfMetrics {
    let total_ns = measurement.as_nanos();
    let ops_per_sec = if total_ns > 0 {
        1_000_000_000.0 / total_ns as f64
    } else {
        0.0
    };

    PerfMetrics {
        avg_ns: total_ns,
        total_ns,
        ops_per_sec,
    }
}

/// Collects memory metrics from a measurement.
pub fn collect_memory(measurement: &Measurement) -> MemMetrics {
    let bytes = measurement.memory.unwrap_or(0);
    MemMetrics {
        peak_bytes: bytes,
        avg_bytes: bytes,
        mb: bytes as f64 / 1_000_000.0,
    }
}

/// Collects throughput metrics from a measurement.
pub fn collect_throughput(measurement: &Measurement) -> ThroughputMetrics {
    let bytes_per_sec = measurement.throughput.unwrap_or(0);
    ThroughputMetrics {
        bytes_per_sec,
        mbs: bytes_per_sec as f64 / 1_000_000.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_collect_performance() {
        let measurement = Measurement::new(Duration::from_millis(100));
        let perf = collect_performance(&measurement);

        assert_eq!(perf.total_ns, 100_000_000);
        assert!(perf.ops_per_sec > 0.0);
    }

    #[test]
    fn test_collect_memory() {
        let mut measurement = Measurement::new(Duration::from_millis(1));
        measurement.memory = Some(1_000_000);

        let mem = collect_memory(&measurement);
        assert_eq!(mem.peak_bytes, 1_000_000);
        assert_eq!(mem.mb, 1.0);
    }

    #[test]
    fn test_collect_throughput() {
        let measurement = Measurement::with_throughput(Duration::from_secs(1), 1_000_000);
        let throughput = collect_throughput(&measurement);

        assert_eq!(throughput.bytes_per_sec, 1_000_000);
        assert_eq!(throughput.mbs, 1.0);
    }

    #[test]
    fn test_metric_collector() {
        let mut collector = MetricCollector::new();
        let measurement = Measurement::new(Duration::from_millis(100));
        let result = BenchResult::new("test", 10, measurement);

        collector.collect(&result);

        assert!(collector.get_perf_metrics("test").is_some());
        assert_eq!(collector.get_perf_metrics("test").unwrap().len(), 1);
    }
}
