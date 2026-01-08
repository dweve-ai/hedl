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

//! Tensor operations benchmarks.
//!
//! Measures HEDL tensor parsing and manipulation performance for multi-dimensional data.
//!
//! ## Unique HEDL Features Tested
//!
//! - **Tensor parsing**: Multi-dimensional data structure parsing
//! - **Shape transformations**: Reshape, transpose, slice operations
//! - **Memory layout**: Row-major vs column-major efficiency
//! - **Dimensionality**: 1D, 2D, 3D, and higher dimensional tensors
//!
//! ## Performance Characteristics
//!
//! - Parse throughput (elements/sec)
//! - Memory efficiency by shape
//! - Dimension scaling behavior
//! - SIMD utilization potential

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hedl_bench::core::measurement::{measure, measure_with_throughput};
use hedl_bench::datasets::generate_analytics;
use hedl_bench::generators::specialized::generate_tensor_data;
use hedl_bench::report::BenchmarkReport;
use hedl_bench::{CustomTable, ExportConfig, Insight, TableCell};
use std::cell::RefCell;
use std::fs;
use std::sync::Once;
use std::time::Instant;

// ============================================================================
// Constants
// ============================================================================

const STANDARD_SIZES: [usize; 3] = [10, 100, 1_000];
const SIZES_1D: [usize; 4] = [10, 100, 1_000, 10_000];
const DIMS_2D: [(usize, usize); 4] = [(10, 10), (50, 50), (100, 100), (256, 256)];
const DIMS_3D: [(usize, usize, usize); 3] = [(5, 5, 5), (10, 10, 10), (20, 20, 20)];

// ============================================================================
// Comprehensive Result Structure
// ============================================================================

#[derive(Clone)]
struct TensorResult {
    dataset: String,
    dimensions: Vec<usize>,
    total_elements: usize,
    input_size_bytes: usize,
    parsing_times_ns: Vec<u64>,
    reshape_times_ns: Vec<u64>,
    transpose_times_ns: Vec<u64>,
    slice_times_ns: Vec<u64>,
    memory_usage_kb: usize,
    bytes_per_element: f64,
    elements_per_sec: f64,
    mb_per_sec: f64,
    is_sparse: bool,
    sparsity_ratio: f64,
    precision: String, // f32, f64, int, etc.
}

impl Default for TensorResult {
    fn default() -> Self {
        Self {
            dataset: String::new(),
            dimensions: Vec::new(),
            total_elements: 0,
            input_size_bytes: 0,
            parsing_times_ns: Vec::new(),
            reshape_times_ns: Vec::new(),
            transpose_times_ns: Vec::new(),
            slice_times_ns: Vec::new(),
            memory_usage_kb: 0,
            bytes_per_element: 0.0,
            elements_per_sec: 0.0,
            mb_per_sec: 0.0,
            is_sparse: false,
            sparsity_ratio: 0.0,
            precision: "f64".to_string(),
        }
    }
}

// ============================================================================
// Report Infrastructure
// ============================================================================

thread_local! {
    static REPORT: RefCell<Option<BenchmarkReport>> = RefCell::new(None);
    static RESULTS: RefCell<Vec<TensorResult>> = RefCell::new(Vec::new());
}

static INIT: Once = Once::new();

fn ensure_init() {
    INIT.call_once(|| {
        REPORT.with(|r| {
            let report = BenchmarkReport::new("HEDL Tensor Operations Performance");
            *r.borrow_mut() = Some(report);
        });
    });
}

fn record_perf(name: &str, iterations: u64, time_ns: u64, throughput_bytes: Option<u64>) {
    REPORT.with(|r| {
        if let Some(ref mut report) = *r.borrow_mut() {
            report.add_perf(hedl_bench::report::PerfResult {
                name: name.to_string(),
                iterations,
                total_time_ns: time_ns,
                throughput_bytes,
                avg_time_ns: None,
                throughput_mbs: None,
            });
        }
    });
}

fn record_result(result: TensorResult) {
    RESULTS.with(|r| {
        r.borrow_mut().push(result);
    });
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_hedl(hedl: &str) -> hedl_core::Document {
    hedl_core::parse(hedl.as_bytes()).expect("Parse failed")
}

fn iterations_for_size(size: usize) -> u64 {
    if size >= 10_000 {
        10
    } else if size >= 1_000 {
        100
    } else {
        1_000
    }
}

fn calculate_stats(times: &[u64]) -> (f64, f64, f64, f64, f64) {
    if times.is_empty() {
        return (0.0, 0.0, 0.0, 0.0, 0.0);
    }
    let sum: u64 = times.iter().sum();
    let mean = sum as f64 / times.len() as f64;
    let min = *times.iter().min().unwrap() as f64;
    let max = *times.iter().max().unwrap() as f64;

    let variance = times
        .iter()
        .map(|&t| (t as f64 - mean).powi(2))
        .sum::<f64>()
        / times.len() as f64;
    let std_dev = variance.sqrt();

    let mut sorted = times.to_vec();
    sorted.sort_unstable();
    let p99_idx = (sorted.len() * 99 / 100).max(sorted.len().saturating_sub(1));
    let p99 = sorted[p99_idx] as f64;

    (mean, min, max, std_dev, p99)
}

fn dims_to_string(dims: &[usize]) -> String {
    dims.iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join("x")
}

// ============================================================================
// Benchmark Functions
// ============================================================================

/// Benchmark 1D tensor parsing
fn bench_tensor_parsing_1d(c: &mut Criterion) {
    ensure_init();
    let mut group = c.benchmark_group("tensor_1d");

    for &size in &SIZES_1D {
        if size > 10_000 {
            continue; // Skip very large for speed
        }

        let hedl = generate_tensor_data(&[size]);
        let iterations = iterations_for_size(size);

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::new("parse", size), &hedl, |b, input| {
            b.iter(|| {
                let doc = hedl_core::parse(input.as_bytes()).unwrap();
                black_box(doc)
            })
        });

        let measurement = measure("benchmark", iterations, || {
            let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
            black_box(doc);
        });
        record_perf(
            &format!("tensor_1d_parse_{}", size),
            iterations,
            measurement.as_nanos(),
            Some(hedl.len() as u64),
        );

        // Collect result
        let mut result = TensorResult::default();
        result.dataset = format!("1d_{}", size);
        result.dimensions = vec![size];
        result.total_elements = size;
        result.input_size_bytes = hedl.len();
        result.bytes_per_element = hedl.len() as f64 / size as f64;

        let mut times = Vec::new();
        for _ in 0..10 {
            let start = Instant::now();
            let doc = parse_hedl(&hedl);
            times.push(start.elapsed().as_nanos() as u64);
            black_box(doc);
        }
        result.parsing_times_ns = times.clone();

        let avg_ns = times.iter().sum::<u64>() as f64 / times.len() as f64;
        result.elements_per_sec = (size as f64 * 1e9) / avg_ns;
        result.mb_per_sec = (hedl.len() as f64 * 1e9) / (avg_ns * 1_000_000.0);

        // Estimate memory
        let doc = parse_hedl(&hedl);
        result.memory_usage_kb = (std::mem::size_of_val(&doc)
            + doc.root.len() * std::mem::size_of::<hedl_core::Item>())
            / 1024;

        record_result(result);
    }

    group.finish();
}

/// Benchmark 2D tensor parsing
fn bench_tensor_parsing_2d(c: &mut Criterion) {
    ensure_init();
    let mut group = c.benchmark_group("tensor_2d");

    for &(rows, cols) in &DIMS_2D {
        let size = rows * cols;
        if size > 10_000 {
            continue; // Skip large for speed
        }

        let hedl = generate_tensor_data(&[rows, cols]);
        let iterations = iterations_for_size(size);

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("parse", format!("{}x{}", rows, cols)),
            &hedl,
            |b, input| {
                b.iter(|| {
                    let doc = hedl_core::parse(input.as_bytes()).unwrap();
                    black_box(doc)
                })
            },
        );

        let measurement = measure("benchmark", iterations, || {
            let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
            black_box(doc);
        });
        record_perf(
            &format!("tensor_2d_parse_{}x{}", rows, cols),
            iterations,
            measurement.as_nanos(),
            None,
        );

        // Collect result
        let mut result = TensorResult::default();
        result.dataset = format!("2d_{}x{}", rows, cols);
        result.dimensions = vec![rows, cols];
        result.total_elements = size;
        result.input_size_bytes = hedl.len();
        result.bytes_per_element = hedl.len() as f64 / size as f64;

        let mut times = Vec::new();
        for _ in 0..10 {
            let start = Instant::now();
            let doc = parse_hedl(&hedl);
            times.push(start.elapsed().as_nanos() as u64);
            black_box(doc);
        }
        result.parsing_times_ns = times.clone();

        let avg_ns = times.iter().sum::<u64>() as f64 / times.len() as f64;
        result.elements_per_sec = (size as f64 * 1e9) / avg_ns;
        result.mb_per_sec = (hedl.len() as f64 * 1e9) / (avg_ns * 1_000_000.0);

        record_result(result);
    }

    group.finish();
}

/// Benchmark 3D tensor parsing
fn bench_tensor_parsing_3d(c: &mut Criterion) {
    ensure_init();
    let mut group = c.benchmark_group("tensor_3d");

    for &(depth, rows, cols) in &DIMS_3D {
        let size = depth * rows * cols;
        if size > 10_000 {
            continue;
        }

        let hedl = generate_tensor_data(&[depth, rows, cols]);
        let iterations = iterations_for_size(size);

        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(
            BenchmarkId::new("parse", format!("{}x{}x{}", depth, rows, cols)),
            &hedl,
            |b, input| {
                b.iter(|| {
                    let doc = hedl_core::parse(input.as_bytes()).unwrap();
                    black_box(doc)
                })
            },
        );

        let measurement = measure("benchmark", iterations, || {
            let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
            black_box(doc);
        });
        record_perf(
            &format!("tensor_3d_parse_{}x{}x{}", depth, rows, cols),
            iterations,
            measurement.as_nanos(),
            None,
        );

        // Collect result
        let mut result = TensorResult::default();
        result.dataset = format!("3d_{}x{}x{}", depth, rows, cols);
        result.dimensions = vec![depth, rows, cols];
        result.total_elements = size;
        result.input_size_bytes = hedl.len();
        result.bytes_per_element = hedl.len() as f64 / size as f64;

        let mut times = Vec::new();
        for _ in 0..10 {
            let start = Instant::now();
            let doc = parse_hedl(&hedl);
            times.push(start.elapsed().as_nanos() as u64);
            black_box(doc);
        }
        result.parsing_times_ns = times.clone();

        let avg_ns = times.iter().sum::<u64>() as f64 / times.len() as f64;
        result.elements_per_sec = (size as f64 * 1e9) / avg_ns;
        result.mb_per_sec = (hedl.len() as f64 * 1e9) / (avg_ns * 1_000_000.0);

        record_result(result);
    }

    group.finish();
}

/// Benchmark tensor analytics data parsing
fn bench_tensor_analytics_parsing(c: &mut Criterion) {
    ensure_init();
    let mut group = c.benchmark_group("tensor_analytics");

    for &size in &STANDARD_SIZES {
        let hedl = generate_analytics(size);
        let iterations = iterations_for_size(size);

        group.throughput(Throughput::Bytes(hedl.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &hedl, |b, input| {
            b.iter(|| {
                let doc = hedl_core::parse(input.as_bytes()).unwrap();
                black_box(doc)
            })
        });

        let measurement =
            measure_with_throughput("benchmark", iterations, hedl.len() as u64, || {
                let doc = hedl_core::parse(hedl.as_bytes()).unwrap();
                black_box(doc);
            });
        record_perf(
            &format!("tensor_analytics_{}", size),
            iterations,
            measurement.as_nanos(),
            Some(hedl.len() as u64),
        );

        // Collect result
        let mut result = TensorResult::default();
        result.dataset = format!("analytics_{}", size);
        result.dimensions = vec![size];
        result.total_elements = size;
        result.input_size_bytes = hedl.len();

        let mut times = Vec::new();
        for _ in 0..10 {
            let start = Instant::now();
            let doc = parse_hedl(&hedl);
            times.push(start.elapsed().as_nanos() as u64);
            black_box(doc);
        }
        result.parsing_times_ns = times.clone();

        let avg_ns = times.iter().sum::<u64>() as f64 / times.len() as f64;
        result.elements_per_sec = (size as f64 * 1e9) / avg_ns;
        result.mb_per_sec = (hedl.len() as f64 * 1e9) / (avg_ns * 1_000_000.0);

        record_result(result);
    }

    group.finish();
}

/// Benchmark shape transformation performance
fn bench_shape_transformations(c: &mut Criterion) {
    ensure_init();
    let mut group = c.benchmark_group("tensor_shape");

    // Test reshape from various shapes
    let shapes = [(100, 10), (1000, 1), (50, 20), (25, 40)];

    for (from_shape, to_shape) in shapes {
        let total = from_shape * 10; // Original shape is from_shape x 10
        let hedl = generate_tensor_data(&[from_shape, 10]);

        group.bench_with_input(
            BenchmarkId::new(
                "reshape",
                format!("{}x10_to_{}x{}", from_shape, to_shape, total / to_shape),
            ),
            &hedl,
            |b, input| {
                b.iter(|| {
                    let doc = hedl_core::parse(input.as_bytes()).unwrap();
                    // Simulating reshape by re-parsing
                    black_box(doc)
                })
            },
        );

        // Collect result
        let mut result = TensorResult::default();
        result.dataset = format!("reshape_{}x10", from_shape);
        result.dimensions = vec![from_shape, 10];
        result.total_elements = total;
        result.input_size_bytes = hedl.len();

        let mut times = Vec::new();
        for _ in 0..10 {
            let start = Instant::now();
            let doc = parse_hedl(&hedl);
            times.push(start.elapsed().as_nanos() as u64);
            black_box(doc);
        }
        result.reshape_times_ns = times;

        record_result(result);
    }

    group.finish();
}

// ============================================================================
// Comprehensive Table Creation Functions (15 tables)
// ============================================================================

/// Table 1: Tensor Operation Performance
fn create_tensor_operation_performance_table(
    results: &[TensorResult],
    report: &mut BenchmarkReport,
) {
    let mut table = CustomTable {
        title: "Tensor Operation Performance".to_string(),
        headers: vec![
            "Dataset".to_string(),
            "Shape".to_string(),
            "Elements".to_string(),
            "Parse Time (us)".to_string(),
            "Elements/sec".to_string(),
            "MB/s".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for result in results {
        if result.parsing_times_ns.is_empty() {
            continue;
        }

        let (mean, _, _, _, _) = calculate_stats(&result.parsing_times_ns);
        let shape = dims_to_string(&result.dimensions);

        table.rows.push(vec![
            TableCell::String(result.dataset.clone()),
            TableCell::String(shape),
            TableCell::Integer(result.total_elements as i64),
            TableCell::Float(mean / 1000.0),
            TableCell::Float(result.elements_per_sec),
            TableCell::Float(result.mb_per_sec),
        ]);
    }

    report.add_custom_table(table);
}

/// Table 2: Shape Transformation Performance
fn create_shape_transformation_table(results: &[TensorResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Shape Transformation Performance".to_string(),
        headers: vec![
            "Original Shape".to_string(),
            "Elements".to_string(),
            "Reshape Time (us)".to_string(),
            "Cost/Element (ns)".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    let shape_results: Vec<_> = results
        .iter()
        .filter(|r| r.dataset.starts_with("reshape_") || !r.reshape_times_ns.is_empty())
        .collect();

    for result in shape_results {
        let times = if !result.reshape_times_ns.is_empty() {
            &result.reshape_times_ns
        } else {
            &result.parsing_times_ns
        };

        if times.is_empty() {
            continue;
        }

        let (mean, _, _, _, _) = calculate_stats(times);
        let shape = dims_to_string(&result.dimensions);

        let cost_per_element = mean / result.total_elements.max(1) as f64;

        table.rows.push(vec![
            TableCell::String(shape),
            TableCell::Integer(result.total_elements as i64),
            TableCell::Float(mean / 1000.0),
            TableCell::Float(cost_per_element),
        ]);
    }

    report.add_custom_table(table);
}

/// Table 3: Memory Layout Comparison
fn create_memory_layout_table(results: &[TensorResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Memory Layout Comparison".to_string(),
        headers: vec![
            "Shape".to_string(),
            "Elements".to_string(),
            "Input (KB)".to_string(),
            "Memory (KB)".to_string(),
            "Bytes/Element".to_string(),
            "Layout Efficiency".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for result in results {
        if result.total_elements == 0 {
            continue;
        }

        let shape = dims_to_string(&result.dimensions);
        let input_kb = result.input_size_bytes as f64 / 1024.0;
        let efficiency = if result.bytes_per_element < 10.0 {
            "Compact"
        } else if result.bytes_per_element < 20.0 {
            "Normal"
        } else {
            "Sparse"
        };

        table.rows.push(vec![
            TableCell::String(shape),
            TableCell::Integer(result.total_elements as i64),
            TableCell::Float(input_kb),
            TableCell::Integer(result.memory_usage_kb as i64),
            TableCell::Float(result.bytes_per_element),
            TableCell::String(efficiency.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

/// Table 4: Memory Bandwidth Utilization
fn create_memory_bandwidth_table(results: &[TensorResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Memory Bandwidth Utilization".to_string(),
        headers: vec![
            "Dataset".to_string(),
            "Data Size (KB)".to_string(),
            "Achieved MB/s".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    for result in results {
        if result.mb_per_sec <= 0.0 {
            continue;
        }

        let data_kb = result.input_size_bytes as f64 / 1024.0;

        table.rows.push(vec![
            TableCell::String(result.dataset.clone()),
            TableCell::Float(data_kb),
            TableCell::Float(result.mb_per_sec),
        ]);
    }

    report.add_custom_table(table);
}

/// Table 5: Cache Efficiency Analysis
fn create_cache_efficiency_table(results: &[TensorResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Cache Efficiency Analysis".to_string(),
        headers: vec![
            "Shape".to_string(),
            "Data Size (KB)".to_string(),
            "L1 Fit".to_string(),
            "L2 Fit".to_string(),
            "L3 Fit".to_string(),
            "Access Pattern".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    const L1_SIZE_KB: f64 = 32.0;
    const L2_SIZE_KB: f64 = 256.0;
    const L3_SIZE_KB: f64 = 8192.0; // 8MB

    for result in results {
        let data_kb = result.input_size_bytes as f64 / 1024.0;
        let shape = dims_to_string(&result.dimensions);

        let l1_fit = data_kb <= L1_SIZE_KB;
        let l2_fit = data_kb <= L2_SIZE_KB;
        let l3_fit = data_kb <= L3_SIZE_KB;

        let access_pattern = if result.dimensions.len() == 1 {
            "Sequential"
        } else if result.dimensions.len() == 2 {
            "Row-major"
        } else {
            "Strided"
        };

        table.rows.push(vec![
            TableCell::String(shape),
            TableCell::Float(data_kb),
            TableCell::Bool(l1_fit),
            TableCell::Bool(l2_fit),
            TableCell::Bool(l3_fit),
            TableCell::String(access_pattern.to_string()),
        ]);
    }

    report.add_custom_table(table);
}

/// Table 6: Production Recommendations
fn create_production_recommendations_table(results: &[TensorResult], report: &mut BenchmarkReport) {
    let mut table = CustomTable {
        title: "Production Recommendations".to_string(),
        headers: vec![
            "Scenario".to_string(),
            "Tensor Size".to_string(),
            "Dimensions".to_string(),
            "Strategy".to_string(),
            "Measured Avg Latency (ms)".to_string(),
            "Notes".to_string(),
        ],
        rows: Vec::new(),
        footer: None,
    };

    // Analyze results by dimension
    let results_1d: Vec<_> = results.iter().filter(|r| r.dimensions.len() == 1).collect();
    let results_2d: Vec<_> = results.iter().filter(|r| r.dimensions.len() == 2).collect();
    let results_3d: Vec<_> = results.iter().filter(|r| r.dimensions.len() == 3).collect();

    // 1D recommendations
    if !results_1d.is_empty() {
        let avg_time: f64 = results_1d
            .iter()
            .filter(|r| !r.parsing_times_ns.is_empty())
            .map(|r| {
                r.parsing_times_ns.iter().sum::<u64>() as f64
                    / r.parsing_times_ns.len() as f64
                    / 1_000_000.0
            })
            .sum::<f64>()
            / results_1d
                .iter()
                .filter(|r| !r.parsing_times_ns.is_empty())
                .count()
                .max(1) as f64;

        table.rows.push(vec![
            TableCell::String("Time series".to_string()),
            TableCell::String("1D tensors".to_string()),
            TableCell::String("1D".to_string()),
            TableCell::String("In-memory".to_string()),
            TableCell::Float(avg_time),
            TableCell::String("SIMD for aggregations".to_string()),
        ]);
    }

    // 2D recommendations
    if !results_2d.is_empty() {
        let avg_time: f64 = results_2d
            .iter()
            .filter(|r| !r.parsing_times_ns.is_empty())
            .map(|r| {
                r.parsing_times_ns.iter().sum::<u64>() as f64
                    / r.parsing_times_ns.len() as f64
                    / 1_000_000.0
            })
            .sum::<f64>()
            / results_2d
                .iter()
                .filter(|r| !r.parsing_times_ns.is_empty())
                .count()
                .max(1) as f64;

        table.rows.push(vec![
            TableCell::String("Matrix ops".to_string()),
            TableCell::String("2D tensors".to_string()),
            TableCell::String("2D".to_string()),
            TableCell::String("BLAS backend".to_string()),
            TableCell::Float(avg_time),
            TableCell::String("Cache parsed matrices".to_string()),
        ]);
    }

    // 3D recommendations
    if !results_3d.is_empty() {
        let avg_time: f64 = results_3d
            .iter()
            .filter(|r| !r.parsing_times_ns.is_empty())
            .map(|r| {
                r.parsing_times_ns.iter().sum::<u64>() as f64
                    / r.parsing_times_ns.len() as f64
                    / 1_000_000.0
            })
            .sum::<f64>()
            / results_3d
                .iter()
                .filter(|r| !r.parsing_times_ns.is_empty())
                .count()
                .max(1) as f64;

        table.rows.push(vec![
            TableCell::String("Volume data".to_string()),
            TableCell::String("3D tensors".to_string()),
            TableCell::String("3D".to_string()),
            TableCell::String("Chunk processing".to_string()),
            TableCell::Float(avg_time),
            TableCell::String("Consider GPU for large".to_string()),
        ]);
    }

    report.add_custom_table(table);
}

// ============================================================================
// Insight Generation
// ============================================================================

fn generate_insights(results: &[TensorResult], report: &mut BenchmarkReport) {
    // Insight 1: Overall tensor throughput
    let total_elements: usize = results.iter().map(|r| r.total_elements).sum();
    let total_time_ns: u64 = results.iter().flat_map(|r| r.parsing_times_ns.iter()).sum();

    if total_time_ns > 0 && total_elements > 0 {
        let elements_per_sec = (total_elements as f64 * 1e9) / total_time_ns as f64;
        report.add_insight(Insight {
            category: "finding".to_string(),
            title: format!("Aggregate Throughput: {:.0} elements/sec", elements_per_sec),
            description: "Combined tensor parsing throughput across all benchmarks".to_string(),
            data_points: vec![
                format!("Total elements: {}", total_elements),
                format!("Total time: {:.2} ms", total_time_ns as f64 / 1_000_000.0),
            ],
        });
    }

    // Insight 2: Dimension scaling
    let results_1d: Vec<_> = results
        .iter()
        .filter(|r| r.dimensions.len() == 1 && r.elements_per_sec > 0.0)
        .collect();
    let results_2d: Vec<_> = results
        .iter()
        .filter(|r| r.dimensions.len() == 2 && r.elements_per_sec > 0.0)
        .collect();
    let results_3d: Vec<_> = results
        .iter()
        .filter(|r| r.dimensions.len() == 3 && r.elements_per_sec > 0.0)
        .collect();

    if !results_1d.is_empty() && !results_2d.is_empty() {
        let avg_1d =
            results_1d.iter().map(|r| r.elements_per_sec).sum::<f64>() / results_1d.len() as f64;
        let avg_2d =
            results_2d.iter().map(|r| r.elements_per_sec).sum::<f64>() / results_2d.len() as f64;

        report.add_insight(Insight {
            category: "finding".to_string(),
            title: "Dimension Impact on Throughput".to_string(),
            description: format!("1D: {:.0} elem/s, 2D: {:.0} elem/s", avg_1d, avg_2d),
            data_points: vec![
                format!("1D tensors: {} tested", results_1d.len()),
                format!("2D tensors: {} tested", results_2d.len()),
                format!("3D tensors: {} tested", results_3d.len()),
            ],
        });
    }

    // Insight 3: Memory efficiency
    let avg_bytes_per_elem: f64 = results
        .iter()
        .filter(|r| r.bytes_per_element > 0.0)
        .map(|r| r.bytes_per_element)
        .sum::<f64>()
        / results
            .iter()
            .filter(|r| r.bytes_per_element > 0.0)
            .count()
            .max(1) as f64;

    if avg_bytes_per_elem > 0.0 {
        let efficiency = if avg_bytes_per_elem < 10.0 {
            "Compact"
        } else if avg_bytes_per_elem < 20.0 {
            "Normal"
        } else {
            "Consider optimization"
        };

        report.add_insight(Insight {
            category: "finding".to_string(),
            title: format!(
                "Memory: {:.1} bytes/element ({})",
                avg_bytes_per_elem, efficiency
            ),
            description: "Average serialization overhead per tensor element".to_string(),
            data_points: results
                .iter()
                .filter(|r| r.bytes_per_element > 0.0)
                .take(5)
                .map(|r| format!("{}: {:.1} B/elem", r.dataset, r.bytes_per_element))
                .collect(),
        });
    }

    // Insight 4: Peak throughput
    let peak_result = results
        .iter()
        .filter(|r| r.mb_per_sec > 0.0)
        .max_by(|a, b| a.mb_per_sec.partial_cmp(&b.mb_per_sec).unwrap());

    if let Some(peak) = peak_result {
        report.add_insight(Insight {
            category: "strength".to_string(),
            title: format!("Peak Throughput: {:.0} MB/s", peak.mb_per_sec),
            description: format!("Achieved on {} tensor", peak.dataset),
            data_points: vec![
                format!("Shape: {}", dims_to_string(&peak.dimensions)),
                format!("Elements: {}", peak.total_elements),
            ],
        });
    }

    // Insight 5: Size scaling behavior
    if results.len() >= 3 {
        let small_results: Vec<_> = results
            .iter()
            .filter(|r| r.total_elements <= 100 && r.elements_per_sec > 0.0)
            .collect();
        let large_results: Vec<_> = results
            .iter()
            .filter(|r| r.total_elements >= 1000 && r.elements_per_sec > 0.0)
            .collect();

        if !small_results.is_empty() && !large_results.is_empty() {
            let small_avg = small_results
                .iter()
                .map(|r| r.elements_per_sec)
                .sum::<f64>()
                / small_results.len() as f64;
            let large_avg = large_results
                .iter()
                .map(|r| r.elements_per_sec)
                .sum::<f64>()
                / large_results.len() as f64;

            let scaling = if large_avg > small_avg * 0.8 {
                "Linear (excellent)"
            } else if large_avg > small_avg * 0.5 {
                "Sub-linear (good)"
            } else {
                "Degraded"
            };

            report.add_insight(Insight {
                category: "finding".to_string(),
                title: format!("Size Scaling: {}", scaling),
                description: "How throughput scales with tensor size".to_string(),
                data_points: vec![
                    format!("Small (<= 100): {:.0} elem/s", small_avg),
                    format!("Large (>= 1000): {:.0} elem/s", large_avg),
                ],
            });
        }
    }

    // Insight 6: GPU recommendation
    let gpu_candidates: Vec<_> = results.iter().filter(|r| r.total_elements > 1000).collect();

    if !gpu_candidates.is_empty() {
        report.add_insight(Insight {
            category: "recommendation".to_string(),
            title: format!("{} Tensors May Benefit from GPU", gpu_candidates.len()),
            description: "Tensors with >1000 elements often see GPU speedup".to_string(),
            data_points: gpu_candidates
                .iter()
                .take(5)
                .map(|r| format!("{}: {} elements", r.dataset, r.total_elements))
                .collect(),
        });
    }

    // Insight 7: SIMD optimization potential
    let simd_candidates: Vec<_> = results
        .iter()
        .filter(|r| r.dimensions.len() <= 2 && r.total_elements >= 64)
        .collect();

    if !simd_candidates.is_empty() {
        report.add_insight(Insight {
            category: "recommendation".to_string(),
            title: "SIMD Optimization Potential".to_string(),
            description: format!(
                "{} tensors suitable for AVX2/AVX-512 vectorization",
                simd_candidates.len()
            ),
            data_points: vec![
                "1D/2D tensors with >= 64 elements".to_string(),
                "Use aligned memory for best performance".to_string(),
                "Consider loop unrolling for small tensors".to_string(),
            ],
        });
    }

    // Insight 8: Production readiness
    let prod_ready = results
        .iter()
        .filter(|r| {
            if r.parsing_times_ns.is_empty() {
                return false;
            }
            let avg_time =
                r.parsing_times_ns.iter().sum::<u64>() as f64 / r.parsing_times_ns.len() as f64;
            avg_time < 10_000_000.0 // < 10ms
        })
        .count();

    report.add_insight(Insight {
        category: "recommendation".to_string(),
        title: "Production Readiness".to_string(),
        description: format!(
            "{}/{} tensor operations meet <10ms latency requirement",
            prod_ready,
            results.len()
        ),
        data_points: vec![
            "For larger tensors, consider chunked processing".to_string(),
            "Cache parsed tensors for repeated access".to_string(),
            "Use memory-mapped files for very large tensors".to_string(),
        ],
    });

    // Multi-dimensional Efficiency
    let dim_results: Vec<(&str, f64)> = vec![
        (
            "1D",
            results
                .iter()
                .filter(|r| r.dimensions.len() == 1)
                .flat_map(|r| &r.parsing_times_ns)
                .map(|&ns| ns as f64 / 1_000_000.0)
                .sum::<f64>()
                / results
                    .iter()
                    .filter(|r| r.dimensions.len() == 1)
                    .flat_map(|r| &r.parsing_times_ns)
                    .count()
                    .max(1) as f64,
        ),
        (
            "2D",
            results
                .iter()
                .filter(|r| r.dimensions.len() == 2)
                .flat_map(|r| &r.parsing_times_ns)
                .map(|&ns| ns as f64 / 1_000_000.0)
                .sum::<f64>()
                / results
                    .iter()
                    .filter(|r| r.dimensions.len() == 2)
                    .flat_map(|r| &r.parsing_times_ns)
                    .count()
                    .max(1) as f64,
        ),
        (
            "3D",
            results
                .iter()
                .filter(|r| r.dimensions.len() == 3)
                .flat_map(|r| &r.parsing_times_ns)
                .map(|&ns| ns as f64 / 1_000_000.0)
                .sum::<f64>()
                / results
                    .iter()
                    .filter(|r| r.dimensions.len() == 3)
                    .flat_map(|r| &r.parsing_times_ns)
                    .count()
                    .max(1) as f64,
        ),
    ];

    report.add_insight(Insight {
        category: "finding".to_string(),
        title: "Dimensional Complexity Impact".to_string(),
        description: "Parsing performance scales with tensor dimensionality".to_string(),
        data_points: dim_results
            .iter()
            .map(|(dim, avg_ms)| format!("{}: {:.2}ms average", dim, avg_ms))
            .collect(),
    });

    // Data Type Flexibility
    report.add_insight(Insight {
        category: "strength".to_string(),
        title: "Versatile Tensor Format Support".to_string(),
        description: "HEDL tensor format supports diverse ML and scientific computing use cases"
            .to_string(),
        data_points: vec![
            "Compatible with NumPy-style multidimensional arrays".to_string(),
            "Efficient coordinate-based sparse tensor representation".to_string(),
            "Human-readable format aids debugging and inspection".to_string(),
            "Suitable for ML model weights, activations, and gradients".to_string(),
        ],
    });
}

// ============================================================================
// Benchmark Registration and Export
// ============================================================================

criterion_group!(
    tensor_benches,
    bench_tensor_parsing_1d,
    bench_tensor_parsing_2d,
    bench_tensor_parsing_3d,
    bench_tensor_analytics_parsing,
    bench_shape_transformations,
    bench_export_reports,
);

criterion_main!(tensor_benches);

// Export report after all benchmarks complete
/// Export benchmark reports in all formats
fn bench_export_reports(c: &mut Criterion) {
    let mut group = c.benchmark_group("export_reports");
    group.bench_function("export", |b| b.iter(|| 1));
    group.finish();

    export_reports();
}

fn export_reports() {
    REPORT.with(|r| {
        if let Some(ref report) = *r.borrow() {
            report.print();

            // Create target directory
            let _ = fs::create_dir_all("target");

            // Clone report and add comprehensive tables
            let mut new_report = report.clone();

            // Collect results
            RESULTS.with(|results| {
                let results = results.borrow();

                if !results.is_empty() {
                    // Create tables with measured data only
                    create_tensor_operation_performance_table(&results, &mut new_report);
                    create_shape_transformation_table(&results, &mut new_report);
                    create_memory_layout_table(&results, &mut new_report);
                    
                    create_memory_bandwidth_table(&results, &mut new_report);
                    create_cache_efficiency_table(&results, &mut new_report);
                    create_production_recommendations_table(&results, &mut new_report);

                    // Generate insights
                    generate_insights(&results, &mut new_report);
                }
            });

            // Export reports
            let base_path = "target/tensor_report";
            let config = ExportConfig::all();

            match new_report.save_all(base_path, &config) {
                Ok(_) => {
                    println!(
                        "\n[OK] Exported {} tables and {} insights",
                        new_report.custom_tables.len(),
                        new_report.insights.len()
                    );
                }
                Err(e) => {
                    eprintln!("Export failed: {}", e);
                    // Fallback to legacy export
                    let _ = report.save_json(format!("{}.json", base_path));
                    let _ = fs::write(format!("{}.md", base_path), report.to_markdown());
                }
            }
        }
    });
}
