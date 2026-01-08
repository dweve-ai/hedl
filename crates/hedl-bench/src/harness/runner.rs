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

//! Benchmark runner for executing and collecting results.

use crate::core::config::BenchConfig;
use crate::core::Measurement;
use crate::report::{BenchmarkReport, PerfResult};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A benchmark function that takes a dataset size and returns a result.
pub type BenchFn<T> = Box<dyn Fn(usize) -> T>;

/// Result from a single benchmark run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BenchResult {
    /// Name of the benchmark.
    pub name: String,
    /// Number of iterations performed.
    pub iterations: u64,
    /// Measurement data.
    pub measurement: Measurement,
    /// Dataset size (if applicable).
    pub size: Option<usize>,
}

impl BenchResult {
    /// Creates a new benchmark result.
    pub fn new(name: impl Into<String>, iterations: u64, measurement: Measurement) -> Self {
        Self {
            name: name.into(),
            iterations,
            measurement,
            size: None,
        }
    }

    /// Sets the dataset size.
    pub fn with_size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    /// Returns the average duration per iteration.
    pub fn avg_duration(&self) -> Duration {
        self.measurement.duration / self.iterations.max(1) as u32
    }

    /// Returns throughput in MB/s if available.
    pub fn throughput_mbs(&self) -> Option<f64> {
        self.measurement.throughput_mbs()
    }
}

/// Results from a scaling benchmark run.
#[derive(Debug, Clone)]
pub struct ScalingResults {
    pub name: String,
    pub results: HashMap<usize, PerfResult>,
}

/// Runner for executing benchmarks with standardized configuration.
pub struct BenchmarkRunner {
    config: BenchConfig,
    benchmarks: Vec<(String, BenchFn<()>)>,
    results: Vec<ScalingResults>,
}

impl BenchmarkRunner {
    /// Creates a new benchmark runner with the specified configuration.
    pub fn new(config: BenchConfig) -> Self {
        Self {
            config,
            benchmarks: Vec::new(),
            results: Vec::new(),
        }
    }

    /// Registers a scaling benchmark that runs across all configured sizes.
    pub fn register_scaling<F>(&mut self, name: &str, bench_fn: F)
    where
        F: Fn(usize) + 'static,
    {
        self.benchmarks.push((name.to_string(), Box::new(bench_fn)));
    }

    /// Runs all registered benchmarks.
    pub fn run_all(&mut self) -> Vec<ScalingResults> {
        for (name, bench_fn) in &self.benchmarks {
            let mut results = HashMap::new();

            for &size in &self.config.sizes {
                let iterations = self.config.get_iterations(size);

                // Collect timing data manually for reporting
                let mut total_ns = 0u64;
                for _ in 0..iterations {
                    let start = Instant::now();
                    bench_fn(size);
                    total_ns += start.elapsed().as_nanos() as u64;
                }

                results.insert(
                    size,
                    PerfResult {
                        name: format!("{}_{}", name, size),
                        iterations,
                        total_time_ns: total_ns,
                        throughput_bytes: None,
                        avg_time_ns: Some(total_ns / iterations.max(1)),
                        throughput_mbs: None,
                    },
                );
            }

            self.results.push(ScalingResults {
                name: name.clone(),
                results,
            });
        }

        self.results.clone()
    }

    /// Generates a report from collected results.
    pub fn generate_report(&self, title: &str) -> BenchmarkReport {
        let mut report = BenchmarkReport::new(title);
        report.set_timestamp();

        for scaling_result in &self.results {
            for perf_result in scaling_result.results.values() {
                report.add_perf(perf_result.clone());
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let config = BenchConfig::default();
        let runner = BenchmarkRunner::new(config);
        assert_eq!(runner.benchmarks.len(), 0);
    }

    #[test]
    fn test_register_benchmark() {
        let config = BenchConfig::default();
        let mut runner = BenchmarkRunner::new(config);

        runner.register_scaling("test", |_size| {
            // Test benchmark
        });

        assert_eq!(runner.benchmarks.len(), 1);
    }
}
