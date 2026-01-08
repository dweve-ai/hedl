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

//! Common utilities for LSP, MCP, and linting tool benchmarks.

use std::time::Duration;

/// Latency metrics for tool operations
#[derive(Debug, Clone)]
pub struct LatencyMetrics {
    pub min_ns: u64,
    pub max_ns: u64,
    pub mean_ns: u64,
    pub p50_ns: u64,
    pub p95_ns: u64,
    pub p99_ns: u64,
    pub total_ns: u64,
    pub iterations: u64,
}

impl LatencyMetrics {
    pub fn from_samples(mut samples: Vec<u64>) -> Self {
        samples.sort_unstable();
        let len = samples.len();
        let total_ns: u64 = samples.iter().sum();
        let mean_ns = if len > 0 { total_ns / len as u64 } else { 0 };

        let min_ns = samples.first().copied().unwrap_or(0);
        let max_ns = samples.last().copied().unwrap_or(0);

        let p50_ns = percentile(&samples, 50.0);
        let p95_ns = percentile(&samples, 95.0);
        let p99_ns = percentile(&samples, 99.0);

        LatencyMetrics {
            min_ns,
            max_ns,
            mean_ns,
            p50_ns,
            p95_ns,
            p99_ns,
            total_ns,
            iterations: len as u64,
        }
    }
}

/// Calculate percentile from sorted samples
fn percentile(sorted_samples: &[u64], p: f64) -> u64 {
    if sorted_samples.is_empty() {
        return 0;
    }

    let index = (p / 100.0 * (sorted_samples.len() as f64 - 1.0)).round() as usize;
    sorted_samples[index.min(sorted_samples.len() - 1)]
}

/// Measure tool latency with multiple samples
pub fn measure_tool_latency<F>(name: &str, iterations: u64, f: F) -> LatencyMetrics
where
    F: Fn(),
{
    let mut samples = Vec::with_capacity(iterations as usize);

    for _ in 0..iterations {
        let start = std::time::Instant::now();
        f();
        samples.push(start.elapsed().as_nanos() as u64);
    }

    LatencyMetrics::from_samples(samples)
}

/// Overhead statistics for protocol/tool operations
#[derive(Debug, Clone)]
pub struct ProtocolOverhead {
    pub operation_time_ns: u64,
    pub pure_work_time_ns: u64,
    pub protocol_overhead_ns: u64,
    pub overhead_percent: f64,
}

/// Measure protocol overhead (e.g., JSON-RPC serialization/deserialization)
pub fn measure_protocol_overhead(
    operation_time_ns: u64,
    pure_work_time_ns: u64,
) -> ProtocolOverhead {
    let protocol_overhead_ns = operation_time_ns.saturating_sub(pure_work_time_ns);
    let overhead_percent = if pure_work_time_ns > 0 {
        (protocol_overhead_ns as f64 / pure_work_time_ns as f64) * 100.0
    } else {
        0.0
    };

    ProtocolOverhead {
        operation_time_ns,
        pure_work_time_ns,
        protocol_overhead_ns,
        overhead_percent,
    }
}
