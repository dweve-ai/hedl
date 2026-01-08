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

//! Common utilities for FFI and bindings benchmarks.

use std::time::Duration;

/// Overhead analysis results comparing binding performance to native Rust
#[derive(Debug, Clone)]
pub struct OverheadAnalysis {
    pub binding_time_ns: u64,
    pub native_time_ns: u64,
    pub overhead_ns: u64,
    pub overhead_percent: f64,
}

/// Comparison between binding and native performance
#[derive(Debug, Clone)]
pub struct Comparison {
    pub binding_time: Duration,
    pub native_time: Duration,
    pub overhead: Duration,
    pub overhead_percent: f64,
    pub throughput_degradation: Option<f64>,
}

/// Measure FFI overhead by comparing binding execution to native execution
pub fn measure_ffi_overhead<F>(name: &str, iterations: u64, f: F) -> OverheadAnalysis
where
    F: Fn() -> (u64, u64), // Returns (binding_time_ns, native_time_ns)
{
    let mut total_binding_ns = 0u64;
    let mut total_native_ns = 0u64;

    for _ in 0..iterations {
        let (binding_ns, native_ns) = f();
        total_binding_ns += binding_ns;
        total_native_ns += native_ns;
    }

    let overhead_ns = total_binding_ns.saturating_sub(total_native_ns);
    let overhead_percent = if total_native_ns > 0 {
        (overhead_ns as f64 / total_native_ns as f64) * 100.0
    } else {
        0.0
    };

    OverheadAnalysis {
        binding_time_ns: total_binding_ns,
        native_time_ns: total_native_ns,
        overhead_ns,
        overhead_percent,
    }
}

/// Compare binding performance to native performance
pub fn compare_to_native(binding_time: Duration, native_time: Duration) -> Comparison {
    let overhead = binding_time.saturating_sub(native_time);
    let overhead_percent = if native_time.as_nanos() > 0 {
        (overhead.as_nanos() as f64 / native_time.as_nanos() as f64) * 100.0
    } else {
        0.0
    };

    Comparison {
        binding_time,
        native_time,
        overhead,
        overhead_percent,
        throughput_degradation: None,
    }
}

/// Calculate throughput degradation percentage
pub fn throughput_degradation(binding_throughput_mbs: f64, native_throughput_mbs: f64) -> f64 {
    if native_throughput_mbs > 0.0 {
        ((native_throughput_mbs - binding_throughput_mbs) / native_throughput_mbs) * 100.0
    } else {
        0.0
    }
}
